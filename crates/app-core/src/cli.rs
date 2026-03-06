use crate::diagnostics::{ping, DiagnosticsSummary};
use anyhow::Result;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendCommand {
    Help,
    Diagnostics,
    ScanAddLibrary { path: PathBuf },
    ScanRun { library_id: String },
    ScanStats { library_id: String },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliExecutionOutput {
    pub command: String,
    pub config_path: PathBuf,
    pub config_exists: bool,
    pub summary: DiagnosticsSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<Value>,
}

pub fn parse_command(args: &[String]) -> BackendCommand {
    match args.first().map(String::as_str) {
        Some("diagnostics") => BackendCommand::Diagnostics,
        Some("scan") => match (args.get(1).map(String::as_str), args.get(2)) {
            (Some("add-library"), Some(path)) => BackendCommand::ScanAddLibrary {
                path: PathBuf::from(path),
            },
            (Some("run"), Some(library_id)) => BackendCommand::ScanRun {
                library_id: library_id.clone(),
            },
            (Some("stats"), Some(library_id)) => BackendCommand::ScanStats {
                library_id: library_id.clone(),
            },
            _ => BackendCommand::Help,
        },
        _ => BackendCommand::Help,
    }
}

pub fn run_command(
    command: BackendCommand,
    config_path: PathBuf,
    payload: Option<Value>,
) -> Result<CliExecutionOutput> {
    let summary = ping();
    let command_name = match command {
        BackendCommand::Help => "help",
        BackendCommand::Diagnostics => "diagnostics",
        BackendCommand::ScanAddLibrary { .. } => "scan.add-library",
        BackendCommand::ScanRun { .. } => "scan.run",
        BackendCommand::ScanStats { .. } => "scan.stats",
    };

    Ok(CliExecutionOutput {
        command: command_name.to_string(),
        config_exists: config_path.exists(),
        config_path,
        summary,
        payload,
    })
}

pub fn help_payload() -> Value {
    json!({
        "usage": [
            "cargo run --bin backend_harness -- diagnostics",
            "cargo run --bin backend_harness -- scan add-library <path>",
            "cargo run --bin backend_harness -- scan run <library-id>",
            "cargo run --bin backend_harness -- scan stats <library-id>"
        ]
    })
}
