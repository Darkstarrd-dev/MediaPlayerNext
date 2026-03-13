use crate::profiles::ThumbnailProfile;
use crate::source::ThumbnailSource;
use anyhow::Result;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailCacheLayout {
    pub root_dir: PathBuf,
    pub relative_path: PathBuf,
    pub absolute_path: PathBuf,
}

pub fn thumbnail_key_for_source(
    source: &ThumbnailSource,
    profile: ThumbnailProfile,
    pipeline_version: &str,
) -> String {
    let digest = Sha256::digest(format!(
        "{}\n{}\n{}\n{}",
        source.source_identity(),
        source.source_revision(),
        profile.as_str(),
        pipeline_version,
    ));

    let mut key = String::with_capacity(digest.len() * 2);
    for byte in digest {
        key.push_str(&format!("{byte:02x}"));
    }
    key
}

pub fn cache_path_for_key(cache_root: &Path, thumbnail_key: &str) -> ThumbnailCacheLayout {
    cache_path_for_key_with_extension(cache_root, thumbnail_key, "jpg")
}

pub fn cache_path_for_key_with_extension(
    cache_root: &Path,
    thumbnail_key: &str,
    extension: &str,
) -> ThumbnailCacheLayout {
    let level_one = &thumbnail_key[0..2];
    let level_two = &thumbnail_key[2..4];
    let relative_path = PathBuf::from(level_one)
        .join(level_two)
        .join(format!("{thumbnail_key}.{extension}"));

    ThumbnailCacheLayout {
        root_dir: cache_root.to_path_buf(),
        absolute_path: cache_root.join(&relative_path),
        relative_path,
    }
}

pub fn write_thumbnail_atomically(target_path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = target_path
        .parent()
        .expect("thumbnail target path should always have parent directory");
    fs::create_dir_all(parent)?;

    let temp_extension = target_path
        .extension()
        .and_then(|value| value.to_str())
        .map(|value| format!("{value}.tmp"))
        .unwrap_or_else(|| "tmp".to_string());
    let temp_path = target_path.with_extension(temp_extension);
    fs::write(&temp_path, bytes)?;
    if target_path.exists() {
        fs::remove_file(target_path)?;
    }
    fs::rename(temp_path, target_path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        cache_path_for_key, cache_path_for_key_with_extension, thumbnail_key_for_source,
        write_thumbnail_atomically,
    };
    use crate::profiles::ThumbnailProfile;
    use crate::source::ThumbnailSource;
    use tempfile::tempdir;

    #[test]
    fn builds_stable_cache_key() {
        let source = ThumbnailSource::FilePath {
            source_identity: "source-1".to_string(),
            source_revision: "fp-1".to_string(),
            file_path: "Z:/Library/cover.png".to_string(),
        };

        let first = thumbnail_key_for_source(&source, ThumbnailProfile::GridSm, "v1");
        let second = thumbnail_key_for_source(&source, ThumbnailProfile::GridSm, "v1");
        let third = thumbnail_key_for_source(&source, ThumbnailProfile::GridMd, "v1");

        assert_eq!(first, second);
        assert_ne!(first, third);
    }

    #[test]
    fn maps_key_to_two_level_cache_path() {
        let layout = cache_path_for_key(
            tempdir().expect("tempdir").path(),
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        );

        assert!(layout.relative_path.ends_with(
            "ab/cd/abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789.jpg"
        ));
    }

    #[test]
    fn maps_key_to_custom_extension_cache_path() {
        let layout = cache_path_for_key_with_extension(
            tempdir().expect("tempdir").path(),
            "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
            "webp",
        );

        assert!(layout.relative_path.ends_with(
            "ab/cd/abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789.webp"
        ));
    }

    #[test]
    fn writes_thumbnail_atomically() {
        let temp = tempdir().expect("tempdir");
        let target = temp.path().join("ab").join("cd").join("thumb.jpg");
        write_thumbnail_atomically(&target, b"jpeg").expect("atomic write should succeed");

        let written = std::fs::read(&target).expect("thumbnail should exist");
        assert_eq!(written, b"jpeg");
    }
}
