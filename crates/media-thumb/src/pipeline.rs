use crate::profiles::ThumbnailProfile;
use anyhow::Result;
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use image::{DynamicImage, ExtendedColorType, GenericImageView, ImageEncoder, ImageReader};
use std::io::Cursor;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailRender {
    pub width: u32,
    pub height: u32,
    pub format: &'static str,
    pub bytes: Vec<u8>,
}

pub fn render_thumbnail_from_path(
    path: &std::path::Path,
    profile: ThumbnailProfile,
) -> Result<ThumbnailRender> {
    let image = ImageReader::open(path)?.decode()?;
    render_thumbnail(image, profile)
}

pub fn render_thumbnail_from_memory(
    bytes: &[u8],
    profile: ThumbnailProfile,
) -> Result<ThumbnailRender> {
    let image = image::load_from_memory(bytes)?;
    render_thumbnail(image, profile)
}

fn render_thumbnail(image: DynamicImage, profile: ThumbnailProfile) -> Result<ThumbnailRender> {
    let (target_width, target_height) = profile.target_size();
    let resized = image.resize(target_width, target_height, FilterType::Lanczos3);
    let (width, height) = resized.dimensions();
    let rgba = resized.to_rgba8();
    let mut bytes = Cursor::new(Vec::new());
    WebPEncoder::new_lossless(&mut bytes).write_image(
        rgba.as_raw(),
        width,
        height,
        ExtendedColorType::Rgba8,
    )?;

    Ok(ThumbnailRender {
        width,
        height,
        format: "webp",
        bytes: bytes.into_inner(),
    })
}

#[cfg(test)]
mod tests {
    use super::{render_thumbnail_from_memory, render_thumbnail_from_path};
    use crate::profiles::ThumbnailProfile;
    use image::{ImageBuffer, Rgba};
    use tempfile::tempdir;

    #[test]
    fn renders_thumbnail_from_file_path() {
        let temp = tempdir().expect("tempdir");
        let source_path = temp.path().join("source.png");
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(1200, 800, Rgba([255, 0, 0, 255]));
        image
            .save(&source_path)
            .expect("fixture image should be written");

        let thumbnail = render_thumbnail_from_path(&source_path, ThumbnailProfile::GridSm)
            .expect("thumbnail render should succeed");

        assert_eq!(thumbnail.format, "webp");
        assert!(thumbnail.width <= 240);
        assert!(thumbnail.height <= 240);
        assert!(!thumbnail.bytes.is_empty());
    }

    #[test]
    fn renders_thumbnail_from_memory() {
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(400, 600, Rgba([0, 255, 0, 255]));
        let mut png_bytes = Vec::new();
        DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .expect("png fixture should encode");

        let thumbnail = render_thumbnail_from_memory(&png_bytes, ThumbnailProfile::GridSm)
            .expect("thumbnail render should succeed");

        assert_eq!(thumbnail.format, "webp");
        assert!(thumbnail.width <= 240);
        assert!(thumbnail.height <= 240);
    }

    use image::DynamicImage;
    use std::io::Cursor;
}
