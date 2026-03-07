use serde::{Deserialize, Serialize};
use shared_model::SourceId;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedArchiveLayout {
    pub source_id: String,
    pub revision: String,
    pub root_dir: PathBuf,
    pub work_dir: PathBuf,
    pub extracted_dir: PathBuf,
    pub output_zip_path: PathBuf,
}

pub fn normalized_archive_layout(
    normalize_root: &Path,
    source_id: &SourceId,
    revision: &str,
) -> NormalizedArchiveLayout {
    let revision_key = format!(
        "{:016x}",
        stable_hash(&format!("{}::{revision}", source_id.0))
    );
    let work_dir = normalize_root.join(&source_id.0).join(&revision_key);

    NormalizedArchiveLayout {
        source_id: source_id.0.clone(),
        revision: revision.to_string(),
        root_dir: normalize_root.to_path_buf(),
        extracted_dir: work_dir.join("extract"),
        output_zip_path: work_dir.join("normalized.zip"),
        work_dir,
    }
}

fn stable_hash(value: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[cfg(test)]
mod tests {
    use super::normalized_archive_layout;
    use shared_model::SourceId;
    use std::path::Path;

    #[test]
    fn builds_stable_layout_paths() {
        let first = normalized_archive_layout(
            Path::new("Z:/cache/normalized"),
            &SourceId("source_rar_primary".to_string()),
            "fp:1",
        );
        let second = normalized_archive_layout(
            Path::new("Z:/cache/normalized"),
            &SourceId("source_rar_primary".to_string()),
            "fp:1",
        );

        assert_eq!(first.work_dir, second.work_dir);
        assert!(first.output_zip_path.ends_with("normalized.zip"));
        assert!(first.extracted_dir.ends_with("extract"));
    }
}
