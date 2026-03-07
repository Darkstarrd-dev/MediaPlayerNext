use crate::mpv::{MpvLauncher, MpvOpenRequest};
use anyhow::{Context, Result};
use shared_model::{AssetId, PlaybackSessionId, PlaybackSessionState, PlaybackSessionSummary};
use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionCommand {
    Pause,
    Play,
    Stop,
    Seek { position_ms: i64 },
}

#[derive(Debug, Clone)]
pub struct PlaybackSessionStore {
    root_dir: PathBuf,
}

impl PlaybackSessionStore {
    pub fn new(root_dir: impl AsRef<Path>) -> Self {
        Self {
            root_dir: root_dir.as_ref().to_path_buf(),
        }
    }

    pub fn open_with_launcher<L>(
        &self,
        launcher: &L,
        mpv_path: &Path,
        asset_id: &AssetId,
        media_url: &str,
        title: Option<String>,
    ) -> Result<PlaybackSessionSummary>
    where
        L: MpvLauncher,
    {
        let session = PlaybackSessionSummary {
            session_id: PlaybackSessionId(generate_session_id(asset_id, media_url)),
            asset_id: asset_id.clone(),
            media_url: media_url.to_string(),
            state: PlaybackSessionState::Opening,
            position_ms: 0,
            created_at: now_string(),
            updated_at: now_string(),
        };
        self.write(&session)?;

        let request = MpvOpenRequest {
            media_url: media_url.to_string(),
            start_paused: true,
            title,
        };

        match launcher.launch(mpv_path, &request) {
            Ok(()) => self.apply_command(&session.session_id, SessionCommand::Pause),
            Err(error) => {
                let failed = PlaybackSessionSummary {
                    state: PlaybackSessionState::Failed,
                    updated_at: now_string(),
                    ..session
                };
                self.write(&failed)?;
                Err(error)
            }
        }
    }

    pub fn get(&self, session_id: &PlaybackSessionId) -> Result<PlaybackSessionSummary> {
        let path = self.path_for_session(session_id);
        let raw = fs::read_to_string(&path)
            .with_context(|| format!("read playback session file: {}", path.display()))?;
        serde_json::from_str(&raw).context("parse playback session json")
    }

    pub fn apply_command(
        &self,
        session_id: &PlaybackSessionId,
        command: SessionCommand,
    ) -> Result<PlaybackSessionSummary> {
        let current = self.get(session_id)?;
        let next = match command {
            SessionCommand::Pause => PlaybackSessionSummary {
                state: PlaybackSessionState::Paused,
                updated_at: now_string(),
                ..current
            },
            SessionCommand::Play => PlaybackSessionSummary {
                state: PlaybackSessionState::Playing,
                updated_at: now_string(),
                ..current
            },
            SessionCommand::Stop => PlaybackSessionSummary {
                state: PlaybackSessionState::Stopped,
                updated_at: now_string(),
                ..current
            },
            SessionCommand::Seek { position_ms } => PlaybackSessionSummary {
                position_ms,
                updated_at: now_string(),
                ..current
            },
        };
        self.write(&next)?;
        Ok(next)
    }

    fn write(&self, session: &PlaybackSessionSummary) -> Result<()> {
        fs::create_dir_all(&self.root_dir)?;
        let path = self.path_for_session(&session.session_id);
        fs::write(path, serde_json::to_vec_pretty(session)?)
            .context("write playback session file")?;
        Ok(())
    }

    fn path_for_session(&self, session_id: &PlaybackSessionId) -> PathBuf {
        self.root_dir.join(format!("{}.json", session_id.0))
    }
}

fn generate_session_id(asset_id: &AssetId, media_url: &str) -> String {
    let mut hasher = DefaultHasher::new();
    format!("{}::{}::{}", asset_id.0, media_url, now_string()).hash(&mut hasher);
    format!("playback_{:016x}", hasher.finish())
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
    use super::{PlaybackSessionStore, SessionCommand};
    use crate::mpv::{MpvLauncher, MpvOpenRequest};
    use anyhow::{anyhow, Result};
    use shared_model::{AssetId, PlaybackSessionState};
    use std::path::Path;
    use tempfile::tempdir;

    #[derive(Debug)]
    struct MockLauncher {
        should_fail: bool,
    }

    impl MpvLauncher for MockLauncher {
        fn launch(&self, _mpv_path: &Path, _request: &MpvOpenRequest) -> Result<()> {
            if self.should_fail {
                return Err(anyhow!("mock mpv launch failure"));
            }
            Ok(())
        }
    }

    #[test]
    fn persists_and_updates_session_state() {
        let temp = tempdir().expect("tempdir should exist");
        let store = PlaybackSessionStore::new(temp.path());
        let summary = store
            .open_with_launcher(
                &MockLauncher { should_fail: false },
                Path::new("C:/mpv.exe"),
                &AssetId("asset_video_primary".to_string()),
                "media://asset/asset_video_primary",
                Some("Asset Video".to_string()),
            )
            .expect("session should open");

        assert_eq!(summary.state, PlaybackSessionState::Paused);
        let updated = store
            .apply_command(
                &summary.session_id,
                SessionCommand::Seek { position_ms: 9000 },
            )
            .expect("seek should persist");
        assert_eq!(updated.position_ms, 9000);
    }

    #[test]
    fn marks_failed_session_when_launcher_fails() {
        let temp = tempdir().expect("tempdir should exist");
        let store = PlaybackSessionStore::new(temp.path());
        let result = store.open_with_launcher(
            &MockLauncher { should_fail: true },
            Path::new("C:/mpv.exe"),
            &AssetId("asset_video_fail".to_string()),
            "media://asset/asset_video_fail",
            None,
        );

        assert!(result.is_err());
    }
}
