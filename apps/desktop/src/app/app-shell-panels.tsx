import type { LibraryDetail, LibrarySummary } from '@mediaplayernext/contracts'
import { AppShellClearDatabaseDialog } from './app-shell-clear-database-dialog'
import { AppShellSettingsPanel } from './app-shell-settings-panel'
import type { AppShellDatabaseActionKind, AppShellSettingsPage } from './app-shell-settings-types'
import { AppShellThemeDebugPanel } from './app-shell-theme-debug-panel'
import type { AppShellThemeDebugPage } from './app-shell-theme-debug-types'
import { ImportTaskPanel } from './ImportTaskPanel'

interface ImportActivityViewModel {
  id: string
  title: string
  source: string
  status: 'running' | 'completed' | 'failed'
  detail: string
  createdAt: string
}

interface AppShellPanelsProps {
  importTaskPanelOpen: boolean
  settingsOpen: boolean
  themeDebugOpen: boolean
  settingsPage: AppShellSettingsPage
  themeDebugPage: AppShellThemeDebugPage
  importRootPath: string
  actionPendingLabel: string | null
  actionMessage: string | null
  actionError: string | null
  importBusy: boolean
  libraries: LibrarySummary[]
  selectedLibraryDetail: LibraryDetail | null
  selectedLibrarySummary: LibrarySummary | null
  scanStateLabel: string
  scanSummary: string
  scanProgressPercent: number | null
  importActivitiesForPanel: ImportActivityViewModel[]
  settingsBackdropOpacity: number
  layoutGapScaleCoeff: number
  paneInnerGapScaleCoeff: number
  paneStackGapScaleCoeff: number
  splitterWidthScaleCoeff: number
  layoutGapPx: number
  paneInnerPaddingPx: number
  paneStackGapPx: number
  splitterWidthPx: number
  runtimeInfoError: string | null
  databaseActionError: string | null
  databaseActionMessage: string | null
  databasePendingLabel: string | null
  runtimeInfoDatabasePath: string
  runtimeInfoThumbnailCachePath: string
  databaseActionBusy: AppShellDatabaseActionKind | null
  clearDatabaseDialogOpen: boolean
  onImportTaskPanelClose: () => void
  onImportRootPathChange: (value: string) => void
  onPickDirectory: () => void
  onAddLibrary: () => void
  onAddAndScan: () => void
  onSettingsClose: () => void
  onThemeDebugClose: () => void
  onSettingsPageChange: (page: AppShellSettingsPage) => void
  onThemeDebugPageChange: (page: AppShellThemeDebugPage) => void
  onSettingsBackdropOpacityChange: (value: number) => void
  onLayoutGapScaleCoeffChange: (value: number) => void
  onPaneInnerGapScaleCoeffChange: (value: number) => void
  onPaneStackGapScaleCoeffChange: (value: number) => void
  onSplitterWidthScaleCoeffChange: (value: number) => void
  onRequestClearDatabase: () => void
  onPickDatabaseDirectory: () => void
  onPickThumbnailDirectory: () => void
  onCloseClearDatabaseDialog: () => void
  onConfirmClearDatabase: () => void
}

