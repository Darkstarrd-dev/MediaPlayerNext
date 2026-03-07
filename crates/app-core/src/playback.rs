use crate::asset::{resolve_asset, AssetResolution};
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository, SourceRepository,
};
use anyhow::{anyhow, Result};
use media_playback::{
    probe_media_file, MpvLauncher, MpvProcessLauncher, PlaybackSessionStore, SessionCommand,
};
use serde::Serialize;
use shared_model::{
    AssetId, MediaAssetRecord, MediaProbeSummary, MediaUrlSummary, PlaybackSessionId,
    PlaybackSessionSummary,
};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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
        |input_path| probe_media_file(ffprobe_path, input_path),
    )
}

pub fn media_url_for_asset(asset_id: &AssetId) -> MediaUrlSummary {
    MediaUrlSummary {
        asset_id: asset_id.clone(),
        url: format!("media://asset/{}", asset_id.0),
        mime: "application/octet-stream".to_string(),
    }
}

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

pub fn resolve_media_asset_path<L, S, A, E, R>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    asset_repository: &R,
    asset_id: &AssetId,
) -> Result<(PathBuf, String)>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
{
    let asset = asset_repository
        .get(asset_id)?
        .ok_or_else(|| anyhow!("asset not found: {}", asset_id.0))?;
    let resolution = resolve_asset(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        asset_repository,
        asset_id,
    )?;

    match resolution {
        AssetResolution::File(file) => Ok((PathBuf::from(file.file_path), asset.mime)),
        AssetResolution::ArchiveEntry(_) => Err(anyhow!(
            "archive entry assets must use archive://entry protocol"
        )),
    }
}

fn probe_asset_with_runner<L, S, A, E, R, F>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    asset_repository: &R,
    asset_id: &AssetId,
    runner: F,
) -> Result<PlaybackProbeResult>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
    F: Fn(&Path) -> Result<MediaProbeSummary>,
{
    let asset = asset_repository
        .get(asset_id)?
        .ok_or_else(|| anyhow!("asset not found: {}", asset_id.0))?;
    let resolution = resolve_asset(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        asset_repository,
        asset_id,
    )?;
    let input_path = match resolution {
        AssetResolution::File(item) => PathBuf::from(item.file_path),
        AssetResolution::ArchiveEntry(_) => {
            return Err(anyhow!("ffprobe currently only supports file-based assets"));
        }
    };
    let probe = runner(&input_path)?;
    let updated_asset = backfill_asset_from_probe(&asset, &probe)?;
    asset_repository.upsert(&updated_asset)?;

    Ok(PlaybackProbeResult {
        asset_id: asset_id.0.clone(),
        media_url: format!("media://asset/{}", asset_id.0),
        probe,
    })
}

fn open_playback_session_with_launcher<L, S, A, E, R, X>(
    library_repository: &L,
    source_repository: &S,
    archive_repository: &A,
    archive_entry_repository: &E,
    asset_repository: &R,
    sessions_root: &Path,
    mpv_path: &Path,
    asset_id: &AssetId,
    launcher: &X,
) -> Result<PlaybackSessionSummary>
where
    L: LibraryRepository,
    S: SourceRepository,
    A: ArchiveRepository,
    E: ArchiveEntryRepository,
    R: AssetRepository,
    X: MpvLauncher,
{
    let (file_path, _mime) = resolve_media_asset_path(
        library_repository,
        source_repository,
        archive_repository,
        archive_entry_repository,
        asset_repository,
        asset_id,
    )?;
    let store = PlaybackSessionStore::new(sessions_root);

    store.open_with_launcher(
        launcher,
        mpv_path,
        asset_id,
        &format!("media://asset/{}", asset_id.0),
        file_path
            .file_name()
            .map(|value| value.to_string_lossy().to_string()),
    )
}

fn backfill_asset_from_probe(
    asset: &MediaAssetRecord,
    probe: &MediaProbeSummary,
) -> Result<MediaAssetRecord> {
    let mut updated = asset.clone();
    updated.mime = if probe.mime == "application/octet-stream" {
        asset.mime.clone()
    } else {
        probe.mime.clone()
    };
    updated.width = probe.width;
    updated.height = probe.height;
    updated.duration_ms = probe.duration_ms;
    updated.codec_info_json = Some(serde_json::to_string(probe)?);
    updated.created_at = now_string();
    Ok(updated)
}

