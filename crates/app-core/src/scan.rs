use crate::ports::{LibraryRepository, SourceRepository, TaskRepository};
use anyhow::{anyhow, Result};
use media_io::{candidate_kind_to_source_kind, discover_media_files};
use serde::Serialize;
use shared_model::{LibraryId, SourceId, SourceRecord, TaskId, TaskKind, TaskRecord, TaskState};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanRunSummary {
    pub library_id: String,
    pub discovered: u64,
    pub inserted_or_updated: u64,
    pub task_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanStatsSummary {
    pub library_id: String,
    pub source_count: u64,
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

    library_repository.upsert(&shared_model::LibraryRecord {
        id: library_id.clone(),
        root_path: root_path.display().to_string(),
        library_type: "filesystem".to_string(),
        scan_mode: "full".to_string(),
        created_at: now.clone(),
        updated_at: now,
    })?;

    Ok(library_id)
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

    let task_id = TaskId(format!("task_scan_{:016x}", stable_hash(&library.id.0)));
    task_repository.upsert(&TaskRecord {
        id: task_id.clone(),
        task_type: TaskKind::Scan,
        state: TaskState::Running,
        current: 0,
        total: None,
        message: Some("scan started".to_string()),
        error_code: None,
        error_message: None,
        started_at: Some(now_string()),
        finished_at: None,
    })?;

    let discovered = discover_media_files(Path::new(&library.root_path))?;
    let total = discovered.len() as u64;

    for (index, item) in discovered.iter().enumerate() {
        let source_id = SourceId(format!(
            "source_{:016x}",
            stable_hash(&format!("{}::{}", library.id.0, item.normalized_path))
        ));

        source_repository.upsert(&SourceRecord {
            id: source_id,
            library_id: library.id.clone(),
            normalized_path: item.normalized_path.clone(),
            file_name: item.file_name.clone(),
            ext: item.extension.clone(),
            kind: candidate_kind_to_source_kind(&item.kind),
            size: item.size,
            mtime_ms: item.mtime_ms,
            fingerprint: None,
            exists: true,
            last_seen_at: now_string(),
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
            started_at: Some(now_string()),
            finished_at: None,
        })?;
    }

    task_repository.upsert(&TaskRecord {
        id: task_id.clone(),
        task_type: TaskKind::Scan,
        state: TaskState::Completed,
        current: total,
        total: Some(total),
        message: Some("scan completed".to_string()),
        error_code: None,
        error_message: None,
        started_at: Some(now_string()),
        finished_at: Some(now_string()),
    })?;

    Ok(ScanRunSummary {
        library_id: library.id.0.clone(),
        discovered: total,
        inserted_or_updated: total,
        task_id: task_id.0,
    })
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
    let exists = library_repository.exists(library_id)?;
    if !exists {
        return Err(anyhow!("library not found: {}", library_id.0));
    }

    Ok(ScanStatsSummary {
        library_id: library_id.0.clone(),
        source_count: source_repository.count_by_library(library_id)?,
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

#[cfg(test)]
mod tests {
    use super::{register_library, run_scan, scan_stats};
    use crate::ports::{LibraryRepository, SourceRepository, TaskRepository};
    use shared_model::{LibraryId, LibraryRecord, SourceId, SourceRecord, TaskId, TaskRecord};
    use std::collections::HashMap;
    use std::path::Path;
    use tempfile::tempdir;

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
    }

    impl TaskRepository for MemoryRepos {
        fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool> {
            Ok(self.tasks.contains_key(&task_id.0))
        }
        fn upsert(&self, _task: &TaskRecord) -> anyhow::Result<()> {
            unreachable!()
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
        assert_eq!(stats.source_count, 2);
    }
}
