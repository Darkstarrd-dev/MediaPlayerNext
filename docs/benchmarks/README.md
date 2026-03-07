# Benchmarks

本目录用于记录后端先行阶段的 benchmark 方案、样本口径与结果基线。

`B1` 先冻结口径，不在本阶段追求完整 benchmark 报表。

当前已形成的验证 / 回归记录：

- `scan-validation-20260307.md`
- `archive-validation-20260307.md`
- `thumbnail-validation-20260307.md`
- `normalize-validation-20260307.md`
- `playback-validation-20260307.md`
- `sidecar-validation-20260307.md`
- `backend-regression-20260307.md`

说明：

- 当前多数记录仍以 validation / regression 为主，优先固定链路闭环、协议边界与回归口径
- 更细的真实性能 benchmark（真实大样本、冷/热命中耗时、工具级毫秒统计）可在后续 `P6` 持续补强

后续最低基线项：

- 首次扫描耗时
- 重扫耗时
- 关键查询耗时
- zip 目录读取耗时
- zip 连续 entry 读取耗时
- `ffprobe` 元数据读取耗时
- `mpv` 会话启动耗时
- sidecar `ping / health / restart` 耗时
