# MediaPlayerNext UI 接入具体实施计划 I1-I4 v1

## 1. 文档定位

本文件不是替代 `docs/archive/root-plans/00-MediaPlayerNext_实施计划_v2.md`，而是在 `docs/archive/root-plans/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md` 已完成首版收口后，把 UI 接入阶段的前四个里程碑继续拆成可直接执行的实施清单。

五份文档的职责固定如下：

- `docs/archive/root-plans/00-MediaPlayerNext_实施计划_v2.md`
  - 负责总路线、工作包边界、接口策略与总体顺序
- `docs/archive/root-plans/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
  - 负责质量门禁、测试要求、迁移流程、benchmark 与发布约束
- `docs/archive/root-plans/02-MediaPlayerNext_后端先行具体实施计划_B1-B4_v1.md`
  - 负责 `B1-B4` 的数据地基、扫描闭环与 zip 主链路
- `docs/archive/root-plans/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md`
  - 负责 `B5-B8` 的缩略图、归一化、播放后端与字幕 sidecar 宿主协议
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md`
  - 负责 `I1-I4` 的 repository / adapter、缩略图列表、zip 浏览与媒体库全链路 UI 接入

本文件只覆盖 **UI 接入阶段的前四个里程碑**：

- `I1`：repository / adapter 接上
- `I2`：缩略图列表页可用
- `I3`：zip 浏览可用
- `I4`：媒体库全链路可用

不覆盖：

- `I5` 播放器接回
- `I6` 字幕接回
- `I7` 性能 / 包体 / 稳定性替代门槛收口
- theme 系统整包迁入
- 高级播放 UI、字幕编辑 UI、复杂批量交互

这样拆的原因是：`B1-B8` 已把后端服务层、协议输入面与 sidecar 宿主协议做成了可复用地基，接下来真正阻塞 UI 的不再是“后端能力是否存在”，而是 repository / adapter 收口、页面迁移顺序与前端消费方式是否稳定。

---

## 2. 已完成内容与当前进度

截至目前，后端先行阶段 `B1-B8` 已完成首版闭环，`P6` 也已完成首轮收口；`I1` 已进入 repository / adapter 与宿主接线阶段，`I2-I4` 仍未开始。

### 2.1 已完成内容（承接 `B1-B8`）

- Rust workspace、共享模型、错误码、任务状态、CLI harness 已稳定
- SQLite schema、migration、repository 已稳定可测
- `scan add-library/run/resume/stats/diff` 已跑通首版链路
- zip 目录读取、页序排序、entry 读取、`archive_entries` 落库与 `archive read-entry` 已具备首版能力
- Rust 缩略图主链路、`thumb://`、thumbnail fixture / validation 文档已具备
- `rar/7z -> zip` 归一化、retry 语义、fixture / validation 文档已具备
- `ffprobe / ffmpeg / mpv` 最小适配、`media://` / `archive://`、playback fixture / validation 文档已具备
- subtitle sidecar 的 `stdio + JSON` 协议、Rust host wrapper、fixture / validation 文档已具备
- `docs/benchmarks/backend-regression-20260307.md` 已形成 `B5-B8` 的统一回归收口记录

### 2.2 当前进度判断

- `I1`：进行中（`MediaRepository`、`tauriMediaRepository`、最小 app shell 与 `library/scan/items/archive/thumbnail.ensure/playback` 首轮宿主命令已接通，thumbnail progress 与真实页面迁移继续后补）
- `I2`：未开始（尚无真实媒体库选择、扫描入口、缩略图列表页）
- `I3`：未开始（尚无 archive entries UI 浏览页）
- `I4`：未开始（尚无媒体库全链路 UI，仍停留在最小宿主桥接演示）

### 2.3 当前最自然的下一步

1. 继续沿现有 `MediaRepository` 边界推进，不让 React 组件直接依赖 `invoke`
2. 在已接通 `library / scan / items / archive / thumbnail.ensure / playback` 的基础上，开始进入真实页面与 channel 补齐前的 UI 数据流验证

