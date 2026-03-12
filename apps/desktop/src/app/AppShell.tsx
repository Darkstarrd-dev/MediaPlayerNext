import type {
  ItemDetail,
  ItemListEntry,
  LibraryDetail,
  LibrarySummary,
  RuntimeInfo,
  ScanStats,
  SidebarNodeSummary,
  TaskProgress,
  WorkspaceCursor,
} from '@mediaplayernext/contracts'
import { open as openDialog } from '@tauri-apps/plugin-dialog'
import { readText as readClipboardText } from '@tauri-apps/plugin-clipboard-manager'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { hasFiles as clipboardHasFiles, readFiles as readClipboardFiles } from 'tauri-plugin-clipboard-x-api'
import type { CSSProperties, PointerEvent as ReactPointerEvent } from 'react'
import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { consumeE2eDirectorySelection } from './e2e-test-bridge'
import { ImportTaskPanel } from './ImportTaskPanel'
import { SettingsIcon } from './SettingsIcon'
import {
  clampNumber,
  formatDateTime,
  formatTaskStateLabel,
  getErrorMessage,
  isActiveTaskProgress,
  isEditablePasteTarget,
  isTaskNotFoundError,
  normalizePathBatch,
  parseClipboardPaths,
  readSessionNumber,
  resolveItemDisplayLabel,
  resolveItemLocation,
  resolveLibrarySelection,
  resolvePathLeaf,
  resolveSpacingPx,
  resolveWorkspaceWidths,
} from './app-shell-utils'
import {
  computeThumbnailGridLayout,
  THUMBNAIL_ZOOM_LEVELS,
  toThumbnailZoomLevel,
  type ThumbnailZoomLevel,
} from './thumbnail-grid-layout'
import { useMediaRepository } from './use-media-repository'

const DEFAULT_VIEWPORT_WIDTH = 1280
const DEFAULT_SETTINGS_BACKDROP_OPACITY = 18
const DEFAULT_LAYOUT_GAP_SCALE_COEFF = 1
const DEFAULT_PANE_INNER_GAP_SCALE_COEFF = 1
const DEFAULT_PANE_STACK_GAP_SCALE_COEFF = 1
const DEFAULT_SPLITTER_WIDTH_SCALE_COEFF = 1
const DEFAULT_SIDEBAR_WIDTH_PX = 300
const DEFAULT_META_WIDTH_PX = 340
const DEFAULT_THUMBNAIL_ZOOM_LEVEL: ThumbnailZoomLevel = 4
const THUMBNAIL_GRID_GAP_PX = 14
const THUMBNAIL_GRID_MIN_CELL_PX = 96

const SETTINGS_STORAGE_KEYS = {
  settingsBackdropOpacity: 'mpnext.ui.settingsBackdropOpacity',
  layoutGapScaleCoeff: 'mpnext.ui.layoutGapScaleCoeff',
  paneInnerGapScaleCoeff: 'mpnext.ui.paneInnerGapScaleCoeff',
  paneStackGapScaleCoeff: 'mpnext.ui.paneStackGapScaleCoeff',
  splitterWidthScaleCoeff: 'mpnext.ui.splitterWidthScaleCoeff',
  sidebarWidthPx: 'mpnext.ui.sidebarWidthPx',
  metaWidthPx: 'mpnext.ui.metaWidthPx',
  thumbnailZoomLevel: 'mpnext.ui.thumbnailZoomLevel',
} as const

const ACTION_LABELS = {
  addLibrary: '登记媒体库',
  addAndScan: '登记并扫描',
  dropImport: '拖拽导入',
  pasteImport: '粘贴导入',
  scan: '开始扫描',
  resume: '恢复扫描',
  refresh: '刷新主界面',
} as const

const DATABASE_ACTION_LABELS = {
  pickDatabaseDir: '选择 SQL 目录',
  pickThumbnailDir: '选择缩略图目录',
  clearDatabase: '清除数据库',
} as const

type DragTarget = 'left' | 'right'
type ActionKind = keyof typeof ACTION_LABELS
type DatabaseActionKind = keyof typeof DATABASE_ACTION_LABELS
type SettingsPage = 'ui' | 'database'

interface DragState {
  target: DragTarget
  startX: number
  startSidebarWidthPx: number
  startMetaWidthPx: number
}

type ImportActivityStatus = 'running' | 'completed' | 'failed'

interface ImportActivity {
  id: string
  title: string
  source: string
  status: ImportActivityStatus
  detail: string
  createdAt: string
}

interface SidebarSelection {
  selectedSidebarNodeId: string | null
  selectedMediaSourceId: string | null
}

function isMediaSourceNode(node: SidebarNodeSummary): boolean {
  return node.nodeType === 'media_source' && typeof node.mediaSourceId === 'string'
}

