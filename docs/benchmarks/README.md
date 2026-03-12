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
- `p6-performance-baseline-20260307.md`
- `p6-quality-gates-20260307.md`
- `p6-duplicate-deps-baseline-20260308.md`
- `p6-debt-delta-baseline-20260312.md`
- `p6-bad-path-validation-20260308.md`
- `benchmark-threshold-policy.md`
- `docs/benchmarks/business-benchmark-latest.json`（业务路径 benchmark 当前值）
- `docs/benchmarks/business-benchmark-history.json`（业务路径 benchmark 历史样本）

说明：

- 当前多数记录仍以 validation / regression 为主，优先固定链路闭环、协议边界与回归口径
- `P6-0` 已补首轮真实性能基线，后续可继续扩成更大样本与多机对照
- `P6-1` 已补统一质量门禁入口，后续可继续围绕 duplicate deps 做 P2 治理收敛
- `P2` 已补 Rust debt-delta 基线，后续可继续围绕 `expect(` 与少量 `panic! / #[allow(...)]` 做定点收敛
- duplicate deps 当前已改为 baseline-delta 口径，重点防止新增多版本分叉，而不是伪装成一次性清零
- 已新增 `check:benchmark-thresholds`，用于对业务路径耗时做 compare + threshold 校验
- 已新增 `bench:collect-business` 与 `check:benchmark-pipeline`，用于自动采样并立即执行阈值门禁
- 已新增 `report:benchmark-trends`，可按 machine 维度输出多机趋势对照产物

后续最低基线项：

- 首次扫描耗时
- 重扫耗时
- 关键查询耗时
- zip 目录读取耗时
- zip 连续 entry 读取耗时
- `ffprobe` 元数据读取耗时
- `mpv` 会话启动耗时
- sidecar `ping / health / restart` 耗时
