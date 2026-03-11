mod runtime_check;
pub mod subtitle_sidecar;

use anyhow::{Context, Error};
use app_core::archive::read_archive_entry;
use app_core::archive::{
    archive_snapshot, index_library_archives, normalize_archive_source, normalize_archive_status,
    resolve_archive_entry_location, ArchiveNormalizeSummary,
};
use app_core::asset::{
    asset_snapshot_for_library, ensure_media_assets_for_library, resolve_asset, AssetResolution,
};
use app_core::library::{get_library, list_libraries, remove_library};
use app_core::playback::{
    open_playback_session, playback_seek, playback_status, resolve_media_asset_path,
};
use app_core::scan::{
    register_library, resume_scan, run_scan, scan_stats, scan_task_id_for_library, ScanRunSummary,
    ScanStatsSummary,
};
use app_core::subtitle_host::{
    subtitle_get_progress, subtitle_health, subtitle_ping, subtitle_start_session,
    subtitle_stop_session,
};
use app_core::thumbnail::{ensure_thumbnail_for_asset, get_thumbnail, parse_thumbnail_profile};
use media_db::{DatabaseLocation, MediaDatabase};
use runtime_check::{run_runtime_smoke_check, RuntimeSmokeCheckResult};
use serde::Deserialize;
use serde_json::json;
use shared_model::{
    AppError, AppErrorCode, ArchiveEntryId, AssetId, LibraryId, LibraryRecord, PlaybackSessionId,
    PlaybackSessionSummary, SourceId, SubtitleSessionId, TaskProgress, TaskRecord, ThumbnailKey,
};
use std::env;
use std::path::{Path, PathBuf};
use tauri::http::{header::CONTENT_TYPE, Response, StatusCode, Uri};
use tauri::{AppHandle, Manager};

const ERROR_CODE_HEADER: &str = "x-mediaplayernext-error-code";
const ERROR_RETRIABLE_HEADER: &str = "x-mediaplayernext-error-retriable";
type CommandResult<T> = Result<T, AppError>;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimePathsConfig {
    mpv: Option<String>,
    sevenz: Option<String>,
}

#[derive(Debug, Clone)]
struct RuntimePaths {
    mpv_path: PathBuf,
    sevenz_path: PathBuf,
}

struct CommandEnvironment {
    database: MediaDatabase,
    runtime_paths: RuntimePaths,
    thumbnail_cache_root: PathBuf,
    normalize_root: PathBuf,
    playback_sessions_root: PathBuf,
}

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! MediaPlayerNext host is ready.")
}

#[tauri::command]
fn runtime_smoke_check(
    ffmpeg_path: String,
    ffprobe_path: String,
    mpv_path: String,
) -> CommandResult<RuntimeSmokeCheckResult> {
    run_runtime_smoke_check(&ffmpeg_path, &ffprobe_path, &mpv_path)
        .map_err(|error| map_runtime_command_error("runtime_smoke_check", error))
}

#[tauri::command]
fn subtitle_ping_command(
    app: tauri::AppHandle,
) -> CommandResult<shared_model::SubtitleHostSummary> {
    let host = subtitle_sidecar::tauri_subtitle_host(&app)
        .map_err(|error| map_subtitle_command_error("subtitle_ping_command", error))?;
    subtitle_ping(&host).map_err(|error| map_subtitle_command_error("subtitle_ping_command", error))
}

#[tauri::command]
fn subtitle_health_command(
    app: tauri::AppHandle,
) -> CommandResult<shared_model::SubtitleHostSummary> {
    let host = subtitle_sidecar::tauri_subtitle_host(&app)
        .map_err(|error| map_subtitle_command_error("subtitle_health_command", error))?;
    subtitle_health(&host)
        .map_err(|error| map_subtitle_command_error("subtitle_health_command", error))
}

#[tauri::command]
fn subtitle_start_session_command(
    app: tauri::AppHandle,
    asset_id: Option<String>,
) -> CommandResult<shared_model::SubtitleSessionSummary> {
    let host = subtitle_sidecar::tauri_subtitle_host(&app)
        .map_err(|error| map_subtitle_command_error("subtitle_start_session_command", error))?;
    let asset_id = asset_id.map(AssetId);
    subtitle_start_session(&host, asset_id.as_ref())
        .map_err(|error| map_subtitle_command_error("subtitle_start_session_command", error))
}

#[tauri::command]
fn subtitle_stop_session_command(
    app: tauri::AppHandle,
    session_id: String,
) -> CommandResult<shared_model::SubtitleSessionSummary> {
    let host = subtitle_sidecar::tauri_subtitle_host(&app)
        .map_err(|error| map_subtitle_command_error("subtitle_stop_session_command", error))?;
    subtitle_stop_session(&host, &SubtitleSessionId(session_id))
        .map_err(|error| map_subtitle_command_error("subtitle_stop_session_command", error))
}

#[tauri::command]
fn subtitle_get_progress_command(
    app: tauri::AppHandle,
    session_id: String,
) -> CommandResult<shared_model::SubtitleProgressEvent> {
    let host = subtitle_sidecar::tauri_subtitle_host(&app)
        .map_err(|error| map_subtitle_command_error("subtitle_get_progress_command", error))?;
    subtitle_get_progress(&host, &SubtitleSessionId(session_id))
        .map_err(|error| map_subtitle_command_error("subtitle_get_progress_command", error))
}

#[tauri::command]
fn library_list_command(app: tauri::AppHandle) -> CommandResult<Vec<LibraryRecord>> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        list_libraries(&repositories)
    })
    .map_err(|error| map_backend_command_error("library_list_command", "library", error))
}

#[tauri::command]
fn library_add_command(app: tauri::AppHandle, root_path: String) -> CommandResult<LibraryRecord> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let library_id = register_library(&repositories, Path::new(&root_path))?;
        get_library(&repositories, &library_id)
    })
    .map_err(|error| map_backend_command_error("library_add_command", "library", error))
}

#[tauri::command]
fn library_get_command(app: tauri::AppHandle, library_id: String) -> CommandResult<LibraryRecord> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        get_library(&repositories, &LibraryId(library_id))
    })
    .map_err(|error| map_backend_command_error("library_get_command", "library", error))
}

