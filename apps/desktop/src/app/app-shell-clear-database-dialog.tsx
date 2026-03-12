import type { AppShellDatabaseActionKind } from './app-shell-settings-types'

interface AppShellClearDatabaseDialogProps {
  open: boolean
  databaseActionBusy: AppShellDatabaseActionKind | null
  databaseActionError: string | null
  onClose: () => void
  onConfirm: () => void
}

export function AppShellClearDatabaseDialog({
  open,
  databaseActionBusy,
  databaseActionError,
  onClose,
  onConfirm,
}: AppShellClearDatabaseDialogProps) {
  if (!open) {
    return null
  }

  return (
    <div
      className="settings-subdialog-overlay"
      onClick={onClose}
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
            onClick={onClose}
            disabled={databaseActionBusy === 'clearDatabase'}
          >
            取消
          </button>
          <button
            className="mpx-btn is-danger"
            type="button"
            data-testid="database-clear-confirm"
            onClick={onConfirm}
            disabled={databaseActionBusy === 'clearDatabase'}
          >
            {databaseActionBusy === 'clearDatabase' ? '清除中...' : '确认清除'}
          </button>
        </div>
      </section>
    </div>
  )
}
