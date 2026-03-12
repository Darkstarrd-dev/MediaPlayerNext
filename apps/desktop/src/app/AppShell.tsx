import type {
  ItemDetail,
  ItemListEntry,
  LibraryDetail,
  LibrarySummary,
  RuntimeInfo,
  ScanStats,
  SidebarNodeSummary,
  TaskProgress,
} from '@mediaplayernext/contracts'
import type { PointerEvent as ReactPointerEvent } from 'react'
import { useCallback, useEffect, useRef, useState } from 'react'
import { ImportTaskPanel } from './ImportTaskPanel'
import { SettingsIcon } from './SettingsIcon'
import {
  clampNumber,
  formatDateTime,
  formatTaskStateLabel,
  getErrorMessage,
  isActiveTaskProgress,
  resolveItemDisplayLabel,
  resolveItemLocation,
  resolvePathLeaf,
} from './app-shell-utils'
import {
  THUMBNAIL_ZOOM_LEVELS,
  toThumbnailZoomLevel,
} from './thumbnail-grid-layout'
import { useAppShellLayout, type DragTarget } from './use-app-shell-layout'
import { useAppShellImportActivities } from './use-app-shell-import-activities'
import { useAppShellImportController } from './use-app-shell-import-controller'
import { useAppShellScanState } from './use-app-shell-scan-state'
import { useAppShellWorkspaceCursor } from './use-app-shell-workspace-cursor'
import { useAppShellWorkspaceData } from './use-app-shell-workspace-data'
import { useAppShellWorkspaceSelection } from './use-app-shell-workspace-selection'
import { useAppShellItemData } from './use-app-shell-item-data'
import { useMediaRepository } from './use-media-repository'

const ACTION_LABELS = {
  addLibrary: '登记媒体库',
  addAndScan: '登记并扫描',
  dropImport: '拖拽导入',
  pasteImport: '粘贴导入',
} as const

const DATABASE_ACTION_LABELS = {
  pickDatabaseDir: '选择 SQL 目录',
  pickThumbnailDir: '选择缩略图目录',
  clearDatabase: '清除数据库',
} as const

type DatabaseActionKind = keyof typeof DATABASE_ACTION_LABELS
type SettingsPage = 'ui' | 'database'

