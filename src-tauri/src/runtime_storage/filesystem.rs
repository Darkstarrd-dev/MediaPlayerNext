use anyhow::{anyhow, Context};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

pub(super) fn remove_database_artifacts(database_path: &Path) -> anyhow::Result<()> {
    for artifact in database_artifact_paths(database_path) {
        remove_file_if_exists(&artifact)?;
    }

    Ok(())
}

pub(super) fn move_database_artifacts(
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

pub(super) fn remove_dir_all_if_exists(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_dir_all(path).with_context(|| format!("remove directory tree: {}", path.display()))
}

pub(super) fn remove_file_if_exists(path: &Path) -> anyhow::Result<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_file(path).with_context(|| format!("remove file: {}", path.display()))
}

pub(super) fn normalize_directory_path(raw_path: &str) -> anyhow::Result<PathBuf> {
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

pub(super) fn path_to_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub(super) fn env_path(env_name: &str) -> Option<PathBuf> {
    env::var_os(env_name).map(PathBuf::from)
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

#[cfg(test)]
mod tests {
    use super::{move_database_artifacts, remove_database_artifacts};
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn moves_database_file_and_sidecars() {
        let temp_result = tempdir();
        assert!(temp_result.is_ok());
        let Some(temp) = temp_result.ok() else {
            return;
        };

        let source_dir = temp.path().join("source");
        let target_dir = temp.path().join("target");
        let source_dir_result = fs::create_dir_all(&source_dir);
        let target_dir_result = fs::create_dir_all(&target_dir);
        assert!(source_dir_result.is_ok());
        assert!(target_dir_result.is_ok());

        let source_db = source_dir.join("mediaplayernext-dev.db");
        let target_db = target_dir.join("mediaplayernext-dev.db");
        let write_db_result = fs::write(&source_db, b"db-bytes");
        let write_wal_result = fs::write(format!("{}-wal", source_db.display()), b"wal-bytes");
        let write_shm_result = fs::write(format!("{}-shm", source_db.display()), b"shm-bytes");
        assert!(write_db_result.is_ok());
        assert!(write_wal_result.is_ok());
        assert!(write_shm_result.is_ok());

        let move_result = move_database_artifacts(&source_db, &target_db);
        assert!(move_result.is_ok());

        assert!(!source_db.exists());
        assert!(target_db.exists());
        assert!(target_dir.join("mediaplayernext-dev.db-wal").exists());
        assert!(target_dir.join("mediaplayernext-dev.db-shm").exists());
    }

    #[test]
    fn removes_database_file_and_sidecars() {
        let temp_result = tempdir();
        assert!(temp_result.is_ok());
        let Some(temp) = temp_result.ok() else {
            return;
        };

        let database_path = temp.path().join("mediaplayernext-dev.db");
        let write_db_result = fs::write(&database_path, b"db-bytes");
        let write_wal_result = fs::write(format!("{}-wal", database_path.display()), b"wal-bytes");
        let write_shm_result = fs::write(format!("{}-shm", database_path.display()), b"shm-bytes");
        assert!(write_db_result.is_ok());
        assert!(write_wal_result.is_ok());
        assert!(write_shm_result.is_ok());

        let remove_result = remove_database_artifacts(&database_path);
        assert!(remove_result.is_ok());

        assert!(!database_path.exists());
        assert!(!temp.path().join("mediaplayernext-dev.db-wal").exists());
        assert!(!temp.path().join("mediaplayernext-dev.db-shm").exists());
    }
}
