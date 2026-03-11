interface ImportTaskPanelProps {
  open: boolean
  onClose: () => void
  importRootPath: string
  onImportRootPathChange: (value: string) => void
  onPickDirectory: () => void
  onAddLibrary: () => void
  onAddAndScan: () => void
  busy: boolean
  actionPendingLabel: string | null
  actionMessage: string | null
  actionError: string | null
  libraryCount: number
  currentLibraryPath: string | null
  scanStateLabel: string
  scanSummary: string
  scanProgressPercent: number | null
}

const IMPORT_TASK_PANEL_ITEMS = [
  {
    label: '路径登记',
    status: '已接首轮',
    detail: '当前通过本地路径登记媒体库，并可选择登记后立即扫描。',
  },
  {
    label: '扫描触发',
    status: '已接首轮',
    detail: '登记后可立即触发扫描，扫描完成后刷新主界面三列快照。',
  },
  {
    label: '主界面回流',
    status: '首轮可用',
    detail: '导入或扫描完成后，Sidebar、Main、Metadata 会基于当前库重新读取。',
  },
] as const

export function ImportTaskPanel({
  open,
  onClose,
  importRootPath,
  onImportRootPathChange,
  onPickDirectory,
  onAddLibrary,
  onAddAndScan,
  busy,
  actionPendingLabel,
  actionMessage,
  actionError,
  libraryCount,
  currentLibraryPath,
  scanStateLabel,
  scanSummary,
  scanProgressPercent,
}: ImportTaskPanelProps) {
  if (!open) {
    return null
  }

  return (
    <div
      className="settings-mask import-task-overlay"
      data-slot="fg-import-task-ovl"
      data-overlay-close="import-task-panel"
      onClick={onClose}
    >
      <section
        id="import-task-panel"
        className="mpx-large-panel import-task-panel"
        data-slot="fg-import-task-root"
        data-overlay-close="import-task-panel"
        role="dialog"
        aria-modal="true"
        aria-labelledby="import-task-panel-title"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="mpx-large-panel-head import-task-panel-head">
          <div className="mpx-large-panel-head-spacer" aria-hidden="true" />
          <h2 id="import-task-panel-title">导入任务</h2>
          <button className="mpx-btn import-task-panel-close-btn" type="button" onClick={onClose}>
            关闭
          </button>
        </header>

        <div className="mpx-large-panel-shell is-no-side import-task-panel-shell">
          <section className="mpx-large-panel-main import-task-panel-main" role="status" aria-live="polite">
            <div className="import-task-panel-summary">
              <article className="import-task-panel-card">
                <span className="workspace-label">媒体库数量</span>
                <strong>{libraryCount}</strong>
                <p>导入入口不单独生成结果页，完成后会把最新结果回流到主界面三列。</p>
              </article>
              <article className="import-task-panel-card">
                <span className="workspace-label">当前聚焦</span>
                <strong>{currentLibraryPath ?? '未选择媒体库'}</strong>
                <p>导入或扫描成功后，当前媒体库会成为 Sidebar、Main、Metadata 的刷新依据。</p>
              </article>
              <article className="import-task-panel-card">
                <span className="workspace-label">扫描状态</span>
                <strong>{scanStateLabel}</strong>
                <p>{scanSummary}</p>
                {scanProgressPercent === null ? null : (
                  <div className="progress-track" aria-hidden="true">
                    <div className="progress-bar" style={{ width: `${scanProgressPercent}%` }} />
                  </div>
                )}
              </article>
            </div>

            <section className="import-task-panel-section">
              <div className="panel-heading import-task-panel-heading">
                <div>
                  <span className="section-kicker">Import</span>
                  <h2>导入入口</h2>
                </div>
              </div>

              <div className="import-task-panel-form">
                <label className="field">
                  <span>本地路径</span>
                  <input
                    type="text"
                    value={importRootPath}
                    placeholder="例如：D:\\Media\\Gallery"
                    onChange={(event) => onImportRootPathChange(event.target.value)}
                  />
                </label>

                <div className="actions import-task-panel-actions">
                  <button className="mpx-btn" type="button" onClick={onPickDirectory} disabled={busy}>
                    选择文件夹
                  </button>
                  <button className="mpx-btn" type="button" onClick={onAddLibrary} disabled={busy}>
                    登记媒体库
                  </button>
                  <button className="mpx-btn" type="button" onClick={onAddAndScan} disabled={busy}>
                    登记并扫描
                  </button>
                </div>
              </div>
            </section>

            <section className="import-task-panel-section">
              <div className="panel-heading import-task-panel-heading">
                <div>
                  <span className="section-kicker">Pipeline</span>
                  <h2>当前链路</h2>
                </div>
              </div>

              <div className="import-task-panel-grid">
                {IMPORT_TASK_PANEL_ITEMS.map((item) => (
                  <article key={item.label} className="import-task-panel-card compact">
                    <span className="workspace-label">{item.label}</span>
                    <strong>{item.status}</strong>
                    <p>{item.detail}</p>
                  </article>
                ))}
              </div>
            </section>

            {actionPendingLabel === null ? null : (
              <section className="import-task-panel-empty">
                <span className="workspace-label">当前动作</span>
                <strong>{actionPendingLabel}</strong>
                <p>
                  {busy
                    ? '正在执行当前导入或刷新动作，请等待主界面快照同步完成。'
                    : '当前动作已经结束，如有需要可继续下一次导入或刷新。'}
                </p>
              </section>
            )}

            {actionMessage === null ? null : (
              <section className="import-task-panel-results">
                <article className="result-card">
                  <span className="workspace-label">最近结果</span>
                  <strong>{actionPendingLabel ?? '已完成'}</strong>
                  <p>{actionMessage}</p>
                </article>
              </section>
            )}

            {actionError === null ? null : <div className="error-text">{actionError}</div>}
          </section>
        </div>
      </section>
    </div>
  )
}
