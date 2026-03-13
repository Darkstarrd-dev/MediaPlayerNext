import type { PointerEvent as ReactPointerEvent } from 'react'
import { useState } from 'react'
import { AppShellHeader } from './app-shell-header'
import { AppShellPanels } from './app-shell-panels'
import type { AppShellThemeDebugPage } from './app-shell-theme-debug-types'
import { AppShellWorkspace } from './app-shell-workspace'
import type { AppShellSettingsPage } from './app-shell-settings-types'
import { useAppShellDatabaseSettings } from './use-app-shell-database-settings'
import { useAppShellLayout, type DragTarget } from './use-app-shell-layout'
import { resolveThumbnailProfileForGrid } from './thumbnail-grid-enhancements'
import { useAppShellImportActivities } from './use-app-shell-import-activities'
import { useAppShellImportController } from './use-app-shell-import-controller'
import { useAppShellOverlayEscape } from './use-app-shell-overlay-escape'
import { useAppShellScanState } from './use-app-shell-scan-state'
import { useAppShellViewState } from './use-app-shell-view-state'
import { useAppShellWorkspaceCursor } from './use-app-shell-workspace-cursor'
import { useAppShellWorkspaceData } from './use-app-shell-workspace-data'
import { useAppShellWorkspaceNavigation } from './use-app-shell-workspace-navigation'
import { useAppShellWorkspaceSelection } from './use-app-shell-workspace-selection'
import { useAppShellWorkspaceState } from './use-app-shell-workspace-state'
import { useAppShellItemData } from './use-app-shell-item-data'
import { useMediaRepository } from './use-media-repository'
import type { SidebarLabelDisplayMode } from './sidebar-main-image-tree'

