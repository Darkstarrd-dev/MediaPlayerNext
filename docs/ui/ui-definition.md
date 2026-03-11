# MediaPlayerNext UI 定义工作文档

## 1. 文档定位

本文件只记录 `MediaPlayerNext` 的正式 UI 定义。

边界固定如下：

- 只写新项目当前决定采用的布局、变量、回落规则与切片方案
- 不记录源项目文件位置、历史实现细节与旧文档锚点
- 源项目已知信息统一记录在 `docs/ui/source-ui-known-data.md`

## 2. UI 定义方法

当前 UI 采用两级定义，再加一层消费约束：

1. 通用设定
2. 派生设定
3. 消费约束

### 2.1 通用设定

- 负责定义所有容器都可共享的背景、frame、布局和间距基础
- 新项目当前先从 `1.0 背景层` 与 `2.0 共享壳层` 开始

### 2.2 派生设定

- 负责在通用设定之上，按容器建立差异化覆写入口
- 当前先预留四个容器派生层：
  - `Header`
  - `Sidebar`
  - `Main`
  - `Metadata`

### 2.3 消费约束

- React 组件和样式层只能消费容器别名或 slot 变量
- 不允许直接在组件层写死颜色值替代通用设定
- 不允许跳过回落链直接耦合最终主题实现

## 3. 当前阶段目标

先完成新的初始 UI 架构：

1. App 总背景层
2. 主页面四大容器分隔
3. 背景层与共享壳层变量合同
4. 默认回落机制
5. U0 验收基线

## 4. U0 范围定义

### 4.1 本批实现内容

- `1.0 背景层`
- `2.0 共享壳层`
- 四大容器布局分隔：
  - `Header`
  - `Sidebar`
  - `Main`
  - `Metadata`

### 4.2 本批不做内容

- 不接入真实面板业务内容
- 不做复杂视觉细节和主题等价复刻
- 不引入页面级交互打磨

## 5. 稳定层级与容器定义

当前先固定以下稳定层级：

- `bg.app.root`
- `fg.header.root`
- `fg.sidebar.root`
- `fg.main.root`
- `fg.meta.root`

说明：

- `bg.app.root` 负责最外层背景
- 其余四项负责主页面的四大前景容器
- `Header` 已继续细化为 Logo 入口与设置入口所在的稳定根容器
- `Sidebar / Main / Metadata` 当前已建立各自的 `header / main / footer` 三段结构与对应 slot

## 6. 当前变量合同

### 6.1 1.0 背景层

- `--mpx-bg-app-fill`

说明：

- 当前先用于应用最外层背景填充
- 后续如有需要，再补工作区背景承载变量

### 6.2 2.0 共享壳层

- `--mpx-container-frame-fill-start`
- `--mpx-container-frame-fill-end`
- `--mpx-container-frame-fill-angle`
- `--mpx-container-frame-fill`
- `--mpx-container-frame-border-color`
- `--mpx-container-frame-edge-color`
- `--mpx-container-frame-shadow`
- `--mpx-container-frame-radius`
- `--mpx-layout-padding`
- `--mpx-splitter-width`

### 6.3 容器派生层预留

当前已开始首轮落地：

- `--mpx-header-*`
- `--mpx-sidebar-*`
- `--mpx-main-*`
- `--mpx-metadata-*`

当前已落的运行时布局变量补充：

- `--mpx-pane-inner-gap-scale`
- `--mpx-pane-inner-padding-px`
- `--mpx-pane-stack-gap-scale`
- `--mpx-pane-stack-gap-px`
- `--mpx-pane-section-gap-px`
- `--mpx-pane-header-height-px`
- `--mpx-pane-footer-height-px`

## 7. 通用组件定义编号

当前阶段，通用组件定义编号固定如下：

- `3`：大面板层
- `4`：小面板层
- `5`：按钮层

补充约束：

- `4 小面板层` 完全独立，不继承 `3 大面板层`
- `5.1 通用变体` 是整个 app 的默认按钮根样式
- 特殊按钮只允许在 `5.1` 的下一层做局部覆盖，不再开官方专门按钮分支

## 8. 3 大面板层

### 8.1 定位

- 用于设置、帮助、主题参数等大型面板的统一骨架
- 负责大面板的 root、shell、head、side、main 五段结构
- 当前不负责具体业务内部件语义

### 8.2 结构

