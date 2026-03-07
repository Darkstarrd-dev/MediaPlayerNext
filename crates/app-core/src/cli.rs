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
    ArchiveNormalize {
        source_id: String,
    },
    ArchiveNormalizeStatus {
        task_id: String,
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
    PlaybackProbe {
        asset_id: String,
    },
    PlaybackOpen {
        asset_id: String,
    },
    PlaybackStatus {
        session_id: String,
    },
    PlaybackPause {
        session_id: String,
    },
    PlaybackSeek {
        session_id: String,
        position_ms: String,
    },
    PlaybackStop {
        session_id: String,
    },
    SubtitlePing,
    SubtitleHealth,
    SubtitleStartSession {
        asset_id: Option<String>,
    },
    SubtitleStopSession {
        session_id: String,
    },
    SubtitleGetProgress {
        session_id: String,
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
            (Some("normalize"), Some(source_id)) => BackendCommand::ArchiveNormalize {
                source_id: source_id.clone(),
            },
            (Some("normalize-status"), Some(task_id)) => BackendCommand::ArchiveNormalizeStatus {
                task_id: task_id.clone(),
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
        Some("playback") => match (args.get(1).map(String::as_str), args.get(2), args.get(3)) {
            (Some("probe"), Some(asset_id), _) => BackendCommand::PlaybackProbe {
                asset_id: asset_id.clone(),
            },
            (Some("open"), Some(asset_id), _) => BackendCommand::PlaybackOpen {
                asset_id: asset_id.clone(),
            },
            (Some("status"), Some(session_id), _) => BackendCommand::PlaybackStatus {
                session_id: session_id.clone(),
            },
            (Some("pause"), Some(session_id), _) => BackendCommand::PlaybackPause {
                session_id: session_id.clone(),
            },
            (Some("seek"), Some(session_id), Some(position_ms)) => BackendCommand::PlaybackSeek {
                session_id: session_id.clone(),
                position_ms: position_ms.clone(),
            },
            (Some("stop"), Some(session_id), _) => BackendCommand::PlaybackStop {
                session_id: session_id.clone(),
            },
            _ => BackendCommand::Help,
        },
        Some("subtitle") => match args.get(1).map(String::as_str) {
            Some("ping") => BackendCommand::SubtitlePing,
            Some("health") => BackendCommand::SubtitleHealth,
            Some("start-session") => BackendCommand::SubtitleStartSession {
                asset_id: args.get(2).cloned(),
            },
            Some("stop-session") if args.get(2).is_some() => BackendCommand::SubtitleStopSession {
                session_id: args[2].clone(),
            },
            Some("get-progress") if args.get(2).is_some() => BackendCommand::SubtitleGetProgress {
                session_id: args[2].clone(),
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
        BackendCommand::ArchiveNormalize { .. } => "archive.normalize",
        BackendCommand::ArchiveNormalizeStatus { .. } => "archive.normalize-status",
        BackendCommand::ArchiveReadEntry { .. } => "archive.read-entry",
        BackendCommand::AssetEnsure { .. } => "asset.ensure",
        BackendCommand::AssetResolve { .. } => "asset.resolve",
        BackendCommand::ThumbnailEnsure { .. } => "thumbnail.ensure",
        BackendCommand::ThumbnailShow { .. } => "thumbnail.show",
        BackendCommand::PlaybackProbe { .. } => "playback.probe",
        BackendCommand::PlaybackOpen { .. } => "playback.open",
        BackendCommand::PlaybackStatus { .. } => "playback.status",
        BackendCommand::PlaybackPause { .. } => "playback.pause",
        BackendCommand::PlaybackSeek { .. } => "playback.seek",
        BackendCommand::PlaybackStop { .. } => "playback.stop",
        BackendCommand::SubtitlePing => "subtitle.ping",
        BackendCommand::SubtitleHealth => "subtitle.health",
        BackendCommand::SubtitleStartSession { .. } => "subtitle.start-session",
        BackendCommand::SubtitleStopSession { .. } => "subtitle.stop-session",
        BackendCommand::SubtitleGetProgress { .. } => "subtitle.get-progress",
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
            "cargo run --bin backend_harness -- archive normalize <source-id>",
            "cargo run --bin backend_harness -- archive normalize-status <task-id>",
            "cargo run --bin backend_harness -- archive read-entry <source-id> <entry-path>",
            "cargo run --bin backend_harness -- asset ensure <library-id>",
            "cargo run --bin backend_harness -- asset resolve <asset-id>",
            "cargo run --bin backend_harness -- thumbnail ensure <asset-id> <profile>",
            "cargo run --bin backend_harness -- thumbnail show <thumbnail-key>",
            "cargo run --bin backend_harness -- playback probe <asset-id>",
            "cargo run --bin backend_harness -- playback open <asset-id>",
            "cargo run --bin backend_harness -- playback status <session-id>",
            "cargo run --bin backend_harness -- playback pause <session-id>",
            "cargo run --bin backend_harness -- playback seek <session-id> <position-ms>",
            "cargo run --bin backend_harness -- playback stop <session-id>",
            "cargo run --bin backend_harness -- subtitle ping",
            "cargo run --bin backend_harness -- subtitle health",
            "cargo run --bin backend_harness -- subtitle start-session [asset-id]",
            "cargo run --bin backend_harness -- subtitle stop-session <session-id>",
            "cargo run --bin backend_harness -- subtitle get-progress <session-id>"
        ]
    })
}
