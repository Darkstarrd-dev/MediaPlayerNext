import type { ScanStats, TaskProgress } from '@mediaplayernext/contracts'
import { useEffect, useRef } from 'react'
import type { MediaRepository } from '../repositories/media-repository'
import { formatTaskStateLabel, getErrorMessage, isActiveTaskProgress, isTaskNotFoundError } from './app-shell-utils'
import type { ImportActivity } from './use-app-shell-import-activities'
import type { SidebarSelection } from './use-app-shell-workspace-selection'

interface UseAppShellScanStateParams {
  repository: MediaRepository
  selectedLibraryId: string | null
  selectedSidebarNodeId: string | null
  selectedMediaSourceId: string | null
  itemsPageIndex: number
  scanSnapshot: TaskProgress | null
  loadLibrarySurface: (options: {
    libraryId: string
    mediaSourceId: string | null
    requestedPageIndex: number
    preferredAssetId?: string | null
  }) => Promise<void>
  refreshSidebarNodes: (
    libraryId: string,
    preferredSidebarNodeId?: string | null,
    preferredMediaSourceId?: string | null,
  ) => Promise<SidebarSelection>
  setScanSnapshot: (value: TaskProgress | null) => void
  setScanStats: (value: ScanStats | null) => void
  setActionError: (value: string | null) => void
  appendImportActivity: (activity: Omit<ImportActivity, 'id' | 'createdAt'>) => string
  updateImportActivity: (
    activityId: string,
    patch: Partial<Omit<ImportActivity, 'id' | 'createdAt'>>,
  ) => void
}

export function useAppShellScanState(params: UseAppShellScanStateParams) {
  const {
    repository,
    selectedLibraryId,
    selectedSidebarNodeId,
    selectedMediaSourceId,
    itemsPageIndex,
    scanSnapshot,
    loadLibrarySurface,
    refreshSidebarNodes,
    setScanSnapshot,
    setScanStats,
    setActionError,
    appendImportActivity,
    updateImportActivity,
  } = params
  const activeScanActivityTaskIdRef = useRef<string | null>(null)
  const activeScanActivityEntryIdRef = useRef<string | null>(null)
  const completedScanSurfaceSyncTaskIdRef = useRef<string | null>(null)

  useEffect(() => {
    if (selectedLibraryId === null || !isActiveTaskProgress(scanSnapshot)) {
      return
    }

    let cancelled = false

    const pollSnapshot = async (): Promise<void> => {
      try {
        const [nextSnapshot, nextStats] = await Promise.all([
          repository.scan.snapshot(selectedLibraryId).catch((error: unknown) => {
            if (isTaskNotFoundError(error)) {
              return null
            }

            throw error
          }),
          repository.scan.stats(selectedLibraryId),
        ])

        if (cancelled) {
          return
        }

        setScanSnapshot(nextSnapshot)
        setScanStats(nextStats)

        if (isActiveTaskProgress(scanSnapshot) && nextSnapshot !== null && !isActiveTaskProgress(nextSnapshot)) {
          const nextSelection = await refreshSidebarNodes(
            selectedLibraryId,
            selectedSidebarNodeId,
            selectedMediaSourceId,
          )
          await loadLibrarySurface({
            libraryId: selectedLibraryId,
            mediaSourceId: nextSelection.selectedMediaSourceId,
            requestedPageIndex: itemsPageIndex,
          })
        }
      } catch (error) {
        if (cancelled) {
          return
        }

        setActionError(`扫描进度轮询失败：${getErrorMessage(error)}`)
      }
    }

    void pollSnapshot()
    const timer = window.setInterval(() => {
      void pollSnapshot()
    }, 1500)

    return () => {
      cancelled = true
      window.clearInterval(timer)
    }
  }, [
    itemsPageIndex,
    loadLibrarySurface,
    refreshSidebarNodes,
    repository,
    scanSnapshot,
    selectedLibraryId,
    selectedSidebarNodeId,
    selectedMediaSourceId,
    setActionError,
    setScanSnapshot,
    setScanStats,
  ])

  useEffect(() => {
    if (scanSnapshot === null || selectedLibraryId === null) {
      completedScanSurfaceSyncTaskIdRef.current = null
      return
    }

    if (isActiveTaskProgress(scanSnapshot)) {
      return
    }

    if (completedScanSurfaceSyncTaskIdRef.current === scanSnapshot.taskId) {
      return
    }

    completedScanSurfaceSyncTaskIdRef.current = scanSnapshot.taskId

    void (async () => {
      const nextSelection = await refreshSidebarNodes(selectedLibraryId, selectedSidebarNodeId, selectedMediaSourceId)
      await loadLibrarySurface({
        libraryId: selectedLibraryId,
        mediaSourceId: nextSelection.selectedMediaSourceId,
        requestedPageIndex: itemsPageIndex,
      })
    })()
  }, [
    itemsPageIndex,
    loadLibrarySurface,
    refreshSidebarNodes,
    scanSnapshot,
    selectedLibraryId,
    selectedSidebarNodeId,
    selectedMediaSourceId,
  ])

  useEffect(() => {
    if (scanSnapshot === null) {
      activeScanActivityTaskIdRef.current = null
      activeScanActivityEntryIdRef.current = null
      return
    }

    const statusMap: Record<TaskProgress['state'], ImportActivity['status']> = {
      queued: 'running',
      running: 'running',
      completed: 'completed',
      failed: 'failed',
      cancelled: 'failed',
    }
    const detail = `${formatTaskStateLabel(scanSnapshot.state)} · ${scanSnapshot.current}/${scanSnapshot.total ?? '?'} · ${scanSnapshot.message ?? '暂无消息'}`

    if (activeScanActivityTaskIdRef.current !== scanSnapshot.taskId || activeScanActivityEntryIdRef.current === null) {
      const nextActivityId = appendImportActivity({
        title: '扫描任务',
        source: '扫描轮询',
        status: statusMap[scanSnapshot.state],
        detail,
      })

      activeScanActivityTaskIdRef.current = scanSnapshot.taskId
      activeScanActivityEntryIdRef.current = nextActivityId
      return
    }

    updateImportActivity(activeScanActivityEntryIdRef.current, {
      status: statusMap[scanSnapshot.state],
      detail,
    })

    if (!isActiveTaskProgress(scanSnapshot)) {
      activeScanActivityTaskIdRef.current = null
      activeScanActivityEntryIdRef.current = null
    }
  }, [appendImportActivity, scanSnapshot, updateImportActivity])
}
