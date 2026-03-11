# I1 Items / Archive / Thumbnail Contract Draft

## 文档定位

本文件是 `P6-3` 下第二批细化草案。

目标：

- 把 `items` / `archive` / `thumbnail` 三组 `I1` 需要的 contracts 继续细化到可直接开文件施工
- 仍然只覆盖后端边界，不绑定旧 UI 最终页面结构

## 设计依据

- `crates/app-core/src/asset.rs`
- `crates/app-core/src/archive.rs`
- `crates/app-core/src/thumbnail.rs`
- `crates/shared-model/src/records.rs`
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md:197`
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md:246`

## 一、items contracts 草案

### 建议新增文件

- `packages/contracts/src/commands/items.ts`
- `packages/contracts/src/models/items.ts`

### 建议命令面

| command | 用途 | 当前后端依据 |
|---|---|---|
| `items.list` | 列出媒体项 | `asset_snapshot_for_library(...)` |
| `items.detail` | 读取单个媒体项详情 | `resolve_asset(...)` + 资产记录 |

说明：

- `items.list` 当前最接近 `asset_snapshot_for_library(...)` 的快照能力
- `items.detail` 当前可先围绕 `resolve_asset(...)` 的结果做最小详情，不要求一开始就补齐页面专属聚合字段

### 建议模型

#### `ItemListEntry`

```ts
type ItemListEntry = {
  assetId: string
  sourceKind: "file" | "archive_entry"
  sourceRefId: string
  libraryId: string
  sourceId: string
  archiveId?: string
  entryPath?: string
  mime: string
  thumbnailKey?: string
}
```

说明：

- 当前主体字段可直接映射 `AssetSnapshotItem`
- `thumbnailKey` 当前允许为空，避免强依赖 `thumbnail.ensure` 已先跑过

#### `ItemDetail`

```ts
type ItemDetail = {
  assetId: string
  sourceKind: "file" | "archive_entry"
  mime: string
  libraryId: string
  sourceId: string
  filePath?: string
  archiveId?: string
  archivePath?: string
  entryPath?: string
}
```

说明：

- 文件型详情可直接映射 `ResolvedFileAsset`
- 归档型详情可直接映射 `ResolvedArchiveEntryAsset`

### 建议请求

#### `ItemsListQuery`

```ts
type ItemsListQuery = {
  libraryId: string
  page?: number
  pageSize?: number
}
```

说明：

- `I1` 首轮先只冻结最小分页壳，不冻结复杂筛选项

## 二、archive contracts 草案

### 建议新增文件

- `packages/contracts/src/commands/archive.ts`
- `packages/contracts/src/models/archive.ts`

### 建议命令面

| command | 用途 | 当前后端依据 |
|---|---|---|
| `archive.entries` | 列出归档页/entry | `archive_snapshot(...)` |
| `archive.entry` 或 `archive.entryDetail` | 读取 entry 详情 | `resolve_archive_entry_location(...)` |
| `archive.normalize` | 触发 rar/7z 归一化 | `normalize_archive_source(...)` |
| `archive.normalizeStatus` | 查询归一化任务状态 | `normalize_archive_status(...)` |

说明：

- `I1` 若只做 zip 浏览，`archive.entries` 与 `archive.entryDetail` 优先级最高
- `normalize` / `normalizeStatus` 更像 `I2/I3` 预备能力，可先写 contracts，再决定是否在 `I1` 立即接到 UI

### 建议模型

#### `ArchiveEntrySummary`

```ts
type ArchiveEntrySummary = {
  id: string
  archiveId: string
  entryPath: string
  entryName: string
  pageIndex: number
  mediaKind: string
  width?: number
  height?: number
}
```

直接映射：`ArchiveEntryRecord`

#### `ArchiveEntryDetail`

```ts
type ArchiveEntryDetail = {
  archiveEntryId: string
  archiveId: string
  sourceId: string
  archivePath: string
  entryPath: string
  mediaKind: string
  archiveUrl: string
}
```

说明：

- `archiveUrl` 不是后端已有字段，但可在 repository/adapter 层由 `archive://entry/<id>` 推导
- contracts 层也可以只先固定不含 `archiveUrl` 的纯后端版对象

