use crate::asset::{resolve_asset, AssetResolution};
use crate::playback::PlaybackProbeResult;
use crate::ports::{
    ArchiveEntryRepository, ArchiveRepository, AssetRepository, LibraryRepository, SourceRepository,
};
use anyhow::{anyhow, Result};
use media_playback::{MpvLauncher, PlaybackSessionStore};
use shared_model::{AssetId, MediaAssetRecord, MediaProbeSummary, PlaybackSessionSummary};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

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

pub(crate) fn probe_asset_with_runner<L, S, A, E, R, F>(
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

pub(crate) fn open_playback_session_with_launcher<L, S, A, E, R, X>(
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