export function AppShell() {
  const repository = useMediaRepository()
  const libraryLoadRequestIdRef = useRef(0)
  const itemDetailRequestIdRef = useRef(0)
  const [importTaskPanelOpen, setImportTaskPanelOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [settingsPage, setSettingsPage] = useState<SettingsPage>('ui')
  const [clearDatabaseDialogOpen, setClearDatabaseDialogOpen] = useState(false)
  const {
    dragState,
    beginSplitterDrag,
    itemGridStyle,
    layoutPreview,
    setMainGridElement,
    setSettingsBackdropOpacity,
    setLayoutGapScaleCoeff,
    setPaneInnerGapScaleCoeff,
    setPaneStackGapScaleCoeff,
    setSplitterWidthScaleCoeff,
    settingsBackdropOpacity,
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    splitterWidthScaleCoeff,
    thumbnailGridLayout,
    thumbnailZoomLevel,
    setThumbnailZoomLevel,
    workspaceStyle,
  } = useAppShellLayout()

  const [libraries, setLibraries] = useState<LibrarySummary[]>([])
  const [librariesLoading, setLibrariesLoading] = useState(false)
  const [selectedLibraryId, setSelectedLibraryId] = useState<string | null>(null)
  const [sidebarNodes, setSidebarNodes] = useState<SidebarNodeSummary[]>([])
  const [sidebarNodesLoading, setSidebarNodesLoading] = useState(false)
  const [selectedSidebarNodeId, setSelectedSidebarNodeId] = useState<string | null>(null)
  const [selectedMediaSourceId, setSelectedMediaSourceId] = useState<string | null>(null)
  const [selectedLibraryDetail, setSelectedLibraryDetail] = useState<LibraryDetail | null>(null)
  const [scanStats, setScanStats] = useState<ScanStats | null>(null)
  const [scanSnapshot, setScanSnapshot] = useState<TaskProgress | null>(null)
  const [workspaceRefreshing, setWorkspaceRefreshing] = useState(false)
  const [workspaceHydrated, setWorkspaceHydrated] = useState(false)
  const [workspaceError, setWorkspaceError] = useState<string | null>(null)
  const [items, setItems] = useState<ItemListEntry[]>([])
  const [itemsPageIndex, setItemsPageIndex] = useState(1)
  const [itemsHasNextPage, setItemsHasNextPage] = useState(false)
  const [selectedAssetId, setSelectedAssetId] = useState<string | null>(null)
  const [selectedItemDetail, setSelectedItemDetail] = useState<ItemDetail | null>(null)
  const [itemDetailLoading, setItemDetailLoading] = useState(false)
  const [itemDetailError, setItemDetailError] = useState<string | null>(null)
  const [itemThumbnailUrls, setItemThumbnailUrls] = useState<Record<string, string>>({})

  const [runtimeInfo, setRuntimeInfo] = useState<RuntimeInfo | null>(null)
  const [runtimeInfoLoading, setRuntimeInfoLoading] = useState(false)
  const [runtimeInfoError, setRuntimeInfoError] = useState<string | null>(null)
  const [databaseActionBusy, setDatabaseActionBusy] = useState<DatabaseActionKind | null>(null)
  const [databaseActionMessage, setDatabaseActionMessage] = useState<string | null>(null)
  const [databaseActionError, setDatabaseActionError] = useState<string | null>(null)

  const clearWorkspaceData = useCallback(() => {
    libraryLoadRequestIdRef.current += 1
    itemDetailRequestIdRef.current += 1
    setSelectedLibraryDetail(null)
    setSidebarNodes([])
    setSelectedSidebarNodeId(null)
    setSelectedMediaSourceId(null)
    setSidebarNodesLoading(false)
    setScanStats(null)
    setScanSnapshot(null)
    setWorkspaceError(null)
    setWorkspaceRefreshing(false)
    setWorkspaceHydrated(false)
    setItems([])
    setItemsPageIndex(1)
    setItemsHasNextPage(false)
    setSelectedAssetId(null)
    setSelectedItemDetail(null)
    setItemDetailError(null)
    setItemDetailLoading(false)
    setItemThumbnailUrls({})
  }, [])

  const { importActivities, appendImportActivity, updateImportActivity } = useAppShellImportActivities()

  const { refreshLibraries, refreshSidebarNodes } = useAppShellWorkspaceSelection({
    repository,
    clearWorkspaceData,
    setLibraries,
    setLibrariesLoading,
    setSelectedLibraryId,
    setSidebarNodes,
    setSidebarNodesLoading,
    setSelectedSidebarNodeId,
    setSelectedMediaSourceId,
    setWorkspaceError,
  })

  const { bootstrapScanSnapshot, loadLibrarySurface, refreshWorkspace } =
    useAppShellWorkspaceData({
      repository,
      thumbnailPageSize: thumbnailGridLayout.pageSize,
      libraryLoadRequestIdRef,
      selectedLibraryId,
      selectedMediaSourceId,
      itemsPageIndex,
      workspaceHydrated,
      clearWorkspaceData,
      setSelectedLibraryDetail,
      setScanStats,
      setScanSnapshot,
      setWorkspaceRefreshing,
      setWorkspaceHydrated,
      setWorkspaceError,
      setItems,
      setItemsPageIndex,
      setItemsHasNextPage,
      setSelectedAssetId,
      setItemThumbnailUrls,
      refreshLibraries,
      refreshSidebarNodes,
    })

  useAppShellWorkspaceCursor({
    cursorStore: repository.database,
    refreshWorkspace,
    workspaceHydrated,
    selectedLibraryId,
    selectedSidebarNodeId,
    selectedMediaSourceId,
    itemsPageIndex,
    selectedAssetId,
  })

  useEffect(() => {
    if (!settingsOpen && !importTaskPanelOpen) {
      return
    }

    const handleEscape = (event: KeyboardEvent): void => {
      if (event.key === 'Escape') {
        if (settingsOpen) {
          setSettingsOpen(false)
          return
        }

        setImportTaskPanelOpen(false)
      }
    }

    window.addEventListener('keydown', handleEscape)

    return () => {
      window.removeEventListener('keydown', handleEscape)
    }
  }, [importTaskPanelOpen, settingsOpen])

  useAppShellItemData({
    repository,
    itemDetailRequestIdRef,
    selectedAssetId,
    items,
    setSelectedItemDetail,
    setItemDetailError,
    setItemDetailLoading,
    setItemThumbnailUrls,
  })

  function handleSplitterPointerDown(target: DragTarget) {
    return (event: ReactPointerEvent<HTMLDivElement>): void => {
      event.preventDefault()
      beginSplitterDrag(target, event.clientX)
    }
  }

  const handleLibrarySelect = useCallback(
    (libraryId: string) => {
      void refreshWorkspace({
        preferredLibraryId: libraryId,
        preferredPageIndex: 1,
      })
    },
    [refreshWorkspace],
  )

  const handleSidebarNodeSelect = useCallback(
    (nodeId: string) => {
      if (selectedLibraryId === null) {
        return
      }

      const matchedNode = sidebarNodes.find((node) => node.nodeId === nodeId) ?? null
      if (matchedNode === null) {
        return
      }

      if (matchedNode.nodeType !== 'media_source') {
        setSelectedSidebarNodeId(nodeId)
        return
      }

      const nextMediaSourceId = matchedNode.mediaSourceId ?? null

      setSelectedSidebarNodeId(nodeId)
      setSelectedMediaSourceId(nextMediaSourceId)
      setItemsPageIndex(1)
      void loadLibrarySurface({
        libraryId: selectedLibraryId,
        mediaSourceId: nextMediaSourceId,
        requestedPageIndex: 1,
      })
    },
    [loadLibrarySurface, selectedLibraryId, sidebarNodes],
  )

  const {
    actionBusy,
    actionError,
    actionMessage,
    dropImportActive,
    handleAddLibrary,
    handlePickDirectory,
    importRootPath,
    pickSingleDirectory,
    setActionError,
    setImportRootPath,
  } = useAppShellImportController({
    repository,
    selectedLibraryId,
    refreshWorkspace,
    bootstrapScanSnapshot,
    appendImportActivity,
    updateImportActivity,
    setImportTaskPanelOpen,
  })

  useAppShellScanState({
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
  })

  const refreshRuntimeInfo = useCallback(async () => {
    setRuntimeInfoLoading(true)
    setRuntimeInfoError(null)

    try {
      const nextRuntimeInfo = await repository.database.readRuntimeInfo()
      setRuntimeInfo(nextRuntimeInfo)
    } catch (error) {
      setRuntimeInfo(null)
      setRuntimeInfoError(getErrorMessage(error))
    } finally {
      setRuntimeInfoLoading(false)
    }
  }, [repository])

  useEffect(() => {
    if (!settingsOpen || settingsPage !== 'database') {
      return
    }

    void refreshRuntimeInfo()
  }, [refreshRuntimeInfo, settingsOpen, settingsPage])

  useEffect(() => {
    if (settingsOpen) {
      return
    }

    setClearDatabaseDialogOpen(false)
  }, [settingsOpen])

  useEffect(() => {
    if (!clearDatabaseDialogOpen) {
      return
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key !== 'Escape' || databaseActionBusy === 'clearDatabase') {
        return
      }

      setClearDatabaseDialogOpen(false)
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [clearDatabaseDialogOpen, databaseActionBusy])

  const handlePickDatabaseDirectory = useCallback(async () => {
    setRuntimeInfoError(null)
    setDatabaseActionError(null)

    try {
      const nextPath = await pickSingleDirectory('选择 SQL 目录')
      if (nextPath === null) {
        return
      }

      setDatabaseActionBusy('pickDatabaseDir')
      setDatabaseActionMessage(null)
      const nextRuntimeInfo = await repository.database.setStoragePaths({ databaseDir: nextPath })
      setRuntimeInfo(nextRuntimeInfo)
      setDatabaseActionMessage('SQL 目录已保存。')
    } catch (error) {
      setDatabaseActionError(getErrorMessage(error))
    } finally {
      setDatabaseActionBusy(null)
    }
  }, [pickSingleDirectory, repository])

  const handlePickThumbnailDirectory = useCallback(async () => {
    setRuntimeInfoError(null)
    setDatabaseActionError(null)

    try {
      const nextPath = await pickSingleDirectory('选择缩略图目录')
      if (nextPath === null) {
        return
      }

      setDatabaseActionBusy('pickThumbnailDir')
      setDatabaseActionMessage(null)
      const nextRuntimeInfo = await repository.database.setStoragePaths({
        thumbnailCacheDir: nextPath,
      })
      setRuntimeInfo(nextRuntimeInfo)
      setDatabaseActionMessage('缩略图目录已保存。')
    } catch (error) {
      setDatabaseActionError(getErrorMessage(error))
    } finally {
      setDatabaseActionBusy(null)
    }
  }, [pickSingleDirectory, repository])

  const handleRequestClearDatabase = () => {
    if (databaseActionBusy !== null) {
      return
    }

    setDatabaseActionError(null)
    setDatabaseActionMessage(null)
    setClearDatabaseDialogOpen(true)
  }

  const handleCloseClearDatabaseDialog = () => {
    if (databaseActionBusy === 'clearDatabase') {
      return
    }

    setClearDatabaseDialogOpen(false)
  }

  const handleConfirmClearDatabase = useCallback(async () => {
    setRuntimeInfoError(null)
    setDatabaseActionBusy('clearDatabase')
    setDatabaseActionError(null)
    setDatabaseActionMessage(null)

    try {
      await repository.database.clear()
      setDatabaseActionMessage('已清除数据库，正在重新加载。')
      setClearDatabaseDialogOpen(false)
      window.location.reload()
    } catch (error) {
      setDatabaseActionError(getErrorMessage(error))
    } finally {
      setDatabaseActionBusy(null)
    }
  }, [repository])

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
    [loadLibrarySurface, selectedLibraryId, selectedMediaSourceId],
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
  const metadataCaption = selectedItemDetail
    ? resolveItemLocation(selectedItemDetail)
    : selectedLibraryDetail?.rootPath ?? 'Caption / 摘要预留，后续根据选中项显示真实内容。'
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
  const importActivitiesForPanel = importActivities.map((activity) => ({
    ...activity,
    createdAt: formatDateTime(activity.createdAt),
  }))
  const databasePendingLabel = databaseActionBusy === null ? null : DATABASE_ACTION_LABELS[databaseActionBusy]
  const runtimeInfoDatabasePath = runtimeInfoLoading
    ? '正在读取当前 SQL 路径...'
    : runtimeInfo?.databasePath ?? '未读取'
  const runtimeInfoThumbnailCachePath = runtimeInfoLoading
    ? '正在读取当前缩略图目录...'
    : runtimeInfo?.thumbnailCachePath ?? '未读取'

  return (
    <main className="app-shell" data-slot="bg-app-root">
      <div className="app-background-layer" aria-hidden="true" />
      {dropImportActive ? (
        <div className="app-drag-import-overlay" role="status" aria-live="polite">
          <div className="app-drag-import-card">
            <span className="section-kicker">Import</span>
            <strong>释放即可登记并扫描</strong>
            <p>拖入本地目录后，会直接通过当前导入链路登记媒体库并刷新 Sidebar、Main、Metadata。</p>
          </div>
        </div>
      ) : null}

      <div className="app-chrome">
        <header className="app-frame app-header app-header-root" data-slot="fg-header-root">
          <div className="app-header-frame">
            <div className="header-left">
              <button
                className="mpx-btn header-logo-btn"
                type="button"
                data-testid="header-logo-trigger"
                aria-haspopup="dialog"
                aria-expanded={importTaskPanelOpen}
                aria-controls="import-task-panel"
                data-slot="fg-header-logo"
                data-slot-state={logoButtonState}
                onClick={() => {
                  setSettingsOpen(false)
                  setImportTaskPanelOpen((open) => !open)
                }}
              >
                <span className="header-logo-mark" aria-hidden="true">
                  M
                </span>
                <span className="header-logo-label">{logoLoading ? 'Loading' : 'MediaPlayerNext'}</span>
              </button>
            </div>

            <div className="header-right">
              <button
                className="mpx-btn header-settings-trigger"
                type="button"
                data-testid="header-settings-trigger"
                aria-haspopup="dialog"
                aria-expanded={settingsOpen}
                onClick={() => {
                  setImportTaskPanelOpen(false)
                  setSettingsPage('ui')
                  setSettingsOpen(true)
                }}
              >
                <SettingsIcon className="settings-trigger-icon" />
                <span className="settings-trigger-label">设置</span>
              </button>
            </div>
          </div>
        </header>

        <div
          className="app-workspace"
          style={workspaceStyle}
          data-testid="workspace-root"
          data-active-sidebar-node-id={selectedSidebarNodeId ?? ''}
          data-active-media-source-id={selectedMediaSourceId ?? ''}
        >
          <aside className="app-frame app-sidebar-root" data-slot="fg-sidebar-root">
            <section className="workspace-pane sidebar-frame">
              <header className="workspace-pane-header sidebar-header" data-slot="fg-sidebar-header">
                <div className="pane-title-stack sidebar-title-stack">
                  <h2>直属节点</h2>
                  <span className="sidebar-selected-library">
                    {selectedLibrarySummary === null
                      ? '未选择媒体库'
                      : resolvePathLeaf(selectedLibrarySummary.rootPath)}
                  </span>
                </div>

                <label className="sidebar-library-select-wrap" aria-label="切换媒体库">
                  <span>媒体库</span>
                  <select
                    className="sidebar-library-select"
                    value={selectedLibraryId ?? ''}
                    onChange={(event) => {
                      const nextLibraryId = event.target.value.trim()
                      if (nextLibraryId.length === 0) {
                        return
                      }

                      handleLibrarySelect(nextLibraryId)
                    }}
                    disabled={librariesLoading || libraries.length === 0}
                  >
                    {libraries.map((library) => (
                      <option key={library.id} value={library.id}>
                        {resolvePathLeaf(library.rootPath)}
                      </option>
                    ))}
                  </select>
                </label>
              </header>

              <div className="workspace-pane-main sidebar-main-shell" data-slot="fg-sidebar-main">
                {selectedLibraryId === null ? (
                  <section className="workspace-stage compact">
                    <span className="workspace-label">Sidebar</span>
                    <strong>等待导入媒体库</strong>
                    <p>导入后将显示当前媒体库的直属节点列表。</p>
                  </section>
                ) : sidebarNodesLoading && sidebarNodes.length === 0 ? (
                  <section className="workspace-stage compact">
                    <span className="workspace-label">Sidebar</span>
                    <strong>正在读取直属节点</strong>
                    <p>当前媒体库的首层节点正在同步，请稍候。</p>
                  </section>
                ) : sidebarNodes.length === 0 ? (
                  <section className="workspace-stage compact">
                    <span className="workspace-label">Sidebar</span>
                    <strong>当前媒体库暂无直属节点</strong>
                    <p>可以先执行扫描，或继续导入新的本地路径。</p>
                  </section>
                ) : (
                  <div className="sidebar-tree" role="tree" aria-label="直属媒体节点列表">
                    {sidebarNodes.map((node) => {
                      const isActive = node.nodeId === selectedSidebarNodeId

                      return (
                        <button
                          key={node.nodeId}
                          className={`sidebar-tree-node ${isActive ? 'is-active' : ''}`}
                          type="button"
                          role="treeitem"
                          aria-selected={isActive}
                          data-testid="sidebar-tree-node"
                          data-node-id={node.nodeId}
                          data-node-type={node.nodeType}
                          data-media-source-id={node.mediaSourceId ?? ''}
                          data-has-direct-media-child={node.hasDirectMediaChild ? '1' : '0'}
                          style={{ paddingInlineStart: `${14 + node.depth * 14}px` }}
                          onClick={() => handleSidebarNodeSelect(node.nodeId)}
                        >
                          <span className="sidebar-tree-node-rail" aria-hidden="true" />
                          <span className="sidebar-tree-node-dot" aria-hidden="true" />
                          <span className="sidebar-tree-node-copy">
                            <strong>{node.label}</strong>
                            <span>
                              {node.nodeType === 'media_source'
                                ? `${node.sourceType ?? 'source'} · ${node.itemCount ?? 0} 项`
                                : node.hasDirectMediaChild
                                  ? '包含直属媒体节点'
                                  : '路径节点'}
                            </span>
                          </span>
                        </button>
                      )
                    })}
                  </div>
                )}
              </div>

              <footer className="workspace-pane-footer sidebar-footer" data-slot="fg-sidebar-footer">
                <span>{sidebarFooterText}</span>
              </footer>
            </section>
          </aside>

          <div
            className={`workspace-splitter ${dragState?.target === 'left' ? 'is-dragging' : ''}`}
            role="separator"
            aria-orientation="vertical"
            aria-label="调整 Sidebar 与 Main 宽度"
            onPointerDown={handleSplitterPointerDown('left')}
          />

          <section className="app-frame app-main-root" data-slot="fg-main-root">
            <section className="workspace-pane main-pane-frame">
              <header className="workspace-pane-header main-header" data-slot="fg-main-header">
                {selectedLibraryDetail === null ? (
                  <div />
                ) : (
                  <h2 className="pane-title-single">{resolvePathLeaf(selectedLibraryDetail.rootPath)}</h2>
                )}

                <label className="main-zoom-control" aria-label="缩略图缩放级别">
                  <span>缩放</span>
                  <select
                    className="main-zoom-select"
                    value={thumbnailZoomLevel}
                    onChange={(event) => setThumbnailZoomLevel(toThumbnailZoomLevel(Number(event.target.value)))}
                    disabled={selectedLibraryId === null}
                  >
                    {THUMBNAIL_ZOOM_LEVELS.map((level) => (
                      <option key={level} value={level}>
                        {level}
                      </option>
                    ))}
                  </select>
                </label>
              </header>

              <div
                ref={setMainGridElement}
                className="workspace-pane-main main-pane-main"
                data-slot="fg-main-main"
              >
                {selectedLibraryId === null ? (
                  <div className="workspace-stage">
                    <span className="workspace-label">主工作区</span>
                    <strong>等待导入媒体库</strong>
                    <p>完成路径登记后，这里会按当前媒体库快照显示条目预览，并与 Sidebar/Metadata 同步刷新。</p>
                  </div>
                ) : !workspaceHydrated && workspaceRefreshing ? (
                  <div className="workspace-stage">
                    <span className="workspace-label">主工作区</span>
                    <strong>正在刷新快照</strong>
                    <p>正在读取当前媒体库的条目列表、扫描状态与详情摘要。</p>
                  </div>
                ) : workspaceError !== null ? (
                  <div className="error-text">{workspaceError}</div>
                ) : items.length === 0 ? (
                  <div className="workspace-stage">
                    <span className="workspace-label">Items</span>
                    <strong>当前媒体库暂无条目</strong>
                    <p>可以直接开始扫描，或回到导入面板登记新的本地路径。</p>
                  </div>
                ) : (
                  <div className="item-grid" style={itemGridStyle}>
                    {items.map((item) => {
                      const isActive = item.assetId === selectedAssetId
                      const thumbnailUrl = itemThumbnailUrls[item.assetId] ?? null

                      return (
                        <button
                          key={item.assetId}
                          className={`workspace-card-button item-card-button ${isActive ? 'is-active' : ''}`}
                          type="button"
                          aria-pressed={isActive}
                          onClick={() => setSelectedAssetId(item.assetId)}
                        >
                          {thumbnailUrl === null ? (
                            <div className="item-card-thumbnail item-card-thumbnail-placeholder">
                              <span>{item.sourceKind === 'archive_entry' ? 'Archive' : 'Media'}</span>
                            </div>
                          ) : (
                            <img
                              className="item-card-thumbnail"
                              src={thumbnailUrl}
                              alt=""
                              loading="lazy"
                              onError={() => {
                                setItemThumbnailUrls((current) => {
                                  if (!(item.assetId in current)) {
                                    return current
                                  }

                                  const next = { ...current }
                                  delete next[item.assetId]
                                  return next
                                })
                              }}
                            />
                          )}
                        </button>
                      )
                    })}
                  </div>
                )}
              </div>

              <footer className="workspace-pane-footer main-footer" data-slot="fg-main-footer">
                <div className="main-footer-meta" data-slot="fg-main-footer-meta">
                  <span>{mainFooterPrimary}</span>
                  <span>{mainFooterSecondary}</span>
                </div>

                <div className="main-footer-pagination" data-slot="fg-main-footer-pagination">
                  <button
                    className="mpx-btn pane-pagination-btn"
                    type="button"
                    disabled={selectedLibraryId === null || itemsPageIndex <= 1}
                    onClick={handleGoPreviousItemsPage}
                  >
                    Prev
                  </button>
                  <span>{mainFooterPageLabel}</span>
                  <button
                    className="mpx-btn pane-pagination-btn"
                    type="button"
                    disabled={selectedLibraryId === null || !itemsHasNextPage}
                    onClick={handleGoNextItemsPage}
                  >
                    Next
                  </button>
                </div>
              </footer>
            </section>
          </section>

          <div
            className={`workspace-splitter ${dragState?.target === 'right' ? 'is-dragging' : ''}`}
            role="separator"
            aria-orientation="vertical"
            aria-label="调整 Main 与 Metadata 宽度"
            onPointerDown={handleSplitterPointerDown('right')}
          />

          <aside className="app-frame app-meta-root" data-slot="fg-meta-root">
            <section className="workspace-pane metadata-frame">
              <header className="workspace-pane-header metadata-header" data-slot="fg-meta-header">
                {selectedLibraryId === null ? (
                  <div />
                ) : (
                  <div className="pane-title-stack metadata-header-title">
                    <h2>{resolveItemDisplayLabel(selectedItemDetail) || resolvePathLeaf(selectedLibraryDetail?.rootPath ?? '')}</h2>
                    {selectedItemDetail === null ? null : (
                      <p className="pane-title-caption">{resolveItemLocation(selectedItemDetail)}</p>
                    )}
                  </div>
                )}

                <div className="workspace-pane-actions metadata-header-g3">
                  <span className="status-pill" data-state={scanSnapshot?.state ?? 'idle'}>
                    {importBusy && actionPendingLabel !== null ? `${actionPendingLabel} / ${scanStateLabel}` : scanStateLabel}
                  </span>
                </div>
              </header>

              <div className="workspace-pane-main metadata-main" data-slot="fg-meta-main">
                {selectedLibraryId === null ? (
                  <div className="workspace-stage compact">
                    <span className="workspace-label">详情区</span>
                    <strong>等待可用上下文</strong>
                    <p>选择媒体库后，这里会显示当前库、扫描状态和当前条目的最小摘要。</p>
                  </div>
                ) : (
                  <div className="metadata-stack">
                    {workspaceError === null ? null : <div className="error-text">{workspaceError}</div>}

                    <section className="pane-section">
                      <div className="panel-heading pane-section-heading">
                        <div>
                          <span className="workspace-label">Library</span>
                          <h3>当前媒体库</h3>
                        </div>
                      </div>

                      <dl className="kv-list">
                        <div>
                          <dt>Root Path</dt>
                          <dd>{selectedLibraryDetail?.rootPath ?? '未读取'}</dd>
                        </div>
                        <div>
                          <dt>Library Type</dt>
                          <dd>{selectedLibraryDetail?.libraryType ?? '未读取'}</dd>
                        </div>
                        <div>
                          <dt>Scan Mode</dt>
                          <dd>{selectedLibraryDetail?.scanMode ?? '未读取'}</dd>
                        </div>
                        <div>
                          <dt>Updated At</dt>
                          <dd>
                            {selectedLibraryDetail === null ? '未读取' : formatDateTime(selectedLibraryDetail.updatedAt)}
                          </dd>
                        </div>
                      </dl>
                    </section>

                    <section className="pane-section">
                      <div className="panel-heading pane-section-heading">
                        <div>
                          <span className="workspace-label">Scan Snapshot</span>
                          <h3>扫描摘要</h3>
                        </div>
                      </div>

                      <dl className="kv-list">
                        <div>
                          <dt>Task State</dt>
                          <dd>{scanStateLabel}</dd>
                        </div>
                        <div>
                          <dt>Progress</dt>
                          <dd>{scanSummary}</dd>
                        </div>
                        <div>
                          <dt>Active Sources</dt>
                          <dd>{scanStats?.activeSourceCount ?? 0}</dd>
                        </div>
                        <div>
                          <dt>Missing Sources</dt>
                          <dd>{scanStats?.missingSourceCount ?? 0}</dd>
                        </div>
                      </dl>
                    </section>

                    <section className="pane-section">
                      <div className="panel-heading pane-section-heading">
                        <div>
                          <span className="workspace-label">Current Item</span>
                          <h3>当前条目</h3>
                        </div>
                      </div>

                      {itemDetailLoading ? (
                        <div className="workspace-stage compact">
                          <span className="workspace-label">Item</span>
                          <strong>正在读取条目详情</strong>
                          <p>当前选中的条目详情正在从 repository 拉取。</p>
                        </div>
                      ) : itemDetailError !== null ? (
                        <div className="error-text">{itemDetailError}</div>
                      ) : selectedItemDetail === null ? (
                        <div className="workspace-stage compact">
                          <span className="workspace-label">Item</span>
                          <strong>当前没有可显示条目</strong>
                          <p>扫描后如果产生可浏览条目，Metadata 会在这里显示当前选中项摘要。</p>
                        </div>
                      ) : (
                        <dl className="kv-list">
                          <div>
                            <dt>Asset ID</dt>
                            <dd>{selectedItemDetail.assetId}</dd>
                          </div>
                          <div>
                            <dt>MIME</dt>
                            <dd>{selectedItemDetail.mime}</dd>
                          </div>
                          <div>
                            <dt>Source Kind</dt>
                            <dd>{selectedItemDetail.sourceKind}</dd>
                          </div>
                          <div>
                            <dt>Location</dt>
                            <dd>{resolveItemLocation(selectedItemDetail)}</dd>
                          </div>
                        </dl>
                      )}
                    </section>
                  </div>
                )}
              </div>

              <footer className="workspace-pane-footer metadata-footer" data-slot="fg-meta-footer">
                <div className="metadata-image-caption" data-slot="fg-meta-footer-caption">
                  {metadataCaption}
                </div>
              </footer>
            </section>
          </aside>
        </div>
      </div>

      <ImportTaskPanel
        open={importTaskPanelOpen}
        onClose={() => setImportTaskPanelOpen(false)}
        importRootPath={importRootPath}
        onImportRootPathChange={setImportRootPath}
        onPickDirectory={() => void handlePickDirectory()}
        onAddLibrary={() => void handleAddLibrary(false)}
        onAddAndScan={() => void handleAddLibrary(true)}
        actionPendingLabel={actionPendingLabel}
        actionMessage={actionMessage}
        actionError={actionError}
        busy={importBusy}
        libraryCount={libraries.length}
        currentLibraryPath={selectedLibraryDetail?.rootPath ?? selectedLibrarySummary?.rootPath ?? null}
        scanStateLabel={scanStateLabel}
        scanSummary={scanSummary}
        scanProgressPercent={scanProgressPercent}
        activities={importActivitiesForPanel}
      />

      {settingsOpen ? (
        <div className="settings-mask" onClick={() => setSettingsOpen(false)}>
          <section
            className="mpx-large-panel settings-panel"
            role="dialog"
            data-testid="settings-panel"
            aria-modal="true"
            aria-labelledby="app-settings-title"
            onClick={(event) => event.stopPropagation()}
          >
            <header className="mpx-large-panel-head settings-panel-head">
              <div className="mpx-large-panel-head-spacer" aria-hidden="true" />
              <h2 id="app-settings-title">设置</h2>
              <button className="mpx-btn settings-close-btn" type="button" onClick={() => setSettingsOpen(false)}>
                关闭
              </button>
            </header>

            <div className="mpx-large-panel-shell settings-panel-shell">
              <aside className="mpx-large-panel-side settings-panel-side">
                <button
                  className={`mpx-btn ${settingsPage === 'ui' ? 'is-active' : ''}`}
                  type="button"
                  data-testid="settings-page-ui"
                  aria-pressed={settingsPage === 'ui'}
                  onClick={() => setSettingsPage('ui')}
                >
                  界面设置
                </button>
                <button
                  className={`mpx-btn ${settingsPage === 'database' ? 'is-active' : ''}`}
                  type="button"
                  data-testid="settings-page-database"
                  aria-pressed={settingsPage === 'database'}
                  onClick={() => setSettingsPage('database')}
                >
                  数据库管理
                </button>
              </aside>

              <section className="mpx-large-panel-main settings-panel-main">
                {settingsPage === 'ui' ? (
                  <div className="settings-page-block" data-testid="settings-page-ui-body">
                    <div className="panel-heading settings-page-heading">
                      <div>
                        <span className="section-kicker">Interface</span>
                        <h2>界面设置</h2>
                      </div>
                    </div>

                    <UiSettingsRangeField
                      label="面板背景遮罩透明度"
                      valueLabel={`${Math.round(settingsBackdropOpacity)}%`}
                      hint="数值越高背景越暗，用于控制设置类大面板出现时的遮罩深度。"
                      min={0}
                      max={100}
                      step={1}
                      value={settingsBackdropOpacity}
                      onChange={setSettingsBackdropOpacity}
                    />
                    <UiSettingsRangeField
                      label="容器外边界系数"
                      valueLabel={`${layoutGapScaleCoeff.toFixed(2)}x / ${layoutPreview.layoutGapPx}px`}
                      hint="基准为窗口宽度的 1%，当前只驱动外留白与 Header 间距。"
                      min={0}
                      max={3}
                      step={0.1}
                      value={layoutGapScaleCoeff}
                      onChange={setLayoutGapScaleCoeff}
                    />
                    <UiSettingsRangeField
                      label="容器内边距系数"
                      valueLabel={`${paneInnerGapScaleCoeff.toFixed(2)}x / ${layoutPreview.paneInnerPaddingPx}px`}
                      hint="基准同样为窗口宽度的 1%，当前用于控制容器内部 padding。"
                      min={0}
                      max={2}
                      step={0.1}
                      value={paneInnerGapScaleCoeff}
                      onChange={setPaneInnerGapScaleCoeff}
                    />
                    <UiSettingsRangeField
                      label="容器内上中下间距系数"
                      valueLabel={`${paneStackGapScaleCoeff.toFixed(2)}x / ${layoutPreview.paneStackGapPx}px`}
                      hint="按容器内边距的 75% 计算，仅用于控制 Sidebar、Main、Metadata 三列中 header、main、footer 之间的纵向间距。"
                      min={0}
                      max={2}
                      step={0.1}
                      value={paneStackGapScaleCoeff}
                      onChange={setPaneStackGapScaleCoeff}
                    />
                    <UiSettingsRangeField
                      label="分割条宽度系数"
                      valueLabel={`${splitterWidthScaleCoeff.toFixed(2)}x / ${layoutPreview.splitterWidthPx}px`}
                      hint="仅控制 Sidebar/Main/Metadata 之间的分隔宽度，不影响 Header 与工作区的间距。"
                      min={0.5}
                      max={2}
                      step={0.1}
                      value={splitterWidthScaleCoeff}
                      onChange={setSplitterWidthScaleCoeff}
                    />
                  </div>
                ) : (
                  <div className="settings-page-block" data-testid="settings-page-database-body">
                    <div className="panel-heading settings-page-heading">
                      <div>
                        <span className="section-kicker">Database</span>
                        <h2>数据库管理</h2>
                      </div>
                    </div>

                    <p className="settings-page-caption">
                      当前首轮只接 3 个动作：清除数据库、选择 SQL 目录、选择缩略图目录。清除后会恢复到首次打开 App 的初始化状态。
                    </p>

                    {runtimeInfoError === null ? null : <div className="error-text">{runtimeInfoError}</div>}
                    {databaseActionError === null ? null : <div className="error-text">{databaseActionError}</div>}

                    {databaseActionMessage === null ? null : (
                      <div className="result-card" data-testid="database-action-message">
                        <span className="result-label">数据库动作</span>
                        <strong>{databasePendingLabel ?? '已完成'}</strong>
                        <p>{databaseActionMessage}</p>
                      </div>
                    )}

                    <article className="settings-manage-card">
                      <div className="settings-manage-heading">
                        <span className="workspace-label">Reset</span>
                        <h3>清除数据库</h3>
                      </div>
                      <p className="settings-page-caption">
                        会清掉当前 SQL、缩略图缓存、normalize 缓存、playback sessions 和运行时存储路径配置，然后自动重新加载应用。
                      </p>
                      <div className="settings-action-row">
                        <button
                          className="mpx-btn is-danger"
                          type="button"
                          data-testid="database-clear-button"
                          onClick={handleRequestClearDatabase}
                          disabled={databaseActionBusy !== null}
                        >
                          {databaseActionBusy === 'clearDatabase' ? '清除中...' : '清除数据库'}
                        </button>
                        <span className="settings-inline-note">成功后会自动 reload。</span>
                      </div>
                    </article>

                    <article className="settings-manage-card">
                      <div className="settings-manage-heading">
                        <span className="workspace-label">SQL</span>
                        <h3>SQL 目录</h3>
                      </div>
                      <div className="settings-path-output" data-testid="database-sql-path">{runtimeInfoDatabasePath}</div>
                      <div className="settings-action-row">
                        <button
                          className="mpx-btn"
                          type="button"
                          data-testid="database-select-sql-dir"
                          onClick={() => void handlePickDatabaseDirectory()}
                          disabled={databaseActionBusy !== null}
                        >
                          {databaseActionBusy === 'pickDatabaseDir' ? '保存中...' : '选择 SQL 目录'}
                        </button>
                        <span className="settings-inline-note">选择的是目录；旧 DB 会迁移 `db / -wal / -shm`。</span>
                      </div>
                    </article>

                    <article className="settings-manage-card">
                      <div className="settings-manage-heading">
                        <span className="workspace-label">Thumbnail</span>
                        <h3>缩略图目录</h3>
                      </div>
                      <div className="settings-path-output" data-testid="database-thumbnail-path">{runtimeInfoThumbnailCachePath}</div>
                      <div className="settings-action-row">
                        <button
                          className="mpx-btn"
                          type="button"
                          data-testid="database-select-thumbnail-dir"
                          onClick={() => void handlePickThumbnailDirectory()}
                          disabled={databaseActionBusy !== null}
                        >
                          {databaseActionBusy === 'pickThumbnailDir' ? '保存中...' : '选择缩略图目录'}
                        </button>
                        <span className="settings-inline-note">只切换目录并确保存在，首轮不迁移旧缓存。</span>
                      </div>
                    </article>
                  </div>
                )}
              </section>
            </div>
          </section>
        </div>
      ) : null}

      {clearDatabaseDialogOpen ? (
        <div
          className="settings-subdialog-overlay"
          onClick={handleCloseClearDatabaseDialog}
          data-testid="database-clear-dialog-overlay"
          role="presentation"
        >
          <section
            className="mpx-dialog-panel settings-confirm-panel"
            role="dialog"
            data-testid="database-clear-dialog"
            aria-modal="true"
            aria-labelledby="clear-database-dialog-title"
            onClick={(event) => event.stopPropagation()}
          >
            <div className="settings-confirm-heading">
              <span className="workspace-label">Reset Confirmation</span>
              <h3 id="clear-database-dialog-title">确认清除数据库</h3>
            </div>

            <p className="settings-page-caption">
              这会把应用恢复到首次打开状态，不保留任何用户数据或运行时状态。只有点击下方“确认清除”后，才会真正执行清理。
            </p>

            <div className="settings-confirm-copy">
              <span>将清理：</span>
              <span>当前 SQL 文件与 `-wal / -shm`</span>
              <span>缩略图缓存、normalize 缓存、playback sessions</span>
              <span>runtime storage 配置文件</span>
            </div>

            {databaseActionError === null ? null : <div className="error-text">{databaseActionError}</div>}

            <div className="settings-confirm-actions">
              <button
                className="mpx-btn"
                type="button"
                data-testid="database-clear-cancel"
                onClick={handleCloseClearDatabaseDialog}
                disabled={databaseActionBusy === 'clearDatabase'}
              >
                取消
              </button>
              <button
                className="mpx-btn is-danger"
                type="button"
                data-testid="database-clear-confirm"
                onClick={() => void handleConfirmClearDatabase()}
                disabled={databaseActionBusy === 'clearDatabase'}
              >
                {databaseActionBusy === 'clearDatabase' ? '清除中...' : '确认清除'}
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </main>
  )
}

interface UiSettingsRangeFieldProps {
  label: string
  valueLabel: string
  hint: string
  min: number
  max: number
  step: number
  value: number
  onChange: (value: number) => void
}

function UiSettingsRangeField({
  label,
  valueLabel,
  hint,
  min,
  max,
  step,
  value,
  onChange,
}: UiSettingsRangeFieldProps) {
  return (
    <label className="settings-slider-field">
      <div className="settings-slider-row">
        <span className="settings-slider-label">{label}</span>
        <span className="settings-slider-value">{valueLabel}</span>
      </div>
      <input
        className="settings-range"
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(event) => onChange(Number(event.target.value))}
      />
      <span className="settings-slider-hint">{hint}</span>
    </label>
  )
}
