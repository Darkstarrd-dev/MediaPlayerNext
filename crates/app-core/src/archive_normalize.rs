use crate::archive::ArchiveNormalizeSummary;
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository, TaskRepository,
};
use anyhow::{anyhow, Result};
use media_io::archive::build_zip_index;
use media_io::normalize::{normalize_archive_to_zip, normalized_archive_layout, ArchiveExtractor};
use media_io::{is_normalizable_archive_extension, normalize_path};
use shared_model::{
    ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, SourceId, SourceKind, TaskId,
    TaskKind, TaskRecord, TaskState,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn normalize_archive_source<L, S, A, E, T, X>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    task_repository: &T,
    normalize_root: &Path,
    source_id: &SourceId,
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
    let task_id = normalize_task_id(source_id);
    normalize_archive_source_with_extractor(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        task_repository,
        normalize_root,
        source_id,
        &task_id,
        extractor,
    )
}

pub(crate) fn normalize_archive_status<T>(
    task_repository: &T,
    task_id: &TaskId,
) -> Result<TaskRecord>
where
    T: TaskRepository,
{
    task_repository
        .get(task_id)?
        .ok_or_else(|| anyhow!("task not found: {}", task_id.0))
}

pub(crate) fn normalize_archive_source_with_extractor<L, S, A, E, T, X>(
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

pub(crate) fn archive_id_from_source(source_id: &SourceId) -> ArchiveId {
    ArchiveId(format!(
        "archive_{:016x}",
        stable_hash(&format!("archive::{}", source_id.0))
    ))
}

pub(crate) fn archive_entry_id_from_path(
    archive_id: &ArchiveId,
    entry_path: &str,
) -> ArchiveEntryId {
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

pub(crate) fn normalize_task_id(source_id: &SourceId) -> TaskId {
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
