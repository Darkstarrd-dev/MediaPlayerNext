use anyhow::{anyhow, Context, Result};
use image::codecs::jpeg::JpegEncoder;
use image::codecs::webp::WebPEncoder;
use image::imageops::FilterType;
use image::{ExtendedColorType, GenericImageView, ImageEncoder, ImageReader};
use media_thumb::pipeline::{render_jpeg_thumbnail_from_path, JpegThumbnailConfig};
use rayon::prelude::*;
use serde::Serialize;
use std::ffi::OsStr;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Debug, Clone)]
struct CliArgs {
    input_dir: PathBuf,
    output_dir: PathBuf,
    width: u32,
    format: EncodeFormat,
    quality_input: Option<u32>,
}

#[derive(Debug, Clone, Copy)]
enum EncodeFormat {
    WebpLossless,
    Jpeg,
}

impl EncodeFormat {
    fn parse(value: &str) -> Option<Self> {
        match value {
            "webp-lossless" => Some(Self::WebpLossless),
            "jpeg" => Some(Self::Jpeg),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::WebpLossless => "webp-lossless",
            Self::Jpeg => "jpeg",
        }
    }

    fn extension(self) -> &'static str {
        match self {
            Self::WebpLossless => "webp",
            Self::Jpeg => "jpg",
        }
    }

    fn quality_mode(self) -> &'static str {
        match self {
            Self::WebpLossless => "ignored",
            Self::Jpeg => "effective",
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BatchResult {
    engine: &'static str,
    format: &'static str,
    quality_mode: &'static str,
    width: u32,
    quality_input: Option<u32>,
    total_images: usize,
    total_output_bytes: usize,
    elapsed_ms: f64,
}

fn main() {
    if let Err(error) = try_main() {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

fn try_main() -> Result<()> {
    let args = parse_args()?;
    let files = list_input_files(&args.input_dir)?;
    if files.is_empty() {
        return Err(anyhow!(
            "no .webp files found in {}",
            args.input_dir.display()
        ));
    }

    reset_output_dir(&args.output_dir)?;

    let start = Instant::now();
    let total_output_bytes = files
        .par_iter()
        .map(|input_path| process_one_image(input_path, &args.output_dir, &args))
        .try_reduce(|| 0usize, |left, right| Ok(left + right))?;
    let elapsed_ms = start.elapsed().as_secs_f64() * 1000.0;

    let result = BatchResult {
        engine: "rust-image-lossless",
        format: args.format.as_str(),
        quality_mode: args.format.quality_mode(),
        width: args.width,
        quality_input: args.quality_input,
        total_images: files.len(),
        total_output_bytes,
        elapsed_ms,
    };
    println!("{}", serde_json::to_string(&result)?);

    Ok(())
}

fn parse_args() -> Result<CliArgs> {
    let mut input_dir: Option<PathBuf> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut width: Option<u32> = None;
    let mut format = EncodeFormat::WebpLossless;
    let mut quality_input: Option<u32> = None;

    let mut args = std::env::args().skip(1);
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--input-dir" => {
                input_dir = args.next().map(PathBuf::from);
            }
            "--output-dir" => {
                output_dir = args.next().map(PathBuf::from);
            }
            "--width" => {
                width = args
                    .next()
                    .map(|value| value.parse::<u32>())
                    .transpose()
                    .context("--width parse failed")?;
            }
            "--format" => {
                let value = args
                    .next()
                    .ok_or_else(|| anyhow!("missing value for --format"))?;
                format = EncodeFormat::parse(&value)
                    .ok_or_else(|| anyhow!("unsupported --format value: {value}"))?;
            }
            "--quality" => {
                quality_input = args
                    .next()
                    .map(|value| value.parse::<u32>())
                    .transpose()
                    .context("--quality parse failed")?;
            }
            _ => {
                return Err(anyhow!("unsupported argument: {flag}"));
            }
        }
    }

    let input_dir = input_dir.ok_or_else(|| anyhow!("missing --input-dir"))?;
    let output_dir = output_dir.ok_or_else(|| anyhow!("missing --output-dir"))?;
    let width = width.ok_or_else(|| anyhow!("missing --width"))?;
    let width = width.max(1);

    Ok(CliArgs {
        input_dir,
        output_dir,
        width,
        format,
        quality_input,
    })
}