function resolveSidebarSelection(
  nodes: SidebarNodeSummary[],
  preferredSidebarNodeId?: string | null,
  preferredMediaSourceId?: string | null,
): SidebarSelection {
  if (preferredSidebarNodeId !== undefined && preferredSidebarNodeId !== null) {
    const matchedNode = nodes.find((node) => node.nodeId === preferredSidebarNodeId)
    if (matchedNode) {
      return {
        selectedSidebarNodeId: matchedNode.nodeId,
        selectedMediaSourceId: isMediaSourceNode(matchedNode) ? matchedNode.mediaSourceId ?? null : null,
      }
    }
  }

  if (preferredMediaSourceId !== undefined && preferredMediaSourceId !== null) {
    const matchedNode = nodes.find((node) => node.mediaSourceId === preferredMediaSourceId)
    if (matchedNode) {
      return {
        selectedSidebarNodeId: matchedNode.nodeId,
        selectedMediaSourceId: matchedNode.mediaSourceId ?? null,
      }
    }
  }

  const firstMediaNode = nodes.find((node) => isMediaSourceNode(node))
  if (firstMediaNode) {
    return {
      selectedSidebarNodeId: firstMediaNode.nodeId,
      selectedMediaSourceId: firstMediaNode.mediaSourceId ?? null,
    }
  }

  return {
    selectedSidebarNodeId: nodes[0]?.nodeId ?? null,
    selectedMediaSourceId: null,
  }
}

