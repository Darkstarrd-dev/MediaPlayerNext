pub use crate::playback_runtime::resolve_media_asset_path;
pub(crate) use crate::playback_runtime::{
    open_playback_session_with_launcher, probe_asset_with_runner,
};
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository, SourceRepository,
};
use anyhow::Result;
use media_playback::{
    probe_media_file_with_context, MpvProcessLauncher, PlaybackSessionStore, SessionCommand,
};
use serde::Serialize;
use shared_model::{
    AssetId, LogContext, MediaProbeSummary, MediaUrlSummary, PlaybackSessionId,
    PlaybackSessionSummary,
};
use std::path::Path;
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlaybackProbeResult {
    pub asset_id: String,
    pub media_url: String,
    pub probe: MediaProbeSummary,
}

pub fn playback_probe<L, S, A, E, R>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    asset_repository: &R,
    ffprobe_path: &Path,
    asset_id: &AssetId,
) -> Result<PlaybackProbeResult>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
{
    probe_asset_with_runner(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        asset_repository,
        asset_id,
        |input_path| {
            probe_media_file_with_context(
                ffprobe_path,
                input_path,
                LogContext {
                    asset_id: Some(asset_id.clone()),
                    ..LogContext::default()
                },
            )
        },
    )
}
pub fn media_url_for_asset(asset_id: &AssetId) -> MediaUrlSummary {
    MediaUrlSummary {
        asset_id: asset_id.clone(),
        url: format!("media://asset/{}", asset_id.0),
        mime: "application/octet-stream".to_string(),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn open_playback_session<L, S, A, E, R>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    asset_repository: &R,
    sessions_root: &Path,
    mpv_path: &Path,
    asset_id: &AssetId,
) -> Result<PlaybackSessionSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
{
    let launcher = MpvProcessLauncher;
    open_playback_session_with_launcher(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        asset_repository,
        sessions_root,
        mpv_path,
        asset_id,
        &launcher,
    )
}

pub fn playback_status(
    sessions_root: &Path,
    session_id: &PlaybackSessionId,
) -> Result<PlaybackSessionSummary> {
    PlaybackSessionStore::new(sessions_root).get(session_id)
}

pub fn playback_pause(
    sessions_root: &Path,
    session_id: &PlaybackSessionId,
) -> Result<PlaybackSessionSummary> {
    PlaybackSessionStore::new(sessions_root).apply_command(session_id, SessionCommand::Pause)
}

pub fn playback_seek(
    sessions_root: &Path,
    session_id: &PlaybackSessionId,
    position_ms: i64,
) -> Result<PlaybackSessionSummary> {
    PlaybackSessionStore::new(sessions_root)
        .apply_command(session_id, SessionCommand::Seek { position_ms })
}

pub fn playback_stop(
    sessions_root: &Path,
    session_id: &PlaybackSessionId,
) -> Result<PlaybackSessionSummary> {
    PlaybackSessionStore::new(sessions_root).apply_command(session_id, SessionCommand::Stop)
}
#[cfg(test)]
mod tests {
    use super::{open_playback_session_with_launcher, playback_status, probe_asset_with_runner};
    use crate::asset::file_asset_id_from_source;
    use crate::ports::{
        ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
        SourceRepository,
    };
    #[path = "../playback_test_support.rs"]
    mod support;
    use shared_model::{
        ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, LibraryId,
        LibraryRecord, MediaAssetRecord, MediaProbeSummary, MediaSourceKind, SourceId, SourceKind,
        SourceRecord,
    };
    use std::path::Path;
    use support::{MemoryRepos, MockLauncher};
    use tempfile::tempdir;

    #[test]
    fn probes_file_asset_and_backfills_metadata() {
        let repos = MemoryRepos::default();
        let temp = tempdir().expect("tempdir should exist");
        let source_path = temp.path().join("video.mp4");
        std::fs::write(&source_path, b"video").expect("file should exist");
        let library = LibraryRecord {
            id: LibraryId("library_playback_probe".to_string()),
            root_path: temp.path().display().to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        let source = SourceRecord {
            id: SourceId("source_video_probe".to_string()),
            library_id: library.id.clone(),
            normalized_path: source_path.display().to_string(),
            file_name: "video.mp4".to_string(),
            ext: "mp4".to_string(),
            kind: SourceKind::Video,
            size: 10,
            mtime_ms: 1,
            fingerprint: Some("fp-video-probe".to_string()),
            exists: true,
            last_seen_at: "1".to_string(),
        };
        let asset = MediaAssetRecord {
            id: file_asset_id_from_source(&source.id),
            source_kind: MediaSourceKind::File,
            source_ref_id: source.id.0.clone(),
            mime: "video/mp4".to_string(),
            width: None,
            height: None,
            duration_ms: None,
            codec_info_json: None,
            orientation: None,
            created_at: "1".to_string(),
        };
        LibraryRepository::upsert(&repos, &library).expect("library should exist");
        SourceRepository::upsert(&repos, &source).expect("source should exist");
        AssetRepository::upsert(&repos, &asset).expect("asset should exist");
        let summary =
            probe_asset_with_runner(&repos, &repos, &repos, &repos, &repos, &asset.id, |_path| {
                Ok(MediaProbeSummary {
                    mime: "video/mp4".to_string(),
                    container_format: Some("mp4".to_string()),
                    duration_ms: Some(9876),
                    width: Some(1280),
                    height: Some(720),
                    video_codec: Some("h264".to_string()),
                    audio_codec: Some("aac".to_string()),
                })
            })
            .expect("probe should succeed");
        assert_eq!(summary.asset_id, asset.id.0);
        let updated = AssetRepository::get(&repos, &asset.id)
            .expect("asset fetch should work")
            .expect("asset should exist");
        assert_eq!(updated.duration_ms, Some(9876));
        assert_eq!(updated.width, Some(1280));
        assert!(updated.codec_info_json.is_some());
    }
    #[test]
    fn opens_and_reads_playback_session() {
        let repos = MemoryRepos::default();
        let temp = tempdir().expect("tempdir should exist");
        let source_path = temp.path().join("video.mp4");
        std::fs::write(&source_path, b"video").expect("file should exist");
        let library = LibraryRecord {
            id: LibraryId("library_playback_session".to_string()),
            root_path: temp.path().display().to_string(),
            library_type: "filesystem".to_string(),
            scan_mode: "full".to_string(),
            created_at: "1".to_string(),
            updated_at: "1".to_string(),
        };
        let source = SourceRecord {
            id: SourceId("source_video_session".to_string()),
            library_id: library.id.clone(),
            normalized_path: source_path.display().to_string(),
            file_name: "video.mp4".to_string(),
            ext: "mp4".to_string(),
            kind: SourceKind::Video,
            size: 10,
            mtime_ms: 1,
            fingerprint: Some("fp-video-session".to_string()),
            exists: true,
            last_seen_at: "1".to_string(),
        };
        let asset = MediaAssetRecord {
            id: file_asset_id_from_source(&source.id),
            source_kind: MediaSourceKind::File,
            source_ref_id: source.id.0.clone(),
            mime: "video/mp4".to_string(),
            width: None,
            height: None,
            duration_ms: None,
            codec_info_json: None,
            orientation: None,
            created_at: "1".to_string(),
        };
        LibraryRepository::upsert(&repos, &library).expect("library should exist");
        SourceRepository::upsert(&repos, &source).expect("source should exist");
        AssetRepository::upsert(&repos, &asset).expect("asset should exist");
        let summary = open_playback_session_with_launcher(
            &repos,
            &repos,
            &repos,
            &repos,
            &repos,
            temp.path(),
            Path::new("C:/mpv.exe"),
            &asset.id,
            &MockLauncher,
        )
        .expect("session should open");
        let fetched =
            playback_status(temp.path(), &summary.session_id).expect("session fetch should work");
        assert_eq!(fetched.asset_id, asset.id);
        assert_eq!(fetched.media_url, format!("media://asset/{}", asset.id.0));
    }
}
