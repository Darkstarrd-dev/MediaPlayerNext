# U2 缩略图网格增强项实施计划

## 1. 文档定位

本文件用于指导 `U2` 基础版完成之后的增强项实施，覆盖以下 4 条增强链路：

- 滚轮翻页与页码预览
- `ready-commit` 翻页缓冲
- `gap snap` 网格贴合吸附
- 缩略图分辨率自适应

本文件是执行型计划文档，按 phase 拆分工作；每个 phase 都明确：

- 开始前必须读取的文件
- todo 顺序
- 要做的内容
- 完成后需要回填的状态 check

本文件本身应包含在新对话中继续推进该任务所需的最小上下文，避免再次做整轮源项目考古。

## 2. 生效前提

本文件不是 `U2` 基础版的替代文档，而是其后续增强计划。

只有在以下前提至少基本成立后，才建议开始执行本文件：

- `Main.header` 已有 `1~7` 缩放控件
- `Main.main` 已经接上容器驱动的分页网格
- `pageSize = columns * rows` 已成立
- `Main.footer` 已切到页面级分页
- `items.list({ page, pageSize })` 已成为主读取链路

对应基础实施文档：

- `docs/ui/u2-thumbnail-grid-phase-plan.md`

如果基础版还没完成，应先回到上面的基础实施文档，只做最前面的 `pending phase`。

## 3. 本批增强项目标

### 3.1 本批要做的内容

- 为缩略图容器增加滚轮翻页与页码预览
- 为翻页切换增加 `ready-commit` 缓冲，减少白屏和布局抖动
- 为主区宽度与网格列数之间增加 `gap snap` 吸附逻辑
- 为缩略图请求尺寸增加分辨率自适应计算
- 明确各增强链路的触发时机、状态边界与验证方式

### 3.2 本批明确不做的内容

- 不做虚拟网格 (grid virtualization)
- 不做无限滚动
- 不做多页并行预加载缓存中心的完整系统
- 不做后端缩略图生成算法重写
- 不做完整旧仓播放器 / metadata 管理态联动
- 不在本批处理真实后端节点树

## 4. 源项目已确认结论

以下结论已经根据源项目 `Z:\Playground\CurrentWorking\MediaPlayerX` 整理完成，后续不必重新全量探索。

### 4.1 滚轮翻页

- 缩略图容器支持滚轮翻页
- 滚轮不是每次事件都直接翻页，而是：
  - 先累积 delta
  - 进入页码预览状态
  - 等 settle 后才真正 commit 页面切换
- 这样做的目的不是动画，而是防止高频滚轮输入导致页码连续抖动与误翻页

### 4.2 ready-commit 缓冲

- 当前页切换时不是直接清空旧页
- 旧页内容会保留到新页 ready 后再 commit
- 这条链路的目标是减少翻页瞬间的白屏和布局跳变

### 4.3 gap snap

- 网格右侧如果留下多余空白，会尝试把 `Sidebar/Main/Metadata` 的分割比例吸附到更贴合网格列的位置
- `gap snap` 不在拖动 splitter 过程中实时触发
- 只在以下节点触发：
  - splitter commit 后
  - 窗口尺寸变化后
  - 缩放级别变化后
  - 初始挂载后

### 4.4 缩略图分辨率自适应

- 这条链路和“网格布局自适应”不是同一件事
- 网格布局决定：
  - `columns`
  - `rows`
  - `cellWidth`
  - `pageSize`
- 分辨率自适应决定：
  - 实际请求的缩略图最大边尺寸
- 核心输入是：
  - `devicePixelRatio`
  - `actualCellWidth`
  - `actualMediaHeight`
- 核心目标是：在高 DPI 和较大格子下请求更合适的缩略图尺寸，避免模糊或过度浪费

## 5. 源项目关键文件（按增强项归类）

### 5.1 滚轮翻页 / 页码预览

- `src/components/ImageMainSection.tsx`
- `src/features/app/useWorkspaceNodeBrowsePaging.ts`
- `docs/05-interaction-v1.md`

### 5.2 ready-commit 缓冲

- `src/components/ImageMainSection.tsx`
- `docs/05-interaction-v1.md`

### 5.3 gap snap

- `src/features/app/useAppNavigationState.ts`
- `docs/05-interaction-v1.md`

### 5.4 缩略图分辨率自适应

- `src/features/app/useResolvedMediaState.ts`
- `src/features/backend/mediaResolveUtils.ts`
- `electron/fileSystemThumbnailResolver.ts`
- `docs/24-high-optimization-demand-table.md`
- `docs/17-thumb_acceleration_implementation_plan.md`

