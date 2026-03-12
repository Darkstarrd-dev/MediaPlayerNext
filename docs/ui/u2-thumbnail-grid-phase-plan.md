# U2 缩略图网格分页与缩放实施计划

## 1. 文档定位

本文件用于指导 `Main` 区域的 `U2` 首轮实施，把当前“固定 12 项预览 + 条目级前后切换”推进到更接近源项目语义的“容器驱动分页网格 + 1~7 档缩放”。

本文件是执行型计划文档，按 phase 拆分工作；每个 phase 都明确：

- 开始前必须读取的文件
- todo 顺序
- 具体要做的内容
- 完成后需要回填的状态 check

本文件本身应包含在新对话中继续推进该任务所需的最小上下文，不依赖额外口头补充。

基础版完成后的增强项实施文档：

- `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md`

## 2. 当前目标与固定产出

### 2.1 本批要做的内容

- 在 `Main.header` 增加 `缩略图缩放级别` 控件
- 首轮控件使用原生下拉菜单，取值固定为 `1~7`
- 默认缩放级别固定为 `4`
- `Main.main` 从固定预览网格改为“整页分页网格”
- `pageSize` 不再写死，而是由容器尺寸与缩放级别共同推导
- `Main.footer` 从“条目级 Prev / Next”改为“页面级 Prev / Next”
- 切换缩放级别后，重新计算 `columns / rows / pageSize / totalPages`
- 缩略图容器的自适应起点固定为 `ResizeObserver`
- 首轮继续复用当前 `repository.items.list({ page, pageSize })`

### 2.2 本批明确不做的内容

- 不做无限滚动
- 不做虚拟网格 (grid virtualization)
- 不做滚轮翻页预览与 settle 翻页
- 不做页码预览浮层
- 不做“旧页保留到新页 ready 再 commit”的缓冲机制
- 不做 `gap snap`
- 不做缩略图分辨率自适应联动策略
- 不补真实后端节点树；本批仍只聚焦 `Main`

## 3. 当前已确认的固定语义

### 3.1 分页语义

- `Main` 的缩略图容器采用“整页分页网格”，不是无限滚动
- 当前页显示条目数固定为：

```text
pageSize = columns * rows
```

- `pageSize` 必须由容器尺寸实时推导，而不是固定常量
- 页面切换入口首轮只保留 `Main.footer`

### 3.2 缩放语义

- 缩放不是连续值，而是 `1~7` 的离散级别
- 首轮沿用源项目核心口径：`缩放级别 = 目标行数锚点`
- 默认级别固定为 `4`
- 缩放级别变化后必须触发：
  - 网格重新计算
  - 当前页修正
  - 当前页数据重新读取

### 3.3 自适应语义

- 自适应起点固定为缩略图容器可用尺寸，而不是窗口尺寸
- 容器尺寸变化通过 `ResizeObserver` 驱动
- 首轮算法以“在固定容器内尽量铺满整页、避免主区滚动”为目标
- Root 级仍不允许滚动；`Header / Footer` 固定，`Main.main` 为主要伸缩区

## 4. 源项目结论（已整理）

以下结论已经从源项目 `Z:\Playground\CurrentWorking\MediaPlayerX` 整理出来，后续新对话不必重新做全量代码考古。

### 4.1 分页策略

- 核心策略是“分页网格”，不是无限滚动
- 页大小由布局算法实时计算：`pageSize = columns * rows`
- 图片分页视图模型统一计算：
  - `totalPages`
  - `normalizedPageIndex`
  - `pageStart`
  - `refsInPage`
- Footer 是统一页面翻页入口
- 节点浏览与普通图片网格复用同一分页语义
- 当前页切换采用整页渲染，不使用虚拟网格

### 4.2 自适应策略

- `ResizeObserver` 负责把网格容器尺寸回写到状态
- 布局算法根据：
  - `gridWidth`
  - `gridHeight`
  - `zoomLevel`
  - `gap`
  - `cardChrome`
  计算出：
  - `rows`
  - `columns`
  - `cellWidth`
  - `mediaHeight`
  - `pageSize`
- 缩放级别是 `1~7` 离散档位，默认 `4`
- 最终列数和尺寸直接注入 CSS Grid

### 4.3 源项目关键文件（按主题归类）

- 分页与布局：
  - `src/features/layout/thumbnailLayout.ts`
  - `src/features/app/useImageBrowserViewModel.ts`
  - `src/features/app/useWorkspaceNodeBrowsePaging.ts`
- 容器观测与自适应：
  - `src/features/app/useAppEffects.ts`
  - `src/features/app/useAppNavigationState.ts`
