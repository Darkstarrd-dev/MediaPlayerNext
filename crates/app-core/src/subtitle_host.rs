use crate::ports::SubtitleHostPort;
use anyhow::Result;
use shared_model::{
    AssetId, SubtitleHostSummary, SubtitleProgressEvent, SubtitleSessionId, SubtitleSessionSummary,
};

pub fn subtitle_ping<H>(host: &H) -> Result<SubtitleHostSummary>
where
    H: SubtitleHostPort,
{
    host.ping()
}

pub fn subtitle_health<H>(host: &H) -> Result<SubtitleHostSummary>
where
    H: SubtitleHostPort,
{
    host.health()
}

pub fn subtitle_start_session<H>(
    host: &H,
    asset_id: Option<&AssetId>,
) -> Result<SubtitleSessionSummary>
where
    H: SubtitleHostPort,
{
    host.start_session(asset_id)
}

pub fn subtitle_stop_session<H>(
    host: &H,
    session_id: &SubtitleSessionId,
) -> Result<SubtitleSessionSummary>
where
    H: SubtitleHostPort,
{
    host.stop_session(session_id)
}

pub fn subtitle_get_progress<H>(
    host: &H,
    session_id: &SubtitleSessionId,
) -> Result<SubtitleProgressEvent>
where
    H: SubtitleHostPort,
{
    host.get_progress(session_id)
}

#[cfg(test)]
mod tests {
    use super::{
        subtitle_get_progress, subtitle_health, subtitle_ping, subtitle_start_session,
        subtitle_stop_session,
    };
    use crate::ports::SubtitleHostPort;
    use anyhow::Result;
    use shared_model::{
        AssetId, SubtitleHealthSummary, SubtitleHostSummary, SubtitlePingSummary,
        SubtitleProgressEvent, SubtitleSessionId, SubtitleSessionState, SubtitleSessionSummary,
    };

    struct MockHost;

    impl SubtitleHostPort for MockHost {
        fn ping(&self) -> Result<SubtitleHostSummary> {
            Ok(SubtitleHostSummary {
                executable: "node".to_string(),
                entry_path: "dist/src/index.js".to_string(),
                running: true,
                restart_count: 0,
                last_error: None,
                ping: Some(SubtitlePingSummary {
                    service: "subtitle-sidecar".to_string(),
                    protocol_version: "b8-v1".to_string(),
                    transport: "stdio".to_string(),
                }),
                health: None,
            })
        }

        fn health(&self) -> Result<SubtitleHostSummary> {
            Ok(SubtitleHostSummary {
                executable: "node".to_string(),
                entry_path: "dist/src/index.js".to_string(),
                running: true,
                restart_count: 1,
                last_error: Some("first attempt failed".to_string()),
                ping: None,
                health: Some(SubtitleHealthSummary {
                    service: "subtitle-sidecar".to_string(),
                    protocol_version: "b8-v1".to_string(),
                    transport: "stdio".to_string(),
                    node_version: "v22.13.1".to_string(),
                    sharp_version: Some("0.34.4".to_string()),
                    uptime_ms: 10,
                    active_sessions: 1,
                    sessions_root: "data/cache/subtitle/sessions".to_string(),
                }),
            })
        }

        fn start_session(&self, asset_id: Option<&AssetId>) -> Result<SubtitleSessionSummary> {
            Ok(SubtitleSessionSummary {
                session_id: SubtitleSessionId("subtitle_001".to_string()),
                asset_id: asset_id.cloned(),
                state: SubtitleSessionState::Idle,
                progress: 0.0,
                created_at: "1".to_string(),
                updated_at: "1".to_string(),
                output_path: None,
            })
        }

        fn stop_session(&self, session_id: &SubtitleSessionId) -> Result<SubtitleSessionSummary> {
            Ok(SubtitleSessionSummary {
                session_id: session_id.clone(),
                asset_id: None,
                state: SubtitleSessionState::Stopped,
                progress: 1.0,
                created_at: "1".to_string(),
                updated_at: "2".to_string(),
                output_path: None,
            })
        }

        fn get_progress(&self, session_id: &SubtitleSessionId) -> Result<SubtitleProgressEvent> {
            Ok(SubtitleProgressEvent {
                session_id: session_id.clone(),
                state: SubtitleSessionState::Idle,
                progress: 0.0,
                message: Some("waiting".to_string()),
            })
        }
    }

    #[test]
    fn forwards_subtitle_host_calls() {
        let host = MockHost;
        let ping = subtitle_ping(&host).expect("ping should succeed");
        let health = subtitle_health(&host).expect("health should succeed");
        let session = subtitle_start_session(&host, Some(&AssetId("asset_001".to_string())))
            .expect("start session should succeed");
        let progress =
            subtitle_get_progress(&host, &session.session_id).expect("progress should succeed");
        let stopped =
            subtitle_stop_session(&host, &session.session_id).expect("stop session should succeed");

        assert_eq!(ping.ping.expect("ping payload").transport, "stdio");
        assert_eq!(health.restart_count, 1);
        assert_eq!(session.asset_id.expect("asset id").0, "asset_001");
        assert_eq!(progress.message.expect("progress message"), "waiting");
        assert_eq!(stopped.state, SubtitleSessionState::Stopped);
    }
}
