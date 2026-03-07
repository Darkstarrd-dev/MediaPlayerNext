import assert from "node:assert/strict";
import { mkdtemp, rm } from "node:fs/promises";
import { tmpdir } from "node:os";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";
import { spawn } from "node:child_process";
import { createInterface } from "node:readline";

type SidecarResponse = {
  id: string;
  type: "response";
  ok: boolean;
  payload?: unknown;
  error?: {
    code: string;
    message: string;
    retriable: boolean;
  };
};

async function main(): Promise<void> {
  const projectRoot = dirname(dirname(fileURLToPath(import.meta.url)));
  const entryPath = join(projectRoot, "dist", "src", "index.js");
  const sessionsRoot = await mkdtemp(join(tmpdir(), "subtitle-sidecar-check-"));
  const child = spawn(process.execPath, [entryPath, "--sessions-root", sessionsRoot], {
    cwd: projectRoot,
    stdio: ["pipe", "pipe", "pipe"],
  });
  const stderr = child.stderr;
  const stdin = child.stdin;

  if (!stderr || !stdin) {
    throw new Error("sidecar stdio is unavailable");
  }

  const readline = createInterface({ input: child.stdout, crlfDelay: Infinity });

  try {
    const ping = await sendRequest(child, stdin, stderr, readline, "req-ping", "ping");
    assert.equal(ping.ok, true);
    assert.equal((ping.payload as { transport: string }).transport, "stdio");

    const health = await sendRequest(child, stdin, stderr, readline, "req-health", "health");
    assert.equal(health.ok, true);
    assert.equal((health.payload as { service: string }).service, "subtitle-sidecar");

    const session = await sendRequest(child, stdin, stderr, readline, "req-start", "start_session", {
      assetId: "asset_sidecar_check",
    });
    assert.equal(session.ok, true);
    const sessionId = (session.payload as { sessionId: string }).sessionId;
    assert.ok(sessionId.startsWith("subtitle_"));

    const progress = await sendRequest(child, stdin, stderr, readline, "req-progress", "get_progress", {
      sessionId,
    });
    assert.equal(progress.ok, true);
    assert.equal((progress.payload as { message: string }).message, "waiting");

    const exportSrt = await sendRequest(child, stdin, stderr, readline, "req-export", "export_srt", {
      sessionId,
    });
    assert.equal(exportSrt.ok, false);
    assert.equal(exportSrt.error?.code, "UNSUPPORTED_FORMAT");

    const stopped = await sendRequest(child, stdin, stderr, readline, "req-stop", "stop_session", {
      sessionId,
    });
    assert.equal(stopped.ok, true);
    assert.equal((stopped.payload as { state: string }).state, "stopped");

    const shutdown = await sendRequest(child, stdin, stderr, readline, "req-shutdown", "shutdown");
    assert.equal(shutdown.ok, true);

    await waitForExit(child);

    console.info(
      JSON.stringify(
        {
          protocol: "b8-v1",
          sessionsRoot,
          sessionId,
          transport: "stdio",
        },
        null,
        2,
      ),
    );
  } finally {
    readline.close();
    child.kill();
    await rm(sessionsRoot, { recursive: true, force: true });
  }
}

function sendRequest(
  child: ReturnType<typeof spawn>,
  stdin: NonNullable<ReturnType<typeof spawn>["stdin"]>,
  stderr: NonNullable<ReturnType<typeof spawn>["stderr"]>,
  readline: ReturnType<typeof createInterface>,
  id: string,
  type: string,
  payload?: Record<string, unknown>,
): Promise<SidecarResponse> {
  return new Promise((resolve, reject) => {
    const cleanup = () => {
      readline.off("line", onLine);
      stderr.off("data", onError);
      child.off("exit", onExit);
    };

    const onLine = (line: string) => {
      try {
        const response = JSON.parse(line) as SidecarResponse;
        if (response.id !== id) {
          return;
        }
        cleanup();
        resolve(response);
      } catch (error) {
        cleanup();
        reject(error);
      }
    };

    const onError = (buffer: Buffer) => {
      cleanup();
      reject(new Error(buffer.toString("utf8")));
    };

    const onExit = (code: number | null) => {
      cleanup();
      reject(new Error(`sidecar exited early with code ${code}`));
    };

    readline.on("line", onLine);
    stderr.on("data", onError);
    child.once("exit", onExit);
    stdin.write(`${JSON.stringify({ id, type, payload })}\n`);
  });
}

function waitForExit(child: ReturnType<typeof spawn>): Promise<void> {
  return new Promise((resolve, reject) => {
    child.once("exit", (code) => {
      if (code === 0) {
        resolve();
        return;
      }
      reject(new Error(`sidecar exited with code ${code}`));
    });
  });
}

void main().catch((error: unknown) => {
  console.error(error);
  process.exitCode = 1;
});
