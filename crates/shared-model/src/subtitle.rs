use crate::{AssetId, SubtitleSessionId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitlePingSummary {
    pub service: String,
    pub protocol_version: String,
    pub transport: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleHealthSummary {
    pub service: String,
    pub protocol_version: String,
    pub transport: String,
    pub node_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sharp_version: Option<String>,
    pub uptime_ms: u64,
    pub active_sessions: u64,
    pub sessions_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleHostSummary {
    pub executable: String,
    pub entry_path: String,
    pub running: bool,
    pub restart_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ping: Option<SubtitlePingSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub health: Option<SubtitleHealthSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubtitleSessionState {
    Idle,
    Running,
    Stopped,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleSessionSummary {
    pub session_id: SubtitleSessionId,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<AssetId>,
    pub state: SubtitleSessionState,
    pub progress: f64,
    pub created_at: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SubtitleProgressEvent {
    pub session_id: SubtitleSessionId,
    pub state: SubtitleSessionState,
    pub progress: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
