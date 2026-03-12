import type {
  ItemDetail,
  ItemListEntry,
  LibraryDetail,
  LibrarySummary,
  ScanStats,
  SidebarNodeSummary,
} from '@mediaplayernext/contracts'
import type {
  CSSProperties,
  PointerEvent as ReactPointerEvent,
  WheelEvent as ReactWheelEvent,
} from 'react'
import { AppShellMainPane } from './app-shell-main-pane'
import { AppShellMetadataPane } from './app-shell-metadata-pane'
import { AppShellSidebarPane } from './app-shell-sidebar-pane'
import type { ThumbnailZoomLevel } from './thumbnail-grid-layout'
import type { ItemsPageTransitionState } from './use-app-shell-workspace-state'

interface AppShellWorkspaceProps {
  workspaceStyle: CSSProperties
  selectedSidebarNodeId: string | null
  selectedMediaSourceId: string | null
  isLeftSplitterDragging: boolean
  isRightSplitterDragging: boolean
  onLeftSplitterPointerDown: (event: ReactPointerEvent<HTMLDivElement>) => void
  onRightSplitterPointerDown: (event: ReactPointerEvent<HTMLDivElement>) => void
  selectedLibrarySummary: LibrarySummary | null
  selectedLibraryId: string | null
  libraries: LibrarySummary[]
  librariesLoading: boolean
  sidebarNodesLoading: boolean
  sidebarNodes: SidebarNodeSummary[]
  sidebarFooterText: string
  onLibrarySelect: (libraryId: string) => void
  onSidebarNodeSelect: (nodeId: string) => void
  selectedLibraryDetail: LibraryDetail | null
  thumbnailZoomLevel: ThumbnailZoomLevel
  workspaceHydrated: boolean
  workspaceRefreshing: boolean
  workspaceError: string | null
  items: ItemListEntry[]
  selectedAssetId: string | null
  itemThumbnailUrls: Record<string, string>
  itemGridStyle: CSSProperties
  mainFooterPrimary: string
  mainFooterSecondary: string
  mainFooterPageLabel: string
  mainFooterTransitionLabel: string | null
  pageTransitionState: ItemsPageTransitionState
  itemsHasNextPage: boolean
  itemsPageIndex: number
  setMainGridElement: (element: HTMLDivElement | null) => void
  onThumbnailZoomLevelChange: (value: ThumbnailZoomLevel) => void
  onSelectAsset: (assetId: string) => void
  onThumbnailError: (assetId: string) => void
  onGoPreviousItemsPage: () => void
  onGoNextItemsPage: () => void
  onMainGridWheel: (event: ReactWheelEvent<HTMLDivElement>) => void
  selectedItemDetail: ItemDetail | null
  scanStateLabel: string
  scanStateData: string
  scanSummary: string
  scanStats: ScanStats | null
  importBusy: boolean
  actionPendingLabel: string | null
  itemDetailLoading: boolean
  itemDetailError: string | null
}

