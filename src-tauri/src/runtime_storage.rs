use anyhow::{anyhow, Context};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

const RUNTIME_STORAGE_CONFIG_FILE_NAME: &str = "runtime-storage-paths.json";
const DEVELOPMENT_DATABASE_FILE_NAME: &str = "mediaplayernext-dev.db";
const PACKAGED_DATABASE_FILE_NAME: &str = "mediaplayernext.db";
const RUNTIME_STORAGE_CONFIG_PATH_ENV: &str = "MPNEXT_RUNTIME_STORAGE_CONFIG_PATH";
const RUNTIME_DEFAULT_DATA_DIR_ENV: &str = "MPNEXT_RUNTIME_DEFAULT_DATA_DIR";
const RUNTIME_DEFAULT_CACHE_DIR_ENV: &str = "MPNEXT_RUNTIME_DEFAULT_CACHE_DIR";

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
    let config_path = runtime_storage_config_path(app)?;
    let config = load_runtime_storage_config_from_path(&config_path)?;
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
    let config_path = runtime_storage_config_path(app)?;
    load_runtime_storage_config_from_path(&config_path)
}

fn runtime_storage_config_path(app: &AppHandle) -> anyhow::Result<PathBuf> {
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

fn write_runtime_storage_config(
    app: &AppHandle,
    config: &RuntimeStorageConfig,
) -> anyhow::Result<()> {
    let config_path = runtime_storage_config_path(app)?;
    write_runtime_storage_config_to_path(&config_path, config)
}

fn load_runtime_storage_config_from_path(path: &Path) -> anyhow::Result<RuntimeStorageConfig> {
    if !path.exists() {
        return Ok(RuntimeStorageConfig::default());
    }

    let raw = fs::read(path)
        .with_context(|| format!("read runtime storage config: {}", path.display()))?;
    serde_json::from_slice::<RuntimeStorageConfig>(&raw)
        .with_context(|| format!("parse runtime storage config: {}", path.display()))
}

fn write_runtime_storage_config_to_path(
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

fn remove_database_artifacts(database_path: &Path) -> anyhow::Result<()> {
    for artifact in database_artifact_paths(database_path) {
        remove_file_if_exists(&artifact)?;
    }

    Ok(())
}

fn move_database_artifacts(
    source_database_path: &Path,
    target_database_path: &Path,
) -> anyhow::Result<()> {
    if source_database_path == target_database_path {
        return Ok(());
    }

    let existing_pairs = database_artifact_paths(source_database_path)
        .into_iter()
        .zip(database_artifact_paths(target_database_path))
        .filter(|(source, _)| source.exists())
        .collect::<Vec<_>>();

    if existing_pairs.is_empty() {
        return Ok(());
    }

    for (_, target) in &existing_pairs {
        if target.exists() {
            return Err(anyhow!(
                "target database artifact already exists: {}",
                target.display()
            ));
        }
    }

    for (source, target) in existing_pairs {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("create database target dir: {}", parent.display()))?;
        }

        move_file(&source, &target)?;
    }

    Ok(())
}

fn move_file(source: &Path, target: &Path) -> anyhow::Result<()> {
    match fs::rename(source, target) {
        Ok(()) => Ok(()),
        Err(_) => {
            fs::copy(source, target).with_context(|| {
                format!(
                    "copy database artifact from {} to {}",
                    source.display(),
                    target.display()
                )
            })?;
            fs::remove_file(source)
                .with_context(|| format!("remove source database artifact: {}", source.display()))
        }
    }
}

fn database_artifact_paths(database_path: &Path) -> Vec<PathBuf> {
    let database = database_path.to_path_buf();
    let wal = PathBuf::from(format!("{}-wal", database_path.display()));
    let shm = PathBuf::from(format!("{}-shm", database_path.display()));
    vec![database, wal, shm]
}

fn remove_dir_all_if_exists(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_dir_all(path).with_context(|| format!("remove directory tree: {}", path.display()))
}

fn remove_file_if_exists(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path).with_context(|| format!("remove file: {}", path.display()))
}

fn normalize_directory_path(raw_path: &str) -> anyhow::Result<PathBuf> {
    let trimmed_path = raw_path.trim();
    if trimmed_path.is_empty() {
        return Err(anyhow!("runtime storage directory cannot be empty"));
    }

    let candidate = PathBuf::from(trimmed_path);
    if candidate.is_absolute() {
        return Ok(candidate);
    }

    Ok(env::current_dir()
        .context("resolve current working directory for runtime storage path")?
        .join(candidate))
}

