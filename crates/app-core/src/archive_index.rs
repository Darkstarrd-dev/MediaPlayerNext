use crate::archive::archive_index_path_for_source;
use crate::archive_normalize::{archive_entry_id_from_path, archive_id_from_source};
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository,
};
use anyhow::{anyhow, Result};
use media_io::archive::build_zip_index;
use media_io::is_primary_archive_extension;
use serde::Serialize;
use shared_model::{ArchiveEntryRecord, ArchiveRecord, LibraryId, SourceId, SourceKind};

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
