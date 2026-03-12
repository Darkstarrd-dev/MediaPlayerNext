import { useCallback } from 'react'
import type { TaskProgress } from '@mediaplayernext/contracts'
import type { MediaRepository } from '../repositories/media-repository'
import { isTaskNotFoundError } from './app-shell-utils'

interface UseAppShellBootstrapScanSnapshotParams {
  repository: MediaRepository
  setScanSnapshot: (value: TaskProgress | null) => void
}

export function useAppShellBootstrapScanSnapshot(params: UseAppShellBootstrapScanSnapshotParams) {
  const { repository, setScanSnapshot } = params

  return useCallback(
    async (libraryId: string): Promise<void> => {
      for (let attempt = 0; attempt < 24; attempt += 1) {
        try {
          const snapshot = await repository.scan.snapshot(libraryId)
          setScanSnapshot(snapshot)
          return
        } catch (error) {
          if (!isTaskNotFoundError(error)) {
            throw error
          }
        }

        await new Promise<void>((resolve) => {
          window.setTimeout(resolve, 250)
        })
      }
    },
    [repository, setScanSnapshot],
  )
}
