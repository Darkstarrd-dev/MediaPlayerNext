use crate::asset::{
    archive_entry_asset_id_from_entry, file_asset_id_from_source, file_mime_from_extension,
    file_mime_from_path, now_string, AssetEnsureSummary, AssetSnapshotItem,
};
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository, SourceRepository,
};
use anyhow::{anyhow, Result};
use shared_model::{LibraryId, MediaAssetRecord, MediaSourceKind, SourceKind};

pub fn ensure_media_assets_for_library<L, S, A, E, R>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    asset_repository: &R,
    library_id: &LibraryId,
) -> Result<AssetEnsureSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
{
    if !library_repository.exists(library_id)? {
        return Err(anyhow!("library not found: {}", library_id.0));
    }

    let mut ensured_assets = 0_u64;
    let mut file_assets = 0_u64;
    let mut archive_entry_assets = 0_u64;
    let mut skipped_sources = 0_u64;
    let mut skipped_archive_entries = 0_u64;

    for source in source_repository.list_by_library(library_id)? {
        if !source.exists {
            skipped_sources += 1;
            continue;
        }

        match source.kind {
            SourceKind::Image => {
                let asset = MediaAssetRecord {
                    id: file_asset_id_from_source(&source.id),
                    source_kind: MediaSourceKind::File,
                    source_ref_id: source.id.0.clone(),
                    mime: file_mime_from_extension(&source.ext),
                    width: None,
                    height: None,
                    duration_ms: None,
                    codec_info_json: None,
                    orientation: None,
                    created_at: now_string(),
                };
                asset_repository.upsert(&asset)?;
                ensured_assets += 1;
                file_assets += 1;
            }
            SourceKind::Archive => {
                let Some(archive) = archive_repository.get_by_source(&source.id)? else {
                    skipped_sources += 1;
                    continue;
                };

                for entry in archive_entry_repository.list_by_archive(&archive.id)? {
                    if !entry.media_kind.eq_ignore_ascii_case("image") {
                        skipped_archive_entries += 1;
                        continue;
                    }

                    let asset = MediaAssetRecord {
                        id: archive_entry_asset_id_from_entry(&entry.id),
                        source_kind: MediaSourceKind::ArchiveEntry,
                        source_ref_id: entry.id.0.clone(),
                        mime: file_mime_from_path(&entry.entry_path),
                        width: entry.width,
                        height: entry.height,
                        duration_ms: None,
                        codec_info_json: None,
                        orientation: None,
                        created_at: now_string(),
                    };
                    asset_repository.upsert(&asset)?;
                    ensured_assets += 1;
                    archive_entry_assets += 1;
                }
            }
            _ => {
                skipped_sources += 1;
            }
        }
    }

    Ok(AssetEnsureSummary {
        library_id: library_id.0.clone(),
        ensured_assets,
        file_assets,
        archive_entry_assets,
        skipped_sources,
        skipped_archive_entries,
    })
}

pub fn asset_snapshot_for_library<L, S, A, E>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    library_id: &LibraryId,
) -> Result<Vec<AssetSnapshotItem>>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
{
    if !library_repository.exists(library_id)? {
        return Err(anyhow!("library not found: {}", library_id.0));
    }

    let mut items = Vec::new();

    for source in source_repository.list_by_library(library_id)? {
        if !source.exists {
            continue;
        }

        match source.kind {
            SourceKind::Image => items.push(AssetSnapshotItem {
                asset_id: file_asset_id_from_source(&source.id).0,
                source_kind: "file".to_string(),
                source_ref_id: source.id.0.clone(),
                library_id: library_id.0.clone(),
                source_id: source.id.0,
                archive_id: None,
                entry_path: None,
                mime: file_mime_from_extension(&source.ext),
            }),
            SourceKind::Archive => {
                let Some(archive) = archive_repository.get_by_source(&source.id)? else {
                    continue;
                };

                for entry in archive_entry_repository.list_by_archive(&archive.id)? {
                    if !entry.media_kind.eq_ignore_ascii_case("image") {
                        continue;
                    }

                    items.push(AssetSnapshotItem {
                        asset_id: archive_entry_asset_id_from_entry(&entry.id).0,
                        source_kind: "archive_entry".to_string(),
                        source_ref_id: entry.id.0.clone(),
                        library_id: library_id.0.clone(),
                        source_id: source.id.0.clone(),
                        archive_id: Some(archive.id.0.clone()),
                        entry_path: Some(entry.entry_path.clone()),
                        mime: file_mime_from_path(&entry.entry_path),
                    });
                }
            }
            _ => {}
        }
    }

    items.sort_by(|left, right| {
        left.source_kind
            .cmp(&right.source_kind)
            .then(left.source_id.cmp(&right.source_id))
            .then(left.entry_path.cmp(&right.entry_path))
    });
    Ok(items)
}
