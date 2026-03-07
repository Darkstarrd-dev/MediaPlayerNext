# 2026-03-07 subtitle sidecar 宿主协议验证记录（B8）

本记录用于确认 `B8` 阶段 subtitle sidecar 宿主协议已经具备：

- `stdio + JSON` sidecar 最小服务
- `ping / health / start_session / stop_session / get_progress / shutdown` 首版协议
- `app-core` 侧 `SubtitleHostPort` 与 `subtitle.*` use case
- `src-tauri` 侧最小 host wrapper 与 retry / restart 语义
- `backend_harness` 的 `subtitle ping/health/start-session/stop-session/get-progress`

本轮验证基于：

- `apps/subtitle-sidecar/scripts/check.ts`
- `src-tauri/src/subtitle_sidecar.rs` 单元测试
- `crates/app-core/src/subtitle_host.rs` 单元测试
- `packages/contracts` subtitle fixtures

## 基本信息

- 日期：`2026-03-07`
- 阶段：`B8`
- 验证入口：`subtitle ping/health/start-session/stop-session/get-progress`
- 验证目标：确认宿主边界、协议 framing 与 restart 语义已冻结，后续字幕 UI 可直接围绕 stable contracts 接入

## 验证场景

### sidecar 协议 smoke

- 启动入口：`apps/subtitle-sidecar/dist/src/index.js`
- 结果：
  - 可完成 `ping`
  - 可完成 `health`
  - 可创建最小 subtitle session
  - 可查询最小 progress
  - 可完成 `shutdown`

### 宿主 retry / restart

- 输入场景：sidecar 首次启动故意异常退出
- 结果：
  - `src-tauri` host wrapper 会重试一次
  - 成功后 `restartCount=1`
  - 首次失败信息会写入 `lastError`

### 会话占位语义

- 当前已固定：
  - `start_session` 会生成稳定 session JSON 记录
  - `stop_session` 会把 state 标为 `stopped`
  - `get_progress` 会返回最小进度快照
- 当前未实现真实字幕识别 / 对齐 / 导出

## 当前结果判断

- 结果符合预期：`B8` 首版已经完成协议冻结、sidecar 最小服务、Rust host wrapper 与 restart 语义
- 当前阶段已经把字幕 sidecar 的 transport、会话边界与错误观测固定下来，后续 UI 不需要再边接宿主边定协议

## 当前缺口

- 还没有真实字幕识别 / 对齐 / 导出能力
- 还没有高级 transport（named pipe / websocket）
- 还没有长期驻留型 sidecar 管理器；当前以最小宿主请求模型为主

## 下一步动作

- 进入后端先行阶段收口后的回归与文档整理
- 后续如进入 UI 接入阶段，可直接开始 `subtitle.*` repository / adapter 接线