补充约束：

- 当前已确认“交互关系已收口，theme/CSS/页面内部调用链仍待收口”
- 因此页面级 `repository / command / protocol / event` 依赖矩阵可以先冻结
- 真实页面视觉、旧 theme 迁移与逐组件调用链复刻继续后补

---

## 3. 当前仓库基线

截至当前更新时，仓库实际状态如下：

- `apps/desktop/src/App.tsx` 仍是最小 `greet` command 演示，不承载真实媒体库 UI
- `packages/contracts` 已具备 `library / scan / items / archive / thumbnail / playback / subtitle` 的首轮 command model；channel/event 仍以扫描、缩略图、播放、字幕为主
- `src-tauri/src/lib.rs` 已注册：
  - `greet`
  - `runtime_smoke_check`
  - `subtitle.*`
  - `library.*`
  - `scan.*`
  - `items.*`
  - `archive.*`
  - `thumbnail.ensure`
  - `thumb://` / `media://` / `archive://` 协议
- `crates/app-core` 已形成可由前端首轮消费的 Tauri command 集合，宿主层仍保持薄接线
- `docs/benchmarks/` 已有 scan / archive / thumbnail / normalize / playback / sidecar / backend regression 记录

因此接下来的首要目标，不再是继续补后端骨架，而是把“后端已具备的能力”收口成前端可直接依赖的 repository / adapter 与页面级 UI 链路。

---

## 4. 执行原则

### 4.1 总原则

1. **先 repository，后页面**
   - 先把 UI 调用面收口到 `MediaRepository` 抽象，再开始迁移列表页、浏览页与详情页
2. **先 contracts，后 React hooks**
   - 先冻结 commands / channels / events / models，再写 hooks、providers 与页面状态管理
3. **先 URL 消费，后交互打磨**
   - 图片与归档页优先直接消费 `thumb://` / `media://` / `archive://`，不回退到 command 拉字节
4. **先图片/归档主链路，后播放/字幕 UI**
   - 当前阶段只推进 `I1-I4`，不提前把播放器与字幕 UI 混入同一批实施
5. **先可运行的垂直切片，后视觉完全迁入**
   - 优先做“能选库、能扫描、能看缩略图、能看 zip、能看基础详情”的可用链路，再处理复杂视觉细节

### 4.2 当前阶段禁止事项

- 不把 React 组件直接写成 `invoke(...)` 集合
- 不让前端继续通过 command 请求大块媒体字节
- 不为了“看起来完整”而一次性迁移全部页面与 theme
- 不把页面级数据整形、分页、缓存策略堆进 `src-tauri`
- 不在 `I1-I4` 阶段提前做复杂播放器 UI 或字幕编辑 UI

### 4.3 对 `B5-B8` 的继承要求

`I1-I4` 默认继承并复用以下既有基础：

- `thumb://` / `media://` / `archive://` 协议输入面
- `thumbnail.ensure`、`archive.*`、`playback.*`、`subtitle.*` 的后端首版能力
- `docs/fixtures/*` 与 `docs/benchmarks/*` 已有验证基线
- `docs/benchmarks/backend-regression-20260307.md` 的 `B5-B8` 回归结果

说明：`I1-I4` 的目标不是重新设计后端接口，而是围绕这些既有能力建立稳定的前端接入面。

---

## 5. 目标目录与职责落位

在 `I1-I4` 内，目录职责固定如下：

```text
MediaPlayerNext/
  apps/
    desktop/
      src/
        app/                        # app shell、providers、路由与全局状态入口
        repositories/               # MediaRepository 抽象与实现
        adapters/                   # Tauri command/channel/protocol 适配层
        features/
          library/                  # 媒体库选择、扫描入口、统计与状态
          items/                    # 列表、缩略图网格、基础筛选与分页
          archive/                  # zip 浏览页与页序导航
          viewer/                   # 基础图片查看与元数据详情
        components/                 # 跨页面 UI 组件（尽量薄）
        hooks/                      # repository 消费 hooks
  packages/
    contracts/                     # I1-I4 继续补齐 library/scan/items/archive/thumbnail 合同
  crates/
    app-core/                      # 继续承载 use case 编排，不下沉到前端
  src-tauri/                       # 只负责 command/channel/protocol 注册与接线
  docs/
    fixtures/                      # 可补 UI state / response snapshot 说明
    benchmarks/                    # 可补 UI integration validation / regression 记录
```