#[tauri::command]
fn library_remove_command(app: tauri::AppHandle, library_id: String) -> CommandResult<()> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        remove_library(&repositories, &LibraryId(library_id))
    })
    .map_err(|error| map_backend_command_error("library_remove_command", "library", error))
}

#[tauri::command]
fn scan_start_command(app: tauri::AppHandle, library_id: String) -> CommandResult<ScanRunSummary> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        run_scan(
            &repositories,
            &repositories,
            &repositories,
            &LibraryId(library_id),
        )
    })
    .map_err(|error| map_backend_command_error("scan_start_command", "scan", error))
}

#[tauri::command]
fn scan_resume_command(app: tauri::AppHandle, library_id: String) -> CommandResult<ScanRunSummary> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        resume_scan(
            &repositories,
            &repositories,
            &repositories,
            &LibraryId(library_id),
        )
    })
    .map_err(|error| map_backend_command_error("scan_resume_command", "scan", error))
}

#[tauri::command]
fn scan_stats_command(
    app: tauri::AppHandle,
    library_id: String,
) -> CommandResult<ScanStatsSummary> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        scan_stats(&repositories, &repositories, &LibraryId(library_id))
    })
    .map_err(|error| map_backend_command_error("scan_stats_command", "scan", error))
}

#[tauri::command]
fn scan_snapshot_command(app: tauri::AppHandle, library_id: String) -> CommandResult<TaskProgress> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let task_id = scan_task_id_for_library(&LibraryId(library_id));
        let task = app_core::ports::TaskRepository::get(&repositories, &task_id)?
            .ok_or_else(|| anyhow::anyhow!("task not found: {}", task_id.0))?;
        Ok(task_progress_from_record(task))
    })
    .map_err(|error| map_backend_command_error("scan_snapshot_command", "scan", error))
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemListEntryPayload {
    asset_id: String,
    source_kind: String,
    source_ref_id: String,
    library_id: String,
    source_id: String,
    archive_id: Option<String>,
    entry_path: Option<String>,
    mime: String,
    thumbnail_key: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct ItemDetailPayload {
    asset_id: String,
    source_kind: String,
    mime: String,
    library_id: String,
    source_id: String,
    file_path: Option<String>,
    archive_id: Option<String>,
    archive_entry_id: Option<String>,
    archive_path: Option<String>,
    entry_path: Option<String>,
}

#[tauri::command]
fn items_list_command(
    app: tauri::AppHandle,
    library_id: String,
    page: Option<u32>,
    page_size: Option<u32>,
) -> CommandResult<Vec<ItemListEntryPayload>> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let library_id = LibraryId(library_id);
        let _ = index_library_archives(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &library_id,
        )?;
        let _ = ensure_media_assets_for_library(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &library_id,
        )?;

        let snapshot = asset_snapshot_for_library(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &library_id,
        )?;
        let offset = page
            .zip(page_size)
            .map(|(page_value, page_size_value)| {
                page_value.saturating_sub(1) as usize * page_size_value as usize
            })
            .unwrap_or(0);
        let limit = page_size
            .map(|value| value as usize)
            .unwrap_or(snapshot.len());

        Ok(snapshot
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|item| ItemListEntryPayload {
                asset_id: item.asset_id,
                source_kind: item.source_kind,
                source_ref_id: item.source_ref_id,
                library_id: item.library_id,
                source_id: item.source_id,
                archive_id: item.archive_id,
                entry_path: item.entry_path,
                mime: item.mime,
                thumbnail_key: None,
            })
            .collect())
    })
    .map_err(|error| map_backend_command_error("items_list_command", "items", error))
}

#[tauri::command]
fn item_detail_command(
    app: tauri::AppHandle,
    asset_id: String,
) -> CommandResult<ItemDetailPayload> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let asset_id = AssetId(asset_id);
        let asset = app_core::ports::AssetRepository::get(&repositories, &asset_id)?
            .ok_or_else(|| anyhow::anyhow!("asset not found: {}", asset_id.0))?;
        let resolution = resolve_asset(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &asset_id,
        )?;

        Ok(match resolution {
            AssetResolution::File(item) => ItemDetailPayload {
                asset_id: item.asset_id,
                source_kind: "file".to_string(),
                mime: asset.mime,
                library_id: item.library_id,
                source_id: item.source_id,
                file_path: Some(item.file_path),
                archive_id: None,
                archive_entry_id: None,
                archive_path: None,
                entry_path: None,
            },
            AssetResolution::ArchiveEntry(item) => ItemDetailPayload {
                asset_id: item.asset_id,
                source_kind: "archive_entry".to_string(),
                mime: asset.mime,
                library_id: item.library_id,
                source_id: item.source_id,
                file_path: None,
                archive_id: Some(item.archive_id),
                archive_entry_id: Some(item.archive_entry_id),
                archive_path: Some(item.archive_path),
                entry_path: Some(item.entry_path),
            },
        })
    })
    .map_err(|error| map_backend_command_error("item_detail_command", "items", error))
}

#[tauri::command]
fn archive_entries_command(
    app: tauri::AppHandle,
    source_id: String,
) -> CommandResult<Vec<shared_model::ArchiveEntryRecord>> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let source_id = SourceId(source_id);
        let snapshot = archive_snapshot(&repositories, &repositories, &source_id)?;
        Ok(snapshot.entries)
    })
    .map_err(|error| map_backend_command_error("archive_entries_command", "archive", error))
}

#[tauri::command]
fn archive_entry_detail_command(
    app: tauri::AppHandle,
    archive_entry_id: String,
) -> CommandResult<app_core::archive::ResolvedArchiveEntryLocation> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        resolve_archive_entry_location(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &ArchiveEntryId(archive_entry_id),
        )
    })
    .map_err(|error| map_backend_command_error("archive_entry_detail_command", "archive", error))
}