- `3.0 Root/Shell`
- `3.1 Shared`
- `3.2 Head`
- `3.3 Side`
- `3.4 Main`

### 8.3 变量范围

`3.0 Root/Shell`：

- `--mpx-large-panel-width`
- `--mpx-large-panel-height`
- `--mpx-large-panel-max-width`
- `--mpx-large-panel-max-height`
- `--mpx-large-panel-border-width`
- `--mpx-large-panel-border-color`
- `--mpx-large-panel-radius`
- `--mpx-large-panel-fill-start`
- `--mpx-large-panel-fill-end`
- `--mpx-large-panel-fill-angle`
- `--mpx-large-panel-fill`
- `--mpx-large-panel-bg`
- `--mpx-large-panel-shadow`
- `--mpx-large-panel-shell-columns`
- `--mpx-large-panel-shell-no-side-columns`
- `--mpx-large-panel-shell-gap`
- `--mpx-large-panel-shell-padding`

`3.1 Shared`：

- `--mpx-large-panel-section-border-width`
- `--mpx-large-panel-section-border-color`
- `--mpx-large-panel-section-fill-start`
- `--mpx-large-panel-section-fill-end`
- `--mpx-large-panel-section-fill-angle`

`3.2 Head`：

- `--mpx-large-panel-head-border-width`
- `--mpx-large-panel-head-border-color`
- `--mpx-large-panel-head-fill-start`
- `--mpx-large-panel-head-fill-end`
- `--mpx-large-panel-head-fill-angle`
- `--mpx-large-panel-head-bg`
- `--mpx-large-panel-head-text`
- `--mpx-large-panel-head-padding-y`
- `--mpx-large-panel-head-padding-x`

`3.3 Side`：

- `--mpx-large-panel-side-border-width`
- `--mpx-large-panel-side-border-color`
- `--mpx-large-panel-side-fill-start`
- `--mpx-large-panel-side-fill-end`
- `--mpx-large-panel-side-fill-angle`
- `--mpx-large-panel-side-bg`
- `--mpx-large-panel-side-radius`
- `--mpx-large-panel-side-padding`
- `--mpx-large-panel-side-gap`

`3.4 Main`：

- `--mpx-large-panel-main-border-width`
- `--mpx-large-panel-main-border-color`
- `--mpx-large-panel-main-fill-start`
- `--mpx-large-panel-main-fill-end`
- `--mpx-large-panel-main-fill-angle`
- `--mpx-large-panel-main-bg`
- `--mpx-large-panel-main-radius`
- `--mpx-large-panel-main-padding-y`
- `--mpx-large-panel-main-padding-x`

### 8.4 消费约束

- 大面板实例通过统一骨架类消费上述变量
- `settings/help/theme-parameter` 等实例内部件 token 不属于 `3` 的定义范围
- 当前阶段允许 `side` 为空，但定义仍保留 `side` 结构

## 9. 4 小面板层

### 9.1 定位

- 用于对话框、小确认面板、小编辑面板等小型浮层
- 只定义小面板 root 骨架
- 具体业务弹窗默认回落到 `4.0 Root`

### 9.2 独立性

- `4 小面板层` 不继承 `3 大面板层`
- 不复用 `3` 的布局结构
- 即使视觉方向相近，也保持独立变量与独立回落链

### 9.3 变量范围

- `--mpx-dialog-panel-width`
- `--mpx-dialog-panel-max-width`
- `--mpx-dialog-panel-height`
- `--mpx-dialog-panel-max-height`
- `--mpx-dialog-panel-border-width`
- `--mpx-dialog-panel-border-color`
- `--mpx-dialog-panel-radius`
- `--mpx-dialog-panel-root-border-color`
- `--mpx-dialog-panel-root-fill-start`
- `--mpx-dialog-panel-root-fill-end`
- `--mpx-dialog-panel-root-fill-angle`
- `--mpx-dialog-panel-fill-start`
- `--mpx-dialog-panel-fill-end`
- `--mpx-dialog-panel-fill-angle`
- `--mpx-dialog-panel-bg`
- `--mpx-dialog-panel-shadow`
- `--mpx-dialog-panel-padding`
- `--mpx-dialog-panel-gap`

### 9.4 消费约束

- 小面板只消费自身 root 变量
- 不引入 `head/side/main/shell` 四段布局
- 当前阶段小面板内的按钮统一使用 `5.1 通用变体`

## 10. 5 按钮层

### 10.1 定位

