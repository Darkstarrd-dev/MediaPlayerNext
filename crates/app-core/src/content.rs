use crate::asset::{
    archive_entry_asset_id_from_entry, file_asset_id_from_source, AssetSnapshotItem,
};
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, ImageItemRepository,
    LibraryRepository, MediaSourceRepository, SourceRepository,
};
use anyhow::{anyhow, Result};
use serde::Serialize;
use serde_json::json;
use shared_model::{
    ArchiveEntryId, ImageItemId, ImageItemRecord, LibraryId, MediaAssetRecord, MediaSourceId,
    MediaSourceKind, MediaSourceRecord, MediaSourceType, SourceId, SourceKind,
};
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentSyncSummary {
    pub library_id: String,
    pub media_source_count: u64,
    pub image_item_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSourceSnapshotItem {
    pub media_source_id: String,
    pub library_id: String,
    pub source_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backing_source_id: Option<String>,
    pub absolute_path: String,
    pub tree_path_json: String,
    pub display_name: String,
    pub item_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_asset_id: Option<String>,
}

#[allow(clippy::too_many_arguments)]
pub fn sync_library_content<L, S, A, E, R, M, I>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    _asset_repository: &R,
    media_source_repository: &M,
    image_item_repository: &I,
    library_id: &LibraryId,
) -> Result<ContentSyncSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
    M: MediaSourceRepository,
    I: ImageItemRepository,
{
    if !library_repository.exists(library_id)? {
        return Err(anyhow!("library not found: {}", library_id.0));
    }

    image_item_repository.delete_by_library(library_id)?;
    media_source_repository.delete_by_library(library_id)?;

    let sources = source_repository.list_by_library(library_id)?;
    let mut grouped_directories: BTreeMap<String, Vec<shared_model::SourceRecord>> =
        BTreeMap::new();
    let mut package_sources = Vec::new();

    for source in sources {
        if !source.exists {
            continue;
        }

        match source.kind {
            SourceKind::Image => {
                let directory_path = parent_path_for(&source.normalized_path);
                grouped_directories
                    .entry(directory_path)
                    .or_default()
                    .push(source);
            }
            SourceKind::Archive => {
                package_sources.push(source);
            }
            _ => {}
        }
    }

    let mut media_source_count = 0_u64;
    let mut image_item_count = 0_u64;
    let now = now_string();

    for (directory_path, mut directory_sources) in grouped_directories {
        directory_sources.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
        if directory_sources.is_empty() {
            continue;
        }

        let media_source_id = directory_media_source_id(library_id, &directory_path);
        let display_name = leaf_name_or_fallback(&directory_path, "Root");
        let tree_path_json = serde_json::to_string(&path_segments(&directory_path))?;

        let mut image_items = Vec::with_capacity(directory_sources.len());
        for (index, source) in directory_sources.iter().enumerate() {
            let asset_id = file_asset_id_from_source(&source.id);
            let image_item = ImageItemRecord {
                id: image_item_id_from_asset(&media_source_id, &asset_id, index as i64),
                media_source_id: media_source_id.clone(),
                asset_id,
                ordinal: index as i64,
                width: None,
                height: None,
                size_bytes: Some(source.size),
                media_locator_json: Some(
                    json!({
                        "kind": "filesystem",
                        "sourceId": source.id.0,
                        "absolutePath": source.normalized_path,
                    })
                    .to_string(),
                ),
                hidden: false,
                last_seen_revision: source.fingerprint.clone(),
                updated_at: now.clone(),
            };
            image_items.push(image_item);
        }

        let media_source = MediaSourceRecord {
            id: media_source_id.clone(),
            library_id: library_id.clone(),
            source_type: MediaSourceType::Directory,
            backing_source_id: None,
            absolute_path: directory_path,
            tree_path_json,
            display_name,
            item_count: image_items.len() as i64,
            cover_asset_id: image_items.first().map(|item| item.asset_id.clone()),
            last_seen_revision: None,
            exists: true,
            updated_at: now.clone(),
        };

        media_source_repository.upsert(&media_source)?;
        image_item_repository.replace_for_media_source(&media_source_id, &image_items)?;
        media_source_count += 1;
        image_item_count += image_items.len() as u64;
    }

    package_sources.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
    for source in package_sources {
        let media_source_id = package_media_source_id(&source.id);
        let tree_path_json = serde_json::to_string(&path_segments(&source.normalized_path))?;
        let display_name = source.file_name.clone();

        let mut image_items = Vec::new();
        if let Some(archive) = archive_repository.get_by_source(&source.id)? {
            let mut entries = archive_entry_repository.list_by_archive(&archive.id)?;
            entries.sort_by(|left, right| {
                left.page_index
                    .cmp(&right.page_index)
                    .then(left.entry_path.cmp(&right.entry_path))
            });

            for entry in entries {
                if !entry.media_kind.eq_ignore_ascii_case("image") {
                    continue;
                }
                let asset_id = archive_entry_asset_id_from_entry(&entry.id);
                let image_item = ImageItemRecord {
                    id: image_item_id_from_asset(&media_source_id, &asset_id, entry.page_index),
                    media_source_id: media_source_id.clone(),
                    asset_id,
                    ordinal: entry.page_index,
                    width: entry.width,
                    height: entry.height,
                    size_bytes: entry.uncompressed_size.or(entry.compressed_size),
                    media_locator_json: Some(
                        json!({
                            "kind": "archive-entry",
                            "sourceId": source.id.0,
                            "archiveEntryId": entry.id.0,
                            "entryPath": entry.entry_path,
                        })
                        .to_string(),
                    ),
                    hidden: false,
                    last_seen_revision: entry.crc32.map(|crc32| format!("crc32:{crc32}")),
                    updated_at: now.clone(),
                };
                image_items.push(image_item);
            }
        }

        let media_source = MediaSourceRecord {
            id: media_source_id.clone(),
            library_id: library_id.clone(),
            source_type: MediaSourceType::Package,
            backing_source_id: Some(source.id.clone()),
            absolute_path: source.normalized_path.clone(),
            tree_path_json,
            display_name,
            item_count: image_items.len() as i64,
            cover_asset_id: image_items.first().map(|item| item.asset_id.clone()),
            last_seen_revision: source
                .fingerprint
                .clone()
                .or_else(|| Some(format!("{}:{}", source.mtime_ms, source.size))),
            exists: true,
            updated_at: now.clone(),
        };

        media_source_repository.upsert(&media_source)?;
        image_item_repository.replace_for_media_source(&media_source_id, &image_items)?;
        media_source_count += 1;
        image_item_count += image_items.len() as u64;
    }

    Ok(ContentSyncSummary {
        library_id: library_id.0.clone(),
        media_source_count,
        image_item_count,
    })
}

