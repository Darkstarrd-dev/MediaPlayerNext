use crate::cache::{
    cache_path_for_key_with_extension, thumbnail_key_for_source, write_thumbnail_atomically,
};
use crate::pipeline::{render_thumbnail_from_memory, render_thumbnail_from_path};
use crate::profiles::ThumbnailProfile;
use crate::source::ThumbnailSource;
use anyhow::Result;
use media_io::archive::read_zip_entry_bytes;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedThumbnail {
    pub thumbnail_key: String,
    pub disk_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub format: String,
    pub byte_size: usize,
    pub cache_hit: bool,
}

#[derive(Debug, Clone)]
pub struct ThumbnailService {
    cache_root: PathBuf,
    pipeline_version: String,
}

impl ThumbnailService {
    pub fn new(cache_root: impl AsRef<Path>, pipeline_version: impl Into<String>) -> Self {
        Self {
            cache_root: cache_root.as_ref().to_path_buf(),
            pipeline_version: pipeline_version.into(),
        }
    }

    pub fn ensure_thumbnail(
        &self,
        source: &ThumbnailSource,
        profile: ThumbnailProfile,
    ) -> Result<GeneratedThumbnail> {
        let thumbnail_key = thumbnail_key_for_source(source, profile, &self.pipeline_version);
        let layout = cache_path_for_key_with_extension(&self.cache_root, &thumbnail_key, "jpg");

        if layout.absolute_path.exists() {
            let metadata = std::fs::metadata(&layout.absolute_path)?;
            let dimensions = image::image_dimensions(&layout.absolute_path)?;
            return Ok(GeneratedThumbnail {
                thumbnail_key,
                disk_path: layout.absolute_path,
                width: dimensions.0,
                height: dimensions.1,
                format: "jpeg".to_string(),
                byte_size: metadata.len() as usize,
                cache_hit: true,
            });
        }

        let render = match source {
            ThumbnailSource::FilePath { file_path, .. } => {
                render_thumbnail_from_path(Path::new(file_path), profile)?
            }
            ThumbnailSource::ArchiveEntry {
                archive_path,
                entry_path,
                ..
            } => {
                let entry_bytes = read_zip_entry_bytes(Path::new(archive_path), entry_path)?;
                render_thumbnail_from_memory(&entry_bytes, profile)?
            }
        };

        let extension = extension_for_format(render.format);
        let layout = cache_path_for_key_with_extension(&self.cache_root, &thumbnail_key, extension);
        write_thumbnail_atomically(&layout.absolute_path, &render.bytes)?;

        Ok(GeneratedThumbnail {
            thumbnail_key,
            disk_path: layout.absolute_path,
            width: render.width,
            height: render.height,
            format: render.format.to_string(),
            byte_size: render.bytes.len(),
            cache_hit: false,
        })
    }
}

fn extension_for_format(format: &str) -> &'static str {
    match format {
        "jpeg" => "jpg",
        "webp" => "webp",
        _ => "bin",
    }
}

#[cfg(test)]
mod tests {
    use super::ThumbnailService;
    use crate::profiles::ThumbnailProfile;
    use crate::source::ThumbnailSource;
    use image::{DynamicImage, ImageBuffer, Rgba};
    use std::fs::File;
    use std::io::{Cursor, Write};
    use tempfile::tempdir;
    use zip::write::SimpleFileOptions;
    use zip::{CompressionMethod, ZipWriter};

    #[test]
    fn ensures_file_thumbnail_and_reuses_cache() {
        let temp = tempdir().expect("tempdir");
        let source_path = temp.path().join("cover.png");
        ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(640, 480, Rgba([10, 20, 30, 255]))
            .save(&source_path)
            .expect("source image should be written");
        let service = ThumbnailService::new(temp.path().join("cache"), "v1");
        let source = ThumbnailSource::FilePath {
            source_identity: "source-file".to_string(),
            source_revision: "fp-1".to_string(),
            file_path: source_path.display().to_string(),
        };

        let first = service
            .ensure_thumbnail(&source, ThumbnailProfile::GridSm)
            .expect("first thumbnail ensure should succeed");
        let second = service
            .ensure_thumbnail(&source, ThumbnailProfile::GridSm)
            .expect("second thumbnail ensure should succeed");

        assert!(!first.cache_hit);
        assert!(second.cache_hit);
        assert_eq!(first.thumbnail_key, second.thumbnail_key);
        assert!(first.disk_path.exists());
    }

    #[test]
    fn ensures_archive_entry_thumbnail() {
        let temp = tempdir().expect("tempdir");
        let archive_path = temp.path().join("chapter.cbz");
        let image = ImageBuffer::<Rgba<u8>, Vec<u8>>::from_pixel(800, 1200, Rgba([0, 0, 255, 255]));
        let mut png_bytes = Vec::new();
        DynamicImage::ImageRgba8(image)
            .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .expect("png fixture should encode");
        write_zip(&archive_path, "001-cover.png", &png_bytes);

        let service = ThumbnailService::new(temp.path().join("cache"), "v1");
        let source = ThumbnailSource::ArchiveEntry {
            source_identity: "archive-entry-1".to_string(),
            source_revision: "archive::entry::crc".to_string(),
            archive_path: archive_path.display().to_string(),
            entry_path: "001-cover.png".to_string(),
        };

        let thumbnail = service
            .ensure_thumbnail(&source, ThumbnailProfile::GridSm)
            .expect("archive thumbnail ensure should succeed");

        assert!(!thumbnail.cache_hit);
        assert!(thumbnail.disk_path.exists());
        assert!(thumbnail.width <= 240);
        assert!(thumbnail.height <= 240);
    }

    fn write_zip(path: &std::path::Path, entry_name: &str, contents: &[u8]) {
        let file = File::create(path).expect("zip file should be created");
        let mut writer = ZipWriter::new(file);
        let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);

        writer
            .start_file(entry_name, options)
            .expect("zip entry should start");
        writer
            .write_all(contents)
            .expect("zip entry should be written");
        writer.finish().expect("zip writer should finish");
    }
}
