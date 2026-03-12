import type { ChangeEvent, RefObject } from 'react'
import { useEffect, useRef, useState } from 'react'
import type { AppShellThemeDebugPage } from './app-shell-theme-debug-types'

type ThemeDebugCssVarKey =
  | '--mpx-bg-app-fill'
  | '--mpx-container-frame-fill-start'
  | '--mpx-container-frame-fill-end'
  | '--mpx-container-frame-fill-angle'
  | '--mpx-container-frame-border-color'
  | '--mpx-container-frame-edge-color'
  | '--mpx-container-frame-shadow'
  | '--mpx-container-frame-radius'
  | '--mpx-slot-fg-header-root-bg'
  | '--mpx-slot-fg-header-root-border'
  | '--mpx-slot-fg-header-root-shadow'
  | '--mpx-slot-fg-sidebar-root-bg'
  | '--mpx-slot-fg-sidebar-root-border'
  | '--mpx-slot-fg-sidebar-root-shadow'
  | '--mpx-slot-fg-main-root-bg'
  | '--mpx-slot-fg-main-root-border'
  | '--mpx-slot-fg-main-root-shadow'
  | '--mpx-slot-fg-meta-root-bg'
  | '--mpx-slot-fg-meta-root-border'
  | '--mpx-slot-fg-meta-root-shadow'

interface ThemeDebugFieldDefinition {
  cssVar: ThemeDebugCssVarKey
  label: string
  hint: string
  placeholder: string
}

interface ThemeDebugSectionDefinition {
  sectionId: '1.0' | '2.0' | '2.1' | '2.2' | '2.3' | '2.4'
  title: string
  description: string
  fields: readonly ThemeDebugFieldDefinition[]
  includeLayoutCoefficients?: boolean
}

interface ThemeDebugSnapshotPayload {
  version: '1.0'
  scope: '1.0-2.4-root'
  exportedAt: string
  values: Record<string, string | number>
}

interface AppShellThemeDebugPanelProps {
  open: boolean
  themeDebugPage: AppShellThemeDebugPage
  layoutGapScaleCoeff: number
  splitterWidthScaleCoeff: number
  onClose: () => void
  onThemeDebugPageChange: (page: AppShellThemeDebugPage) => void
  onLayoutGapScaleCoeffChange: (value: number) => void
  onSplitterWidthScaleCoeffChange: (value: number) => void
}

const SNAPSHOT_KEY_LAYOUT_GAP_SCALE_COEFF = 'layoutGapScaleCoeff'
const SNAPSHOT_KEY_SPLITTER_WIDTH_SCALE_COEFF = 'splitterWidthScaleCoeff'

