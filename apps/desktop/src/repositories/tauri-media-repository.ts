import type { MediaRepository } from './media-repository'
import {
  invokeRuntimeSmokeCheck,
  invokeSubtitleGetProgress,
  invokeSubtitleHealth,
  invokeSubtitlePing,
  invokeSubtitleStartSession,
  invokeSubtitleStopSession,
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
    library: {
      list: async () => plannedOperation('library.list'),
      add: async (input) => {
        void input
        return plannedOperation('library.add')
      },
      get: async (libraryId) => {
        void libraryId
        return plannedOperation('library.get')
      },
      remove: async (libraryId) => {
        void libraryId
        return plannedOperation('library.remove')
      },
    },
    scan: {
      start: async (libraryId) => {
        void libraryId
        return plannedOperation('scan.start')
      },
      resume: async (libraryId) => {
        void libraryId
        return plannedOperation('scan.resume')
      },
      stats: async (libraryId) => {
        void libraryId
        return plannedOperation('scan.stats')
      },
      snapshot: async (libraryId) => {
        void libraryId
        return plannedOperation('scan.snapshot')
      },
    },
    items: {
      list: async (query) => {
        void query
        return plannedOperation('items.list')
      },
      detail: async (assetId) => {
        void assetId
        return plannedOperation('items.detail')
      },
    },
    archive: {
      entries: async (sourceId) => {
        void sourceId
        return plannedOperation('archive.entries')
      },
      entryDetail: async (archiveEntryId) => {
        void archiveEntryId
        return plannedOperation('archive.entryDetail')
      },
      normalize: async (sourceId) => {
        void sourceId
        return plannedOperation('archive.normalize')
      },
      normalizeStatus: async (taskId) => {
        void taskId
        return plannedOperation('archive.normalizeStatus')
      },
    },
    thumbnail: {
      ensure: async (assetId, profile) => {
        void assetId
        void profile
        return plannedOperation('thumbnail.ensure')
      },
      get: async (thumbnailKey) => buildThumbnailUrl(thumbnailKey),
      progress: async (taskId) => {
        void taskId
        return plannedOperation('thumbnail.progress')
      },
    },
    playback: {
      open: async (assetId) => {
        void assetId
        return plannedOperation('playback.open')
      },
      status: async (sessionId) => {
        void sessionId
        return plannedOperation('playback.status')
      },
      seek: async (sessionId, positionMs) => {
        void sessionId
        void positionMs
        return plannedOperation('playback.seek')
      },
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
