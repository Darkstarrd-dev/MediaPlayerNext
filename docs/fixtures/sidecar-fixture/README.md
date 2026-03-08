# sidecar-fixture 说明

本目录用于承载 `B8` 阶段 subtitle sidecar 宿主协议的 fixture、transcript 与错误语义说明。

当前阶段固定内容：

- transport：`stdio`
- framing：newline-delimited JSON（每行一个 JSON message）
- sidecar 入口：`apps/subtitle-sidecar/dist/src/index.js`
- 会话缓存：`data/cache/subtitle/sessions/`

## 当前固定消息

- `ping`
- `health`
- `start_session`
- `stop_session`
- `get_progress`
- `export_srt`
- `shutdown`

说明：

- `export_srt` 在 `B8` 只固定错误语义，不迁移真实导出能力
- `start_session` / `stop_session` / `get_progress` 当前只提供最小会话占位语义，用于冻结宿主协议与会话边界

## 当前样本

- `ping.request.json`
  - 固定最小 ping 请求 framing
- `ping.response.json`
  - 固定最小 ping 返回体
- `health.response.json`
  - 固定 health 返回字段
- `export-srt.response.json`
  - 固定 `export_srt` 在 `B8` 阶段的占位错误语义
- `sidecar-transcript.sample.ndjson`
  - 固定一轮 `ping -> health -> shutdown` 对话样本
- `timeout.transcript.md`
  - 固定 sidecar 无响应时的宿主超时语义
- `malformed-payload.response.txt`
  - 固定 sidecar 返回非法 JSON 时的坏路径样本
- `missing-payload.response.json`
  - 固定 sidecar `ok=true` 但缺失 payload 时的坏路径样本

## 当前验证方式

- `apps/subtitle-sidecar/scripts/check.ts`
  - 启动真实 sidecar 进程，验证 `ping / health / start_session / get_progress / export_srt / stop_session / shutdown`
- `src-tauri/src/subtitle_sidecar.rs`
  - 验证宿主 request/response 解析、restart retry、timeout、bad payload 与错误透传
- `crates/app-core/src/subtitle_host.rs`
  - 验证 `subtitle.*` use case 对宿主 port 的编排边界