- 渲染与 footer：
  - `src/components/ImageMainSection.tsx`
  - `src/components/ImageMainSection.renderers.tsx`
  - `src/features/app/buildMainFooter.tsx`
- 文档：
  - `docs/05-interaction-v1.md`
  - `docs/08-theme-system-v2.md`
  - `docs/10-ui_definition.md`

## 5. 新仓当前现状判断

截至本计划建立时，新仓 `MediaPlayerNext` 的当前状态是：

- `Main.main` 已经切成纯缩略图网格，但网格仍是简单 CSS 自适应
- 当前列表读取仍写死为 `page = 1` 与固定 `pageSize`
- `Main.footer` 仍是“选中条目级 Prev / Next”，还不是页面级分页
- 缩略图生成与显示链路已经可用：
  - `thumbnail.ensure()` 已可调用
  - `thumb://` / `media://` / `archive://` 协议已可用
- `items.list()` 已支持 `page / pageSize`，因此本批不需要先改后端分页接口

### 5.1 当前最相关文件

- 当前主实现：`apps/desktop/src/app/AppShell.tsx`
- 当前样式：`apps/desktop/src/App.css`
- repository surface：`apps/desktop/src/repositories/media-repository.ts`
- Tauri command adapter：`apps/desktop/src/adapters/tauri/commands.ts`
- contracts：`packages/contracts/src/models/items.ts`
- 当前 UI 定义：`docs/ui/ui-definition.md`

### 5.2 当前最关键缺口

- 还没有 `gridSize -> layout -> pageSize -> items.list(page,pageSize)` 的完整状态链
- 还没有 `thumbnailZoomLevel` 状态
- 还没有页面级分页模型
- 还没有网格容器尺寸观测
- Footer 语义还停留在“条目切换”

结论：

- 这不是单纯“补一个下拉框”的任务
- 必须按 `布局状态 -> 容器观测 -> 分页读取 -> footer 翻页 -> 文档回填` 的顺序推进

## 6. 建议文件落点

首轮尽量保持最小改动，但建议把“纯计算”从 `AppShell` 中拆开，避免继续把大组件堆大。

建议文件落点：

- `apps/desktop/src/app/AppShell.tsx`
  - 继续承载页面状态、repository 读取、footer 交互
- `apps/desktop/src/app/thumbnail-grid-layout.ts`
  - 新增纯函数：根据容器尺寸与缩放级别计算网格布局
- `apps/desktop/src/App.css`
  - 承载 header 控件、分页 footer、动态 grid 的基础样式
- `docs/ui/u2-thumbnail-grid-phase-plan.md`
  - 本计划文档，执行中持续回填

说明：

- 首轮不建议为了这个任务再额外抽 `useResizeObserver` 公共 hook，除非后续出现第二个复用点
- 若实现中发现 `AppShell.tsx` 已明显过大，可以在 Phase 2 再决定是否抽轻量 helper

## 7. Phase 总览

| Phase | 目标 | 状态 |
|---|---|---|
| `Phase 0` | 冻结语义并建立实施文档 | `done` |
| `Phase 1` | 建立缩略图网格布局纯计算层 | `pending` |
| `Phase 2` | 接入 `Main.header` 缩放控件与容器尺寸观测 | `pending` |
| `Phase 3` | 接入页面级分页读取与 Footer 翻页 | `pending` |
| `Phase 4` | 联调缩放 / 分页 / 选中项同步并补样式 | `pending` |
| `Phase 5` | 验证、日志、文档回填 | `pending` |

---

## 8. Phase 0：冻结语义并建立实施文档

### 8.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `docs/ui/ui-definition.md`
- `docs/ui/source-ui-known-data.md`
- `docs/README.md`

### 8.2 todo 顺序

1. 冻结本批边界与不做项
2. 冻结分页语义与缩放语义
3. 冻结建议文件落点
4. 把本计划文档加入 docs 索引

### 8.3 要做的内容

- 把本文件作为 `U2` 首轮执行计划入口
- 明确本批不是“补全旧仓全部图片浏览器”，而是先建立最小可用分页网格骨架
- 明确后续 phase 推进时，优先按本文件读取顺序加载上下文，而不是重新全仓探索

### 8.4 phase 完成后状态 check

- [x] 本批目标、边界与不做项已冻结
- [x] 分页与缩放语义已写清楚
- [x] docs 索引可找到本计划文档

---

## 9. Phase 1：建立缩略图网格布局纯计算层

### 9.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `apps/desktop/src/app/AppShell.tsx`
- `apps/desktop/src/App.css`
- 源项目参考：
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\layout\thumbnailLayout.ts`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\docs\05-interaction-v1.md`

