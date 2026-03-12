pub use crate::archive_index::{
    archive_snapshot, index_library_archives, ArchiveIndexSummary, ArchiveSnapshot,
};
use crate::archive_normalize::normalize_task_id;
pub use crate::archive_resolve::{
    read_archive_entry, read_archive_entry_by_source, resolve_archive_entry_location,
};
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository, TaskRepository,
};
use anyhow::Result;
use media_io::is_primary_archive_extension;
use media_io::normalize::SevenZipExtractor;
use serde::Serialize;
use shared_model::{ArchiveRecord, LogContext, SourceId, TaskId, TaskRecord};
use std::path::{Path, PathBuf};

pub use crate::archive_resolve::{ArchiveEntryReadSummary, ResolvedArchiveEntryLocation};

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
    let extractor = SevenZipExtractor::new(sevenz_path).with_context(LogContext {
        task_id: Some(normalize_task_id(source_id)),
        source_id: Some(source_id.clone()),
        ..LogContext::default()
    });
    crate::archive_normalize::normalize_archive_source(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        task_repository,
        normalize_root,
        source_id,
        &extractor,
    )
}

pub fn normalize_archive_status<T>(task_repository: &T, task_id: &TaskId) -> Result<TaskRecord>
where
    T: TaskRepository,
{
    crate::archive_normalize::normalize_archive_status(task_repository, task_id)
}

pub(crate) fn archive_path_from_source(
    library_root: &str,
    normalized_source_path: &str,
) -> PathBuf {
    let candidate = PathBuf::from(normalized_source_path);
    if candidate.is_absolute() {
        return candidate;
    }

    Path::new(library_root).join(normalized_source_path)
}

pub(crate) fn archive_index_path_for_source(
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

#[cfg(test)]
mod tests {
    use super::{
        archive_snapshot, index_library_archives, normalize_archive_status,
        read_archive_entry_by_source,
    };
    use crate::archive_normalize::{normalize_archive_source_with_extractor, normalize_task_id};
    use crate::ports::{
        ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository,
    };
    #[path = "../archive_test_support.rs"]
    mod support;
    use shared_model::{
        ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, LibraryId, LibraryRecord,
        SourceId, SourceKind, SourceRecord, TaskId, TaskKind, TaskRecord, TaskState,
    };
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::sync::Mutex;
    use support::{lock_or_poison, MemoryRepos, MockExtractor};
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};
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
        let latest_failed = lock_or_poison(&repos.tasks)
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
