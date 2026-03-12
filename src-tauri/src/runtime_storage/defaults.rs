use anyhow::Context;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

use super::{
    DEVELOPMENT_DATABASE_FILE_NAME, PACKAGED_DATABASE_FILE_NAME, RUNTIME_DEFAULT_CACHE_DIR_ENV,
    RUNTIME_DEFAULT_DATA_DIR_ENV,
};
use crate::runtime_storage::filesystem::env_path;

pub(super) fn packaged_local_data_dir(app: &AppHandle) -> anyhow::Result<PathBuf> {
    if let Some(path) = env_path(RUNTIME_DEFAULT_DATA_DIR_ENV) {
        return Ok(path);
    }

    if tauri::is_dev() {
        return Ok(crate::workspace_root().join("data"));
    }

    app.path()
        .app_local_data_dir()
        .context("resolve packaged app local data dir")
}

pub(super) fn packaged_cache_dir(app: &AppHandle) -> anyhow::Result<PathBuf> {
    if let Some(path) = env_path(RUNTIME_DEFAULT_CACHE_DIR_ENV) {
        return Ok(path);
    }

    if tauri::is_dev() {
        return Ok(crate::workspace_root().join("data").join("cache"));
    }

    app.path()
        .app_cache_dir()
        .context("resolve packaged app cache dir")
}

pub(super) fn default_database_path(packaged_local_data_dir: &Path) -> PathBuf {
    if tauri::is_dev() {
        return crate::workspace_root()
            .join("data")
            .join(DEVELOPMENT_DATABASE_FILE_NAME);
    }

    packaged_local_data_dir.join(PACKAGED_DATABASE_FILE_NAME)
}

pub(super) fn default_database_file_name() -> &'static str {
    if tauri::is_dev() {
        DEVELOPMENT_DATABASE_FILE_NAME
    } else {
        PACKAGED_DATABASE_FILE_NAME
    }
}

pub(super) fn default_thumbnail_cache_root(packaged_cache_dir: &Path) -> PathBuf {
    if tauri::is_dev() {
        return crate::workspace_root()
            .join("data")
            .join("cache")
            .join("thumbs");
    }

    packaged_cache_dir.join("thumbs")
}

pub(super) fn default_normalize_root(packaged_cache_dir: &Path) -> PathBuf {
    if tauri::is_dev() {
        return crate::workspace_root()
            .join("data")
            .join("cache")
            .join("normalized");
    }

    packaged_cache_dir.join("normalized")
}

pub(super) fn default_playback_sessions_root(packaged_cache_dir: &Path) -> PathBuf {
    if tauri::is_dev() {
        return crate::workspace_root()
            .join("data")
            .join("cache")
            .join("playback")
            .join("sessions");
    }

    packaged_cache_dir.join("playback").join("sessions")
}
