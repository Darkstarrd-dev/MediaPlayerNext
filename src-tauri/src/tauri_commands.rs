use shared_model::AppError;

use crate::runtime_check::{run_runtime_smoke_check, RuntimeSmokeCheckResult};
use crate::runtime_storage::RuntimeInfoPayload;

type CommandResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {name}! MediaPlayerNext host is ready.")
}

#[tauri::command]
pub fn runtime_smoke_check(
    ffmpeg_path: String,
    ffprobe_path: String,
    mpv_path: String,
) -> CommandResult<RuntimeSmokeCheckResult> {
    run_runtime_smoke_check(&ffmpeg_path, &ffprobe_path, &mpv_path)
        .map_err(|error| crate::map_runtime_command_error("runtime_smoke_check", error))
}

#[tauri::command]
pub fn read_runtime_info_command(app: tauri::AppHandle) -> CommandResult<RuntimeInfoPayload> {
    crate::runtime_storage::read_runtime_info(&app).map_err(|error| {
        crate::map_backend_command_error("read_runtime_info_command", "database", error)
    })
}

#[tauri::command]
pub fn set_runtime_storage_paths_command(
    app: tauri::AppHandle,
    database_dir: Option<String>,
    thumbnail_cache_dir: Option<String>,
) -> CommandResult<RuntimeInfoPayload> {
    crate::runtime_storage::set_runtime_storage_paths(&app, database_dir, thumbnail_cache_dir)
        .map_err(|error| {
            crate::map_backend_command_error("set_runtime_storage_paths_command", "database", error)
        })
}

#[tauri::command]
pub fn clear_database_command(app: tauri::AppHandle) -> CommandResult<()> {
    crate::runtime_storage::clear_database(&app).map_err(|error| {
        crate::map_backend_command_error("clear_database_command", "database", error)
    })
}
