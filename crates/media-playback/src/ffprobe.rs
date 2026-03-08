use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use shared_model::{
    build_command_line, emit_external_process_log, ExternalProcessLog, LogContext,
    MediaProbeSummary,
};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

pub fn build_ffprobe_args(input_path: &Path) -> Vec<String> {
    vec![
        "-v".to_string(),
        "quiet".to_string(),
        "-print_format".to_string(),
        "json".to_string(),
        "-show_streams".to_string(),
        "-show_format".to_string(),
        input_path.display().to_string(),
    ]
}

pub fn probe_media_file(ffprobe_path: &Path, input_path: &Path) -> Result<MediaProbeSummary> {
    probe_media_file_with_context(ffprobe_path, input_path, LogContext::default())
}

pub fn probe_media_file_with_context(
    ffprobe_path: &Path,
    input_path: &Path,
    context: LogContext,
) -> Result<MediaProbeSummary> {
    let arguments = build_ffprobe_args(input_path);
    let command_line = build_command_line(&ffprobe_path.display().to_string(), &arguments);
    let started_at = Instant::now();
    let output = match Command::new(ffprobe_path).args(&arguments).output() {
        Ok(output) => output,
        Err(error) => {
            emit_external_process_log(&ExternalProcessLog {
                event: "external-process".to_string(),
                phase: "spawn_failed".to_string(),
                tool: "ffprobe".to_string(),
                executable: ffprobe_path.display().to_string(),
                arguments,
                command_line,
                exit_code: None,
                duration_ms: Some(started_at.elapsed().as_millis() as u64),
                ok: false,
                context: context.clone(),
                stderr_excerpt: Some(error.to_string()),
            });
            return Err(error)
                .with_context(|| format!("spawn ffprobe: {}", ffprobe_path.display()));
        }
    };

    emit_external_process_log(&ExternalProcessLog {
        event: "external-process".to_string(),
        phase: "completed".to_string(),
        tool: "ffprobe".to_string(),
        executable: ffprobe_path.display().to_string(),
        arguments: build_ffprobe_args(input_path),
        command_line: build_command_line(
            &ffprobe_path.display().to_string(),
            &build_ffprobe_args(input_path),
        ),
        exit_code: output.status.code(),
        duration_ms: Some(started_at.elapsed().as_millis() as u64),
        ok: output.status.success(),
        context,
        stderr_excerpt: stderr_excerpt(&output.stderr),
    });

    if !output.status.success() {
        return Err(anyhow!(
            "ffprobe returned non-zero status for {}",
            input_path.display()
        ));
    }

    parse_ffprobe_output(&String::from_utf8_lossy(&output.stdout))
}

fn stderr_excerpt(raw: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(raw).trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text.chars().take(240).collect())
    }
}

pub fn parse_ffprobe_output(raw: &str) -> Result<MediaProbeSummary> {
    let parsed: FfprobeOutput = serde_json::from_str(raw).context("parse ffprobe json")?;
    let video_stream = parsed
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("video"));
    let audio_stream = parsed
        .streams
        .iter()
        .find(|stream| stream.codec_type.as_deref() == Some("audio"));
    let mime = infer_mime(
        parsed
            .format
            .as_ref()
            .and_then(|format| format.format_name.as_deref()),
        video_stream.is_some(),
        audio_stream.is_some(),
    );

    Ok(MediaProbeSummary {
        mime,
        container_format: parsed
            .format
            .as_ref()
            .and_then(|format| format.format_name.clone()),
        duration_ms: parsed
            .format
            .as_ref()
            .and_then(|format| format.duration.as_deref())
            .and_then(parse_duration_ms),
        width: video_stream.and_then(|stream| stream.width),
        height: video_stream.and_then(|stream| stream.height),
        video_codec: video_stream.and_then(|stream| stream.codec_name.clone()),
        audio_codec: audio_stream.and_then(|stream| stream.codec_name.clone()),
    })
}

fn parse_duration_ms(value: &str) -> Option<i64> {
    let seconds = value.parse::<f64>().ok()?;
    Some((seconds * 1000.0).round() as i64)
}

fn infer_mime(format_name: Option<&str>, has_video: bool, has_audio: bool) -> String {
    match format_name.unwrap_or_default() {
        name if name.contains("mp4") => "video/mp4",
        name if name.contains("matroska") => "video/x-matroska",
        name if name.contains("webm") => "video/webm",
        name if name.contains("mp3") => "audio/mpeg",
        name if name.contains("flac") => "audio/flac",
        name if name.contains("wav") => "audio/wav",
        _ if has_video => "video/*",
        _ if has_audio => "audio/*",
        _ => "application/octet-stream",
    }
    .to_string()
}

#[derive(Debug, Deserialize)]
struct FfprobeOutput {
    #[serde(default)]
    streams: Vec<FfprobeStream>,
    format: Option<FfprobeFormat>,
}

#[derive(Debug, Deserialize)]
struct FfprobeStream {
    codec_name: Option<String>,
    codec_type: Option<String>,
    width: Option<u32>,
    height: Option<u32>,
}

#[derive(Debug, Deserialize)]
struct FfprobeFormat {
    format_name: Option<String>,
    duration: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{build_ffprobe_args, parse_ffprobe_output};
    use std::path::Path;

    #[test]
    fn builds_expected_ffprobe_args() {
        let args = build_ffprobe_args(Path::new("Z:/media/video.mp4"));
        assert_eq!(args[0], "-v");
        assert_eq!(args[1], "quiet");
        assert_eq!(args[6], "Z:/media/video.mp4");
    }

    #[test]
    fn parses_video_probe_json() {
        let summary = parse_ffprobe_output(
            r#"{
              "streams": [
                {"codec_name": "h264", "codec_type": "video", "width": 1920, "height": 1080},
                {"codec_name": "aac", "codec_type": "audio"}
              ],
              "format": {"format_name": "mov,mp4,m4a,3gp,3g2,mj2", "duration": "120.500000"}
            }"#,
        )
        .expect("probe json should parse");

        assert_eq!(summary.mime, "video/mp4");
        assert_eq!(summary.duration_ms, Some(120500));
        assert_eq!(summary.width, Some(1920));
        assert_eq!(summary.video_codec.as_deref(), Some("h264"));
        assert_eq!(summary.audio_codec.as_deref(), Some("aac"));
    }
}
