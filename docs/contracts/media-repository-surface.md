# MediaRepository Surface（首版骨架）

## 文档定位

本文件不是最终 UI 页面接口表，而是 `P6-3` 在旧 UI 未完全收口前，先固定的 repository 能力骨架。

当前目标：

- 先让 `I1` 能围绕稳定后端边界建 repository / adapter
- 不在当前阶段假设最终页面信息架构

## 设计原则

- repository 对上暴露稳定能力语义，不暴露 Tauri command 名细节
- repository 对下封装 `command / channel / protocol`
- repository 先按后端能力域拆分，不按旧页面拆分

## 首版能力面

## 建议最小 TypeScript 形状

```ts
export interface MediaRepository {
  diagnostics: {
    checkRuntimeHealth(): Promise<RuntimeSmokeCheckResult>
  }
  subtitle: {
    ping(): Promise<SubtitleHost>
    health(): Promise<SubtitleHost>
    startSession(assetId?: AssetId): Promise<SubtitleSession>
    stopSession(sessionId: SubtitleSessionId): Promise<SubtitleSession>
    getProgress(sessionId: SubtitleSessionId): Promise<SubtitleProgress>
  }
  urls: {
    thumbnail(thumbnailKey: ThumbnailKey): string
    media(assetId: AssetId): string
    archiveEntry(entryId: ArchiveEntryId): string
  }
}
```

说明：

- 这里只固定能力分区，不固定最终文件名或前端目录结构。
- `RuntimeSmokeCheckResult` 当前来自 Rust command；后续若不进入正式 UI 主链路，可只保留在 diagnostics 侧。
- `urls` 统一由 repository/adapter 生成，组件层不直接拼协议字符串。

### A. runtime / diagnostics

| repository 方法 | 底层入口 | 当前状态 | 说明 |
|---|---|---|---|
| `checkRuntimeHealth()` | `runtime_smoke_check` | 已有最小首版 | 偏开发/诊断能力 |

### B. subtitle

| repository 方法 | 底层入口 | 当前状态 | 说明 |
|---|---|---|---|
| `subtitle.ping()` | `subtitle_ping_command` | 已有首版 | 返回 host summary |
| `subtitle.health()` | `subtitle_health_command` | 已有首版 | 返回 health + restart 信息 |
| `subtitle.startSession(assetId?)` | `subtitle_start_session_command` | 已有首版 | 返回 session summary |
| `subtitle.stopSession(sessionId)` | `subtitle_stop_session_command` | 已有首版 | 返回 session summary |
| `subtitle.getProgress(sessionId)` | `subtitle_get_progress_command` | 已有首版 | 当前仍是请求式，不是 channel |

### C. protocol URL builders

| repository 方法 | 底层入口 | 当前状态 | 说明 |
|---|---|---|---|
| `buildThumbnailUrl(thumbnailKey)` | `thumb://cache/<thumbnail_key>` | 已有首版 | UI 图片主入口 |
| `buildMediaUrl(assetId)` | `media://asset/<asset_id>` | 已有首版 | 媒体字节入口 |
| `buildArchiveEntryUrl(entryId)` | `archive://entry/<archive_entry_id>` | 已有首版 | archive 图片入口 |

## 当前可直接复用的 contracts 类型

- `packages/contracts/src/models/ids.ts`
  - `assetIdSchema`
  - `thumbnailKeySchema`
  - `archiveEntryIdSchema`
  - `playbackSessionIdSchema`
  - `subtitleSessionIdSchema`
- `packages/contracts/src/models/subtitle.ts`
  - `SubtitleHost`
  - `SubtitleSession`
  - `SubtitleProgress`
- `packages/contracts/src/models/playback.ts`
  - `MediaProbe`
  - `MediaUrl`
  - `PlaybackSession`

## 待补能力面（I1 优先）

这些方法名当前只作为能力占位，不代表最终命名已冻结：

### library

- `listLibraries()`
- `createLibrary(input)`
- `getLibrary(libraryId)`
- `removeLibrary(libraryId)`

### scan

- `startScan(libraryId)`
- `resumeScan(taskId)`
- `getScanTask(taskId)`
- `subscribeScanProgress(taskId)`

### items

- `listItems(query)`
- `getItemDetail(itemId)`

### archive

- `listArchiveEntries(assetId | archiveId)`
- `getArchiveEntryDetail(entryId)`

### thumbnail

- `ensureThumbnail(assetId, profile)`
- `subscribeThumbnailProgress(assetId | taskId)`

### playback

- `openPlaybackSession(assetId)`
- `getPlaybackSession(sessionId)`
- `seekPlayback(sessionId, positionMs)`
- `subscribePlaybackProgress(sessionId)`

## 当前不在首版骨架内冻结的内容

- 分页参数的最终字段
- 页面级聚合 DTO
- 旧 UI 每个页面具体依赖哪些方法
- repository 在桌面端与未来 Web/mock 端的最终抽象拆分

## 后补项

等待旧仓 UI 定义收口后，再追加：

- 页面 -> repository 方法映射
- 页面需要的 DTO / URL / command 矩阵
- 每个方法的缓存/分页/预取策略

## I1 实施前的最小落地建议

1. 先在 `apps/desktop/src/repositories/media-repository.ts` 固定接口骨架
2. 再在 `apps/desktop/src/adapters/tauri/` 封装 command/protocol 适配
3. 最后把 `apps/desktop/src/App.tsx` demo 替换成最小 app shell + repository provider
