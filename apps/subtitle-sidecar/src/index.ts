import { createInterface } from "node:readline";
import { handleExportSrt } from "./handlers/export-srt.js";
import { handleGetProgress } from "./handlers/get-progress.js";
import { handleHealth } from "./handlers/health.js";
import { handlePing } from "./handlers/ping.js";
import { handleStartSession } from "./handlers/start-session.js";
import { handleStopSession } from "./handlers/stop-session.js";
import {
  type AppError,
  getProgressPayloadSchema,
  sidecarRequestSchema,
  sidecarResponseSchema,
  startSessionPayloadSchema,
  stopSessionPayloadSchema,
} from "./protocol.js";
import { SessionStore } from "./session-store.js";

async function main(): Promise<void> {
  const sessionsRoot = resolveSessionsRoot();
  const sessions = new SessionStore(sessionsRoot);
  await sessions.init();
  process.env.MEDIAPLAYERNEXT_SUBTITLE_SESSIONS_ROOT = sessionsRoot;

  const startedAt = Date.now();
  const readline = createInterface({
    input: process.stdin,
    crlfDelay: Infinity,
  });

  for await (const line of readline) {
    const trimmed = line.trim();
    if (!trimmed) {
      continue;
    }

    let shouldExit = false;
    let response;
    try {
      const request = sidecarRequestSchema.parse(JSON.parse(trimmed));
      switch (request.type) {
        case "ping":
          response = okResponse(request.id, handlePing());
          break;
        case "health":
          response = okResponse(request.id, await handleHealth(startedAt, sessions));
          break;
        case "start_session": {
          const payload = startSessionPayloadSchema.parse(request.payload);
          response = okResponse(
            request.id,
            await handleStartSession(sessions, payload?.assetId),
          );
          break;
        }
        case "stop_session": {
          const payload = stopSessionPayloadSchema.parse(request.payload);
          response = okResponse(
            request.id,
            await handleStopSession(sessions, payload.sessionId),
          );
          break;
        }
        case "get_progress": {
          const payload = getProgressPayloadSchema.parse(request.payload);
          response = okResponse(
            request.id,
            await handleGetProgress(sessions, payload.sessionId),
          );
          break;
        }
        case "export_srt":
          response = errorResponse(request.id, handleExportSrt());
          break;
        case "shutdown":
          response = okResponse(request.id, { accepted: true });
          shouldExit = true;
          break;
      }
    } catch (error) {
      const requestId = safeRequestId(trimmed);
      response = errorResponse(requestId, {
        code: "INVALID_ARGUMENT",
        message: error instanceof Error ? error.message : String(error),
        retriable: false,
      });
    }

    process.stdout.write(`${JSON.stringify(sidecarResponseSchema.parse(response))}\n`);
    if (shouldExit) {
      break;
    }
  }

  readline.close();
}

function okResponse(id: string, payload: unknown) {
  return {
    id,
    type: "response",
    ok: true,
    payload,
  };
}

function errorResponse(
  id: string,
  error: AppError,
) {
  return {
    id,
    type: "response",
    ok: false,
    error,
  };
}

function resolveSessionsRoot(): string {
  const args = process.argv.slice(2);
  const index = args.indexOf("--sessions-root");
  if (index >= 0 && args[index + 1]) {
    return args[index + 1];
  }
  return "./data/cache/subtitle/sessions";
}

function safeRequestId(rawLine: string): string {
  try {
    const parsed = JSON.parse(rawLine) as { id?: string };
    return typeof parsed.id === "string" && parsed.id.length > 0 ? parsed.id : "invalid-request";
  } catch {
    return "invalid-request";
  }
}

void main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