#### `ArchiveNormalizeResult`

```ts
type ArchiveNormalizeResult = {
  taskId: string
  sourceId: string
  archiveId: string
  normalizedZipPath: string
  extractedFileCount: number
  indexedEntries: number
  archiveStatus: string
}
```

直接映射：`ArchiveNormalizeSummary`

## 三、thumbnail contracts 草案

### 建议新增文件

- `packages/contracts/src/commands/thumbnail.ts`
- `packages/contracts/src/channels/thumbnail-progress.ts`
- `packages/contracts/src/models/thumbnail.ts`

### 建议命令面

| command | 用途 | 当前后端依据 |
|---|---|---|
| `thumbnail.ensure` | 生成或复用缩略图 | `ensure_thumbnail_for_asset(...)` |
| `thumbnail.get` 或仅协议 URL 读取 | 读取已生成缩略图元数据 | `get_thumbnail(...)` |

说明：

- 真正图片字节仍走 `thumb://`
- `thumbnail.get` 是否需要单独暴露，可在 `I1` 根据 repository 实现再决定；首轮更重要的是 `thumbnail.ensure`

### 建议模型

#### `ThumbnailEnsureRequest`

```ts
type ThumbnailEnsureRequest = {
  assetId: string
  profile: "grid-sm" | "grid-md" | "detail-md" | "detail-lg"
}
```

#### `ThumbnailEnsureResult`

```ts
type ThumbnailEnsureResult = {
  assetId: string
  thumbnailKey: string
  profile: string
  width: number
  height: number
  format: string
  diskPath: string
  byteSize: number
  state: string
  cacheHit: boolean
}
```

直接映射：`ThumbnailEnsureSummary`

#### `ThumbnailProgressEvent`

```ts
type ThumbnailProgressEvent = {
  taskId: string
  assetId: string
  state: "queued" | "running" | "completed" | "failed" | "cancelled"
  profile: string
  thumbnailKey?: string
  message?: string
}
```

说明：

- 当前后端尚无统一 thumbnail progress 流
- 因此这里先给出建议 contracts 形状，不宣称现状已经具备

## 四、建议文件职责草案

### `packages/contracts/src/commands/items.ts`

- `itemsListRequestSchema`
- `itemDetailRequestSchema`

### `packages/contracts/src/models/items.ts`

- `itemListEntrySchema`
- `itemDetailSchema`
- `itemsListQuerySchema`

### `packages/contracts/src/commands/archive.ts`

- `archiveEntriesRequestSchema`
- `archiveEntryDetailRequestSchema`
- `archiveNormalizeRequestSchema`
- `archiveNormalizeStatusRequestSchema`

### `packages/contracts/src/models/archive.ts`

- `archiveEntrySummarySchema`
- `archiveEntryDetailSchema`
- `archiveNormalizeResultSchema`

### `packages/contracts/src/commands/thumbnail.ts`

- `thumbnailEnsureRequestSchema`
- 可选：`thumbnailGetRequestSchema`

### `packages/contracts/src/models/thumbnail.ts`

- `thumbnailEnsureResultSchema`
- `thumbnailProfileSchema`

### `packages/contracts/src/channels/thumbnail-progress.ts`

- `thumbnailProgressEventSchema`

## 五、I1 首轮建议不先做的内容

- items 复杂筛选、排序、聚合统计
- archive viewer 的页面级 route state
- thumbnail 批量预热调度策略
- 面向 UI 的 gallery/card 专属 DTO

## 六、建议施工顺序

1. 先补 `models/items.ts` 与 `commands/items.ts`
2. 再补 `models/archive.ts` 与 `commands/archive.ts`
3. 最后补 `models/thumbnail.ts` / `commands/thumbnail.ts` / `channels/thumbnail-progress.ts`

这样安排的原因：

- `items.list` 是 `I1` 页面最早能消费的数据面
- `archive.entries` 紧跟其后，支撑 `I2/I3` 的归档浏览
- `thumbnail.ensure` 更适合在 repository/adapter 已成型后补进来
