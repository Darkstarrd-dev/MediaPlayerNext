use crate::ports::{LibraryRepository, SourceRepository, TaskRepository};
use anyhow::{anyhow, Result};
use media_io::{candidate_kind_to_source_kind, discover_media_files};
use serde::Serialize;
use shared_model::{
    LibraryId, LibraryRecord, SourceId, SourceRecord, TaskId, TaskKind, TaskRecord, TaskState,
};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRunSummary {
    pub library_id: String,
    pub discovered: u64,
    pub inserted_or_updated: u64,
    pub skipped_unchanged: u64,
    pub tombstoned: u64,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatsSummary {
    pub library_id: String,
    pub source_count: u64,
    pub active_source_count: u64,
    pub missing_source_count: u64,
}

pub fn register_library<L: LibraryRepository>(
    library_repository: &L,
    root_path: &Path,
) -> Result<LibraryId> {
    if !root_path.is_dir() {
        return Err(anyhow!(
            "library root does not exist: {}",
            root_path.display()
        ));
    }

    let library_id = LibraryId(format!(
        "library_{:016x}",
        stable_hash(&root_path.display().to_string())
    ));
    let now = now_string();

    library_repository.upsert(&LibraryRecord {
        id: library_id.clone(),
        root_path: root_path.display().to_string(),
        library_type: "filesystem".to_string(),
        scan_mode: "full".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })?;

    Ok(library_id)
}

pub fn scan_task_id_for_library(library_id: &LibraryId) -> TaskId {
    TaskId(format!("task_scan_{:016x}", stable_hash(&library_id.0)))
}

pub fn run_scan<L, S, T>(
    library_repository: &L,
    source_repository: &S,
    task_repository: &T,
    library_id: &LibraryId,
) -> Result<ScanRunSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    T: TaskRepository,
{
    let library = library_repository
        .get(library_id)?
        .ok_or_else(|| anyhow!("library not found: {}", library_id.0))?;

    let task_id = scan_task_id_for_library(&library.id);
    let started_at = now_string();

    task_repository.upsert(&TaskRecord {
        id: task_id.clone(),
        task_type: TaskKind::Scan,
        state: TaskState::Running,
        current: 0,
        total: None,
        message: Some("scan started".to_string()),
        error_code: None,
        error_message: None,
        started_at: Some(started_at.clone()),
        finished_at: None,
    })?;

    match run_scan_inner(
        source_repository,
        task_repository,
        &library,
        &task_id,
        &started_at,
    ) {
        Ok(summary) => Ok(summary),
        Err(error) => {
            task_repository.upsert(&TaskRecord {
                id: task_id,
                task_type: TaskKind::Scan,
                state: TaskState::Failed,
                current: 0,
                total: None,
                message: Some("scan failed".to_string()),
                error_code: Some("IO_ERROR".to_string()),
                error_message: Some(error.to_string()),
                started_at: Some(started_at),
                finished_at: Some(now_string()),
            })?;

            Err(error)
        }
    }
}

pub fn resume_scan<L, S, T>(
    library_repository: &L,
    source_repository: &S,
    task_repository: &T,
    library_id: &LibraryId,
) -> Result<ScanRunSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    T: TaskRepository,
{
    run_scan(
        library_repository,
        source_repository,
        task_repository,
        library_id,
    )
}

pub fn scan_stats<L, S>(
    library_repository: &L,
    source_repository: &S,
    library_id: &LibraryId,
) -> Result<ScanStatsSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
{
    if !library_repository.exists(library_id)? {
        return Err(anyhow!("library not found: {}", library_id.0));
    }

    let records = source_repository.list_by_library(library_id)?;
    let active_source_count = records.iter().filter(|item| item.exists).count() as u64;
    let missing_source_count = records.iter().filter(|item| !item.exists).count() as u64;

    Ok(ScanStatsSummary {
        library_id: library_id.0.clone(),
        source_count: source_repository.count_by_library(library_id)?,
        active_source_count,
        missing_source_count,
    })
}

pub fn scan_snapshot<L, S>(
    library_repository: &L,
    source_repository: &S,
    library_id: &LibraryId,
) -> Result<Vec<SourceRecord>>
where
    L: LibraryRepository,
    S: SourceRepository,
{
    if !library_repository.exists(library_id)? {
        return Err(anyhow!("library not found: {}", library_id.0));
    }

    source_repository.list_by_library(library_id)
}

fn run_scan_inner<S, T>(
    source_repository: &S,
    task_repository: &T,
    library: &LibraryRecord,
    task_id: &TaskId,
    started_at: &str,
) -> Result<ScanRunSummary>
where
    S: SourceRepository,
    T: TaskRepository,
{
    let existing_sources = source_repository.list_by_library(&library.id)?;
    let existing_by_path: HashMap<String, SourceRecord> = existing_sources
        .into_iter()
        .map(|item| (item.normalized_path.clone(), item))
        .collect();

    let discovered = discover_media_files(Path::new(&library.root_path))?;
    let total = discovered.len() as u64;
    let scan_time = now_string();
    let mut inserted_or_updated = 0_u64;
    let mut skipped_unchanged = 0_u64;
    let mut seen_paths = HashSet::new();

    for (index, item) in discovered.iter().enumerate() {
        seen_paths.insert(item.normalized_path.clone());
        let source_id = SourceId(format!(
            "source_{:016x}",
            stable_hash(&format!("{}::{}", library.id.0, item.normalized_path))
        ));
        let fingerprint = quick_fingerprint(&item.normalized_path, item.size, item.mtime_ms);

        let unchanged = existing_by_path
            .get(&item.normalized_path)
            .is_some_and(|existing| {
                existing.size == item.size
                    && existing.mtime_ms == item.mtime_ms
                    && existing.exists
                    && existing.fingerprint.as_deref() == Some(fingerprint.as_str())
            });

        if unchanged {
            skipped_unchanged += 1;
        } else {
            inserted_or_updated += 1;
        }

        source_repository.upsert(&SourceRecord {
            id: source_id,
            library_id: library.id.clone(),
            normalized_path: item.normalized_path.clone(),
            file_name: item.file_name.clone(),
            ext: item.extension.clone(),
            kind: candidate_kind_to_source_kind(&item.kind),
            size: item.size,
            mtime_ms: item.mtime_ms,
            fingerprint: Some(fingerprint),
            exists: true,
            last_seen_at: scan_time.clone(),
        })?;

        task_repository.upsert(&TaskRecord {
            id: task_id.clone(),
            task_type: TaskKind::Scan,
            state: TaskState::Running,
            current: (index + 1) as u64,
            total: Some(total),
            message: Some(format!("scanned {} items", index + 1)),
            error_code: None,
            error_message: None,
            started_at: Some(started_at.to_string()),
            finished_at: None,
        })?;
    }

    let mut tombstoned = 0_u64;
    for existing in existing_by_path.values() {
        if seen_paths.contains(&existing.normalized_path) || !existing.exists {
            continue;
        }

        let mut missing_record = existing.clone();
        missing_record.exists = false;
        missing_record.last_seen_at = scan_time.clone();
        source_repository.upsert(&missing_record)?;
        tombstoned += 1;
    }

    task_repository.upsert(&TaskRecord {
        id: task_id.clone(),
        task_type: TaskKind::Scan,
        state: TaskState::Completed,
        current: total,
        total: Some(total),
        message: Some(format!(
            "scan completed: inserted_or_updated={inserted_or_updated}, skipped_unchanged={skipped_unchanged}, tombstoned={tombstoned}"
        )),
        error_code: None,
        error_message: None,
        started_at: Some(started_at.to_string()),
        finished_at: Some(now_string()),
    })?;

    Ok(ScanRunSummary {
        library_id: library.id.0.clone(),
        discovered: total,
        inserted_or_updated,
        skipped_unchanged,
        tombstoned,
        task_id: task_id.0.clone(),
    })
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn now_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.to_string()
}

fn quick_fingerprint(normalized_path: &str, size: i64, mtime_ms: i64) -> String {
    format!(
        "{:016x}",
        stable_hash(&format!("{normalized_path}:{size}:{mtime_ms}"))
    )
}

#[cfg(test)]
mod tests {
    use super::{register_library, resume_scan, run_scan, scan_snapshot, scan_stats, stable_hash};
    use crate::ports::{LibraryRepository, SourceRepository, TaskRepository};
    use serde::Deserialize;
    use shared_model::{
        LibraryId, LibraryRecord, SourceId, SourceRecord, TaskId, TaskRecord, TaskState,
    };
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    #[serde(rename_all = "camelCase")]
    struct SnapshotExpectation {
        file_name: String,
        ext: String,
        kind: String,
        exists: bool,
    }

    #[derive(Default)]
    struct MemoryRepos {
        libraries: HashMap<String, LibraryRecord>,
        sources: HashMap<String, SourceRecord>,
        tasks: HashMap<String, TaskRecord>,
    }

    impl LibraryRepository for MemoryRepos {
        fn exists(&self, library_id: &LibraryId) -> anyhow::Result<bool> {
            Ok(self.libraries.contains_key(&library_id.0))
        }

        fn upsert(&self, _library: &LibraryRecord) -> anyhow::Result<()> {
            unreachable!()
        }

        fn get(&self, library_id: &LibraryId) -> anyhow::Result<Option<LibraryRecord>> {
            Ok(self.libraries.get(&library_id.0).cloned())
        }
    }

    impl SourceRepository for MemoryRepos {
        fn exists(&self, source_id: &SourceId) -> anyhow::Result<bool> {
            Ok(self.sources.contains_key(&source_id.0))
        }

        fn upsert(&self, _source: &SourceRecord) -> anyhow::Result<()> {
            unreachable!()
        }

        fn get(&self, source_id: &SourceId) -> anyhow::Result<Option<SourceRecord>> {
            Ok(self.sources.get(&source_id.0).cloned())
        }

        fn count(&self) -> anyhow::Result<u64> {
            Ok(self.sources.len() as u64)
        }

        fn count_by_library(&self, library_id: &LibraryId) -> anyhow::Result<u64> {
            Ok(self
                .sources
                .values()
                .filter(|item| item.library_id == *library_id)
                .count() as u64)
        }

        fn list_by_library(&self, library_id: &LibraryId) -> anyhow::Result<Vec<SourceRecord>> {
            let mut items: Vec<_> = self
                .sources
                .values()
                .filter(|item| item.library_id == *library_id)
                .cloned()
                .collect();
            items.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
            Ok(items)
        }
    }

    impl TaskRepository for MemoryRepos {
        fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool> {
            Ok(self.tasks.contains_key(&task_id.0))
        }

        fn upsert(&self, _task: &TaskRecord) -> anyhow::Result<()> {
            unreachable!()
        }

        fn get(&self, task_id: &TaskId) -> anyhow::Result<Option<TaskRecord>> {
            Ok(self.tasks.get(&task_id.0).cloned())
        }
    }

    struct MutableMemoryRepos {
        inner: std::sync::Mutex<MemoryRepos>,
    }

    impl Default for MutableMemoryRepos {
        fn default() -> Self {
            Self {
                inner: std::sync::Mutex::new(MemoryRepos::default()),
            }
        }
    }

    impl LibraryRepository for MutableMemoryRepos {
        fn exists(&self, library_id: &LibraryId) -> anyhow::Result<bool> {
            Ok(self
                .inner
                .lock()
                .expect("lock")
                .libraries
                .contains_key(&library_id.0))
        }

        fn upsert(&self, library: &LibraryRecord) -> anyhow::Result<()> {
            self.inner
                .lock()
                .expect("lock")
                .libraries
                .insert(library.id.0.clone(), library.clone());
            Ok(())
        }

        fn get(&self, library_id: &LibraryId) -> anyhow::Result<Option<LibraryRecord>> {
            Ok(self
                .inner
                .lock()
                .expect("lock")
                .libraries
                .get(&library_id.0)
                .cloned())
        }
    }

    impl SourceRepository for MutableMemoryRepos {
        fn exists(&self, source_id: &SourceId) -> anyhow::Result<bool> {
            Ok(self
                .inner
                .lock()
                .expect("lock")
                .sources
                .contains_key(&source_id.0))
        }

        fn upsert(&self, source: &SourceRecord) -> anyhow::Result<()> {
            self.inner
                .lock()
                .expect("lock")
                .sources
                .insert(source.id.0.clone(), source.clone());
            Ok(())
        }

        fn get(&self, source_id: &SourceId) -> anyhow::Result<Option<SourceRecord>> {
            Ok(self
                .inner
                .lock()
                .expect("lock")
                .sources
                .get(&source_id.0)
                .cloned())
        }

        fn count(&self) -> anyhow::Result<u64> {
            Ok(self.inner.lock().expect("lock").sources.len() as u64)
        }

        fn count_by_library(&self, library_id: &LibraryId) -> anyhow::Result<u64> {
            Ok(self
                .inner
                .lock()
                .expect("lock")
                .sources
                .values()
                .filter(|item| item.library_id == *library_id)
                .count() as u64)
        }

        fn list_by_library(&self, library_id: &LibraryId) -> anyhow::Result<Vec<SourceRecord>> {
            let mut items: Vec<_> = self
                .inner
                .lock()
                .expect("lock")
                .sources
                .values()
                .filter(|item| item.library_id == *library_id)
                .cloned()
                .collect();
            items.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
            Ok(items)
        }
    }

    impl TaskRepository for MutableMemoryRepos {
        fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool> {
            Ok(self
                .inner
                .lock()
                .expect("lock")
                .tasks
                .contains_key(&task_id.0))
        }

        fn upsert(&self, task: &TaskRecord) -> anyhow::Result<()> {
            self.inner
                .lock()
                .expect("lock")
                .tasks
                .insert(task.id.0.clone(), task.clone());
            Ok(())
        }

        fn get(&self, task_id: &TaskId) -> anyhow::Result<Option<TaskRecord>> {
            Ok(self
                .inner
                .lock()
                .expect("lock")
                .tasks
                .get(&task_id.0)
                .cloned())
        }
    }

    #[test]
    fn registers_library_and_scans_files() {
        let repos = MutableMemoryRepos::default();
        let temp = tempdir().expect("tempdir should be created");
        std::fs::write(temp.path().join("cover.png"), b"png").expect("fixture should be written");
        std::fs::create_dir_all(temp.path().join("nested")).expect("nested dir");
        std::fs::write(temp.path().join("nested").join("chapter.cbz"), b"zip")
            .expect("fixture should be written");

        let library_id =
            register_library(&repos, Path::new(temp.path())).expect("library should register");
        let summary = run_scan(&repos, &repos, &repos, &library_id).expect("scan should succeed");
        let stats = scan_stats(&repos, &repos, &library_id).expect("stats should succeed");

        assert_eq!(summary.discovered, 2);
        assert_eq!(summary.inserted_or_updated, 2);
        assert_eq!(summary.skipped_unchanged, 0);
        assert_eq!(summary.tombstoned, 0);
        assert_eq!(stats.source_count, 2);
        assert_eq!(stats.active_source_count, 2);
        assert_eq!(stats.missing_source_count, 0);
    }

    #[test]
    fn rescans_skip_unchanged_and_tombstone_missing_files() {
        let repos = MutableMemoryRepos::default();
        let temp = tempdir().expect("tempdir should be created");
        let cover_path = temp.path().join("cover.png");
        let nested = temp.path().join("nested");
        std::fs::create_dir_all(&nested).expect("nested dir");
        let chapter_path = nested.join("chapter.cbz");
        std::fs::write(&cover_path, b"png").expect("fixture should be written");
        std::fs::write(&chapter_path, b"zip").expect("fixture should be written");

        let library_id =
            register_library(&repos, Path::new(temp.path())).expect("library should register");
        let first_summary =
            run_scan(&repos, &repos, &repos, &library_id).expect("first scan should succeed");

        std::fs::remove_file(chapter_path).expect("fixture should be removed");
        let second_summary =
            run_scan(&repos, &repos, &repos, &library_id).expect("second scan should succeed");
        let snapshot = scan_snapshot(&repos, &repos, &library_id).expect("snapshot should succeed");
        let stats = scan_stats(&repos, &repos, &library_id).expect("stats should succeed");

        assert_eq!(first_summary.inserted_or_updated, 2);
        assert_eq!(second_summary.skipped_unchanged, 1);
        assert_eq!(second_summary.tombstoned, 1);
        assert_eq!(stats.source_count, 2);
        assert_eq!(stats.active_source_count, 1);
        assert_eq!(stats.missing_source_count, 1);
        assert!(snapshot
            .iter()
            .any(|item| item.file_name == "chapter.cbz" && !item.exists));
    }

    #[test]
    fn records_failed_task_when_scan_root_disappears() {
        let repos = MutableMemoryRepos::default();
        let temp = tempdir().expect("tempdir should be created");

        let library_id =
            register_library(&repos, Path::new(temp.path())).expect("library should register");
        std::fs::remove_dir_all(temp.path()).expect("tempdir should be removed before scan");

        let error = run_scan(&repos, &repos, &repos, &library_id).expect_err("scan should fail");
        assert!(!error.to_string().is_empty());

        let task_id = TaskId(format!("task_scan_{:016x}", stable_hash(&library_id.0)));
        let task = TaskRepository::get(&repos, &task_id)
            .expect("task should be queryable")
            .expect("failed task should be persisted");

        assert_eq!(task.state, TaskState::Failed);
        assert_eq!(task.error_code.as_deref(), Some("IO_ERROR"));
    }

    #[test]
    fn resume_scan_reuses_existing_library_flow() {
        let repos = MutableMemoryRepos::default();
        let temp = tempdir().expect("tempdir should be created");
        std::fs::write(temp.path().join("cover.png"), b"png").expect("fixture should be written");

        let library_id =
            register_library(&repos, Path::new(temp.path())).expect("library should register");
        let first =
            run_scan(&repos, &repos, &repos, &library_id).expect("first scan should succeed");
        let resumed =
            resume_scan(&repos, &repos, &repos, &library_id).expect("resume scan should succeed");

        assert_eq!(first.discovered, 1);
        assert_eq!(resumed.skipped_unchanged, 1);
    }

    #[test]
    fn matches_workspace_scan_smoke_snapshot_fixture() {
        let repos = MutableMemoryRepos::default();
        let fixture_root = workspace_root().join("docs/fixtures/small-fixture/scan-smoke");
        let expectation_path =
            workspace_root().join("docs/fixtures/small-fixture/scan-smoke.expected.json");

        let library_id =
            register_library(&repos, &fixture_root).expect("workspace fixture should register");
        run_scan(&repos, &repos, &repos, &library_id)
            .expect("workspace fixture scan should succeed");
        let snapshot = scan_snapshot(&repos, &repos, &library_id).expect("snapshot should succeed");

        let reduced: Vec<SnapshotExpectation> = snapshot
            .into_iter()
            .map(|item| SnapshotExpectation {
                file_name: item.file_name,
                ext: item.ext,
                kind: format!("{:?}", item.kind),
                exists: item.exists,
            })
            .collect();

        let expected: Vec<SnapshotExpectation> = serde_json::from_str(
            &fs::read_to_string(expectation_path).expect("snapshot fixture should be readable"),
        )
        .expect("snapshot fixture should deserialize");

        assert_eq!(reduced, expected);
    }

    fn workspace_root() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf()
    }
}
