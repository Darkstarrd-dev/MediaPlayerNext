mod runtime_check;

use app_core::thumbnail::get_thumbnail;
use media_db::{DatabaseLocation, MediaDatabase};
use runtime_check::{run_runtime_smoke_check, RuntimeSmokeCheckResult};
use shared_model::ThumbnailKey;
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
    mpv_path: String,
) -> Result<RuntimeSmokeCheckResult, String> {
    run_runtime_smoke_check(&ffmpeg_path, &mpv_path).map_err(|error| error.to_string())
}

pub fn runtime_smoke_check_entry() -> anyhow::Result<()> {
    let mut ffmpeg_path: Option<String> = None;
    let mut mpv_path: Option<String> = None;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--ffmpeg-path" => {
                ffmpeg_path = args.next();
            }
            "--mpv-path" => {
                mpv_path = args.next();
            }
            _ => {}
        }
    }

    let ffmpeg_path = ffmpeg_path.ok_or_else(|| anyhow::anyhow!("missing --ffmpeg-path"))?;
    let mpv_path = mpv_path.ok_or_else(|| anyhow::anyhow!("missing --mpv-path"))?;
    let result = run_runtime_smoke_check(&ffmpeg_path, &mpv_path)?;
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

fn content_type_for_thumbnail_format(format: &str) -> &'static str {
    match format {
        "webp" => "image/webp",
        "png" => "image/png",
        "jpeg" | "jpg" => "image/jpeg",
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
    use super::{parse_thumbnail_key_from_uri, thumbnail_protocol_response};
    use app_core::ports::{AssetRepository, ThumbnailRepository};
    use media_db::{DatabaseLocation, MediaDatabase};
    use shared_model::{AssetId, MediaAssetRecord, MediaSourceKind, ThumbnailKey, ThumbnailRecord};
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
}
