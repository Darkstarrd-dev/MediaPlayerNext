import { SessionStore } from "../session-store.js";

export async function handleStopSession(sessions: SessionStore, sessionId: string) {
  return sessions.stop(sessionId);
}