fn list_input_files(input_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files: Vec<PathBuf> = fs::read_dir(input_dir)
        .with_context(|| format!("read_dir failed: {}", input_dir.display()))?
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| {
            path.extension()
                .and_then(OsStr::to_str)
                .map(|ext| ext.eq_ignore_ascii_case("webp"))
                .unwrap_or(false)
        })
        .collect();

    files.sort();
    Ok(files)
}

fn reset_output_dir(output_dir: &Path) -> Result<()> {
    if output_dir.exists() {
        fs::remove_dir_all(output_dir)
            .with_context(|| format!("remove_dir_all failed: {}", output_dir.display()))?;
    }
    fs::create_dir_all(output_dir)
        .with_context(|| format!("create_dir_all failed: {}", output_dir.display()))?;
    Ok(())
}

fn process_one_image(input_path: &Path, output_dir: &Path, args: &CliArgs) -> Result<usize> {
    let file_name = input_path
        .file_stem()
        .ok_or_else(|| anyhow!("input file missing file name: {}", input_path.display()))?;
    let output_path = output_dir.join(format!(
        "{}.{}",
        file_name.to_string_lossy(),
        args.format.extension()
    ));

    let encoded = match args.format {
        EncodeFormat::Jpeg => {
            let quality = args
                .quality_input
                .unwrap_or(75)
                .clamp(1, 100)
                .try_into()
                .map_err(|_| anyhow!("invalid jpeg quality"))?;

            render_jpeg_thumbnail_from_path(
                input_path,
                JpegThumbnailConfig::new(args.width, args.width, quality),
            )
            .with_context(|| format!("jpeg pipeline failed: {}", input_path.display()))?
            .bytes
        }
        EncodeFormat::WebpLossless => {
            let image = ImageReader::open(input_path)
                .with_context(|| format!("ImageReader::open failed: {}", input_path.display()))?
                .decode()
                .with_context(|| format!("decode failed: {}", input_path.display()))?;

            let resized = image.resize(args.width, args.width, FilterType::Lanczos3);
            let (target_width, target_height) = resized.dimensions();
            encode_image(args, &resized, target_width, target_height)
                .with_context(|| format!("encode failed: {}", input_path.display()))?
        }
    };

    let output_len = encoded.len();
    fs::write(&output_path, encoded)
        .with_context(|| format!("write failed: {}", output_path.display()))?;

    Ok(output_len)
}

fn encode_image(
    args: &CliArgs,
    resized: &image::DynamicImage,
    target_width: u32,
    target_height: u32,
) -> Result<Vec<u8>> {
    let mut encoded = Cursor::new(Vec::new());
    match args.format {
        EncodeFormat::WebpLossless => {
            let rgba = resized.to_rgba8();
            WebPEncoder::new_lossless(&mut encoded).write_image(
                rgba.as_raw(),
                target_width,
                target_height,
                ExtendedColorType::Rgba8,
            )?;
        }
        EncodeFormat::Jpeg => {
            let quality = args
                .quality_input
                .unwrap_or(75)
                .clamp(1, 100)
                .try_into()
                .map_err(|_| anyhow!("invalid jpeg quality"))?;
            let rgb = resized.to_rgb8();
            JpegEncoder::new_with_quality(&mut encoded, quality).write_image(
                rgb.as_raw(),
                target_width,
                target_height,
                ExtendedColorType::Rgb8,
            )?;
        }
    }
    Ok(encoded.into_inner())
}
