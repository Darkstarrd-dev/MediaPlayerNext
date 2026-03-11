# MediaPlayerX UI 已知数据记录

## 1. 文档定位

本文件只用于记录源项目当前已知的 UI / theme 信息，方便在迁移过程中查阅。

边界固定如下：

- 本文件记录源项目的文件位置、代码锚点、已知变量、命名漂移与当前判断
- 本文件不是 `MediaPlayerNext` 的 UI 定义文档
- `MediaPlayerNext` 的正式定义只写在 `docs/ui/ui-definition.md`
- 若后续发现源项目信息有变化，优先在本文件追加“已知事实”，不要直接污染新项目定义

## 2. 当前已确认的实施结论

### 2.1 方案边界

- 新项目先实现两层：
  - `1.0 背景层`
  - `2.0 共享壳层`
- 第一批只做 App 总背景，以及主页面四大容器分隔：
  - `Header`
  - `Sidebar`
  - `Main`
  - `Metadata`
- 当前不接入面板内容，不展开单容器细化样式，不追求视觉等价复刻

### 2.2 定义方式

- 先定义通用设定
- 再定义派生设定
- 未单独指定派生设定时，统一回落到通用设定
- 这套结构将作为后续 theme 系统的基础

### 2.3 当前命名决策

- 先沿用 `--mpx-*` 变量前缀
- 先不做新前缀迁移
- 先保留“通用层 -> 派生层 -> 消费层”的变量结构

### 2.4 当前编号决策

- 新项目编号固定为：
  - `3`：大面板层
  - `4`：小面板层
  - `5`：按钮层
- `4 小面板层` 在新项目中完全独立，不继承 `3 大面板层`
- `5.1 通用变体` 在新项目中作为所有按钮的默认根样式

## 3. 源项目范围说明

- 源项目工作目录：`Z:\Playground\CurrentWorking\MediaPlayerX`
- 本文件内列出的路径，除特别说明外，均以该目录为根的相对路径

## 4. 1.0 背景层已知数据

### 4.1 当前定义

- 当前只暴露 `1` 个字段：
  - `container-bg-app-fill -> --mpx-bg-app-fill`
- 字段定义位置：`src/components/theme-parameter/themeParameterPanelCatalog.ts:15`

### 4.2 面板呈现

- 所属分页：`containerLayer`
- 折叠段标题：`1.0 背景`
- 标题文案：`src/i18n/locales/zh-CN.part1.ts:373`
- 页面结构：`src/components/theme-parameter/ThemeParameterPanelPages.tsx:335`
- 渲染接线：`src/components/theme-parameter/ThemeParameterPanelMain.tsx:1519`

### 4.3 实际消费

- App 根层背景消费：
  - `src/styles/app/layout/layout.part1.css:1`
- 关键表达式：
  - `.app { background: var(--mpx-slot-bg-app-root-bg, var(--mpx-bg-app-fill)); }`

### 4.4 默认值与主题覆盖

- 合同默认值：`src/styles/themes/contract.css:163`
- 当前样式主题覆盖：`src/styles/themes/styles/soft-skeuomorphic.css:117`
- 当前调色板覆盖：`src/styles/themes/palettes/soft-skeuomorphic/skeuomorphic-luxury-white.css:39`

### 4.5 文档记录

- 当前追踪说明：`docs/32-ui-design-tracking-v1.md:3`
- 稳定路径归属：`docs/10-ui_definition.md:31`

### 4.6 已知命名漂移

- `docs/32-ui-design-tracking-v1.md:17` 仍写旧名 `--mpx-bg-app`
- 当前代码实际使用名已是 `--mpx-bg-app-fill`
- 迁移时应以代码现状为准，不以旧文案为准

## 5. 2.0 共享壳层已知数据

### 5.1 当前定义目标

- 这是四大容器 `Header / Sidebar / Main / Metadata` 共用的外观基础层
- 其作用是先定义共享 frame，再由单容器按需覆写

### 5.2 当前字段

