mod paths;
mod wire;

use anyhow::{anyhow, Context, Result};
use app_core::ports::SubtitleHostPort;
use serde::Deserialize;
use serde_json::{json, Value};
use shared_model::{
    build_command_line, emit_external_process_log, AssetId, ExternalProcessLog,
    SubtitleHealthSummary, SubtitleHostSummary, SubtitlePingSummary, SubtitleProgressEvent,
    SubtitleSessionId, SubtitleSessionSummary,
};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};
use tauri::AppHandle;

use self::paths::{
    development_subtitle_entry_path, development_subtitle_sessions_root, ensure_runtime_ready,
    env_path_or_default, env_path_or_else, load_node_path, resolve_tauri_subtitle_entry_path,
    resolve_tauri_subtitle_sessions_root, workspace_root,
};
use self::wire::{
    next_request_id, request_log_context, stderr_excerpt_text, WireError, WireRequest, WireResponse,
};

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

pub fn development_subtitle_host() -> Result<StdioSubtitleHost> {
    let workspace_root = workspace_root();
    let config_path = workspace_root.join("config").join("local.paths.json");
    let node_path = env_path_or_else("MPNEXT_SUBTITLE_NODE_PATH", || load_node_path(&config_path))?;
    let entry_path = env_path_or_default(
        "MPNEXT_SUBTITLE_ENTRY_PATH",
        development_subtitle_entry_path(),
    );
    let sessions_root = env_path_or_default(
        "MPNEXT_SUBTITLE_SESSIONS_ROOT",
        development_subtitle_sessions_root(),
    );

    Ok(StdioSubtitleHost::new(SubtitleSidecarRuntime {
        node_path,
        entry_path,
        sessions_root,
        extra_env: Vec::new(),
    }))
}

pub fn tauri_subtitle_host(app: &AppHandle) -> Result<StdioSubtitleHost> {
    let workspace_root = workspace_root();
    let config_path = workspace_root.join("config").join("local.paths.json");
    let node_path = env_path_or_else("MPNEXT_SUBTITLE_NODE_PATH", || load_node_path(&config_path))?;
    let entry_path = resolve_tauri_subtitle_entry_path(app)?;
    let sessions_root = resolve_tauri_subtitle_sessions_root(app)?;

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
        let request_context = request_log_context(&request.payload);

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

        let command_arguments = vec![
            self.runtime.entry_path.display().to_string(),
            "--sessions-root".to_string(),
            self.runtime.sessions_root.display().to_string(),
        ];
        let command_line = build_command_line(
            &self.runtime.node_path.display().to_string(),
            &command_arguments,
        );

        let mut child = match command.spawn() {
            Ok(child) => child,
            Err(error) => {
                emit_external_process_log(&ExternalProcessLog {
                    event: "external-process".to_string(),
                    phase: "spawn_failed".to_string(),
                    tool: "subtitle-sidecar".to_string(),
                    executable: self.runtime.node_path.display().to_string(),
                    arguments: command_arguments,
                    command_line,
                    exit_code: None,
                    duration_ms: Some(0),
                    ok: false,
                    context: request_context,
                    stderr_excerpt: Some(error.to_string()),
                });
                return Err(error).with_context(|| {
                    format!(
                        "spawn subtitle sidecar: {} {}",
                        self.runtime.node_path.display(),
                        self.runtime.entry_path.display()
                    )
                });
            }
        };

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
                emit_external_process_log(&ExternalProcessLog {
                    event: "external-process".to_string(),
                    phase: "timeout".to_string(),
                    tool: "subtitle-sidecar".to_string(),
                    executable: self.runtime.node_path.display().to_string(),
                    arguments: vec![
                        self.runtime.entry_path.display().to_string(),
                        "--sessions-root".to_string(),
                        self.runtime.sessions_root.display().to_string(),
                    ],
                    command_line: build_command_line(
                        &self.runtime.node_path.display().to_string(),
                        &[
                            self.runtime.entry_path.display().to_string(),
                            "--sessions-root".to_string(),
                            self.runtime.sessions_root.display().to_string(),
                        ],
                    ),
                    exit_code: output.status.code(),
                    duration_ms: Some(started_at.elapsed().as_millis() as u64),
                    ok: false,
                    context: request_log_context(&request.payload),
                    stderr_excerpt: stderr_excerpt_text(&stderr),
                });
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

        emit_external_process_log(&ExternalProcessLog {
            event: "external-process".to_string(),
            phase: "completed".to_string(),
            tool: "subtitle-sidecar".to_string(),
            executable: self.runtime.node_path.display().to_string(),
            arguments: vec![
                self.runtime.entry_path.display().to_string(),
                "--sessions-root".to_string(),
                self.runtime.sessions_root.display().to_string(),
            ],
            command_line: build_command_line(
                &self.runtime.node_path.display().to_string(),
                &[
                    self.runtime.entry_path.display().to_string(),
                    "--sessions-root".to_string(),
                    self.runtime.sessions_root.display().to_string(),
                ],
            ),
            exit_code: output.status.code(),
            duration_ms: Some(started_at.elapsed().as_millis() as u64),
            ok: output.status.success(),
            context: request_log_context(&request.payload),
            stderr_excerpt: stderr_excerpt_text(&stderr),
        });

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

#[cfg(test)]
mod tests;