说明：

- `apps/desktop` 在 `I1` 开始时必须从“command demo”升级为“repository 驱动”的最小桌面壳
- `packages/contracts` 在 `I1-I2` 必须补齐 UI 主链路依赖的命令与模型，而不是只保留 playback / subtitle
- `src-tauri` 在 `I1-I4` 允许继续增加 command / channel wiring，但仍必须保持“薄宿主”原则

---

## 6. 阶段拆分总览

| 阶段 | 目标 | 核心交付物 | 是否阻塞后续 |
|---|---|---|---|
| `I1` | repository / adapter 接上 | `MediaRepository`、`tauriMediaRepository`、UI 所需 contracts 与 command 面 | 是 |
| `I2` | 缩略图列表页可用 | 媒体库选择、扫描入口、缩略图网格、`thumb://` 消费 | 是 |
| `I3` | zip 浏览可用 | archive 浏览页、页序导航、`archive://` 消费 | 是 |
| `I4` | 媒体库全链路可用 | 选库、扫描、列表、基础图片查看、archive 浏览、详情闭环 | 否（但阻塞 `I5/I6`） |

推荐顺序仍以串行为主：`I1 -> I2 -> I3 -> I4`。

说明：

- `I2-I4` 必须建立在 `I1` 已完成 repository / adapter 收口之上
- `I4` 的完成不等于播放器或字幕 UI 已接回，那是 `I5/I6` 的范围

---

## 7. I1：repository / adapter 接上

当前状态：进行中

## 7.1 阶段目标

把 `apps/desktop` 从“最小命令演示页”推进为“通过统一 repository 消费后端能力”的桌面壳，让后续页面迁移不再直接依赖 `invoke`、宿主命令名或协议拼接细节。

## 7.2 范围

### 本阶段要做

1. 在 `apps/desktop` 中定义 `MediaRepository` 抽象
2. 建立 `tauriMediaRepository`，统一封装 command / channel / protocol URL 消费
3. 补齐 UI 接入所需的首批 contracts：
   - `library.*`
   - `scan.*`
   - `items.*`
   - `archive.*`
   - `thumbnail.*`
4. 在 `src-tauri` 中补对应最小 command / channel 接线
5. 把 `apps/desktop/src/App.tsx` 从 demo 改成最小 app shell
6. 为后续页面建立统一状态入口、错误处理与 loading 约定

当前已落地：

- `apps/desktop/src/repositories/media-repository.ts`
- `apps/desktop/src/repositories/tauri-media-repository.ts`
- `apps/desktop/src/adapters/tauri/commands.ts`
- `apps/desktop/src/adapters/tauri/protocols.ts`
- `apps/desktop/src/app/AppShell.tsx`
- `apps/desktop/src/app/MediaRepositoryProvider.tsx`
- `docs/contracts/ui-dependency-matrix.md`

### 本阶段不做

- 不迁移真实页面视觉
- 不迁移旧仓 theme 系统
- 不做播放与字幕 UI
- 不引入与当前规模不匹配的重型状态管理框架

## 7.3 模块与文件计划

### `apps/desktop`

建议最小模块：

- `src/app/AppShell.tsx`
- `src/repositories/media-repository.ts`
- `src/repositories/tauri-media-repository.ts`
- `src/adapters/tauri/commands.ts`
- `src/adapters/tauri/channels.ts`
- `src/adapters/tauri/protocols.ts`

职责建议：

- `media-repository.ts`
  - 定义前端唯一依赖的接口抽象
- `tauri-media-repository.ts`
  - 负责 command / channel / protocol URL 的具体适配
