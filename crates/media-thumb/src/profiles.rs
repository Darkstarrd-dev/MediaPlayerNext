use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ThumbnailProfile {
    GridSm,
    GridMd,
    DetailMd,
    DetailLg,
}

impl ThumbnailProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GridSm => "grid-sm",
            Self::GridMd => "grid-md",
            Self::DetailMd => "detail-md",
            Self::DetailLg => "detail-lg",
        }
    }

    pub fn target_size(self) -> (u32, u32) {
        match self {
            Self::GridSm => (240, 240),
            Self::GridMd => (480, 480),
            Self::DetailMd => (960, 960),
            Self::DetailLg => (1440, 1440),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThumbnailProfile;

    #[test]
    fn exposes_expected_target_sizes() {
        assert_eq!(ThumbnailProfile::GridSm.target_size(), (240, 240));
        assert_eq!(ThumbnailProfile::GridMd.target_size(), (480, 480));
        assert_eq!(ThumbnailProfile::DetailMd.target_size(), (960, 960));
        assert_eq!(ThumbnailProfile::DetailLg.target_size(), (1440, 1440));
    }
}
