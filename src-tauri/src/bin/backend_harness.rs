use app_core::archive::{
    archive_snapshot, index_library_archives, normalize_archive_source, normalize_archive_status,
    read_archive_entry_by_source,
};
use app_core::asset::{asset_snapshot_for_library, ensure_media_assets_for_library, resolve_asset};
use app_core::cli::{help_payload, parse_command, run_command, BackendCommand};
use app_core::playback::{
    open_playback_session, playback_pause, playback_probe, playback_seek, playback_status,
    playback_stop,
};
use app_core::scan::{register_library, resume_scan, run_scan, scan_snapshot, scan_stats};
use app_core::subtitle_host::{
    subtitle_get_progress, subtitle_health, subtitle_ping, subtitle_start_session,
    subtitle_stop_session,
};
use app_core::thumbnail::{ensure_thumbnail_for_asset, get_thumbnail, parse_thumbnail_profile};
use media_db::{DatabaseLocation, MediaDatabase};
use mediaplayernext_lib::subtitle_sidecar::development_subtitle_host;
use serde::Deserialize;
use serde_json::json;
use shared_model::{
    AssetId, LibraryId, PlaybackSessionId, SourceId, SubtitleSessionId, TaskId, ThumbnailKey,
};
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
    let db_path = env_path_or_default(
        "MPNEXT_BACKEND_DB_PATH",
        workspace_root().join("data").join("mediaplayernext-dev.db"),
    );
    let thumbnail_cache_root = env_path_or_default(
        "MPNEXT_BACKEND_THUMB_CACHE_ROOT",
        workspace_root().join("data").join("cache").join("thumbs"),
    );
    let playback_sessions_root = workspace_root()
        .join("data")
        .join("cache")
        .join("playback")
        .join("sessions");
    let playback_sessions_root = env_path_or_default(
        "MPNEXT_BACKEND_PLAYBACK_SESSIONS_ROOT",
        playback_sessions_root,
    );
    let normalize_root = env_path_or_default(
        "MPNEXT_BACKEND_NORMALIZE_ROOT",
        workspace_root()
            .join("data")
            .join("cache")
            .join("normalized"),
    );
    let runtime_paths = load_runtime_paths(&config_path)?;

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
                &repositories,
                &SourceId(source_id.clone()),
                entry_path,
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::ArchiveNormalize { source_id } => {
            let summary = normalize_archive_source(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &runtime_paths.sevenz_path,
                &normalize_root,
                &SourceId(source_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::ArchiveNormalizeStatus { task_id } => {
            let summary = normalize_archive_status(&repositories, &TaskId(task_id.clone()))?;
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
        BackendCommand::AssetSnapshot { library_id } => {
            let summary = asset_snapshot_for_library(
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
        BackendCommand::PlaybackProbe { asset_id } => {
            let summary = playback_probe(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &runtime_paths.ffprobe_path,
                &AssetId(asset_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::PlaybackOpen { asset_id } => {
            let summary = open_playback_session(
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &repositories,
                &playback_sessions_root,
                &runtime_paths.mpv_path,
                &AssetId(asset_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::PlaybackStatus { session_id } => {
            let summary = playback_status(
                &playback_sessions_root,
                &PlaybackSessionId(session_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::PlaybackPause { session_id } => {
            let summary = playback_pause(
                &playback_sessions_root,
                &PlaybackSessionId(session_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::PlaybackSeek {
            session_id,
            position_ms,
        } => {
            let summary = playback_seek(
                &playback_sessions_root,
                &PlaybackSessionId(session_id.clone()),
                position_ms.parse::<i64>()?,
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::PlaybackStop { session_id } => {
            let summary = playback_stop(
                &playback_sessions_root,
                &PlaybackSessionId(session_id.clone()),
            )?;
            Some(serde_json::to_value(summary)?)
        }
        BackendCommand::SubtitlePing => {
            let host = development_subtitle_host()?;
            Some(serde_json::to_value(subtitle_ping(&host)?)?)
        }
        BackendCommand::SubtitleHealth => {
            let host = development_subtitle_host()?;
            Some(serde_json::to_value(subtitle_health(&host)?)?)
        }
        BackendCommand::SubtitleStartSession { asset_id } => {
            let host = development_subtitle_host()?;
            let asset_id = asset_id.as_ref().map(|value| AssetId(value.clone()));
            Some(serde_json::to_value(subtitle_start_session(
                &host,
                asset_id.as_ref(),
            )?)?)
        }
        BackendCommand::SubtitleStopSession { session_id } => {
            let host = development_subtitle_host()?;
            Some(serde_json::to_value(subtitle_stop_session(
                &host,
                &SubtitleSessionId(session_id.clone()),
            )?)?)
        }
        BackendCommand::SubtitleGetProgress { session_id } => {
            let host = development_subtitle_host()?;
            Some(serde_json::to_value(subtitle_get_progress(
                &host,
                &SubtitleSessionId(session_id.clone()),
            )?)?)
        }
    };

    let output = run_command(command, config_path, payload)?;
    println!("{}", serde_json::to_string_pretty(&output)?);

    Ok(())
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RuntimePathsConfig {
    ffprobe: Option<String>,
    mpv: Option<String>,
    sevenz: Option<String>,
}

#[derive(Debug, Clone)]
struct RuntimePaths {
    ffprobe_path: PathBuf,
    mpv_path: PathBuf,
    sevenz_path: PathBuf,
}

fn load_runtime_paths(config_path: &PathBuf) -> anyhow::Result<RuntimePaths> {
    let config = if config_path.exists() {
        serde_json::from_slice::<RuntimePathsConfig>(&std::fs::read(config_path)?)?
    } else {
        RuntimePathsConfig::default()
    };

    Ok(RuntimePaths {
        ffprobe_path: env_path_or_config_or_default(
            "MPNEXT_RUNTIME_FFPROBE_PATH",
            config.ffprobe,
            PathBuf::from("C:/Tools/ffmpeg/bin/ffprobe.exe"),
        ),
        mpv_path: env_path_or_config_or_default(
            "MPNEXT_RUNTIME_MPV_PATH",
            config.mpv,
            PathBuf::from("C:/mpv/mpv.exe"),
        ),
        sevenz_path: env_path_or_config_or_default(
            "MPNEXT_RUNTIME_SEVENVZ_PATH",
            config.sevenz,
            PathBuf::from("C:/Program Files/7-Zip/7z.exe"),
        ),
    })
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .expect("workspace root should be available")
}

fn env_path_or_default(name: &str, default: PathBuf) -> PathBuf {
    env::var_os(name).map(PathBuf::from).unwrap_or(default)
}

fn env_path_or_config_or_default(
    env_name: &str,
    config_value: Option<String>,
    default: PathBuf,
) -> PathBuf {
    match env::var_os(env_name) {
        Some(value) => PathBuf::from(value),
        None => config_value.map(PathBuf::from).unwrap_or(default),
    }
}
