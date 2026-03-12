use crate::asset::{
    source_path_from_library, AssetResolution, ResolvedArchiveEntryAsset, ResolvedFileAsset,
};
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository, SourceRepository,
};
use anyhow::{anyhow, Result};
use shared_model::{ArchiveEntryId, AssetId, MediaSourceKind, SourceId};

pub fn resolve_asset<L, S, A, E, R>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    asset_repository: &R,
    asset_id: &AssetId,
) -> Result<AssetResolution>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
{
    let asset = asset_repository
        .get(asset_id)?
        .ok_or_else(|| anyhow!("asset not found: {}", asset_id.0))?;

    match asset.source_kind {
        MediaSourceKind::File => {
            let source_id = SourceId(asset.source_ref_id.clone());
            let source = source_repository
                .get(&source_id)?
                .ok_or_else(|| anyhow!("source not found: {}", source_id.0))?;
            let library = library_repository
                .get(&source.library_id)?
                .ok_or_else(|| anyhow!("library not found: {}", source.library_id.0))?;
            let file_path = source_path_from_library(&library.root_path, &source.normalized_path);

            Ok(AssetResolution::File(ResolvedFileAsset {
                asset_id: asset.id.0,
                source_id: source.id.0,
                library_id: library.id.0,
                mime: asset.mime,
                file_path: file_path.display().to_string(),
            }))
        }
        MediaSourceKind::ArchiveEntry => {
            let archive_entry_id = ArchiveEntryId(asset.source_ref_id.clone());
            let entry = archive_entry_repository
                .get(&archive_entry_id)?
                .ok_or_else(|| anyhow!("archive entry not found: {}", archive_entry_id.0))?;
            let archive = archive_repository
                .get(&entry.archive_id)?
                .ok_or_else(|| anyhow!("archive not found: {}", entry.archive_id.0))?;
            let source_id = archive.source_id.clone();
            let source = source_repository
                .get(&source_id)?
                .ok_or_else(|| anyhow!("source not found: {}", source_id.0))?;
            let library = library_repository
                .get(&source.library_id)?
                .ok_or_else(|| anyhow!("library not found: {}", source.library_id.0))?;
            let archive_path =
                source_path_from_library(&library.root_path, &source.normalized_path);

            Ok(AssetResolution::ArchiveEntry(ResolvedArchiveEntryAsset {
                asset_id: asset.id.0,
                archive_entry_id: entry.id.0,
                archive_id: archive.id.0,
                source_id: source.id.0,
                library_id: library.id.0,
                mime: asset.mime,
                archive_path: archive_path.display().to_string(),
                entry_path: entry.entry_path,
            }))
        }
        MediaSourceKind::NormalizedFile => Err(anyhow!(
            "normalized_file asset resolution is not implemented yet"
        )),
    }
}