fn packaged_local_data_dir(app: &AppHandle) -> anyhow::Result<PathBuf> {
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

fn packaged_cache_dir(app: &AppHandle) -> anyhow::Result<PathBuf> {
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

fn default_database_path(packaged_local_data_dir: &Path) -> PathBuf {
    if tauri::is_dev() {
        return crate::workspace_root()
            .join("data")
            .join(DEVELOPMENT_DATABASE_FILE_NAME);
    }

    packaged_local_data_dir.join(PACKAGED_DATABASE_FILE_NAME)
}

fn default_database_file_name() -> &'static str {
    if tauri::is_dev() {
        DEVELOPMENT_DATABASE_FILE_NAME
    } else {
        PACKAGED_DATABASE_FILE_NAME
    }
}

fn default_thumbnail_cache_root(packaged_cache_dir: &Path) -> PathBuf {
    if tauri::is_dev() {
        return crate::workspace_root()
            .join("data")
            .join("cache")
            .join("thumbs");
    }

    packaged_cache_dir.join("thumbs")
}

fn default_normalize_root(packaged_cache_dir: &Path) -> PathBuf {
    if tauri::is_dev() {
        return crate::workspace_root()
            .join("data")
            .join("cache")
            .join("normalized");
    }

    packaged_cache_dir.join("normalized")
}

fn default_playback_sessions_root(packaged_cache_dir: &Path) -> PathBuf {
    if tauri::is_dev() {
        return crate::workspace_root()
            .join("data")
            .join("cache")
            .join("playback")
            .join("sessions");
    }

    packaged_cache_dir.join("playback").join("sessions")
}

fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn env_path(env_name: &str) -> Option<PathBuf> {
    env::var_os(env_name).map(PathBuf::from)
}

#[cfg(test)]
mod tests {
    use super::{
        load_runtime_storage_config_from_path, move_database_artifacts, remove_database_artifacts,
        write_runtime_storage_config_to_path, RuntimeStorageConfig,
    };
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn moves_database_file_and_sidecars() {
        let temp = tempdir().expect("tempdir should exist");
        let source_dir = temp.path().join("source");
        let target_dir = temp.path().join("target");
        fs::create_dir_all(&source_dir).expect("source dir should exist");
        fs::create_dir_all(&target_dir).expect("target dir should exist");

        let source_db = source_dir.join("mediaplayernext-dev.db");
        let target_db = target_dir.join("mediaplayernext-dev.db");
        fs::write(&source_db, b"db-bytes").expect("db should exist");
        fs::write(format!("{}-wal", source_db.display()), b"wal-bytes").expect("wal should exist");
        fs::write(format!("{}-shm", source_db.display()), b"shm-bytes").expect("shm should exist");

        move_database_artifacts(&source_db, &target_db).expect("artifacts should move");

        assert!(!source_db.exists());
        assert!(target_db.exists());
        assert!(target_dir.join("mediaplayernext-dev.db-wal").exists());
        assert!(target_dir.join("mediaplayernext-dev.db-shm").exists());
    }

    #[test]
    fn removes_database_file_and_sidecars() {
        let temp = tempdir().expect("tempdir should exist");
        let database_path = temp.path().join("mediaplayernext-dev.db");
        fs::write(&database_path, b"db-bytes").expect("db should exist");
        fs::write(format!("{}-wal", database_path.display()), b"wal-bytes")
            .expect("wal should exist");
        fs::write(format!("{}-shm", database_path.display()), b"shm-bytes")
            .expect("shm should exist");

        remove_database_artifacts(&database_path).expect("artifacts should be removed");

        assert!(!database_path.exists());
        assert!(!temp.path().join("mediaplayernext-dev.db-wal").exists());
        assert!(!temp.path().join("mediaplayernext-dev.db-shm").exists());
    }

    #[test]
    fn reads_back_written_runtime_storage_config() {
        let temp = tempdir().expect("tempdir should exist");
        let config_path = temp.path().join("runtime-storage-paths.json");
        let config = RuntimeStorageConfig {
            database_dir: Some(temp.path().join("sql").display().to_string()),
            thumbnail_cache_dir: Some(temp.path().join("thumbs").display().to_string()),
        };

        write_runtime_storage_config_to_path(&config_path, &config)
            .expect("config should be written");
        let loaded = load_runtime_storage_config_from_path(&config_path)
            .expect("config should be read back");

        assert_eq!(loaded, config);
    }
}