- `5.1 通用变体` 是整个 app 的默认按钮根样式
- 所有按钮默认先走 `5.1`，再决定是否做局部覆写
- 当前不定义官方专门按钮分支

### 10.2 结构

- `5.0 按钮层`
- `5.1 通用变体`

### 10.3 变量范围

- `--mpx-btn-variant-default-border`
- `--mpx-btn-variant-default-bg-idle`
- `--mpx-btn-variant-default-bg-hover`
- `--mpx-btn-variant-default-bg-active`
- `--mpx-btn-variant-default-bg-pressed`
- `--mpx-btn-variant-default-text-idle`
- `--mpx-btn-variant-default-text-active`
- `--mpx-btn-variant-default-text-pressed`
- `--mpx-btn-variant-default-text-merged`
- `--mpx-btn-variant-default-text-disabled`
- `--mpx-btn-variant-default-shadow-idle`
- `--mpx-btn-variant-default-shadow-hover`
- `--mpx-btn-variant-default-shadow-active`
- `--mpx-btn-variant-default-shadow-pressed`
- `--mpx-btn-variant-default-transform-hover`
- `--mpx-btn-variant-default-transform-active`
- `--mpx-btn-variant-default-transform-pressed`
- `--mpx-btn-variant-default-danger-hover-bg`
- `--mpx-btn-variant-default-danger-hover-border`
- `--mpx-btn-variant-default-danger-hover-text`
- `--mpx-btn-variant-default-danger-hover-shadow`

### 10.4 消费约束

- 当前所有按钮默认使用统一按钮基类消费 `5.1`
- 特殊按钮不再走独立按钮分支
- 如需差异，只能在下一层通过 slot 或局部变量覆写

## 11. 默认回落规则

未单独指定派生设定时，统一应用通用设定。

回落链固定如下：

```text
slot/局部覆写
-> 单容器派生变量
-> 共享壳层通用变量
-> 合同默认值
```

补充要求：

- 所有单容器变量都必须允许回落到共享壳层
- 所有共享壳层变量都必须有合同默认值
- 当前阶段不允许出现“无回落、仅某主题可用”的变量

主题基础层补充回落规则：

`3 大面板层`：

```text
实例 slot 覆写
-> 3.2/3.3/3.4 局部变量
-> 3.1 shared
-> 3.0 root/shell
-> 合同默认值
```

`4 小面板层`：

```text
实例 slot 覆写
-> 4.0 root 变量
-> 合同默认值
```

`5.1 通用按钮`：

```text
按钮 slot 覆写
-> 5.1 通用变体
-> 合同默认值
```

## 12. 布局实施方式

### 12.1 页面骨架

- 顶层 App 背景层
- 顶部 `Header`
- 中部 workspace
- workspace 内部分为：
  - 左侧 `Sidebar`
  - 中部 `Main`
  - 右侧 `Metadata`

当前主界面固定为：

```text
Header
Sidebar | Main | Metadata
```

补充约束：

- `Header` 当前已包含 `Logo` 按钮与设置按钮
- `Logo` 按钮打开的是独立的大面板 overlay，不回退为说明型入口
- `Sidebar / Main / Metadata` 三列当前都固定采用 `header / main / footer` 三段结构
- 三列中的 `main` 是主要伸缩区
- 三列中的 `header / footer` 以固定高度和统一基线保持对齐

### 12.2 实施顺序

1. 先建立 DOM 骨架
2. 再接背景层变量
3. 再接共享壳层变量
4. 再接布局参数与 splitter 宽度
5. 最后再进入单容器派生层细化

### 12.3 当前布局口径

- 不再把主界面做成说明页或自动折叠页
- Root 级不允许滚动条
- 滚动只允许发生在三列各自的 `main` 区域
- `splitter width` 只影响三列之间的横向间距，不影响 `Header` 与 workspace 的纵向间距
- `pane stack gap` 只影响 `Sidebar / Main / Metadata` 三列中 `header / main / footer` 的纵向间距，不下沉到 `main` 内部内容间距

## 13. 切片规划

| 切片 | 目标 | 验收重点 | 状态 |
|---|---|---|---|
| `U0` | 背景层 + 共享壳层 + 四容器分隔 | 结构、变量合同、回落链、基础布局 | 已完成首轮 |
| `U1` | `LibraryPanel` + `ScanPanel` | 基础操作流与状态展示 | 进行中 |
| `U2` | `ItemsPanel` | 列表与缩略图主链路 | 待开始 |
| `U3` | `ArchivePanel` | 归档浏览与页序导航 | 待开始 |
| `U4` | `DetailPanel` | 详情与基础元数据 | 待开始 |
| `U5` | 播放/字幕相关面板回接 | 非当前优先级 | 暂缓 |

