import sharp from "sharp";
import { SessionStore } from "../session-store.js";
import { protocolVersion, subtitleServiceName } from "../protocol.js";

export async function handleHealth(startedAt: number, sessions: SessionStore) {
  return {
    service: subtitleServiceName,
    protocolVersion: protocolVersion,
    transport: "stdio",
    nodeVersion: process.version,
    sharpVersion: sharp.versions.sharp,
    uptimeMs: Date.now() - startedAt,
    activeSessions: await sessions.count(),
    sessionsRoot: process.env.MEDIAPLAYERNEXT_SUBTITLE_SESSIONS_ROOT ?? "",
  };
}
