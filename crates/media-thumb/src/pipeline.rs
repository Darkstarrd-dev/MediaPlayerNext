use crate::profiles::ThumbnailProfile;
use anyhow::{anyhow, Result};
use fast_image_resize as fr;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, ExtendedColorType, ImageEncoder, ImageReader};
use jpeg_decoder as jpeg;
use std::fs::File;
use std::io::{BufReader, Cursor};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThumbnailRender {
    pub width: u32,
    pub height: u32,
    pub format: &'static str,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JpegThumbnailConfig {
    pub max_width: u32,
    pub max_height: u32,
    pub quality: u8,
}

impl JpegThumbnailConfig {
    pub fn new(max_width: u32, max_height: u32, quality: u8) -> Self {
        Self {
            max_width: max_width.max(1),
            max_height: max_height.max(1),
            quality: quality.clamp(1, 100),
        }
    }
}

pub fn render_thumbnail_from_path(
    path: &std::path::Path,
    profile: ThumbnailProfile,
) -> Result<ThumbnailRender> {
    render_jpeg_thumbnail_from_path(path, profile.to_jpeg_config())
}

pub fn render_thumbnail_from_memory(
    bytes: &[u8],
    profile: ThumbnailProfile,
) -> Result<ThumbnailRender> {
    render_jpeg_thumbnail_from_memory(bytes, profile.to_jpeg_config())
}

pub fn render_jpeg_thumbnail_from_path(
    path: &Path,
    config: JpegThumbnailConfig,
) -> Result<ThumbnailRender> {
    let image = if is_jpeg_extension(path) {
        decode_scaled_jpeg_from_path(path, config)?
    } else {
        ImageReader::open(path)?.decode()?
    };
    render_jpeg_thumbnail(image, config)
}

pub fn render_jpeg_thumbnail_from_memory(
    bytes: &[u8],
    config: JpegThumbnailConfig,
) -> Result<ThumbnailRender> {
    let image = if is_jpeg_signature(bytes) {
        decode_scaled_jpeg_from_memory(bytes, config)?
    } else {
        image::load_from_memory(bytes)?
    };
    render_jpeg_thumbnail(image, config)
}

fn render_jpeg_thumbnail(
    image: DynamicImage,
    config: JpegThumbnailConfig,
) -> Result<ThumbnailRender> {
    let resized = resize_rgb_with_simd(image, config.max_width, config.max_height)?;
    let (width, height) = resized.dimensions();
    let mut bytes = Cursor::new(Vec::new());
    JpegEncoder::new_with_quality(&mut bytes, config.quality).write_image(
        resized.as_raw(),
        width,
        height,
        ExtendedColorType::Rgb8,
    )?;

    Ok(ThumbnailRender {
        width,
        height,
        format: "jpeg",
        bytes: bytes.into_inner(),
    })
}

fn decode_scaled_jpeg_from_path(path: &Path, config: JpegThumbnailConfig) -> Result<DynamicImage> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    decode_scaled_jpeg(reader, config)
}

fn decode_scaled_jpeg_from_memory(
    bytes: &[u8],
    config: JpegThumbnailConfig,
) -> Result<DynamicImage> {
    let reader = Cursor::new(bytes);
    decode_scaled_jpeg(reader, config)
}

fn decode_scaled_jpeg<R>(reader: R, config: JpegThumbnailConfig) -> Result<DynamicImage>
where
    R: std::io::Read,
{
    let mut decoder = jpeg::Decoder::new(reader);
    let requested = scaled_decode_request(config.max_width, config.max_height);
    decoder.scale(requested.0, requested.1)?;
    let pixels = decoder.decode()?;
    let info = decoder
        .info()
        .ok_or_else(|| anyhow!("jpeg decoder missing image info"))?;

    let width = u32::from(info.width);
    let height = u32::from(info.height);
    match info.pixel_format {
        jpeg::PixelFormat::L8 => image::GrayImage::from_raw(width, height, pixels)
            .map(DynamicImage::ImageLuma8)
            .ok_or_else(|| anyhow!("failed to build L8 image from jpeg decoder")),
        jpeg::PixelFormat::L16 => {
            let mut gray8 = Vec::with_capacity(pixels.len() / 2);
            for chunk in pixels.chunks_exact(2) {
                gray8.push(chunk[0]);
            }
            image::GrayImage::from_raw(width, height, gray8)
                .map(DynamicImage::ImageLuma8)
                .ok_or_else(|| anyhow!("failed to build L16-converted image from jpeg decoder"))
        }
        jpeg::PixelFormat::RGB24 => image::RgbImage::from_raw(width, height, pixels)
            .map(DynamicImage::ImageRgb8)
            .ok_or_else(|| anyhow!("failed to build RGB image from jpeg decoder")),
        jpeg::PixelFormat::CMYK32 => {
            let mut rgb = Vec::with_capacity((width * height * 3) as usize);
            for chunk in pixels.chunks_exact(4) {
                let c = u16::from(chunk[0]);
                let m = u16::from(chunk[1]);
                let y = u16::from(chunk[2]);
                let k = u16::from(chunk[3]);
                let r = 255u16.saturating_sub((c + k).min(255)) as u8;
                let g = 255u16.saturating_sub((m + k).min(255)) as u8;
                let b = 255u16.saturating_sub((y + k).min(255)) as u8;
                rgb.push(r);
                rgb.push(g);
                rgb.push(b);
            }
            image::RgbImage::from_raw(width, height, rgb)
                .map(DynamicImage::ImageRgb8)
                .ok_or_else(|| anyhow!("failed to build CMYK-converted image from jpeg decoder"))
        }
    }
}