## 14. U0 验收标准

- App 最外层背景只由背景层变量驱动
- `Header / Sidebar / Main / Metadata` 四个容器都能独立分隔显示
- 四个容器默认共享同一套 frame 基架
- `layout padding` 能影响主工作区留白
- `splitter width` 有明确变量入口
- `Sidebar / Main / Metadata` 都具有稳定的 `header / main / footer` 子结构
- 三列 `header / footer` 保持固定高度与垂直对齐
- `pane stack gap` 有明确变量入口，并且只影响三列 `header / main / footer` 的纵向间距
- 在没有单容器覆写时，界面不出现样式断链

## 15. 每个切片的固定模板

后续新增或更新某个切片时，统一按以下结构补充：

### 切片编号

- 目标：
- 范围：
- 不做什么：
- 依赖的 repository 方法：
- 需要的状态：
- 需要的布局块：
- 验收标准：
- 当前结论：

## 16. 当前待确认清单

- `Sidebar` 是否在 `U1` 就承担扫描入口
- `Metadata` 在早期是否只保留空态与占位
- `Main` 在 `U2` 前是否只作为结构容器存在
- 第一版基础样式基座的视觉方向是否继续沿用当前柔和浅色系

## 17. 变更记录

### 2026-03-11

- 建立新项目 UI 定义文档
- 固定“通用设定 -> 派生设定 -> 消费约束”的定义方法
- 固定 `U0` 仅实现背景层、共享壳层与四大容器分隔
- 将源项目已知信息拆分到 `docs/ui/source-ui-known-data.md`
- `apps/desktop/src/app/AppShell.tsx` 已切到 `bg.app.root + fg.header.root + fg.sidebar.root + fg.main.root + fg.meta.root` 的四容器骨架
- `apps/desktop/src/App.css` 已建立 `--mpx-bg-app-fill`、`--mpx-container-frame-*`、`--mpx-layout-padding`、`--mpx-splitter-width` 的首轮共享壳层样式基座
- 固定 `3 大面板 / 4 小面板 / 5 按钮` 的新编号
- 固定 `4 小面板层` 完全独立，不继承 `3 大面板层`
- 固定 `5.1 通用变体` 为整个 app 的默认按钮根样式
- `Header` 已增加设置入口按钮，并开始接入最小设置面板
- 最小设置面板当前只开放 `界面设置`，首轮包含遮罩透明度、容器外边界系数、容器内边距系数、分割条宽度系数四项运行时设置
- `Header` 已补入 `Logo` 按钮，当前文案为 `MediaPlayerNext`，并可打开独立的大面板 overlay
- `fg-import-task-root` / `fg-import-task-ovl` 已作为大面板实例首轮接入，当前用于导入任务入口占位
- `Sidebar / Main / Metadata` 已从统一占位块细化为 `header / main / footer` 三段结构，并挂上对应 slot
- 当前已新增 `容器内上中下间距系数` 设置项，范围 `0~2`，会话级保留，仅影响三列 `header / main / footer` 的纵向间距
- `U1` 已开始首轮接入：当前通过 `ImportTaskPanel` 支持本地路径登记媒体库、登记并扫描，并在成功后刷新 `Sidebar / Main / Metadata`
- `ImportTaskPanel` 当前已补入系统文件夹选择器，并继续保留手动路径输入作为补充入口
- 主窗口当前已开始监听全窗口拖拽，拖入本地路径后会直接进入当前导入链路并刷新主界面三列
- 主窗口当前已开始监听全局 `paste`，当剪贴板中存在本地路径文本时，会直接进入当前导入链路并刷新主界面三列
- Header Logo 当前已补入 `busy` 态；`ImportTaskPanel` 与扫描摘要已开始显示轮询中的扫描状态与进度条
- `Sidebar` 当前已开始显示真实媒体库列表、当前扫描摘要与最小扫描动作
- `Main` 当前已开始显示当前媒体库的最小条目预览，并支持当前页内条目切换
- `Metadata` 当前已开始显示当前媒体库、扫描摘要与选中条目的最小详情