- 颜色：
  - `--mpx-container-frame-fill-start`
  - `--mpx-container-frame-fill-end`
  - `--mpx-container-frame-fill-angle`
  - `--mpx-container-frame-border-color`
  - `--mpx-container-frame-edge-color`
- 文本串：
  - `--mpx-container-frame-shadow`
- 形态/布局参数：
  - `layout-padding`
  - `splitter-width`
  - `container-frame-radius`

### 5.3 参数与面板定义位置

- 字段分组定义：
  - `src/components/theme-parameter/themeParameterPanelCatalog.ts:25`
  - `src/components/theme-parameter/themeParameterPanelCatalog.ts:52`
- 参数 ID 汇总：`src/components/theme-parameter/themeParameterPanelCatalog.ts:978`
- 标题文案：`src/i18n/locales/zh-CN.part1.ts:374`
- UI 分组呈现：`src/components/theme-parameter/ThemeParameterPanelMain.tsx:1533`
- 参数定义：
  - `layout-padding`、`splitter-width`：`src/components/theme-parameter/themeParameterDefinitions.ts:703`
  - `container-frame-radius`、`container-frame-fill-angle`：`src/components/theme-parameter/themeParameterDefinitions.ts:721`

### 5.4 实际消费

- 共享变量骨架/别名：`src/styles/themes/contract.css:352`
- 当前主题默认值：`src/styles/themes/styles/soft-skeuomorphic.css:6`
- 四大容器 frame 消费：`src/styles/themes/styles/soft-skeuomorphic-components/soft-skeuomorphic.components.part1.css:516`
- 布局参数消费：
  - Header 外间距：`src/styles/app/layout/layout.part1.css:14`
  - Workspace padding：`src/styles/app/layout/layout.part2.css:495`
  - Splitter 宽度：`src/styles/app/sidebar.css:846`
  - Splitter 宽度：`src/styles/app/metadata.css:50`

### 5.5 测试覆盖

- 共享壳层圆角联动：`src/components/ThemeParameterPanel.test.tsx:371`
- 阴影联动：`src/components/ThemeParameterPanel.test.tsx:441`
- fill 三件套联动：`src/components/ThemeParameterPanel.test.tsx:491`
- 分段折叠结构：`src/components/ThemeParameterPanel.test.tsx:1916`

## 6. 3 大面板层已知数据

说明：本节是按新项目编号整理；对应源项目当前的 `3.0~3.4`。

### 6.1 当前分页定义

- `3.0 Root/Shell 共用层`
- `3.1 Head / Side / Main 共享总控`
- `3.2 Head`
- `3.3 Side`
- `3.4 Main`

文案位置：`src/i18n/locales/zh-CN.part1.ts:410`

### 6.2 页面结构

- 页面骨架与预览按钮：`src/components/theme-parameter/ThemeParameterPanelPages.tsx:414`
- 真正挂载 `3.0~3.4` 各段：`src/components/theme-parameter/ThemeParameterPanelMain.tsx:2048`
- 通用行渲染器：`src/components/theme-parameter/ThemeParameterLayerSections.tsx:213`

### 6.3 3.0 Root/Shell 共用层

- 颜色：
  - `--mpx-large-panel-border-color`
  - `--mpx-large-panel-fill-start`
  - `--mpx-large-panel-fill-end`
- 文本串：
  - `--mpx-large-panel-shadow`
- 数值：
  - `width`
  - `height`
  - `fill-angle`
  - `radius`
  - `border-width`
  - `shell-padding`
  - `shell-gap`

字段定义位置：

- `src/components/theme-parameter/themeParameterPanelCatalog.ts:3705`
- `src/components/theme-parameter/themeParameterPanelCatalog.ts:4639`
- `src/components/theme-parameter/themeParameterPanelCatalog.ts:1021`
- `src/components/theme-parameter/themeParameterDefinitions.ts:988`

### 6.4 3.1 Shared

- 颜色：
  - `--mpx-large-panel-section-border-color`
  - `--mpx-large-panel-section-fill-start`
  - `--mpx-large-panel-section-fill-end`