fn scaled_decode_request(max_width: u32, max_height: u32) -> (u16, u16) {
    let request_width = max_width.saturating_mul(2).min(u16::MAX as u32).max(1) as u16;
    let request_height = max_height.saturating_mul(2).min(u16::MAX as u32).max(1) as u16;
    (request_width, request_height)
}

fn resize_rgb_with_simd(
    image: DynamicImage,
    max_width: u32,
    max_height: u32,
) -> Result<image::RgbImage> {
    let source = image.to_rgb8();
    let (src_width, src_height) = source.dimensions();
    let (dst_width, dst_height) = fit_with_aspect(src_width, src_height, max_width, max_height);

    if dst_width == src_width && dst_height == src_height {
        return Ok(source);
    }

    let src_image = fr::images::Image::from_vec_u8(
        src_width,
        src_height,
        source.into_raw(),
        fr::PixelType::U8x3,
    )?;
    let mut dst_image = fr::images::Image::new(dst_width, dst_height, fr::PixelType::U8x3);

    let mut resizer = fr::Resizer::new();
    resizer.resize(
        &src_image,
        &mut dst_image,
        &fr::ResizeOptions::new().resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3)),
    )?;

    image::RgbImage::from_raw(dst_width, dst_height, dst_image.into_vec())
        .ok_or_else(|| anyhow!("failed to build RGB image after SIMD resize"))
}

fn fit_with_aspect(src_width: u32, src_height: u32, max_width: u32, max_height: u32) -> (u32, u32) {
    let src_width_f = src_width.max(1) as f64;
    let src_height_f = src_height.max(1) as f64;
    let width_scale = max_width.max(1) as f64 / src_width_f;
    let height_scale = max_height.max(1) as f64 / src_height_f;
    let scale = width_scale.min(height_scale);

    let dst_width = (src_width_f * scale).round().max(1.0) as u32;
    let dst_height = (src_height_f * scale).round().max(1.0) as u32;
    (dst_width, dst_height)
}

fn is_jpeg_extension(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| {
            let lower = value.to_ascii_lowercase();
            lower == "jpg" || lower == "jpeg"
        })
        .unwrap_or(false)
}

fn is_jpeg_signature(bytes: &[u8]) -> bool {
    bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF
}

#[cfg(test)]
mod tests {
    use super::{render_thumbnail_from_memory, render_thumbnail_from_path};
    use crate::profiles::ThumbnailProfile;
    use image::{ImageBuffer, Rgb, Rgba};
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

        assert_eq!(thumbnail.format, "jpeg");
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

        assert_eq!(thumbnail.format, "jpeg");
        assert!(thumbnail.width <= 240);
        assert!(thumbnail.height <= 240);
    }

    #[test]
    fn uses_scaled_decode_for_jpeg_input() {
        let temp = tempdir().expect("tempdir");
        let source_path = temp.path().join("source.jpg");
        let image = ImageBuffer::<Rgb<u8>, Vec<u8>>::from_pixel(3200, 2400, Rgb([120, 80, 20]));
        image
            .save(&source_path)
            .expect("jpeg fixture should be written");

        let thumbnail = render_thumbnail_from_path(&source_path, ThumbnailProfile::GridSm)
            .expect("thumbnail render should succeed");

        assert_eq!(thumbnail.format, "jpeg");
        assert!(thumbnail.width <= 240);
        assert!(thumbnail.height <= 240);
    }

    use image::DynamicImage;
    use std::io::Cursor;
}
