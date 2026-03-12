import type { MediaRepository } from './media-repository'
import {
  invokeClearDatabase,
  invokeArchiveEntries,
  invokeArchiveEntryDetail,
  invokeArchiveNormalize,
  invokeArchiveNormalizeStatus,
  invokeItemDetail,
  invokeItemsList,
  invokeLibraryAdd,
  invokeLibraryGet,
  invokeLibraryList,
  invokeLibraryNodes,
  invokeLibraryRemove,
  invokePlaybackOpen,
  invokePlaybackSeek,
  invokePlaybackStatus,
  invokeReadRuntimeInfo,
  invokeRuntimeSmokeCheck,
  invokeScanResume,
  invokeScanSnapshot,
  invokeScanStart,
  invokeScanStats,
  invokeSetRuntimeStoragePaths,
  invokeSubtitleGetProgress,
  invokeSubtitleHealth,
  invokeSubtitlePing,
  invokeSubtitleStartSession,
  invokeSubtitleStopSession,
  invokeThumbnailEnsure,
  invokeWorkspaceCursorRead,
  invokeWorkspaceCursorWrite,
} from '../adapters/tauri/commands'
import {
  buildArchiveEntryUrl,
  buildMediaUrl,
  buildThumbnailUrl,
} from '../adapters/tauri/protocols'

function createPlannedOperationError(operation: string): Error {
  return new Error(`${operation} 仍在等待 src-tauri command/channel 接线`)
}

async function plannedOperation<T>(operation: string): Promise<T> {
  throw createPlannedOperationError(operation)
}

export function createTauriMediaRepository(): MediaRepository {
  return {
    diagnostics: {
      checkRuntimeHealth: invokeRuntimeSmokeCheck,
    },
    database: {
      readRuntimeInfo: invokeReadRuntimeInfo,
      setStoragePaths: invokeSetRuntimeStoragePaths,
      clear: invokeClearDatabase,
      readWorkspaceCursor: invokeWorkspaceCursorRead,
      writeWorkspaceCursor: invokeWorkspaceCursorWrite,
    },
    library: {
      list: invokeLibraryList,
      add: async (input) => invokeLibraryAdd(input.rootPath),
      get: invokeLibraryGet,
      remove: invokeLibraryRemove,
      nodes: invokeLibraryNodes,
    },
    scan: {
      start: invokeScanStart,
      resume: invokeScanResume,
      stats: invokeScanStats,
      snapshot: invokeScanSnapshot,
    },
    items: {
      list: invokeItemsList,
      detail: invokeItemDetail,
    },
    archive: {
      entries: invokeArchiveEntries,
      entryDetail: invokeArchiveEntryDetail,
      normalize: invokeArchiveNormalize,
      normalizeStatus: invokeArchiveNormalizeStatus,
    },
    thumbnail: {
      ensure: invokeThumbnailEnsure,
      get: async (thumbnailKey) => buildThumbnailUrl(thumbnailKey),
      progress: async (taskId) => {
        void taskId
        return plannedOperation('thumbnail.progress')
      },
    },
    playback: {
      open: invokePlaybackOpen,
      status: invokePlaybackStatus,
      seek: invokePlaybackSeek,
    },
    subtitle: {
      ping: invokeSubtitlePing,
      health: invokeSubtitleHealth,
      startSession: invokeSubtitleStartSession,
      stopSession: invokeSubtitleStopSession,
      getProgress: invokeSubtitleGetProgress,
    },
    urls: {
      thumbnail: buildThumbnailUrl,
      media: buildMediaUrl,
      archiveEntry: buildArchiveEntryUrl,
    },
  }
}
