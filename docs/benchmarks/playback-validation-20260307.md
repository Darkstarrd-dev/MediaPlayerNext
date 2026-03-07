# 2026-03-07 播放链路验证记录（B7）

本记录用于确认 `B7` 阶段播放后端与媒体协议输入面已经具备：

- `media-playback` 真实 crate
- `ffprobe` 参数与 JSON 解析
- `ffmpeg` 抽帧参数与 progress 解析
- `mpv` 启动参数与最小会话持久化
- `playback probe/open/status/pause/seek/stop` 开发期命令面
- `media://asset/<asset_id>` 与 `archive://entry/<archive_entry_id>` 协议输入面
- runtime smoke 纳入 `ffprobe`

本轮验证基于：

- `crates/media-playback` 单元测试
- `crates/app-core` playback 单元测试
- `src-tauri` 协议 smoke 测试
- `packages/contracts` zod fixtures

## 基本信息

- 日期：`2026-03-07`
- 阶段：`B7`
- 验证入口：`playback probe/open/status`、`media://`、`archive://`
- 验证目标：确认未来 UI 已可围绕 URL 与 session，而不是 JSON 大字节传输，继续推进播放主链路

## 验证场景

### ffprobe 与元数据回填

- 输入类型：`MediaAssetRecord(source_kind=file)`
- 结果：
  - `ffprobe` JSON 可解析为 `MediaProbeSummary`
  - `duration_ms / width / height / codec_info_json` 可回填到 `media_assets`

### ffmpeg 与首帧输入面

- 当前已固定：
  - 首帧抽帧参数构造
  - `-progress` 输出解析
- 当前未接入真实大样本 benchmark，但已具备继续扩写的包装层

### mpv 会话

- 当前已固定：
  - `mpv` 启动参数
  - `open/status/pause/seek/stop` 最小会话状态持久化
- 当前未实现真正的 `mpv` IPC，仅提供首版会话语义

### 协议访问

- `media://asset/<asset_id>`
  - 可解析文件型 asset 并返回媒体字节
- `archive://entry/<archive_entry_id>`
  - 可解析 entry 并从 zip / normalized zip 读取字节

## 当前结果判断

- 结果符合预期：`B7` 首版已经完成“probe / session / protocol”三层闭环
- 当前阶段已经把媒体字节入口从 JSON IPC 中剥离出来，后续 UI 可直接围绕 `thumb://` / `media://` / `archive://` 继续推进

## 当前缺口

- 还没有真实视频样本 benchmark
- 还没有真正的 `mpv` IPC 控制
- 还没有把视频首帧正式接入缩略图 pipeline

## 下一步动作

- 进入 `B8`，开始 sidecar 宿主协议
- 后续如需要，再补真实视频样本 benchmark 与 `mpv` IPC
