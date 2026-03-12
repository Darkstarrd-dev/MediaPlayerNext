use crate::asset::{resolve_asset, AssetResolution};
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
    SourceRepository, ThumbnailRepository,
};
use anyhow::{anyhow, Result};
use media_thumb::{ThumbnailProfile, ThumbnailService, ThumbnailSource};
use serde::Serialize;
use shared_model::{
    ArchiveEntryId, AssetId, MediaSourceKind, SourceId, ThumbnailKey, ThumbnailRecord,
};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const THUMBNAIL_PIPELINE_VERSION: &str = "b5-v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailEnsureSummary {
    pub asset_id: String,
    pub thumbnail_key: String,
    pub profile: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub disk_path: String,
    pub byte_size: i64,
    pub state: String,
    pub cache_hit: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn ensure_thumbnail_for_asset<L, S, A, E, R, T>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    asset_repository: &R,
    thumbnail_repository: &T,
    cache_root: &Path,
    asset_id: &AssetId,
    profile: ThumbnailProfile,
) -> Result<ThumbnailEnsureSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
    T: ThumbnailRepository,
{
    let asset = asset_repository
        .get(asset_id)?
        .ok_or_else(|| anyhow!("asset not found: {}", asset_id.0))?;
    let source = thumbnail_source_from_asset(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        asset_repository,
        &asset.id,
        &asset.source_kind,
    )?;

    let service = ThumbnailService::new(cache_root, THUMBNAIL_PIPELINE_VERSION);
    let generated = service.ensure_thumbnail(&source, profile)?;
    let record = ThumbnailRecord {
        thumbnail_key: ThumbnailKey(generated.thumbnail_key.clone()),
        asset_id: asset.id.clone(),
        profile: profile.as_str().to_string(),
        width: generated.width,
        height: generated.height,
        format: generated.format.clone(),
        disk_path: generated.disk_path.display().to_string(),
        byte_size: generated.byte_size as i64,
        state: "ready".to_string(),
        updated_at: now_string(),
    };
    thumbnail_repository.upsert(&record)?;

    Ok(ThumbnailEnsureSummary {
        asset_id: asset.id.0,
        thumbnail_key: record.thumbnail_key.0,
        profile: record.profile,
        width: record.width,
        height: record.height,
        format: record.format,
        disk_path: record.disk_path,
        byte_size: record.byte_size,
        state: record.state,
        cache_hit: generated.cache_hit,
    })
}

pub fn get_thumbnail<T>(
    thumbnail_repository: &T,
    thumbnail_key: &ThumbnailKey,
) -> Result<ThumbnailRecord>
where
    T: ThumbnailRepository,
{
    thumbnail_repository
        .get(thumbnail_key)?
        .ok_or_else(|| anyhow!("thumbnail not found: {}", thumbnail_key.0))
}

pub fn parse_thumbnail_profile(value: &str) -> Result<ThumbnailProfile> {
    match value {
        "grid-sm" => Ok(ThumbnailProfile::GridSm),
        "grid-md" => Ok(ThumbnailProfile::GridMd),
        "detail-md" => Ok(ThumbnailProfile::DetailMd),
        "detail-lg" => Ok(ThumbnailProfile::DetailLg),
        _ => Err(anyhow!("unsupported thumbnail profile: {value}")),
    }
}

