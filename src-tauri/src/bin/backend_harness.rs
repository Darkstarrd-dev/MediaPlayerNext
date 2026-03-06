use app_core::cli::{help_payload, parse_command, run_command, BackendCommand};
use app_core::scan::{register_library, run_scan, scan_stats};
use media_db::{DatabaseLocation, MediaDatabase};
use serde_json::json;
use shared_model::LibraryId;
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
        BackendCommand::ScanStats { library_id } => {
            let summary = scan_stats(&repositories, &repositories, &LibraryId(library_id.clone()))?;
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
