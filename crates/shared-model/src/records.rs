use crate::{
    ArchiveEntryId, ArchiveId, AssetId, ImageItemId, LibraryId, MediaSourceId, MediaSourceKind,
    SourceId, TaskId, TaskKind, TaskState, ThumbnailKey,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Image,
    Video,
    Archive,
    Audio,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LibraryRecord {
    pub id: LibraryId,
    pub root_path: String,
    pub library_type: String,
    pub scan_mode: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceRecord {
    pub id: SourceId,
    pub library_id: LibraryId,
    pub normalized_path: String,
    pub file_name: String,
    pub ext: String,
    pub kind: SourceKind,
    pub size: i64,
    pub mtime_ms: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
    pub exists: bool,
    pub last_seen_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaSourceType {
    Package,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaSourceRecord {
    pub id: MediaSourceId,
    pub library_id: LibraryId,
    pub source_type: MediaSourceType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backing_source_id: Option<SourceId>,
    pub absolute_path: String,
    pub tree_path_json: String,
    pub display_name: String,
    pub item_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_asset_id: Option<AssetId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_revision: Option<String>,
    pub exists: bool,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageItemRecord {
    pub id: ImageItemId,
    pub media_source_id: MediaSourceId,
    pub asset_id: AssetId,
    pub ordinal: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_locator_json: Option<String>,
    pub hidden: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_revision: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveRecord {
    pub id: ArchiveId,
    pub source_id: SourceId,
    pub archive_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub normalized_zip_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub page_count: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cover_entry_id: Option<ArchiveEntryId>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveEntryRecord {
    pub id: ArchiveEntryId,
    pub archive_id: ArchiveId,
    pub entry_path: String,
    pub entry_name: String,
    pub page_index: i64,
    pub media_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compressed_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uncompressed_size: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crc32: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MediaAssetRecord {
    pub id: AssetId,
    pub source_kind: MediaSourceKind,
    pub source_ref_id: String,
    pub mime: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub codec_info_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orientation: Option<i32>,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThumbnailRecord {
    pub thumbnail_key: ThumbnailKey,
    pub asset_id: AssetId,
    pub profile: String,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub disk_path: String,
    pub byte_size: i64,
    pub state: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskRecord {
    pub id: TaskId,
    pub task_type: TaskKind,
    pub state: TaskState,
    pub current: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
}
