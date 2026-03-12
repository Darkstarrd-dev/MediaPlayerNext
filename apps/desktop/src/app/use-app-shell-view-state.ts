import type {
  ItemListEntry,
  LibraryDetail,
  LibrarySummary,
  ScanStats,
  SidebarNodeSummary,
  TaskProgress,
} from '@mediaplayernext/contracts'
import type { WheelEvent as ReactWheelEvent } from 'react'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { clampNumber, formatDateTime, formatTaskStateLabel, isActiveTaskProgress } from './app-shell-utils'
import {
  PAGE_WHEEL_DELTA_THRESHOLD_PX,
  PAGE_WHEEL_SETTLE_MS,
} from './thumbnail-grid-enhancements'
import type { ImportActivity } from './use-app-shell-import-activities'
import type { ImportActionKind } from './use-app-shell-import-controller'
import type { AppShellSettingsPage } from './app-shell-settings-types'
import type { AppShellThemeDebugPage } from './app-shell-theme-debug-types'
import type { ItemsPageTransitionState } from './use-app-shell-workspace-state'

const ACTION_LABELS: Record<ImportActionKind, string> = {
  addLibrary: '登记媒体库',
  addAndScan: '登记并扫描',
  dropImport: '拖拽导入',
  pasteImport: '粘贴导入',
}

interface LoadLibrarySurfaceOptions {
  libraryId: string
  mediaSourceId: string | null
  requestedPageIndex: number
  includeWorkspaceSummary?: boolean
}

interface ThumbnailGridLayoutSnapshot {
  columns: number
  rows: number
  pageSize: number
}

interface UseAppShellViewStateParams {
  actionBusy: ImportActionKind | null
  scanSnapshot: TaskProgress | null
  librariesLoading: boolean
  workspaceRefreshing: boolean
  importTaskPanelOpen: boolean
  libraries: LibrarySummary[]
  selectedLibraryId: string | null
  selectedSidebarNodeId: string | null
  sidebarNodes: SidebarNodeSummary[]
  selectedLibraryDetail: LibraryDetail | null
  scanStats: ScanStats | null
  thumbnailGridLayout: ThumbnailGridLayoutSnapshot
  items: ItemListEntry[]
  itemsHasNextPage: boolean
  itemsPageIndex: number
  itemsTargetPageIndex: number
  itemsPageTransitionState: ItemsPageTransitionState
  importActivities: ImportActivity[]
  selectedMediaSourceId: string | null
  loadLibrarySurface: (options: LoadLibrarySurfaceOptions) => Promise<void>
  setItemThumbnailUrls: (
    updater: (current: Record<string, string>) => Record<string, string>,
  ) => void
  setSettingsOpen: (value: boolean | ((current: boolean) => boolean)) => void
  setImportTaskPanelOpen: (value: boolean | ((open: boolean) => boolean)) => void
  setThemeDebugOpen: (value: boolean | ((open: boolean) => boolean)) => void
  setSettingsPage: (page: AppShellSettingsPage) => void
  setThemeDebugPage: (page: AppShellThemeDebugPage) => void
}