#[tauri::command]
fn archive_normalize_command(
    app: tauri::AppHandle,
    source_id: String,
) -> CommandResult<ArchiveNormalizeSummary> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        normalize_archive_source(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &environment.runtime_paths.sevenz_path,
            &environment.normalize_root,
            &SourceId(source_id),
        )
    })
    .map_err(|error| map_backend_command_error("archive_normalize_command", "archive", error))
}

#[tauri::command]
fn archive_normalize_status_command(
    app: tauri::AppHandle,
    task_id: String,
) -> CommandResult<TaskProgress> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        let task = normalize_archive_status(&repositories, &shared_model::TaskId(task_id))?;
        Ok(task_progress_from_record(task))
    })
    .map_err(|error| {
        map_backend_command_error("archive_normalize_status_command", "archive", error)
    })
}

#[tauri::command]
fn thumbnail_ensure_command(
    app: tauri::AppHandle,
    asset_id: String,
    profile: String,
) -> CommandResult<app_core::thumbnail::ThumbnailEnsureSummary> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        ensure_thumbnail_for_asset(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &environment.thumbnail_cache_root,
            &AssetId(asset_id),
            parse_thumbnail_profile(&profile)?,
        )
    })
    .map_err(|error| map_backend_command_error("thumbnail_ensure_command", "thumbnail", error))
}

#[tauri::command]
fn playback_open_command(
    app: tauri::AppHandle,
    asset_id: String,
) -> CommandResult<PlaybackSessionSummary> {
    with_command_environment(&app, |environment| {
        let repositories = environment.database.repositories();
        open_playback_session(
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &repositories,
            &environment.playback_sessions_root,
            &environment.runtime_paths.mpv_path,
            &AssetId(asset_id),
        )
    })
    .map_err(|error| map_backend_command_error("playback_open_command", "playback", error))
}

#[tauri::command]
fn playback_status_command(
    app: tauri::AppHandle,
    session_id: String,
) -> CommandResult<PlaybackSessionSummary> {
    with_command_environment(&app, |environment| {
        playback_status(
            &environment.playback_sessions_root,
            &PlaybackSessionId(session_id),
        )
    })
    .map_err(|error| map_backend_command_error("playback_status_command", "playback", error))
}

#[tauri::command]
fn playback_seek_command(
    app: tauri::AppHandle,
    session_id: String,
    position_ms: i64,
) -> CommandResult<PlaybackSessionSummary> {
    with_command_environment(&app, |environment| {
        playback_seek(
            &environment.playback_sessions_root,
            &PlaybackSessionId(session_id),
            position_ms,
        )
    })
    .map_err(|error| map_backend_command_error("playback_seek_command", "playback", error))
}

pub fn runtime_smoke_check_entry() -> anyhow::Result<()> {
    let mut ffmpeg_path: Option<String> = None;
    let mut ffprobe_path: Option<String> = None;
    let mut mpv_path: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ffmpeg-path" => {
                ffmpeg_path = args.next();
            }
            "--ffprobe-path" => {
                ffprobe_path = args.next();
            }
            "--mpv-path" => {
                mpv_path = args.next();
            }
            _ => {}
        }
    }

    let ffmpeg_path = ffmpeg_path.ok_or_else(|| anyhow::anyhow!("missing --ffmpeg-path"))?;
    let ffprobe_path = ffprobe_path.ok_or_else(|| anyhow::anyhow!("missing --ffprobe-path"))?;
    let mpv_path = mpv_path.ok_or_else(|| anyhow::anyhow!("missing --mpv-path"))?;
    let result = run_runtime_smoke_check(&ffmpeg_path, &ffprobe_path, &mpv_path)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            runtime_smoke_check,
            subtitle_ping_command,
            subtitle_health_command,
            subtitle_start_session_command,
            subtitle_stop_session_command,
            subtitle_get_progress_command,
            library_list_command,
            library_add_command,
            library_get_command,
            library_remove_command,
            scan_start_command,
            scan_resume_command,
            scan_stats_command,
            scan_snapshot_command,
            items_list_command,
            item_detail_command,
            archive_entries_command,
            archive_entry_detail_command,
            archive_normalize_command,
            archive_normalize_status_command,
            thumbnail_ensure_command,
            playback_open_command,
            playback_status_command,
            playback_seek_command
        ])
        .register_uri_scheme_protocol("thumb", |app, request| {
            match protocol_database_path(app.app_handle())
                .and_then(|db_path| thumbnail_protocol_response(&db_path, request.uri()))
            {
                Ok(response) => response,
                Err(error) => app_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppErrorCode::InternalError,
                    error.to_string(),
                    false,
                    "text/plain; charset=utf-8",
                ),
            }
        })
        .register_uri_scheme_protocol("media", |app, request| {
            match protocol_database_path(app.app_handle())
                .and_then(|db_path| media_protocol_response(&db_path, request.uri()))
            {
                Ok(response) => response,
                Err(error) => app_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppErrorCode::InternalError,
                    error.to_string(),
                    false,
                    "text/plain; charset=utf-8",
                ),
            }
        })
        .register_uri_scheme_protocol("archive", |app, request| {
            match protocol_database_path(app.app_handle())
                .and_then(|db_path| archive_protocol_response(&db_path, request.uri()))
            {
                Ok(response) => response,
                Err(error) => app_error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    AppErrorCode::InternalError,
                    error.to_string(),
                    false,
                    "text/plain; charset=utf-8",
                ),
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn thumbnail_protocol_response(db_path: &Path, uri: &Uri) -> anyhow::Result<Response<Vec<u8>>> {
    let thumbnail_key = match parse_thumbnail_key_from_uri(uri) {
        Ok(thumbnail_key) => thumbnail_key,
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
        Ok(bytes) => bytes,
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

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            content_type_for_thumbnail_format(&thumbnail.format),
        )
        .body(bytes)
        .expect("thumbnail response should build"))
}

fn media_protocol_response(db_path: &Path, uri: &Uri) -> anyhow::Result<Response<Vec<u8>>> {
    let asset_id = match parse_media_asset_id_from_uri(uri) {
        Ok(asset_id) => asset_id,
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
        Ok(result) => result,
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
        Ok(bytes) => bytes,
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

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            content_type_for_media_path(&media_path, &mime),
        )
        .body(bytes)
        .expect("media response should build"))
}