- `commands.ts`
  - 封装 `invoke` 与参数/结果类型
- `channels.ts`
  - 封装高频进度订阅
- `protocols.ts`
  - 负责 `thumb://` / `media://` / `archive://` URL 构造

### `packages/contracts`

建议补首批合同：

- `commands/library.ts`
- `commands/scan.ts`
- `commands/items.ts`
- `commands/archive.ts`
- `commands/thumbnail.ts`
- `channels/scan-progress.ts`
- `channels/thumbnail-progress.ts`
- `events/library-events.ts`

### `src-tauri`

建议补首批 command：

- `library.list`
- `library.add`
- `scan.start`
- `scan.status`
- `scan.stats`
- `items.list`
- `archive.entries`
- `thumbnail.ensure`

## 7.4 交付物

- `MediaRepository` 抽象
- `tauriMediaRepository` 实现
- UI 接入首批 contracts
- 对应 Tauri command / channel 接线
- 最小 app shell

## 7.5 验收标准

- React 组件不再直接依赖 `invoke`
- `thumb://` / `media://` / `archive://` URL 构造集中在 adapter / repository 层
- `apps/desktop` 已可以通过 repository 获取库、扫描状态与列表数据
- `src-tauri` 新增接线仍保持“薄宿主”，不承载业务实现

当前首轮结果：

- `apps/desktop/src/App.tsx` 已不再直接依赖 `invoke`
- `AppShell` 已通过 repository 调用 `runtime_smoke_check` 与 `subtitle.*`
- `docs/contracts/ui-dependency-matrix.md` 已补基于既有交互关系的页面依赖矩阵
- `docs/benchmarks/i1-repository-shell-validation-20260308.md` 已记录 repository shell 首轮验证结果

## 7.6 本阶段必须补的测试

- contracts zod fixture / schema tests
- repository adapter 单元测试
- command 参数映射 tests
- app shell 基础 smoke tests（可从构建与最小渲染开始）

## 7.7 本阶段验证命令

```bash
npm run build:web
npm --workspace @mediaplayernext/desktop run lint
npm run test --workspace @mediaplayernext/contracts
scripts/run-cargo-with-msvc.cmd test --workspace
```

## 7.8 本阶段涉及文件与目录

优先会涉及：

- `apps/desktop/src/App.tsx`
- `apps/desktop/src/`
- `apps/desktop/package.json`
- `packages/contracts/src/index.ts`
- `packages/contracts/src/commands/`
- `packages/contracts/src/channels/`
- `packages/contracts/src/events/`
- `src-tauri/src/lib.rs`
- `src-tauri/src/bin/backend_harness.rs`
- `crates/app-core/src/lib.rs`
- `crates/app-core/src/ports.rs`
- `docs/benchmarks/`

## 7.9 本阶段完成后必须更新的 check 项

- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md`
  - `I1` 当前状态
  - `I1` 完成情况
  - `I1` 完成定义
- `docs/logs/<当天日期>.md`
- `packages/contracts/src/index.ts` 与对应 `tests` / `fixtures`
- `apps/desktop/README.md`（如 app shell 或运行方式有明显变化）
- `docs/benchmarks/` 下的 repository / adapter 验证记录

## 7.10 用于新对话启动的最小提示

当后续要直接开始 `I1` 开发时，可在新对话中只给下面这段提示：

```text
请按 `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 的 `7. I1` 章节开始开发，只聚焦 I1。

先读取：
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 中 `7. I1`
- `docs/archive/root-plans/00-MediaPlayerNext_实施计划_v2.md` 中 UI 接入阶段说明
- `docs/archive/root-plans/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
- `apps/desktop/src/App.tsx`
- `packages/contracts/src/index.ts`
- `src-tauri/src/lib.rs`

工作目标：
- 建 `MediaRepository` 与 `tauriMediaRepository`
- 补 library/scan/items/archive/thumbnail 首批 contracts
- 把 `apps/desktop` 从 command demo 推进为 repository 驱动的最小 app shell

