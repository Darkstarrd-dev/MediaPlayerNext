use super::{StdioSubtitleHost, SubtitleSidecarRuntime};
use anyhow::{Context, Result};
use app_core::ports::SubtitleHostPort;
use shared_model::{AssetId, SubtitleSessionId, SubtitleSessionState};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;

#[test]
fn talks_to_mock_sidecar_and_persists_session_flow() -> Result<()> {
    let temp = tempdir()?;
    let script_path = temp.path().join("mock-sidecar.mjs");
    let sessions_root = temp.path().join("sessions");
    fs::write(&script_path, mock_sidecar_script())?;

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

    let ping = host.ping()?;
    let health = host.health()?;
    let session = host.start_session(Some(&AssetId("asset_sidecar_primary".to_string())))?;
    let progress = host.get_progress(&session.session_id)?;
    let stopped = host.stop_session(&session.session_id)?;

    assert_eq!(ping.ping.context("ping payload")?.protocol_version, "b8-v1");
    assert_eq!(health.health.context("health payload")?.active_sessions, 0);
    assert_eq!(
        session.asset_id.context("asset id")?.0,
        "asset_sidecar_primary"
    );
    assert_eq!(progress.state, SubtitleSessionState::Idle);
    assert_eq!(stopped.state, SubtitleSessionState::Stopped);
    Ok(())
}

#[test]
fn retries_health_after_retryable_crash() -> Result<()> {
    let temp = tempdir()?;
    let script_path = temp.path().join("retry-sidecar.mjs");
    let sessions_root = temp.path().join("sessions");
    fs::write(&script_path, retry_sidecar_script())?;

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

    let health = host.health()?;

    assert_eq!(health.restart_count, 1);
    assert!(health
        .last_error
        .context("last error")?
        .contains("exited with status"));
    assert_eq!(
        health.health.context("health payload")?.service,
        "subtitle-sidecar"
    );
    Ok(())
}

#[test]
fn reports_not_found_for_missing_session_progress() -> Result<()> {
    let temp = tempdir()?;
    let script_path = temp.path().join("mock-sidecar.mjs");
    let sessions_root = temp.path().join("sessions");
    fs::write(&script_path, mock_sidecar_script())?;

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
    Ok(())
}

#[test]
fn reports_missing_sidecar_entry_before_spawn() -> Result<()> {
    let temp = tempdir()?;

    let host = StdioSubtitleHost::with_options(
        SubtitleSidecarRuntime {
            node_path: PathBuf::from("node"),
            entry_path: temp.path().join("missing-sidecar.mjs"),
            sessions_root: temp.path().join("sessions"),
            extra_env: Vec::new(),
        },
        Duration::from_millis(150),
        0,
    );

    let error = host.ping().expect_err("missing sidecar entry should fail");

    assert!(error.to_string().contains("subtitle sidecar entry missing"));
    Ok(())
}

#[test]
fn times_out_when_sidecar_does_not_respond() -> Result<()> {
    let temp = tempdir()?;
    let script_path = temp.path().join("timeout-sidecar.mjs");
    let sessions_root = temp.path().join("sessions");
    fs::write(&script_path, timeout_sidecar_script())?;

    let host = StdioSubtitleHost::with_options(
        SubtitleSidecarRuntime {
            node_path: PathBuf::from("node"),
            entry_path: script_path,
            sessions_root,
            extra_env: Vec::new(),
        },
        Duration::from_millis(150),
        0,
    );

    let error = host.ping().expect_err("timed out sidecar should fail");

    assert!(error.to_string().contains("timed out"));
    Ok(())
}

#[test]
fn fails_when_sidecar_returns_malformed_payload() -> Result<()> {
    let temp = tempdir()?;
    let script_path = temp.path().join("malformed-sidecar.mjs");
    let sessions_root = temp.path().join("sessions");
    fs::write(&script_path, malformed_sidecar_script())?;

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
        .ping()
        .expect_err("malformed sidecar payload should fail");

    assert!(error
        .to_string()
        .contains("parse subtitle sidecar response"));
    Ok(())
}

#[test]
fn fails_when_sidecar_omits_success_payload() -> Result<()> {
    let temp = tempdir()?;
    let script_path = temp.path().join("missing-payload-sidecar.mjs");
    let sessions_root = temp.path().join("sessions");
    fs::write(&script_path, missing_payload_sidecar_script())?;

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

    let error = host.ping().expect_err("missing payload should fail");

    assert!(error.to_string().contains("response payload missing"));
    Ok(())
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

fn timeout_sidecar_script() -> &'static str {
    r#"import { createInterface } from 'node:readline';

const rl = createInterface({ input: process.stdin, crlfDelay: Infinity });
for await (const line of rl) {
  if (!line.trim()) {
    continue;
  }

  await new Promise((resolve) => setTimeout(resolve, 5_000));
}
"#
}

fn malformed_sidecar_script() -> &'static str {
    r#"process.stdin.resume();
process.stdout.write('{bad json\n');
"#
}

fn missing_payload_sidecar_script() -> &'static str {
    r#"import { createInterface } from 'node:readline';

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
  })}\n`);
}
"#
}