- 数值：
  - `--mpx-large-panel-section-border-width`
  - `--mpx-large-panel-section-fill-angle`

共享映射表：`src/components/theme-parameter/themeParameterPanelCatalog.ts:1042`

### 6.5 3.2 Head

- 颜色：
  - `--mpx-large-panel-head-border-color`
  - `--mpx-large-panel-head-fill-start`
  - `--mpx-large-panel-head-fill-end`
  - `--mpx-large-panel-head-text`
- 数值：
  - `head-border-width`
  - `head-padding-y`
  - `head-padding-x`
  - `head-fill-angle`

定义位置：

- `src/components/theme-parameter/themeParameterPanelCatalog.ts:3750`
- `src/components/theme-parameter/themeParameterPanelCatalog.ts:4838`

### 6.6 3.3 Side

- 颜色：
  - `--mpx-large-panel-side-border-color`
  - `--mpx-large-panel-side-fill-start`
  - `--mpx-large-panel-side-fill-end`
- 数值：
  - `side-border-width`
  - `side-radius`
  - `side-padding`
  - `side-gap`
  - `side-fill-angle`

定义位置：

- `src/components/theme-parameter/themeParameterPanelCatalog.ts:3779`
- `src/components/theme-parameter/themeParameterPanelCatalog.ts:4838`

### 6.7 3.4 Main

- 颜色：
  - `--mpx-large-panel-main-border-color`
  - `--mpx-large-panel-main-fill-start`
  - `--mpx-large-panel-main-fill-end`
- 数值：
  - `main-border-width`
  - `main-radius`
  - `main-padding-y`
  - `main-padding-x`
  - `main-fill-angle`

定义位置：

- `src/components/theme-parameter/themeParameterPanelCatalog.ts:3800`
- `src/components/theme-parameter/themeParameterPanelCatalog.ts:4838`

### 6.8 变量说明与消费落点

- 中文标签与用途说明：`src/components/theme-parameter/themeParameterPanelCatalog.ts:2337`
- 大面板 token 总合同：`src/styles/themes/contract.css:1214`
- 大面板骨架消费：`src/styles/app/settings/settings.part1.css:1814`
- ThemeParameter 自己作为大面板实例：`src/styles/app/settings/settings.part1.css:2806`
- ThemeParameter 预览骨架：`src/components/theme-parameter/ThemeParameterPanelContainer.tsx:683`

### 6.9 对应面板实例 / slot

- Settings：`src/components/SettingsPanel.impl.tsx:1230` + `src/styles/app/settings/settings.part1.css:341`
- Help：`src/components/HelpPanel.tsx:364` + `src/styles/app/settings/settings.part1.css:428`
- ThemeParameter：`src/components/theme-parameter/ThemeParameterPanelContainer.tsx:745` + `src/styles/app/settings/settings.part1.css:515`
- SidebarRename：`src/components/SidebarRenameDialog.tsx:353` + `src/styles/app/settings/settings.part1.css:746`

### 6.10 测试与文档

- 分组与字段顺序：`src/components/ThemeParameterPanel.test.tsx:1160`
- `3.0 root` 变量作用到 ThemeParameter 面板本体：`src/components/ThemeParameterPanel.test.tsx:1549`
- `3.1` 共享总控同步到 `Head/Side/Main`：`src/components/ThemeParameterPanel.test.tsx:1631`
- 大面板层总章：`docs/32-ui-design-tracking-v1.md:440`
- 分页归属与覆盖范围：`docs/32-ui-design-tracking-v1.md:1390`
- 分页归属与覆盖范围：`docs/32-ui-design-tracking-v1.md:1408`
- 稳定路径归属：`docs/10-ui_definition.md:33`
- token 规则：`docs/11-token_design.md:112`
- 验收状态：`docs/35-ui-theme-config-tauri-roadmap-v1.md:109`

### 6.11 已知备注

- 当前 UI 里的 `3.0~3.4 = root/shared/head/side/main` 这套精确小节，主要记录在代码、i18n、测试里
- `docs/32-ui-design-tracking-v1.md` 仍保留旧编号语义，存在分页编号漂移

