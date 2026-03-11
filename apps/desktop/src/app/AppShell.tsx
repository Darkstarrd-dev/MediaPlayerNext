import type { CSSProperties, PointerEvent as ReactPointerEvent } from 'react'
import { useEffect, useMemo, useState } from 'react'
import { ImportTaskPanel } from './ImportTaskPanel'
import { SettingsIcon } from './SettingsIcon'

const DEFAULT_VIEWPORT_WIDTH = 1280
const DEFAULT_SETTINGS_BACKDROP_OPACITY = 18
const DEFAULT_LAYOUT_GAP_SCALE_COEFF = 1
const DEFAULT_PANE_INNER_GAP_SCALE_COEFF = 1
const DEFAULT_PANE_STACK_GAP_SCALE_COEFF = 1
const DEFAULT_SPLITTER_WIDTH_SCALE_COEFF = 1
const DEFAULT_SIDEBAR_WIDTH_PX = 300
const DEFAULT_META_WIDTH_PX = 340

const SETTINGS_STORAGE_KEYS = {
  settingsBackdropOpacity: 'mpnext.ui.settingsBackdropOpacity',
  layoutGapScaleCoeff: 'mpnext.ui.layoutGapScaleCoeff',
  paneInnerGapScaleCoeff: 'mpnext.ui.paneInnerGapScaleCoeff',
  paneStackGapScaleCoeff: 'mpnext.ui.paneStackGapScaleCoeff',
  splitterWidthScaleCoeff: 'mpnext.ui.splitterWidthScaleCoeff',
  sidebarWidthPx: 'mpnext.ui.sidebarWidthPx',
  metaWidthPx: 'mpnext.ui.metaWidthPx',
} as const

type DragTarget = 'left' | 'right'

interface DragState {
  target: DragTarget
  startX: number
  startSidebarWidthPx: number
  startMetaWidthPx: number
}

function clampNumber(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}

function resolveSpacingPx(viewportWidth: number, scaleCoeff: number): number {
  return Math.max(0, Math.round(Math.max(0, viewportWidth) * 0.01 * scaleCoeff))
}

function readSessionNumber(key: string, fallback: number, min: number, max: number): number {
  if (typeof window === 'undefined') {
    return fallback
  }

  const rawValue = window.sessionStorage.getItem(key)
  if (rawValue === null) {
    return fallback
  }

  const parsedValue = Number.parseFloat(rawValue)
  if (!Number.isFinite(parsedValue)) {
    return fallback
  }

  return clampNumber(parsedValue, min, max)
}

function resolveWorkspaceWidths(
  viewportWidth: number,
  layoutPaddingPx: number,
  splitterWidthPx: number,
  preferredSidebarWidthPx: number,
  preferredMetaWidthPx: number,
) {
  const availableWidth = Math.max(0, viewportWidth - layoutPaddingPx * 2 - splitterWidthPx * 2)
  const minSidebarWidthPx = Math.min(220, Math.max(160, Math.round(availableWidth * 0.22)))
  const minMetaWidthPx = Math.min(280, Math.max(200, Math.round(availableWidth * 0.24)))
  const minMainWidthPx = Math.min(420, Math.max(280, Math.round(availableWidth * 0.34)))

  const maxSidebarWidthPx = Math.max(
    minSidebarWidthPx,
    availableWidth - minMetaWidthPx - minMainWidthPx,
  )
  const sidebarWidthPx = clampNumber(
    preferredSidebarWidthPx,
    minSidebarWidthPx,
    maxSidebarWidthPx,
  )

  const maxMetaWidthPx = Math.max(minMetaWidthPx, availableWidth - sidebarWidthPx - minMainWidthPx)
  const metaWidthPx = clampNumber(preferredMetaWidthPx, minMetaWidthPx, maxMetaWidthPx)

  const stabilizedSidebarWidthPx = clampNumber(
    sidebarWidthPx,
    minSidebarWidthPx,
    Math.max(minSidebarWidthPx, availableWidth - metaWidthPx - minMainWidthPx),
  )

  return {
    availableWidth,
    sidebarWidthPx: stabilizedSidebarWidthPx,
    metaWidthPx,
    mainWidthPx: Math.max(0, availableWidth - stabilizedSidebarWidthPx - metaWidthPx),
  }
}