## 6. 新仓当前假设与边界

执行本文件时，默认新仓 `MediaPlayerNext` 已具备以下基础能力：

- `Main` 已有缩略图分页网格
- `thumbnailZoomLevel` 已存在
- 容器尺寸观测已存在
- 页面级 Footer 翻页已存在
- 前端已可调用：
  - `repository.items.list()`
  - `repository.thumbnail.ensure()`
  - `repository.urls.thumbnail()`

执行本文件时，默认仍保持以下边界：

- Root 级不允许滚动
- 三列 `header / footer` 保持固定高度
- 主工作区依然是分页整页渲染，而不是虚拟网格
- 导入/扫描状态继续集中在 `Header Logo + ImportTaskPanel`

## 7. 建议文件落点

为避免继续把所有增强项都塞回 `AppShell.tsx`，建议在增强阶段进一步拆分纯逻辑。

建议文件落点：

- `apps/desktop/src/app/AppShell.tsx`
  - 继续承载主状态接线与 repository 调用
- `apps/desktop/src/app/thumbnail-grid-layout.ts`
  - 基础布局算法，继续复用
- `apps/desktop/src/app/thumbnail-grid-enhancements.ts`
  - 建议新增：页码预览、settle、snap、分辨率计算等纯函数与类型
- `apps/desktop/src/App.css`
  - 预览态、缓冲态、分页提示态、控件样式

说明：

- 如果实现时发现 `thumbnail-grid-enhancements.ts` 过大，可按增强项继续拆成 2~3 个小文件
- 首轮不强求一次性抽出完整 hooks 库，但要避免把纯计算硬塞进 JSX 里

## 8. Phase 总览

| Phase | 目标 | 状态 |
|---|---|---|
| `Phase 0` | 冻结增强项边界与前置条件 | `done` |
| `Phase 1` | 滚轮翻页与页码预览状态链 | `done` |
| `Phase 2` | ready-commit 翻页缓冲 | `done` |
| `Phase 3` | gap snap 吸附策略 | `pending` |
| `Phase 4` | 缩略图分辨率自适应 | `done` |
| `Phase 5` | 联调、验证、文档回填 | `in-progress` |

---

## 9. Phase 0：冻结增强项边界与前置条件

### 9.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md`
- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `docs/ui/ui-definition.md`
- `docs/README.md`

### 9.2 todo 顺序

1. 冻结增强项的 4 条主链路
2. 冻结基础版前置条件
3. 冻结建议文件落点
4. 把本计划文档加入 docs 索引

### 9.3 要做的内容

- 明确本文件只在基础版完成后执行
- 明确本批不是“完整复刻旧仓所有图片浏览器体验”，而是优先补 4 条最关键增强链路
- 明确后续新对话优先从本文件和基础版文档读取最小上下文

### 9.4 phase 完成后状态 check

- [x] 增强项范围已冻结
- [x] 基础版前置条件已写清楚
- [x] docs 索引可找到本计划文档

---

## 10. Phase 1：滚轮翻页与页码预览状态链

### 10.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md`
- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `apps/desktop/src/app/AppShell.tsx`
- `apps/desktop/src/App.css`
- 源项目参考：
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\components\ImageMainSection.tsx`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\useWorkspaceNodeBrowsePaging.ts`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\docs\05-interaction-v1.md`

### 10.2 todo 顺序

1. 为缩略图容器补 wheel 事件入口
2. 新增滚轮 delta 累积状态
3. 新增页码预览状态
4. 新增 settle 计时与真正翻页 commit
5. 给 footer 或主区补当前预览页提示

### 10.3 要做的内容

建议新增最小状态：

- `wheelPagingDelta`
- `wheelPreviewPageIndex`
- `wheelPreviewActive`
- `wheelPreviewDirection`
- `wheelCommitTimer`

建议交互口径：

- 容器滚轮事件先 `preventDefault()`，不让浏览器滚动主区
- 先累积 `deltaY`
- 达到阈值后只更新“预览页码”，不立刻读新页
- 在短暂 settle 后，再统一执行一次真正翻页
- 当已经在第一页或最后一页时，预览与 commit 都必须 clamp

建议首轮阈值策略：

- 使用固定阈值，例如 `80~120` 像素区间
- 将阈值写成常量，避免先做用户设置项

### 10.4 phase 完成后状态 check

- [x] 缩略图容器已接上滚轮事件入口
- [x] 滚轮输入不会导致主区原生滚动
- [x] 滚轮可进入页码预览态
- [x] settle 后才真正翻页 commit
- [x] 到达边界页时不会出现非法页码

---

## 11. Phase 2：ready-commit 翻页缓冲

### 11.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md`
- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `apps/desktop/src/app/AppShell.tsx`
- 源项目参考：
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\components\ImageMainSection.tsx`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\docs\05-interaction-v1.md`

