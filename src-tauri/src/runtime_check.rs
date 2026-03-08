use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use serde::Serialize;
use shared_model::{build_command_line, emit_external_process_log, ExternalProcessLog, LogContext};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Serialize)]
pub struct RuntimeSmokeCheckResult {
    pub sqlite_version: String,
    pub ffmpeg_path: String,
    pub ffmpeg_first_line: String,
    pub ffprobe_path: String,
    pub ffprobe_first_line: String,
    pub mpv_path: String,
    pub mpv_first_line: String,
}

pub fn run_runtime_smoke_check(
    ffmpeg_path: &str,
    ffprobe_path: &str,
    mpv_path: &str,
) -> Result<RuntimeSmokeCheckResult> {
    let sqlite_version = read_sqlite_version()?;
    let ffmpeg_first_line = read_process_first_line(ffmpeg_path, &["-version"])?;
    let ffprobe_first_line = read_process_first_line(ffprobe_path, &["-version"])?;
    let mpv_first_line = read_process_first_line(mpv_path, &["--version"])?;

    Ok(RuntimeSmokeCheckResult {
        sqlite_version,
        ffmpeg_path: ffmpeg_path.to_string(),
        ffmpeg_first_line,
        ffprobe_path: ffprobe_path.to_string(),
        ffprobe_first_line,
        mpv_path: mpv_path.to_string(),
        mpv_first_line,
    })
}

fn read_sqlite_version() -> Result<String> {
    let connection = Connection::open_in_memory().context("open sqlite in-memory database")?;
    let version: String = connection
        .query_row("select sqlite_version()", [], |row| row.get(0))
        .context("query sqlite version")?;
    Ok(version)
}

fn read_process_first_line(program: &str, args: &[&str]) -> Result<String> {
    let program_path = Path::new(program);
    if !program_path.is_file() {
        return Err(anyhow!("runtime binary not found: {program}"));
    }

    let arguments = args
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    let command_line = build_command_line(program, &arguments);
    let started_at = Instant::now();

    let output = match Command::new(program).args(args).output() {
        Ok(output) => output,
        Err(error) => {
            emit_external_process_log(&ExternalProcessLog {
                event: "external-process".to_string(),
                phase: "spawn_failed".to_string(),
                tool: "runtime-check".to_string(),
                executable: program.to_string(),
                arguments,
                command_line,
                exit_code: None,
                duration_ms: Some(started_at.elapsed().as_millis() as u64),
                ok: false,
                context: LogContext::default(),
                stderr_excerpt: Some(error.to_string()),
            });
            return Err(error).with_context(|| format!("spawn runtime binary: {program}"));
        }
    };

    emit_external_process_log(&ExternalProcessLog {
        event: "external-process".to_string(),
        phase: "completed".to_string(),
        tool: "runtime-check".to_string(),
        executable: program.to_string(),
        arguments: args.iter().map(|value| (*value).to_string()).collect(),
        command_line: build_command_line(
            program,
            &args
                .iter()
                .map(|value| (*value).to_string())
                .collect::<Vec<_>>(),
        ),
        exit_code: output.status.code(),
        duration_ms: Some(started_at.elapsed().as_millis() as u64),
        ok: output.status.success(),
        context: LogContext::default(),
        stderr_excerpt: stderr_excerpt(&output.stderr),
    });

    if !output.status.success() {
        return Err(anyhow!(
            "runtime binary returned non-zero status: {program}"
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let first_line = stdout.lines().next().unwrap_or_default().trim().to_string();
    if first_line.is_empty() {
        return Err(anyhow!("runtime binary produced empty output: {program}"));
    }

    Ok(first_line)
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
    use super::read_process_first_line;

    #[test]
    fn reports_missing_runtime_binary() {
        let error = read_process_first_line("Z:/missing-runtime-binary.exe", &["--version"])
            .expect_err("missing runtime binary should fail");

        assert!(error.to_string().contains("runtime binary not found"));
    }

    #[test]
    fn reports_non_zero_runtime_status() {
        let cmd_path = std::env::var("ComSpec").expect("ComSpec should exist on Windows");

        let error = read_process_first_line(&cmd_path, &["/C", "exit", "9"])
            .expect_err("non-zero runtime should fail");

        assert!(error
            .to_string()
            .contains("runtime binary returned non-zero status"));
    }

    #[test]
    fn reports_empty_runtime_output() {
        let cmd_path = std::env::var("ComSpec").expect("ComSpec should exist on Windows");

        let error = read_process_first_line(&cmd_path, &["/C", "exit", "0"])
            .expect_err("empty runtime output should fail");

        assert!(error
            .to_string()
            .contains("runtime binary produced empty output"));
    }
}
