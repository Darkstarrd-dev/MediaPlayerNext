use super::layout::NormalizedArchiveLayout;
use super::sevenz::ArchiveExtractor;
use crate::archive::build_zip_index;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizationResult {
    pub normalized_zip_path: PathBuf,
    pub extracted_file_count: usize,
}

pub fn normalize_archive_to_zip<E>(
    extractor: &E,
    archive_path: &Path,
    layout: &NormalizedArchiveLayout,
) -> Result<NormalizationResult>
where
    E: ArchiveExtractor,
{
    prepare_layout_for_retry(layout)?;
    extractor.extract_archive(archive_path, &layout.extracted_dir)?;
    let extracted_file_count =
        pack_directory_to_zip(&layout.extracted_dir, &layout.output_zip_path)?;
    verify_normalized_zip(&layout.output_zip_path)?;

    Ok(NormalizationResult {
        normalized_zip_path: layout.output_zip_path.clone(),
        extracted_file_count,
    })
}

pub fn prepare_layout_for_retry(layout: &NormalizedArchiveLayout) -> Result<()> {
    if layout.work_dir.exists() {
        fs::remove_dir_all(&layout.work_dir).with_context(|| {
            format!(
                "failed to clean normalize work dir: {}",
                layout.work_dir.display()
            )
        })?;
    }

    fs::create_dir_all(&layout.extracted_dir).with_context(|| {
        format!(
            "failed to create normalize extract dir: {}",
            layout.extracted_dir.display()
        )
    })?;
    Ok(())
}

pub fn pack_directory_to_zip(input_dir: &Path, output_zip_path: &Path) -> Result<usize> {
    let files = collect_relative_files(input_dir, input_dir)?;
    let parent = output_zip_path
        .parent()
        .expect("output zip path should have parent directory");
    fs::create_dir_all(parent)?;

    let output_file = fs::File::create(output_zip_path).with_context(|| {
        format!(
            "failed to create normalized zip: {}",
            output_zip_path.display()
        )
    })?;
    let mut writer = ZipWriter::new(output_file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);

    for relative_path in &files {
        let source_path = input_dir.join(relative_path);
        let entry_name = relative_path.to_string_lossy().replace('\\', "/");
        writer
            .start_file(&entry_name, options)
            .with_context(|| format!("failed to start zip entry: {}", relative_path.display()))?;

        let mut file = fs::File::open(&source_path)
            .with_context(|| format!("failed to open extracted file: {}", source_path.display()))?;
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)
            .with_context(|| format!("failed to read extracted file: {}", source_path.display()))?;
        writer
            .write_all(&bytes)
            .with_context(|| format!("failed to write zip entry: {}", relative_path.display()))?;
    }

    writer
        .finish()
        .context("failed to finalize normalized zip")?;
    Ok(files.len())
}

pub fn verify_normalized_zip(output_zip_path: &Path) -> Result<()> {
    build_zip_index(output_zip_path).map(|_| ())
}

fn collect_relative_files(root: &Path, current: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(current)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;

        if metadata.is_dir() {
            files.extend(collect_relative_files(root, &path)?);
            continue;
        }

        if metadata.is_file() {
            let relative_path = path
                .strip_prefix(root)
                .expect("normalized output should be inside root")
                .to_path_buf();
            files.push(relative_path);
        }
    }

    files.sort();
    Ok(files)
}

#[cfg(test)]
mod tests {
    use super::{normalize_archive_to_zip, prepare_layout_for_retry};
    use crate::archive::read_zip_entry_bytes;
    use crate::normalize::layout::normalized_archive_layout;
    use crate::normalize::sevenz::ArchiveExtractor;
    use anyhow::{anyhow, Result};
    use shared_model::SourceId;
    use std::fs;
    use std::path::Path;
    use std::sync::Mutex;
    use tempfile::tempdir;

    struct MockExtractor {
        files: Vec<(String, Vec<u8>)>,
        fail_first: Mutex<bool>,
    }

    impl ArchiveExtractor for MockExtractor {
        fn extract_archive(&self, _archive_path: &Path, output_dir: &Path) -> Result<()> {
            let mut fail_first = self.fail_first.lock().expect("lock");
            if *fail_first {
                *fail_first = false;
                return Err(anyhow!("mock extractor failure"));
            }

            for (relative_path, bytes) in &self.files {
                let target_path = output_dir.join(relative_path);
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::write(target_path, bytes)?;
            }
            Ok(())
        }
    }

    #[test]
    fn normalizes_extracted_files_to_zip() {
        let temp = tempdir().expect("tempdir should exist");
        let archive_path = temp.path().join("sample.7z");
        fs::write(&archive_path, b"fake-archive").expect("archive placeholder should exist");
        let layout = normalized_archive_layout(
            temp.path(),
            &SourceId("source_norm_primary".to_string()),
            "fp-1",
        );
        let extractor = MockExtractor {
            files: vec![
                ("nested/001-cover.png".to_string(), b"cover".to_vec()),
                ("nested/002-page.png".to_string(), b"page".to_vec()),
            ],
            fail_first: Mutex::new(false),
        };

        let result = normalize_archive_to_zip(&extractor, &archive_path, &layout)
            .expect("normalization should succeed");

        assert_eq!(result.extracted_file_count, 2);
        assert!(result.normalized_zip_path.exists());
        assert_eq!(
            read_zip_entry_bytes(&result.normalized_zip_path, "nested/001-cover.png")
                .expect("zip entry should be readable"),
            b"cover"
        );
    }

    #[test]
    fn retry_preparation_cleans_previous_output() {
        let temp = tempdir().expect("tempdir should exist");
        let layout = normalized_archive_layout(
            temp.path(),
            &SourceId("source_norm_retry".to_string()),
            "fp-2",
        );

        fs::create_dir_all(&layout.extracted_dir).expect("extract dir should exist");
        fs::write(layout.extracted_dir.join("stale.txt"), b"stale")
            .expect("stale file should exist");

        prepare_layout_for_retry(&layout).expect("retry preparation should succeed");
        assert!(!layout.extracted_dir.join("stale.txt").exists());
    }

    #[test]
    fn supports_retry_after_extractor_failure() {
        let temp = tempdir().expect("tempdir should exist");
        let archive_path = temp.path().join("sample.rar");
        fs::write(&archive_path, b"fake-archive").expect("archive placeholder should exist");
        let layout = normalized_archive_layout(
            temp.path(),
            &SourceId("source_norm_fail_then_retry".to_string()),
            "fp-3",
        );
        let extractor = MockExtractor {
            files: vec![("001-cover.png".to_string(), b"cover".to_vec())],
            fail_first: Mutex::new(true),
        };

        let first = normalize_archive_to_zip(&extractor, &archive_path, &layout);
        assert!(first.is_err());

        let second = normalize_archive_to_zip(&extractor, &archive_path, &layout)
            .expect("retry normalization should succeed");
        assert_eq!(second.extracted_file_count, 1);
    }
}