约束：
- 不迁 theme
- 不让组件直接依赖 `invoke`
- 保持 `src-tauri` 极薄
```

---

## 8. I2：缩略图列表页可用

当前状态：未开始

## 8.1 阶段目标

在 `I1` 的 repository / adapter 基础上，先打通最有价值、最容易验证的 UI 垂直切片：媒体库选择、扫描入口与缩略图网格列表页。

## 8.2 范围

### 本阶段要做

1. 建立媒体库选择 / 创建入口
2. 建立扫描启动、扫描状态与扫描统计展示
3. 建立 items 列表与缩略图网格
4. 通过 `thumb://` 直接消费缩略图 URL
5. 建立空状态、错误状态、loading 状态
6. 建立最小分页或“加载更多”语义

### 本阶段不做

- 不做 archive 浏览页
- 不做图片详情页的复杂交互
- 不做视频播放控件
- 不做复杂筛选器、排序器与批量操作

## 8.3 模块与文件计划

建议最小模块：

- `src/features/library/LibraryPicker.tsx`
- `src/features/library/ScanPanel.tsx`
- `src/features/items/ItemsGridPage.tsx`
- `src/features/items/ThumbnailCard.tsx`
- `src/hooks/useLibraries.ts`
- `src/hooks/useScanStatus.ts`
- `src/hooks/useItems.ts`

建议补的 contracts / backend 面：

- `items.list`
- `thumbnail.ensure`
- `scan.start`
- `scan.status`
- `scan.stats`
- `library.list`

## 8.4 交付物

- 媒体库选择入口
- 扫描控制面板
- 缩略图网格页
- `thumb://` 在前端的直接消费链路

## 8.5 验收标准

- 可选择已有媒体库并显示基本统计
- 可触发扫描并观察状态变化
- items 列表可展示缩略图网格
- 缩略图不通过 JSON IPC 取字节，而是直接消费协议 URL
- 空库、空列表、扫描中、失败状态均可观测

## 8.6 本阶段必须补的测试

- repository 到页面状态的 hook tests
- 缩略图 URL 构造 tests
- 列表页最小渲染 smoke
- `build:web` 构建回归

## 8.7 本阶段验证命令

```bash
npm run build:web
npm --workspace @mediaplayernext/desktop run lint
scripts/run-cargo-with-msvc.cmd test --workspace
npm run tauri:dev
```

## 8.8 本阶段涉及文件与目录

- `apps/desktop/src/features/library/`
- `apps/desktop/src/features/items/`
- `apps/desktop/src/hooks/`
- `packages/contracts/src/commands/`
- `packages/contracts/src/channels/`
- `src-tauri/src/lib.rs`
- `docs/benchmarks/`

## 8.9 本阶段完成后必须更新的 check 项

- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md`
  - `I2` 当前状态
  - `I2` 完成情况
  - `I2` 完成定义
- `docs/logs/<当天日期>.md`
- `docs/benchmarks/` 下的 thumbnail list / UI integration validation 记录
- 若新增 fixtures：
  - `docs/fixtures/README.md`
  - 对应说明文件

## 8.10 用于新对话启动的最小提示

```text
请按 `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 的 `8. I2` 章节开始开发，只聚焦 I2。

先读取：
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 中 `8. I2`
- `apps/desktop/src/`
- `packages/contracts/src/index.ts`
- `src-tauri/src/lib.rs`
- `docs/fixtures/thumbnail-fixture/README.md`

工作目标：
- 建媒体库选择、扫描入口与缩略图网格页
- 通过 `thumb://` 直接消费缩略图
- 改动完成后更新当天日志与 UI integration validation 文档

约束：
- 不取大块图片字节
- 不迁 archive/page viewer
- 不迁播放器 UI
```

---

## 9. I3：zip 浏览可用

当前状态：未开始

## 9.1 阶段目标

把 `archive://` 与 `B4/B6` 已完成的 archive entry 链路真正接到桌面 UI，让 zip 图片浏览成为第一个可用的“详情页级”能力。

## 9.2 范围

