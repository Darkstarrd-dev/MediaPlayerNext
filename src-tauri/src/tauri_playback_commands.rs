use app_core::playback::{open_playback_session, playback_seek, playback_status};
use shared_model::{AppError, AssetId, PlaybackSessionId, PlaybackSessionSummary};

type CommandResult<T> = Result<T, AppError>;

#[tauri::command]
pub fn playback_open_command(
    app: tauri::AppHandle,
    asset_id: String,
) -> CommandResult<PlaybackSessionSummary> {
    crate::with_command_environment(&app, |environment| {
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
    .map_err(|error| crate::map_backend_command_error("playback_open_command", "playback", error))
}

#[tauri::command]
pub fn playback_status_command(
    app: tauri::AppHandle,
    session_id: String,
) -> CommandResult<PlaybackSessionSummary> {
    crate::with_command_environment(&app, |environment| {
        playback_status(
            &environment.playback_sessions_root,
            &PlaybackSessionId(session_id),
        )
    })
    .map_err(|error| crate::map_backend_command_error("playback_status_command", "playback", error))
}

#[tauri::command]
pub fn playback_seek_command(
    app: tauri::AppHandle,
    session_id: String,
    position_ms: i64,
) -> CommandResult<PlaybackSessionSummary> {
    crate::with_command_environment(&app, |environment| {
        playback_seek(
            &environment.playback_sessions_root,
            &PlaybackSessionId(session_id),
            position_ms,
        )
    })
    .map_err(|error| crate::map_backend_command_error("playback_seek_command", "playback", error))
}
