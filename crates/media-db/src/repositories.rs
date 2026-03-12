use anyhow::Result;
use app_core::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, ImageItemRepository,
    LibraryRepository, MediaSourceRepository, SourceRepository, TaskRepository,
    ThumbnailRepository,
};
use rusqlite::{named_params, Connection, OptionalExtension};
use shared_model::{
    AppErrorCode, ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId,
    ImageItemId, ImageItemRecord, LibraryId, LibraryRecord, MediaAssetRecord, MediaSourceId,
    MediaSourceKind, MediaSourceRecord, MediaSourceType, SourceId, SourceKind, SourceRecord,
    TaskId, TaskKind, TaskRecord, TaskState, ThumbnailKey, ThumbnailRecord,
};

pub struct SqliteRepositories<'a> {
    connection: &'a Connection,
}

impl<'a> SqliteRepositories<'a> {
    pub fn new(connection: &'a Connection) -> Self {
        Self { connection }
    }
}

include!("repositories/library_impl.rs");
include!("repositories/source_impl.rs");
include!("repositories/media_source_impl.rs");
include!("repositories/image_item_impl.rs");
include!("repositories/archive_impl.rs");
include!("repositories/archive_entry_impl.rs");
include!("repositories/asset_impl.rs");
include!("repositories/task_impl.rs");
include!("repositories/thumbnail_impl.rs");

fn source_kind_to_db(kind: &SourceKind) -> &'static str {
    match kind {
        SourceKind::Image => "image",
        SourceKind::Video => "video",
        SourceKind::Archive => "archive",
        SourceKind::Audio => "audio",
        SourceKind::Other => "other",
    }
}

fn source_kind_from_db(value: &str) -> SourceKind {
    match value {
        "image" => SourceKind::Image,
        "video" => SourceKind::Video,
        "archive" => SourceKind::Archive,
        "audio" => SourceKind::Audio,
        _ => SourceKind::Other,
    }
}

fn media_source_type_to_db(source_type: &MediaSourceType) -> &'static str {
    match source_type {
        MediaSourceType::Package => "package",
        MediaSourceType::Directory => "directory",
    }
}

fn media_source_type_from_db(value: &str) -> MediaSourceType {
    match value {
        "package" => MediaSourceType::Package,
        "directory" => MediaSourceType::Directory,
        _ => MediaSourceType::Directory,
    }
}

fn media_source_kind_to_db(kind: &MediaSourceKind) -> &'static str {
    match kind {
        MediaSourceKind::File => "file",
        MediaSourceKind::ArchiveEntry => "archive_entry",
        MediaSourceKind::NormalizedFile => "normalized_file",
    }
}

fn media_source_kind_from_db(value: &str) -> MediaSourceKind {
    match value {
        "file" => MediaSourceKind::File,
        "archive_entry" => MediaSourceKind::ArchiveEntry,
        "normalized_file" => MediaSourceKind::NormalizedFile,
        _ => MediaSourceKind::File,
    }
}

fn task_kind_to_db(kind: &TaskKind) -> &'static str {
    match kind {
        TaskKind::Scan => "scan",
        TaskKind::Ingest => "ingest",
        TaskKind::Normalize => "normalize",
        TaskKind::Thumbnail => "thumbnail",
        TaskKind::Ffmpeg => "ffmpeg",
        TaskKind::Subtitle => "subtitle",
    }
}

fn task_kind_from_db(value: &str) -> TaskKind {
    match value {
        "scan" => TaskKind::Scan,
        "ingest" => TaskKind::Ingest,
        "normalize" => TaskKind::Normalize,
        "thumbnail" => TaskKind::Thumbnail,
        "ffmpeg" => TaskKind::Ffmpeg,
        "subtitle" => TaskKind::Subtitle,
        _ => TaskKind::Scan,
    }
}

fn task_state_to_db(state: &TaskState) -> &'static str {
    match state {
        TaskState::Queued => "queued",
        TaskState::Running => "running",
        TaskState::Completed => "completed",
        TaskState::Failed => "failed",
        TaskState::Cancelled => "cancelled",
    }
}

fn task_state_from_db(value: &str) -> TaskState {
    match value {
        "queued" => TaskState::Queued,
        "running" => TaskState::Running,
        "completed" => TaskState::Completed,
        "failed" => TaskState::Failed,
        "cancelled" => TaskState::Cancelled,
        _ => TaskState::Failed,
    }
}

#[allow(dead_code)]
fn _error_code_to_db(code: &AppErrorCode) -> &'static str {
    match code {
        AppErrorCode::NotFound => "NOT_FOUND",
        AppErrorCode::AlreadyExists => "ALREADY_EXISTS",
        AppErrorCode::UnsupportedFormat => "UNSUPPORTED_FORMAT",
        AppErrorCode::PermissionDenied => "PERMISSION_DENIED",
        AppErrorCode::InvalidArgument => "INVALID_ARGUMENT",
        AppErrorCode::IoError => "IO_ERROR",
        AppErrorCode::DbError => "DB_ERROR",
        AppErrorCode::ExternalToolError => "EXTERNAL_TOOL_ERROR",
        AppErrorCode::Cancelled => "CANCELLED",
        AppErrorCode::Timeout => "TIMEOUT",
        AppErrorCode::InternalError => "INTERNAL_ERROR",
    }
}
