mod app_protocol;
mod protocol_helpers;
mod runtime_check;
mod runtime_storage;
pub mod subtitle_sidecar;
mod tauri_commands;
mod tauri_media_commands;
mod tauri_playback_commands;
mod tauri_scan_commands;
mod tauri_subtitle_commands;
mod tauri_workspace_commands;

use anyhow::{Context, Error};
use app_protocol::{
    app_error_response, archive_protocol_response, media_protocol_response, protocol_database_path,
    thumbnail_protocol_response,
};
use media_db::{DatabaseLocation, MediaDatabase};
use runtime_check::run_runtime_smoke_check;
use runtime_storage::resolve_runtime_storage_paths;
use serde::Deserialize;
use serde_json::json;
use shared_model::{AppError, AppErrorCode};
use std::env;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::http::StatusCode;
use tauri::AppHandle;

pub(crate) const SIDEBAR_THUMBNAIL_PROFILE: &str = "grid-md";

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimePathsConfig {
    mpv: Option<String>,
    sevenz: Option<String>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimePaths {
    pub(crate) mpv_path: PathBuf,
    pub(crate) sevenz_path: PathBuf,
}

pub(crate) struct CommandEnvironment {
    pub(crate) database: MediaDatabase,
    pub(crate) runtime_paths: RuntimePaths,
    pub(crate) thumbnail_cache_root: PathBuf,
    pub(crate) normalize_root: PathBuf,
    pub(crate) playback_sessions_root: PathBuf,
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
        .plugin(tauri_plugin_clipboard_x::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            tauri_commands::greet,
            tauri_commands::runtime_smoke_check,
            tauri_commands::read_runtime_info_command,
            tauri_commands::set_runtime_storage_paths_command,
            tauri_commands::clear_database_command,
            tauri_workspace_commands::workspace_cursor_read_command,
            tauri_workspace_commands::workspace_cursor_write_command,
            tauri_subtitle_commands::subtitle_ping_command,
            tauri_subtitle_commands::subtitle_health_command,
            tauri_subtitle_commands::subtitle_start_session_command,
            tauri_subtitle_commands::subtitle_stop_session_command,
            tauri_subtitle_commands::subtitle_get_progress_command,
            tauri_workspace_commands::library_list_command,
            tauri_workspace_commands::library_add_command,
            tauri_workspace_commands::library_get_command,
            tauri_workspace_commands::library_remove_command,
            tauri_workspace_commands::library_nodes_command,
            tauri_scan_commands::scan_start_command,
            tauri_scan_commands::scan_resume_command,
            tauri_scan_commands::scan_stats_command,
            tauri_scan_commands::scan_snapshot_command,
            tauri_media_commands::items_list_command,
            tauri_media_commands::item_detail_command,
            tauri_media_commands::archive_entries_command,
            tauri_media_commands::archive_entry_detail_command,
            tauri_media_commands::archive_normalize_command,
            tauri_media_commands::archive_normalize_status_command,
            tauri_media_commands::thumbnail_ensure_command,
            tauri_playback_commands::playback_open_command,
            tauri_playback_commands::playback_status_command,
            tauri_playback_commands::playback_seek_command
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

pub(crate) fn map_runtime_command_error(command: &str, error: Error) -> AppError {
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

pub(crate) fn map_subtitle_command_error(command: &str, error: Error) -> AppError {
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
    } else if message.contains("subtitle sidecar entry missing")
        || message.contains("not found")
        || message.contains("missing")
    {
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

pub(crate) fn with_command_environment<T, F>(app: &AppHandle, run: F) -> anyhow::Result<T>
where
    F: FnOnce(&CommandEnvironment) -> anyhow::Result<T>,
{
    let environment = command_environment(app)?;
    run(&environment)
}

fn command_environment(app: &AppHandle) -> anyhow::Result<CommandEnvironment> {
    let config_path = workspace_root().join("config").join("local.paths.json");
    let storage_paths = resolve_runtime_storage_paths(app)?;

    if let Some(parent) = storage_paths.database_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("create command database parent dir: {}", parent.display()))?;
    }
    std::fs::create_dir_all(&storage_paths.thumbnail_cache_root).with_context(|| {
        format!(
            "create thumbnail cache root for commands: {}",
            storage_paths.thumbnail_cache_root.display()
        )
    })?;
    std::fs::create_dir_all(&storage_paths.normalize_root).with_context(|| {
        format!(
            "create archive normalize root for commands: {}",
            storage_paths.normalize_root.display()
        )
    })?;
    std::fs::create_dir_all(&storage_paths.playback_sessions_root).with_context(|| {
        format!(
            "create playback sessions root for commands: {}",
            storage_paths.playback_sessions_root.display()
        )
    })?;

    Ok(CommandEnvironment {
        database: MediaDatabase::open(DatabaseLocation::File(&storage_paths.database_path))?,
        runtime_paths: load_runtime_paths(&config_path),
        thumbnail_cache_root: storage_paths.thumbnail_cache_root,
        normalize_root: storage_paths.normalize_root,
        playback_sessions_root: storage_paths.playback_sessions_root,
    })
}

pub(crate) fn map_backend_command_error(command: &str, domain: &str, error: Error) -> AppError {
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

pub(crate) fn now_epoch_millis_string() -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    millis.to_string()
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

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

#[cfg(test)]
mod lib_tests;