export function AppShell() {
  const repository = useMediaRepository()
  const [importTaskPanelOpen, setImportTaskPanelOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
  const [themeDebugOpen, setThemeDebugOpen] = useState(false)
  const [sidebarLabelDisplayMode, setSidebarLabelDisplayMode] = useState<SidebarLabelDisplayMode>('full')
  const [settingsPage, setSettingsPage] = useState<AppShellSettingsPage>('ui')
  const [themeDebugPage, setThemeDebugPage] = useState<AppShellThemeDebugPage>('snapshot')
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
  const thumbnailProfile = resolveThumbnailProfileForGrid(thumbnailGridLayout.cellSizePx)

  const {
    libraryLoadRequestIdRef,
    itemDetailRequestIdRef,
    libraries,
    setLibraries,
    librariesLoading,
    setLibrariesLoading,
    selectedLibraryId,
    setSelectedLibraryId,
    sidebarNodes,
    setSidebarNodes,
    sidebarNodesLoading,
    setSidebarNodesLoading,
    selectedSidebarNodeId,
    setSelectedSidebarNodeId,
    selectedMediaSourceId,
    setSelectedMediaSourceId,
    selectedLibraryDetail,
    setSelectedLibraryDetail,
    scanStats,
    setScanStats,
    scanSnapshot,
    setScanSnapshot,
    workspaceRefreshing,
    setWorkspaceRefreshing,
    workspaceHydrated,
    setWorkspaceHydrated,
    workspaceError,
    setWorkspaceError,
    items,
    setItems,
    itemsPageIndex,
    setItemsPageIndex,
    itemsTargetPageIndex,
    setItemsTargetPageIndex,
    itemsHasNextPage,
    setItemsHasNextPage,
    itemsPageTransitionState,
    setItemsPageTransitionState,
    selectedAssetId,
    setSelectedAssetId,
    selectedItemDetail,
    setSelectedItemDetail,
    itemDetailLoading,
    setItemDetailLoading,
    itemDetailError,
    setItemDetailError,
    itemThumbnailUrls,
    setItemThumbnailUrls,
    clearWorkspaceData,
  } = useAppShellWorkspaceState()
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
      thumbnailProfile,
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
      setItemsTargetPageIndex,
      setItemsHasNextPage,
      setItemsPageTransitionState,
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
  useAppShellOverlayEscape({
    settingsOpen,
    themeDebugOpen,
    importTaskPanelOpen,
    setSettingsOpen,
    setThemeDebugOpen,
    setImportTaskPanelOpen,
  })
  useAppShellItemData({
    repository,
    itemDetailRequestIdRef,
    selectedAssetId,
    items,
    selectedLibraryId,
    selectedMediaSourceId,
    itemsPageIndex,
    itemsHasNextPage,
    thumbnailPageSize: thumbnailGridLayout.pageSize,
    thumbnailProfile,
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
  const { handleLibrarySelect, handleSidebarNodeSelect } = useAppShellWorkspaceNavigation({
    selectedLibraryId,
    sidebarNodes,
    refreshWorkspace,
    loadLibrarySurface,
    setSelectedSidebarNodeId,
    setSelectedMediaSourceId,
  })
  const handleLeftSplitterPointerDown = handleSplitterPointerDown('left')
  const handleRightSplitterPointerDown = handleSplitterPointerDown('right')
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
  const {
    clearDatabaseDialogOpen,
    runtimeInfoError,
    databaseActionError,
    databaseActionMessage,
    databasePendingLabel,
    runtimeInfoDatabasePath,
    runtimeInfoThumbnailCachePath,
    databaseActionBusy,
    handleRequestClearDatabase,
    handleCloseClearDatabaseDialog,
    handlePickDatabaseDirectory,
    handlePickThumbnailDirectory,
    handleConfirmClearDatabase,
  } = useAppShellDatabaseSettings({
    repository,
    settingsOpen,
    settingsPage,
    pickSingleDirectory,
  })
  const {
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
    mainFooterTransitionLabel,
    importActivitiesForPanel,
    handleGoPreviousItemsPage,
    handleGoNextItemsPage,
    handleMainGridWheel,
    handleItemThumbnailError,
    handleToggleImportTaskPanel,
    handleOpenSettings,
    handleOpenThemeDebug,
  } = useAppShellViewState({
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
  })
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
        <AppShellHeader
          importTaskPanelOpen={importTaskPanelOpen}
          settingsOpen={settingsOpen}
          themeDebugOpen={themeDebugOpen}
          logoLoading={logoLoading}
          logoButtonState={logoButtonState}
          onToggleImportTaskPanel={handleToggleImportTaskPanel}
          onOpenSettings={handleOpenSettings}
          onOpenThemeDebug={handleOpenThemeDebug}
        />
        <AppShellWorkspace
          workspaceStyle={workspaceStyle}
          selectedSidebarNodeId={selectedSidebarNodeId}
          selectedMediaSourceId={selectedMediaSourceId}
          isLeftSplitterDragging={dragState?.target === 'left'}
          isRightSplitterDragging={dragState?.target === 'right'}
          onLeftSplitterPointerDown={handleLeftSplitterPointerDown}
          onRightSplitterPointerDown={handleRightSplitterPointerDown}
          selectedLibrarySummary={selectedLibrarySummary}
          selectedLibraryId={selectedLibraryId}
          libraries={libraries}
          librariesLoading={librariesLoading}
          sidebarNodesLoading={sidebarNodesLoading}
          sidebarNodes={sidebarNodes}
          sidebarFooterText={sidebarFooterText}
          sidebarLabelDisplayMode={sidebarLabelDisplayMode}
          onToggleSidebarLabelDisplayMode={() => {
            setSidebarLabelDisplayMode((current) => (current === 'full' ? 'leaf' : 'full'))
          }}
          onLibrarySelect={handleLibrarySelect}
          onSidebarNodeSelect={handleSidebarNodeSelect}
          selectedLibraryDetail={selectedLibraryDetail}
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
          pageTransitionState={itemsPageTransitionState}
          mainFooterTransitionLabel={mainFooterTransitionLabel}
          itemsHasNextPage={itemsHasNextPage}
          itemsPageIndex={itemsPageIndex}
          setMainGridElement={setMainGridElement}
          onThumbnailZoomLevelChange={setThumbnailZoomLevel}
          onSelectAsset={setSelectedAssetId}
          onThumbnailError={handleItemThumbnailError}
          onGoPreviousItemsPage={handleGoPreviousItemsPage}
          onGoNextItemsPage={handleGoNextItemsPage}
          onMainGridWheel={handleMainGridWheel}
          selectedItemDetail={selectedItemDetail}
          scanStateLabel={scanStateLabel}
          scanStateData={scanSnapshot?.state ?? 'idle'}
          scanSummary={scanSummary}
          scanStats={scanStats}
          importBusy={importBusy}
          actionPendingLabel={actionPendingLabel}
          itemDetailLoading={itemDetailLoading}
          itemDetailError={itemDetailError}
        />
      </div>
      <AppShellPanels
        importTaskPanelOpen={importTaskPanelOpen}
        settingsOpen={settingsOpen}
        themeDebugOpen={themeDebugOpen}
        settingsPage={settingsPage}
        themeDebugPage={themeDebugPage}
        importRootPath={importRootPath}
        actionPendingLabel={actionPendingLabel}
        actionMessage={actionMessage}
        actionError={actionError}
        importBusy={importBusy}
        libraries={libraries}
        selectedLibraryDetail={selectedLibraryDetail}
        selectedLibrarySummary={selectedLibrarySummary}
        scanStateLabel={scanStateLabel}
        scanSummary={scanSummary}
        scanProgressPercent={scanProgressPercent}
        importActivitiesForPanel={importActivitiesForPanel}
        settingsBackdropOpacity={settingsBackdropOpacity}
        layoutGapScaleCoeff={layoutGapScaleCoeff}
        paneInnerGapScaleCoeff={paneInnerGapScaleCoeff}
        paneStackGapScaleCoeff={paneStackGapScaleCoeff}
        splitterWidthScaleCoeff={splitterWidthScaleCoeff}
        layoutGapPx={layoutPreview.layoutGapPx}
        paneInnerPaddingPx={layoutPreview.paneInnerPaddingPx}
        paneStackGapPx={layoutPreview.paneStackGapPx}
        splitterWidthPx={layoutPreview.splitterWidthPx}
        runtimeInfoError={runtimeInfoError}
        databaseActionError={databaseActionError}
        databaseActionMessage={databaseActionMessage}
        databasePendingLabel={databasePendingLabel}
        runtimeInfoDatabasePath={runtimeInfoDatabasePath}
        runtimeInfoThumbnailCachePath={runtimeInfoThumbnailCachePath}
        databaseActionBusy={databaseActionBusy}
        clearDatabaseDialogOpen={clearDatabaseDialogOpen}
        onImportTaskPanelClose={() => setImportTaskPanelOpen(false)}
        onImportRootPathChange={setImportRootPath}
        onPickDirectory={() => void handlePickDirectory()}
        onAddLibrary={() => void handleAddLibrary(false)}
        onAddAndScan={() => void handleAddLibrary(true)}
        onSettingsClose={() => setSettingsOpen(false)}
        onThemeDebugClose={() => setThemeDebugOpen(false)}
        onSettingsPageChange={setSettingsPage}
        onThemeDebugPageChange={setThemeDebugPage}
        onSettingsBackdropOpacityChange={setSettingsBackdropOpacity}
        onLayoutGapScaleCoeffChange={setLayoutGapScaleCoeff}
        onPaneInnerGapScaleCoeffChange={setPaneInnerGapScaleCoeff}
        onPaneStackGapScaleCoeffChange={setPaneStackGapScaleCoeff}
        onSplitterWidthScaleCoeffChange={setSplitterWidthScaleCoeff}
        onRequestClearDatabase={handleRequestClearDatabase}
        onPickDatabaseDirectory={() => void handlePickDatabaseDirectory()}
        onPickThumbnailDirectory={() => void handlePickThumbnailDirectory()}
        onCloseClearDatabaseDialog={handleCloseClearDatabaseDialog}
        onConfirmClearDatabase={() => void handleConfirmClearDatabase()}
      />
    </main>
  )
}
