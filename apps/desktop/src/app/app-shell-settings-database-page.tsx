import type { AppShellDatabaseActionKind } from './app-shell-settings-types'

interface AppShellSettingsDatabasePageProps {
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

export function AppShellSettingsDatabasePage({
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
}: AppShellSettingsDatabasePageProps) {
  return (
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
            onClick={onRequestClearDatabase}
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
            onClick={onPickDatabaseDirectory}
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
            onClick={onPickThumbnailDirectory}
            disabled={databaseActionBusy !== null}
          >
            {databaseActionBusy === 'pickThumbnailDir' ? '保存中...' : '选择缩略图目录'}
          </button>
          <span className="settings-inline-note">只切换目录并确保存在，首轮不迁移旧缓存。</span>
        </div>
      </article>
    </div>
  )
}
