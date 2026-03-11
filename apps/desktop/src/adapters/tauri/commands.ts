import { invoke } from '@tauri-apps/api/core'
import type {
  ArchiveEntryDetail,
  ArchiveEntrySummary,
  ArchiveNormalizeResult,
  ItemDetail,
  ItemListEntry,
  ItemsListQuery,
  LibraryDetail,
  LibrarySummary,
  PlaybackSession,
  RuntimeInfo,
  ScanRunResult,
  ScanStats,
  SetRuntimeStoragePathsInput,
  SubtitleHost,
  SubtitleProgress,
  SubtitleSession,
  TaskProgress,
  ThumbnailEnsureResult,
  ThumbnailProfile,
} from '@mediaplayernext/contracts'

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

function withArgAliases<T extends Record<string, unknown>>(input: T): Record<string, unknown> {
  const entries = Object.entries(input).filter(([, value]) => value !== undefined)
  const aliased = entries.flatMap(([key, value]) => {
    const snakeKey = key.replace(/[A-Z]/g, (char) => `_${char.toLowerCase()}`)
    return [
      [key, value],
      [snakeKey, value],
    ]
  })

  return Object.fromEntries(aliased)
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

export async function invokeReadRuntimeInfo(): Promise<RuntimeInfo> {
  return invoke<RuntimeInfo>('read_runtime_info_command')
}

export async function invokeSetRuntimeStoragePaths(
  input: SetRuntimeStoragePathsInput,
): Promise<RuntimeInfo> {
  return invoke<RuntimeInfo>('set_runtime_storage_paths_command', withArgAliases(input))
}

export async function invokeClearDatabase(): Promise<void> {
  return invoke<void>('clear_database_command')
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

export async function invokeLibraryList(): Promise<LibrarySummary[]> {
  return invoke<LibrarySummary[]>('library_list_command')
}

export async function invokeLibraryAdd(rootPath: string): Promise<LibraryDetail> {
  return invoke<LibraryDetail>('library_add_command', withArgAliases({ rootPath }))
}

export async function invokeLibraryGet(libraryId: string): Promise<LibraryDetail> {
  return invoke<LibraryDetail>('library_get_command', withArgAliases({ libraryId }))
}

export async function invokeLibraryRemove(libraryId: string): Promise<void> {
  return invoke<void>('library_remove_command', withArgAliases({ libraryId }))
}

export async function invokeScanStart(libraryId: string): Promise<ScanRunResult> {
  return invoke<ScanRunResult>('scan_start_command', withArgAliases({ libraryId }))
}

export async function invokeScanResume(libraryId: string): Promise<ScanRunResult> {
  return invoke<ScanRunResult>('scan_resume_command', withArgAliases({ libraryId }))
}

export async function invokeScanStats(libraryId: string): Promise<ScanStats> {
  return invoke<ScanStats>('scan_stats_command', withArgAliases({ libraryId }))
}

export async function invokeScanSnapshot(libraryId: string): Promise<TaskProgress> {
  return invoke<TaskProgress>('scan_snapshot_command', withArgAliases({ libraryId }))
}

export async function invokeItemsList(query: ItemsListQuery): Promise<ItemListEntry[]> {
  return invoke<ItemListEntry[]>('items_list_command', withArgAliases(query))
}

export async function invokeItemDetail(assetId: string): Promise<ItemDetail> {
  return invoke<ItemDetail>('item_detail_command', withArgAliases({ assetId }))
}

export async function invokeArchiveEntries(sourceId: string): Promise<ArchiveEntrySummary[]> {
  return invoke<ArchiveEntrySummary[]>('archive_entries_command', withArgAliases({ sourceId }))
}

export async function invokeArchiveEntryDetail(
  archiveEntryId: string,
): Promise<ArchiveEntryDetail> {
  return invoke<ArchiveEntryDetail>('archive_entry_detail_command', withArgAliases({ archiveEntryId }))
}

export async function invokeArchiveNormalize(
  sourceId: string,
): Promise<ArchiveNormalizeResult> {
  return invoke<ArchiveNormalizeResult>('archive_normalize_command', withArgAliases({ sourceId }))
}

export async function invokeArchiveNormalizeStatus(taskId: string): Promise<TaskProgress> {
  return invoke<TaskProgress>('archive_normalize_status_command', withArgAliases({ taskId }))
}

export async function invokeThumbnailEnsure(
  assetId: string,
  profile: ThumbnailProfile,
): Promise<ThumbnailEnsureResult> {
  return invoke<ThumbnailEnsureResult>(
    'thumbnail_ensure_command',
    withArgAliases({ assetId, profile }),
  )
}

export async function invokePlaybackOpen(assetId: string): Promise<PlaybackSession> {
  return invoke<PlaybackSession>('playback_open_command', withArgAliases({ assetId }))
}

export async function invokePlaybackStatus(sessionId: string): Promise<PlaybackSession> {
  return invoke<PlaybackSession>('playback_status_command', withArgAliases({ sessionId }))
}

export async function invokePlaybackSeek(
  sessionId: string,
  positionMs: number,
): Promise<PlaybackSession> {
  return invoke<PlaybackSession>('playback_seek_command', withArgAliases({ sessionId, positionMs }))
}
