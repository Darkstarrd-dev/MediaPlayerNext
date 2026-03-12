import type { AppShellDatabaseActionKind, AppShellSettingsPage } from './app-shell-settings-types'
import { AppShellSettingsDatabasePage } from './app-shell-settings-database-page'
import { AppShellSettingsUiPage } from './app-shell-settings-ui-page'

interface AppShellSettingsPanelProps {
  open: boolean
  settingsPage: AppShellSettingsPage
  onClose: () => void
  onSettingsPageChange: (page: AppShellSettingsPage) => void
  settingsBackdropOpacity: number
  layoutGapScaleCoeff: number
  paneInnerGapScaleCoeff: number
  paneStackGapScaleCoeff: number
  splitterWidthScaleCoeff: number
  layoutGapPx: number
  paneInnerPaddingPx: number
  paneStackGapPx: number
  splitterWidthPx: number
  onSettingsBackdropOpacityChange: (value: number) => void
  onLayoutGapScaleCoeffChange: (value: number) => void
  onPaneInnerGapScaleCoeffChange: (value: number) => void
  onPaneStackGapScaleCoeffChange: (value: number) => void
  onSplitterWidthScaleCoeffChange: (value: number) => void
  runtimeInfoError: string | null
  databaseActionError: string | null
  databaseActionMessage: string | null
  databasePendingLabel: string | null
  runtimeInfoDatabasePath: string
  runtimeInfoThumbnailCachePath: string
  databaseActionBusy: AppShellDatabaseActionKind | null
  onRequestClearDatabase: () => void
  onPickDatabaseDirectory: () => void
  onPickThumbnailDirectory: () => void
}

export function AppShellSettingsPanel({
  open,
  settingsPage,
  onClose,
  onSettingsPageChange,
  settingsBackdropOpacity,
  layoutGapScaleCoeff,
  paneInnerGapScaleCoeff,
  paneStackGapScaleCoeff,
  splitterWidthScaleCoeff,
  layoutGapPx,
  paneInnerPaddingPx,
  paneStackGapPx,
  splitterWidthPx,
  onSettingsBackdropOpacityChange,
  onLayoutGapScaleCoeffChange,
  onPaneInnerGapScaleCoeffChange,
  onPaneStackGapScaleCoeffChange,
  onSplitterWidthScaleCoeffChange,
  runtimeInfoError,
  databaseActionError,
  databaseActionMessage,
  databasePendingLabel,
  runtimeInfoDatabasePath,
  runtimeInfoThumbnailCachePath,
  databaseActionBusy,
  onRequestClearDatabase,
  onPickDatabaseDirectory,
  onPickThumbnailDirectory,
}: AppShellSettingsPanelProps) {
  if (!open) {
    return null
  }

  return (
    <div className="settings-mask" onClick={onClose}>
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
          <button className="mpx-btn settings-close-btn" type="button" onClick={onClose}>
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
              onClick={() => onSettingsPageChange('ui')}
            >
              界面设置
            </button>
            <button
              className={`mpx-btn ${settingsPage === 'database' ? 'is-active' : ''}`}
              type="button"
              data-testid="settings-page-database"
              aria-pressed={settingsPage === 'database'}
              onClick={() => onSettingsPageChange('database')}
            >
              数据库管理
            </button>
          </aside>

          <section className="mpx-large-panel-main settings-panel-main">
            {settingsPage === 'ui' ? (
              <AppShellSettingsUiPage
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
              />
            ) : (
              <AppShellSettingsDatabasePage
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
            )}
          </section>
        </div>
      </section>
    </div>
  )
}
