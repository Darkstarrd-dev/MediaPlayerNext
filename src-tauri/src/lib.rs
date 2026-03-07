mod runtime_check;

use app_core::archive::{read_archive_entry, resolve_archive_entry_location};
use app_core::playback::resolve_media_asset_path;
use app_core::thumbnail::get_thumbnail;
use media_db::{DatabaseLocation, MediaDatabase};
use runtime_check::{run_runtime_smoke_check, RuntimeSmokeCheckResult};
use shared_model::{ArchiveEntryId, AssetId, ThumbnailKey};
use std::env;
use std::path::{Path, PathBuf};
use tauri::http::{header::CONTENT_TYPE, Response, StatusCode, Uri};

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {name}! MediaPlayerNext host is ready.")
}

#[tauri::command]
fn runtime_smoke_check(
    ffmpeg_path: String,
    ffprobe_path: String,
    mpv_path: String,
) -> Result<RuntimeSmokeCheckResult, String> {
    run_runtime_smoke_check(&ffmpeg_path, &ffprobe_path, &mpv_path)
        .map_err(|error| error.to_string())
}

pub fn runtime_smoke_check_entry() -> anyhow::Result<()> {
    let mut ffmpeg_path: Option<String> = None;
    let mut ffprobe_path: Option<String> = None;
    let mut mpv_path: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ffmpeg-path" => {
                ffmpeg_path = args.next();
            }
            "--ffprobe-path" => {
                ffprobe_path = args.next();
            }
            "--mpv-path" => {
                mpv_path = args.next();
            }
            _ => {}
        }
    }

    let ffmpeg_path = ffmpeg_path.ok_or_else(|| anyhow::anyhow!("missing --ffmpeg-path"))?;
    let ffprobe_path = ffprobe_path.ok_or_else(|| anyhow::anyhow!("missing --ffprobe-path"))?;
    let mpv_path = mpv_path.ok_or_else(|| anyhow::anyhow!("missing --mpv-path"))?;
    let result = run_runtime_smoke_check(&ffmpeg_path, &ffprobe_path, &mpv_path)?;
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, runtime_smoke_check])
        .register_uri_scheme_protocol("thumb", |_app, request| {
            match thumbnail_protocol_response(&development_database_path(), request.uri()) {
                Ok(response) => response,
                Err(error) => text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error.to_string(),
                    "text/plain; charset=utf-8",
                ),
            }
        })
        .register_uri_scheme_protocol("media", |_app, request| {
            match media_protocol_response(&development_database_path(), request.uri()) {
                Ok(response) => response,
                Err(error) => text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error.to_string(),
                    "text/plain; charset=utf-8",
                ),
            }
        })
        .register_uri_scheme_protocol("archive", |_app, request| {
            match archive_protocol_response(&development_database_path(), request.uri()) {
                Ok(response) => response,
                Err(error) => text_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    error.to_string(),
                    "text/plain; charset=utf-8",
                ),
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn thumbnail_protocol_response(db_path: &Path, uri: &Uri) -> anyhow::Result<Response<Vec<u8>>> {
    let thumbnail_key = parse_thumbnail_key_from_uri(uri)?;
    let database = MediaDatabase::open(DatabaseLocation::File(db_path))?;
    let repositories = database.repositories();
    let thumbnail = match get_thumbnail(&repositories, &ThumbnailKey(thumbnail_key.clone())) {
        Ok(record) => record,
        Err(_) => {
            return Ok(text_response(
                StatusCode::NOT_FOUND,
                format!("thumbnail not found: {thumbnail_key}"),
                "text/plain; charset=utf-8",
            ));
        }
    };

    let bytes = match std::fs::read(&thumbnail.disk_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(text_response(
                StatusCode::NOT_FOUND,
                format!("thumbnail file not found: {}", thumbnail.disk_path),
                "text/plain; charset=utf-8",
            ));
        }
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            content_type_for_thumbnail_format(&thumbnail.format),
        )
        .body(bytes)
        .expect("thumbnail response should build"))
}

