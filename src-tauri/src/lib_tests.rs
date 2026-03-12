use anyhow::{anyhow, Context, Result};
use shared_model::AppErrorCode;

#[test]
fn maps_runtime_command_errors_to_app_error() -> Result<()> {
    let error = crate::map_runtime_command_error(
        "runtime_smoke_check",
        anyhow!("runtime binary not found: C:/missing/ffmpeg.exe"),
    );

    assert_eq!(error.code, AppErrorCode::NotFound);
    assert!(!error.retriable);
    let details = error.details.context("details")?;
    assert_eq!(
        details["command"],
        serde_json::Value::String("runtime_smoke_check".to_string())
    );

    Ok(())
}

#[test]
fn maps_subtitle_command_timeout_to_app_error() -> Result<()> {
    let error = crate::map_subtitle_command_error(
        "subtitle_ping_command",
        anyhow!("subtitle sidecar timed out after 150 ms"),
    );

    assert_eq!(error.code, AppErrorCode::Timeout);
    assert!(error.retriable);

    Ok(())
}

#[test]
fn maps_subtitle_command_embedded_error_code_to_app_error() -> Result<()> {
    let error = crate::map_subtitle_command_error(
        "subtitle_get_progress_command",
        anyhow!("subtitle sidecar get_progress failed [NOT_FOUND]: session missing"),
    );

    assert_eq!(error.code, AppErrorCode::NotFound);
    assert!(!error.retriable);

    Ok(())
}

#[test]
fn maps_subtitle_command_payload_errors_to_external_tool_error() -> Result<()> {
    let error = crate::map_subtitle_command_error(
        "subtitle_ping_command",
        anyhow!("subtitle sidecar response payload missing"),
    );

    assert_eq!(error.code, AppErrorCode::ExternalToolError);
    assert!(!error.retriable);

    Ok(())
}