fn now_string() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::{open_playback_session_with_launcher, playback_status, probe_asset_with_runner};
    use crate::asset::file_asset_id_from_source;
    use crate::ports::{
        ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository,
        SourceRepository,
    };
    use anyhow::Result;
    use media_playback::{MpvLauncher, MpvOpenRequest};
    use shared_model::{
        ArchiveEntryId, ArchiveEntryRecord, ArchiveId, ArchiveRecord, AssetId, LibraryId,
        LibraryRecord, MediaAssetRecord, MediaProbeSummary, MediaSourceKind, SourceId, SourceKind,
        SourceRecord,
    };
    use std::collections::HashMap;
    use std::path::Path;
    use std::sync::Mutex;
    use tempfile::tempdir;

    #[derive(Default)]
    struct MemoryRepos {
        libraries: Mutex<HashMap<String, LibraryRecord>>,
        sources: Mutex<HashMap<String, SourceRecord>>,
        archives: Mutex<HashMap<String, ArchiveRecord>>,
        archive_entries: Mutex<HashMap<String, Vec<ArchiveEntryRecord>>>,
        assets: Mutex<HashMap<String, MediaAssetRecord>>,
    }

    #[derive(Debug)]
    struct MockLauncher;

    impl MpvLauncher for MockLauncher {
        fn launch(&self, _mpv_path: &Path, _request: &MpvOpenRequest) -> Result<()> {
            Ok(())
        }
    }

    impl LibraryRepository for MemoryRepos {
        fn exists(&self, library_id: &LibraryId) -> Result<bool> {
            Ok(self
                .libraries
                .lock()
                .expect("lock")
                .contains_key(&library_id.0))
        }

        fn upsert(&self, library: &LibraryRecord) -> Result<()> {
            self.libraries
                .lock()
                .expect("lock")
                .insert(library.id.0.clone(), library.clone());
            Ok(())
        }

        fn get(&self, library_id: &LibraryId) -> Result<Option<LibraryRecord>> {
            Ok(self
                .libraries
                .lock()
                .expect("lock")
                .get(&library_id.0)
                .cloned())
        }
    }

    impl SourceRepository for MemoryRepos {
        fn exists(&self, source_id: &SourceId) -> Result<bool> {
            Ok(self
                .sources
                .lock()
                .expect("lock")
                .contains_key(&source_id.0))
        }

        fn upsert(&self, source: &SourceRecord) -> Result<()> {
            self.sources
                .lock()
                .expect("lock")
                .insert(source.id.0.clone(), source.clone());
            Ok(())
        }

        fn get(&self, source_id: &SourceId) -> Result<Option<SourceRecord>> {
            Ok(self
                .sources
                .lock()
                .expect("lock")
                .get(&source_id.0)
                .cloned())
        }

        fn count(&self) -> Result<u64> {
            Ok(self.sources.lock().expect("lock").len() as u64)
        }

        fn count_by_library(&self, library_id: &LibraryId) -> Result<u64> {
            Ok(self
                .sources
                .lock()
                .expect("lock")
                .values()
                .filter(|item| item.library_id == *library_id)
                .count() as u64)
        }

        fn list_by_library(&self, library_id: &LibraryId) -> Result<Vec<SourceRecord>> {
            let mut items = self
                .sources
                .lock()
                .expect("lock")
                .values()
                .filter(|item| item.library_id == *library_id)
                .cloned()
                .collect::<Vec<_>>();
            items.sort_by(|left, right| left.normalized_path.cmp(&right.normalized_path));
            Ok(items)
        }
    }

    impl ArchiveRepository for MemoryRepos {
        fn exists(&self, archive_id: &ArchiveId) -> Result<bool> {
            Ok(self
                .archives
                .lock()
                .expect("lock")
                .contains_key(&archive_id.0))
        }

        fn upsert(&self, archive: &ArchiveRecord) -> Result<()> {
            self.archives
                .lock()
                .expect("lock")
                .insert(archive.id.0.clone(), archive.clone());
            Ok(())
        }

        fn get(&self, archive_id: &ArchiveId) -> Result<Option<ArchiveRecord>> {
            Ok(self
                .archives
                .lock()
                .expect("lock")
                .get(&archive_id.0)
                .cloned())
        }

        fn get_by_source(&self, source_id: &SourceId) -> Result<Option<ArchiveRecord>> {
            Ok(self
                .archives
                .lock()
                .expect("lock")
                .values()
                .find(|item| item.source_id == *source_id)
                .cloned())
        }
    }

    impl ArchiveEntryRepository for MemoryRepos {
        fn replace_for_archive(
            &self,
            archive_id: &ArchiveId,
            entries: &[ArchiveEntryRecord],
        ) -> Result<()> {
            self.archive_entries
                .lock()
                .expect("lock")
                .insert(archive_id.0.clone(), entries.to_vec());
            Ok(())
        }

        fn get(&self, archive_entry_id: &ArchiveEntryId) -> Result<Option<ArchiveEntryRecord>> {
            Ok(self
                .archive_entries
                .lock()
                .expect("lock")
                .values()
                .flat_map(|items| items.iter())
                .find(|item| item.id == *archive_entry_id)
                .cloned())
        }

        fn list_by_archive(&self, archive_id: &ArchiveId) -> Result<Vec<ArchiveEntryRecord>> {
            Ok(self
                .archive_entries
                .lock()
                .expect("lock")
                .get(&archive_id.0)
                .cloned()
                .unwrap_or_default())
        }
    }

    impl AssetRepository for MemoryRepos {
        fn exists(&self, asset_id: &AssetId) -> Result<bool> {
            Ok(self.assets.lock().expect("lock").contains_key(&asset_id.0))
        }

        fn upsert(&self, asset: &MediaAssetRecord) -> Result<()> {
            self.assets
                .lock()
                .expect("lock")
                .insert(asset.id.0.clone(), asset.clone());
            Ok(())
        }

        fn get(&self, asset_id: &AssetId) -> Result<Option<MediaAssetRecord>> {
            Ok(self.assets.lock().expect("lock").get(&asset_id.0).cloned())
        }
    }

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