const THEME_DEBUG_SECTIONS: readonly ThemeDebugSectionDefinition[] = [
  {
    sectionId: '1.0',
    title: '1.0 背景层',
    description: '当前只维护应用根背景填充变量。',
    fields: [
      {
        cssVar: '--mpx-bg-app-fill',
        label: 'App 背景填充',
        hint: '对应 `--mpx-bg-app-fill`。',
        placeholder: '#ebe3d3',
      },
    ],
  },
  {
    sectionId: '2.0',
    title: '2.0 共享壳层',
    description: '四大容器共享 frame 基架；布局项沿用系数链路。',
    includeLayoutCoefficients: true,
    fields: [
      {
        cssVar: '--mpx-container-frame-fill-start',
        label: 'Frame 渐变起点',
        hint: '对应 `--mpx-container-frame-fill-start`。',
        placeholder: 'rgba(255, 250, 242, 0.96)',
      },
      {
        cssVar: '--mpx-container-frame-fill-end',
        label: 'Frame 渐变终点',
        hint: '对应 `--mpx-container-frame-fill-end`。',
        placeholder: 'rgba(244, 235, 223, 0.94)',
      },
      {
        cssVar: '--mpx-container-frame-fill-angle',
        label: 'Frame 渐变角度',
        hint: '对应 `--mpx-container-frame-fill-angle`。',
        placeholder: '180deg',
      },
      {
        cssVar: '--mpx-container-frame-border-color',
        label: 'Frame 边框颜色',
        hint: '对应 `--mpx-container-frame-border-color`。',
        placeholder: 'rgba(119, 99, 71, 0.18)',
      },
      {
        cssVar: '--mpx-container-frame-edge-color',
        label: 'Frame 内缘颜色',
        hint: '对应 `--mpx-container-frame-edge-color`。',
        placeholder: 'rgba(255, 255, 255, 0.82)',
      },
      {
        cssVar: '--mpx-container-frame-shadow',
        label: 'Frame 阴影',
        hint: '对应 `--mpx-container-frame-shadow`。',
        placeholder: '0 0 0 1px rgba(255, 255, 255, 0.44), 0 20px 50px rgba(92, 71, 45, 0.14)',
      },
      {
        cssVar: '--mpx-container-frame-radius',
        label: 'Frame 圆角',
        hint: '对应 `--mpx-container-frame-radius`。',
        placeholder: '26px',
      },
    ],
  },
  {
    sectionId: '2.1',
    title: '2.1 Header Root',
    description: 'Header 根容器 slot 覆写变量。',
    fields: [
      {
        cssVar: '--mpx-slot-fg-header-root-bg',
        label: 'Header Root 背景',
        hint: '对应 `--mpx-slot-fg-header-root-bg`。',
        placeholder: 'var(--mpx-container-frame-fill)',
      },
      {
        cssVar: '--mpx-slot-fg-header-root-border',
        label: 'Header Root 边框',
        hint: '对应 `--mpx-slot-fg-header-root-border`。',
        placeholder: 'var(--mpx-container-frame-border-color)',
      },
      {
        cssVar: '--mpx-slot-fg-header-root-shadow',
        label: 'Header Root 阴影',
        hint: '对应 `--mpx-slot-fg-header-root-shadow`。',
        placeholder: 'var(--mpx-container-frame-shadow)',
      },
    ],
  },
  {
    sectionId: '2.2',
    title: '2.2 Sidebar Root',
    description: 'Sidebar 根容器 slot 覆写变量。',
    fields: [
      {
        cssVar: '--mpx-slot-fg-sidebar-root-bg',
        label: 'Sidebar Root 背景',
        hint: '对应 `--mpx-slot-fg-sidebar-root-bg`。',
        placeholder: 'var(--mpx-container-frame-fill)',
      },
      {
        cssVar: '--mpx-slot-fg-sidebar-root-border',
        label: 'Sidebar Root 边框',
        hint: '对应 `--mpx-slot-fg-sidebar-root-border`。',
        placeholder: 'var(--mpx-container-frame-border-color)',
      },
      {
        cssVar: '--mpx-slot-fg-sidebar-root-shadow',
        label: 'Sidebar Root 阴影',
        hint: '对应 `--mpx-slot-fg-sidebar-root-shadow`。',
        placeholder: 'var(--mpx-container-frame-shadow)',
      },
    ],
  },
  {
    sectionId: '2.3',
    title: '2.3 Main Root',
    description: 'Main 根容器 slot 覆写变量。',
    fields: [
      {
        cssVar: '--mpx-slot-fg-main-root-bg',
        label: 'Main Root 背景',
        hint: '对应 `--mpx-slot-fg-main-root-bg`。',
        placeholder: 'var(--mpx-container-frame-fill)',
      },
      {
        cssVar: '--mpx-slot-fg-main-root-border',
        label: 'Main Root 边框',
        hint: '对应 `--mpx-slot-fg-main-root-border`。',
        placeholder: 'var(--mpx-container-frame-border-color)',
      },
      {
        cssVar: '--mpx-slot-fg-main-root-shadow',
        label: 'Main Root 阴影',
        hint: '对应 `--mpx-slot-fg-main-root-shadow`。',
        placeholder: 'var(--mpx-container-frame-shadow)',
      },
    ],
  },
  {
    sectionId: '2.4',
    title: '2.4 Metadata Root',
    description: 'Metadata 根容器 slot 覆写变量。',
    fields: [
      {
        cssVar: '--mpx-slot-fg-meta-root-bg',
        label: 'Metadata Root 背景',
        hint: '对应 `--mpx-slot-fg-meta-root-bg`。',
        placeholder: 'var(--mpx-container-frame-fill)',
      },
      {
        cssVar: '--mpx-slot-fg-meta-root-border',
        label: 'Metadata Root 边框',
        hint: '对应 `--mpx-slot-fg-meta-root-border`。',
        placeholder: 'var(--mpx-container-frame-border-color)',
      },
      {
        cssVar: '--mpx-slot-fg-meta-root-shadow',
        label: 'Metadata Root 阴影',
        hint: '对应 `--mpx-slot-fg-meta-root-shadow`。',
        placeholder: 'var(--mpx-container-frame-shadow)',
      },
    ],
  },
]