export function useAppShellViewState(params: UseAppShellViewStateParams) {
  const {
    actionBusy,
    scanSnapshot,
    librariesLoading,
    workspaceRefreshing,
    importTaskPanelOpen,
    libraries,
    selectedLibraryId,
    selectedSidebarNodeId,
    sidebarNodes,
    selectedLibraryDetail,
    scanStats,
    thumbnailGridLayout,
    items,
    itemsHasNextPage,
    itemsPageIndex,
    itemsTargetPageIndex,
    itemsPageTransitionState,
    importActivities,
    selectedMediaSourceId,
    loadLibrarySurface,
    setItemThumbnailUrls,
    setSettingsOpen,
    setImportTaskPanelOpen,
    setThemeDebugOpen,
    setSettingsPage,
    setThemeDebugPage,
  } = params

  const [wheelPreviewPageIndex, setWheelPreviewPageIndex] = useState<number | null>(null)
  const [wheelPreviewDirection, setWheelPreviewDirection] = useState<'prev' | 'next' | null>(null)
  const wheelDeltaRef = useRef(0)
  const wheelCommitTimerRef = useRef<number | null>(null)

  const clearWheelPreview = useCallback(() => {
    wheelDeltaRef.current = 0
    if (wheelCommitTimerRef.current !== null) {
      window.clearTimeout(wheelCommitTimerRef.current)
      wheelCommitTimerRef.current = null
    }
    setWheelPreviewPageIndex(null)
    setWheelPreviewDirection(null)
  }, [])

  useEffect(
    () => () => {
      if (wheelCommitTimerRef.current !== null) {
        window.clearTimeout(wheelCommitTimerRef.current)
      }
    },
    [],
  )

  useEffect(() => {
    clearWheelPreview()
  }, [clearWheelPreview, itemsPageTransitionState, itemsTargetPageIndex])

  const handleGoToItemsPage = useCallback(
    (nextPageIndex: number) => {
      if (selectedLibraryId === null) {
        return
      }

      const normalizedPageIndex = Math.max(1, nextPageIndex)
      if (normalizedPageIndex === itemsTargetPageIndex && itemsPageTransitionState !== 'idle') {
        return
      }

      void loadLibrarySurface({
        libraryId: selectedLibraryId,
        mediaSourceId: selectedMediaSourceId,
        requestedPageIndex: normalizedPageIndex,
        includeWorkspaceSummary: false,
      })
    },
    [
      itemsPageTransitionState,
      itemsTargetPageIndex,
      loadLibrarySurface,
      selectedLibraryId,
      selectedMediaSourceId,
    ],
  )

  const handleGoPreviousItemsPage = useCallback(() => {
    if (itemsPageIndex <= 1) {
      return
    }

    handleGoToItemsPage(itemsTargetPageIndex - 1)
  }, [handleGoToItemsPage, itemsTargetPageIndex])

  const handleGoNextItemsPage = useCallback(() => {
    if (!itemsHasNextPage) {
      return
    }

    handleGoToItemsPage(itemsTargetPageIndex + 1)
  }, [handleGoToItemsPage, itemsHasNextPage, itemsTargetPageIndex])

  const handleMainGridWheel = useCallback(
    (event: ReactWheelEvent<HTMLDivElement>) => {
      if (selectedLibraryId === null) {
        return
      }

      event.preventDefault()
      if (itemsPageTransitionState !== 'idle') {
        return
      }
      wheelDeltaRef.current += event.deltaY

      if (Math.abs(wheelDeltaRef.current) < PAGE_WHEEL_DELTA_THRESHOLD_PX) {
        return
      }

      const direction = wheelDeltaRef.current > 0 ? 'next' : 'prev'
      wheelDeltaRef.current = 0

      const basePageIndex = wheelPreviewPageIndex ?? itemsTargetPageIndex
      const candidatePageIndex =
        direction === 'next'
          ? itemsHasNextPage
            ? basePageIndex + 1
            : basePageIndex
          : Math.max(1, basePageIndex - 1)

      if (candidatePageIndex === basePageIndex) {
        return
      }

      setWheelPreviewDirection(direction)
      setWheelPreviewPageIndex(candidatePageIndex)

      if (wheelCommitTimerRef.current !== null) {
        window.clearTimeout(wheelCommitTimerRef.current)
      }

      wheelCommitTimerRef.current = window.setTimeout(() => {
        wheelCommitTimerRef.current = null
        const commitPageIndex = candidatePageIndex
        clearWheelPreview()
        if (commitPageIndex !== itemsTargetPageIndex) {
          handleGoToItemsPage(commitPageIndex)
        }
      }, PAGE_WHEEL_SETTLE_MS)
    },
    [
      clearWheelPreview,
      handleGoToItemsPage,
      itemsHasNextPage,
      itemsPageTransitionState,
      itemsTargetPageIndex,
      selectedLibraryId,
      wheelPreviewPageIndex,
    ],
  )

  const handleItemThumbnailError = useCallback(
    (assetId: string) => {
      setItemThumbnailUrls((current) => {
        if (!(assetId in current)) {
          return current
        }

        const next = { ...current }
        delete next[assetId]
        return next
      })
    },
    [setItemThumbnailUrls],
  )

  const handleToggleImportTaskPanel = useCallback(() => {
    setSettingsOpen(false)
    setThemeDebugOpen(false)
    setImportTaskPanelOpen((open) => !open)
  }, [setImportTaskPanelOpen, setSettingsOpen, setThemeDebugOpen])

  const handleOpenSettings = useCallback(() => {
    setImportTaskPanelOpen(false)
    setThemeDebugOpen(false)
    setSettingsPage('ui')
    setSettingsOpen(true)
  }, [setImportTaskPanelOpen, setSettingsOpen, setSettingsPage, setThemeDebugOpen])

  const handleOpenThemeDebug = useCallback(() => {
    setImportTaskPanelOpen(false)
    setSettingsOpen(false)
    setThemeDebugPage('snapshot')
    setThemeDebugOpen(true)
  }, [setImportTaskPanelOpen, setSettingsOpen, setThemeDebugOpen, setThemeDebugPage])

  const importBusy = actionBusy !== null || isActiveTaskProgress(scanSnapshot)
  const logoLoading = importBusy || librariesLoading || workspaceRefreshing
  const logoButtonState = importTaskPanelOpen
    ? 'fg-header-logo-state-open'
    : importBusy
      ? 'fg-header-logo-state-busy'
      : 'fg-header-logo-state-idle'
  const actionPendingLabel = actionBusy !== null
    ? ACTION_LABELS[actionBusy]
    : isActiveTaskProgress(scanSnapshot)
      ? '扫描进行中'
      : null
  const selectedLibrarySummary = libraries.find((library) => library.id === selectedLibraryId) ?? null
  const selectedSidebarNode =
    sidebarNodes.find((node) => node.nodeId === selectedSidebarNodeId) ?? null
  const scanStateLabel = scanSnapshot === null ? '未建立任务' : formatTaskStateLabel(scanSnapshot.state)
  const scanSummary = scanSnapshot === null
    ? '当前媒体库尚未执行扫描。'
    : `${scanSnapshot.current}/${scanSnapshot.total ?? '?'} · ${scanSnapshot.message ?? '暂无消息'}`
  const scanProgressPercent =
    scanSnapshot?.total && scanSnapshot.total > 0
      ? clampNumber((scanSnapshot.current / scanSnapshot.total) * 100, 0, 100)
      : null
  const sidebarFooterText = selectedLibraryDetail === null
    ? '尚未选择媒体库。'
    : `${selectedLibraryDetail.libraryType} · 节点 ${sidebarNodes.length} · 活跃源 ${scanStats?.activeSourceCount ?? 0}`
  const mainFooterPrimary = selectedLibraryDetail === null
    ? '当前未选择媒体库'
    : selectedSidebarNode === null
      ? selectedLibraryDetail.rootPath
      : selectedSidebarNode.treePath.join(' / ')
  const mainFooterSecondary = selectedLibraryDetail === null
    ? '当前作用域暂无条目'
    : `${thumbnailGridLayout.columns} 列 × ${thumbnailGridLayout.rows} 行 · 每页 ${thumbnailGridLayout.pageSize} 项 · 当前页 ${items.length} 项`
  const displayedPageLabel = itemsHasNextPage ? `${itemsPageIndex} / ?` : `${itemsPageIndex} / ${itemsPageIndex}`
  const wheelPreviewLabel =
    wheelPreviewPageIndex === null || wheelPreviewPageIndex === itemsTargetPageIndex
      ? null
      : `${wheelPreviewDirection === 'prev' ? '预览上一页' : '预览下一页'}：${wheelPreviewPageIndex}`
  const transitionLabel =
    itemsPageTransitionState === 'loading-next-page'
      ? `正在切换到第 ${itemsTargetPageIndex} 页`
      : itemsPageTransitionState === 'committing'
        ? `已就绪，提交第 ${itemsTargetPageIndex} 页`
        : wheelPreviewLabel
  const mainFooterPageLabel = selectedLibraryDetail === null
    ? '0 / 0'
    : itemsPageTransitionState !== 'idle' && itemsTargetPageIndex !== itemsPageIndex
      ? `${displayedPageLabel} → ${itemsTargetPageIndex}`
      : displayedPageLabel

  const importActivitiesForPanel = useMemo(
    () =>
      importActivities.map((activity) => ({
        ...activity,
        createdAt: formatDateTime(activity.createdAt),
      })),
    [importActivities],
  )

  return {
    importBusy,
    logoLoading,
    logoButtonState,
    actionPendingLabel,
    selectedLibrarySummary,
    scanStateLabel,
    scanSummary,
    scanProgressPercent,
    sidebarFooterText,
    mainFooterPrimary,
    mainFooterSecondary,
    mainFooterPageLabel,
    mainFooterTransitionLabel: transitionLabel,
    importActivitiesForPanel,
    handleGoPreviousItemsPage,
    handleGoNextItemsPage,
    handleMainGridWheel,
    handleItemThumbnailError,
    handleToggleImportTaskPanel,
    handleOpenSettings,
    handleOpenThemeDebug,
  }
}