### 11.2 todo 顺序

1. 明确“展示页”和“目标页”两套状态
2. 翻页时保留旧页内容
3. 新页数据 ready 后再 commit 到显示层
4. 处理中途重复翻页与竞态取消
5. 给缓冲期补最小可感知状态

### 11.3 要做的内容

建议新增最小状态：

- `displayedPageIndex`
- `targetPageIndex`
- `pageTransitionState`
- `pageRequestIdRef`
- `pendingItems`

建议状态语义：

- `displayedPageIndex`
  - 当前真正渲染在屏幕上的页
- `targetPageIndex`
  - 用户操作后希望切到的页
- `pageTransitionState`
  - `idle | loading-next-page | committing`

建议交互边界：

- 用户请求翻页后：
  - 旧页继续显示
  - footer / 预览提示可显示目标页
- 新页数据 ready 后：
  - 再切换到新页内容
- 如果用户在新页 ready 前再次翻页：
  - 前一次请求失效
  - 只保留最后一次目标页

### 11.4 phase 完成后状态 check

- [x] 翻页时不会先清空旧页
- [x] 新页 ready 后才 commit 到显示层
- [x] 重复翻页时旧请求可被取消或失效
- [x] footer / 预览态能反映目标页与显示页差异
- [x] 白屏与闪烁明显减少

---

## 12. Phase 3：gap snap 吸附策略

### 12.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md`
- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `apps/desktop/src/app/AppShell.tsx`
- `apps/desktop/src/App.css`
- 源项目参考：
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\useAppNavigationState.ts`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\docs\05-interaction-v1.md`

### 12.2 todo 顺序

1. 明确 snap 触发时机
2. 根据当前 layout 计算“更贴合列数”的 main 宽度候选值
3. splitter commit 后尝试吸附
4. resize / zoom / mount 后尝试吸附
5. 排除拖动中的实时吸附

### 12.3 要做的内容

建议新增纯计算：

- `computeGapSnapTargetWidth(...)`
- `shouldApplyGapSnap(...)`

建议触发时机固定为：

- splitter 拖动结束后
- 窗口 resize 后
- 缩放级别变化后
- 首次挂载完成后

建议首轮不触发的时机：

- splitter 正在拖动时
- wheel 预览过程中
- page request 尚未稳定时

建议吸附判断原则：

- 吸附只在“当前剩余空白足够明显”时触发
- 不能为了贴合列数而让 Metadata / Sidebar 被挤压到不可用宽度
- 吸附结果必须继续服从现有 `Sidebar / Main / Metadata` 最小宽度边界

### 12.4 phase 完成后状态 check

- [ ] 已明确并实现 snap 触发时机
- [ ] splitter 拖动中不会实时 snap
- [ ] resize / zoom / mount 后可触发 snap
- [ ] snap 不会破坏三列最小宽度边界
- [ ] 网格右侧多余空白明显减少

---

## 13. Phase 4：缩略图分辨率自适应

