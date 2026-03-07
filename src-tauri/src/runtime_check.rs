use anyhow::{anyhow, Context, Result};
use rusqlite::Connection;
use serde::Serialize;
use std::path::Path;
use std::process::Command;

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

    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("spawn runtime binary: {program}"))?;

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