## 7. 4 小面板层已知数据

说明：本节是按新项目编号整理；对应源项目当前的 `5.0 Root`。

### 7.1 当前定义

- `ThemeParameter -> 小面板层调试 -> 5.0 Root` 是小面板骨架总控
- 页面标题：`src/i18n/locales/zh-CN.part1.ts:436`
- 页面结构：`src/components/theme-parameter/ThemeParameterPanelPages.tsx:483`
- 根段 `<summary>`：`src/components/theme-parameter/ThemeParameterPanelPages.tsx:525`
- 根段内容注入：`src/components/theme-parameter/ThemeParameterPanelMain.tsx:2192`

### 7.2 5.0 暴露字段

- 颜色：
  - `--mpx-dialog-panel-root-border-color`
  - `--mpx-dialog-panel-root-fill-start`
  - `--mpx-dialog-panel-root-fill-end`
- 文本串：
  - `--mpx-dialog-panel-shadow`
- inline 数值：
  - `small-panel-fill-angle -> --mpx-dialog-panel-root-fill-angle`
- 普通数值：
  - `width`
  - `max-width`
  - `height`
  - `max-height`
  - `border-width`
  - `radius`
  - `padding`
  - `gap`

位置：

- `src/components/theme-parameter/themeParameterPanelCatalog.ts:4876`
- `src/components/theme-parameter/themeParameterPanelCatalog.ts:5226`
- `src/components/theme-parameter/themeParameterPanelCatalog.ts:1060`
- `src/components/theme-parameter/themeParameterDefinitions.ts:1200`
- `src/components/theme-parameter/themeParameterDefinitions.ts:1210`

### 7.3 同步关系

- `5.0` 会把 root 值同步到 `5.1~5.8` 各小面板分控
- 颜色同步表：`src/components/theme-parameter/themeParameterPanelCatalog.ts:1075`
- 渐变角度同步表：`src/components/theme-parameter/themeParameterSync.ts:42`

### 7.4 实际样式链路

- 语义基线定义：`src/styles/themes/contract.css:1662`
- style 模板同构定义：`src/styles/themes/styles/_style-template.css:465`
- 小面板骨架消费：`src/styles/app/settings/settings.part2.css:300`

实际消费变量：

- `--mpx-dialog-panel-width`
- `--mpx-dialog-panel-max-width`
- `--mpx-dialog-panel-height`
- `--mpx-dialog-panel-max-height`
- `--mpx-dialog-panel-border-width`
- `--mpx-dialog-panel-root-border-color`
- `--mpx-dialog-panel-root-fill-start`
- `--mpx-dialog-panel-root-fill-end`
- `--mpx-dialog-panel-root-fill-angle`
- `--mpx-dialog-panel-shadow`
- `--mpx-dialog-panel-padding`
- `--mpx-dialog-panel-gap`

### 7.5 预览 / 验证

- 小面板分页预览模式：`bg-plus-small-panel`
- 预览按钮：`src/components/theme-parameter/ThemeParameterPanelPages.tsx:506`
- 预览 mock 卡片：`src/components/theme-parameter/ThemeParameterPanelContainer.tsx:718`
- `5.0/5.1~5.8` 分段结构：`src/components/ThemeParameterPanel.test.tsx:1459`
- `5.0 root` 字段可见：`src/components/ThemeParameterPanel.test.tsx:1474`
- `5.0 root` 总控同步各分控：`src/components/ThemeParameterPanel.test.tsx:1754`

### 7.6 文档记录

