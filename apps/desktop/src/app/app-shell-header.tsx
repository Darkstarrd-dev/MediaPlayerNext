import { SettingsIcon } from './SettingsIcon'

interface AppShellHeaderProps {
  importTaskPanelOpen: boolean
  settingsOpen: boolean
  logoLoading: boolean
  logoButtonState: string
  onToggleImportTaskPanel: () => void
  onOpenSettings: () => void
}

export function AppShellHeader({
  importTaskPanelOpen,
  settingsOpen,
  logoLoading,
  logoButtonState,
  onToggleImportTaskPanel,
  onOpenSettings,
}: AppShellHeaderProps) {
  return (
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
            onClick={onToggleImportTaskPanel}
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
            onClick={onOpenSettings}
          >
            <SettingsIcon className="settings-trigger-icon" />
            <span className="settings-trigger-label">设置</span>
          </button>
        </div>
      </div>
    </header>
  )
}