fn archive_protocol_response(db_path: &Path, uri: &Uri) -> anyhow::Result<Response<Vec<u8>>> {
    let archive_entry_id = match parse_archive_entry_id_from_uri(uri) {
        Ok(archive_entry_id) => archive_entry_id,
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
        Ok(location) => location,
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
        Ok(bytes) => bytes,
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

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            content_type_for_path(Path::new(&location.entry_path), &location.media_kind),
        )
        .body(bytes)
        .expect("archive response should build"))
}

fn parse_thumbnail_key_from_uri(uri: &Uri) -> anyhow::Result<String> {
    let path_segments = uri
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match path_segments.as_slice() {
        ["cache", thumbnail_key] => Ok((*thumbnail_key).to_string()),
        [thumbnail_key] if uri.host() == Some("cache") => Ok((*thumbnail_key).to_string()),
        _ => Err(anyhow::anyhow!("unsupported thumb uri: {uri}")),
    }
}

fn parse_media_asset_id_from_uri(uri: &Uri) -> anyhow::Result<String> {
    let path_segments = uri
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match path_segments.as_slice() {
        ["asset", asset_id] => Ok((*asset_id).to_string()),
        [asset_id] if uri.host() == Some("asset") => Ok((*asset_id).to_string()),
        _ => Err(anyhow::anyhow!("unsupported media uri: {uri}")),
    }
}

fn parse_archive_entry_id_from_uri(uri: &Uri) -> anyhow::Result<String> {
    let path_segments = uri
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match path_segments.as_slice() {
        ["entry", archive_entry_id] => Ok((*archive_entry_id).to_string()),
        [archive_entry_id] if uri.host() == Some("entry") => Ok((*archive_entry_id).to_string()),
        _ => Err(anyhow::anyhow!("unsupported archive uri: {uri}")),
    }
}

fn content_type_for_thumbnail_format(format: &str) -> &'static str {
    match format {
        "webp" => "image/webp",
        "png" => "image/png",
        "jpeg" | "jpg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

fn content_type_for_media_path(path: &Path, fallback_mime: &str) -> &'static str {
    match fallback_mime {
        "video/mp4" => "video/mp4",
        "video/webm" => "video/webm",
        "video/x-matroska" => "video/x-matroska",
        "audio/mpeg" => "audio/mpeg",
        "audio/flac" => "audio/flac",
        "audio/wav" => "audio/wav",
        _ => content_type_for_path(path, ""),
    }
}

fn content_type_for_path(path: &Path, media_kind: &str) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        _ if media_kind.eq_ignore_ascii_case("image") => "image/*",
        _ if media_kind.eq_ignore_ascii_case("video") => "video/*",
        _ if media_kind.eq_ignore_ascii_case("audio") => "audio/*",
        _ => "application/octet-stream",
    }
}

fn app_error_response(
    status: StatusCode,
    code: AppErrorCode,
    message: String,
    retriable: bool,
    content_type: &str,
) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, content_type)
        .header(ERROR_CODE_HEADER, app_error_code_name(&code))
        .header(
            ERROR_RETRIABLE_HEADER,
            if retriable { "true" } else { "false" },
        )
        .body(message.into_bytes())
        .expect("app error response should build")
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

fn map_runtime_command_error(command: &str, error: Error) -> AppError {
    let message = error.to_string();
    let code = if message.contains("runtime binary not found") {
        AppErrorCode::NotFound
    } else if message.contains("open sqlite") || message.contains("query sqlite version") {
        AppErrorCode::DbError
    } else if message.contains("runtime binary") || message.contains("spawn runtime binary") {
        AppErrorCode::ExternalToolError
    } else {
        AppErrorCode::InternalError
    };

    AppError {
        code,
        message,
        retriable: false,
        details: Some(json!({
            "surface": "tauri-command",
            "command": command,
            "domain": "runtime-smoke-check"
        })),
    }
}

fn map_subtitle_command_error(command: &str, error: Error) -> AppError {
    let message = error.to_string();
    let retriable = message.contains("timed out")
        || message.contains(", retriable")
        || message.contains("[TIMEOUT]");
    let code = if let Some(code) = parse_embedded_app_error_code(&message) {
        code
    } else if message.contains("timed out") {
        AppErrorCode::Timeout
    } else if message.contains("response payload missing")
        || message.contains("parse subtitle sidecar response")
        || message.contains("returned empty stdout")
        || message.contains("stdout is not valid utf-8")
        || message.contains("response id mismatch")
        || message.contains("subtitle sidecar exited with status")
        || message.contains("spawn subtitle sidecar")
    {
        AppErrorCode::ExternalToolError
    } else if message.contains("subtitle sidecar entry missing") {
        AppErrorCode::NotFound
    } else if message.contains("not found") || message.contains("missing") {
        AppErrorCode::NotFound
    } else if message.contains("unsupported") {
        AppErrorCode::InvalidArgument
    } else if message.contains("subtitle sidecar") || message.contains("spawn") {
        AppErrorCode::ExternalToolError
    } else {
        AppErrorCode::InternalError
    };

    AppError {
        code,
        message,
        retriable,
        details: Some(json!({
            "surface": "tauri-command",
            "command": command,
            "domain": "subtitle-sidecar"
        })),
    }
}

fn parse_embedded_app_error_code(message: &str) -> Option<AppErrorCode> {
    [
        ("[NOT_FOUND]", AppErrorCode::NotFound),
        ("[ALREADY_EXISTS]", AppErrorCode::AlreadyExists),
        ("[UNSUPPORTED_FORMAT]", AppErrorCode::UnsupportedFormat),
        ("[PERMISSION_DENIED]", AppErrorCode::PermissionDenied),
        ("[INVALID_ARGUMENT]", AppErrorCode::InvalidArgument),
        ("[IO_ERROR]", AppErrorCode::IoError),
        ("[DB_ERROR]", AppErrorCode::DbError),
        ("[EXTERNAL_TOOL_ERROR]", AppErrorCode::ExternalToolError),
        ("[CANCELLED]", AppErrorCode::Cancelled),
        ("[TIMEOUT]", AppErrorCode::Timeout),
        ("[INTERNAL_ERROR]", AppErrorCode::InternalError),
    ]
    .into_iter()
    .find_map(|(needle, code)| message.contains(needle).then_some(code))
}

