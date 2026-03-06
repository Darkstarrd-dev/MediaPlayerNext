use crate::diagnostics::{ping, DiagnosticsSummary};
use anyhow::Result;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendCommand {
    Help,
    Diagnostics,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CliExecutionOutput {
    pub command: String,
    pub config_path: PathBuf,
    pub config_exists: bool,
    pub summary: DiagnosticsSummary,
}

pub fn parse_command(args: &[String]) -> BackendCommand {
    match args.first().map(String::as_str) {
        Some("diagnostics") => BackendCommand::Diagnostics,
        _ => BackendCommand::Help,
    }
}

pub fn run_command(command: BackendCommand, config_path: PathBuf) -> Result<CliExecutionOutput> {
    let summary = ping();
    let command_name = match command {
        BackendCommand::Help => "help",
        BackendCommand::Diagnostics => "diagnostics",
    };

    Ok(CliExecutionOutput {
        command: command_name.to_string(),
        config_exists: config_path.exists(),
        config_path,
        summary,
    })
}
