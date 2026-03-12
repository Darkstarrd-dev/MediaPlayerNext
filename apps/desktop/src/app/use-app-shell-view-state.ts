import type {
  ItemListEntry,
  LibraryDetail,
  LibrarySummary,
  ScanStats,
  SidebarNodeSummary,
  TaskProgress,
} from '@mediaplayernext/contracts'
import { useCallback, useMemo } from 'react'
import { clampNumber, formatDateTime, formatTaskStateLabel, isActiveTaskProgress } from './app-shell-utils'
import type { ImportActivity } from './use-app-shell-import-activities'
import type { ImportActionKind } from './use-app-shell-import-controller'
import type { AppShellSettingsPage } from './app-shell-settings-types'

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
  importActivities: ImportActivity[]
  selectedMediaSourceId: string | null
  loadLibrarySurface: (options: LoadLibrarySurfaceOptions) => Promise<void>
  setItemsPageIndex: (value: number) => void
  setItemThumbnailUrls: (
    updater: (current: Record<string, string>) => Record<string, string>,
  ) => void
  setSettingsOpen: (value: boolean | ((current: boolean) => boolean)) => void
  setImportTaskPanelOpen: (value: boolean | ((open: boolean) => boolean)) => void
  setSettingsPage: (page: AppShellSettingsPage) => void
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
    importActivities,
    selectedMediaSourceId,
    loadLibrarySurface,
    setItemsPageIndex,
    setItemThumbnailUrls,
    setSettingsOpen,
    setImportTaskPanelOpen,
    setSettingsPage,
  } = params

  const handleGoToItemsPage = useCallback(
    (nextPageIndex: number) => {
      if (selectedLibraryId === null) {
        return
      }

      const normalizedPageIndex = Math.max(1, nextPageIndex)
      setItemsPageIndex(normalizedPageIndex)
      void loadLibrarySurface({
        libraryId: selectedLibraryId,
        mediaSourceId: selectedMediaSourceId,
        requestedPageIndex: normalizedPageIndex,
      })
    },
    [loadLibrarySurface, selectedLibraryId, selectedMediaSourceId, setItemsPageIndex],
  )

  const handleGoPreviousItemsPage = useCallback(() => {
    if (itemsPageIndex <= 1) {
      return
    }

    handleGoToItemsPage(itemsPageIndex - 1)
  }, [handleGoToItemsPage, itemsPageIndex])

  const handleGoNextItemsPage = useCallback(() => {
    if (!itemsHasNextPage) {
      return
    }

    handleGoToItemsPage(itemsPageIndex + 1)
  }, [handleGoToItemsPage, itemsHasNextPage, itemsPageIndex])

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
    setImportTaskPanelOpen((open) => !open)
  }, [setImportTaskPanelOpen, setSettingsOpen])

  const handleOpenSettings = useCallback(() => {
    setImportTaskPanelOpen(false)
    setSettingsPage('ui')
    setSettingsOpen(true)
  }, [setImportTaskPanelOpen, setSettingsOpen, setSettingsPage])

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
  const mainFooterPageLabel = selectedLibraryDetail === null
    ? '0 / 0'
    : itemsHasNextPage
      ? `${itemsPageIndex} / ?`
      : `${itemsPageIndex} / ${itemsPageIndex}`

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
    importActivitiesForPanel,
    handleGoPreviousItemsPage,
    handleGoNextItemsPage,
    handleItemThumbnailError,
    handleToggleImportTaskPanel,
    handleOpenSettings,
  }
}