fn with_command_environment<T, F>(app: &AppHandle, run: F) -> anyhow::Result<T>
where
    F: FnOnce(&CommandEnvironment) -> anyhow::Result<T>,
{
    let environment = command_environment(app)?;
    run(&environment)
}

fn command_environment(app: &AppHandle) -> anyhow::Result<CommandEnvironment> {
    let config_path = workspace_root().join("config").join("local.paths.json");
    let db_path = command_database_path(app)?;
    let thumbnail_cache_root = command_cache_path(
        app,
        "MPNEXT_BACKEND_THUMB_CACHE_ROOT",
        development_thumbnail_cache_root(),
        "thumbs",
    )?;
    let normalize_root = command_cache_path(
        app,
        "MPNEXT_BACKEND_NORMALIZE_ROOT",
        development_normalize_root(),
        "normalized",
    )?;
    let playback_sessions_root = command_cache_path(
        app,
        "MPNEXT_BACKEND_PLAYBACK_SESSIONS_ROOT",
        development_playback_sessions_root(),
        "playback/sessions",
    )?;

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create command database parent dir: {}", parent.display()))?;
    }
    std::fs::create_dir_all(&thumbnail_cache_root).with_context(|| {
        format!(
            "create thumbnail cache root for commands: {}",
            thumbnail_cache_root.display()
        )
    })?;
    std::fs::create_dir_all(&normalize_root).with_context(|| {
        format!(
            "create archive normalize root for commands: {}",
            normalize_root.display()
        )
    })?;
    std::fs::create_dir_all(&playback_sessions_root).with_context(|| {
        format!(
            "create playback sessions root for commands: {}",
            playback_sessions_root.display()
        )
    })?;

    Ok(CommandEnvironment {
        database: MediaDatabase::open(DatabaseLocation::File(&db_path))?,
        runtime_paths: load_runtime_paths(&config_path),
        thumbnail_cache_root,
        normalize_root,
        playback_sessions_root,
    })
}

fn map_backend_command_error(command: &str, domain: &str, error: Error) -> AppError {
    let message = error.to_string();
    let lower = message.to_ascii_lowercase();
    let retriable = lower.contains("timed out") || lower.contains("timeout");
    let code = if let Some(code) = parse_embedded_app_error_code(&message) {
        code
    } else if lower.contains("already exists") {
        AppErrorCode::AlreadyExists
    } else if lower.contains("not found") || lower.contains("missing") {
        AppErrorCode::NotFound
    } else if lower.contains("permission denied") || lower.contains("access is denied") {
        AppErrorCode::PermissionDenied
    } else if lower.contains("unsupported") || lower.contains("invalid") {
        AppErrorCode::InvalidArgument
    } else if lower.contains("sqlite") || lower.contains("database") || lower.contains("db") {
        AppErrorCode::DbError
    } else if lower.contains("spawn ")
        || lower.contains("7z")
        || lower.contains("sevenz")
        || lower.contains("ffmpeg")
        || lower.contains("ffprobe")
        || lower.contains("mpv")
    {
        AppErrorCode::ExternalToolError
    } else if lower.contains("io error")
        || lower.contains("failed to read")
        || lower.contains("failed to write")
        || lower.contains("open ")
        || lower.contains("create ")
    {
        AppErrorCode::IoError
    } else if retriable {
        AppErrorCode::Timeout
    } else {
        AppErrorCode::InternalError
    };

    AppError {
        code,
        message,
        retriable,
        details: Some(json!({
            "surface": "tauri-command",
            "command": command,
            "domain": domain,
        })),
    }
}

fn task_progress_from_record(task: TaskRecord) -> TaskProgress {
    TaskProgress {
        task_id: task.id,
        task_type: task.task_type,
        state: task.state,
        current: task.current,
        total: task.total,
        message: task.message,
        error_code: task.error_code,
    }
}

fn development_database_path() -> PathBuf {
    workspace_root().join("data").join("mediaplayernext-dev.db")
}

fn development_thumbnail_cache_root() -> PathBuf {
    workspace_root().join("data").join("cache").join("thumbs")
}

fn development_normalize_root() -> PathBuf {
    workspace_root()
        .join("data")
        .join("cache")
        .join("normalized")
}

fn development_playback_sessions_root() -> PathBuf {
    workspace_root()
        .join("data")
        .join("cache")
        .join("playback")
        .join("sessions")
}

fn command_database_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
    if let Some(path) = env::var_os("MPNEXT_BACKEND_DB_PATH") {
        return Ok(PathBuf::from(path));
    }

    if tauri::is_dev() {
        return Ok(development_database_path());
    }

    Ok(app
        .path()
        .app_local_data_dir()
        .context("resolve command app local data dir")?
        .join("mediaplayernext.db"))
}

fn command_cache_path(
    app: &AppHandle,
    env_name: &str,
    development_default: PathBuf,
    packaged_segment: &str,
) -> anyhow::Result<PathBuf> {
    if let Some(path) = env::var_os(env_name) {
        return Ok(PathBuf::from(path));
    }

    if tauri::is_dev() {
        return Ok(development_default);
    }

    Ok(app
        .path()
        .app_cache_dir()
        .context("resolve command app cache dir")?
        .join(packaged_segment))
}

fn load_runtime_paths(config_path: &Path) -> RuntimePaths {
    let config = if config_path.exists() {
        std::fs::read(config_path)
            .ok()
            .and_then(|raw| serde_json::from_slice::<RuntimePathsConfig>(&raw).ok())
            .unwrap_or_default()
    } else {
        RuntimePathsConfig::default()
    };

    RuntimePaths {
        mpv_path: env_path_or_config_or_default(
            "MPNEXT_RUNTIME_MPV_PATH",
            config.mpv,
            PathBuf::from("C:/mpv/mpv.exe"),
        ),
        sevenz_path: env_path_or_config_or_default(
            "MPNEXT_RUNTIME_SEVENVZ_PATH",
            config.sevenz,
            PathBuf::from("C:/Program Files/7-Zip/7z.exe"),
        ),
    }
}

