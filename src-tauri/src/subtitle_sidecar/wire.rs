use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared_model::{AssetId, LogContext};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireRequest {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireResponse {
    pub id: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub ok: bool,
    #[serde(default)]
    pub payload: Option<Value>,
    #[serde(default)]
    pub error: Option<WireError>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WireError {
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub retriable: Option<bool>,
}

pub(super) fn next_request_id(message_type: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("req-{message_type}-{millis}")
}

pub(super) fn request_log_context(payload: &Option<Value>) -> LogContext {
    let mut context = LogContext::default();

    let Some(payload) = payload.as_ref() else {
        return context;
    };

    if let Some(asset_id) = payload.get("assetId").and_then(Value::as_str) {
        context.asset_id = Some(AssetId(asset_id.to_string()));
    }
    if let Some(session_id) = payload.get("sessionId").and_then(Value::as_str) {
        context.session_id = Some(session_id.to_string());
    }

    context
}

pub(super) fn stderr_excerpt_text(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.chars().take(240).collect())
    }
}