const THEME_DEBUG_CSS_KEYS: readonly ThemeDebugCssVarKey[] = THEME_DEBUG_SECTIONS.flatMap((section) =>
  section.fields.map((field) => field.cssVar),
)

const THEME_DEBUG_DEFAULT_VALUES: Record<ThemeDebugCssVarKey, string> = {
  '--mpx-bg-app-fill': '',
  '--mpx-container-frame-fill-start': '',
  '--mpx-container-frame-fill-end': '',
  '--mpx-container-frame-fill-angle': '',
  '--mpx-container-frame-border-color': '',
  '--mpx-container-frame-edge-color': '',
  '--mpx-container-frame-shadow': '',
  '--mpx-container-frame-radius': '',
  '--mpx-slot-fg-header-root-bg': '',
  '--mpx-slot-fg-header-root-border': '',
  '--mpx-slot-fg-header-root-shadow': '',
  '--mpx-slot-fg-sidebar-root-bg': '',
  '--mpx-slot-fg-sidebar-root-border': '',
  '--mpx-slot-fg-sidebar-root-shadow': '',
  '--mpx-slot-fg-main-root-bg': '',
  '--mpx-slot-fg-main-root-border': '',
  '--mpx-slot-fg-main-root-shadow': '',
  '--mpx-slot-fg-meta-root-bg': '',
  '--mpx-slot-fg-meta-root-border': '',
  '--mpx-slot-fg-meta-root-shadow': '',
}

