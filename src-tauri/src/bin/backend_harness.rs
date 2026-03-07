use app_core::archive::{archive_snapshot, index_library_archives, read_archive_entry_by_source};
use app_core::asset::{ensure_media_assets_for_library, resolve_asset};
use app_core::cli::{help_payload, parse_command, run_command, BackendCommand};
use app_core::scan::{register_library, resume_scan, run_scan, scan_snapshot, scan_stats};
use app_core::thumbnail::{ensure_thumbnail_for_asset, get_thumbnail, parse_thumbnail_profile};
use media_db::{DatabaseLocation, MediaDatabase};
use serde_json::json;
use shared_model::{AssetId, LibraryId, SourceId, ThumbnailKey};
use std::env;
use std::path::PathBuf;

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn try_main() -> anyhow::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();
    let command = parse_command(&args);
    let config_path = workspace_root().join("config").join("local.paths.json");
    let db_path = workspace_root().join("data").join("mediaplayernext-dev.db");
    let thumbnail_cache_root = workspace_root().join("data").join("cache").join("thumbs");

    if let Some(parent) = db_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let database = MediaDatabase::open(DatabaseLocation::File(&db_path))?;
    let repositories = database.repositories();

    let payload = match &command {
        BackendCommand::Help => Some(help_payload()),
        BackendCommand::Diagnostics => Some(json!({ "databasePath": db_path })),
        BackendCommand::ScanAddLibrary { path } => {
            let library_id = register_library(&repositories, path)?;
            Some(json!({
                "libraryId": library_id.0,
                "rootPath": path,
            }))
        }
        BackendCommand::ScanRun { library_id } => {
            let summary = run_scan(
                &repositories,
                &repositories,
                &repositories,
                &LibraryId(library_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::ScanResume { library_id } => {
            let summary = resume_scan(
                &repositories,
                &repositories,
                &repositories,
                &LibraryId(library_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::ScanStats { library_id } => {
            let summary = scan_stats(&repositories, &repositories, &LibraryId(library_id.clone()))?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::ScanDiff { library_id } => {
            let snapshot =
                scan_snapshot(&repositories, &repositories, &LibraryId(library_id.clone()))?;
            Some(serde_json::to_value(snapshot)?)
        }
        BackendCommand::ArchiveIndex { library_id } => {
            let summary = index_library_archives(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &LibraryId(library_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::ArchiveShow { source_id } => {
            let snapshot =
                archive_snapshot(&repositories, &repositories, &SourceId(source_id.clone()))?;
            Some(serde_json::to_value(snapshot)?)
        }
        BackendCommand::ArchiveReadEntry {
            source_id,
            entry_path,
        } => {
            let summary = read_archive_entry_by_source(
                &repositories,
                &repositories,
                &SourceId(source_id.clone()),
                entry_path,
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::AssetEnsure { library_id } => {
            let summary = ensure_media_assets_for_library(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &LibraryId(library_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::AssetResolve { asset_id } => {
            let summary = resolve_asset(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &AssetId(asset_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::ThumbnailEnsure { asset_id, profile } => {
            let summary = ensure_thumbnail_for_asset(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &thumbnail_cache_root,
                &AssetId(asset_id.clone()),
                parse_thumbnail_profile(profile)?,
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::ThumbnailShow { thumbnail_key } => {
            let summary = get_thumbnail(&repositories, &ThumbnailKey(thumbnail_key.clone()))?;
            Some(serde_json::to_value(summary)?)
        }
    };

    let output = run_command(command, config_path, payload)?;
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .expect("workspace root should be available")
}