### 13.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md`
- `apps/desktop/src/app/AppShell.tsx`
- `apps/desktop/src/repositories/media-repository.ts`
- 源项目参考：
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\useResolvedMediaState.ts`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\backend\mediaResolveUtils.ts`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\electron\fileSystemThumbnailResolver.ts`
  - `Z:\Playground\CurrentWorking\MediaPlayerX\docs\24-high-optimization-demand-table.md`

### 13.2 todo 顺序

1. 明确请求尺寸计算公式
2. 在前端把当前格子尺寸转换为缩略图请求尺寸
3. 为不同 zoomLevel / DPR 复用或区分缓存键策略
4. 检查高 DPI 下的清晰度与请求放大幅度
5. 给超大请求增加上限保护

### 13.3 要做的内容

建议首轮公式：

```text
thumbnailMaxEdge = ceil(devicePixelRatio * max(actualCellWidth, actualMediaHeight))
```

建议首轮边界：

- `devicePixelRatio` 最低按 `1`
- 给 `thumbnailMaxEdge` 加上限，避免异常容器尺寸导致过大请求
- 只有当新请求尺寸明显高于当前缓存尺寸时，才考虑重新 ensure

当前新仓需要重点确认的点：

- `thumbnail.ensure(assetId, profile)` 目前是 profile 语义，不是自由像素尺寸接口
- 因此本 phase 可能需要先做“前端 adaptive profile 映射”，而不是直接复刻旧仓的 `maxEdge` 输入面

建议首轮降级策略：

- 如果当前后端 contracts 还不适合接自由尺寸，就先建立：
  - `grid-sm / grid-md / detail-md / detail-lg`
  与当前计算尺寸之间的映射规则
- 等未来 contracts 扩展后，再演进到更精细的自由尺寸请求

### 13.4 phase 完成后状态 check

- [x] 已固定首轮分辨率自适应计算规则
- [x] 高 DPI 下的缩略图请求已比固定低档位更合理
- [x] 已有过大请求的上限保护
- [x] 当前 contracts 无法直接支持自由尺寸时，已定义清晰的 profile 映射策略
- [x] 文档中已写明这条链路和网格布局自适应是两回事

---

## 14. Phase 5：联调、验证、文档回填

### 14.1 开始前读取文件

- `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md`
- `docs/ui/u2-thumbnail-grid-phase-plan.md`
- `docs/ui/ui-definition.md`
- `docs/logs/20260313.md`
- `docs/README.md`

### 14.2 todo 顺序

1. 按增强项逐条手动联调
2. 执行前端构建验证
3. 必要时执行桌面手动点验
4. 回填本计划文档 phase/check
5. 回填 UI 定义与今日日志

### 14.3 要做的内容

至少执行：

- `npm run build:web`

必要时补充：

- `npm run tauri:dev`
- `npm run e2e:desktop`

建议手动点验清单：

- 连续滚轮输入时是否只在 settle 后真正翻页
- 翻页过程中是否还能稳定保留旧页
- resize / zoom 后是否会自动 snap 到更贴合的宽度
- 高 DPI 或较大格子下缩略图是否明显更清晰

文档回填要求：

- 本计划文档必须回填：
  - phase 状态
  - 每项 check 勾选情况
  - 实际偏离点
- `docs/ui/ui-definition.md` 需要同步记录：
  - wheel paging
  - ready-commit 缓冲
  - gap snap
  - 分辨率自适应
- `docs/logs/20260313.md` 需要同步记录：
  - 实际改动
  - 实际验证
  - 未做项

### 14.4 phase 完成后状态 check

- [x] `npm run build:web` 已通过
- [x] `npm run e2e:desktop` 已通过（`Spec Files: 4 passed, 4 total`）
- [x] 本计划文档已回填 phase/check 状态
- [x] `docs/ui/ui-definition.md` 已同步增强项口径
- [x] `docs/logs/20260313.md` 已同步增强项记录

---

## 15. 新对话启动时的最小执行顺序

如果后续在新对话中继续推进增强项，默认按以下顺序加载上下文：

1. 先读：`docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md`
2. 再读：`docs/ui/u2-thumbnail-grid-phase-plan.md`
3. 再读：`docs/ui/ui-definition.md`
4. 再读：`apps/desktop/src/app/AppShell.tsx`
5. 再读：`apps/desktop/src/App.css`
6. 如果当前要做滚轮或 ready-commit：再读源项目
   - `Z:\Playground\CurrentWorking\MediaPlayerX\src\components\ImageMainSection.tsx`
   - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\useWorkspaceNodeBrowsePaging.ts`
7. 如果当前要做 gap snap：再读源项目
   - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\useAppNavigationState.ts`
8. 如果当前要做分辨率自适应：再读源项目
   - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\app\useResolvedMediaState.ts`
   - `Z:\Playground\CurrentWorking\MediaPlayerX\src\features\backend\mediaResolveUtils.ts`
9. 只做本文件中最前面的 `pending phase`

禁止在新对话中重新把任务放大成“完整复刻旧仓所有主区行为”；必须先按本计划一条增强链路一条增强链路推进。

## 16. 当前待确认 / 待补充项

- `ready-commit` 是只作用于翻页，还是将来也要作用于节点切换
- `wheel paging` 的阈值是否要暴露到设置页，还是永远保持内部常量
- `gap snap` 是否只作用于 Main 宽度，还是将来也要对 Sidebar / Metadata 更细粒度处理
- 当前 thumbnail contracts 是否需要从 `profile` 语义继续演进到自由尺寸请求

## 17. 当前结论

- 本文件已经足以支撑后续新对话继续推进增强项，不需要重新整理源项目上下文
- 4 条增强链路里，最适合先做的是：
  1. 滚轮翻页与页码预览
  2. ready-commit 缓冲
  3. gap snap
  4. 分辨率自适应
- 这个顺序的原因是：前两者直接改善交互连续性，第三项改善布局贴合感，第四项再进一步补清晰度与性能平衡