fn env_path_or_config_or_default(
    env_name: &str,
    config_value: Option<String>,
    default: PathBuf,
) -> PathBuf {
    match env::var_os(env_name) {
        Some(value) => PathBuf::from(value),
        None => config_value.map(PathBuf::from).unwrap_or(default),
    }
}

fn protocol_database_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
    command_database_path(app)
}

fn open_protocol_database(db_path: &Path) -> anyhow::Result<MediaDatabase> {
    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent).with_context(|| {
            format!("create protocol database parent dir: {}", parent.display())
        })?;
    }

    MediaDatabase::open(DatabaseLocation::File(db_path))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .expect("workspace root should be available")
}

#[cfg(test)]
mod tests {
    use super::{
        archive_protocol_response, map_runtime_command_error, map_subtitle_command_error,
        media_protocol_response, parse_archive_entry_id_from_uri, parse_media_asset_id_from_uri,
        parse_thumbnail_key_from_uri, thumbnail_protocol_response, ERROR_CODE_HEADER,
        ERROR_RETRIABLE_HEADER,
    };
    use anyhow::anyhow;
    use app_core::ports::{
        ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
        SourceRepository, ThumbnailRepository,
    };
    use media_db::{DatabaseLocation, MediaDatabase};
    use shared_model::{
        AppErrorCode, ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId,
        LibraryId, LibraryRecord, MediaAssetRecord, MediaSourceKind, SourceId, SourceKind,
        SourceRecord, ThumbnailKey, ThumbnailRecord,
    };
    use tauri::http::{header::CONTENT_TYPE, StatusCode, Uri};
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn parses_thumb_uri_key() {
        let uri: Uri = "thumb://cache/thumb_primary"
            .parse()
            .expect("uri should parse");
        let key = parse_thumbnail_key_from_uri(&uri).expect("key should parse");

        assert_eq!(key, "thumb_primary");
    }

    #[test]
    fn parses_media_and_archive_uri_keys() {
        let media_uri: Uri = "media://asset/asset_video_primary"
            .parse()
            .expect("media uri should parse");
        let archive_uri: Uri = "archive://entry/archive_entry_primary"
            .parse()
            .expect("archive uri should parse");

        assert_eq!(
            parse_media_asset_id_from_uri(&media_uri).expect("media asset id should parse"),
            "asset_video_primary"
        );
        assert_eq!(
            parse_archive_entry_id_from_uri(&archive_uri).expect("archive entry id should parse"),
            "archive_entry_primary"
        );
    }