### 本阶段要做

1. 建立 archive item 详情入口
2. 建立 archive entry 列表 / 页序导航
3. 通过 `archive://entry/<archive_entry_id>` 直接消费归档页图片
4. 建立页码、上一页、下一页与最小预加载策略
5. 补 archive 浏览状态与错误提示

### 本阶段不做

- 不做高级缩放、拖拽、双页模式
- 不做复杂快捷键层
- 不做 rar/7z 直接浏览（仍统一走归一化后 zip）

## 9.3 模块与文件计划

建议最小模块：

- `src/features/archive/ArchiveViewerPage.tsx`
- `src/features/archive/ArchiveStrip.tsx`
- `src/features/archive/PageNavigator.tsx`
- `src/hooks/useArchiveEntries.ts`

建议补的 contracts / backend 面：

- `archive.entries`
- `archive.entry-detail`
- `items.detail`

## 9.4 交付物

- zip 浏览页
- 页序导航
- `archive://` 前端直接消费链路
- archive UI integration validation 文档

## 9.5 验收标准

- 可从 items 列表进入 archive 浏览页
- 可按 page index 显示 archive 内图片
- 可用上一页 / 下一页切换
- 归档页图片通过 `archive://` 直接加载
- 归一化后的 zip 与原生 zip 在 UI 上保持统一入口

## 9.6 本阶段必须补的测试

- archive entry hook tests
- 页面切换状态 tests
- `archive://` URL 构造 tests
- archive viewer 构建 / smoke 验证

## 9.7 本阶段验证命令

```bash
npm run build:web
npm --workspace @mediaplayernext/desktop run lint
scripts/run-cargo-with-msvc.cmd test --workspace
npm run tauri:dev
```

## 9.8 本阶段涉及文件与目录

- `apps/desktop/src/features/archive/`
- `apps/desktop/src/hooks/`
- `packages/contracts/src/commands/archive.ts`
- `src-tauri/src/lib.rs`
- `docs/fixtures/archive-fixture/`
- `docs/benchmarks/`

## 9.9 本阶段完成后必须更新的 check 项

- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md`
  - `I3` 当前状态
  - `I3` 完成情况
  - `I3` 完成定义
- `docs/logs/<当天日期>.md`
- `docs/benchmarks/` 下的 archive viewer / UI integration validation 记录
- 若新增 fixtures：
  - `docs/fixtures/archive-fixture/README.md`

## 9.10 用于新对话启动的最小提示

```text
请按 `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 的 `9. I3` 章节开始开发，只聚焦 I3。

先读取：
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 中 `9. I3`
- `apps/desktop/src/`
- `packages/contracts/src/index.ts`
- `docs/fixtures/archive-fixture/README.md`
- `src-tauri/src/lib.rs`

工作目标：
- 建 archive 浏览页与页序导航
- 通过 `archive://` 直接显示归档页
- 改动完成后更新 archive viewer 验证记录

约束：
- 不做高级 viewer 交互
- 不让 UI 自己拼磁盘路径
- 不改播放与字幕 UI
```

---

## 10. I4：媒体库全链路可用

当前状态：未开始

## 10.1 阶段目标

把 `I1-I3` 已完成的 repository、缩略图网格与 archive 浏览收拢成一个真正可用的媒体库最小产品闭环：能选库、能扫描、能看列表、能看基础图片与 archive、能看详情与元数据。

## 10.2 范围

### 本阶段要做

1. 把 library / scan / items / archive / basic viewer 串成完整路由流
2. 建立 item 详情面与基础元数据显示
3. 对普通图片建立基础查看页，直接消费 `media://asset/<asset_id>`
4. 建立最小导航、面包屑或返回流
5. 统一错误提示、空状态、loading 状态与基础布局
6. 补首份“媒体库 UI 全链路可用”验证记录

### 本阶段不做

- 不做视频播放 UI
- 不做字幕 UI
- 不做复杂高级筛选、批量编辑、可视化扩展
- 不做最终视觉与 theme 完全迁移

## 10.3 模块与文件计划