fn media_protocol_response(db_path: &Path, uri: &Uri) -> anyhow::Result<Response<Vec<u8>>> {
    let asset_id = parse_media_asset_id_from_uri(uri)?;
    let database = MediaDatabase::open(DatabaseLocation::File(db_path))?;
    let repositories = database.repositories();
    let (media_path, mime) = match resolve_media_asset_path(
        &repositories,
        &repositories,
        &repositories,
        &repositories,
        &repositories,
        &AssetId(asset_id.clone()),
    ) {
        Ok(result) => result,
        Err(_) => {
            return Ok(text_response(
                StatusCode::NOT_FOUND,
                format!("media asset not found: {asset_id}"),
                "text/plain; charset=utf-8",
            ));
        }
    };
    let bytes = match std::fs::read(&media_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(text_response(
                StatusCode::NOT_FOUND,
                format!("media file not found: {}", media_path.display()),
                "text/plain; charset=utf-8",
            ));
        }
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            content_type_for_media_path(&media_path, &mime),
        )
        .body(bytes)
        .expect("media response should build"))
}

fn archive_protocol_response(db_path: &Path, uri: &Uri) -> anyhow::Result<Response<Vec<u8>>> {
    let archive_entry_id = parse_archive_entry_id_from_uri(uri)?;
    let database = MediaDatabase::open(DatabaseLocation::File(db_path))?;
    let repositories = database.repositories();
    let location = match resolve_archive_entry_location(
        &repositories,
        &repositories,
        &repositories,
        &repositories,
        &ArchiveEntryId(archive_entry_id.clone()),
    ) {
        Ok(location) => location,
        Err(_) => {
            return Ok(text_response(
                StatusCode::NOT_FOUND,
                format!("archive entry not found: {archive_entry_id}"),
                "text/plain; charset=utf-8",
            ));
        }
    };

    let bytes = match read_archive_entry(Path::new(&location.archive_path), &location.entry_path) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(text_response(
                StatusCode::NOT_FOUND,
                format!(
                    "archive entry file not found: {}::{}",
                    location.archive_path, location.entry_path
                ),
                "text/plain; charset=utf-8",
            ));
        }
    };

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header(
            CONTENT_TYPE,
            content_type_for_path(Path::new(&location.entry_path), &location.media_kind),
        )
        .body(bytes)
        .expect("archive response should build"))
}

fn parse_thumbnail_key_from_uri(uri: &Uri) -> anyhow::Result<String> {
    let path_segments = uri
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match path_segments.as_slice() {
        ["cache", thumbnail_key] => Ok((*thumbnail_key).to_string()),
        [thumbnail_key] if uri.host() == Some("cache") => Ok((*thumbnail_key).to_string()),
        _ => Err(anyhow::anyhow!("unsupported thumb uri: {uri}")),
    }
}

fn parse_media_asset_id_from_uri(uri: &Uri) -> anyhow::Result<String> {
    let path_segments = uri
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match path_segments.as_slice() {
        ["asset", asset_id] => Ok((*asset_id).to_string()),
        [asset_id] if uri.host() == Some("asset") => Ok((*asset_id).to_string()),
        _ => Err(anyhow::anyhow!("unsupported media uri: {uri}")),
    }
}

fn parse_archive_entry_id_from_uri(uri: &Uri) -> anyhow::Result<String> {
    let path_segments = uri
        .path()
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    match path_segments.as_slice() {
        ["entry", archive_entry_id] => Ok((*archive_entry_id).to_string()),
        [archive_entry_id] if uri.host() == Some("entry") => Ok((*archive_entry_id).to_string()),
        _ => Err(anyhow::anyhow!("unsupported archive uri: {uri}")),
    }
}

fn content_type_for_thumbnail_format(format: &str) -> &'static str {
    match format {
        "webp" => "image/webp",
        "png" => "image/png",
        "jpeg" | "jpg" => "image/jpeg",
        _ => "application/octet-stream",
    }
}

fn content_type_for_media_path(path: &Path, fallback_mime: &str) -> &'static str {
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

fn content_type_for_path(path: &Path, media_kind: &str) -> &'static str {
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

fn text_response(status: StatusCode, message: String, content_type: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header(CONTENT_TYPE, content_type)
        .body(message.into_bytes())
        .expect("text response should build")
}

fn development_database_path() -> PathBuf {
    workspace_root().join("data").join("mediaplayernext-dev.db")
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .expect("workspace root should be available")
}

