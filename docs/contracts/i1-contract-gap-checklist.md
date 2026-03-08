# I1 Contract Gap Checklist

## 文档定位

本文件把 `P6-3` 当前已经识别出来的 `I1` 前置 contracts 缺口整理成可直接施工的待补清单。

约束：

- 本清单只覆盖不依赖旧 UI 最终页面定义的后端边界项
- 页面级 DTO / command / URL 依赖矩阵不在本文件内冻结

## 当前已具备

### 已有 commands contracts

- `packages/contracts/src/commands/playback.ts`
- `packages/contracts/src/commands/subtitle.ts`

### 已有 channels contracts

- `packages/contracts/src/channels/ffmpeg-progress.ts`
- `packages/contracts/src/channels/subtitle-progress.ts`

### 已有 events contracts

- `packages/contracts/src/events/playback-events.ts`
- `packages/contracts/src/events/subtitle-events.ts`

### 已有共享模型

- `packages/contracts/src/models/ids.ts`
- `packages/contracts/src/models/playback.ts`
- `packages/contracts/src/models/subtitle.ts`
- `packages/contracts/src/models/tasks.ts`
- `packages/contracts/src/errors/app-error.ts`

## I1 前必须补齐的 contracts

### A. library

建议新增：

- `packages/contracts/src/commands/library.ts`
- `packages/contracts/src/models/library.ts`
- `packages/contracts/src/events/library-events.ts`

细化草案：

- `docs/contracts/i1-library-scan-contract-draft.md`

最小能力：

- `library.list`
- `library.add`
- `library.remove` 或 `library.delete`
- `library.get`

最小模型：

- `LibrarySummary`
- `LibraryDetail`（如暂时无额外字段，可先与 summary 共用）
- `AddLibraryInput`

### B. scan

建议新增：

- `packages/contracts/src/commands/scan.ts`
- `packages/contracts/src/channels/scan-progress.ts`
- `packages/contracts/src/events/scan-events.ts`

细化草案：

- `docs/contracts/i1-library-scan-contract-draft.md`

可直接复用：

- `packages/contracts/src/models/tasks.ts`

最小能力：

- `scan.start`
- `scan.resume`
- `scan.status`
- `scan.stats`

最小事件/进度：

- `scan-progress`
- `scan.finished`

### C. items

建议新增：

- `packages/contracts/src/commands/items.ts`
- `packages/contracts/src/models/items.ts`

细化草案：

- `docs/contracts/i1-items-archive-thumbnail-contract-draft.md`

最小能力：

- `items.list`
- `items.detail`

最小模型：

- `ItemListQuery`
- `ItemListEntry`
- `ItemDetail`
- `PageResponse<ItemListEntry>`（如当前 contracts 不单独落泛型，可先写具体分页对象）

### D. archive

建议新增：

- `packages/contracts/src/commands/archive.ts`
- `packages/contracts/src/models/archive.ts`

细化草案：

- `docs/contracts/i1-items-archive-thumbnail-contract-draft.md`

最小能力：

- `archive.entries`
- `archive.entryDetail` 或 `archive.entry`

最小模型：

- `ArchiveEntrySummary`
- `ArchiveEntryDetail`
- `ArchiveEntryPage`

### E. thumbnail

建议新增：

- `packages/contracts/src/commands/thumbnail.ts`
- `packages/contracts/src/channels/thumbnail-progress.ts`
- `packages/contracts/src/models/thumbnail.ts`

细化草案：

- `docs/contracts/i1-items-archive-thumbnail-contract-draft.md`

最小能力：

- `thumbnail.ensure`

最小模型：

- `ThumbnailEnsureRequest`
- `ThumbnailEnsureResult`
- `ThumbnailProgressEvent`

## 宿主接线对应关系

contracts 只是第一步，后续需要同步落到 `src-tauri`：

| contracts 域 | 目标 Tauri 输入面 | 当前状态 |
|---|---|---|
| `library.*` | command | 待补 |
| `scan.*` | command + channel + event | 待补 |
| `items.*` | command | 待补 |
| `archive.*` | command | 待补 |
| `thumbnail.ensure.*` | command + channel | 待补 |

## 建议施工顺序

1. `library`
2. `scan`
3. `items`
4. `archive`
5. `thumbnail`

原因：

- `I1` 最先需要先选库、启动扫描、看到列表数据
- `archive` 与 `thumbnail.ensure` 更适合在 repository 骨架成型后再接

## 完成定义

当本清单对应的首批文件全部存在，且：

- `packages/contracts/src/index.ts` 已导出这些模块
- 至少有 fixture / schema test 覆盖新增 contracts
- `docs/contracts/transport-boundary.md` 与 `docs/contracts/media-repository-surface.md` 不再把这些域标成“空洞”

即可认为 `P6-3` 在 contracts 缺口层面完成首轮收口。
