# contracts 文档目录说明

本目录用于承载 `P6-3` 的接口收口文档。

当前阶段约束：

- 旧仓 theme 系统、CSS 层级与页面内部调用链尚未完全收口
- 但页面交互关系已可用于冻结 repository 依赖矩阵
- 因此本目录当前同时维护“后端边界版”文档与“交互已收口版”页面依赖矩阵

当前文件：

- `docs/contracts/window-to-tauri-mapping.md`
  - 旧 `window.*` 大类到新仓 `command/channel/protocol/sidecar` 的映射
- `docs/contracts/transport-boundary.md`
  - `command / channel / protocol / sidecar` 的分工边界
- `docs/contracts/media-repository-surface.md`
  - `MediaRepository` 首版能力骨架
- `docs/contracts/i1-contract-gap-checklist.md`
  - `I1` 前必须补齐的 contracts 缺口与建议顺序
- `docs/contracts/i1-library-scan-contract-draft.md`
  - `library` / `scan` 首批 contracts 的细化草案
- `docs/contracts/i1-items-archive-thumbnail-contract-draft.md`
  - `items` / `archive` / `thumbnail` 首批 contracts 的细化草案
- `docs/contracts/ui-dependency-matrix.md`
  - 基于已确认交互关系的页面 -> repository -> transport 依赖矩阵

后补文件：

- 逐组件调用链迁移清单
  - 等旧仓 theme/CSS/内部调用链收口后，再补更细粒度的实现顺序
