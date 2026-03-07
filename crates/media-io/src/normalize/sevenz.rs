use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

pub trait ArchiveExtractor {
    fn extract_archive(&self, archive_path: &Path, output_dir: &Path) -> Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SevenZipInvocation {
    pub executable_path: PathBuf,
    pub archive_path: PathBuf,
    pub output_dir: PathBuf,
    pub arguments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SevenZipExtractor {
    pub executable_path: PathBuf,
}

impl SevenZipExtractor {
    pub fn new(executable_path: impl AsRef<Path>) -> Self {
        Self {
            executable_path: executable_path.as_ref().to_path_buf(),
        }
    }

    pub fn invocation(&self, archive_path: &Path, output_dir: &Path) -> SevenZipInvocation {
        SevenZipInvocation {
            executable_path: self.executable_path.clone(),
            archive_path: archive_path.to_path_buf(),
            output_dir: output_dir.to_path_buf(),
            arguments: vec![
                "x".to_string(),
                "-y".to_string(),
                "-bso1".to_string(),
                "-bse2".to_string(),
                format!("-o{}", output_dir.display()),
                archive_path.display().to_string(),
            ],
        }
    }
}

impl ArchiveExtractor for SevenZipExtractor {
    fn extract_archive(&self, archive_path: &Path, output_dir: &Path) -> Result<()> {
        let invocation = self.invocation(archive_path, output_dir);
        let output = Command::new(&invocation.executable_path)
            .args(&invocation.arguments)
            .output()
            .with_context(|| {
                format!(
                    "failed to launch sevenz executable: {}",
                    invocation.executable_path.display()
                )
            })?;

        if output.status.success() {
            return Ok(());
        }

        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let message = if !stderr.is_empty() { stderr } else { stdout };

        Err(anyhow!(
            "sevenz extract failed (code {:?}): {}",
            output.status.code(),
            message
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::SevenZipExtractor;
    use std::path::Path;

    #[test]
    fn builds_expected_extract_arguments() {
        let extractor = SevenZipExtractor::new("C:/Program Files/7-Zip/7z.exe");
        let invocation = extractor.invocation(
            Path::new("Z:/fixtures/archive/sample.7z"),
            Path::new("Z:/cache/normalized/source/extract"),
        );

        assert_eq!(invocation.arguments[0], "x");
        assert_eq!(invocation.arguments[1], "-y");
        assert!(invocation.arguments[4].starts_with("-oZ:/cache/normalized/source/extract"));
        assert_eq!(invocation.arguments[5], "Z:/fixtures/archive/sample.7z");
    }
}
