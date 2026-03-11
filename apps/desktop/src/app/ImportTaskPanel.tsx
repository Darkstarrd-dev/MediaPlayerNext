interface ImportTaskPanelProps {
  open: boolean
  onClose: () => void
}

const IMPORT_TASK_PANEL_ITEMS = [
  {
    label: '导入文件',
    status: '待接入',
    detail: '后续从这里承接文件导入入口与任务流。',
  },
  {
    label: '导入文件夹',
    status: '待接入',
    detail: '后续从这里承接文件夹导入、扫描启动与恢复。',
  },
  {
    label: '扫描摘要',
    status: '待接入',
    detail: '后续在这里展示运行状态、进度消息与结果摘要。',
  },
] as const

export function ImportTaskPanel({ open, onClose }: ImportTaskPanelProps) {
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
                <span className="workspace-label">面板状态</span>
                <strong>已打开</strong>
                <p>当前由 Header Logo 按钮打开，可随时收起。</p>
              </article>
              <article className="import-task-panel-card">
                <span className="workspace-label">任务状态</span>
                <strong>空闲</strong>
                <p>本轮先落入口与大面板骨架，忙碌态后续再接真实状态。</p>
              </article>
              <article className="import-task-panel-card">
                <span className="workspace-label">面板定位</span>
                <strong>Large Panel</strong>
                <p>沿用大面板层骨架，后续继续承接导入与扫描链路。</p>
              </article>
            </div>

            <section className="import-task-panel-section">
              <div className="panel-heading import-task-panel-heading">
                <div>
                  <span className="section-kicker">Import</span>
                  <h2>任务入口</h2>
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

            <section className="import-task-panel-empty">
              <span className="workspace-label">当前队列</span>
              <strong>暂无进行中的导入任务</strong>
              <p>后续这里将承接任务列表、进度、消息与失败重试入口。</p>
            </section>
          </section>
        </div>
      </section>
    </div>
  )
}
