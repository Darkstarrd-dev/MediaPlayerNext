use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository,
};
use anyhow::{anyhow, Result};
use media_io::archive::{build_zip_index, read_zip_entry_bytes};
use media_io::is_primary_archive_extension;
use serde::Serialize;
use shared_model::{
    ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, LibraryId, SourceId, SourceKind,
};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

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

        if !is_primary_archive_extension(&source.ext) {
            skipped_non_primary_archives += 1;
            continue;
        }

        let archive_path = archive_path_from_source(&library.root_path, &source.normalized_path);
        let zip_index = build_zip_index(&archive_path)?;
        let archive_id = archive_id_from_source(&source.id);
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
            status: if is_empty_archive {
                "empty".to_string()
            } else {
                "indexed".to_string()
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

fn archive_path_from_source(library_root: &str, normalized_source_path: &str) -> PathBuf {
    let candidate = PathBuf::from(normalized_source_path);
    if candidate.is_absolute() {
        return candidate;
    }

    Path::new(library_root).join(normalized_source_path)
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

#[cfg(test)]
mod tests {
    use super::{archive_snapshot, index_library_archives};
    use crate::ports::{
        ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository,
    };
    use shared_model::{
        ArchiveEntryRecord, ArchiveId, ArchiveRecord, LibraryId, LibraryRecord, SourceId,
        SourceKind, SourceRecord,
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
