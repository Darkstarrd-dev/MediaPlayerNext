use anyhow::Context;
use app_core::archive::{read_archive_entry, resolve_archive_entry_location};
use app_core::playback::resolve_media_asset_path;
use app_core::thumbnail::get_thumbnail;
use media_db::{DatabaseLocation, MediaDatabase};
use shared_model::{AppErrorCode, ArchiveEntryId, AssetId, ThumbnailKey};
use std::path::{Path, PathBuf};
use tauri::http::{header::CONTENT_TYPE, Response, StatusCode, Uri};
use tauri::AppHandle;

use crate::protocol_helpers::{
    content_type_for_media_path, content_type_for_path, content_type_for_thumbnail_format,
    parse_archive_entry_id_from_uri, parse_media_asset_id_from_uri, parse_thumbnail_key_from_uri,
};
use crate::runtime_storage::resolve_database_path;

const ERROR_CODE_HEADER: &str = "x-mediaplayernext-error-code";
const ERROR_RETRIABLE_HEADER: &str = "x-mediaplayernext-error-retriable";

pub(crate) fn protocol_database_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
    resolve_database_path(app)
}

pub(crate) fn thumbnail_protocol_response(
    db_path: &Path,
    uri: &Uri,
) -> anyhow::Result<Response<Vec<u8>>> {
    let thumbnail_key = match parse_thumbnail_key_from_uri(uri) {
        Ok(value) => value,
        Err(_) => {
            return Ok(app_error_response(
                StatusCode::BAD_REQUEST,
                AppErrorCode::InvalidArgument,
                format!("unsupported thumb uri: {uri}"),
                false,
                "text/plain; charset=utf-8",
            ));
        }
    };

    let database = open_protocol_database(db_path)?;
    let repositories = database.repositories();
    let thumbnail = match get_thumbnail(&repositories, &ThumbnailKey(thumbnail_key.clone())) {
        Ok(record) => record,
        Err(_) => {
            return Ok(app_error_response(
                StatusCode::NOT_FOUND,
                AppErrorCode::NotFound,
                format!("thumbnail not found: {thumbnail_key}"),
                false,
                "text/plain; charset=utf-8",
            ));
        }
    };

    let bytes = match std::fs::read(&thumbnail.disk_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(app_error_response(
                StatusCode::NOT_FOUND,
                AppErrorCode::NotFound,
                format!("thumbnail file not found: {}", thumbnail.disk_path),
                false,
                "text/plain; charset=utf-8",
            ));
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            content_type_for_thumbnail_format(&thumbnail.format),
        )
        .body(bytes)
        .context("build thumbnail response")
}

pub(crate) fn media_protocol_response(
    db_path: &Path,
    uri: &Uri,
) -> anyhow::Result<Response<Vec<u8>>> {
    let asset_id = match parse_media_asset_id_from_uri(uri) {
        Ok(value) => value,
        Err(_) => {
            return Ok(app_error_response(
                StatusCode::BAD_REQUEST,
                AppErrorCode::InvalidArgument,
                format!("unsupported media uri: {uri}"),
                false,
                "text/plain; charset=utf-8",
            ));
        }
    };

    let database = open_protocol_database(db_path)?;
    let repositories = database.repositories();
    let (media_path, mime) = match resolve_media_asset_path(
        &repositories,
        &repositories,
        &repositories,
        &repositories,
        &repositories,
        &AssetId(asset_id.clone()),
    ) {
        Ok(value) => value,
        Err(_) => {
            return Ok(app_error_response(
                StatusCode::NOT_FOUND,
                AppErrorCode::NotFound,
                format!("media asset not found: {asset_id}"),
                false,
                "text/plain; charset=utf-8",
            ));
        }
    };

    let bytes = match std::fs::read(&media_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(app_error_response(
                StatusCode::NOT_FOUND,
                AppErrorCode::NotFound,
                format!("media file not found: {}", media_path.display()),
                false,
                "text/plain; charset=utf-8",
            ));
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            content_type_for_media_path(&media_path, &mime),
        )
        .body(bytes)
        .context("build media response")
}

pub(crate) fn archive_protocol_response(
    db_path: &Path,
    uri: &Uri,
) -> anyhow::Result<Response<Vec<u8>>> {
    let archive_entry_id = match parse_archive_entry_id_from_uri(uri) {
        Ok(value) => value,
        Err(_) => {
            return Ok(app_error_response(
                StatusCode::BAD_REQUEST,
                AppErrorCode::InvalidArgument,
                format!("unsupported archive uri: {uri}"),
                false,
                "text/plain; charset=utf-8",
            ));
        }
    };

    let database = open_protocol_database(db_path)?;
    let repositories = database.repositories();
    let location = match resolve_archive_entry_location(
        &repositories,
        &repositories,
        &repositories,
        &repositories,
        &ArchiveEntryId(archive_entry_id.clone()),
    ) {
        Ok(value) => value,
        Err(_) => {
            return Ok(app_error_response(
                StatusCode::NOT_FOUND,
                AppErrorCode::NotFound,
                format!("archive entry not found: {archive_entry_id}"),
                false,
                "text/plain; charset=utf-8",
            ));
        }
    };

    let bytes = match read_archive_entry(Path::new(&location.archive_path), &location.entry_path) {
        Ok(value) => value,
        Err(_) => {
            return Ok(app_error_response(
                StatusCode::NOT_FOUND,
                AppErrorCode::NotFound,
                format!(
                    "archive entry file not found: {}::{}",
                    location.archive_path, location.entry_path
                ),
                false,
                "text/plain; charset=utf-8",
            ));
        }
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            content_type_for_path(Path::new(&location.entry_path), &location.media_kind),
        )
        .body(bytes)
        .context("build archive response")
}

pub(crate) fn app_error_response(
    status: StatusCode,
    code: AppErrorCode,
    message: String,
    retriable: bool,
    content_type: &str,
) -> Response<Vec<u8>> {
    let payload = message.into_bytes();
    match Response::builder()
        .status(status)
        .header(CONTENT_TYPE, content_type)
        .header(ERROR_CODE_HEADER, app_error_code_name(&code))
        .header(
            ERROR_RETRIABLE_HEADER,
            if retriable { "true" } else { "false" },
        )
        .body(payload)
    {
        Ok(response) => response,
        Err(_) => Response::new(Vec::new()),
    }
}

fn app_error_code_name(code: &AppErrorCode) -> &'static str {
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

fn open_protocol_database(db_path: &Path) -> anyhow::Result<MediaDatabase> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("create protocol database parent dir: {}", parent.display())
        })?;
    }

    MediaDatabase::open(DatabaseLocation::File(db_path))
}
