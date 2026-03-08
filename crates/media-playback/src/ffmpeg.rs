use anyhow::{anyhow, Context, Result};
use shared_model::{
    build_command_line, emit_external_process_log, ExternalProcessLog, FfmpegProgressEvent,
    LogContext,
};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameExtractRequest {
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    pub timestamp_seconds: u32,
}

pub fn build_extract_frame_args(request: &FrameExtractRequest) -> Vec<String> {
    vec![
        "-y".to_string(),
        "-ss".to_string(),
        request.timestamp_seconds.to_string(),
        "-i".to_string(),
        request.input_path.display().to_string(),
        "-frames:v".to_string(),
        "1".to_string(),
        "-progress".to_string(),
        "pipe:1".to_string(),
        request.output_path.display().to_string(),
    ]
}

pub fn extract_video_frame(ffmpeg_path: &Path, request: &FrameExtractRequest) -> Result<()> {
    let arguments = build_extract_frame_args(request);
    let command_line = build_command_line(&ffmpeg_path.display().to_string(), &arguments);
    let started_at = Instant::now();
    let output = match Command::new(ffmpeg_path).args(&arguments).output() {
        Ok(output) => output,
        Err(error) => {
            emit_external_process_log(&ExternalProcessLog {
                event: "external-process".to_string(),
                phase: "spawn_failed".to_string(),
                tool: "ffmpeg".to_string(),
                executable: ffmpeg_path.display().to_string(),
                arguments,
                command_line,
                exit_code: None,
                duration_ms: Some(started_at.elapsed().as_millis() as u64),
                ok: false,
                context: LogContext::default(),
                stderr_excerpt: Some(error.to_string()),
            });
            return Err(error).with_context(|| format!("spawn ffmpeg: {}", ffmpeg_path.display()));
        }
    };

    emit_external_process_log(&ExternalProcessLog {
        event: "external-process".to_string(),
        phase: "completed".to_string(),
        tool: "ffmpeg".to_string(),
        executable: ffmpeg_path.display().to_string(),
        arguments: build_extract_frame_args(request),
        command_line: build_command_line(
            &ffmpeg_path.display().to_string(),
            &build_extract_frame_args(request),
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

    Err(anyhow!(
        "ffmpeg extract frame failed for {}",
        request.input_path.display()
    ))
}

fn stderr_excerpt(raw: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(raw).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text.chars().take(240).collect())
    }
}

pub fn parse_ffmpeg_progress(raw: &str) -> FfmpegProgressEvent {
    let mut event = FfmpegProgressEvent {
        frame: None,
        fps: None,
        total_size: None,
        out_time_ms: None,
        speed: None,
        progress: "unknown".to_string(),
    };

    for line in raw.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        match key.trim() {
            "frame" => event.frame = value.trim().parse::<u64>().ok(),
            "fps" => event.fps = value.trim().parse::<f64>().ok(),
            "total_size" => event.total_size = value.trim().parse::<u64>().ok(),
            "out_time_ms" => event.out_time_ms = value.trim().parse::<i64>().ok(),
            "speed" => {
                event.speed = value.trim().trim_end_matches('x').parse::<f64>().ok();
            }
            "progress" => event.progress = value.trim().to_string(),
            _ => {}
        }
    }

    event
}

#[cfg(test)]
mod tests {
    use super::{build_extract_frame_args, parse_ffmpeg_progress, FrameExtractRequest};
    use std::path::PathBuf;

    #[test]
    fn builds_frame_extract_args() {
        let request = FrameExtractRequest {
            input_path: PathBuf::from("Z:/media/video.mp4"),
            output_path: PathBuf::from("Z:/cache/frame.webp"),
            timestamp_seconds: 3,
        };
        let args = build_extract_frame_args(&request);

        assert_eq!(args[0], "-y");
        assert_eq!(args[2], "3");
        assert_eq!(args[4], "Z:/media/video.mp4");
        assert_eq!(args[9], "Z:/cache/frame.webp");
    }

    #[test]
    fn parses_ffmpeg_progress_kv_output() {
        let event = parse_ffmpeg_progress(
            "frame=12\nfps=25.0\ntotal_size=4096\nout_time_ms=5000000\nspeed=1.5x\nprogress=continue\n",
        );

        assert_eq!(event.frame, Some(12));
        assert_eq!(event.fps, Some(25.0));
        assert_eq!(event.total_size, Some(4096));
        assert_eq!(event.out_time_ms, Some(5000000));
        assert_eq!(event.speed, Some(1.5));
        assert_eq!(event.progress, "continue");
    }
}
