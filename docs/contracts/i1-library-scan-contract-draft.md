# I1 Library / Scan Contract Draft

## 文档定位

本文件是 `P6-3` 下的首批细化草案。

目标：

- 把 `library` / `scan` 两组最先要补的 contracts 先细化到可直接开文件施工
- 不依赖旧 UI 最终页面结构
- 只冻结 `I1` 最小 repository / adapter 需要的输入输出面

## 设计依据

- `crates/app-core/src/scan.rs`
- `crates/shared-model/src/records.rs`
- `crates/shared-model/src/tasks.rs`
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md:195`
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md:242`

## 一、library contracts 草案

### 建议新增文件

- `packages/contracts/src/commands/library.ts`
- `packages/contracts/src/models/library.ts`
- `packages/contracts/src/events/library-events.ts`

### 建议命令面

| command | 用途 | 当前后端依据 |
|---|---|---|
| `library.list` | 列出现有媒体库 | `LibraryRecord` |
| `library.add` | 注册媒体库根路径 | `register_library(...)` |
| `library.get` | 读取单个媒体库详情 | `LibraryRecord` |
| `library.remove` | 删除/移除媒体库 | 当前后端未实现，先保留占位 |

### 建议模型

#### `LibrarySummary`

```ts
type LibrarySummary = {
  id: string
  rootPath: string
  libraryType: string
  scanMode: string
  createdAt: string
  updatedAt: string
}
```

说明：

- 当前可直接映射 `shared_model::LibraryRecord`
- `I1` 首轮不额外引入页面专属派生字段

#### `AddLibraryInput`

```ts
type AddLibraryInput = {
  rootPath: string
}
```

说明：

- 当前 `register_library(...)` 只需要根路径
- `libraryType` / `scanMode` 暂不开放给前端自定义，先由后端固定为 `filesystem/full`

### 建议响应

- `library.list` -> `LibrarySummary[]`
- `library.add` -> `LibrarySummary`
- `library.get` -> `LibrarySummary`
- `library.remove` -> `void | { id: string }`

## 二、scan contracts 草案

### 建议新增文件

- `packages/contracts/src/commands/scan.ts`
- `packages/contracts/src/channels/scan-progress.ts`
- `packages/contracts/src/events/scan-events.ts`
- 如有必要：`packages/contracts/src/models/scan.ts`

### 建议命令面

| command | 用途 | 当前后端依据 |
|---|---|---|
| `scan.start` | 启动扫描 | `run_scan(...)` |
| `scan.resume` | 恢复扫描 | `resume_scan(...)` |
| `scan.stats` | 读取统计 | `scan_stats(...)` |
| `scan.snapshot` 或 `items.list` 前置替代 | 读取扫描结果快照 | `scan_snapshot(...)` |

说明：

- `scan.status` 若只读任务状态，可直接复用 `tasks` 域，不一定单独成为长期命令名
- `scan.diff` 当前更偏开发/回归口径，不一定进入 `I1` 最小 UI 面

### 建议模型

#### `ScanRunResult`

```ts
type ScanRunResult = {
  libraryId: string
  discovered: number
  insertedOrUpdated: number
  skippedUnchanged: number
  tombstoned: number
  taskId: string
}
```

对应：`shared_model::ScanRunSummary` 当前在 `app-core` 内部已存在同形结构。

#### `ScanStats`

```ts
type ScanStats = {
  libraryId: string
  sourceCount: number
  activeSourceCount: number
  missingSourceCount: number
}
```

对应：`shared_model::ScanStatsSummary` 当前在 `app-core` 内部已存在同形结构。

#### `ScanProgressEvent`

建议直接基于 `TaskProgress` 首轮收口：

```ts
type ScanProgressEvent = {
  taskId: string
  taskType: "scan"
  state: "queued" | "running" | "completed" | "failed" | "cancelled"
  current: number
  total?: number
  message?: string
  errorCode?: string
}
```

说明：

- `I1` 首轮不必先发明 scan 专属进度对象
- 可以先在 contracts 层明确：scan progress 只是 `TaskProgress` 的 scan 特化视图

### 建议事件

- `scan.finished`
- `scan.failed`

首轮可只保留事件名和最小 payload：

```ts
type ScanFinishedEvent = {
  libraryId: string
  taskId: string
}
```

## 三、建议文件职责草案

### `packages/contracts/src/commands/library.ts`

- `libraryListRequestSchema`
- `libraryAddRequestSchema`
- `libraryGetRequestSchema`
- `libraryRemoveRequestSchema`

### `packages/contracts/src/models/library.ts`

- `librarySummarySchema`
- `addLibraryInputSchema`

### `packages/contracts/src/commands/scan.ts`

- `scanStartRequestSchema`
- `scanResumeRequestSchema`
- `scanStatsRequestSchema`
- `scanSnapshotRequestSchema`

### `packages/contracts/src/channels/scan-progress.ts`

- `scanProgressEventSchema`

### `packages/contracts/src/events/library-events.ts`

- `libraryChangedEventSchema`

### `packages/contracts/src/events/scan-events.ts`

- `scanFinishedEventSchema`
- `scanFailedEventSchema`

## 四、I1 首轮建议不先做的内容

- library 分组/标签/排序偏好
- scan 历史记录列表页专属 DTO
- diff 级别的复杂比较结果模型
- 页面级筛选条件与聚合统计对象

## 五、建议施工顺序

1. 先补 `models/library.ts`
2. 再补 `commands/library.ts`
3. 再补 `commands/scan.ts` 与 `channels/scan-progress.ts`
4. 最后补 `events/library-events.ts` / `events/scan-events.ts`

这样安排的原因：

- `library + scan` 是 `I1` 最先要消费的能力
- 模型先行可以减少后续 `command/event` 重复改名
- `event` 在首轮里优先级低于 `command/channel`
