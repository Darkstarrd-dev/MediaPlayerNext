use crate::{AssetId, SourceId, TaskId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct LogContext {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_id: Option<TaskId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub asset_id: Option<AssetId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<SourceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExternalProcessLog {
    pub event: String,
    pub phase: String,
    pub tool: String,
    pub executable: String,
    pub arguments: Vec<String>,
    pub command_line: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    pub ok: bool,
    pub context: LogContext,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr_excerpt: Option<String>,
}

pub fn build_command_line(executable: &str, arguments: &[String]) -> String {
    std::iter::once(executable.to_string())
        .chain(arguments.iter().cloned())
        .map(|part| quote_command_part(&part))
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn emit_external_process_log(log: &ExternalProcessLog) {
    match serde_json::to_string(log) {
        Ok(line) => eprintln!("{line}"),
        Err(error) => eprintln!(
            "{{\"event\":\"external-process\",\"phase\":\"serialize_failed\",\"tool\":\"observability\",\"ok\":false,\"stderrExcerpt\":\"{}\"}}",
            error
        ),
    }
}

fn quote_command_part(value: &str) -> String {
    if value.is_empty() || value.chars().any(|ch| ch.is_whitespace() || ch == '"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{build_command_line, ExternalProcessLog, LogContext};
    use crate::{AssetId, SubtitleSessionId};

    #[test]
    fn builds_command_line_with_quotes() {
        let command_line = build_command_line(
            "C:/Program Files/nodejs/node.exe",
            &[
                "Z:/repo/apps/subtitle-sidecar/dist/src/index.js".to_string(),
                "--sessions-root".to_string(),
                "C:/Users/Test User/subtitle sessions".to_string(),
            ],
        );

        assert!(command_line.contains("\"C:/Program Files/nodejs/node.exe\""));
        assert!(command_line.contains("\"C:/Users/Test User/subtitle sessions\""));
    }

    #[test]
    fn serializes_external_process_log() {
        let log = ExternalProcessLog {
            event: "external-process".to_string(),
            phase: "completed".to_string(),
            tool: "subtitle-sidecar".to_string(),
            executable: "node".to_string(),
            arguments: vec!["sidecar/index.js".to_string()],
            command_line: "node sidecar/index.js".to_string(),
            exit_code: Some(0),
            duration_ms: Some(42),
            ok: true,
            context: LogContext {
                asset_id: Some(AssetId("asset_log_001".to_string())),
                session_id: Some(SubtitleSessionId("subtitle_log_001".to_string()).0),
                ..LogContext::default()
            },
            stderr_excerpt: None,
        };

        let json = serde_json::to_string(&log).expect("log should serialize");
        assert!(json.contains("external-process"));
        assert!(json.contains("asset_log_001"));
        assert!(json.contains("subtitle_log_001"));
    }
}
