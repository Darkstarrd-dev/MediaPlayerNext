use serde::{Deserialize, Serialize};

macro_rules! id_type {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub String);

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self(value.to_string())
            }
        }
    };
}

id_type!(LibraryId);
id_type!(SourceId);
id_type!(MediaSourceId);
id_type!(ImageItemId);
id_type!(ArchiveId);
id_type!(ArchiveEntryId);
id_type!(AssetId);
id_type!(ThumbnailKey);
id_type!(TaskId);
id_type!(PlaybackSessionId);
id_type!(SubtitleSessionId);
