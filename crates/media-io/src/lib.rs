pub mod archive;
pub mod normalize;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use shared_model::SourceKind;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateMediaKind {
    Image,
    Video,
    Archive,
    Audio,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredFile {
    pub absolute_path: PathBuf,
    pub normalized_path: String,
    pub file_name: String,
    pub extension: String,
    pub kind: CandidateMediaKind,
    pub size: i64,
    pub mtime_ms: i64,
}

pub fn classify_extension(extension: &str) -> CandidateMediaKind {
    match extension.to_ascii_lowercase().as_str() {
        "jpg" | "jpeg" | "png" | "webp" | "gif" | "bmp" => CandidateMediaKind::Image,
        "zip" | "cbz" | "rar" | "7z" => CandidateMediaKind::Archive,
        "mp4" | "mkv" | "webm" | "avi" => CandidateMediaKind::Video,
        "mp3" | "flac" | "wav" | "m4a" => CandidateMediaKind::Audio,
        _ => CandidateMediaKind::Other,
    }
}

pub fn discover_media_files(root: &Path) -> Result<Vec<DiscoveredFile>> {
    let mut files = Vec::new();
    visit_directory(root, &mut files)?;
    files.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
    Ok(files)
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase()
}

pub fn candidate_kind_to_source_kind(kind: &CandidateMediaKind) -> SourceKind {
    match kind {
        CandidateMediaKind::Image => SourceKind::Image,
        CandidateMediaKind::Video => SourceKind::Video,
        CandidateMediaKind::Archive => SourceKind::Archive,
        CandidateMediaKind::Audio => SourceKind::Audio,
        CandidateMediaKind::Other => SourceKind::Other,
    }
}

pub fn is_primary_archive_extension(extension: &str) -> bool {
    matches!(extension.to_ascii_lowercase().as_str(), "zip" | "cbz")
}

pub fn is_normalizable_archive_extension(extension: &str) -> bool {
    matches!(extension.to_ascii_lowercase().as_str(), "rar" | "7z")
}

fn visit_directory(directory: &Path, files: &mut Vec<DiscoveredFile>) -> Result<()> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name.starts_with('.') {
            continue;
        }

        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            visit_directory(&path, files)?;
            continue;
        }

        if !metadata.is_file() {
            continue;
        }

        let extension = path
            .extension()
            .and_then(|value| value.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();

        let kind = classify_extension(&extension);
        if kind == CandidateMediaKind::Other {
            continue;
        }

        let mtime_ms = metadata
            .modified()?
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as i64;

        files.push(DiscoveredFile {
            absolute_path: path.clone(),
            normalized_path: normalize_path(&path),
            file_name,
            extension,
            kind,
            size: metadata.len() as i64,
            mtime_ms,
        });
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        candidate_kind_to_source_kind, classify_extension, discover_media_files,
        is_normalizable_archive_extension, normalize_path, CandidateMediaKind,
    };
    use shared_model::SourceKind;
    use std::fs;
    use std::path::Path;
    use tempfile::tempdir;

    #[test]
    fn classifies_archive_extensions() {
        assert_eq!(classify_extension("zip"), CandidateMediaKind::Archive);
        assert_eq!(classify_extension("cbz"), CandidateMediaKind::Archive);
        assert!(is_normalizable_archive_extension("rar"));
        assert!(is_normalizable_archive_extension("7z"));
    }

    #[test]
    fn normalizes_windows_style_paths() {
        let normalized = normalize_path(Path::new("Z:\\Library\\Item.PNG"));
        assert_eq!(normalized, "z:/library/item.png");
    }

    #[test]
    fn maps_candidate_kind_to_source_kind() {
        assert_eq!(
            candidate_kind_to_source_kind(&CandidateMediaKind::Image),
            SourceKind::Image
        );
        assert_eq!(
            candidate_kind_to_source_kind(&CandidateMediaKind::Archive),
            SourceKind::Archive
        );
    }

    #[test]
    fn discovers_supported_files_recursively() {
        let temp = tempdir().expect("temporary fixture directory should be created");
        let nested = temp.path().join("nested");
        fs::create_dir_all(&nested).expect("nested directory should be created");
        fs::write(temp.path().join("cover.png"), b"png").expect("png fixture should be written");
        fs::write(nested.join("chapter.cbz"), b"zip").expect("cbz fixture should be written");
        fs::write(temp.path().join("notes.txt"), b"ignore")
            .expect("text fixture should be written");

        let discovered = discover_media_files(temp.path()).expect("file discovery should succeed");

        assert_eq!(discovered.len(), 2);
        assert!(discovered.iter().any(|item| item.file_name == "cover.png"));
        assert!(discovered
            .iter()
            .any(|item| item.file_name == "chapter.cbz"));
    }
}
