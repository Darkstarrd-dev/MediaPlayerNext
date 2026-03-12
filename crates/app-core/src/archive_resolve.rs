use crate::archive::archive_index_path_for_source;
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, LibraryRepository, SourceRepository,
};
use anyhow::{anyhow, Result};
use media_io::archive::read_zip_entry_bytes;
use shared_model::{ArchiveEntryId, SourceId};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEntryReadSummary {
    pub source_id: String,
    pub entry_path: String,
    pub byte_count: usize,
    pub preview_hex: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedArchiveEntryLocation {
    pub archive_entry_id: String,
    pub archive_id: String,
    pub source_id: String,
    pub archive_path: String,
    pub entry_path: String,
    pub media_kind: String,
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