export function AppShell() {
  const repository = useMediaRepository()
  const libraryLoadRequestIdRef = useRef(0)
  const itemDetailRequestIdRef = useRef(0)
  const handleDropImportRef = useRef<(paths: string[]) => Promise<void>>(async () => undefined)
  const handlePasteImportRef = useRef<(text: string) => Promise<void>>(async () => undefined)
  const activeScanActivityTaskIdRef = useRef<string | null>(null)
  const activeScanActivityEntryIdRef = useRef<string | null>(null)
  const completedScanSurfaceSyncTaskIdRef = useRef<string | null>(null)
  const [mainGridElement, setMainGridElement] = useState<HTMLDivElement | null>(null)

  const [importTaskPanelOpen, setImportTaskPanelOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [settingsPage, setSettingsPage] = useState<SettingsPage>('ui')
  const [clearDatabaseDialogOpen, setClearDatabaseDialogOpen] = useState(false)
  const [viewportWidth, setViewportWidth] = useState(DEFAULT_VIEWPORT_WIDTH)
  const [settingsBackdropOpacity, setSettingsBackdropOpacity] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.settingsBackdropOpacity,
      DEFAULT_SETTINGS_BACKDROP_OPACITY,
      0,
      100,
    ),
  )
  const [layoutGapScaleCoeff, setLayoutGapScaleCoeff] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.layoutGapScaleCoeff,
      DEFAULT_LAYOUT_GAP_SCALE_COEFF,
      0,
      3,
    ),
  )
  const [paneInnerGapScaleCoeff, setPaneInnerGapScaleCoeff] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.paneInnerGapScaleCoeff,
      DEFAULT_PANE_INNER_GAP_SCALE_COEFF,
      0,
      2,
    ),
  )
  const [paneStackGapScaleCoeff, setPaneStackGapScaleCoeff] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.paneStackGapScaleCoeff,
      DEFAULT_PANE_STACK_GAP_SCALE_COEFF,
      0,
      2,
    ),
  )
  const [splitterWidthScaleCoeff, setSplitterWidthScaleCoeff] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.splitterWidthScaleCoeff,
      DEFAULT_SPLITTER_WIDTH_SCALE_COEFF,
      0.5,
      2,
    ),
  )
  const [sidebarWidthPx, setSidebarWidthPx] = useState(() =>
    readSessionNumber(
      SETTINGS_STORAGE_KEYS.sidebarWidthPx,
      DEFAULT_SIDEBAR_WIDTH_PX,
      160,
      640,
    ),
  )
  const [metaWidthPx, setMetaWidthPx] = useState(() =>
    readSessionNumber(SETTINGS_STORAGE_KEYS.metaWidthPx, DEFAULT_META_WIDTH_PX, 200, 720),
  )
  const [dragState, setDragState] = useState<DragState | null>(null)
  const [dropImportActive, setDropImportActive] = useState(false)
  const [mainGridSize, setMainGridSize] = useState({ width: 960, height: 640 })
  const [thumbnailZoomLevel, setThumbnailZoomLevel] = useState<ThumbnailZoomLevel>(() =>
    toThumbnailZoomLevel(
      readSessionNumber(
        SETTINGS_STORAGE_KEYS.thumbnailZoomLevel,
        DEFAULT_THUMBNAIL_ZOOM_LEVEL,
        THUMBNAIL_ZOOM_LEVELS[0],
        THUMBNAIL_ZOOM_LEVELS[THUMBNAIL_ZOOM_LEVELS.length - 1],
      ),
    ),
  )

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

  const [importRootPath, setImportRootPath] = useState('')
  const [actionBusy, setActionBusy] = useState<ActionKind | null>(null)
  const [actionMessage, setActionMessage] = useState<string | null>(null)
  const [actionError, setActionError] = useState<string | null>(null)
  const [importActivities, setImportActivities] = useState<ImportActivity[]>([])
  const [runtimeInfo, setRuntimeInfo] = useState<RuntimeInfo | null>(null)
  const [runtimeInfoLoading, setRuntimeInfoLoading] = useState(false)
  const [runtimeInfoError, setRuntimeInfoError] = useState<string | null>(null)
  const [databaseActionBusy, setDatabaseActionBusy] = useState<DatabaseActionKind | null>(null)
  const [databaseActionMessage, setDatabaseActionMessage] = useState<string | null>(null)
  const [databaseActionError, setDatabaseActionError] = useState<string | null>(null)

  const layoutPreview = useMemo(() => {
    const normalizedLayoutGapScaleCoeff = clampNumber(layoutGapScaleCoeff, 0, 3)
    const normalizedPaneInnerGapScaleCoeff = clampNumber(paneInnerGapScaleCoeff, 0, 2)
    const normalizedPaneStackGapScaleCoeff = clampNumber(paneStackGapScaleCoeff, 0, 2)
    const normalizedSplitterWidthScaleCoeff = clampNumber(splitterWidthScaleCoeff, 0.5, 2)
    const layoutGapPx = resolveSpacingPx(viewportWidth, normalizedLayoutGapScaleCoeff)
    const paneInnerPaddingPx = resolveSpacingPx(viewportWidth, normalizedPaneInnerGapScaleCoeff)
    const paneStackGapPx = Math.max(
      0,
      Math.round(paneInnerPaddingPx * 0.75 * normalizedPaneStackGapScaleCoeff),
    )
    const splitterWidthPx = Math.max(0, Math.round(layoutGapPx * normalizedSplitterWidthScaleCoeff))
    const paneHeaderHeightPx = Math.max(68, Math.round(paneInnerPaddingPx * 3.2))
    const paneFooterHeightPx = Math.max(48, Math.round(paneInnerPaddingPx * 2.2))

    return {
      layoutGapPx,
      paneInnerPaddingPx,
      paneStackGapPx,
      paneHeaderHeightPx,
      paneFooterHeightPx,
      splitterWidthPx,
      normalizedLayoutGapScaleCoeff,
      normalizedPaneInnerGapScaleCoeff,
      normalizedPaneStackGapScaleCoeff,
      normalizedSplitterWidthScaleCoeff,
    }
  }, [
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    splitterWidthScaleCoeff,
    viewportWidth,
  ])

  const workspaceLayout = useMemo(
    () =>
      resolveWorkspaceWidths(
        viewportWidth,
        layoutPreview.layoutGapPx,
        layoutPreview.splitterWidthPx,
        sidebarWidthPx,
        metaWidthPx,
      ),
    [layoutPreview.layoutGapPx, layoutPreview.splitterWidthPx, metaWidthPx, sidebarWidthPx, viewportWidth],
  )

  const workspaceStyle = useMemo(
    () =>
      ({
        '--app-sidebar-width-px': `${workspaceLayout.sidebarWidthPx}px`,
        '--app-meta-width-px': `${workspaceLayout.metaWidthPx}px`,
      }) as CSSProperties,
    [workspaceLayout.metaWidthPx, workspaceLayout.sidebarWidthPx],
  )

  const thumbnailGridLayout = useMemo(
    () =>
      computeThumbnailGridLayout({
        containerWidth: mainGridSize.width,
        containerHeight: mainGridSize.height,
        zoomLevel: thumbnailZoomLevel,
        gapPx: THUMBNAIL_GRID_GAP_PX,
        minCellSizePx: THUMBNAIL_GRID_MIN_CELL_PX,
      }),
    [mainGridSize.height, mainGridSize.width, thumbnailZoomLevel],
  )

  const itemGridStyle = useMemo(
    () =>
      ({
        gridTemplateColumns: `repeat(${thumbnailGridLayout.columns}, minmax(0, ${thumbnailGridLayout.cellSizePx}px))`,
        gap: `${thumbnailGridLayout.gapPx}px`,
      }) as CSSProperties,
    [thumbnailGridLayout.cellSizePx, thumbnailGridLayout.columns, thumbnailGridLayout.gapPx],
  )

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

  const appendImportActivity = useCallback(
    (activity: Omit<ImportActivity, 'id' | 'createdAt'>): string => {
      const nextActivity: ImportActivity = {
        ...activity,
        id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
        createdAt: new Date().toISOString(),
      }

      setImportActivities((current) => [nextActivity, ...current].slice(0, 8))
      return nextActivity.id
    },
    [],
  )

  const updateImportActivity = useCallback(
    (activityId: string, patch: Partial<Omit<ImportActivity, 'id' | 'createdAt'>>) => {
      setImportActivities((current) =>
        current.map((activity) =>
          activity.id === activityId
            ? {
                ...activity,
                ...patch,
              }
            : activity,
        ),
      )
    },
    [],
  )

  const bootstrapScanSnapshot = useCallback(
    async (libraryId: string): Promise<void> => {
      const maxAttempts = 24

      for (let attempt = 0; attempt < maxAttempts; attempt += 1) {
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
    [repository],
  )

  const loadLibrarySurface = useCallback(
    async (options: {
      libraryId: string
      mediaSourceId: string | null
      requestedPageIndex: number
      preferredAssetId?: string | null
    }) => {
      const { libraryId, mediaSourceId, requestedPageIndex, preferredAssetId } = options
      const requestId = libraryLoadRequestIdRef.current + 1
      libraryLoadRequestIdRef.current = requestId

      setWorkspaceRefreshing(true)
      setWorkspaceError(null)

      try {
        const pageSize = thumbnailGridLayout.pageSize
        let resolvedPageIndex = Math.max(1, requestedPageIndex)

        const readItemsPage = (pageIndex: number) =>
          repository.items.list({
            libraryId,
            mediaSourceId: mediaSourceId ?? undefined,
            page: pageIndex,
            pageSize,
          })

        let nextItems = await readItemsPage(resolvedPageIndex)

        while (resolvedPageIndex > 1 && nextItems.length === 0) {
          resolvedPageIndex -= 1
          nextItems = await readItemsPage(resolvedPageIndex)
        }

        const [detail, stats, snapshot] = await Promise.all([
          repository.library.get(libraryId),
          repository.scan.stats(libraryId),
          repository.scan.snapshot(libraryId).catch((error: unknown) => {
            if (isTaskNotFoundError(error)) {
              return null
            }

            throw error
          }),
        ])

        if (libraryLoadRequestIdRef.current !== requestId) {
          return
        }

        setSelectedLibraryDetail(detail)
        setScanStats(stats)
        setScanSnapshot(snapshot)
        setItemsPageIndex(resolvedPageIndex)
        setItemsHasNextPage(resolvedPageIndex === requestedPageIndex && nextItems.length === pageSize)
        setItems(nextItems)
        setWorkspaceHydrated(true)
        setItemThumbnailUrls((current) => {
          const allowedAssetIds = new Set(nextItems.map((item) => item.assetId))
          const next: Record<string, string> = {}

          for (const [assetId, url] of Object.entries(current)) {
            if (allowedAssetIds.has(assetId)) {
              next[assetId] = url
            }
          }

          return next
        })
        setSelectedAssetId((currentAssetId) => {
          if (preferredAssetId && nextItems.some((item) => item.assetId === preferredAssetId)) {
            return preferredAssetId
          }

          if (currentAssetId !== null && nextItems.some((item) => item.assetId === currentAssetId)) {
            return currentAssetId
          }

          return nextItems[0]?.assetId ?? null
        })
      } catch (error) {
        if (libraryLoadRequestIdRef.current !== requestId) {
          return
        }

        setWorkspaceError(getErrorMessage(error))

        if (!workspaceHydrated) {
          setSelectedLibraryDetail(null)
          setScanStats(null)
          setScanSnapshot(null)
          setItems([])
          setItemsPageIndex(1)
          setItemsHasNextPage(false)
          setSelectedAssetId(null)
          setItemThumbnailUrls({})
        }
      } finally {
        if (libraryLoadRequestIdRef.current === requestId) {
          setWorkspaceRefreshing(false)
        }
      }
    },
    [repository, thumbnailGridLayout.pageSize, workspaceHydrated],
  )

  const refreshLibraries = useCallback(
    async (preferredLibraryId?: string | null): Promise<string | null> => {
      setLibrariesLoading(true)

      try {
        const nextLibraries = await repository.library.list()
        const nextSelectedLibraryId = resolveLibrarySelection(nextLibraries, preferredLibraryId)

        setLibraries(nextLibraries)
        setSelectedLibraryId(nextSelectedLibraryId)

        return nextSelectedLibraryId
      } catch (error) {
        setLibraries([])
        setSelectedLibraryId(null)
        setWorkspaceError(getErrorMessage(error))
        clearWorkspaceData()
        return null
      } finally {
        setLibrariesLoading(false)
      }
    },
    [clearWorkspaceData, repository],
  )

  const refreshSidebarNodes = useCallback(
    async (
      libraryId: string,
      preferredSidebarNodeId?: string | null,
      preferredMediaSourceId?: string | null,
    ): Promise<SidebarSelection> => {
      setSidebarNodesLoading(true)
      setSidebarNodes([])
      setSelectedSidebarNodeId(null)
      setSelectedMediaSourceId(null)

      try {
        const nextNodes = await repository.library.nodes(libraryId)
        const nextSelection = resolveSidebarSelection(
          nextNodes,
          preferredSidebarNodeId,
          preferredMediaSourceId,
        )

        setSidebarNodes(nextNodes)
        setSelectedSidebarNodeId(nextSelection.selectedSidebarNodeId)
        setSelectedMediaSourceId(nextSelection.selectedMediaSourceId)
        return nextSelection
      } catch (error) {
        setSidebarNodes([])
        setSelectedSidebarNodeId(null)
        setSelectedMediaSourceId(null)
        setWorkspaceError(getErrorMessage(error))
        return {
          selectedSidebarNodeId: null,
          selectedMediaSourceId: null,
        }
      } finally {
        setSidebarNodesLoading(false)
      }
    },
    [repository],
  )

  const refreshWorkspace = useCallback(
    async (options?: {
      preferredLibraryId?: string | null
      preferredSidebarNodeId?: string | null
      preferredMediaSourceId?: string | null
      preferredPageIndex?: number
      preferredAssetId?: string | null
    }) => {
      const preferredLibraryId = options?.preferredLibraryId
      const nextSelectedLibraryId = await refreshLibraries(preferredLibraryId)

      if (nextSelectedLibraryId === null) {
        clearWorkspaceData()
        return
      }

      const nextSelection = await refreshSidebarNodes(
        nextSelectedLibraryId,
        options?.preferredSidebarNodeId,
        options?.preferredMediaSourceId,
      )
      const nextPageIndex = Math.max(1, options?.preferredPageIndex ?? 1)

      setItemsPageIndex(nextPageIndex)
      await loadLibrarySurface({
        libraryId: nextSelectedLibraryId,
        mediaSourceId: nextSelection.selectedMediaSourceId,
        requestedPageIndex: nextPageIndex,
        preferredAssetId: options?.preferredAssetId,
      })
    },
    [clearWorkspaceData, loadLibrarySurface, refreshLibraries, refreshSidebarNodes],
  )

  const previousPageSizeRef = useRef(thumbnailGridLayout.pageSize)

  useEffect(() => {
    const previousPageSize = previousPageSizeRef.current
    previousPageSizeRef.current = thumbnailGridLayout.pageSize

    if (selectedLibraryId === null || previousPageSize === thumbnailGridLayout.pageSize) {
      return
    }

    void loadLibrarySurface({
      libraryId: selectedLibraryId,
      mediaSourceId: selectedMediaSourceId,
      requestedPageIndex: itemsPageIndex,
    })
  }, [
    itemsPageIndex,
    loadLibrarySurface,
    selectedLibraryId,
    selectedMediaSourceId,
    thumbnailGridLayout.pageSize,
  ])

  useEffect(() => {
    if (typeof window === 'undefined') {
      return
    }

    const updateViewportWidth = (): void => {
      setViewportWidth(window.innerWidth)
    }

    updateViewportWidth()
    window.addEventListener('resize', updateViewportWidth)

    return () => {
      window.removeEventListener('resize', updateViewportWidth)
    }
  }, [])

  useEffect(() => {
    let cancelled = false

    const hydrateWorkspace = async (): Promise<void> => {
      try {
        const cursor = await repository.database.readWorkspaceCursor()
        if (cancelled) {
          return
        }

        const normalizedCursor: WorkspaceCursor = {
          selectedLibraryId: cursor?.selectedLibraryId ?? null,
          selectedSidebarNodeId:
            cursor?.selectedSidebarNodeId ?? cursor?.selectedNodeId ?? null,
          selectedMediaSourceId: cursor?.selectedMediaSourceId ?? null,
          selectedNodeId: cursor?.selectedNodeId ?? null,
          itemsPageIndex: Math.max(1, cursor?.itemsPageIndex ?? 1),
          selectedAssetId: cursor?.selectedAssetId ?? null,
        }

        await refreshWorkspace({
          preferredLibraryId: normalizedCursor.selectedLibraryId,
          preferredSidebarNodeId: normalizedCursor.selectedSidebarNodeId,
          preferredMediaSourceId: normalizedCursor.selectedMediaSourceId,
          preferredPageIndex: normalizedCursor.itemsPageIndex ?? 1,
          preferredAssetId: normalizedCursor.selectedAssetId,
        })
      } catch {
        if (cancelled) {
          return
        }

        await refreshWorkspace()
      }
    }

    void hydrateWorkspace()

    return () => {
      cancelled = true
    }
  }, [refreshWorkspace, repository])

  useEffect(() => {
    const root = document.documentElement

    root.style.setProperty(
      '--mpx-settings-backdrop-opacity',
      `${clampNumber(settingsBackdropOpacity, 0, 100).toFixed(0)}%`,
    )
    root.style.setProperty(
      '--mpx-layout-gap-scale',
      layoutPreview.normalizedLayoutGapScaleCoeff.toFixed(2),
    )
    root.style.setProperty('--mpx-layout-gap-px', `${layoutPreview.layoutGapPx}px`)
    root.style.setProperty('--mpx-layout-padding', `${layoutPreview.layoutGapPx}px`)
    root.style.setProperty(
      '--mpx-header-floating-gap',
      `${layoutPreview.layoutGapPx}px ${layoutPreview.layoutGapPx}px 0px`,
    )
    root.style.setProperty(
      '--mpx-pane-inner-gap-scale',
      layoutPreview.normalizedPaneInnerGapScaleCoeff.toFixed(2),
    )
    root.style.setProperty('--mpx-pane-inner-padding-px', `${layoutPreview.paneInnerPaddingPx}px`)
    root.style.setProperty(
      '--mpx-pane-stack-gap-scale',
      layoutPreview.normalizedPaneStackGapScaleCoeff.toFixed(2),
    )
    root.style.setProperty('--mpx-pane-stack-gap-px', `${layoutPreview.paneStackGapPx}px`)
    root.style.setProperty('--mpx-pane-section-gap-px', `${layoutPreview.paneStackGapPx}px`)
    root.style.setProperty('--mpx-pane-header-height-px', `${layoutPreview.paneHeaderHeightPx}px`)
    root.style.setProperty('--mpx-pane-footer-height-px', `${layoutPreview.paneFooterHeightPx}px`)
    root.style.setProperty(
      '--mpx-splitter-width-scale',
      layoutPreview.normalizedSplitterWidthScaleCoeff.toFixed(2),
    )
    root.style.setProperty('--mpx-splitter-width', `${layoutPreview.splitterWidthPx}px`)
  }, [layoutPreview, settingsBackdropOpacity])

  useEffect(() => {
    if (typeof window === 'undefined') {
      return
    }

    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.settingsBackdropOpacity,
      settingsBackdropOpacity.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.layoutGapScaleCoeff,
      layoutGapScaleCoeff.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.paneInnerGapScaleCoeff,
      paneInnerGapScaleCoeff.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.paneStackGapScaleCoeff,
      paneStackGapScaleCoeff.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.splitterWidthScaleCoeff,
      splitterWidthScaleCoeff.toString(),
    )
    window.sessionStorage.setItem(
      SETTINGS_STORAGE_KEYS.sidebarWidthPx,
      workspaceLayout.sidebarWidthPx.toString(),
    )
    window.sessionStorage.setItem(SETTINGS_STORAGE_KEYS.metaWidthPx, workspaceLayout.metaWidthPx.toString())
    window.sessionStorage.setItem(SETTINGS_STORAGE_KEYS.thumbnailZoomLevel, thumbnailZoomLevel.toString())
  }, [
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    settingsBackdropOpacity,
    splitterWidthScaleCoeff,
    thumbnailZoomLevel,
    workspaceLayout.metaWidthPx,
    workspaceLayout.sidebarWidthPx,
  ])

  useEffect(() => {
    if (!workspaceHydrated) {
      return
    }

    const timer = window.setTimeout(() => {
      void repository.database
        .writeWorkspaceCursor({
          selectedLibraryId,
          selectedSidebarNodeId,
          selectedMediaSourceId,
          selectedNodeId: selectedSidebarNodeId,
          itemsPageIndex,
          selectedAssetId,
        })
        .catch(() => {
          // ignore workspace cursor persistence failure
        })
    }, 180)

    return () => {
      window.clearTimeout(timer)
    }
  }, [
    itemsPageIndex,
    repository,
    selectedAssetId,
    selectedLibraryId,
    selectedMediaSourceId,
    selectedSidebarNodeId,
    workspaceHydrated,
  ])

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

  useEffect(() => {
    if (!dragState) {
      return
    }

    const previousUserSelect = document.body.style.userSelect
    const previousCursor = document.body.style.cursor
    document.body.style.userSelect = 'none'
    document.body.style.cursor = 'col-resize'

    const handlePointerMove = (event: PointerEvent): void => {
      const deltaX = event.clientX - dragState.startX

      if (dragState.target === 'left') {
        setSidebarWidthPx(dragState.startSidebarWidthPx + deltaX)
        return
      }

      setMetaWidthPx(dragState.startMetaWidthPx - deltaX)
    }

    const handlePointerUp = (): void => {
      setDragState(null)
    }

    window.addEventListener('pointermove', handlePointerMove)
    window.addEventListener('pointerup', handlePointerUp)

    return () => {
      document.body.style.userSelect = previousUserSelect
      document.body.style.cursor = previousCursor
      window.removeEventListener('pointermove', handlePointerMove)
      window.removeEventListener('pointerup', handlePointerUp)
    }
  }, [dragState])

  useEffect(() => {
    if (mainGridElement === null) {
      return
    }

    const updateGridSize = (width: number, height: number) => {
      const nextWidth = Math.max(0, Math.round(width))
      const nextHeight = Math.max(0, Math.round(height))

      setMainGridSize((current) => {
        if (current.width === nextWidth && current.height === nextHeight) {
          return current
        }

        return {
          width: nextWidth,
          height: nextHeight,
        }
      })
    }

    const initialRect = mainGridElement.getBoundingClientRect()
    updateGridSize(initialRect.width, initialRect.height)

    const observer = new ResizeObserver((entries) => {
      const entry = entries[0]
      if (!entry) {
        return
      }

      updateGridSize(entry.contentRect.width, entry.contentRect.height)
    })

    observer.observe(mainGridElement)
    return () => observer.disconnect()
  }, [mainGridElement])

  useEffect(() => {
    if (selectedAssetId === null) {
      itemDetailRequestIdRef.current += 1
      setSelectedItemDetail(null)
      setItemDetailError(null)
      setItemDetailLoading(false)
      return
    }

    const requestId = itemDetailRequestIdRef.current + 1
    itemDetailRequestIdRef.current = requestId
    setItemDetailLoading(true)
    setItemDetailError(null)

    void repository.items
      .detail(selectedAssetId)
      .then((detail) => {
        if (itemDetailRequestIdRef.current !== requestId) {
          return
        }

        setSelectedItemDetail(detail)
      })
      .catch((error: unknown) => {
        if (itemDetailRequestIdRef.current !== requestId) {
          return
        }

        setSelectedItemDetail(null)
        setItemDetailError(getErrorMessage(error))
      })
      .finally(() => {
        if (itemDetailRequestIdRef.current === requestId) {
          setItemDetailLoading(false)
        }
      })
  }, [repository, selectedAssetId])

  useEffect(() => {
    if (items.length === 0) {
      setItemThumbnailUrls({})
      return
    }

    let disposed = false

    const visibleAssetIds = new Set(items.map((item) => item.assetId))
    setItemThumbnailUrls((current) => {
      const next: Record<string, string> = {}
      for (const [assetId, url] of Object.entries(current)) {
        if (visibleAssetIds.has(assetId)) {
          next[assetId] = url
        }
      }
      return next
    })

    const applyThumbnailUrl = (assetId: string, thumbnailUrl: string) => {
      if (disposed) {
        return
      }

      setItemThumbnailUrls((current) => {
        if (current[assetId] === thumbnailUrl) {
          return current
        }

        return {
          ...current,
          [assetId]: thumbnailUrl,
        }
      })
    }

    for (const item of items) {
      if (item.thumbnailKey) {
        applyThumbnailUrl(item.assetId, repository.urls.thumbnail(item.thumbnailKey))
        continue
      }

      void repository.thumbnail
        .ensure(item.assetId, 'grid-md')
        .then((ensuredThumbnail) => {
          applyThumbnailUrl(item.assetId, repository.urls.thumbnail(ensuredThumbnail.thumbnailKey))
        })
        .catch(() => {
          // ignore missing thumbnail and keep placeholder
        })
    }

    return () => {
      disposed = true
    }
  }, [items, repository])

  function handleSplitterPointerDown(target: DragTarget) {
    return (event: ReactPointerEvent<HTMLDivElement>): void => {
      event.preventDefault()
      setDragState({
        target,
        startX: event.clientX,
        startSidebarWidthPx: workspaceLayout.sidebarWidthPx,
        startMetaWidthPx: workspaceLayout.metaWidthPx,
      })
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

  const handleAddLibrary = useCallback(
    async (shouldScanAfterAdd: boolean) => {
      const rootPath = importRootPath.trim()

      if (rootPath.length === 0) {
        setActionError('请先输入要登记的本地路径。')
        return
      }

      const activityId = appendImportActivity({
        title: shouldScanAfterAdd ? '登记并扫描' : '登记媒体库',
        source: '手动路径',
        status: 'running',
        detail: shouldScanAfterAdd
          ? `正在登记并扫描：${rootPath}`
          : `正在登记媒体库：${rootPath}`,
      })

      setActionBusy(shouldScanAfterAdd ? 'addAndScan' : 'addLibrary')
      setActionError(null)

      try {
        const createdLibrary = await repository.library.add({ rootPath })

        if (shouldScanAfterAdd) {
          setActionMessage(`已登记并启动扫描：${createdLibrary.rootPath}`)
          updateImportActivity(activityId, {
            status: 'completed',
            detail: `已登记并启动扫描：${createdLibrary.rootPath}`,
          })
          void repository.scan
            .start(createdLibrary.id)
            .then(async () => {
              await bootstrapScanSnapshot(createdLibrary.id)
            })
            .catch((error: unknown) => {
              setActionError(`扫描启动失败：${getErrorMessage(error)}`)
            })
        } else {
          setActionMessage(`已登记媒体库：${createdLibrary.rootPath}`)
          updateImportActivity(activityId, {
            status: 'completed',
            detail: `已登记媒体库：${createdLibrary.rootPath}`,
          })
        }

        setImportRootPath('')
        await refreshWorkspace({
          preferredLibraryId: createdLibrary.id,
          preferredPageIndex: 1,
        })
      } catch (error) {
        setActionError(getErrorMessage(error))
        updateImportActivity(activityId, {
          status: 'failed',
          detail: `登记失败：${getErrorMessage(error)}`,
        })
      } finally {
        setActionBusy(null)
      }
    },
    [
      appendImportActivity,
      bootstrapScanSnapshot,
      importRootPath,
      refreshWorkspace,
      repository,
      updateImportActivity,
    ],
  )

  const runPathImport = useCallback(
    async (
      rawPaths: string[],
      actionKind: Extract<ActionKind, 'dropImport' | 'pasteImport'>,
      emptyErrorMessage: string,
      successSummaryLabel: string,
      failureSummaryLabel: string,
    ) => {
      const paths = normalizePathBatch(rawPaths)

      if (paths.length === 0) {
        setActionError(emptyErrorMessage)
        return
      }

      const activityId = appendImportActivity({
        title: ACTION_LABELS[actionKind],
        source: actionKind === 'dropImport' ? '拖拽导入' : '粘贴导入',
        status: 'running',
        detail: `正在处理 ${paths.length} 条路径。`,
      })

      setActionBusy(actionKind)
      setActionError(null)
      setActionMessage(null)

      const successLibraries: LibraryDetail[] = []
      const failedPaths: string[] = []

      for (const path of paths) {
        try {
          const createdLibrary = await repository.library.add({ rootPath: path })
          successLibraries.push(createdLibrary)
          void repository.scan
            .start(createdLibrary.id)
            .then(async () => {
              await bootstrapScanSnapshot(createdLibrary.id)
            })
            .catch((error: unknown) => {
              setActionError(`扫描启动失败：${getErrorMessage(error)}`)
            })
        } catch (error) {
          failedPaths.push(`${path}：${getErrorMessage(error)}`)
        }
      }

      try {
        const preferredLibraryId = successLibraries.at(-1)?.id ?? selectedLibraryId
        await refreshWorkspace({
          preferredLibraryId,
          preferredPageIndex: 1,
        })
      } finally {
        setActionBusy(null)
      }

      if (successLibraries.length > 0) {
        const successSummary = `已处理 ${successLibraries.length} 条${successSummaryLabel}，并刷新主界面快照。`

        if (failedPaths.length > 0) {
          setActionMessage(`${successSummary} 部分路径失败。`)
          setActionError(failedPaths.join('；'))
          updateImportActivity(activityId, {
            status: 'completed',
            detail: `${successSummary} 部分路径失败。`,
          })
        } else {
          setActionMessage(successSummary)
          setActionError(null)
          updateImportActivity(activityId, {
            status: 'completed',
            detail: successSummary,
          })
        }

        return
      }

      setActionMessage(null)
      setActionError(failedPaths.join('；') || failureSummaryLabel)
      updateImportActivity(activityId, {
        status: 'failed',
        detail: failedPaths.join('；') || failureSummaryLabel,
      })
    },
    [
      appendImportActivity,
      bootstrapScanSnapshot,
      refreshWorkspace,
      repository,
      selectedLibraryId,
      updateImportActivity,
    ],
  )

  const handleDropImport = useCallback(
    async (rawPaths: string[]) => {
      await runPathImport(
        rawPaths,
        'dropImport',
        '未从拖拽事件中解析到可用路径。',
        '拖拽路径',
        '拖拽导入失败。',
      )
    },
    [runPathImport],
  )

  const handlePasteImport = useCallback(
    async (rawText: string) => {
      await runPathImport(
        parseClipboardPaths(rawText),
        'pasteImport',
        '未从剪贴板解析到可用路径。',
        '粘贴路径',
        '粘贴导入失败。',
      )
    },
    [runPathImport],
  )

  useEffect(() => {
    handleDropImportRef.current = handleDropImport
  }, [handleDropImport])

  useEffect(() => {
    handlePasteImportRef.current = handlePasteImport
  }, [handlePasteImport])

  useEffect(() => {
    let cancelled = false
    let cleanup: (() => void) | null = null

    void getCurrentWindow()
      .onDragDropEvent((event) => {
        if (cancelled) {
          return
        }

        if (event.payload.type === 'enter' || event.payload.type === 'over') {
          setDropImportActive(true)
          return
        }

        if (event.payload.type === 'leave') {
          setDropImportActive(false)
          return
        }

        setDropImportActive(false)
        setImportTaskPanelOpen(true)
        void handleDropImportRef.current(event.payload.paths)
      })
      .then((unlisten) => {
        if (cancelled) {
          void unlisten()
          return
        }

        cleanup = unlisten
      })
      .catch((error: unknown) => {
        setActionError(`拖拽监听初始化失败：${getErrorMessage(error)}`)
      })

    return () => {
      cancelled = true
      cleanup?.()
    }
  }, [])

  useEffect(() => {
    const handlePaste = (event: ClipboardEvent): void => {
      if (isEditablePasteTarget(event.target)) {
        return
      }

      const clipboardText =
        event.clipboardData?.getData('text/plain') || event.clipboardData?.getData('text/uri-list') || ''

      const processPaste = async (): Promise<void> => {
        const nativeFilePaths = await clipboardHasFiles()
          .then(async (hasFiles) => {
            if (!hasFiles) {
              return []
            }

            const result = await readClipboardFiles()
            return normalizePathBatch(result.paths)
          })
          .catch(() => [])

        if (nativeFilePaths.length > 0) {
          event.preventDefault()
          setImportTaskPanelOpen(true)
          await handlePasteImportRef.current(nativeFilePaths.join('\n'))
          return
        }

        const fallbackText = clipboardText.length > 0 ? clipboardText : await readClipboardText().catch(() => '')
        const parsedPaths = parseClipboardPaths(fallbackText)

        if (parsedPaths.length === 0) {
          return
        }

        event.preventDefault()
        setImportTaskPanelOpen(true)
        await handlePasteImportRef.current(fallbackText)
      }

      void processPaste()
    }

    window.addEventListener('paste', handlePaste)

    return () => {
      window.removeEventListener('paste', handlePaste)
    }
  }, [])

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

    const statusMap: Record<TaskProgress['state'], ImportActivityStatus> = {
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

  const pickSingleDirectory = useCallback(async (title: string): Promise<string | null> => {
    const e2eSelection = consumeE2eDirectorySelection(title)
    if (e2eSelection.handled) {
      return e2eSelection.path
    }

    const selection = await openDialog({
      directory: true,
      multiple: false,
      title,
    })

    if (selection === null) {
      return null
    }

    const nextPath = Array.isArray(selection) ? selection[0] : selection
    return typeof nextPath === 'string' && nextPath.trim().length > 0 ? nextPath : null
  }, [])

  const handlePickDirectory = useCallback(async () => {
    setActionError(null)

    try {
      const nextPath = await pickSingleDirectory('选择媒体库目录')
      if (nextPath === null) {
        return
      }

      setImportRootPath(nextPath)
    } catch (error) {
      setActionError(`系统文件夹选择器不可用：${getErrorMessage(error)}`)
    }
  }, [pickSingleDirectory])

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
