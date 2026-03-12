use anyhow::Context;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

use super::{
    RuntimeStorageConfig, RUNTIME_STORAGE_CONFIG_FILE_NAME, RUNTIME_STORAGE_CONFIG_PATH_ENV,
};
use crate::runtime_storage::filesystem::env_path;

pub(super) fn read_runtime_storage_config(app: &AppHandle) -> anyhow::Result<RuntimeStorageConfig> {
    let config_path = runtime_storage_config_path(app)?;
    load_runtime_storage_config_from_path(&config_path)
}

pub(super) fn write_runtime_storage_config(
    app: &AppHandle,
    config: &RuntimeStorageConfig,
) -> anyhow::Result<()> {
    let config_path = runtime_storage_config_path(app)?;
    write_runtime_storage_config_to_path(&config_path, config)
}

pub(super) fn runtime_storage_config_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
    if let Some(path) = env_path(RUNTIME_STORAGE_CONFIG_PATH_ENV) {
        return Ok(path);
    }

    if tauri::is_dev() {
        return Ok(crate::workspace_root()
            .join("data")
            .join(RUNTIME_STORAGE_CONFIG_FILE_NAME));
    }

    Ok(app
        .path()
        .app_local_data_dir()
        .context("resolve runtime storage app local data dir")?
        .join(RUNTIME_STORAGE_CONFIG_FILE_NAME))
}

pub(super) fn load_runtime_storage_config_from_path(
    path: &Path,
) -> anyhow::Result<RuntimeStorageConfig> {
    if !path.exists() {
        return Ok(RuntimeStorageConfig::default());
    }

    let raw = fs::read(path)
        .with_context(|| format!("read runtime storage config: {}", path.display()))?;
    serde_json::from_slice::<RuntimeStorageConfig>(&raw)
        .with_context(|| format!("parse runtime storage config: {}", path.display()))
}

pub(super) fn write_runtime_storage_config_to_path(
    path: &Path,
    config: &RuntimeStorageConfig,
) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create runtime storage config dir: {}", parent.display()))?;
    }

    let payload = serde_json::to_vec_pretty(config).context("serialize runtime storage config")?;
    fs::write(path, payload)
        .with_context(|| format!("write runtime storage config: {}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::{
        load_runtime_storage_config_from_path, write_runtime_storage_config_to_path,
        RuntimeStorageConfig,
    };
    use tempfile::tempdir;

    #[test]
    fn reads_back_written_runtime_storage_config() {
        let temp_result = tempdir();
        assert!(temp_result.is_ok());
        let Some(temp) = temp_result.ok() else {
            return;
        };
        let config_path = temp.path().join("runtime-storage-paths.json");
        let config = RuntimeStorageConfig {
            database_dir: Some(temp.path().join("sql").display().to_string()),
            thumbnail_cache_dir: Some(temp.path().join("thumbs").display().to_string()),
        };

        let write_result = write_runtime_storage_config_to_path(&config_path, &config);
        assert!(write_result.is_ok());

        let loaded_result = load_runtime_storage_config_from_path(&config_path);
        assert!(loaded_result.is_ok());
        let Some(loaded) = loaded_result.ok() else {
            return;
        };

        assert_eq!(loaded, config);
    }
}
