use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository, SourceRepository,
};
use anyhow::{anyhow, Result};
use serde::Serialize;
use shared_model::{
    ArchiveEntryId, AssetId, LibraryId, MediaAssetRecord, MediaSourceKind, SourceId, SourceKind,
};
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
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AssetResolution {
    File(ResolvedFileAsset),
    ArchiveEntry(ResolvedArchiveEntryAsset),
}

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
                    id: asset_id_from_file_source(&source.id),
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
                        id: asset_id_from_archive_entry(&entry.id),
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

fn source_path_from_library(library_root: &str, normalized_source_path: &str) -> PathBuf {
    let candidate = PathBuf::from(normalized_source_path);
    if candidate.is_absolute() {
        return candidate;
    }

    Path::new(library_root).join(normalized_source_path)
}

fn asset_id_from_file_source(source_id: &SourceId) -> AssetId {
    AssetId(format!(
        "asset_{:016x}",
        stable_hash(&format!("file::{}", source_id.0))
    ))
}

fn asset_id_from_archive_entry(entry_id: &ArchiveEntryId) -> AssetId {
    AssetId(format!(
        "asset_{:016x}",
        stable_hash(&format!("archive_entry::{}", entry_id.0))
    ))
}

pub fn file_asset_id_from_source(source_id: &SourceId) -> AssetId {
    asset_id_from_file_source(source_id)
}

pub fn archive_entry_asset_id_from_entry(entry_id: &ArchiveEntryId) -> AssetId {
    asset_id_from_archive_entry(entry_id)
}

fn file_mime_from_path(path: &str) -> String {
    let extension = Path::new(path)
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    file_mime_from_extension(extension)
}

fn file_mime_from_extension(extension: &str) -> String {
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
        archive_entry_asset_id_from_entry, ensure_media_assets_for_library,
        file_asset_id_from_source, resolve_asset, AssetResolution,
    };
    use crate::ports::{
        ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
        SourceRepository,
    };
    use shared_model::{
        ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, LibraryId,
        LibraryRecord, MediaAssetRecord, SourceId, SourceKind, SourceRecord,
    };
    use std::collections::HashMap;
    use std::sync::Mutex;

    #[derive(Default)]
    struct MemoryRepos {
        libraries: Mutex<HashMap<String, LibraryRecord>>,
        sources: Mutex<HashMap<String, SourceRecord>>,
        archives: Mutex<HashMap<String, ArchiveRecord>>,
        archive_entries: Mutex<HashMap<String, Vec<ArchiveEntryRecord>>>,
        assets: Mutex<HashMap<String, MediaAssetRecord>>,
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

    impl AssetRepository for MemoryRepos {
        fn exists(&self, asset_id: &AssetId) -> anyhow::Result<bool> {
            Ok(self.assets.lock().expect("lock").contains_key(&asset_id.0))
        }

        fn upsert(&self, asset: &MediaAssetRecord) -> anyhow::Result<()> {
            self.assets
                .lock()
                .expect("lock")
                .insert(asset.id.0.clone(), asset.clone());
            Ok(())
        }

        fn get(&self, asset_id: &AssetId) -> anyhow::Result<Option<MediaAssetRecord>> {
            Ok(self.assets.lock().expect("lock").get(&asset_id.0).cloned())
        }
    }

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
