# window.* -> Tauri 映射表（后端边界版）

## 文档定位

本文件是 `P6-3` 的后端边界版映射表。

当前约束：

- 旧仓 Electron app 的 UI 定义尚未最终收口
- 因此本文件当前不追求“逐页面、逐按钮、逐调用点”的精确迁移表
- 当前先收口旧仓 `window.*` 调用大类在新仓里应该落到 `command / channel / protocol / sidecar` 的哪一层
- 等旧 UI 定义收口后，再补页面级精确映射

旧仓桥接证据入口：

- `docs/migration-plan.md:29`
- `docs/migration-plan.md:31`
- `docs/migration-plan.md:32`
- `docs/migration-plan.md:33`

## 当前新仓已注册输入面

来自 `src-tauri/src/lib.rs`：

- commands
  - `greet`
  - `runtime_smoke_check`
  - `subtitle_ping_command`
  - `subtitle_health_command`
  - `subtitle_start_session_command`
  - `subtitle_stop_session_command`
  - `subtitle_get_progress_command`
- protocols
  - `thumb://cache/<thumbnail_key>`
  - `media://asset/<asset_id>`
  - `archive://entry/<archive_entry_id>`

来自 `packages/contracts/src/index.ts`：

- commands contracts
  - `playback.*`
  - `subtitle.*`
- channels contracts
  - `ffmpeg-progress`
  - `subtitle-progress`

## 映射原则

- 请求-响应、小 payload、需要稳定返回对象的能力 -> `command`
- 高频进度、流式状态 -> `channel`
- 图片/媒体/归档字节 -> `protocol`
- 仍依赖 Node 或外部模型执行的字幕链路 -> `sidecar + command`

## 首版映射表

| 旧仓调用大类 | 新仓目标层 | 当前新仓入口 | 当前状态 | 备注 |
|---|---|---|---|---|
| `window.mediaPlayerBackend.subtitle.*` | `command + sidecar` | `subtitle_*_command` | 已有最小首版 | 已完成 `AppError` 首轮收口 |
| `window.mediaPlayerBackend.runtime.*` / 开发校验类调用 | `command` | `runtime_smoke_check` | 已有最小首版 | 主要用于开发/验证，不是最终 UI 主链路 |
| `window.mediaPlayerBackend.thumbnail bytes` | `protocol` | `thumb://cache/<thumbnail_key>` | 已有首版 | UI 不应走 command 拉字节 |
| `window.mediaPlayerBackend.media bytes` | `protocol` | `media://asset/<asset_id>` | 已有首版 | 供 `<video>` / 资源读取 |
| `window.mediaPlayerBackend.archive entry bytes` | `protocol` | `archive://entry/<archive_entry_id>` | 已有首版 | 供 zip 浏览 / 图片查看 |
| `window.mediaPlayerBackend.playback.*` | `command + channel + event` | 仅 contracts 已有 `playback.*` | 待接线 | `src-tauri` 尚未注册 playback command |
| `window.mediaPlayerBackend.library.*` | `command + channel + event` | 暂无 | 待补 | `I1` 前需补 contracts 与 command 面 |
| `window.mediaPlayerBackend.scan.*` | `command + channel + event` | 暂无 | 待补 | 扫描进度更适合 channel |
| `window.mediaPlayerBackend.items.*` | `command` | 暂无 | 待补 | 列表查询/详情 DTO 面仍待定义 |
| `window.mediaPlayerBackend.archive list/meta.*` | `command` | 暂无 | 待补 | 当前只有 `archive://` 字节协议 |
| `window.mediaPlayerBackend.thumbnail.ensure.*` | `command + channel` | 暂无 | 待补 | URL 已有，但 ensure/progress 还没暴露给前端 |
| `window.mediaPlayerWindow.*` 窗口级 API | 待定 | 暂无 | 后补 | 依赖旧 UI/窗口行为收口后再判断是否需要保留 |

## 按成熟度分层

### A. 已可作为 `I1` 输入的能力

- `subtitle.*`
- `runtime_smoke_check`
- `thumb://`
- `media://`
- `archive://`

### B. contracts 已先行，但宿主接线未完成的能力

- `playback.*`
- `ffmpeg-progress`
- `subtitle-progress`

### C. `I1` 前必须补齐的能力空洞

- `library.*`
- `scan.*`
- `items.*`
- `archive meta/list.*`
- `thumbnail.ensure.*`

### D. 等旧 UI 定义收口后再精确映射的能力

- `window.mediaPlayerWindow.*`
- 页面级旧 `window.*` 调用点
- 页面级“调用来源 -> repository 方法 -> transport”矩阵

## 当前对 `apps/desktop` 的直接影响

- `apps/desktop/src/App.tsx` 仍是最小 `greet` demo，因此当前映射表只能作为 repository / adapter 的前置边界文档，不能直接推出页面结构。
- `I1` 开始时，前端第一步应先把 demo 页升级成 repository 驱动的 app shell，而不是继续堆新的散装 `invoke(...)`。

## 当前明确不做的映射

- 不把旧 Electron `preload/channel/handler` 名字机械平移成 Tauri command 名
- 不在旧 UI 未收口前，写精确到页面/组件的旧 `window.*` 调用点清单
- 不为尚未暴露到 `src-tauri` 的能力伪造新命令名

## 后补项

当前已通过 `docs/contracts/ui-dependency-matrix.md` 补到页面级交互依赖矩阵。

以下内容仍等待旧仓 theme/CSS/内部调用链收口后，再作为本文件补充件追加：

- 逐组件旧 `window.*` 调用点列表
- 逐调用点的新仓目标入口
- 已删除/不再迁移的旧桥接面标记
