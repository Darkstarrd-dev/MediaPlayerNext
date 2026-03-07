use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ThumbnailSourceKind {
    FilePath,
    ArchiveEntry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ThumbnailSource {
    FilePath {
        source_identity: String,
        source_revision: String,
        file_path: String,
    },
    ArchiveEntry {
        source_identity: String,
        source_revision: String,
        archive_path: String,
        entry_path: String,
    },
}

impl ThumbnailSource {
    pub fn kind(&self) -> ThumbnailSourceKind {
        match self {
            Self::FilePath { .. } => ThumbnailSourceKind::FilePath,
            Self::ArchiveEntry { .. } => ThumbnailSourceKind::ArchiveEntry,
        }
    }

    pub fn source_identity(&self) -> &str {
        match self {
            Self::FilePath {
                source_identity, ..
            }
            | Self::ArchiveEntry {
                source_identity, ..
            } => source_identity,
        }
    }

    pub fn source_revision(&self) -> &str {
        match self {
            Self::FilePath {
                source_revision, ..
            }
            | Self::ArchiveEntry {
                source_revision, ..
            } => source_revision,
        }
    }
}