export function AppShellPanels(props: AppShellPanelsProps) {
  const {
    importTaskPanelOpen,
    settingsOpen,
    themeDebugOpen,
    settingsPage,
    themeDebugPage,
    importRootPath,
    actionPendingLabel,
    actionMessage,
    actionError,
    importBusy,
    libraries,
    selectedLibraryDetail,
    selectedLibrarySummary,
    scanStateLabel,
    scanSummary,
    scanProgressPercent,
    importActivitiesForPanel,
    settingsBackdropOpacity,
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    splitterWidthScaleCoeff,
    layoutGapPx,
    paneInnerPaddingPx,
    paneStackGapPx,
    splitterWidthPx,
    runtimeInfoError,
    databaseActionError,
    databaseActionMessage,
    databasePendingLabel,
    runtimeInfoDatabasePath,
    runtimeInfoThumbnailCachePath,
    databaseActionBusy,
    clearDatabaseDialogOpen,
    onImportTaskPanelClose,
    onImportRootPathChange,
    onPickDirectory,
    onAddLibrary,
    onAddAndScan,
    onSettingsClose,
    onThemeDebugClose,
    onSettingsPageChange,
    onThemeDebugPageChange,
    onSettingsBackdropOpacityChange,
    onLayoutGapScaleCoeffChange,
    onPaneInnerGapScaleCoeffChange,
    onPaneStackGapScaleCoeffChange,
    onSplitterWidthScaleCoeffChange,
    onRequestClearDatabase,
    onPickDatabaseDirectory,
    onPickThumbnailDirectory,
    onCloseClearDatabaseDialog,
    onConfirmClearDatabase,
  } = props

  return (
    <>
      <ImportTaskPanel
        open={importTaskPanelOpen}
        onClose={onImportTaskPanelClose}
        importRootPath={importRootPath}
        onImportRootPathChange={onImportRootPathChange}
        onPickDirectory={onPickDirectory}
        onAddLibrary={onAddLibrary}
        onAddAndScan={onAddAndScan}
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

      <AppShellSettingsPanel
        open={settingsOpen}
        settingsPage={settingsPage}
        onClose={onSettingsClose}
        onSettingsPageChange={onSettingsPageChange}
        settingsBackdropOpacity={settingsBackdropOpacity}
        layoutGapScaleCoeff={layoutGapScaleCoeff}
        paneInnerGapScaleCoeff={paneInnerGapScaleCoeff}
        paneStackGapScaleCoeff={paneStackGapScaleCoeff}
        splitterWidthScaleCoeff={splitterWidthScaleCoeff}
        layoutGapPx={layoutGapPx}
        paneInnerPaddingPx={paneInnerPaddingPx}
        paneStackGapPx={paneStackGapPx}
        splitterWidthPx={splitterWidthPx}
        onSettingsBackdropOpacityChange={onSettingsBackdropOpacityChange}
        onLayoutGapScaleCoeffChange={onLayoutGapScaleCoeffChange}
        onPaneInnerGapScaleCoeffChange={onPaneInnerGapScaleCoeffChange}
        onPaneStackGapScaleCoeffChange={onPaneStackGapScaleCoeffChange}
        onSplitterWidthScaleCoeffChange={onSplitterWidthScaleCoeffChange}
        runtimeInfoError={runtimeInfoError}
        databaseActionError={databaseActionError}
        databaseActionMessage={databaseActionMessage}
        databasePendingLabel={databasePendingLabel}
        runtimeInfoDatabasePath={runtimeInfoDatabasePath}
        runtimeInfoThumbnailCachePath={runtimeInfoThumbnailCachePath}
        databaseActionBusy={databaseActionBusy}
        onRequestClearDatabase={onRequestClearDatabase}
        onPickDatabaseDirectory={onPickDatabaseDirectory}
        onPickThumbnailDirectory={onPickThumbnailDirectory}
      />

      <AppShellThemeDebugPanel
        open={themeDebugOpen}
        themeDebugPage={themeDebugPage}
        layoutGapScaleCoeff={layoutGapScaleCoeff}
        splitterWidthScaleCoeff={splitterWidthScaleCoeff}
        onClose={onThemeDebugClose}
        onThemeDebugPageChange={onThemeDebugPageChange}
        onLayoutGapScaleCoeffChange={onLayoutGapScaleCoeffChange}
        onSplitterWidthScaleCoeffChange={onSplitterWidthScaleCoeffChange}
      />

      <AppShellClearDatabaseDialog
        open={clearDatabaseDialogOpen}
        databaseActionBusy={databaseActionBusy}
        databaseActionError={databaseActionError}
        onClose={onCloseClearDatabaseDialog}
        onConfirm={onConfirmClearDatabase}
      />
    </>
  )
}
