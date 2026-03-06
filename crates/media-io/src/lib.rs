use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateMediaKind {
    Image,
    Video,
    Archive,
    Audio,
    Other,
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

#[cfg(test)]
mod tests {
    use super::{classify_extension, CandidateMediaKind};

    #[test]
    fn classifies_archive_extensions() {
        assert_eq!(classify_extension("zip"), CandidateMediaKind::Archive);
        assert_eq!(classify_extension("cbz"), CandidateMediaKind::Archive);
    }
}
