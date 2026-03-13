use crate::pipeline::JpegThumbnailConfig;
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

    pub fn jpeg_quality(self) -> u8 {
        match self {
            Self::GridSm => 60,
            Self::GridMd => 70,
            Self::DetailMd => 75,
            Self::DetailLg => 80,
        }
    }

    pub fn to_jpeg_config(self) -> JpegThumbnailConfig {
        let (max_width, max_height) = self.target_size();
        JpegThumbnailConfig::new(max_width, max_height, self.jpeg_quality())
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

    #[test]
    fn exposes_expected_jpeg_quality() {
        assert_eq!(ThumbnailProfile::GridSm.jpeg_quality(), 60);
        assert_eq!(ThumbnailProfile::GridMd.jpeg_quality(), 70);
        assert_eq!(ThumbnailProfile::DetailMd.jpeg_quality(), 75);
        assert_eq!(ThumbnailProfile::DetailLg.jpeg_quality(), 80);
    }
}
