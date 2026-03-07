import { SessionStore } from "../session-store.js";

export async function handleStartSession(
  sessions: SessionStore,
  assetId?: string,
) {
  return sessions.create(assetId);
}