fn thumbnail_source_from_asset<L, S, A, E, R>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    asset_repository: &R,
    asset_id: &AssetId,
    source_kind: &MediaSourceKind,
) -> Result<ThumbnailSource>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
{
    let resolution = resolve_asset(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        asset_repository,
        asset_id,
    )?;

    match (source_kind, resolution) {
        (MediaSourceKind::File, AssetResolution::File(item)) => {
            let source = source_repository
                .get(&SourceId(item.source_id.clone()))?
                .ok_or_else(|| anyhow!("source not found: {}", item.source_id))?;
            let source_revision = source
                .fingerprint
                .clone()
                .unwrap_or_else(|| format!("{}:{}", source.mtime_ms, source.size));

            Ok(ThumbnailSource::FilePath {
                source_identity: item.source_id,
                source_revision,
                file_path: item.file_path,
            })
        }
        (MediaSourceKind::ArchiveEntry, AssetResolution::ArchiveEntry(item)) => {
            let entry = archive_entry_repository
                .get(&ArchiveEntryId(item.archive_entry_id.clone()))?
                .ok_or_else(|| anyhow!("archive entry not found: {}", item.archive_entry_id))?;
            let source_revision = format!(
                "{}:{}:{}:{}",
                item.archive_id,
                entry.entry_path,
                entry.crc32.unwrap_or_default(),
                entry.uncompressed_size.unwrap_or_default(),
            );

            Ok(ThumbnailSource::ArchiveEntry {
                source_identity: item.archive_entry_id,
                source_revision,
                archive_path: item.archive_path,
                entry_path: item.entry_path,
            })
        }
        (MediaSourceKind::NormalizedFile, _) => Err(anyhow!(
            "normalized_file thumbnail source is not implemented yet"
        )),
        (_, _) => Err(anyhow!("asset resolution kind mismatch")),
    }
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
    use super::{ensure_thumbnail_for_asset, get_thumbnail, parse_thumbnail_profile};
    use crate::asset::{archive_entry_asset_id_from_entry, file_asset_id_from_source};
    use crate::ports::{
        ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
        SourceRepository, ThumbnailRepository,
    };
    #[path = "../thumbnail_test_support.rs"]
    mod support;
    use image::{DynamicImage, ImageBuffer, Rgba};
    use shared_model::{
        ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, LibraryId,
        LibraryRecord, MediaAssetRecord, MediaSourceKind, SourceId, SourceKind, SourceRecord,
        ThumbnailKey, ThumbnailRecord,
    };
    use std::fs::File;
    use std::io::{Cursor, Write};
    use support::MemoryRepos;
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn ensures_and_persists_thumbnail_for_file_asset() {
        let repos = MemoryRepos::default();
        let temp = tempdir().expect("tempdir");
        let source_path = temp.path().join("cover.png");
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(1200, 800, Rgba([255, 10, 10, 255]))
            .save(&source_path)
            .expect("source image should be saved");

        let library = LibraryRecord {
            id: LibraryId("library_thumb_file".to_string()),
            root_path: temp.path().display().to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        let source = SourceRecord {
            id: SourceId("source_thumb_file".to_string()),
            library_id: library.id.clone(),
            normalized_path: source_path.display().to_string(),
            file_name: "cover.png".to_string(),
            ext: "png".to_string(),
            kind: SourceKind::Image,
            size: 100,
            mtime_ms: 1,
            fingerprint: Some("fp-thumb-file".to_string()),
            exists: true,
            last_seen_at: "1".to_string(),
        };
        let asset = MediaAssetRecord {
            id: file_asset_id_from_source(&source.id),
            source_kind: MediaSourceKind::File,
            source_ref_id: source.id.0.clone(),
            mime: "image/png".to_string(),
            width: None,
            height: None,
            duration_ms: None,
            codec_info_json: None,
            orientation: None,
            created_at: "1".to_string(),
        };

        LibraryRepository::upsert(&repos, &library).expect("library should be saved");
        SourceRepository::upsert(&repos, &source).expect("source should be saved");
        AssetRepository::upsert(&repos, &asset).expect("asset should be saved");

        let summary = ensure_thumbnail_for_asset(
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            &temp.path().join("thumb-cache"),
            &asset.id,
            parse_thumbnail_profile("grid-sm").expect("profile should parse"),
        )
        .expect("thumbnail ensure should succeed");

        assert_eq!(summary.asset_id, asset.id.0);
        assert_eq!(summary.profile, "grid-sm");
        assert!(summary.width <= 240);
        assert!(summary.height <= 240);
        assert!(std::path::Path::new(&summary.disk_path).exists());

        let saved = get_thumbnail(&repos, &ThumbnailKey(summary.thumbnail_key.clone()))
            .expect("thumbnail should be queryable");
        assert_eq!(saved.asset_id, asset.id);
    }

    #[test]
    fn ensures_and_persists_thumbnail_for_archive_asset() {
        let repos = MemoryRepos::default();
        let temp = tempdir().expect("tempdir");
        let archive_path = temp.path().join("chapter.cbz");
        let image =
            ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(900, 1300, Rgba([10, 10, 255, 255]));
        let mut png_bytes = Vec::new();
        DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .expect("png fixture should encode");
        write_zip(&archive_path, "001-cover.png", &png_bytes);

        let library = LibraryRecord {
            id: LibraryId("library_thumb_archive".to_string()),
            root_path: temp.path().display().to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        let source = SourceRecord {
            id: SourceId("source_thumb_archive".to_string()),
            library_id: library.id.clone(),
            normalized_path: archive_path.display().to_string(),
            file_name: "chapter.cbz".to_string(),
            ext: "cbz".to_string(),
            kind: SourceKind::Archive,
            size: 200,
            mtime_ms: 2,
            fingerprint: Some("fp-thumb-archive".to_string()),
            exists: true,
            last_seen_at: "2".to_string(),
        };
        let archive = ArchiveRecord {
            id: ArchiveId("archive_thumb_primary".to_string()),
            source_id: source.id.clone(),
            archive_type: "cbz".to_string(),
            normalized_zip_path: Some(archive_path.display().to_string()),
            page_count: Some(1),
            cover_entry_id: Some(ArchiveEntryId("entry_thumb_cover".to_string())),
            status: "indexed".to_string(),
        };
        let entry = ArchiveEntryRecord {
            id: ArchiveEntryId("entry_thumb_cover".to_string()),
            archive_id: archive.id.clone(),
            entry_path: "001-cover.png".to_string(),
            entry_name: "001-cover.png".to_string(),
            page_index: 0,
            media_kind: "image".to_string(),
            width: Some(900),
            height: Some(1300),
            compressed_size: Some(10),
            uncompressed_size: Some(20),
            crc32: Some(99),
        };
        let asset = MediaAssetRecord {
            id: archive_entry_asset_id_from_entry(&entry.id),
            source_kind: MediaSourceKind::ArchiveEntry,
            source_ref_id: entry.id.0.clone(),
            mime: "image/png".to_string(),
            width: entry.width,
            height: entry.height,
            duration_ms: None,
            codec_info_json: None,
            orientation: None,
            created_at: "1".to_string(),
        };

        LibraryRepository::upsert(&repos, &library).expect("library should be saved");
        SourceRepository::upsert(&repos, &source).expect("source should be saved");
        ArchiveRepository::upsert(&repos, &archive).expect("archive should be saved");
        ArchiveEntryRepository::replace_for_archive(&repos, &archive.id, &[entry.clone()])
            .expect("entry should be saved");
        AssetRepository::upsert(&repos, &asset).expect("asset should be saved");

        let summary = ensure_thumbnail_for_asset(
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            &temp.path().join("thumb-cache"),
            &asset.id,
            parse_thumbnail_profile("grid-sm").expect("profile should parse"),
        )
        .expect("thumbnail ensure should succeed");

        assert_eq!(summary.asset_id, asset.id.0);
        assert!(summary.width <= 240);
        assert!(summary.height <= 240);

        let saved = get_thumbnail(&repos, &ThumbnailKey(summary.thumbnail_key.clone()))
            .expect("thumbnail should be queryable");
        assert_eq!(saved.asset_id, asset.id);
    }

    fn write_zip(path: &std::path::Path, entry_name: &str, contents: &[u8]) {
        let file = File::create(path).expect("zip file should be created");
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        writer
            .start_file(entry_name, options)
            .expect("zip entry should start");
        writer
            .write_all(contents)
            .expect("zip entry should be written");
        writer.finish().expect("zip writer should finish");
    }
}
