import { mkdir, readdir, readFile, writeFile } from "node:fs/promises";
import { join } from "node:path";
import { subtitleProgressSchema, subtitleSessionSchema, type SubtitleProgress, type SubtitleSession } from "./protocol.js";

export class SessionStore {
  constructor(private readonly sessionsRoot: string) {}

  async init(): Promise<void> {
    await mkdir(this.sessionsRoot, { recursive: true });
  }

  async count(): Promise<number> {
    await this.init();
    const entries = await readdir(this.sessionsRoot, { withFileTypes: true });
    return entries.filter((entry) => entry.isFile() && entry.name.endsWith(".json")).length;
  }

  async create(assetId?: string): Promise<SubtitleSession> {
    const timestamp = nowString();
    const session = subtitleSessionSchema.parse({
      sessionId: generateSessionId(),
      assetId,
      state: "idle",
      progress: 0,
      createdAt: timestamp,
      updatedAt: timestamp,
    });
    await this.write(session);
    return session;
  }

  async stop(sessionId: string): Promise<SubtitleSession> {
    const current = await this.get(sessionId);
    const next = subtitleSessionSchema.parse({
      ...current,
      state: "stopped",
      progress: 1,
      updatedAt: nowString(),
    });
    await this.write(next);
    return next;
  }

  async get(sessionId: string): Promise<SubtitleSession> {
    await this.init();
    const raw = await readFile(this.pathFor(sessionId), "utf8");
    return subtitleSessionSchema.parse(JSON.parse(raw));
  }

  async getProgress(sessionId: string): Promise<SubtitleProgress> {
    const session = await this.get(sessionId);
    return subtitleProgressSchema.parse({
      sessionId: session.sessionId,
      state: session.state,
      progress: session.progress,
      message: session.state === "stopped" ? "completed" : "waiting",
    });
  }

  private async write(session: SubtitleSession): Promise<void> {
    await this.init();
    await writeFile(this.pathFor(session.sessionId), JSON.stringify(session, null, 2));
  }

  private pathFor(sessionId: string): string {
    return join(this.sessionsRoot, `${sessionId}.json`);
  }
}

function generateSessionId(): string {
  return `subtitle_${Date.now()}_${Math.random().toString(16).slice(2, 10)}`;
}

function nowString(): string {
  return String(Date.now());
}
