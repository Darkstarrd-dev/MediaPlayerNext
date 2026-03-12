mod config;
mod defaults;
mod filesystem;

use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;
use tauri::AppHandle;

use config::write_runtime_storage_config;
use defaults::{
    default_database_file_name, default_database_path, default_normalize_root,
    default_playback_sessions_root, default_thumbnail_cache_root, packaged_cache_dir,
    packaged_local_data_dir,
};
use filesystem::{
    move_database_artifacts, normalize_directory_path, path_to_string, remove_database_artifacts,
    remove_dir_all_if_exists, remove_file_if_exists,
};

pub(super) const RUNTIME_STORAGE_CONFIG_FILE_NAME: &str = "runtime-storage-paths.json";
pub(super) const DEVELOPMENT_DATABASE_FILE_NAME: &str = "mediaplayernext-dev.db";
pub(super) const PACKAGED_DATABASE_FILE_NAME: &str = "mediaplayernext.db";
pub(super) const RUNTIME_STORAGE_CONFIG_PATH_ENV: &str = "MPNEXT_RUNTIME_STORAGE_CONFIG_PATH";
pub(super) const RUNTIME_DEFAULT_DATA_DIR_ENV: &str = "MPNEXT_RUNTIME_DEFAULT_DATA_DIR";
pub(super) const RUNTIME_DEFAULT_CACHE_DIR_ENV: &str = "MPNEXT_RUNTIME_DEFAULT_CACHE_DIR";

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct RuntimeStorageConfig {
    pub database_dir: Option<String>,
    pub thumbnail_cache_dir: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ResolvedRuntimeStoragePaths {
    pub database_path: PathBuf,
    pub thumbnail_cache_root: PathBuf,
    pub normalize_root: PathBuf,
    pub playback_sessions_root: PathBuf,
    pub config_path: PathBuf,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeInfoPayload {
    pub database_path: String,
    pub thumbnail_cache_path: String,
}

pub fn read_runtime_info(app: &AppHandle) -> anyhow::Result<RuntimeInfoPayload> {
    let resolved = resolve_runtime_storage_paths(app)?;
    Ok(RuntimeInfoPayload {
        database_path: path_to_string(&resolved.database_path),
        thumbnail_cache_path: path_to_string(&resolved.thumbnail_cache_root),
    })
}

pub fn set_runtime_storage_paths(
    app: &AppHandle,
    database_dir: Option<String>,
    thumbnail_cache_dir: Option<String>,
) -> anyhow::Result<RuntimeInfoPayload> {
    if database_dir.is_none() && thumbnail_cache_dir.is_none() {
        return Err(anyhow!(
            "at least one runtime storage path must be provided"
        ));
    }

    let current = resolve_runtime_storage_paths(app)?;
    let mut config = read_runtime_storage_config(app)?;

    if let Some(raw_database_dir) = database_dir {
        let target_database_dir = normalize_directory_path(&raw_database_dir)?;
        fs::create_dir_all(&target_database_dir).with_context(|| {
            format!(
                "create runtime database directory: {}",
                target_database_dir.display()
            )
        })?;

        let target_database_path = target_database_dir.join(default_database_file_name());
        move_database_artifacts(&current.database_path, &target_database_path)?;
        config.database_dir = Some(path_to_string(&target_database_dir));
    }

    if let Some(raw_thumbnail_cache_dir) = thumbnail_cache_dir {
        let target_thumbnail_cache_dir = normalize_directory_path(&raw_thumbnail_cache_dir)?;
        fs::create_dir_all(&target_thumbnail_cache_dir).with_context(|| {
            format!(
                "create runtime thumbnail cache directory: {}",
                target_thumbnail_cache_dir.display()
            )
        })?;
        config.thumbnail_cache_dir = Some(path_to_string(&target_thumbnail_cache_dir));
    }

    write_runtime_storage_config(app, &config)?;
    read_runtime_info(app)
}

pub fn clear_database(app: &AppHandle) -> anyhow::Result<()> {
    let resolved = resolve_runtime_storage_paths(app)?;

    remove_database_artifacts(&resolved.database_path)?;
    remove_dir_all_if_exists(&resolved.thumbnail_cache_root)?;
    remove_dir_all_if_exists(&resolved.normalize_root)?;
    remove_dir_all_if_exists(&resolved.playback_sessions_root)?;
    remove_file_if_exists(&resolved.config_path)?;

    Ok(())
}

pub fn resolve_runtime_storage_paths(
    app: &AppHandle,
) -> anyhow::Result<ResolvedRuntimeStoragePaths> {
    let config_path = config::runtime_storage_config_path(app)?;
    let config = config::load_runtime_storage_config_from_path(&config_path)?;
    let packaged_local_data_dir = packaged_local_data_dir(app)?;
    let packaged_cache_dir = packaged_cache_dir(app)?;

    let database_path = match env::var_os("MPNEXT_BACKEND_DB_PATH") {
        Some(path) => PathBuf::from(path),
        None => match config.database_dir.as_deref() {
            Some(directory) => {
                normalize_directory_path(directory)?.join(default_database_file_name())
            }
            None => default_database_path(&packaged_local_data_dir),
        },
    };

    let thumbnail_cache_root = match env::var_os("MPNEXT_BACKEND_THUMB_CACHE_ROOT") {
        Some(path) => PathBuf::from(path),
        None => match config.thumbnail_cache_dir.as_deref() {
            Some(directory) => normalize_directory_path(directory)?,
            None => default_thumbnail_cache_root(&packaged_cache_dir),
        },
    };

    let normalize_root = match env::var_os("MPNEXT_BACKEND_NORMALIZE_ROOT") {
        Some(path) => PathBuf::from(path),
        None => default_normalize_root(&packaged_cache_dir),
    };

    let playback_sessions_root = match env::var_os("MPNEXT_BACKEND_PLAYBACK_SESSIONS_ROOT") {
        Some(path) => PathBuf::from(path),
        None => default_playback_sessions_root(&packaged_cache_dir),
    };

    Ok(ResolvedRuntimeStoragePaths {
        database_path,
        thumbnail_cache_root,
        normalize_root,
        playback_sessions_root,
        config_path,
    })
}

pub fn resolve_database_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
    Ok(resolve_runtime_storage_paths(app)?.database_path)
}

pub fn read_runtime_storage_config(app: &AppHandle) -> anyhow::Result<RuntimeStorageConfig> {
    config::read_runtime_storage_config(app)
}
