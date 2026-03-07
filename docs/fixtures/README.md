# Fixtures

本目录用于放置后端先行阶段的固定样本与快照说明。

当前阶段约定：

- `small-fixture/`：本地快速调试与单元/集成测试
- `medium-fixture/`：扫描回归与性能回归
- `archive-fixture/`：zip、rar、7z 与损坏包样本

`B1` 仅先落目录约定与说明，真实样本在 `B2-B4` 逐步补齐。

当前还补充了占位数据生成脚本，用于先生成可扫描的样本骨架：

- 命令：`npm run fixtures:scan-placeholders`
- 强制重建：`npm run fixtures:scan-placeholders:force`
- 输出：
  - `docs/fixtures/small-fixture/generated-placeholder/`
  - `docs/fixtures/medium-fixture/generated-placeholder/scan-root/`
  - `data/scan-validation/local-real-dir-mock/`

Git 管理约束：

- `small-fixture` 保留少量、稳定、可回放的仓库内样本
- `medium-fixture/generated-placeholder/` 不纳入 Git，用于本地生成和替换真实文件
- 其中 `realdata/` 作为真实样本池，`scan-root/` 作为实际扫描目录
- `realdata/` 只作为一次性样本输入；生成完成后不保留在最终扫描目录

安全约束：

- 默认命令不会覆盖已有的 fixture 目录内容
- 只有显式使用 `fixtures:scan-placeholders:force` 时，才允许重建生成目录

当前进一步固定为“两层目录”工作流：

- 基线目录：保存已经填充真实文件后的 `small` / `medium` / `local` 样本
- 临时运行目录：每次测试前从基线复制到 `data/scan-validation/runs/`，测试后再删除

一次性真实文件填充：

- `npm run fixtures:fill-real`

创建一轮测试副本：

- `npm run fixtures:create-run -- <run-name>`
- `data/scan-validation/local-real-dir-mock/` 继续只做本地目录，不纳入 Git

后续做真实验证时，可在保持目录结构和类型比例的前提下，用真实文件按类替换这些占位文件。