#[cfg(test)]
mod tests {
    use super::{
        archive_protocol_response, media_protocol_response, parse_archive_entry_id_from_uri,
        parse_media_asset_id_from_uri, parse_thumbnail_key_from_uri, thumbnail_protocol_response,
    };
    use app_core::ports::{
        ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
        SourceRepository, ThumbnailRepository,
    };
    use media_db::{DatabaseLocation, MediaDatabase};
    use shared_model::{
        ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, LibraryId,
        LibraryRecord, MediaAssetRecord, MediaSourceKind, SourceId, SourceKind, SourceRecord,
        ThumbnailKey, ThumbnailRecord,
    };
    use tauri::http::{header::CONTENT_TYPE, StatusCode, Uri};
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn parses_thumb_uri_key() {
        let uri: Uri = "thumb://cache/thumb_primary"
            .parse()
            .expect("uri should parse");
        let key = parse_thumbnail_key_from_uri(&uri).expect("key should parse");

        assert_eq!(key, "thumb_primary");
    }

    #[test]
    fn parses_media_and_archive_uri_keys() {
        let media_uri: Uri = "media://asset/asset_video_primary"
            .parse()
            .expect("media uri should parse");
        let archive_uri: Uri = "archive://entry/archive_entry_primary"
            .parse()
            .expect("archive uri should parse");

        assert_eq!(
            parse_media_asset_id_from_uri(&media_uri).expect("media asset id should parse"),
            "asset_video_primary"
        );
        assert_eq!(
            parse_archive_entry_id_from_uri(&archive_uri).expect("archive entry id should parse"),
            "archive_entry_primary"
        );
    }