### 9.2 todo 顺序

1. 新增布局计算文件与最小类型
2. 冻结 `zoomLevel -> rows` 的首轮映射
3. 计算 `columns / rows / cellSize / pageSize`
4. 给极小尺寸场景加最小保护

### 9.3 要做的内容

建议新增最小类型：

```ts
export interface ThumbnailGridLayoutInput {
  containerWidth: number
  containerHeight: number
  zoomLevel: 1 | 2 | 3 | 4 | 5 | 6 | 7
  gapPx: number
  minCellSizePx: number
}

export interface ThumbnailGridLayoutResult {
  columns: number
  rows: number
  cellSizePx: number
  pageSize: number
}
```

首轮建议口径：

- `rows = zoomLevel`
- `columns` 由可用宽度与目标 `cellSize`/`gap` 推导
- `rows` 在极小高度下允许向下 clamp，但最低不低于 `1`
- `columns` 最低不低于 `1`
- `pageSize = rows * columns`

首轮不追求完全复刻旧仓算法；重点是先建立：

```text
containerSize -> layout -> pageSize
```

### 9.4 phase 完成后状态 check

- [ ] 已新增独立的缩略图布局纯计算文件
- [ ] 已有明确的 `zoomLevel -> rows` 首轮映射
- [ ] 已能输出 `columns / rows / cellSizePx / pageSize`
- [ ] 极小尺寸下不会出现 `0` 行或 `0` 列

---

## 10. Phase 2：接入 Main.header 缩放控件与容器尺寸观测

### 10.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `apps/desktop/src/app/AppShell.tsx`
- `apps/desktop/src/App.css`
- 源项目参考：
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\components\ImageMainSection.tsx`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\useAppEffects.ts`

### 10.2 todo 顺序

1. 在 `AppShell` 新增 `thumbnailZoomLevel` 状态
2. 在 `Main.header` 增加原生 `select` 控件
3. 在网格容器上加 `ref`
4. 用 `ResizeObserver` 写回容器尺寸状态
5. 基于容器尺寸 + 缩放级别计算布局结果

### 10.3 要做的内容

建议新增状态：

- `thumbnailZoomLevel`
- `mainGridSize`
- `thumbnailGridLayout`

建议控件语义：

- label：`缩略图级别`
- options：`1` 到 `7`
- 默认：`4`
- 会话级持久化：建议写入 `sessionStorage`

建议容器观测口径：

- 观测对象是 `Main.main` 中真正承载缩略图 grid 的元素
- 观测值只保留：
  - `width`
  - `height`
- 初始挂载与窗口 resize 后都要触发重算

### 10.4 phase 完成后状态 check

- [ ] `Main.header` 已出现 `1~7` 缩放下拉控件
- [ ] `thumbnailZoomLevel` 默认值为 `4`
- [ ] 网格容器尺寸已接入 `ResizeObserver`
- [ ] 容器尺寸变化会触发布局重算
- [ ] 缩放级别变化会触发布局重算

---

## 11. Phase 3：接入页面级分页读取与 Footer 翻页

### 11.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `apps/desktop/src/app/AppShell.tsx`
- `packages/contracts/src/models/items.ts`
- `apps/desktop/src/repositories/media-repository.ts`
- 源项目参考：
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\useImageBrowserViewModel.ts`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\buildMainFooter.tsx`

### 11.2 todo 顺序

1. 用 `pageIndex / pageSize` 替换固定 `page = 1 / pageSize = 12`
2. 计算 `totalPages`
3. 把 `Main.footer` 改成页面级 Prev / Next
4. 翻页后重新读取当前页条目
5. 切页时修正默认选中项

### 11.3 要做的内容

建议新增最小状态：

- `itemsPageIndex`

建议新增派生值：

- `itemsPageSize`
- `itemsTotalPages`
- `itemsTotalCountEstimate`

首轮可接受的简化：

- 当前后端 `items.list()` 只返回当前页条目，不返回总数
- 因此首轮 `totalPages` 可以用“当前页是否满页 + 当前页号”做临时推导，或在后续 phase 再补 count 输入面

更稳妥的首轮策略：

- 继续把 Footer 先实现为“页面 Prev / Next + 当前页号”
- 若当前页条目数 `< pageSize`，可判断为最后一页
- 不在本 phase 强求精确总页数 contract

选中项同步口径：

- 切页后默认选中新页第一个条目
- 若当前选中项仍存在于新页，则保留选中

### 11.4 phase 完成后状态 check

