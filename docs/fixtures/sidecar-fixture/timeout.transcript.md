# timeout transcript 说明

本样本用于固定 `B8` 阶段 sidecar 超时语义。

## 输入

```json
{"id":"req-ping-timeout-001","type":"ping"}
```

## 预期宿主结果

- `src-tauri` host wrapper 在请求超时后终止 sidecar 子进程
- Rust 侧返回包含 `timed out` 字样的错误
- 该错误属于宿主可观测错误，不要求 sidecar 返回 JSON 错误体

## 用途

- 固定 heartbeat / timeout 场景的最小错误语义
- 避免后续修改 request timeout 行为时悄悄改变宿主错误边界
