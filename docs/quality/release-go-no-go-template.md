# Release Go/No-Go 模板

本模板用于发布前收口，统一记录“是否允许发布”的结论与证据。

## 1. 基本信息

- 版本号：
- 分支 / commit：
- 评审时间：
- 评审人：

## 2. 必要门禁结果

- `check:quality:heavy`：
  - 产物路径：
- `check:release`：
  - 产物路径：
- `e2e:desktop:doctor`：
  - 结果：
- `e2e:desktop`：
  - 结果：

## 3. 风险审计

- 宿主边界（capabilities / resources / sidecar）：
- 数据迁移与回滚风险：
- 兼容性风险（Windows 版本 / 依赖运行时）：

## 4. 体积与性能

- 可执行产物体积：
- 与基线差值：
- benchmark 关键指标：
- 是否触发阈值告警：

## 5. 结论

- 结论：`Go` / `No-Go`
- 结论依据：
- 若 `No-Go`，阻断项与修复 owner：

## 6. 回滚与应急

- 回滚版本：
- 回滚步骤：
- 监控项与观察窗口：