    #[test]
    fn serves_thumbnail_file_from_protocol_handler() {
        let temp = tempdir().expect("tempdir should exist");
        let db_file = NamedTempFile::new().expect("db file should exist");
        let thumbnail_path = temp.path().join("thumb.webp");
        std::fs::write(&thumbnail_path, b"webp-bytes").expect("thumbnail should be written");

        let database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let repositories = database.repositories();
        let asset = MediaAssetRecord {
            id: AssetId("asset_thumb_protocol".to_string()),
            source_kind: MediaSourceKind::File,
            source_ref_id: "source_thumb_protocol".to_string(),
            mime: "image/png".to_string(),
            width: None,
            height: None,
            duration_ms: None,
            codec_info_json: None,
            orientation: None,
            created_at: "1".to_string(),
        };
        AssetRepository::upsert(&repositories, &asset).expect("asset should be stored");
        ThumbnailRepository::upsert(
            &repositories,
            &ThumbnailRecord {
                thumbnail_key: ThumbnailKey("thumb_primary".to_string()),
                asset_id: asset.id,
                profile: "grid-sm".to_string(),
                width: 240,
                height: 180,
                format: "webp".to_string(),
                disk_path: thumbnail_path.display().to_string(),
                byte_size: 10,
                state: "ready".to_string(),
                updated_at: "1".to_string(),
            },
        )
        .expect("thumbnail should be stored");

        let uri: Uri = "thumb://cache/thumb_primary"
            .parse()
            .expect("uri should parse");
        let response = thumbnail_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).expect("content type"),
            "image/webp"
        );
        assert_eq!(response.body(), b"webp-bytes");
    }

    #[test]
    fn returns_not_found_when_thumbnail_record_missing() {
        let db_file = NamedTempFile::new().expect("db file should exist");
        let _database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let uri: Uri = "thumb://cache/thumb_missing"
            .parse()
            .expect("uri should parse");

        let response = thumbnail_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn serves_media_file_from_protocol_handler() {
        let temp = tempdir().expect("tempdir should exist");
        let db_file = NamedTempFile::new().expect("db file should exist");
        let media_path = temp.path().join("video.mp4");
        std::fs::write(&media_path, b"video-bytes").expect("media file should exist");

        let database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let repositories = database.repositories();
        LibraryRepository::upsert(
            &repositories,
            &LibraryRecord {
                id: LibraryId("library_media_protocol".to_string()),
                root_path: temp.path().display().to_string(),
                library_type: "filesystem".to_string(),
                scan_mode: "full".to_string(),
                created_at: "1".to_string(),
                updated_at: "1".to_string(),
            },
        )
        .expect("library should store");
        SourceRepository::upsert(
            &repositories,
            &SourceRecord {
                id: SourceId("source_media_protocol".to_string()),
                library_id: LibraryId("library_media_protocol".to_string()),
                normalized_path: media_path.display().to_string(),
                file_name: "video.mp4".to_string(),
                ext: "mp4".to_string(),
                kind: SourceKind::Video,
                size: 10,
                mtime_ms: 1,
                fingerprint: Some("fp-media-protocol".to_string()),
                exists: true,
                last_seen_at: "1".to_string(),
            },
        )
        .expect("source should store");
        AssetRepository::upsert(
            &repositories,
            &MediaAssetRecord {
                id: AssetId("asset_media_protocol".to_string()),
                source_kind: MediaSourceKind::File,
                source_ref_id: "source_media_protocol".to_string(),
                mime: "video/mp4".to_string(),
                width: None,
                height: None,
                duration_ms: None,
                codec_info_json: None,
                orientation: None,
                created_at: "1".to_string(),
            },
        )
        .expect("asset should store");

        let uri: Uri = "media://asset/asset_media_protocol"
            .parse()
            .expect("uri should parse");
        let response = media_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).expect("content type"),
            "video/mp4"
        );
        assert_eq!(response.body(), b"video-bytes");
    }

    #[test]
    fn serves_archive_entry_from_protocol_handler() {
        let temp = tempdir().expect("tempdir should exist");
        let db_file = NamedTempFile::new().expect("db file should exist");
        let zip_path = temp.path().join("chapter.cbz");
        let file = std::fs::File::create(&zip_path).expect("zip file should exist");
        let mut writer = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored);
        use std::io::Write;
        writer
            .start_file("001-cover.png", options)
            .expect("zip entry should start");
        writer
            .write_all(b"cover-bytes")
            .expect("zip entry should write");
        writer.finish().expect("zip should finish");

        let database = MediaDatabase::open(DatabaseLocation::File(db_file.path()))
            .expect("database should open");
        let repositories = database.repositories();
        let library_id = LibraryId("library_archive_protocol".to_string());
        let source_id = SourceId("source_archive_protocol".to_string());
        let archive_id = ArchiveId("archive_protocol".to_string());

        LibraryRepository::upsert(
            &repositories,
            &LibraryRecord {
                id: library_id.clone(),
                root_path: temp.path().display().to_string(),
                library_type: "filesystem".to_string(),
                scan_mode: "full".to_string(),
                created_at: "1".to_string(),
                updated_at: "1".to_string(),
            },
        )
        .expect("library should store");
        SourceRepository::upsert(
            &repositories,
            &SourceRecord {
                id: source_id.clone(),
                library_id,
                normalized_path: zip_path.display().to_string(),
                file_name: "chapter.cbz".to_string(),
                ext: "cbz".to_string(),
                kind: SourceKind::Archive,
                size: 10,
                mtime_ms: 1,
                fingerprint: Some("fp-archive-protocol".to_string()),
                exists: true,
                last_seen_at: "1".to_string(),
            },
        )
        .expect("source should store");
        ArchiveRepository::upsert(
            &repositories,
            &ArchiveRecord {
                id: archive_id.clone(),
                source_id,
                archive_type: "cbz".to_string(),
                normalized_zip_path: Some(zip_path.display().to_string()),
                page_count: Some(1),
                cover_entry_id: Some(ArchiveEntryId("archive_entry_primary".to_string())),
                status: "indexed".to_string(),
            },
        )
        .expect("archive should store");
        ArchiveEntryRepository::replace_for_archive(
            &repositories,
            &archive_id,
            &[ArchiveEntryRecord {
                id: ArchiveEntryId("archive_entry_primary".to_string()),
                archive_id: archive_id.clone(),
                entry_path: "001-cover.png".to_string(),
                entry_name: "001-cover.png".to_string(),
                page_index: 0,
                media_kind: "image".to_string(),
                width: None,
                height: None,
                compressed_size: Some(10),
                uncompressed_size: Some(10),
                crc32: Some(1),
            }],
        )
        .expect("archive entry should store");

        let uri: Uri = "archive://entry/archive_entry_primary"
            .parse()
            .expect("uri should parse");
        let response = archive_protocol_response(db_file.path(), &uri)
            .expect("protocol response should succeed");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers().get(CONTENT_TYPE).expect("content type"),
            "image/png"
        );
        assert_eq!(response.body(), b"cover-bytes");
    }
}
