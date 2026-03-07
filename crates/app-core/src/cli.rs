use crate::diagnostics::{ping, DiagnosticsSummary};
use anyhow::Result;
use serde::Serialize;
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendCommand {
    Help,
    Diagnostics,
    ScanAddLibrary {
        path: PathBuf,
    },
    ScanRun {
        library_id: String,
    },
    ScanResume {
        library_id: String,
    },
    ScanStats {
        library_id: String,
    },
    ScanDiff {
        library_id: String,
    },
    ArchiveIndex {
        library_id: String,
    },
    ArchiveShow {
        source_id: String,
    },
    ArchiveReadEntry {
        source_id: String,
        entry_path: String,
    },
    AssetEnsure {
        library_id: String,
    },
    AssetResolve {
        asset_id: String,
    },
    ThumbnailEnsure {
        asset_id: String,
        profile: String,
    },
    ThumbnailShow {
        thumbnail_key: String,
    },
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
            (Some("resume"), Some(library_id)) => BackendCommand::ScanResume {
                library_id: library_id.clone(),
            },
            (Some("stats"), Some(library_id)) => BackendCommand::ScanStats {
                library_id: library_id.clone(),
            },
            (Some("diff"), Some(library_id)) => BackendCommand::ScanDiff {
                library_id: library_id.clone(),
            },
            _ => BackendCommand::Help,
        },
        Some("archive") => match (args.get(1).map(String::as_str), args.get(2)) {
            (Some("index"), Some(library_id)) => BackendCommand::ArchiveIndex {
                library_id: library_id.clone(),
            },
            (Some("show"), Some(source_id)) => BackendCommand::ArchiveShow {
                source_id: source_id.clone(),
            },
            (Some("read-entry"), Some(source_id)) if args.get(3).is_some() => {
                BackendCommand::ArchiveReadEntry {
                    source_id: source_id.clone(),
                    entry_path: args[3].clone(),
                }
            }
            _ => BackendCommand::Help,
        },
        Some("asset") => match (args.get(1).map(String::as_str), args.get(2)) {
            (Some("ensure"), Some(library_id)) => BackendCommand::AssetEnsure {
                library_id: library_id.clone(),
            },
            (Some("resolve"), Some(asset_id)) => BackendCommand::AssetResolve {
                asset_id: asset_id.clone(),
            },
            _ => BackendCommand::Help,
        },
        Some("thumbnail") => match (args.get(1).map(String::as_str), args.get(2), args.get(3)) {
            (Some("ensure"), Some(asset_id), Some(profile)) => BackendCommand::ThumbnailEnsure {
                asset_id: asset_id.clone(),
                profile: profile.clone(),
            },
            (Some("show"), Some(thumbnail_key), _) => BackendCommand::ThumbnailShow {
                thumbnail_key: thumbnail_key.clone(),
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
        BackendCommand::ScanResume { .. } => "scan.resume",
        BackendCommand::ScanStats { .. } => "scan.stats",
        BackendCommand::ScanDiff { .. } => "scan.diff",
        BackendCommand::ArchiveIndex { .. } => "archive.index",
        BackendCommand::ArchiveShow { .. } => "archive.show",
        BackendCommand::ArchiveReadEntry { .. } => "archive.read-entry",
        BackendCommand::AssetEnsure { .. } => "asset.ensure",
        BackendCommand::AssetResolve { .. } => "asset.resolve",
        BackendCommand::ThumbnailEnsure { .. } => "thumbnail.ensure",
        BackendCommand::ThumbnailShow { .. } => "thumbnail.show",
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
            "cargo run --bin backend_harness -- scan resume <library-id>",
            "cargo run --bin backend_harness -- scan stats <library-id>",
            "cargo run --bin backend_harness -- scan diff <library-id>",
            "cargo run --bin backend_harness -- archive index <library-id>",
            "cargo run --bin backend_harness -- archive show <source-id>",
            "cargo run --bin backend_harness -- archive read-entry <source-id> <entry-path>",
            "cargo run --bin backend_harness -- asset ensure <library-id>",
            "cargo run --bin backend_harness -- asset resolve <asset-id>",
            "cargo run --bin backend_harness -- thumbnail ensure <asset-id> <profile>",
            "cargo run --bin backend_harness -- thumbnail show <thumbnail-key>"
        ]
    })
}
