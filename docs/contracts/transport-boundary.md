# Transport Boundary

## 文档定位

本文件用于收口 `I1` 前前后端传输边界，避免前端继续直接依赖 `invoke` 细节，或把媒体字节重新塞回 JSON IPC。

## 边界总则

| 传输层 | 适用内容 | 当前状态 | 说明 |
|---|---|---|---|
| `command` | 请求-响应、低频、结构化 DTO | 已有最小首版 | 当前已注册 `runtime_smoke_check` 与 `subtitle.*` |
| `channel` | 高频进度、长任务状态、流式更新 | contracts 有，宿主接线未齐 | `ffmpeg-progress`、`subtitle-progress` 已有 contracts |
| `event` | 低频广播、全局通知 | contracts 有，宿主接线未齐 | 当前不作为 `I1` 首要入口 |
| `protocol` | 图片/媒体/归档字节 | 已有首版 | `thumb://` / `media://` / `archive://` |
| `sidecar` | 仍需 Node/外部模型执行的字幕链路 | 已有首版 | 前端不直接连 sidecar，由 Rust host 包装 |

## 当前建议接法

### 1. command

适合：

- subtitle host 控制
- runtime/debug 校验
- 未来 library / scan / items / archive meta / thumbnail ensure

当前已注册：

- `greet`
- `runtime_smoke_check`
- `subtitle_ping_command`
- `subtitle_health_command`
- `subtitle_start_session_command`
- `subtitle_stop_session_command`
- `subtitle_get_progress_command`

约束：

- command 返回对象错误统一走 `AppError`
- command 不负责返回大块媒体字节

### 2. channel

适合：

- scan progress
- thumbnail warmup progress
- ffmpeg progress
- subtitle progress

当前状态：

- `packages/contracts/src/channels/ffmpeg-progress.ts`
- `packages/contracts/src/channels/subtitle-progress.ts`
- `src-tauri` 侧尚未形成通用 channel 接线

结论：

- `I1` 前应优先补足 channel wiring，而不是让前端轮询 command

### 3. protocol

适合：

- `<img src={thumbUrl}>`
- `<video src={mediaUrl}>`
- archive entry 图片查看

当前已注册：

- `thumb://cache/<thumbnail_key>`
- `media://asset/<asset_id>`
- `archive://entry/<archive_entry_id>`

约束：

- 协议错误当前已稳定到 `status + error headers + body message`
- 前端不应通过 command 请求等价字节流

### 4. sidecar

适合：

- subtitle ASR / VAD / speaker 等 Node 侧能力

当前约束：

- 前端永远不直接与 sidecar 进程通信
- 前端只消费 Rust 暴露的 `subtitle.*` command 与后续 progress channel

## 当前 `MediaRepository` 约束

- repository 层可以封装 command/channel/protocol
- 组件层不得直接拼 `thumb://` / `media://` / `archive://` 以外的底层桥接细节
- 组件层不得直接写散的 `invoke(...)` 集合
- 任何媒体内容 URL 应由 repository / adapter 提供

## 当前 contracts 盘点

### 已导出的部分

- errors
  - `app-error`
- commands
  - `playback`
  - `subtitle`
- channels
  - `ffmpeg-progress`
  - `subtitle-progress`
- events
  - `playback-events`
  - `subtitle-events`
- models
  - `ids`
  - `playback`
  - `subtitle`
  - `tasks`

### 当前缺口

- `library` commands/models/events
- `scan` commands/channels/events
- `items` commands/models
- `archive` commands/models
- `thumbnail` commands/channels/models

## I1 前的最小边界补齐顺序

1. 先补 `library` / `scan` / `items` / `archive` / `thumbnail` contracts
2. 再补对应 Tauri command/channel wiring
3. 最后让 `apps/desktop` 的 repository/adapter 消费这些稳定边界

## 仍待补齐的边界

- library / scan / items / archive meta / thumbnail ensure 的 command 面
- scan / thumbnail / playback 的 channel wiring
- 低频广播 event 的首版消费策略
