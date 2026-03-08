use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository, TaskRepository,
};
use anyhow::{anyhow, Result};
use media_io::archive::{build_zip_index, read_zip_entry_bytes};
use media_io::normalize::{
    normalize_archive_to_zip, normalized_archive_layout, ArchiveExtractor, SevenZipExtractor,
};
use media_io::{is_normalizable_archive_extension, is_primary_archive_extension, normalize_path};
use serde::Serialize;
use shared_model::{
    ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, LibraryId, LogContext, SourceId,
    SourceKind, TaskId, TaskKind, TaskRecord, TaskState,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveIndexSummary {
    pub library_id: String,
    pub indexed_archives: u64,
    pub indexed_entries: u64,
    pub skipped_non_primary_archives: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveSnapshot {
    pub source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive: Option<ArchiveRecord>,
    pub entries: Vec<ArchiveEntryRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEntryReadSummary {
    pub source_id: String,
    pub entry_path: String,
    pub byte_count: usize,
    pub preview_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedArchiveEntryLocation {
    pub archive_entry_id: String,
    pub archive_id: String,
    pub source_id: String,
    pub archive_path: String,
    pub entry_path: String,
    pub media_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveNormalizeSummary {
    pub task_id: String,
    pub source_id: String,
    pub archive_id: String,
    pub normalized_zip_path: String,
    pub extracted_file_count: usize,
    pub indexed_entries: u64,
    pub archive_status: String,
}

pub fn index_library_archives<L, S, A, E>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    library_id: &LibraryId,
) -> Result<ArchiveIndexSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
{
    let library = library_repository
        .get(library_id)?
        .ok_or_else(|| anyhow!("library not found: {}", library_id.0))?;

    let mut indexed_archives = 0_u64;
    let mut indexed_entries = 0_u64;
    let mut skipped_non_primary_archives = 0_u64;

    for source in source_repository.list_by_library(library_id)? {
        if !source.exists || source.kind != SourceKind::Archive {
            continue;
        }

        let existing_archive = archive_repository.get_by_source(&source.id)?;
        let Some(archive_path) =
            archive_index_path_for_source(&library.root_path, &source, existing_archive.as_ref())
        else {
            skipped_non_primary_archives += 1;
            continue;
        };

        let zip_index = build_zip_index(&archive_path)?;
        let archive_id = existing_archive
            .as_ref()
            .map(|item| item.id.clone())
            .unwrap_or_else(|| archive_id_from_source(&source.id));
        let entry_records = zip_index
            .pages
            .iter()
            .map(|page| ArchiveEntryRecord {
                id: archive_entry_id_from_path(&archive_id, &page.entry_path),
                archive_id: archive_id.clone(),
                entry_path: page.entry_path.clone(),
                entry_name: page.entry_name.clone(),
                page_index: page.page_index as i64,
                media_kind: format!("{:?}", page.media_kind).to_ascii_lowercase(),
                width: None,
                height: None,
                compressed_size: page.compressed_size,
                uncompressed_size: page.uncompressed_size,
                crc32: page.crc32,
            })
            .collect::<Vec<_>>();
        let cover_entry_id = entry_records.first().map(|item| item.id.clone());

        let is_empty_archive = entry_records.is_empty();

        archive_repository.upsert(&ArchiveRecord {
            id: archive_id.clone(),
            source_id: source.id.clone(),
            archive_type: source.ext.clone(),
            normalized_zip_path: Some(zip_index.normalized_zip_path),
            page_count: Some(entry_records.len() as i64),
            cover_entry_id,
            status: if is_primary_archive_extension(&source.ext) {
                if is_empty_archive {
                    "empty".to_string()
                } else {
                    "indexed".to_string()
                }
            } else {
                "normalized".to_string()
            },
        })?;
        archive_entry_repository.replace_for_archive(&archive_id, &entry_records)?;

        indexed_archives += 1;
        indexed_entries += entry_records.len() as u64;
    }

    Ok(ArchiveIndexSummary {
        library_id: library_id.0.clone(),
        indexed_archives,
        indexed_entries,
        skipped_non_primary_archives,
    })
}

pub fn archive_snapshot<A, E>(
    archive_repository: &A,
    archive_entry_repository: &E,
    source_id: &SourceId,
) -> Result<ArchiveSnapshot>
where
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
{
    let archive = archive_repository.get_by_source(source_id)?;
    let entries = match &archive {
        Some(archive_record) => archive_entry_repository.list_by_archive(&archive_record.id)?,
        None => Vec::new(),
    };

    Ok(ArchiveSnapshot {
        source_id: source_id.0.clone(),
        archive,
        entries,
    })
}

pub fn read_archive_entry(source_path: &Path, entry_path: &str) -> Result<Vec<u8>> {
    read_zip_entry_bytes(source_path, entry_path)
}

pub fn resolve_archive_entry_location<L, S, A, E>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    archive_entry_id: &ArchiveEntryId,
) -> Result<ResolvedArchiveEntryLocation>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
{
    let entry = archive_entry_repository
        .get(archive_entry_id)?
        .ok_or_else(|| anyhow!("archive entry not found: {}", archive_entry_id.0))?;
    let archive = archive_repository
        .get(&entry.archive_id)?
        .ok_or_else(|| anyhow!("archive not found: {}", entry.archive_id.0))?;
    let source = source_repository
        .get(&archive.source_id)?
        .ok_or_else(|| anyhow!("archive source not found: {}", archive.source_id.0))?;
    let library = library_repository
        .get(&source.library_id)?
        .ok_or_else(|| anyhow!("library not found: {}", source.library_id.0))?;
    let archive_path = archive_index_path_for_source(&library.root_path, &source, Some(&archive))
        .ok_or_else(|| anyhow!("archive path not available: {}", archive.id.0))?;

    Ok(ResolvedArchiveEntryLocation {
        archive_entry_id: entry.id.0,
        archive_id: archive.id.0,
        source_id: source.id.0,
        archive_path: archive_path.display().to_string(),
        entry_path: entry.entry_path,
        media_kind: entry.media_kind,
    })
}

pub fn read_archive_entry_by_source<L, S>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &impl ArchiveRepository,
    source_id: &SourceId,
    entry_path: &str,
) -> Result<ArchiveEntryReadSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
{
    let source = source_repository
        .get(source_id)?
        .ok_or_else(|| anyhow!("archive source not found: {}", source_id.0))?;
    let library = library_repository
        .get(&source.library_id)?
        .ok_or_else(|| anyhow!("library not found: {}", source.library_id.0))?;
    let existing_archive = archive_repository.get_by_source(source_id)?;
    let archive_path =
        archive_index_path_for_source(&library.root_path, &source, existing_archive.as_ref())
            .ok_or_else(|| anyhow!("archive path not available: {}", source_id.0))?;
    let bytes = read_archive_entry(&archive_path, entry_path)?;

    Ok(ArchiveEntryReadSummary {
        source_id: source_id.0.clone(),
        entry_path: entry_path.to_string(),
        byte_count: bytes.len(),
        preview_hex: bytes
            .iter()
            .take(16)
            .map(|value| format!("{value:02x}"))
            .collect::<String>(),
    })
}

#[allow(clippy::too_many_arguments)]
pub fn normalize_archive_source<L, S, A, E, T>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    task_repository: &T,
    sevenz_path: &Path,
    normalize_root: &Path,
    source_id: &SourceId,
) -> Result<ArchiveNormalizeSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    T: TaskRepository,
{
    let task_id = normalize_task_id(source_id);
    let extractor = SevenZipExtractor::new(sevenz_path).with_context(LogContext {
        task_id: Some(task_id.clone()),
        source_id: Some(source_id.clone()),
        ..LogContext::default()
    });
    normalize_archive_source_with_extractor(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        task_repository,
        normalize_root,
        source_id,
        &task_id,
        &extractor,
    )
}

pub fn normalize_archive_status<T>(task_repository: &T, task_id: &TaskId) -> Result<TaskRecord>
where
    T: TaskRepository,
{
    task_repository
        .get(task_id)?
        .ok_or_else(|| anyhow!("task not found: {}", task_id.0))
}

#[allow(clippy::too_many_arguments)]
fn normalize_archive_source_with_extractor<L, S, A, E, T, X>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    task_repository: &T,
    normalize_root: &Path,
    source_id: &SourceId,
    task_id: &TaskId,
    extractor: &X,
) -> Result<ArchiveNormalizeSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    T: TaskRepository,
    X: ArchiveExtractor,
{
    let source = source_repository
        .get(source_id)?
        .ok_or_else(|| anyhow!("archive source not found: {}", source_id.0))?;
    let library = library_repository
        .get(&source.library_id)?
        .ok_or_else(|| anyhow!("library not found: {}", source.library_id.0))?;

    if source.kind != SourceKind::Archive {
        return Err(anyhow!("source is not archive: {}", source_id.0));
    }
    if !is_normalizable_archive_extension(&source.ext) {
        return Err(anyhow!(
            "archive normalization only supports rar/7z sources: {}",
            source.ext
        ));
    }

    let revision = source
        .fingerprint
        .clone()
        .unwrap_or_else(|| format!("{}:{}", source.mtime_ms, source.size));
    let layout = normalized_archive_layout(normalize_root, &source.id, &revision);
    let archive_id = archive_repository
        .get_by_source(&source.id)?
        .map(|item| item.id)
        .unwrap_or_else(|| archive_id_from_source(&source.id));
    let archive_path = archive_path_from_source(&library.root_path, &source.normalized_path);
    task_repository.upsert(&TaskRecord {
        id: task_id.clone(),
        task_type: TaskKind::Normalize,
        state: TaskState::Queued,
        current: 0,
        total: Some(4),
        message: Some("queued".to_string()),
        error_code: None,
        error_message: None,
        started_at: Some(now_string()),
        finished_at: None,
    })?;
    archive_repository.upsert(&ArchiveRecord {
        id: archive_id.clone(),
        source_id: source.id.clone(),
        archive_type: source.ext.clone(),
        normalized_zip_path: None,
        page_count: None,
        cover_entry_id: None,
        status: "queued".to_string(),
    })?;

    let normalization = match (|| -> Result<_> {
        task_repository.upsert(&TaskRecord {
            id: task_id.clone(),
            task_type: TaskKind::Normalize,
            state: TaskState::Running,
            current: 1,
            total: Some(4),
            message: Some("extracting".to_string()),
            error_code: None,
            error_message: None,
            started_at: Some(now_string()),
            finished_at: None,
        })?;
        archive_repository.upsert(&ArchiveRecord {
            id: archive_id.clone(),
            source_id: source.id.clone(),
            archive_type: source.ext.clone(),
            normalized_zip_path: None,
            page_count: None,
            cover_entry_id: None,
            status: "extracting".to_string(),
        })?;

        let normalization = normalize_archive_to_zip(extractor, &archive_path, &layout)?;

        task_repository.upsert(&TaskRecord {
            id: task_id.clone(),
            task_type: TaskKind::Normalize,
            state: TaskState::Running,
            current: 3,
            total: Some(4),
            message: Some("verifying".to_string()),
            error_code: None,
            error_message: None,
            started_at: Some(now_string()),
            finished_at: None,
        })?;
        archive_repository.upsert(&ArchiveRecord {
            id: archive_id.clone(),
            source_id: source.id.clone(),
            archive_type: source.ext.clone(),
            normalized_zip_path: Some(normalize_path(&normalization.normalized_zip_path)),
            page_count: None,
            cover_entry_id: None,
            status: "verifying".to_string(),
        })?;

        let zip_index = build_zip_index(&normalization.normalized_zip_path)?;
        let entry_records = zip_index
            .pages
            .iter()
            .map(|page| ArchiveEntryRecord {
                id: archive_entry_id_from_path(&archive_id, &page.entry_path),
                archive_id: archive_id.clone(),
                entry_path: page.entry_path.clone(),
                entry_name: page.entry_name.clone(),
                page_index: page.page_index as i64,
                media_kind: format!("{:?}", page.media_kind).to_ascii_lowercase(),
                width: None,
                height: None,
                compressed_size: page.compressed_size,
                uncompressed_size: page.uncompressed_size,
                crc32: page.crc32,
            })
            .collect::<Vec<_>>();
        let cover_entry_id = entry_records.first().map(|item| item.id.clone());
        archive_entry_repository.replace_for_archive(&archive_id, &entry_records)?;
        archive_repository.upsert(&ArchiveRecord {
            id: archive_id.clone(),
            source_id: source.id.clone(),
            archive_type: source.ext.clone(),
            normalized_zip_path: Some(zip_index.normalized_zip_path),
            page_count: Some(entry_records.len() as i64),
            cover_entry_id,
            status: "normalized".to_string(),
        })?;
        task_repository.upsert(&TaskRecord {
            id: task_id.clone(),
            task_type: TaskKind::Normalize,
            state: TaskState::Completed,
            current: 4,
            total: Some(4),
            message: Some("normalized".to_string()),
            error_code: None,
            error_message: None,
            started_at: Some(now_string()),
            finished_at: Some(now_string()),
        })?;

        Ok(ArchiveNormalizeSummary {
            task_id: task_id.0.clone(),
            source_id: source.id.0.clone(),
            archive_id: archive_id.0.clone(),
            normalized_zip_path: normalize_path(&normalization.normalized_zip_path),
            extracted_file_count: normalization.extracted_file_count,
            indexed_entries: entry_records.len() as u64,
            archive_status: "normalized".to_string(),
        })
    })() {
        Ok(summary) => summary,
        Err(error) => {
            task_repository.upsert(&TaskRecord {
                id: task_id.clone(),
                task_type: TaskKind::Normalize,
                state: TaskState::Failed,
                current: 0,
                total: Some(4),
                message: Some("failed".to_string()),
                error_code: Some("normalize_failed".to_string()),
                error_message: Some(error.to_string()),
                started_at: Some(now_string()),
                finished_at: Some(now_string()),
            })?;
            archive_repository.upsert(&ArchiveRecord {
                id: archive_id.clone(),
                source_id: source.id.clone(),
                archive_type: source.ext.clone(),
                normalized_zip_path: None,
                page_count: None,
                cover_entry_id: None,
                status: "failed".to_string(),
            })?;
            return Err(error);
        }
    };

    Ok(normalization)
}

