interface AppShellSettingsUiPageProps {
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
}

export function AppShellSettingsUiPage({
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
}: AppShellSettingsUiPageProps) {
  return (
    <div className="settings-page-block" data-testid="settings-page-ui-body">
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
        onChange={onSettingsBackdropOpacityChange}
      />
      <UiSettingsRangeField
        label="容器外边界系数"
        valueLabel={`${layoutGapScaleCoeff.toFixed(2)}x / ${layoutGapPx}px`}
        hint="基准为窗口宽度的 1%，当前只驱动外留白与 Header 间距。"
        min={0}
        max={3}
        step={0.1}
        value={layoutGapScaleCoeff}
        onChange={onLayoutGapScaleCoeffChange}
      />
      <UiSettingsRangeField
        label="容器内边距系数"
        valueLabel={`${paneInnerGapScaleCoeff.toFixed(2)}x / ${paneInnerPaddingPx}px`}
        hint="基准同样为窗口宽度的 1%，当前用于控制容器内部 padding。"
        min={0}
        max={2}
        step={0.1}
        value={paneInnerGapScaleCoeff}
        onChange={onPaneInnerGapScaleCoeffChange}
      />
      <UiSettingsRangeField
        label="容器内上中下间距系数"
        valueLabel={`${paneStackGapScaleCoeff.toFixed(2)}x / ${paneStackGapPx}px`}
        hint="按容器内边距的 75% 计算，仅用于控制 Sidebar、Main、Metadata 三列中 header、main、footer 之间的纵向间距。"
        min={0}
        max={2}
        step={0.1}
        value={paneStackGapScaleCoeff}
        onChange={onPaneStackGapScaleCoeffChange}
      />
      <UiSettingsRangeField
        label="分割条宽度系数"
        valueLabel={`${splitterWidthScaleCoeff.toFixed(2)}x / ${splitterWidthPx}px`}
        hint="仅控制 Sidebar/Main/Metadata 之间的分隔宽度，不影响 Header 与工作区的间距。"
        min={0.5}
        max={2}
        step={0.1}
        value={splitterWidthScaleCoeff}
        onChange={onSplitterWidthScaleCoeffChange}
      />
    </div>
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
