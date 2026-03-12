use anyhow::{anyhow, Result};
use std::path::Path;
use tauri::http::Uri;

pub fn parse_thumbnail_key_from_uri(uri: &Uri) -> Result<String> {
    let path_segments = uri
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match path_segments.as_slice() {
        ["cache", thumbnail_key] => Ok((*thumbnail_key).to_string()),
        [thumbnail_key] if uri.host() == Some("cache") => Ok((*thumbnail_key).to_string()),
        _ => Err(anyhow!("unsupported thumb uri: {uri}")),
    }
}

pub fn parse_media_asset_id_from_uri(uri: &Uri) -> Result<String> {
    let path_segments = uri
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match path_segments.as_slice() {
        ["asset", asset_id] => Ok((*asset_id).to_string()),
        [asset_id] if uri.host() == Some("asset") => Ok((*asset_id).to_string()),
        _ => Err(anyhow!("unsupported media uri: {uri}")),
    }
}

pub fn parse_archive_entry_id_from_uri(uri: &Uri) -> Result<String> {
    let path_segments = uri
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match path_segments.as_slice() {
        ["entry", archive_entry_id] => Ok((*archive_entry_id).to_string()),
        [archive_entry_id] if uri.host() == Some("entry") => Ok((*archive_entry_id).to_string()),
        _ => Err(anyhow!("unsupported archive uri: {uri}")),
    }
}

pub fn content_type_for_thumbnail_format(format: &str) -> &'static str {
    match format {
        "webp" => "image/webp",
        "png" => "image/png",
        "jpeg" | "jpg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

pub fn content_type_for_media_path(path: &Path, fallback_mime: &str) -> &'static str {
    match fallback_mime {
        "video/mp4" => "video/mp4",
        "video/webm" => "video/webm",
        "video/x-matroska" => "video/x-matroska",
        "audio/mpeg" => "audio/mpeg",
        "audio/flac" => "audio/flac",
        "audio/wav" => "audio/wav",
        _ => content_type_for_path(path, ""),
    }
}

pub fn content_type_for_path(path: &Path, media_kind: &str) -> &'static str {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mkv" => "video/x-matroska",
        "mp3" => "audio/mpeg",
        "flac" => "audio/flac",
        "wav" => "audio/wav",
        _ if media_kind.eq_ignore_ascii_case("image") => "image/*",
        _ if media_kind.eq_ignore_ascii_case("video") => "video/*",
        _ if media_kind.eq_ignore_ascii_case("audio") => "audio/*",
        _ => "application/octet-stream",
    }
}