fn archive_path_from_source(library_root: &str, normalized_source_path: &str) -> PathBuf {
    let candidate = PathBuf::from(normalized_source_path);
    if candidate.is_absolute() {
        return candidate;
    }

    Path::new(library_root).join(normalized_source_path)
}

fn archive_index_path_for_source(
    library_root: &str,
    source: &shared_model::SourceRecord,
    archive: Option<&ArchiveRecord>,
) -> Option<PathBuf> {
    if is_primary_archive_extension(&source.ext) {
        return Some(archive_path_from_source(
            library_root,
            &source.normalized_path,
        ));
    }

    archive
        .and_then(|item| item.normalized_zip_path.as_ref())
        .map(PathBuf::from)
}

fn archive_id_from_source(source_id: &SourceId) -> ArchiveId {
    ArchiveId(format!(
        "archive_{:016x}",
        stable_hash(&format!("archive::{}", source_id.0))
    ))
}

fn archive_entry_id_from_path(archive_id: &ArchiveId, entry_path: &str) -> ArchiveEntryId {
    ArchiveEntryId(format!(
        "archive_entry_{:016x}",
        stable_hash(&format!("{}::{entry_path}", archive_id.0))
    ))
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn normalize_task_id(source_id: &SourceId) -> TaskId {
    TaskId(format!(
        "task_norm_{:016x}",
        stable_hash(&format!("normalize::{}::{}", source_id.0, now_string()))
    ))
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
    use super::{
        archive_snapshot, index_library_archives, normalize_archive_source_with_extractor,
        normalize_archive_status, normalize_task_id, read_archive_entry_by_source,
    };
    use crate::ports::{
        ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository,
        TaskRepository,
    };
    use anyhow::anyhow;
    use media_io::normalize::ArchiveExtractor;
    use shared_model::{
        ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, LibraryId, LibraryRecord,
        SourceId, SourceKind, SourceRecord, TaskId, TaskKind, TaskRecord, TaskState,
    };
    use std::collections::HashMap;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::sync::Mutex;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[derive(Default)]
    struct MemoryRepos {
        libraries: Mutex<HashMap<String, LibraryRecord>>,
        sources: Mutex<HashMap<String, SourceRecord>>,
        archives: Mutex<HashMap<String, ArchiveRecord>>,
        archive_entries: Mutex<HashMap<String, Vec<ArchiveEntryRecord>>>,
        tasks: Mutex<HashMap<String, TaskRecord>>,
    }

    struct MockExtractor {
        fail_first: Mutex<bool>,
        files: Vec<(String, Vec<u8>)>,
    }

    impl ArchiveExtractor for MockExtractor {
        fn extract_archive(&self, _archive_path: &Path, output_dir: &Path) -> anyhow::Result<()> {
            let mut fail_first = self.fail_first.lock().expect("lock");
            if *fail_first {
                *fail_first = false;
                return Err(anyhow!("mock normalize failure"));
            }

            for (relative_path, bytes) in &self.files {
                let target_path = output_dir.join(relative_path);
                if let Some(parent) = target_path.parent() {
                    std::fs::create_dir_all(parent).expect("normalize parent should exist");
                }
                std::fs::write(target_path, bytes).expect("normalize output should be written");
            }

            Ok(())
        }
    }

    impl LibraryRepository for MemoryRepos {
        fn exists(&self, library_id: &LibraryId) -> anyhow::Result<bool> {
            Ok(self
                .libraries
                .lock()
                .expect("lock")
                .contains_key(&library_id.0))
        }

        fn upsert(&self, library: &LibraryRecord) -> anyhow::Result<()> {
            self.libraries
                .lock()
                .expect("lock")
                .insert(library.id.0.clone(), library.clone());
            Ok(())
        }

        fn get(&self, library_id: &LibraryId) -> anyhow::Result<Option<LibraryRecord>> {
            Ok(self
                .libraries
                .lock()
                .expect("lock")
                .get(&library_id.0)
                .cloned())
        }
    }

    impl SourceRepository for MemoryRepos {
        fn exists(&self, source_id: &SourceId) -> anyhow::Result<bool> {
            Ok(self
                .sources
                .lock()
                .expect("lock")
                .contains_key(&source_id.0))
        }

        fn upsert(&self, source: &SourceRecord) -> anyhow::Result<()> {
            self.sources
                .lock()
                .expect("lock")
                .insert(source.id.0.clone(), source.clone());
            Ok(())
        }

        fn get(&self, source_id: &SourceId) -> anyhow::Result<Option<SourceRecord>> {
            Ok(self
                .sources
                .lock()
                .expect("lock")
                .get(&source_id.0)
                .cloned())
        }

        fn count(&self) -> anyhow::Result<u64> {
            Ok(self.sources.lock().expect("lock").len() as u64)
        }

        fn count_by_library(&self, library_id: &LibraryId) -> anyhow::Result<u64> {
            Ok(self
                .sources
                .lock()
                .expect("lock")
                .values()
                .filter(|item| item.library_id == *library_id)
                .count() as u64)
        }

        fn list_by_library(&self, library_id: &LibraryId) -> anyhow::Result<Vec<SourceRecord>> {
            let mut items = self
                .sources
                .lock()
                .expect("lock")
                .values()
                .filter(|item| item.library_id == *library_id)
                .cloned()
                .collect::<Vec<_>>();
            items.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
            Ok(items)
        }
    }

    impl ArchiveRepository for MemoryRepos {
        fn exists(&self, archive_id: &ArchiveId) -> anyhow::Result<bool> {
            Ok(self
                .archives
                .lock()
                .expect("lock")
                .contains_key(&archive_id.0))
        }

        fn upsert(&self, archive: &ArchiveRecord) -> anyhow::Result<()> {
            self.archives
                .lock()
                .expect("lock")
                .insert(archive.id.0.clone(), archive.clone());
            Ok(())
        }

        fn get(&self, archive_id: &ArchiveId) -> anyhow::Result<Option<ArchiveRecord>> {
            Ok(self
                .archives
                .lock()
                .expect("lock")
                .get(&archive_id.0)
                .cloned())
        }

        fn get_by_source(&self, source_id: &SourceId) -> anyhow::Result<Option<ArchiveRecord>> {
            Ok(self
                .archives
                .lock()
                .expect("lock")
                .values()
                .find(|item| item.source_id == *source_id)
                .cloned())
        }
    }

    impl ArchiveEntryRepository for MemoryRepos {
        fn replace_for_archive(
            &self,
            archive_id: &ArchiveId,
            entries: &[ArchiveEntryRecord],
        ) -> anyhow::Result<()> {
            self.archive_entries
                .lock()
                .expect("lock")
                .insert(archive_id.0.clone(), entries.to_vec());
            Ok(())
        }

        fn get(
            &self,
            archive_entry_id: &ArchiveEntryId,
        ) -> anyhow::Result<Option<ArchiveEntryRecord>> {
            Ok(self
                .archive_entries
                .lock()
                .expect("lock")
                .values()
                .flat_map(|items| items.iter())
                .find(|item| item.id == *archive_entry_id)
                .cloned())
        }

        fn list_by_archive(
            &self,
            archive_id: &ArchiveId,
        ) -> anyhow::Result<Vec<ArchiveEntryRecord>> {
            Ok(self
                .archive_entries
                .lock()
                .expect("lock")
                .get(&archive_id.0)
                .cloned()
                .unwrap_or_default())
        }
    }

    impl TaskRepository for MemoryRepos {
        fn exists(&self, task_id: &TaskId) -> anyhow::Result<bool> {
            Ok(self.tasks.lock().expect("lock").contains_key(&task_id.0))
        }

        fn upsert(&self, task: &TaskRecord) -> anyhow::Result<()> {
            self.tasks
                .lock()
                .expect("lock")
                .insert(task.id.0.clone(), task.clone());
            Ok(())
        }

        fn get(&self, task_id: &TaskId) -> anyhow::Result<Option<TaskRecord>> {
            Ok(self.tasks.lock().expect("lock").get(&task_id.0).cloned())
        }
    }

    #[test]
    fn indexes_zip_archives_and_persists_entries() {
        let repos = MemoryRepos::default();
        let temp = tempdir().expect("tempdir should be created");
        let zip_path = temp.path().join("chapter.cbz");
        write_test_zip(
            &zip_path,
            &[
                ("001-cover.png", b"cover"),
                ("002-page.png", b"page"),
                ("notes.txt", b"ignore"),
            ],
        );

        let library = LibraryRecord {
            id: LibraryId("library_archive".to_string()),
            root_path: temp.path().display().to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        LibraryRepository::upsert(&repos, &library).expect("library should be stored");
        SourceRepository::upsert(
            &repos,
            &SourceRecord {
                id: SourceId("source_archive".to_string()),
                library_id: library.id.clone(),
                normalized_path: zip_path.display().to_string(),
                file_name: "chapter.cbz".to_string(),
                ext: "cbz".to_string(),
                kind: SourceKind::Archive,
                size: 10,
                mtime_ms: 1,
                fingerprint: None,
                exists: true,
                last_seen_at: "1".to_string(),
            },
        )
        .expect("source should be stored");

        let summary = index_library_archives(&repos, &repos, &repos, &repos, &library.id)
            .expect("archive index should succeed");
        let snapshot = archive_snapshot(&repos, &repos, &SourceId("source_archive".to_string()))
            .expect("archive snapshot should succeed");

        assert_eq!(summary.indexed_archives, 1);
        assert_eq!(summary.indexed_entries, 2);
        assert_eq!(snapshot.entries.len(), 2);
        assert_eq!(snapshot.entries[0].entry_path, "001-cover.png");
        assert_eq!(
            snapshot.archive.expect("archive should exist").status,
            "indexed"
        );
    }

    #[test]
    fn marks_empty_archive_as_empty_status() {
        let repos = MemoryRepos::default();
        let temp = tempdir().expect("tempdir should be created");
        let zip_path = temp.path().join("empty.cbz");
        write_test_zip(&zip_path, &[]);

        let library = LibraryRecord {
            id: LibraryId("library_empty_archive".to_string()),
            root_path: temp.path().display().to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        LibraryRepository::upsert(&repos, &library).expect("library should be stored");
        SourceRepository::upsert(
            &repos,
            &SourceRecord {
                id: SourceId("source_empty_archive".to_string()),
                library_id: library.id.clone(),
                normalized_path: zip_path.display().to_string(),
                file_name: "empty.cbz".to_string(),
                ext: "cbz".to_string(),
                kind: SourceKind::Archive,
                size: 10,
                mtime_ms: 1,
                fingerprint: None,
                exists: true,
                last_seen_at: "1".to_string(),
            },
        )
        .expect("source should be stored");

        let summary = index_library_archives(&repos, &repos, &repos, &repos, &library.id)
            .expect("archive index should succeed");
        let snapshot = archive_snapshot(
            &repos,
            &repos,
            &SourceId("source_empty_archive".to_string()),
        )
        .expect("archive snapshot should succeed");

        assert_eq!(summary.indexed_archives, 1);
        assert_eq!(summary.indexed_entries, 0);
        assert_eq!(snapshot.entries.len(), 0);
        assert_eq!(
            snapshot.archive.expect("archive should exist").status,
            "empty"
        );
    }

    #[test]
    fn reads_archive_entry_bytes_by_source() {
        let repos = MemoryRepos::default();
        let temp = tempdir().expect("tempdir should be created");
        let zip_path = temp.path().join("chapter.cbz");
        write_test_zip(&zip_path, &[("001-cover.png", b"cover-binary")]);

        let library = LibraryRecord {
            id: LibraryId("library_read_entry".to_string()),
            root_path: temp.path().display().to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        LibraryRepository::upsert(&repos, &library).expect("library should be stored");
        SourceRepository::upsert(
            &repos,
            &SourceRecord {
                id: SourceId("source_read_entry".to_string()),
                library_id: library.id.clone(),
                normalized_path: zip_path.display().to_string(),
                file_name: "chapter.cbz".to_string(),
                ext: "cbz".to_string(),
                kind: SourceKind::Archive,
                size: 10,
                mtime_ms: 1,
                fingerprint: None,
                exists: true,
                last_seen_at: "1".to_string(),
            },
        )
        .expect("source should be stored");

        let summary = read_archive_entry_by_source(
            &repos,
            &repos,
            &repos,
            &SourceId("source_read_entry".to_string()),
            "001-cover.png",
        )
        .expect("archive entry should be read");

        assert_eq!(summary.byte_count, b"cover-binary".len());
        assert!(!summary.preview_hex.is_empty());
    }

    #[test]
    fn normalizes_7z_archive_and_indexes_normalized_zip() {
        let repos = MemoryRepos::default();
        let temp = tempdir().expect("tempdir should be created");
        let source_path = temp.path().join("chapter.7z");
        std::fs::write(&source_path, b"fake-7z").expect("source archive should exist");

        let library = LibraryRecord {
            id: LibraryId("library_normalize".to_string()),
            root_path: temp.path().display().to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        let source = SourceRecord {
            id: SourceId("source_normalize".to_string()),
            library_id: library.id.clone(),
            normalized_path: source_path.display().to_string(),
            file_name: "chapter.7z".to_string(),
            ext: "7z".to_string(),
            kind: SourceKind::Archive,
            size: 10,
            mtime_ms: 1,
            fingerprint: Some("fp-normalize".to_string()),
            exists: true,
            last_seen_at: "1".to_string(),
        };
        let extractor = MockExtractor {
            fail_first: Mutex::new(false),
            files: vec![
                ("001-cover.png".to_string(), b"cover".to_vec()),
                ("002-page.png".to_string(), b"page".to_vec()),
            ],
        };

        LibraryRepository::upsert(&repos, &library).expect("library should be stored");
        SourceRepository::upsert(&repos, &source).expect("source should be stored");
        let task_id = normalize_task_id(&source.id);

        let summary = normalize_archive_source_with_extractor(
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            temp.path(),
            &source.id,
            &task_id,
            &extractor,
        )
        .expect("normalize should succeed");
        let archive = ArchiveRepository::get_by_source(&repos, &source.id)
            .expect("archive query should succeed")
            .expect("archive should exist");
        let task = normalize_archive_status(&repos, &TaskId(summary.task_id.clone()))
            .expect("task should exist");

        assert_eq!(summary.indexed_entries, 2);
        assert_eq!(archive.status, "normalized");
        assert!(archive.normalized_zip_path.is_some());
        assert_eq!(task.state, TaskState::Completed);
        assert_eq!(
            ArchiveEntryRepository::list_by_archive(&repos, &archive.id)
                .expect("entries should exist")
                .len(),
            2
        );
    }

    #[test]
    fn supports_normalize_retry_after_failure() {
        let repos = MemoryRepos::default();
        let temp = tempdir().expect("tempdir should be created");
        let source_path = temp.path().join("chapter.rar");
        std::fs::write(&source_path, b"fake-rar").expect("source archive should exist");

        let library = LibraryRecord {
            id: LibraryId("library_normalize_retry".to_string()),
            root_path: temp.path().display().to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        let source = SourceRecord {
            id: SourceId("source_normalize_retry".to_string()),
            library_id: library.id.clone(),
            normalized_path: source_path.display().to_string(),
            file_name: "chapter.rar".to_string(),
            ext: "rar".to_string(),
            kind: SourceKind::Archive,
            size: 10,
            mtime_ms: 1,
            fingerprint: Some("fp-normalize-retry".to_string()),
            exists: true,
            last_seen_at: "1".to_string(),
        };

        LibraryRepository::upsert(&repos, &library).expect("library should be stored");
        SourceRepository::upsert(&repos, &source).expect("source should be stored");
        let task_id = normalize_task_id(&source.id);

        let failing_extractor = MockExtractor {
            fail_first: Mutex::new(true),
            files: vec![("001-cover.png".to_string(), b"cover".to_vec())],
        };
        let first = normalize_archive_source_with_extractor(
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            temp.path(),
            &source.id,
            &task_id,
            &failing_extractor,
        );
        assert!(first.is_err());
        assert_eq!(
            ArchiveRepository::get_by_source(&repos, &source.id)
                .expect("archive query should succeed")
                .expect("archive should exist")
                .status,
            "failed"
        );
        let latest_failed = repos
            .tasks
            .lock()
            .expect("lock")
            .values()
            .find(|item| item.task_type == TaskKind::Normalize && item.state == TaskState::Failed)
            .cloned()
            .expect("failed normalize task should exist");
        assert_eq!(
            latest_failed.error_code.as_deref(),
            Some("normalize_failed")
        );

        let success_extractor = MockExtractor {
            fail_first: Mutex::new(false),
            files: vec![("001-cover.png".to_string(), b"cover".to_vec())],
        };
        let second = normalize_archive_source_with_extractor(
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            temp.path(),
            &source.id,
            &task_id,
            &success_extractor,
        )
        .expect("retry normalize should succeed");

        assert_eq!(second.indexed_entries, 1);
        assert_eq!(
            ArchiveRepository::get_by_source(&repos, &source.id)
                .expect("archive query should succeed")
                .expect("archive should exist")
                .status,
            "normalized"
        );
    }

    fn write_test_zip(path: &Path, files: &[(&str, &[u8])]) {
        let file = File::create(path).expect("zip file should be created");
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        for (name, contents) in files {
            writer
                .start_file(name, options)
                .expect("zip entry should start");
            writer
                .write_all(contents)
                .expect("zip entry should be written");
        }

        writer.finish().expect("zip writer should finish");
    }
}
