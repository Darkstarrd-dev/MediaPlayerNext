use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{path::BaseDirectory, AppHandle, Manager};

use super::SubtitleSidecarRuntime;

const BUNDLED_SUBTITLE_ENTRY_PATH: &str = "sidecar/index.js";

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalPathsConfig {
    node: Option<String>,
}

pub(super) fn ensure_runtime_ready(runtime: &SubtitleSidecarRuntime) -> Result<()> {
    if runtime.entry_path.exists() {
        if let Some(parent) = runtime.sessions_root.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&runtime.sessions_root)?;
        return Ok(());
    }

    Err(anyhow!(
        "subtitle sidecar entry missing: {} (run `npm run build --workspace @mediaplayernext/subtitle-sidecar` first)",
        runtime.entry_path.display()
    ))
}

pub(super) fn load_node_path(config_path: &Path) -> Result<PathBuf> {
    let config = if config_path.exists() {
        serde_json::from_slice::<LocalPathsConfig>(&fs::read(config_path)?)?
    } else {
        LocalPathsConfig::default()
    };

    Ok(PathBuf::from(
        config.node.unwrap_or_else(|| "node".to_string()),
    ))
}

pub(super) fn resolve_tauri_subtitle_entry_path(app: &AppHandle) -> Result<PathBuf> {
    if let Some(path) = env_path("MPNEXT_SUBTITLE_ENTRY_PATH") {
        return Ok(path);
    }

    if tauri::is_dev() {
        return Ok(development_subtitle_entry_path());
    }

    app.path()
        .resolve(BUNDLED_SUBTITLE_ENTRY_PATH, BaseDirectory::Resource)
        .context("resolve bundled subtitle sidecar entry")
}

pub(super) fn resolve_tauri_subtitle_sessions_root(app: &AppHandle) -> Result<PathBuf> {
    if let Some(path) = env_path("MPNEXT_SUBTITLE_SESSIONS_ROOT") {
        return Ok(path);
    }

    if tauri::is_dev() {
        return Ok(development_subtitle_sessions_root());
    }

    Ok(app
        .path()
        .app_cache_dir()
        .context("resolve subtitle sidecar app cache dir")?
        .join("subtitle")
        .join("sessions"))
}

pub(super) fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")))
}

pub(super) fn development_subtitle_entry_path() -> PathBuf {
    workspace_root()
        .join("apps")
        .join("subtitle-sidecar")
        .join("dist")
        .join("src")
        .join("index.js")
}

pub(super) fn development_subtitle_sessions_root() -> PathBuf {
    workspace_root()
        .join("data")
        .join("cache")
        .join("subtitle")
        .join("sessions")
}

fn env_path(name: &str) -> Option<PathBuf> {
    env::var_os(name).map(PathBuf::from)
}

pub(super) fn env_path_or_default(name: &str, default: PathBuf) -> PathBuf {
    env_path(name).unwrap_or(default)
}

pub(super) fn env_path_or_else<F>(name: &str, fallback: F) -> Result<PathBuf>
where
    F: FnOnce() -> Result<PathBuf>,
{
    match env_path(name) {
        Some(value) => Ok(value),
        None => fallback(),
    }
}