建议最小模块：

- `src/features/viewer/ImageViewerPage.tsx`
- `src/features/items/ItemDetailPage.tsx`
- `src/app/routes.tsx`
- `src/app/providers/`
- `src/components/Layout/`

建议补的 contracts / backend 面：

- `items.detail`
- `media.resolve`（若前端需要稳定 URL 汇总 DTO）
- `library.current`

## 10.4 交付物

- 媒体库最小路由流
- 普通图片基础查看页
- item 详情与元数据面板
- 全链路 UI integration validation 文档

## 10.5 验收标准

- 可从选库到扫描、从列表到详情、从详情到基础图片 / archive 浏览完成闭环
- 普通图片通过 `media://` 直接显示
- archive item 通过 `archive://` 直接显示
- 元数据与缩略图、archive 浏览处于同一套 repository / adapter 边界下
- 当前仓库已达到“媒体库全链路可用，但播放器与字幕 UI 未接回”的状态

## 10.6 本阶段必须补的测试

- 路由闭环 smoke
- viewer URL 构造 tests
- item detail 状态 tests
- 全链路构建 / 验证记录

## 10.7 本阶段验证命令

```bash
npm run build:web
npm --workspace @mediaplayernext/desktop run lint
scripts/run-cargo-with-msvc.cmd test --workspace
npm run tauri:dev
```

## 10.8 本阶段涉及文件与目录

- `apps/desktop/src/app/`
- `apps/desktop/src/features/viewer/`
- `apps/desktop/src/features/items/`
- `apps/desktop/src/components/`
- `packages/contracts/src/commands/`
- `src-tauri/src/lib.rs`
- `docs/benchmarks/`

## 10.9 本阶段完成后必须更新的 check 项

- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md`
  - `I4` 当前状态
  - `I4` 完成情况
  - `I4` 完成定义
- `docs/logs/<当天日期>.md`
- `docs/benchmarks/` 下的 media library UI validation 记录
- 若更新运行方式或桌面壳说明：
  - `README.md`
  - `apps/desktop/README.md`

## 10.10 用于新对话启动的最小提示

```text
请按 `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 的 `10. I4` 章节开始开发，只聚焦 I4。

先读取：
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 中 `10. I4`
- `apps/desktop/src/`
- `packages/contracts/src/index.ts`
- `src-tauri/src/lib.rs`
- `docs/benchmarks/backend-regression-20260307.md`

工作目标：
- 建媒体库全链路最小 UI 闭环
- 打通普通图片查看与 archive 浏览
- 改动完成后更新全链路 UI validation 文档

约束：
- 不迁播放器 UI
- 不迁字幕 UI
- 保持 repository / adapter 边界稳定
```

---

## 11. 阶段间依赖关系

### `B8 -> I1`

- 没有稳定的 command / channel / protocol / sidecar 边界，前端 repository 很容易直接绑定到底层实现细节

### `I1 -> I2`

- 没有统一 repository / adapter，缩略图列表页会快速退化成散落的 `invoke` 调用集合

### `I2 -> I3`

- 没有稳定的 items 列表、缩略图入口与基础导航，archive 浏览页缺少可复用的选择与跳转基础

### `I2/I3 -> I4`

- 没有列表页与 archive 浏览页，媒体库全链路 UI 无法形成真正可用的垂直切片

因此当前阶段不建议跳过 `I1/I2` 直接做播放器或字幕 UI。

---

## 12. fixture、golden、benchmark 计划

## 12.1 fixture 目录建议

```text
docs/fixtures/
  thumbnail-fixture/
  archive-fixture/
  playback-fixture/
  sidecar-fixture/
  ui-fixture/
```

### `ui-fixture`

- repository response snapshot
- library list / scan stats / items list 的示例 JSON
- 缩略图网格、archive viewer、image viewer 的状态说明
- 当前完成情况：未开始

## 12.2 当前阶段要产出的 snapshot

- repository response snapshot
- 列表页 state snapshot
- archive viewer route/state snapshot
- image viewer route/state snapshot