- 主说明文档：`docs/32-ui-design-tracking-v1.md:1119`
- 分页归属与覆盖范围：`docs/32-ui-design-tracking-v1.md:1409`
- 手工验收表：`docs/35-ui-theme-config-tauri-roadmap-v1.md:110`
- 稳定路径归属：`docs/10-ui_definition.md:34`
- 具体纳入的小面板路径条目：`docs/10-ui_definition.md:90`
- 具体纳入的小面板路径条目：`docs/10-ui_definition.md:148`
- 具体纳入的小面板路径条目：`docs/10-ui_definition.md:165`
- 具体纳入的小面板路径条目：`docs/10-ui_definition.md:186`
- 具体纳入的小面板路径条目：`docs/10-ui_definition.md:191`
- 具体纳入的小面板路径条目：`docs/10-ui_definition.md:275`
- Token 分页归属：`docs/11-token_design.md:108`
- README 验收索引：`docs/01-README.md:47`

### 7.7 当前结论

- `5.0` 在源项目代码里对应的是“小面板 root 骨架总控”，不是某个具体业务对话框
- 具体业务弹窗 `5.1~5.8` 都默认回退到 `5.0`
- 文档与代码当前基本一致，没有明显语义漂移
- 但在新项目里，小面板层已决定完全独立，不继承大面板层

## 8. 5 按钮层已知数据

说明：本节是按新项目编号整理；对应源项目当前按钮页里的 `4.1 Default 通用变体`。

### 8.1 当前定义

- `buttonStates` 分页里的 `4.1 Default 通用变体`
- 用于调 `--mpx-btn-variant-default-*` 这组默认按钮变体变量
- 文案与说明：
  - `src/i18n/locales/zh-CN.part1.ts:347`
  - `src/i18n/locales/zh-CN.part1.ts:348`

### 8.2 渲染结构

- 分页壳子：`src/components/theme-parameter/ThemeParameterPanelPages.tsx:552`
- `4.1` 折叠段本体：`src/components/theme-parameter/ThemeParameterPreviewSections.tsx:539`
- 面板挂载：`buttonStates` 页内容由 `ThemeParameterButtonStateDebug` 注入：`src/components/theme-parameter/ThemeParameterPanelMain.tsx:2270`

### 8.3 字段定义

- 颜色类：
  - `border`
  - `bg-idle`
  - `bg-hover`
  - `bg-active`
  - `bg-pressed`
  - `text-idle`
  - `text-active`
  - `text-pressed`
  - `text-merged`
  - `text-disabled`
  - `danger-hover-bg`
  - `danger-hover-border`
  - `danger-hover-text`
- 文本串类：
  - `shadow-idle`
  - `shadow-hover`
  - `shadow-active`
  - `shadow-pressed`
  - `transform-hover`
  - `transform-active`
  - `transform-pressed`
  - `danger-hover-shadow`

定义位置：

- `src/components/theme-parameter/themeParameterPanelCatalog.ts:2647`
- `src/components/theme-parameter/themeParameterPanelCatalog.ts:2987`

### 8.4 分类与用途说明

- `4.1` 是从总表中过滤 `button-default-` 前缀得到：`src/components/theme-parameter/themeParameterPanelCatalog.ts:3055`
- 用途说明：用于通用按钮 `default` 变体：`src/components/theme-parameter/themeParameterPanelCatalog.ts:2620`

### 8.5 变量源头与消费

- alias / 回退链定义：`src/styles/themes/contract.css:233`
- 主消费端：`src/styles/app/button-template.css:9`
- 设置 / 通用按钮消费：`src/styles/app/settings/settings.part1.css:33`

### 8.6 测试与文档

- 按钮页变量可写入并进入快照：`src/components/ThemeParameterPanel.test.tsx:687`
- 主体文档：`docs/32-ui-design-tracking-v1.md:1013`
- 分页归属：`docs/10-ui_definition.md:35`
- token 命名规则：`docs/11-token_design.md:15`
- 手工验收状态：`docs/35-ui-theme-config-tauri-roadmap-v1.md:112`

### 8.7 当前结论

- 源项目中的 `4.1` 本质上就是默认按钮变体入口
- 在新项目里，这一组变量已决定上升为所有按钮的默认根样式
- 特殊按钮不再走专门按钮分支，而是后续只做下一层局部覆盖

## 9. 源项目的稳定路径与 token 关系

### 6.1 稳定路径骨架

