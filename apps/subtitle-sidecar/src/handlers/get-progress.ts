import { SessionStore } from "../session-store.js";

export async function handleGetProgress(sessions: SessionStore, sessionId: string) {
  return sessions.getProgress(sessionId);
}