function clampNumber(value: number, min: number, max: number): number {
  return Math.max(min, Math.min(max, value))
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

function readRootCssValues(): Record<ThemeDebugCssVarKey, string> {
  if (typeof window === 'undefined') {
    return { ...THEME_DEBUG_DEFAULT_VALUES }
  }

  const computed = window.getComputedStyle(document.documentElement)
  const nextValues: Record<ThemeDebugCssVarKey, string> = { ...THEME_DEBUG_DEFAULT_VALUES }

  for (const cssVar of THEME_DEBUG_CSS_KEYS) {
    nextValues[cssVar] = computed.getPropertyValue(cssVar).trim()
  }

  return nextValues
}

function applyRootCssValue(cssVar: ThemeDebugCssVarKey, value: string): void {
  if (typeof window === 'undefined') {
    return
  }

  const rootStyle = document.documentElement.style
  if (value.trim().length === 0) {
    rootStyle.removeProperty(cssVar)
    return
  }

  rootStyle.setProperty(cssVar, value)
}

function resolveSnapshotValues(payload: unknown): Record<string, unknown> | null {
  if (!isRecord(payload)) {
    return null
  }

  if ('values' in payload) {
    return isRecord(payload.values) ? payload.values : null
  }

  return payload
}

function resolveSnapshotNumber(rawValue: unknown, min: number, max: number): number | null {
  if (typeof rawValue === 'number' && Number.isFinite(rawValue)) {
    return clampNumber(rawValue, min, max)
  }

  if (typeof rawValue === 'string') {
    const parsed = Number.parseFloat(rawValue)
    if (Number.isFinite(parsed)) {
      return clampNumber(parsed, min, max)
    }
  }

  return null
}

function formatSnapshotJson(payload: ThemeDebugSnapshotPayload): string {
  return JSON.stringify(payload, null, 2)
}

function createSnapshotFileName(): string {
  const dateText = new Date().toISOString().replace(/[:.]/g, '-').slice(0, 19)
  return `theme-debug-1.0-2.4-${dateText}.json`
}

function toTestIdSuffix(value: string): string {
  return value.replace(/[^a-zA-Z0-9]+/g, '-').replace(/^-+|-+$/g, '').toLowerCase()
}

function buildSnapshotPayload(layoutGapScaleCoeff: number, splitterWidthScaleCoeff: number): ThemeDebugSnapshotPayload {
  const values: Record<string, string | number> = {}
  const cssValues = readRootCssValues()

  for (const cssVar of THEME_DEBUG_CSS_KEYS) {
    values[cssVar] = cssValues[cssVar]
  }

  values[SNAPSHOT_KEY_LAYOUT_GAP_SCALE_COEFF] = layoutGapScaleCoeff
  values[SNAPSHOT_KEY_SPLITTER_WIDTH_SCALE_COEFF] = splitterWidthScaleCoeff

  return {
    version: '1.0',
    scope: '1.0-2.4-root',
    exportedAt: new Date().toISOString(),
    values,
  }
}

function triggerFileDownload(fileName: string, content: string): void {
  const blob = new Blob([content], { type: 'application/json;charset=utf-8' })
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = fileName
  anchor.click()
  URL.revokeObjectURL(url)
}

function ThemeDebugPageButton({
  page,
  currentPage,
  label,
  onThemeDebugPageChange,
}: {
  page: AppShellThemeDebugPage
  currentPage: AppShellThemeDebugPage
  label: string
  onThemeDebugPageChange: (page: AppShellThemeDebugPage) => void
}) {
  return (
    <button
      className={`mpx-btn ${currentPage === page ? 'is-active' : ''}`}
      type="button"
      aria-pressed={currentPage === page}
      data-testid={`theme-debug-tab-${page}`}
      onClick={() => onThemeDebugPageChange(page)}
    >
      {label}
    </button>
  )
}

function ThemeDebugSnapshotActions({
  fileInputRef,
  onExportToTextarea,
  onDownload,
  onCopy,
  onLoadFromFile,
  onApplyImport,
  onClear,
}: {
  fileInputRef: RefObject<HTMLInputElement | null>
  onExportToTextarea: () => void
  onDownload: () => void
  onCopy: () => void
  onLoadFromFile: (event: ChangeEvent<HTMLInputElement>) => void
  onApplyImport: () => void
  onClear: () => void
}) {
  return (
    <>
      <input
        ref={fileInputRef}
        className="theme-debug-file-input"
        type="file"
        accept="application/json,.json"
        onChange={onLoadFromFile}
      />

      <div className="theme-debug-action-row">
        <button className="mpx-btn" type="button" data-testid="theme-debug-snapshot-export" onClick={onExportToTextarea}>
          导出到文本框
        </button>
        <button className="mpx-btn" type="button" data-testid="theme-debug-snapshot-download" onClick={onDownload}>
          下载 JSON
        </button>
        <button className="mpx-btn" type="button" data-testid="theme-debug-snapshot-copy" onClick={onCopy}>
          复制 JSON
        </button>
        <button className="mpx-btn" type="button" data-testid="theme-debug-snapshot-load" onClick={() => fileInputRef.current?.click()}>
          加载 JSON 文件
        </button>
        <button className="mpx-btn" type="button" data-testid="theme-debug-snapshot-apply" onClick={onApplyImport}>
          应用导入
        </button>
        <button className="mpx-btn" type="button" data-testid="theme-debug-snapshot-clear" onClick={onClear}>
          清空文本框
        </button>
      </div>
    </>
  )
}

export function AppShellThemeDebugPanel({
  open,
  themeDebugPage,
  layoutGapScaleCoeff,
  splitterWidthScaleCoeff,
  onClose,
  onThemeDebugPageChange,
  onLayoutGapScaleCoeffChange,
  onSplitterWidthScaleCoeffChange,
}: AppShellThemeDebugPanelProps) {
  const [cssValues, setCssValues] = useState<Record<ThemeDebugCssVarKey, string>>(() => ({
    ...THEME_DEBUG_DEFAULT_VALUES,
  }))
  const [snapshotText, setSnapshotText] = useState('')
  const [snapshotMessage, setSnapshotMessage] = useState<string | null>(null)
  const [snapshotError, setSnapshotError] = useState<string | null>(null)
  const fileInputRef = useRef<HTMLInputElement | null>(null)

  useEffect(() => {
    if (!open) {
      return
    }

    setCssValues(readRootCssValues())
  }, [open])

  const containerLayerSections = THEME_DEBUG_SECTIONS

  if (!open) {
    return null
  }

  function setSnapshotFeedback(message: string | null, error: string | null): void {
    setSnapshotMessage(message)
    setSnapshotError(error)
  }

  function handleCssValueChange(cssVar: ThemeDebugCssVarKey, value: string): void {
    setCssValues((current) => ({
      ...current,
      [cssVar]: value,
    }))
    applyRootCssValue(cssVar, value)
  }

  function handleResetSection(section: ThemeDebugSectionDefinition): void {
    for (const field of section.fields) {
      applyRootCssValue(field.cssVar, '')
    }

    setCssValues(readRootCssValues())
    setSnapshotFeedback(`已清空 ${section.title} 的覆盖项。`, null)
  }

  function handleExportToTextarea(): void {
    const payload = buildSnapshotPayload(layoutGapScaleCoeff, splitterWidthScaleCoeff)
    setSnapshotText(formatSnapshotJson(payload))
    setSnapshotFeedback('已生成 1.0~2.4 根级快照。', null)
  }

  function handleDownloadSnapshot(): void {
    const payload = buildSnapshotPayload(layoutGapScaleCoeff, splitterWidthScaleCoeff)
    triggerFileDownload(createSnapshotFileName(), formatSnapshotJson(payload))
    setSnapshotFeedback('已下载快照文件。', null)
  }

  async function handleCopySnapshot(): Promise<void> {
    const text = snapshotText.trim().length > 0
      ? snapshotText
      : formatSnapshotJson(buildSnapshotPayload(layoutGapScaleCoeff, splitterWidthScaleCoeff))

    try {
      await navigator.clipboard.writeText(text)
      setSnapshotText(text)
      setSnapshotFeedback('已复制快照 JSON。', null)
    } catch {
      setSnapshotFeedback(null, '复制失败，请检查当前环境的剪贴板权限。')
    }
  }

  async function handleLoadSnapshotFile(event: ChangeEvent<HTMLInputElement>): Promise<void> {
    const [file] = event.target.files ?? []
    if (file === undefined) {
      return
    }

    try {
      const fileText = await file.text()
      setSnapshotText(fileText)
      setSnapshotFeedback(`已加载文件：${file.name}`, null)
    } catch {
      setSnapshotFeedback(null, '文件读取失败，请确认 JSON 文件可访问。')
    } finally {
      event.target.value = ''
    }
  }

  function handleApplyImportSnapshot(): void {
    const trimmedText = snapshotText.trim()
    if (trimmedText.length === 0) {
      setSnapshotFeedback(null, '当前文本框为空，无法导入。')
      return
    }

    let parsedPayload: unknown
    try {
      parsedPayload = JSON.parse(trimmedText)
    } catch {
      setSnapshotFeedback(null, 'JSON 解析失败，请检查格式。')
      return
    }

    const values = resolveSnapshotValues(parsedPayload)
    if (values === null) {
      setSnapshotFeedback(null, '导入内容缺少可识别的 `values` 对象。')
      return
    }

    let appliedCount = 0
    for (const cssVar of THEME_DEBUG_CSS_KEYS) {
      if (!(cssVar in values)) {
        continue
      }

      const rawValue = values[cssVar]
      if (typeof rawValue !== 'string' && typeof rawValue !== 'number') {
        continue
      }

      applyRootCssValue(cssVar, String(rawValue))
      appliedCount += 1
    }

    const nextLayoutGapScaleCoeff = resolveSnapshotNumber(values[SNAPSHOT_KEY_LAYOUT_GAP_SCALE_COEFF], 0, 3)
    if (nextLayoutGapScaleCoeff !== null) {
      onLayoutGapScaleCoeffChange(nextLayoutGapScaleCoeff)
      appliedCount += 1
    }

    const nextSplitterWidthScaleCoeff = resolveSnapshotNumber(values[SNAPSHOT_KEY_SPLITTER_WIDTH_SCALE_COEFF], 0.5, 2)
    if (nextSplitterWidthScaleCoeff !== null) {
      onSplitterWidthScaleCoeffChange(nextSplitterWidthScaleCoeff)
      appliedCount += 1
    }

    setCssValues(readRootCssValues())
    setSnapshotFeedback(`导入完成，共应用 ${appliedCount} 项。`, null)
  }

  function handleClearSnapshotText(): void {
    setSnapshotText('')
    setSnapshotFeedback('已清空导入文本框。', null)
  }

  return (
    <div className="settings-mask theme-debug-overlay" data-slot="fg-theme-debug-ovl" onClick={onClose}>
      <section
        className="mpx-large-panel theme-debug-panel"
        role="dialog"
        data-testid="theme-debug-panel"
        data-slot="fg-theme-debug-root"
        aria-modal="true"
        aria-labelledby="theme-debug-panel-title"
        onClick={(event) => event.stopPropagation()}
      >
        <header className="mpx-large-panel-head theme-debug-panel-head">
          <div className="mpx-large-panel-head-spacer" aria-hidden="true" />
          <h2 id="theme-debug-panel-title">主题调试</h2>
          <button className="mpx-btn theme-debug-close-btn" type="button" onClick={onClose}>
            关闭
          </button>
        </header>

        <div className="mpx-large-panel-shell theme-debug-panel-shell">
          <aside className="mpx-large-panel-side theme-debug-panel-side">
            <ThemeDebugPageButton
              page="snapshot"
              currentPage={themeDebugPage}
              label="参数导入导出"
              onThemeDebugPageChange={onThemeDebugPageChange}
            />
            <ThemeDebugPageButton
              page="containerLayer"
              currentPage={themeDebugPage}
              label="大容器层调试"
              onThemeDebugPageChange={onThemeDebugPageChange}
            />
          </aside>

          <section className="mpx-large-panel-main theme-debug-panel-main">
            {themeDebugPage === 'snapshot' ? (
              <div className="theme-debug-page-block" data-testid="theme-debug-page-snapshot">
                <div className="panel-heading settings-page-heading">
                  <div>
                    <span className="section-kicker">Snapshot</span>
                    <h2>参数导入导出</h2>
                  </div>
                </div>

                <p className="settings-page-caption">
                  当前仅导入导出 `1.0~2.4` 根级字段，不处理 `2.4.1` 这类子级条目。布局项使用系数：
                  `layoutGapScaleCoeff` 与 `splitterWidthScaleCoeff`。
                </p>

                <ThemeDebugSnapshotActions
                  fileInputRef={fileInputRef}
                  onExportToTextarea={handleExportToTextarea}
                  onDownload={handleDownloadSnapshot}
                  onCopy={() => void handleCopySnapshot()}
                  onLoadFromFile={(event) => void handleLoadSnapshotFile(event)}
                  onApplyImport={handleApplyImportSnapshot}
                  onClear={handleClearSnapshotText}
                />

                <label className="theme-debug-textarea-wrap">
                  <span className="workspace-label">JSON</span>
                  <textarea
                    className="theme-debug-textarea"
                    data-testid="theme-debug-snapshot-textarea"
                    value={snapshotText}
                    placeholder="在此粘贴或加载 1.0~2.4 根级快照 JSON"
                    onChange={(event) => setSnapshotText(event.target.value)}
                  />
                </label>

                {snapshotMessage === null ? null : (
                  <div className="result-card" data-testid="theme-debug-snapshot-message">
                    <span className="result-label">状态</span>
                    <strong>已完成</strong>
                    <p>{snapshotMessage}</p>
                  </div>
                )}

                {snapshotError === null ? null : <div className="error-text">{snapshotError}</div>}
              </div>
            ) : (
              <div className="theme-debug-page-block" data-testid="theme-debug-page-container-layer">
                <div className="panel-heading settings-page-heading">
                  <div>
                    <span className="section-kicker">Container Layer</span>
                    <h2>大容器层调试</h2>
                  </div>
                </div>

                <p className="settings-page-caption">
                  当前仅开放根级 `1.0~2.4`。在输入框中直接填写 CSS 变量值，留空表示移除当前覆盖并回退到默认链路。
                </p>

                <div className="theme-debug-section-list">
                  {containerLayerSections.map((section) => (
                    <article key={section.sectionId} className="theme-debug-section-card">
                      <div className="theme-debug-section-head">
                        <div className="theme-debug-section-title">
                          <span className="workspace-label">{section.sectionId}</span>
                          <h3>{section.title}</h3>
                        </div>
                        <button className="mpx-btn" type="button" onClick={() => handleResetSection(section)}>
                          清空本段覆盖
                        </button>
                      </div>

                      <p className="settings-page-caption">{section.description}</p>

                      {section.includeLayoutCoefficients ? (
                        <div className="theme-debug-range-grid">
                          <label className="settings-slider-field">
                            <div className="settings-slider-row">
                              <span className="settings-slider-label">容器外边界系数</span>
                              <span className="settings-slider-value">{layoutGapScaleCoeff.toFixed(2)}x</span>
                            </div>
                            <input
                              className="settings-range"
                              data-testid="theme-debug-layout-gap-scale"
                              type="range"
                              min={0}
                              max={3}
                              step={0.1}
                              value={layoutGapScaleCoeff}
                              onChange={(event) => onLayoutGapScaleCoeffChange(Number(event.target.value))}
                            />
                            <span className="settings-slider-hint">映射 `layout-padding`，与设置页保持同一系数链路。</span>
                          </label>

                          <label className="settings-slider-field">
                            <div className="settings-slider-row">
                              <span className="settings-slider-label">分割条宽度系数</span>
                              <span className="settings-slider-value">{splitterWidthScaleCoeff.toFixed(2)}x</span>
                            </div>
                            <input
                              className="settings-range"
                              data-testid="theme-debug-splitter-width-scale"
                              type="range"
                              min={0.5}
                              max={2}
                              step={0.1}
                              value={splitterWidthScaleCoeff}
                              onChange={(event) => onSplitterWidthScaleCoeffChange(Number(event.target.value))}
                            />
                            <span className="settings-slider-hint">映射 `splitter-width`，与设置页保持同一系数链路。</span>
                          </label>
                        </div>
                      ) : null}

                      <div className="theme-debug-field-grid">
                        {section.fields.map((field) => (
                          <label key={field.cssVar} className="theme-debug-field">
                            <span className="workspace-label">{field.cssVar}</span>
                            <span className="theme-debug-field-label">{field.label}</span>
                            <input
                              type="text"
                              data-testid={`theme-debug-input-${toTestIdSuffix(field.cssVar)}`}
                              value={cssValues[field.cssVar]}
                              placeholder={field.placeholder}
                              onChange={(event) => handleCssValueChange(field.cssVar, event.target.value)}
                            />
                            <span className="theme-debug-field-hint">{field.hint}</span>
                          </label>
                        ))}
                      </div>
                    </article>
                  ))}
                </div>
              </div>
            )}
          </section>
        </div>
      </section>
    </div>
  )
}