- `bg.app.root`
- `bg.app.workspace`
- `fg.header.root`
- `fg.sidebar.root`
- `fg.main.root`
- `fg.meta.root`

来源：`docs/10-ui_definition.md:44`

### 6.2 token 前缀关系

- `bg.app.* -> --mpx-slot-bg-app-*-*`
- `fg.header.root -> --mpx-slot-fg-header-root-*`
- `fg.sidebar.root -> --mpx-slot-fg-sidebar-root-*`
- `fg.main.root -> --mpx-slot-fg-main-root-*`
- `fg.meta.root -> --mpx-slot-fg-meta-root-*`

来源：`docs/11-token_design.md:281`

## 10. 当前对新项目最有价值的抽取结论

### 10.1 应直接继承的结构结论

- 背景层与容器壳层应分开定义
- 四大容器应共享一套 frame 基架
- 单容器差异应晚于共享壳层落地
- 布局参数与视觉参数应并列存在，不能混写
- 大面板与小面板应分成两套独立骨架
- 按钮应先固定单一默认根样式，再允许局部覆写

### 10.2 应避免直接搬运的内容

- 不直接迁 ThemeParameter 面板实现
- 不直接迁旧项目整套 slot 覆写体系
- 不直接迁旧项目软拟态主题文件
- 不直接复制旧项目页面级 CSS 结构
- 不直接把小面板作为大面板子类布局搬进新仓
- 不继续沿用源项目的多官方按钮分支结构

### 10.3 当前建议的新项目抽取范围

- 抽取“层次结构”
- 抽取“变量责任边界”
- 抽取“回落机制”
- 不抽取旧项目视觉细节和组件树

## 11. 相关文件总表

### 11.1 面板结构 / 文案

- `src/components/theme-parameter/ThemeParameterPanelPages.tsx:300`
- `src/components/theme-parameter/ThemeParameterPanelMain.tsx:1519`
- `src/components/theme-parameter/themeParameterPanelCatalog.ts:15`
- `src/components/theme-parameter/themeParameterDefinitions.ts:703`
- `src/i18n/locales/zh-CN.part1.ts:342`
- `src/i18n/locales/en-US.part1.ts:398`

### 11.2 运行时样式

- `src/styles/app/layout/layout.part1.css:1`
- `src/styles/app/layout/layout.part2.css:495`
- `src/styles/app/sidebar.css:846`
- `src/styles/app/metadata.css:50`
- `src/styles/themes/contract.css:352`
- `src/styles/themes/styles/soft-skeuomorphic.css:6`
- `src/styles/themes/styles/soft-skeuomorphic-components/soft-skeuomorphic.components.part1.css:516`
- `src/styles/app/button-template.css:9`
- `src/styles/app/settings/settings.part1.css:1814`
- `src/styles/app/settings/settings.part2.css:300`
- `src/styles/themes/contract.css:1214`
- `src/styles/themes/contract.css:1662`

### 11.3 文档记录

- `docs/10-ui_definition.md:25`
- `docs/11-token_design.md:252`
- `docs/32-ui-design-tracking-v1.md:3`
- `docs/35-ui-theme-config-tauri-roadmap-v1.md:80`
- `docs/36-theme-container-frame-migration-plan-v1.md:24`
- `docs/39-theme-derived-fallback-audit-and-fix-plan-v1.md:40`
- `docs/02-DOCS_INDEX.md:45`
- `docs/01-README.md:52`

## 12. 变更记录

### 2026-03-11

- 建立源项目 UI 已知数据记录文档
- 固定 `1.0 背景层` 与 `2.0 共享壳层` 的已知锚点
- 固定“通用设定 -> 派生设定 -> 默认回落”的当前实施共识
- 追加 `3 大面板层`、`4 小面板层`、`5 按钮层` 的源项目已知数据整理
- 记录新项目已确认编号：`3/4/5 = 大面板/小面板/按钮`
- 记录新项目已确认边界：`4 小面板层` 不继承 `3 大面板层`，`5.1` 作为所有按钮的默认根样式
