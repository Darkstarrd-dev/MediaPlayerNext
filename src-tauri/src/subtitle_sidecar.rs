use anyhow::{anyhow, Context, Result};
use app_core::ports::SubtitleHostPort;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use shared_model::{
    AssetId, SubtitleHealthSummary, SubtitleHostSummary, SubtitlePingSummary,
    SubtitleProgressEvent, SubtitleSessionId, SubtitleSessionSummary,
};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_REQUEST_TIMEOUT_MS: u64 = 3_000;
const DEFAULT_MAX_RESTARTS: u32 = 1;

#[derive(Debug, Clone)]
pub struct SubtitleSidecarRuntime {
    pub node_path: PathBuf,
    pub entry_path: PathBuf,
    pub sessions_root: PathBuf,
    pub extra_env: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub struct StdioSubtitleHost {
    runtime: SubtitleSidecarRuntime,
    request_timeout: Duration,
    max_restarts: u32,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalPathsConfig {
    node: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct WireRequest {
    id: String,
    #[serde(rename = "type")]
    message_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireResponse {
    id: String,
    #[serde(rename = "type")]
    message_type: String,
    ok: bool,
    #[serde(default)]
    payload: Option<Value>,
    #[serde(default)]
    error: Option<WireError>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireError {
    code: String,
    message: String,
    #[serde(default)]
    retriable: Option<bool>,
}

pub fn development_subtitle_host() -> Result<StdioSubtitleHost> {
    let workspace_root = workspace_root();
    let config_path = workspace_root.join("config").join("local.paths.json");
    let node_path = load_node_path(&config_path)?;
    let entry_path = workspace_root
        .join("apps")
        .join("subtitle-sidecar")
        .join("dist")
        .join("src")
        .join("index.js");
    let sessions_root = workspace_root
        .join("data")
        .join("cache")
        .join("subtitle")
        .join("sessions");

    Ok(StdioSubtitleHost::new(SubtitleSidecarRuntime {
        node_path,
        entry_path,
        sessions_root,
        extra_env: Vec::new(),
    }))
}

impl StdioSubtitleHost {
    pub fn new(runtime: SubtitleSidecarRuntime) -> Self {
        Self {
            runtime,
            request_timeout: Duration::from_millis(DEFAULT_REQUEST_TIMEOUT_MS),
            max_restarts: DEFAULT_MAX_RESTARTS,
        }
    }

    #[cfg(test)]
    fn with_options(
        runtime: SubtitleSidecarRuntime,
        request_timeout: Duration,
        max_restarts: u32,
    ) -> Self {
        Self {
            runtime,
            request_timeout,
            max_restarts,
        }
    }

    fn request<T>(
        &self,
        message_type: &str,
        payload: Option<Value>,
        retryable: bool,
    ) -> Result<(T, u32, Option<String>)>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut restart_count = 0_u32;
        let mut last_error = None;

        loop {
            match self.run_once::<T>(message_type, payload.clone()) {
                Ok(response) => return Ok((response, restart_count, last_error)),
                Err(error) if retryable && restart_count < self.max_restarts => {
                    restart_count += 1;
                    last_error = Some(error.to_string());
                }
                Err(error) => return Err(error),
            }
        }
    }

    fn run_once<T>(&self, message_type: &str, payload: Option<Value>) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        ensure_runtime_ready(&self.runtime)?;

        let request = WireRequest {
            id: next_request_id(message_type),
            message_type: message_type.to_string(),
            payload,
        };
        let request_line = format!("{}\n", serde_json::to_string(&request)?);

        let mut command = Command::new(&self.runtime.node_path);
        command
            .arg(&self.runtime.entry_path)
            .arg("--sessions-root")
            .arg(&self.runtime.sessions_root)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        for (key, value) in &self.runtime.extra_env {
            command.env(key, value);
        }

        let mut child = command.spawn().with_context(|| {
            format!(
                "spawn subtitle sidecar: {} {}",
                self.runtime.node_path.display(),
                self.runtime.entry_path.display()
            )
        })?;

        {
            let stdin = child
                .stdin
                .as_mut()
                .ok_or_else(|| anyhow!("subtitle sidecar stdin unavailable"))?;
            stdin.write_all(request_line.as_bytes())?;
            stdin.flush()?;
        }

        drop(child.stdin.take());

        let started_at = Instant::now();
        loop {
            if child.try_wait()?.is_some() {
                break;
            }

            if started_at.elapsed() > self.request_timeout {
                child.kill().ok();
                let output = child.wait_with_output()?;
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                return Err(anyhow!(
                    "subtitle sidecar timed out after {} ms{}",
                    self.request_timeout.as_millis(),
                    if stderr.is_empty() {
                        String::new()
                    } else {
                        format!(": {stderr}")
                    }
                ));
            }

            thread::sleep(Duration::from_millis(10));
        }

        let output = child.wait_with_output()?;
        let stdout = String::from_utf8(output.stdout)
            .context("subtitle sidecar stdout is not valid utf-8")?;
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

        if !output.status.success() {
            return Err(anyhow!(
                "subtitle sidecar exited with status {}{}",
                output.status,
                if stderr.is_empty() {
                    String::new()
                } else {
                    format!(": {stderr}")
                }
            ));
        }

        let response_line = stdout
            .lines()
            .find(|line| !line.trim().is_empty())
            .ok_or_else(|| anyhow!("subtitle sidecar returned empty stdout"))?;
        let response: WireResponse =
            serde_json::from_str(response_line).context("parse subtitle sidecar response")?;

        if response.id != request.id {
            return Err(anyhow!(
                "subtitle sidecar response id mismatch: expected {}, got {}",
                request.id,
                response.id
            ));
        }

        if !response.ok {
            let error = response.error.unwrap_or(WireError {
                code: "INTERNAL_ERROR".to_string(),
                message: "subtitle sidecar returned error without details".to_string(),
                retriable: Some(false),
            });
            return Err(anyhow!(
                "subtitle sidecar {} failed [{}]{}: {}",
                response.message_type,
                error.code,
                if error.retriable.unwrap_or(false) {
                    ", retriable"
                } else {
                    ""
                },
                error.message
            ));
        }

        let payload = response
            .payload
            .ok_or_else(|| anyhow!("subtitle sidecar response payload missing"))?;
        serde_json::from_value(payload).context("deserialize subtitle sidecar payload")
    }

    fn host_summary(
        &self,
        restart_count: u32,
        last_error: Option<String>,
        ping: Option<SubtitlePingSummary>,
        health: Option<SubtitleHealthSummary>,
    ) -> SubtitleHostSummary {
        SubtitleHostSummary {
            executable: self.runtime.node_path.display().to_string(),
            entry_path: self.runtime.entry_path.display().to_string(),
            running: true,
            restart_count,
            last_error,
            ping,
            health,
        }
    }
}

impl SubtitleHostPort for StdioSubtitleHost {
    fn ping(&self) -> Result<SubtitleHostSummary> {
        let (ping, restart_count, last_error) =
            self.request::<SubtitlePingSummary>("ping", None, true)?;
        Ok(self.host_summary(restart_count, last_error, Some(ping), None))
    }

    fn health(&self) -> Result<SubtitleHostSummary> {
        let (health, restart_count, last_error) =
            self.request::<SubtitleHealthSummary>("health", None, true)?;
        Ok(self.host_summary(restart_count, last_error, None, Some(health)))
    }

    fn start_session(&self, asset_id: Option<&AssetId>) -> Result<SubtitleSessionSummary> {
        let payload = asset_id.map(|asset_id| json!({ "assetId": asset_id.0 }));
        let (session, _, _) =
            self.request::<SubtitleSessionSummary>("start_session", payload, false)?;
        Ok(session)
    }

    fn stop_session(&self, session_id: &SubtitleSessionId) -> Result<SubtitleSessionSummary> {
        let (session, _, _) = self.request::<SubtitleSessionSummary>(
            "stop_session",
            Some(json!({ "sessionId": session_id.0 })),
            false,
        )?;
        Ok(session)
    }

    fn get_progress(&self, session_id: &SubtitleSessionId) -> Result<SubtitleProgressEvent> {
        let (progress, _, _) = self.request::<SubtitleProgressEvent>(
            "get_progress",
            Some(json!({ "sessionId": session_id.0 })),
            true,
        )?;
        Ok(progress)
    }
}

fn ensure_runtime_ready(runtime: &SubtitleSidecarRuntime) -> Result<()> {
    if runtime.entry_path.exists() {
        if let Some(parent) = runtime.sessions_root.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::create_dir_all(&runtime.sessions_root)?;
        return Ok(());
    }

    Err(anyhow!(
        "subtitle sidecar entry missing: {} (run `npm run build --workspace @mediaplayernext/subtitle-sidecar` first)",
        runtime.entry_path.display()
    ))
}

fn load_node_path(config_path: &Path) -> Result<PathBuf> {
    let config = if config_path.exists() {
        serde_json::from_slice::<LocalPathsConfig>(&fs::read(config_path)?)?
    } else {
        LocalPathsConfig::default()
    };

    Ok(PathBuf::from(
        config.node.unwrap_or_else(|| "node".to_string()),
    ))
}

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .map(PathBuf::from)
        .expect("workspace root should be available")
}

fn next_request_id(message_type: &str) -> String {
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("req-{message_type}-{millis}")
}

#[cfg(test)]
mod tests {
    use super::{StdioSubtitleHost, SubtitleSidecarRuntime};
    use app_core::ports::SubtitleHostPort;
    use shared_model::{AssetId, SubtitleSessionId, SubtitleSessionState};
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;
    use tempfile::tempdir;

    #[test]
    fn talks_to_mock_sidecar_and_persists_session_flow() {
        let temp = tempdir().expect("tempdir should exist");
        let script_path = temp.path().join("mock-sidecar.mjs");
        let sessions_root = temp.path().join("sessions");
        fs::write(&script_path, mock_sidecar_script()).expect("mock sidecar script should exist");

        let host = StdioSubtitleHost::with_options(
            SubtitleSidecarRuntime {
                node_path: PathBuf::from("node"),
                entry_path: script_path,
                sessions_root,
                extra_env: Vec::new(),
            },
            Duration::from_millis(1_500),
            1,
        );

        let ping = host.ping().expect("ping should succeed");
        let health = host.health().expect("health should succeed");
        let session = host
            .start_session(Some(&AssetId("asset_sidecar_primary".to_string())))
            .expect("start session should succeed");
        let progress = host
            .get_progress(&session.session_id)
            .expect("progress should succeed");
        let stopped = host
            .stop_session(&session.session_id)
            .expect("stop session should succeed");

        assert_eq!(ping.ping.expect("ping payload").protocol_version, "b8-v1");
        assert_eq!(health.health.expect("health payload").active_sessions, 0);
        assert_eq!(
            session.asset_id.expect("asset id").0,
            "asset_sidecar_primary"
        );
        assert_eq!(progress.state, SubtitleSessionState::Idle);
        assert_eq!(stopped.state, SubtitleSessionState::Stopped);
    }

    #[test]
    fn retries_health_after_retryable_crash() {
        let temp = tempdir().expect("tempdir should exist");
        let script_path = temp.path().join("retry-sidecar.mjs");
        let sessions_root = temp.path().join("sessions");
        fs::write(&script_path, retry_sidecar_script()).expect("retry sidecar script should exist");

        let host = StdioSubtitleHost::with_options(
            SubtitleSidecarRuntime {
                node_path: PathBuf::from("node"),
                entry_path: script_path,
                sessions_root,
                extra_env: vec![(
                    "SUBTITLE_SIDECAR_ATTEMPT_FILE".to_string(),
                    temp.path().join("attempt.txt").display().to_string(),
                )],
            },
            Duration::from_millis(1_500),
            1,
        );

        let health = host.health().expect("health should succeed after retry");

        assert_eq!(health.restart_count, 1);
        assert!(health
            .last_error
            .expect("last error")
            .contains("exited with status"));
        assert_eq!(
            health.health.expect("health payload").service,
            "subtitle-sidecar"
        );
    }

    #[test]
    fn reports_not_found_for_missing_session_progress() {
        let temp = tempdir().expect("tempdir should exist");
        let script_path = temp.path().join("mock-sidecar.mjs");
        let sessions_root = temp.path().join("sessions");
        fs::write(&script_path, mock_sidecar_script()).expect("mock sidecar script should exist");

        let host = StdioSubtitleHost::with_options(
            SubtitleSidecarRuntime {
                node_path: PathBuf::from("node"),
                entry_path: script_path,
                sessions_root,
                extra_env: Vec::new(),
            },
            Duration::from_millis(1_500),
            0,
        );

        let error = host
            .get_progress(&SubtitleSessionId("subtitle_missing".to_string()))
            .expect_err("missing session should fail");

        assert!(error.to_string().contains("NOT_FOUND"));
    }

    fn mock_sidecar_script() -> &'static str {
        r#"import { createInterface } from 'node:readline';
import { mkdir, readFile, readdir, writeFile } from 'node:fs/promises';
import { join } from 'node:path';

const args = process.argv.slice(2);
const sessionsRoot = args[args.indexOf('--sessions-root') + 1];
await mkdir(sessionsRoot, { recursive: true });

const now = () => String(Date.now());
const sessionPath = (sessionId) => join(sessionsRoot, `${sessionId}.json`);
const writeSession = async (session) => {
  await writeFile(sessionPath(session.sessionId), JSON.stringify(session, null, 2));
};
const readSession = async (sessionId) => JSON.parse(await readFile(sessionPath(sessionId), 'utf8'));

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of rl) {
  if (!line.trim()) {
    continue;
  }

  const request = JSON.parse(line);
  let response;
  switch (request.type) {
    case 'ping':
      response = {
        id: request.id,
        type: 'response',
        ok: true,
        payload: {
          service: 'subtitle-sidecar',
          protocolVersion: 'b8-v1',
          transport: 'stdio',
        },
      };
      break;
    case 'health':
      response = {
        id: request.id,
        type: 'response',
        ok: true,
        payload: {
          service: 'subtitle-sidecar',
          protocolVersion: 'b8-v1',
          transport: 'stdio',
          nodeVersion: process.version,
          sharpVersion: '0.34.4',
          uptimeMs: 5,
          activeSessions: (await readdir(sessionsRoot)).filter((name) => name.endsWith('.json')).length,
          sessionsRoot,
        },
      };
      break;
    case 'start_session': {
      const session = {
        sessionId: 'subtitle_001',
        assetId: request.payload?.assetId,
        state: 'idle',
        progress: 0,
        createdAt: now(),
        updatedAt: now(),
      };
      await writeSession(session);
      response = { id: request.id, type: 'response', ok: true, payload: session };
      break;
    }
    case 'stop_session': {
      const session = await readSession(request.payload.sessionId);
      session.state = 'stopped';
      session.progress = 1;
      session.updatedAt = now();
      await writeSession(session);
      response = { id: request.id, type: 'response', ok: true, payload: session };
      break;
    }
    case 'get_progress': {
      try {
        const session = await readSession(request.payload.sessionId);
        response = {
          id: request.id,
          type: 'response',
          ok: true,
          payload: {
            sessionId: session.sessionId,
            state: session.state,
            progress: session.progress,
            message: 'waiting',
          },
        };
      } catch {
        response = {
          id: request.id,
          type: 'response',
          ok: false,
          error: {
            code: 'NOT_FOUND',
            message: 'session missing',
            retriable: false,
          },
        };
      }
      break;
    }
    default:
      response = {
        id: request.id,
        type: 'response',
        ok: false,
        error: {
          code: 'INVALID_ARGUMENT',
          message: `unsupported type: ${request.type}`,
          retriable: false,
        },
      };
      break;
  }

  process.stdout.write(`${JSON.stringify(response)}\n`);
}
"#
    }

    fn retry_sidecar_script() -> &'static str {
        r#"import { createInterface } from 'node:readline';
import { readFile, writeFile } from 'node:fs/promises';

const attemptFile = process.env.SUBTITLE_SIDECAR_ATTEMPT_FILE;

const currentAttempt = await readFile(attemptFile, 'utf8').catch(() => '0');
if (currentAttempt.trim() === '0') {
  await writeFile(attemptFile, '1');
  process.stderr.write('intentional first boot crash');
  process.exit(2);
}

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of rl) {
  if (!line.trim()) {
    continue;
  }

  const request = JSON.parse(line);
  process.stdout.write(`${JSON.stringify({
    id: request.id,
    type: 'response',
    ok: true,
    payload: {
      service: 'subtitle-sidecar',
      protocolVersion: 'b8-v1',
      transport: 'stdio',
      nodeVersion: process.version,
      sharpVersion: '0.34.4',
      uptimeMs: 1,
      activeSessions: 0,
      sessionsRoot: 'sessions',
    },
  })}\n`);
}
"#
    }
}
