use app_core::subtitle_host::{
    subtitle_get_progress, subtitle_health, subtitle_ping, subtitle_start_session,
    subtitle_stop_session,
};
use shared_model::{AppError, AssetId, SubtitleSessionId};

type CommandResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn subtitle_ping_command(
    app: tauri::AppHandle,
) -> CommandResult<shared_model::SubtitleHostSummary> {
    let host = crate::subtitle_sidecar::tauri_subtitle_host(&app)
        .map_err(|error| crate::map_subtitle_command_error("subtitle_ping_command", error))?;
    subtitle_ping(&host)
        .map_err(|error| crate::map_subtitle_command_error("subtitle_ping_command", error))
}

#[tauri::command]
pub fn subtitle_health_command(
    app: tauri::AppHandle,
) -> CommandResult<shared_model::SubtitleHostSummary> {
    let host = crate::subtitle_sidecar::tauri_subtitle_host(&app)
        .map_err(|error| crate::map_subtitle_command_error("subtitle_health_command", error))?;
    subtitle_health(&host)
        .map_err(|error| crate::map_subtitle_command_error("subtitle_health_command", error))
}

#[tauri::command]
pub fn subtitle_start_session_command(
    app: tauri::AppHandle,
    asset_id: Option<String>,
) -> CommandResult<shared_model::SubtitleSessionSummary> {
    let host = crate::subtitle_sidecar::tauri_subtitle_host(&app).map_err(|error| {
        crate::map_subtitle_command_error("subtitle_start_session_command", error)
    })?;
    let asset_id = asset_id.map(AssetId);
    subtitle_start_session(&host, asset_id.as_ref())
        .map_err(|error| crate::map_subtitle_command_error("subtitle_start_session_command", error))
}

#[tauri::command]
pub fn subtitle_stop_session_command(
    app: tauri::AppHandle,
    session_id: String,
) -> CommandResult<shared_model::SubtitleSessionSummary> {
    let host = crate::subtitle_sidecar::tauri_subtitle_host(&app).map_err(|error| {
        crate::map_subtitle_command_error("subtitle_stop_session_command", error)
    })?;
    subtitle_stop_session(&host, &SubtitleSessionId(session_id))
        .map_err(|error| crate::map_subtitle_command_error("subtitle_stop_session_command", error))
}

#[tauri::command]
pub fn subtitle_get_progress_command(
    app: tauri::AppHandle,
    session_id: String,
) -> CommandResult<shared_model::SubtitleProgressEvent> {
    let host = crate::subtitle_sidecar::tauri_subtitle_host(&app).map_err(|error| {
        crate::map_subtitle_command_error("subtitle_get_progress_command", error)
    })?;
    subtitle_get_progress(&host, &SubtitleSessionId(session_id))
        .map_err(|error| crate::map_subtitle_command_error("subtitle_get_progress_command", error))
}
