# Benchmarks

本目录用于记录后端先行阶段的 benchmark 方案、样本口径与结果基线。

`B1` 先冻结口径，不在本阶段追求完整 benchmark 报表。

后续最低基线项：

- 首次扫描耗时
- 重扫耗时
- 关键查询耗时
- zip 目录读取耗时
- zip 连续 entry 读取耗时
- `ffprobe` 元数据读取耗时
- `mpv` 会话启动耗时
- sidecar `ping / health / restart` 耗时