    #[test]
    fn serves_thumbnail_file_from_protocol_handler() {
        let temp = tempdir().expect("tempdir should exist");
        let db_file = NamedTempFile::new().expect("db file should exist");
        let thumbnail_path = temp.path().join("thumb.webp");
        std::fs::write(&thumbnail_path, b"webp-bytes").expect("thumbnail should be written");

        let database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let repositories = database.repositories();
        let asset = MediaAssetRecord {
            id: AssetId("asset_thumb_protocol".to_string()),
            source_kind: MediaSourceKind::File,
            source_ref_id: "source_thumb_protocol".to_string(),
            mime: "image/png".to_string(),
            width: None,
            height: None,
            duration_ms: None,
            codec_info_json: None,
            orientation: None,
            created_at: "1".to_string(),
        };
        AssetRepository::upsert(&repositories, &asset).expect("asset should be stored");
        ThumbnailRepository::upsert(
            &repositories,
            &ThumbnailRecord {
                thumbnail_key: ThumbnailKey("thumb_primary".to_string()),
                asset_id: asset.id,
                profile: "grid-sm".to_string(),
                width: 240,
                height: 180,
                format: "webp".to_string(),
                disk_path: thumbnail_path.display().to_string(),
                byte_size: 10,
                state: "ready".to_string(),
                updated_at: "1".to_string(),
            },
        )
        .expect("thumbnail should be stored");

        let uri: Uri = "thumb://cache/thumb_primary"
            .parse()
            .expect("uri should parse");
        let response = thumbnail_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).expect("content type"),
            "image/webp"
        );
        assert_eq!(response.body(), b"webp-bytes");
    }

    #[test]
    fn returns_not_found_when_thumbnail_record_missing() {
        let db_file = NamedTempFile::new().expect("db file should exist");
        let _database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let uri: Uri = "thumb://cache/thumb_missing"
            .parse()
            .expect("uri should parse");

        let response = thumbnail_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response
                .headers()
                .get(ERROR_CODE_HEADER)
                .expect("error code header"),
            "NOT_FOUND"
        );
        assert_eq!(
            response
                .headers()
                .get(ERROR_RETRIABLE_HEADER)
                .expect("retriable header"),
            "false"
        );
    }

    #[test]
    fn returns_invalid_argument_when_thumb_uri_is_unsupported() {
        let db_file = NamedTempFile::new().expect("db file should exist");
        let _database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let uri: Uri = "thumb://cache".parse().expect("uri should parse");

        let response = thumbnail_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response
                .headers()
                .get(ERROR_CODE_HEADER)
                .expect("error code header"),
            "INVALID_ARGUMENT"
        );
    }

    #[test]
    fn returns_not_found_when_media_asset_missing() {
        let db_file = NamedTempFile::new().expect("db file should exist");
        let _database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let uri: Uri = "media://asset/asset_missing"
            .parse()
            .expect("uri should parse");

        let response = media_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.body(), b"media asset not found: asset_missing");
        assert_eq!(
            response
                .headers()
                .get(ERROR_CODE_HEADER)
                .expect("error code header"),
            "NOT_FOUND"
        );
    }

    #[test]
    fn returns_invalid_argument_when_media_uri_is_unsupported() {
        let db_file = NamedTempFile::new().expect("db file should exist");
        let _database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let uri: Uri = "media://asset".parse().expect("uri should parse");

        let response = media_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response
                .headers()
                .get(ERROR_CODE_HEADER)
                .expect("error code header"),
            "INVALID_ARGUMENT"
        );
    }

    #[test]
    fn serves_media_file_from_protocol_handler() {
        let temp = tempdir().expect("tempdir should exist");
        let db_file = NamedTempFile::new().expect("db file should exist");
        let media_path = temp.path().join("video.mp4");
        std::fs::write(&media_path, b"video-bytes").expect("media file should exist");

        let database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let repositories = database.repositories();
        LibraryRepository::upsert(
            &repositories,
            &LibraryRecord {
                id: LibraryId("library_media_protocol".to_string()),
                root_path: temp.path().display().to_string(),
                library_type: "filesystem".to_string(),
                scan_mode: "full".to_string(),
                created_at: "1".to_string(),
                updated_at: "1".to_string(),
            },
        )
        .expect("library should store");
        SourceRepository::upsert(
            &repositories,
            &SourceRecord {
                id: SourceId("source_media_protocol".to_string()),
                library_id: LibraryId("library_media_protocol".to_string()),
                normalized_path: media_path.display().to_string(),
                file_name: "video.mp4".to_string(),
                ext: "mp4".to_string(),
                kind: SourceKind::Video,
                size: 10,
                mtime_ms: 1,
                fingerprint: Some("fp-media-protocol".to_string()),
                exists: true,
                last_seen_at: "1".to_string(),
            },
        )
        .expect("source should store");
        AssetRepository::upsert(
            &repositories,
            &MediaAssetRecord {
                id: AssetId("asset_media_protocol".to_string()),
                source_kind: MediaSourceKind::File,
                source_ref_id: "source_media_protocol".to_string(),
                mime: "video/mp4".to_string(),
                width: None,
                height: None,
                duration_ms: None,
                codec_info_json: None,
                orientation: None,
                created_at: "1".to_string(),
            },
        )
        .expect("asset should store");

        let uri: Uri = "media://asset/asset_media_protocol"
            .parse()
            .expect("uri should parse");
        let response = media_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).expect("content type"),
            "video/mp4"
        );
        assert_eq!(response.body(), b"video-bytes");
    }

    #[test]
    fn returns_not_found_when_media_file_missing() {
        let temp = tempdir().expect("tempdir should exist");
        let db_file = NamedTempFile::new().expect("db file should exist");
        let media_path = temp.path().join("missing-video.mp4");

        let database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let repositories = database.repositories();
        let library_id = LibraryId("library_media_missing_file".to_string());

        LibraryRepository::upsert(
            &repositories,
            &LibraryRecord {
                id: library_id.clone(),
                root_path: temp.path().display().to_string(),
                library_type: "filesystem".to_string(),
                scan_mode: "full".to_string(),
                created_at: "1".to_string(),
                updated_at: "1".to_string(),
            },
        )
        .expect("library should store");
        SourceRepository::upsert(
            &repositories,
            &SourceRecord {
                id: SourceId("source_media_missing_file".to_string()),
                library_id,
                normalized_path: media_path.display().to_string(),
                file_name: "missing-video.mp4".to_string(),
                ext: "mp4".to_string(),
                kind: SourceKind::Video,
                size: 10,
                mtime_ms: 1,
                fingerprint: Some("fp-media-missing-file".to_string()),
                exists: true,
                last_seen_at: "1".to_string(),
            },
        )
        .expect("source should store");
        AssetRepository::upsert(
            &repositories,
            &MediaAssetRecord {
                id: AssetId("asset_media_missing_file".to_string()),
                source_kind: MediaSourceKind::File,
                source_ref_id: "source_media_missing_file".to_string(),
                mime: "video/mp4".to_string(),
                width: None,
                height: None,
                duration_ms: None,
                codec_info_json: None,
                orientation: None,
                created_at: "1".to_string(),
            },
        )
        .expect("asset should store");

        let uri: Uri = "media://asset/asset_media_missing_file"
            .parse()
            .expect("uri should parse");
        let response = media_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(String::from_utf8_lossy(response.body()).contains("media file not found"));
        assert_eq!(
            response
                .headers()
                .get(ERROR_CODE_HEADER)
                .expect("error code header"),
            "NOT_FOUND"
        );
    }

    #[test]
    fn serves_archive_entry_from_protocol_handler() {
        let temp = tempdir().expect("tempdir should exist");
        let db_file = NamedTempFile::new().expect("db file should exist");
        let zip_path = temp.path().join("chapter.cbz");
        let file = std::fs::File::create(&zip_path).expect("zip file should exist");
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        use std::io::Write;
        writer
            .start_file("001-cover.png", options)
            .expect("zip entry should start");
        writer
            .write_all(b"cover-bytes")
            .expect("zip entry should write");
        writer.finish().expect("zip should finish");

        let database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let repositories = database.repositories();
        let library_id = LibraryId("library_archive_protocol".to_string());
        let source_id = SourceId("source_archive_protocol".to_string());
        let archive_id = ArchiveId("archive_protocol".to_string());

        LibraryRepository::upsert(
            &repositories,
            &LibraryRecord {
                id: library_id.clone(),
                root_path: temp.path().display().to_string(),
                library_type: "filesystem".to_string(),
                scan_mode: "full".to_string(),
                created_at: "1".to_string(),
                updated_at: "1".to_string(),
            },
        )
        .expect("library should store");
        SourceRepository::upsert(
            &repositories,
            &SourceRecord {
                id: source_id.clone(),
                library_id,
                normalized_path: zip_path.display().to_string(),
                file_name: "chapter.cbz".to_string(),
                ext: "cbz".to_string(),
                kind: SourceKind::Archive,
                size: 10,
                mtime_ms: 1,
                fingerprint: Some("fp-archive-protocol".to_string()),
                exists: true,
                last_seen_at: "1".to_string(),
            },
        )
        .expect("source should store");
        ArchiveRepository::upsert(
            &repositories,
            &ArchiveRecord {
                id: archive_id.clone(),
                source_id,
                archive_type: "cbz".to_string(),
                normalized_zip_path: Some(zip_path.display().to_string()),
                page_count: Some(1),
                cover_entry_id: Some(ArchiveEntryId("archive_entry_primary".to_string())),
                status: "indexed".to_string(),
            },
        )
        .expect("archive should store");
        ArchiveEntryRepository::replace_for_archive(
            &repositories,
            &archive_id,
            &[ArchiveEntryRecord {
                id: ArchiveEntryId("archive_entry_primary".to_string()),
                archive_id: archive_id.clone(),
                entry_path: "001-cover.png".to_string(),
                entry_name: "001-cover.png".to_string(),
                page_index: 0,
                media_kind: "image".to_string(),
                width: None,
                height: None,
                compressed_size: Some(10),
                uncompressed_size: Some(10),
                crc32: Some(1),
            }],
        )
        .expect("archive entry should store");

        let uri: Uri = "archive://entry/archive_entry_primary"
            .parse()
            .expect("uri should parse");
        let response = archive_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).expect("content type"),
            "image/png"
        );
        assert_eq!(response.body(), b"cover-bytes");
    }

    #[test]
    fn returns_not_found_when_archive_entry_missing() {
        let db_file = NamedTempFile::new().expect("db file should exist");
        let _database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let uri: Uri = "archive://entry/archive_entry_missing"
            .parse()
            .expect("uri should parse");

        let response = archive_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(
            response.body(),
            b"archive entry not found: archive_entry_missing"
        );
        assert_eq!(
            response
                .headers()
                .get(ERROR_CODE_HEADER)
                .expect("error code header"),
            "NOT_FOUND"
        );
    }

    #[test]
    fn returns_invalid_argument_when_archive_uri_is_unsupported() {
        let db_file = NamedTempFile::new().expect("db file should exist");
        let _database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let uri: Uri = "archive://entry".parse().expect("uri should parse");

        let response = archive_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        assert_eq!(
            response
                .headers()
                .get(ERROR_CODE_HEADER)
                .expect("error code header"),
            "INVALID_ARGUMENT"
        );
    }

    #[test]
    fn returns_not_found_when_archive_file_missing() {
        let temp = tempdir().expect("tempdir should exist");
        let db_file = NamedTempFile::new().expect("db file should exist");
        let zip_path = temp.path().join("missing-chapter.cbz");

        let database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let repositories = database.repositories();
        let library_id = LibraryId("library_archive_missing_file".to_string());
        let source_id = SourceId("source_archive_missing_file".to_string());
        let archive_id = ArchiveId("archive_missing_file".to_string());

        LibraryRepository::upsert(
            &repositories,
            &LibraryRecord {
                id: library_id.clone(),
                root_path: temp.path().display().to_string(),
                library_type: "filesystem".to_string(),
                scan_mode: "full".to_string(),
                created_at: "1".to_string(),
                updated_at: "1".to_string(),
            },
        )
        .expect("library should store");
        SourceRepository::upsert(
            &repositories,
            &SourceRecord {
                id: source_id.clone(),
                library_id,
                normalized_path: zip_path.display().to_string(),
                file_name: "missing-chapter.cbz".to_string(),
                ext: "cbz".to_string(),
                kind: SourceKind::Archive,
                size: 10,
                mtime_ms: 1,
                fingerprint: Some("fp-archive-missing-file".to_string()),
                exists: true,
                last_seen_at: "1".to_string(),
            },
        )
        .expect("source should store");
        ArchiveRepository::upsert(
            &repositories,
            &ArchiveRecord {
                id: archive_id.clone(),
                source_id,
                archive_type: "cbz".to_string(),
                normalized_zip_path: Some(zip_path.display().to_string()),
                page_count: Some(1),
                cover_entry_id: Some(ArchiveEntryId("archive_entry_missing_file".to_string())),
                status: "indexed".to_string(),
            },
        )
        .expect("archive should store");
        ArchiveEntryRepository::replace_for_archive(
            &repositories,
            &archive_id,
            &[ArchiveEntryRecord {
                id: ArchiveEntryId("archive_entry_missing_file".to_string()),
                archive_id: archive_id.clone(),
                entry_path: "001-cover.png".to_string(),
                entry_name: "001-cover.png".to_string(),
                page_index: 0,
                media_kind: "image".to_string(),
                width: None,
                height: None,
                compressed_size: Some(10),
                uncompressed_size: Some(10),
                crc32: Some(1),
            }],
        )
        .expect("archive entry should store");

        let uri: Uri = "archive://entry/archive_entry_missing_file"
            .parse()
            .expect("uri should parse");
        let response = archive_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert!(String::from_utf8_lossy(response.body()).contains("archive entry file not found"));
        assert_eq!(
            response
                .headers()
                .get(ERROR_CODE_HEADER)
                .expect("error code header"),
            "NOT_FOUND"
        );
    }

    #[test]
    fn maps_runtime_command_errors_to_app_error() {
        let error = map_runtime_command_error(
            "runtime_smoke_check",
            anyhow!("runtime binary not found: C:/missing/ffmpeg.exe"),
        );

        assert_eq!(error.code, AppErrorCode::NotFound);
        assert!(!error.retriable);
        assert_eq!(
            error.details.expect("details")["command"],
            serde_json::Value::String("runtime_smoke_check".to_string())
        );
    }

    #[test]
    fn maps_subtitle_command_timeout_to_app_error() {
        let error = map_subtitle_command_error(
            "subtitle_ping_command",
            anyhow!("subtitle sidecar timed out after 150 ms"),
        );

        assert_eq!(error.code, AppErrorCode::Timeout);
        assert!(error.retriable);
    }

    #[test]
    fn maps_subtitle_command_embedded_error_code_to_app_error() {
        let error = map_subtitle_command_error(
            "subtitle_get_progress_command",
            anyhow!("subtitle sidecar get_progress failed [NOT_FOUND]: session missing"),
        );

        assert_eq!(error.code, AppErrorCode::NotFound);
        assert!(!error.retriable);
    }

    #[test]
    fn maps_subtitle_command_payload_errors_to_external_tool_error() {
        let error = map_subtitle_command_error(
            "subtitle_ping_command",
            anyhow!("subtitle sidecar response payload missing"),
        );

        assert_eq!(error.code, AppErrorCode::ExternalToolError);
        assert!(!error.retriable);
    }
}