- [ ] `items.list()` 已吃 `pageIndex / pageSize`
- [ ] Footer 已从条目切换改为页面切换
- [ ] 翻页会重新拉取当前页条目
- [ ] 切页后选中项同步规则已固定
- [ ] 固定常量 `ITEMS_PREVIEW_LIMIT` 已不再承担主分页语义

---

## 12. Phase 4：联调缩放 / 分页 / 选中项同步并补样式

### 12.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `apps/desktop/src/app/AppShell.tsx`
- `apps/desktop/src/App.css`
- `docs/ui/ui-definition.md`

### 12.2 todo 顺序

1. 用计算结果驱动 grid 列模板
2. 调整 header 控件与标题布局
3. 调整 footer 分页区样式
4. 处理缩放变更时页码越界修正
5. 检查极窄 / 极矮窗口下的回退表现

### 12.3 要做的内容

建议样式改动方向：

- `Main.header`
  - 左侧保留真实标题
  - 右侧加入 `select`
- `Main.main`
  - 通过 inline style 或 CSS variable 注入：
    - `grid-template-columns`
    - `gap`
- `Main.footer`
  - 页面分页区与主路径信息并存

越界修正规则：

- 当缩放级别变化导致 `pageSize` 改变时：
  - 若原页码超出最后一页，则自动 clamp 到最后一页
- 当容器高度缩小导致 `rows/columns` 下降时，仍需保证：
  - 当前页不会变成空页

### 12.4 phase 完成后状态 check

- [ ] Grid 列模板已由布局结果驱动
- [ ] Header 标题与缩放控件布局稳定
- [ ] Footer 分页区与路径信息可同时工作
- [ ] 缩放或 resize 后不会落到非法页码
- [ ] 极窄窗口下仍能保持最小可用布局

---

## 13. Phase 5：验证、日志、文档回填

### 13.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `docs/ui/ui-definition.md`
- `docs/logs/20260312.md`
- `docs/README.md`

### 13.2 todo 顺序

1. 执行前端构建验证
2. 记录本批实际完成的 phase / check
3. 回填 UI 定义中的 `U2` 当前状态
4. 回填今日日志
5. 如新增主动维护文档入口，检查 docs 索引

### 13.3 要做的内容

至少执行：

- `npm run build:web`

必要时补充：

- `npm run tauri:dev`

文档回填要求：

- 本计划文档要把：
  - Phase 状态
  - check 勾选情况
  - 实际偏离点
  写回
- `docs/ui/ui-definition.md` 要同步记录：
  - `U2` 是否进入进行中
  - `Main.header` 缩放控件
  - `Main.main` 页面级缩略图网格
  - `Main.footer` 页面分页语义
- `docs/logs/20260312.md` 要记录：
  - 实际改动
  - 实际验证
  - 当前未做项

### 13.4 phase 完成后状态 check

- [ ] `npm run build:web` 已通过
- [ ] 本计划文档已回填 phase/check 状态
- [ ] `docs/ui/ui-definition.md` 已同步
- [ ] `docs/logs/20260312.md` 已同步

---

## 14. 新对话启动时的最小执行顺序

如果后续在新对话中继续推进本任务，默认按以下顺序加载上下文：

1. 先读：`docs/ui/u2-thumbnail-grid-phase-plan.md`
2. 再读：`docs/ui/ui-definition.md`
3. 再读：`apps/desktop/src/app/AppShell.tsx`
4. 再读：`apps/desktop/src/App.css`
5. 如果当前要做的是布局算法，再读源项目：
   - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\layout\thumbnailLayout.ts`
6. 如果当前要做的是分页 footer，再读源项目：
   - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\useImageBrowserViewModel.ts`
   - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\buildMainFooter.tsx`
7. 只做当前最前面的 `pending phase`

禁止在新对话中重新把任务放大成“完整复刻旧仓图片浏览器”；必须先把本计划中的首轮目标完成。

## 15. 当前待确认 / 待补充项

- 是否需要为 `items.list()` 补总数输入面，以获得精确 `totalPages`
- 缩放级别是否只做会话级持久化，还是要进入后续设置页
- `Main.main` 空态在 U2 进入进行中后是否要继续保留当前文案，还是切成更纯的工作区空板
- `Metadata` 是否需要在 U2 同步收紧为更贴近缩略图浏览上下文的详情结构

## 16. 当前结论

- 本批可直接开始，不需要等待后端新增分页能力
- 当前真正缺的是前端的容器驱动布局与页面级分页状态
- 首轮最重要的是先把 `ResizeObserver -> layout -> pageSize -> items.list(page,pageSize) -> footer` 串起来
- 等这条链稳定后，再按 `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md` 推进滚轮翻页、ready-commit 缓冲、gap snap、缩略图分辨率自适应等增强项