pub fn media_source_snapshot_for_library<M>(
    media_source_repository: &M,
    library_id: &LibraryId,
) -> Result<Vec<MediaSourceSnapshotItem>>
where
    M: MediaSourceRepository,
{
    let mut snapshot = media_source_repository
        .list_by_library(library_id)?
        .into_iter()
        .filter(|item| item.exists)
        .map(|item| MediaSourceSnapshotItem {
            media_source_id: item.id.0,
            library_id: item.library_id.0,
            source_type: media_source_type_label(&item.source_type).to_string(),
            backing_source_id: item.backing_source_id.map(|value| value.0),
            absolute_path: item.absolute_path,
            tree_path_json: item.tree_path_json,
            display_name: item.display_name,
            item_count: item.item_count,
            cover_asset_id: item.cover_asset_id.map(|value| value.0),
        })
        .collect::<Vec<_>>();

    snapshot.sort_by(|left, right| left.absolute_path.cmp(&right.absolute_path));
    Ok(snapshot)
}

#[allow(clippy::too_many_arguments)]
pub fn asset_snapshot_for_media_source<M, I, R, A, E>(
    media_source_repository: &M,
    image_item_repository: &I,
    asset_repository: &R,
    archive_repository: &A,
    archive_entry_repository: &E,
    media_source_id: &MediaSourceId,
) -> Result<Vec<AssetSnapshotItem>>
where
    M: MediaSourceRepository,
    I: ImageItemRepository,
    R: AssetRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
{
    let media_source = media_source_repository
        .get(media_source_id)?
        .ok_or_else(|| anyhow!("media source not found: {}", media_source_id.0))?;

    let image_items = image_item_repository.list_by_media_source(media_source_id)?;
    let mut snapshot = Vec::new();

    for image_item in image_items {
        if image_item.hidden {
            continue;
        }
        let Some(asset) = asset_repository.get(&image_item.asset_id)? else {
            continue;
        };

        let item = map_asset_item(
            &media_source,
            &asset,
            archive_repository,
            archive_entry_repository,
        )?;
        snapshot.push(item);
    }

    Ok(snapshot)
}

