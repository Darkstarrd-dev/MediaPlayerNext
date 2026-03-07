# playback-fixture 说明

本目录用于承载 `B7` 阶段播放后端、媒体协议与最小会话编排的 fixture、golden 与协议约定说明。

当前阶段约定：

- 仓库内优先保留轻量、稳定、可重复的文本基线与 golden 描述
- 不提交大体积真实视频样本；首版验证以：
  - `ffprobe` 输出解析 fixture
  - `ffmpeg` 进度解析 fixture
  - `mpv` 启动参数与会话状态 fixture
  - `media://` / `archive://` 协议 smoke
  为主

## 当前固定内容

- `media-url.expected.json`
  - 固定 `media://asset/<asset_id>` 协议格式
- `archive-url.expected.json`
  - 固定 `archive://entry/<archive_entry_id>` 协议格式
- `playback-session-state.expected.json`
  - 固定首版会话状态语义：`opening / paused / playing / stopped / failed`

## 当前验证方式

- `crates/media-playback` 单元测试负责验证：
  - `ffprobe` 参数与 JSON 解析
  - `ffmpeg` 抽帧参数与 progress 解析
  - `mpv` 启动参数
  - 播放会话磁盘持久化
- `crates/app-core` 单元测试负责验证：
  - 文件型 asset probe 后回填 `media_assets`
  - 打开播放会话后可读取状态
- `src-tauri` 单元测试负责验证：
  - `media://asset/<asset_id>`
  - `archive://entry/<archive_entry_id>`
  - 与既有 `thumb://cache/<thumbnail_key>` 共存

## 后续计划

- 后续如补真实视频样本，可再增加：
  - 首帧抽帧 benchmark
  - 更真实的 `ffprobe` 容器/编码覆盖
  - `mpv` IPC 行为验证
