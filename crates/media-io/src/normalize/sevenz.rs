use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use shared_model::{build_command_line, emit_external_process_log, ExternalProcessLog, LogContext};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

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
        let command_line = build_command_line(
            &invocation.executable_path.display().to_string(),
            &invocation.arguments,
        );
        let started_at = Instant::now();
        let output = match Command::new(&invocation.executable_path)
            .args(&invocation.arguments)
            .output()
        {
            Ok(output) => output,
            Err(error) => {
                emit_external_process_log(&ExternalProcessLog {
                    event: "external-process".to_string(),
                    phase: "spawn_failed".to_string(),
                    tool: "sevenz".to_string(),
                    executable: invocation.executable_path.display().to_string(),
                    arguments: invocation.arguments.clone(),
                    command_line,
                    exit_code: None,
                    duration_ms: Some(started_at.elapsed().as_millis() as u64),
                    ok: false,
                    context: LogContext::default(),
                    stderr_excerpt: Some(error.to_string()),
                });
                return Err(error).with_context(|| {
                    format!(
                        "failed to launch sevenz executable: {}",
                        invocation.executable_path.display()
                    )
                });
            }
        };

        emit_external_process_log(&ExternalProcessLog {
            event: "external-process".to_string(),
            phase: "completed".to_string(),
            tool: "sevenz".to_string(),
            executable: invocation.executable_path.display().to_string(),
            arguments: invocation.arguments.clone(),
            command_line: build_command_line(
                &invocation.executable_path.display().to_string(),
                &invocation.arguments,
            ),
            exit_code: output.status.code(),
            duration_ms: Some(started_at.elapsed().as_millis() as u64),
            ok: output.status.success(),
            context: LogContext::default(),
            stderr_excerpt: stderr_excerpt(&output.stderr),
        });

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

fn stderr_excerpt(raw: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(raw).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text.chars().take(240).collect())
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