说明：

- `I1` 起开始正式建立 repository response snapshot
- `I2-I4` 的 snapshot 更偏 UI state 与路由状态，不要求提交大量 UI 二进制产物

## 12.3 benchmark / validation 计划

`I1-I4` 至少记录以下基线：

- repository / adapter 验证记录
- `build:web` 回归记录
- 缩略图列表页加载验证记录
- archive 浏览页加载验证记录
- media library 全链路 UI validation 记录

---

## 13. 与 `src-tauri` 的关系

在 `I1-I4` 阶段，`src-tauri` 会继续承担接线职责，但仍然不是主业务开发面。

允许的改动：

- 注册 `library.*` / `scan.*` / `items.*` / `archive.*` / `thumbnail.*` 最小命令与 channel
- 维持与扩展 `thumb://` / `media://` / `archive://` 协议
- 维持 runtime checks

不允许的改动：

- 在 Tauri command 里直接写分页、过滤、页面状态拼装逻辑
- 在 protocol handler 里直接写业务规则或 UI 状态转换
- 为了调试方便把图片、归档页字节重新塞回 JSON IPC

`src-tauri` 在当前阶段的职责是：

- 把 crate 层能力暴露成前端 repository 可消费的命令、channel 与 protocol
- 不成为 React 页面状态逻辑的承载层

---

## 14. 每阶段完成定义（Definition of Done）

## `I1` 完成定义

- `MediaRepository` 与 `tauriMediaRepository` 已建立
- `apps/desktop` 不再是直接 `invoke` demo
- UI 接入首批 contracts 与 command 面可用

当前状态：未开始

## `I2` 完成定义

- 媒体库选择与扫描入口可用
- 缩略图网格页可用
- `thumb://` 已被真实 UI 直接消费

当前状态：未开始

## `I3` 完成定义

- zip 浏览页可用
- archive entries 页序导航可用
- `archive://` 已被真实 UI 直接消费

当前状态：未开始

## `I4` 完成定义

- 媒体库从选库到详情形成最小闭环
- 普通图片查看与 archive 浏览均可用
- 仍未接回播放器与字幕 UI

当前状态：未开始

---

## 15. 当前建议的实际执行顺序

如果从当前仓库立即继续开工，建议严格按以下顺序落地：

### 第 9 批：`I1`

1. 冻结前端 repository / adapter 边界
2. 补 library / scan / items / archive / thumbnail 合同
3. 把 `apps/desktop` 从 demo 推进为 app shell

### 第 10 批：`I2`

1. 建媒体库选择入口
2. 建扫描控制与状态展示
3. 建缩略图网格页
4. 用 `thumb://` 直接显示缩略图

### 第 11 批：`I3`

1. 建 archive 浏览页
2. 建页序导航
3. 用 `archive://` 直接显示归档页

### 第 12 批：`I4`

1. 建 item 详情页与基础元数据面
2. 建普通图片查看页
3. 把选库、扫描、列表、详情、archive 浏览收成最小闭环

完成以上四批后，再进入 `I5` 播放器接回、`I6` 字幕接回或单独编写 `I5-I7` 的实施文档。

---

## 16. 最终执行结论

当前仓库在 `B1-B8` 已完成首版后，UI 接入阶段的正确落地方式不是：

- 直接开始 theme 大迁移
- 直接开始播放器 UI 或字幕 UI
- 直接在 React 组件里到处写 `invoke`

而是：

1. 先完成 `I1` repository / adapter 收口
2. 再完成 `I2` 缩略图列表页
3. 再完成 `I3` zip 浏览页
4. 再完成 `I4` 媒体库全链路最小闭环

完成这四步后，后续 UI 才会真正具备：

- 只依赖稳定 repository 抽象的能力
- 直接消费 `thumb://` / `media://` / `archive://` URL 的能力
- 在不推翻当前后端边界的前提下继续接回播放器与字幕 UI 的能力
- 可验证、可回归、可逐步替换旧 Electron 接口的迁移路径