fn map_asset_item<A, E>(
    media_source: &MediaSourceRecord,
    asset: &MediaAssetRecord,
    archive_repository: &A,
    archive_entry_repository: &E,
) -> Result<AssetSnapshotItem>
where
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
{
    match asset.source_kind {
        MediaSourceKind::File => Ok(AssetSnapshotItem {
            asset_id: asset.id.0.clone(),
            source_kind: "file".to_string(),
            source_ref_id: asset.source_ref_id.clone(),
            library_id: media_source.library_id.0.clone(),
            source_id: asset.source_ref_id.clone(),
            archive_id: None,
            entry_path: None,
            mime: asset.mime.clone(),
        }),
        MediaSourceKind::ArchiveEntry => {
            let archive_entry_id = ArchiveEntryId(asset.source_ref_id.clone());
            let entry = archive_entry_repository
                .get(&archive_entry_id)?
                .ok_or_else(|| anyhow!("archive entry not found: {}", archive_entry_id.0))?;
            let archive = archive_repository
                .get(&entry.archive_id)?
                .ok_or_else(|| anyhow!("archive not found: {}", entry.archive_id.0))?;

            Ok(AssetSnapshotItem {
                asset_id: asset.id.0.clone(),
                source_kind: "archive_entry".to_string(),
                source_ref_id: asset.source_ref_id.clone(),
                library_id: media_source.library_id.0.clone(),
                source_id: archive.source_id.0,
                archive_id: Some(archive.id.0),
                entry_path: Some(entry.entry_path),
                mime: asset.mime.clone(),
            })
        }
        MediaSourceKind::NormalizedFile => Err(anyhow!(
            "normalized_file media asset mapping is not implemented yet"
        )),
    }
}

fn media_source_type_label(source_type: &MediaSourceType) -> &'static str {
    match source_type {
        MediaSourceType::Package => "package",
        MediaSourceType::Directory => "directory",
    }
}

fn parent_path_for(path: &str) -> String {
    let normalized = path.replace('\\', "/");
    match normalized.rsplit_once('/') {
        Some((parent, _)) if !parent.is_empty() => parent.to_string(),
        _ => normalized,
    }
}

fn leaf_name_or_fallback(path: &str, fallback: &str) -> String {
    let normalized = path.replace('\\', "/");
    normalized
        .split('/')
        .filter(|segment| !segment.is_empty())
        .next_back()
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| fallback.to_string())
}

fn path_segments(path: &str) -> Vec<String> {
    path.replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .map(|segment| segment.to_string())
        .collect()
}

fn directory_media_source_id(library_id: &LibraryId, directory_path: &str) -> MediaSourceId {
    MediaSourceId(format!(
        "media_source_{:016x}",
        stable_hash(&format!("directory::{}::{directory_path}", library_id.0))
    ))
}

fn package_media_source_id(source_id: &SourceId) -> MediaSourceId {
    MediaSourceId(format!(
        "media_source_{:016x}",
        stable_hash(&format!("package::{}", source_id.0))
    ))
}

fn image_item_id_from_asset(
    media_source_id: &MediaSourceId,
    asset_id: &shared_model::AssetId,
    ordinal: i64,
) -> ImageItemId {
    ImageItemId(format!(
        "image_item_{:016x}",
        stable_hash(&format!(
            "image_item::{}::{}::{ordinal}",
            media_source_id.0, asset_id.0
        ))
    ))
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
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
