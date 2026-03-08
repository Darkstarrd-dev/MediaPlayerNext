import { invoke } from '@tauri-apps/api/core'
import type { SubtitleHost, SubtitleProgress, SubtitleSession } from '@mediaplayernext/contracts'

export interface RuntimeSmokeCheckInput {
  ffmpegPath: string
  ffprobePath: string
  mpvPath: string
}

export interface RuntimeSmokeCheckResult {
  sqliteVersion: string
  ffmpegPath: string
  ffmpegFirstLine: string
  ffprobePath: string
  ffprobeFirstLine: string
  mpvPath: string
  mpvFirstLine: string
}

export async function invokeRuntimeSmokeCheck(
  input: RuntimeSmokeCheckInput,
): Promise<RuntimeSmokeCheckResult> {
  return invoke<RuntimeSmokeCheckResult>('runtime_smoke_check', {
    ffmpegPath: input.ffmpegPath,
    ffprobePath: input.ffprobePath,
    mpvPath: input.mpvPath,
  })
}

export async function invokeSubtitlePing(): Promise<SubtitleHost> {
  return invoke<SubtitleHost>('subtitle_ping_command')
}

export async function invokeSubtitleHealth(): Promise<SubtitleHost> {
  return invoke<SubtitleHost>('subtitle_health_command')
}

export async function invokeSubtitleStartSession(
  assetId?: string,
): Promise<SubtitleSession> {
  return invoke<SubtitleSession>('subtitle_start_session_command', { assetId })
}

export async function invokeSubtitleStopSession(
  sessionId: string,
): Promise<SubtitleSession> {
  return invoke<SubtitleSession>('subtitle_stop_session_command', { sessionId })
}

export async function invokeSubtitleGetProgress(
  sessionId: string,
): Promise<SubtitleProgress> {
  return invoke<SubtitleProgress>('subtitle_get_progress_command', { sessionId })
}