export function AppShellWorkspace(props: AppShellWorkspaceProps) {
  const {
    workspaceStyle,
    selectedSidebarNodeId,
    selectedMediaSourceId,
    isLeftSplitterDragging,
    isRightSplitterDragging,
    onLeftSplitterPointerDown,
    onRightSplitterPointerDown,
    selectedLibrarySummary,
    selectedLibraryId,
    libraries,
    librariesLoading,
    sidebarNodesLoading,
    sidebarNodes,
    sidebarFooterText,
    onLibrarySelect,
    onSidebarNodeSelect,
    selectedLibraryDetail,
    thumbnailZoomLevel,
    workspaceHydrated,
    workspaceRefreshing,
    workspaceError,
    items,
    selectedAssetId,
    itemThumbnailUrls,
    itemGridStyle,
    mainFooterPrimary,
    mainFooterSecondary,
    mainFooterPageLabel,
    mainFooterTransitionLabel,
    pageTransitionState,
    itemsHasNextPage,
    itemsPageIndex,
    setMainGridElement,
    onThumbnailZoomLevelChange,
    onSelectAsset,
    onThumbnailError,
    onGoPreviousItemsPage,
    onGoNextItemsPage,
    onMainGridWheel,
    selectedItemDetail,
    scanStateLabel,
    scanStateData,
    scanSummary,
    scanStats,
    importBusy,
    actionPendingLabel,
    itemDetailLoading,
    itemDetailError,
  } = props

  return (
    <div
      className="app-workspace"
      style={workspaceStyle}
      data-testid="workspace-root"
      data-active-sidebar-node-id={selectedSidebarNodeId ?? ''}
      data-active-media-source-id={selectedMediaSourceId ?? ''}
    >
      <AppShellSidebarPane
        selectedLibrarySummary={selectedLibrarySummary}
        selectedLibraryId={selectedLibraryId}
        libraries={libraries}
        librariesLoading={librariesLoading}
        sidebarNodesLoading={sidebarNodesLoading}
        sidebarNodes={sidebarNodes}
        selectedSidebarNodeId={selectedSidebarNodeId}
        sidebarFooterText={sidebarFooterText}
        onLibrarySelect={onLibrarySelect}
        onSidebarNodeSelect={onSidebarNodeSelect}
      />

      <div
        className={`workspace-splitter ${isLeftSplitterDragging ? 'is-dragging' : ''}`}
        role="separator"
        aria-orientation="vertical"
        aria-label="调整 Sidebar 与 Main 宽度"
        onPointerDown={onLeftSplitterPointerDown}
      />

      <AppShellMainPane
        selectedLibraryId={selectedLibraryId}
        selectedLibraryDetail={selectedLibraryDetail}
        selectedLibraryRootPath={selectedLibraryDetail?.rootPath ?? null}
        thumbnailZoomLevel={thumbnailZoomLevel}
        workspaceHydrated={workspaceHydrated}
        workspaceRefreshing={workspaceRefreshing}
        workspaceError={workspaceError}
        items={items}
        selectedAssetId={selectedAssetId}
        itemThumbnailUrls={itemThumbnailUrls}
        itemGridStyle={itemGridStyle}
        mainFooterPrimary={mainFooterPrimary}
        mainFooterSecondary={mainFooterSecondary}
        mainFooterPageLabel={mainFooterPageLabel}
        mainFooterTransitionLabel={mainFooterTransitionLabel}
        pageTransitionState={pageTransitionState}
        itemsHasNextPage={itemsHasNextPage}
        itemsPageIndex={itemsPageIndex}
        setMainGridElement={setMainGridElement}
        onThumbnailZoomLevelChange={onThumbnailZoomLevelChange}
        onSelectAsset={onSelectAsset}
        onThumbnailError={onThumbnailError}
        onGoPreviousItemsPage={onGoPreviousItemsPage}
        onGoNextItemsPage={onGoNextItemsPage}
        onMainGridWheel={onMainGridWheel}
      />

      <div
        className={`workspace-splitter ${isRightSplitterDragging ? 'is-dragging' : ''}`}
        role="separator"
        aria-orientation="vertical"
        aria-label="调整 Main 与 Metadata 宽度"
        onPointerDown={onRightSplitterPointerDown}
      />

      <AppShellMetadataPane
        selectedLibraryId={selectedLibraryId}
        selectedLibraryDetail={selectedLibraryDetail}
        selectedItemDetail={selectedItemDetail}
        scanStateLabel={scanStateLabel}
        scanStateData={scanStateData}
        scanSummary={scanSummary}
        scanActiveSourceCount={scanStats?.activeSourceCount ?? 0}
        scanMissingSourceCount={scanStats?.missingSourceCount ?? 0}
        importBusy={importBusy}
        actionPendingLabel={actionPendingLabel}
        workspaceError={workspaceError}
        itemDetailLoading={itemDetailLoading}
        itemDetailError={itemDetailError}
      />
    </div>
  )
}