export function AppShell() {
  const [importTaskPanelOpen, setImportTaskPanelOpen] = useState(false)
  const [settingsOpen, setSettingsOpen] = useState(false)
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
  }, [layoutGapScaleCoeff, paneInnerGapScaleCoeff, paneStackGapScaleCoeff, splitterWidthScaleCoeff, viewportWidth])

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
  }, [
    layoutGapScaleCoeff,
    paneInnerGapScaleCoeff,
    paneStackGapScaleCoeff,
    settingsBackdropOpacity,
    splitterWidthScaleCoeff,
    workspaceLayout.metaWidthPx,
    workspaceLayout.sidebarWidthPx,
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

  const logoButtonState = importTaskPanelOpen
    ? 'fg-header-logo-state-open'
    : 'fg-header-logo-state-idle'

  return (
    <main className="app-shell" data-slot="bg-app-root">
      <div className="app-background-layer" aria-hidden="true" />

      <div className="app-chrome">
        <header className="app-frame app-header app-header-root" data-slot="fg-header-root">
          <div className="app-header-frame">
            <div className="header-left">
              <button
                className="mpx-btn header-logo-btn"
                type="button"
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
                <span className="header-logo-label">MediaPlayerNext</span>
              </button>
            </div>

            <div className="header-right">
              <button
                className="mpx-btn header-settings-trigger"
                type="button"
                aria-haspopup="dialog"
                aria-expanded={settingsOpen}
                onClick={() => {
                  setImportTaskPanelOpen(false)
                  setSettingsOpen(true)
                }}
              >
                <SettingsIcon className="settings-trigger-icon" />
                <span className="settings-trigger-label">设置</span>
              </button>
            </div>
          </div>
        </header>

        <div className="app-workspace" style={workspaceStyle}>
          <aside className="app-frame app-sidebar-root" data-slot="fg-sidebar-root">
            <section className="workspace-pane sidebar-frame">
              <header className="workspace-pane-header sidebar-header" data-slot="fg-sidebar-header">
                <button className="mpx-btn sidebar-title-btn" type="button" aria-pressed="true">
                  Sidebar
                </button>

                <div className="workspace-pane-actions sidebar-header-actions">
                  <button className="mpx-btn pane-action-btn" type="button" disabled>
                    媒体库
                  </button>
                  <button className="mpx-btn pane-action-btn" type="button" disabled>
                    扫描
                  </button>
                </div>
              </header>

              <div className="workspace-pane-main sidebar-main-shell" data-slot="fg-sidebar-main">
                <div className="workspace-stack sidebar-tree">
                  <article className="workspace-card">
                    <span className="workspace-label">媒体库</span>
                    <strong>等待接入</strong>
                    <p>后续在这里承接媒体库选择、创建与切换。</p>
                  </article>
                  <article className="workspace-card">
                    <span className="workspace-label">扫描</span>
                    <strong>等待接入</strong>
                    <p>后续在这里承接扫描启动、恢复与状态摘要。</p>
                  </article>
                </div>
              </div>

              <footer className="workspace-pane-footer sidebar-footer" data-slot="fg-sidebar-footer">
                <span>Footer 预留，后续接入侧栏底部动作与状态。</span>
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
                <div className="pane-title-stack main-header-title">
                  <span className="section-kicker">Workspace</span>
                  <h2>Main</h2>
                </div>

                <div className="workspace-pane-actions main-header-actions">
                  <button className="mpx-btn pane-action-btn" type="button" disabled>
                    Items
                  </button>
                  <button className="mpx-btn pane-action-btn" type="button" disabled>
                    Archive
                  </button>
                </div>
              </header>

              <div className="workspace-pane-main main-pane-main" data-slot="fg-main-main">
                <div className="workspace-stage">
                  <span className="workspace-label">主工作区</span>
                  <strong>内容区待接入</strong>
                  <p>后续这里承接条目列表、归档浏览、搜索结果与主操作流程。</p>
                </div>

                <div className="workspace-stage-grid">
                  <article className="workspace-card compact">
                    <span className="workspace-label">列表</span>
                    <strong>Items</strong>
                  </article>
                  <article className="workspace-card compact">
                    <span className="workspace-label">归档</span>
                    <strong>Archive</strong>
                  </article>
                  <article className="workspace-card compact">
                    <span className="workspace-label">协议</span>
                    <strong>Protocol</strong>
                  </article>
                </div>
              </div>

              <footer className="workspace-pane-footer main-footer" data-slot="fg-main-footer">
                <div className="main-footer-meta" data-slot="fg-main-footer-meta">
                  <span>主工作区壳层已固定</span>
                  <span>Items / Archive / Protocol 待接入真实链路</span>
                </div>

                <div className="main-footer-pagination" data-slot="fg-main-footer-pagination">
                  <button className="mpx-btn pane-pagination-btn" type="button" disabled>
                    Prev
                  </button>
                  <span>0 / 0</span>
                  <button className="mpx-btn pane-pagination-btn" type="button" disabled>
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
                <div className="pane-title-stack metadata-header-title">
                  <span className="section-kicker">Details</span>
                  <h2>Metadata</h2>
                </div>

                <div className="workspace-pane-actions metadata-header-g3">
                  <button className="mpx-btn pane-action-btn" type="button" disabled>
                    摘要
                  </button>
                </div>
              </header>

              <div className="workspace-pane-main metadata-main" data-slot="fg-meta-main">
                <div className="workspace-stage compact">
                  <span className="workspace-label">详情区</span>
                  <strong>等待接入</strong>
                  <p>后续这里用于展示选中项详情、元数据和辅助状态信息。</p>
                </div>
              </div>

              <footer className="workspace-pane-footer metadata-footer" data-slot="fg-meta-footer">
                <div className="metadata-image-caption" data-slot="fg-meta-footer-caption">
                  Caption / 摘要预留，后续根据选中项显示真实内容。
                </div>
              </footer>
            </section>
          </aside>
        </div>
      </div>

      <ImportTaskPanel open={importTaskPanelOpen} onClose={() => setImportTaskPanelOpen(false)} />

      {settingsOpen ? (
        <div className="settings-mask" onClick={() => setSettingsOpen(false)}>
          <section
            className="mpx-large-panel settings-panel"
            role="dialog"
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
                <button className="mpx-btn is-active" type="button" aria-pressed="true">
                  界面设置
                </button>
              </aside>

              <section className="mpx-large-panel-main settings-panel-main">
                <div className="settings-page-block">
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
              </section>
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
