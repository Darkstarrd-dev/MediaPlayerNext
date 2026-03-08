# UI 依赖矩阵（交互已收口版）

## 文档定位

本文件是 `P6-3` 在“交互关系已明确、但 theme/CSS/页面内部调用链仍待收口”前提下追加的页面依赖矩阵。

当前边界：

- 可以冻结页面交互需要依赖哪些 `repository / command / channel / protocol / event`
- 不在当前阶段冻结 theme 结构、CSS 层级与视觉实现细节
- 不在当前阶段把旧仓页面组件树机械映射到新仓目录

## 页面 -> repository -> transport

| 页面/模块 | 关键交互 | repository 方法 | transport | 当前状态 | 备注 |
|---|---|---|---|---|---|
| `LibraryPicker` | 列库、创建库、删除库 | `library.list` / `library.add` / `library.remove` | `command` + `event` | 待接线 | 交互已定，theme 未定 |
| `ScanPanel` | 启动扫描、恢复扫描、看统计/状态 | `scan.start` / `scan.resume` / `scan.stats` / `scan.snapshot` | `command` + `channel` | 待接线 | 建议优先补 channel |
| `ItemsGridPage` | 查询列表、确保缩略图、展示缩略图 | `items.list` / `thumbnail.ensure` / `urls.thumbnail` | `command` + `protocol` | 待接线 | `thumb://` 已可用 |
| `ArchiveViewerPage` | 拉 entry 列表、取详情、显示 archive 图片 | `archive.entries` / `archive.entryDetail` / `urls.archiveEntry` | `command` + `protocol` | 待接线 | `archive://` 已可用 |
| `ItemDetailPage` | 普通图片/归档详情与元数据 | `items.detail` / `urls.media` / `urls.archiveEntry` | `command` + `protocol` | 待接线 | 视觉与布局后补 |
| `DiagnosticsPanel` | runtime 校验、subtitle host 健康检查 | `diagnostics.checkRuntimeHealth` / `subtitle.ping` / `subtitle.health` | `command` | 已接线 | 当前 desktop shell 已可调用 |

## 当前对 I1 的直接结论

- 页面依赖矩阵现在已经可以冻结到 repository 层，不必继续等待 theme 收口
- `apps/desktop` 可以先实现 repository / adapter / shell，继续避免组件层直接依赖 `invoke`
- 真实页面视觉、旧主题变量、CSS 层级与组件树迁移仍保持暂停

## 当前不在本文件里冻结的内容

- 每个页面的最终路由 path
- 每个页面的最终状态管理拆分
- 旧仓具体 CSS 作用域与 theme token 迁移方式
- 旧仓内部调用链的逐组件复刻顺序

## 下一步建议

1. 先让 `apps/desktop` 的 `MediaRepository` 与 `tauriMediaRepository` 覆盖上表的能力占位
2. 再让 `AppShell` 展示当前 ready/planned 边界，替换掉最小 `greet` demo
3. 等 theme 收口后，再进入 `I2-I4` 的真实页面实现
