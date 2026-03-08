# contracts 文档目录说明

本目录用于承载 `P6-3` 的接口收口文档。

当前阶段约束：

- 旧仓 Electron app 的 UI 定义尚未完全收口
- 因此本目录当前先维护“后端边界版”文档
- 页面级 DTO / URL / command 矩阵等补充件，等待旧 UI 定义收口后再补

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

后补文件：

- `docs/contracts/ui-dependency-matrix.md`
  - 等旧 UI 定义收口后，再补页面依赖矩阵
