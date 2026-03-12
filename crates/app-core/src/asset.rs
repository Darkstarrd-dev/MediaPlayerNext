pub use crate::asset_catalog::{asset_snapshot_for_library, ensure_media_assets_for_library};
pub use crate::asset_resolve::resolve_asset;
use serde::Serialize;
use shared_model::{ArchiveEntryId, AssetId, SourceId};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetEnsureSummary {
    pub library_id: String,
    pub ensured_assets: u64,
    pub file_assets: u64,
    pub archive_entry_assets: u64,
    pub skipped_sources: u64,
    pub skipped_archive_entries: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedFileAsset {
    pub asset_id: String,
    pub source_id: String,
    pub library_id: String,
    pub mime: String,
    pub file_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedArchiveEntryAsset {
    pub asset_id: String,
    pub archive_entry_id: String,
    pub archive_id: String,
    pub source_id: String,
    pub library_id: String,
    pub mime: String,
    pub archive_path: String,
    pub entry_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSnapshotItem {
    pub asset_id: String,
    pub source_kind: String,
    pub source_ref_id: String,
    pub library_id: String,
    pub source_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entry_path: Option<String>,
    pub mime: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssetResolution {
    File(ResolvedFileAsset),
    ArchiveEntry(ResolvedArchiveEntryAsset),
}

pub(crate) fn source_path_from_library(
    library_root: &str,
    normalized_source_path: &str,
) -> PathBuf {
    let candidate = PathBuf::from(normalized_source_path);
    if candidate.is_absolute() {
        return candidate;
    }
    Path::new(library_root).join(normalized_source_path)
}

pub(crate) fn file_asset_id_from_source(source_id: &SourceId) -> AssetId {
    AssetId(format!(
        "asset_{:016x}",
        stable_hash(&format!("file::{}", source_id.0))
    ))
}

pub(crate) fn archive_entry_asset_id_from_entry(entry_id: &ArchiveEntryId) -> AssetId {
    AssetId(format!(
        "asset_{:016x}",
        stable_hash(&format!("archive_entry::{}", entry_id.0))
    ))
}

pub(crate) fn file_mime_from_path(path: &str) -> String {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    file_mime_from_extension(extension)
}

pub(crate) fn file_mime_from_extension(extension: &str) -> String {
    match extension.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" => "image/jpeg",
        "png" => "image/png",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        _ => "application/octet-stream",
    }
    .to_string()
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

pub(crate) fn now_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.to_string()
}
#[cfg(test)]
mod tests {
    use super::{
        archive_entry_asset_id_from_entry, asset_snapshot_for_library,
        ensure_media_assets_for_library, file_asset_id_from_source, resolve_asset, AssetResolution,
    };
    use crate::ports::{
        ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
        SourceRepository,
    };
    #[path = "../asset_test_support.rs"]
    mod support;
    use shared_model::{
        ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, LibraryId, LibraryRecord,
        MediaAssetRecord, SourceId, SourceKind, SourceRecord,
    };
    use support::MemoryRepos;
    #[test]
    fn ensures_file_and_archive_entry_assets_for_library() {
        let repos = MemoryRepos::default();
        let library = LibraryRecord {
            id: LibraryId("library_assets".to_string()),
            root_path: "Z:/Library".to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        LibraryRepository::upsert(&repos, &library).expect("library should exist");
        let image_source = SourceRecord {
            id: SourceId("source_image".to_string()),
            library_id: library.id.clone(),
            normalized_path: "z:/library/cover.png".to_string(),
            file_name: "cover.png".to_string(),
            ext: "png".to_string(),
            kind: SourceKind::Image,
            size: 100,
            mtime_ms: 1,
            fingerprint: Some("fp-image".to_string()),
            exists: true,
            last_seen_at: "1".to_string(),
        };
        let archive_source = SourceRecord {
            id: SourceId("source_archive".to_string()),
            library_id: library.id.clone(),
            normalized_path: "z:/library/chapter.cbz".to_string(),
            file_name: "chapter.cbz".to_string(),
            ext: "cbz".to_string(),
            kind: SourceKind::Archive,
            size: 200,
            mtime_ms: 2,
            fingerprint: Some("fp-archive".to_string()),
            exists: true,
            last_seen_at: "2".to_string(),
        };
        SourceRepository::upsert(&repos, &image_source).expect("image source should exist");
        SourceRepository::upsert(&repos, &archive_source).expect("archive source should exist");
        let archive = ArchiveRecord {
            id: ArchiveId("archive_primary".to_string()),
            source_id: archive_source.id.clone(),
            archive_type: "cbz".to_string(),
            normalized_zip_path: Some(archive_source.normalized_path.clone()),
            page_count: Some(2),
            cover_entry_id: Some(ArchiveEntryId("entry_cover".to_string())),
            status: "indexed".to_string(),
        };
        ArchiveRepository::upsert(&repos, &archive).expect("archive should exist");
        ArchiveEntryRepository::replace_for_archive(
            &repos,
            &archive.id,
            &[
                ArchiveEntryRecord {
                    id: ArchiveEntryId("entry_cover".to_string()),
                    archive_id: archive.id.clone(),
                    entry_path: "001-cover.png".to_string(),
                    entry_name: "001-cover.png".to_string(),
                    page_index: 0,
                    media_kind: "image".to_string(),
                    width: Some(1000),
                    height: Some(800),
                    compressed_size: Some(10),
                    uncompressed_size: Some(20),
                    crc32: Some(1),
                },
                ArchiveEntryRecord {
                    id: ArchiveEntryId("entry_note".to_string()),
                    archive_id: archive.id.clone(),
                    entry_path: "notes.txt".to_string(),
                    entry_name: "notes.txt".to_string(),
                    page_index: 1,
                    media_kind: "text".to_string(),
                    width: None,
                    height: None,
                    compressed_size: None,
                    uncompressed_size: None,
                    crc32: None,
                },
            ],
        )
        .expect("entries should exist");
        let summary =
            ensure_media_assets_for_library(&repos, &repos, &repos, &repos, &repos, &library.id)
                .expect("asset ensure should succeed");
        assert_eq!(summary.ensured_assets, 2);
        assert_eq!(summary.file_assets, 1);
        assert_eq!(summary.archive_entry_assets, 1);
        assert_eq!(summary.skipped_archive_entries, 1);
        assert!(
            AssetRepository::exists(&repos, &file_asset_id_from_source(&image_source.id))
                .expect("file asset should exist")
        );
        assert!(AssetRepository::exists(
            &repos,
            &archive_entry_asset_id_from_entry(&ArchiveEntryId("entry_cover".to_string())),
        )
        .expect("archive entry asset should exist"));
    }
    #[test]
    fn builds_asset_snapshot_for_library() {
        let repos = MemoryRepos::default();
        let library = LibraryRecord {
            id: LibraryId("library_assets_snapshot".to_string()),
            root_path: "Z:/Library".to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        let image_source = SourceRecord {
            id: SourceId("source_snapshot_image".to_string()),
            library_id: library.id.clone(),
            normalized_path: "z:/library/cover.png".to_string(),
            file_name: "cover.png".to_string(),
            ext: "png".to_string(),
            kind: SourceKind::Image,
            size: 100,
            mtime_ms: 1,
            fingerprint: Some("fp-image".to_string()),
            exists: true,
            last_seen_at: "1".to_string(),
        };
        let archive_source = SourceRecord {
            id: SourceId("source_snapshot_archive".to_string()),
            library_id: library.id.clone(),
            normalized_path: "z:/library/chapter.cbz".to_string(),
            file_name: "chapter.cbz".to_string(),
            ext: "cbz".to_string(),
            kind: SourceKind::Archive,
            size: 200,
            mtime_ms: 2,
            fingerprint: Some("fp-archive".to_string()),
            exists: true,
            last_seen_at: "2".to_string(),
        };
        let archive = ArchiveRecord {
            id: ArchiveId("archive_snapshot_primary".to_string()),
            source_id: archive_source.id.clone(),
            archive_type: "cbz".to_string(),
            normalized_zip_path: Some(archive_source.normalized_path.clone()),
            page_count: Some(1),
            cover_entry_id: Some(ArchiveEntryId("entry_snapshot_cover".to_string())),
            status: "indexed".to_string(),
        };
        let entry = ArchiveEntryRecord {
            id: ArchiveEntryId("entry_snapshot_cover".to_string()),
            archive_id: archive.id.clone(),
            entry_path: "001-cover.png".to_string(),
            entry_name: "001-cover.png".to_string(),
            page_index: 0,
            media_kind: "image".to_string(),
            width: Some(1000),
            height: Some(800),
            compressed_size: Some(10),
            uncompressed_size: Some(20),
            crc32: Some(1),
        };
        LibraryRepository::upsert(&repos, &library).expect("library should exist");
        SourceRepository::upsert(&repos, &image_source).expect("image source should exist");
        SourceRepository::upsert(&repos, &archive_source).expect("archive source should exist");
        ArchiveRepository::upsert(&repos, &archive).expect("archive should exist");
        ArchiveEntryRepository::replace_for_archive(&repos, &archive.id, &[entry.clone()])
            .expect("entry should exist");

        let snapshot = asset_snapshot_for_library(&repos, &repos, &repos, &repos, &library.id)
            .expect("asset snapshot should succeed");
        assert_eq!(snapshot.len(), 2);
        assert!(snapshot.iter().any(|item| item.source_kind == "file"));
        assert!(snapshot
            .iter()
            .any(|item| item.source_kind == "archive_entry"
                && item.entry_path.as_deref() == Some("001-cover.png")));
    }
    #[test]
    fn resolves_file_and_archive_entry_assets() {
        let repos = MemoryRepos::default();
        let library = LibraryRecord {
            id: LibraryId("library_assets".to_string()),
            root_path: "Z:/Library".to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        let image_source = SourceRecord {
            id: SourceId("source_image".to_string()),
            library_id: library.id.clone(),
            normalized_path: "z:/library/cover.png".to_string(),
            file_name: "cover.png".to_string(),
            ext: "png".to_string(),
            kind: SourceKind::Image,
            size: 100,
            mtime_ms: 1,
            fingerprint: Some("fp-image".to_string()),
            exists: true,
            last_seen_at: "1".to_string(),
        };
        let archive_source = SourceRecord {
            id: SourceId("source_archive".to_string()),
            library_id: library.id.clone(),
            normalized_path: "z:/library/chapter.cbz".to_string(),
            file_name: "chapter.cbz".to_string(),
            ext: "cbz".to_string(),
            kind: SourceKind::Archive,
            size: 200,
            mtime_ms: 2,
            fingerprint: Some("fp-archive".to_string()),
            exists: true,
            last_seen_at: "2".to_string(),
        };
        let archive = ArchiveRecord {
            id: ArchiveId("archive_primary".to_string()),
            source_id: archive_source.id.clone(),
            archive_type: "cbz".to_string(),
            normalized_zip_path: Some(archive_source.normalized_path.clone()),
            page_count: Some(1),
            cover_entry_id: Some(ArchiveEntryId("entry_cover".to_string())),
            status: "indexed".to_string(),
        };
        let entry = ArchiveEntryRecord {
            id: ArchiveEntryId("entry_cover".to_string()),
            archive_id: archive.id.clone(),
            entry_path: "001-cover.png".to_string(),
            entry_name: "001-cover.png".to_string(),
            page_index: 0,
            media_kind: "image".to_string(),
            width: Some(1000),
            height: Some(800),
            compressed_size: Some(10),
            uncompressed_size: Some(20),
            crc32: Some(1),
        };
        LibraryRepository::upsert(&repos, &library).expect("library should exist");
        SourceRepository::upsert(&repos, &image_source).expect("image source should exist");
        SourceRepository::upsert(&repos, &archive_source).expect("archive source should exist");
        ArchiveRepository::upsert(&repos, &archive).expect("archive should exist");
        ArchiveEntryRepository::replace_for_archive(&repos, &archive.id, &[entry.clone()])
            .expect("entry should exist");
        AssetRepository::upsert(
            &repos,
            &MediaAssetRecord {
                id: file_asset_id_from_source(&image_source.id),
                source_kind: shared_model::MediaSourceKind::File,
                source_ref_id: image_source.id.0.clone(),
                mime: "image/png".to_string(),
                width: None,
                height: None,
                duration_ms: None,
                codec_info_json: None,
                orientation: None,
                created_at: "1".to_string(),
            },
        )
        .expect("file asset should exist");
        AssetRepository::upsert(
            &repos,
            &MediaAssetRecord {
                id: archive_entry_asset_id_from_entry(&entry.id),
                source_kind: shared_model::MediaSourceKind::ArchiveEntry,
                source_ref_id: entry.id.0.clone(),
                mime: "image/png".to_string(),
                width: entry.width,
                height: entry.height,
                duration_ms: None,
                codec_info_json: None,
                orientation: None,
                created_at: "1".to_string(),
            },
        )
        .expect("archive asset should exist");
        let file_resolution = resolve_asset(
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            &file_asset_id_from_source(&image_source.id),
        )
        .expect("file asset should resolve");
        let archive_resolution = resolve_asset(
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            &archive_entry_asset_id_from_entry(&entry.id),
        )
        .expect("archive asset should resolve");
        match file_resolution {
            AssetResolution::File(file) => {
                assert_eq!(file.source_id, image_source.id.0);
                assert_eq!(file.file_path, "z:/library/cover.png");
            }
            other => panic!("unexpected resolution: {other:?}"),
        }
        match archive_resolution {
            AssetResolution::ArchiveEntry(item) => {
                assert_eq!(item.archive_id, archive.id.0);
                assert_eq!(item.entry_path, "001-cover.png");
                assert_eq!(item.archive_path, "z:/library/chapter.cbz");
            }
            other => panic!("unexpected resolution: {other:?}"),
        }
    }
}
