# MediaPlayerNext 实施计划

## 1. 文档目的

本文档是 `C:\opencode\MediaPlayerNext` 的当前主实施计划，用于统一以下事项：

- 新仓整体目标架构
- 初始化阶段边界
- 已确认的技术路线
- 分阶段迁移顺序
- 当前已经完成的初始化项
- 暂缓项与后续里程碑

当前文档覆盖的是“新仓初始化完成，等待旧项目 theme 系统收束后再进入真实迁移”的阶段。

## 2. 推荐路线

推荐路线是：`新仓库 + monorepo 结构`，前端继续 `Vite + React + three.js/WebGL + 现有 theme 体系`，桌面宿主改为 `Tauri 2 + Rust`，字幕保留为 `Node.js sidecar`。

## 3. 目标架构

- `前端`
  - 保留当前 UI 架构、theme token、three.js/WebGL 能力
  - 重点复用旧仓 `C:\opencode\MediaPlayer\src\features\`、theme 和可视化逻辑
  - 证据入口：
    - `C:\opencode\MediaPlayer\src\features\app\useAppDataPipeline.ts:1`
    - `C:\opencode\MediaPlayer\src\features\music-visualizer\webglRenderer.ts:1`
- `桌面宿主`
  - 使用 `Tauri` 替代 `Electron preload + ipcMain/ipcRenderer`
  - 将旧仓分散在以下位置的桥接层收口为 `command + event`：
    - `C:\opencode\MediaPlayer\electron\preload.ts:191`
    - `C:\opencode\MediaPlayer\electron\channels.ts:1`
    - `C:\opencode\MediaPlayer\electron\registerBackendIpcHandlers.ts:271`
- `Rust 后端`
  - 优先承接重 I/O 与本地能力：
    - 库扫描/入库
    - 压缩包读取
    - 媒体资源解析
    - 缩略图/预加载
    - SQLite
    - `mpv/ffmpeg` 控制
- `Node sidecar`
  - 只保留字幕链路
  - 继续承接：
    - `sherpa-onnx-node`
    - VAD
    - 说话人分离
    - SRT 持久化
  - 参考旧仓：
    - `C:\opencode\MediaPlayer\electron\subtitles\asrWorker.ts:134`
    - `C:\opencode\MediaPlayer\electron\subtitles\subtitleSession.ts:220`

## 4. 仓库形态

- `推荐`：新建独立仓库，不回写当前 Electron 主仓
- 原因：
  - 当前主仓仍处于 UI/theme 收束阶段
  - 新仓更适合 PoC、迁移、试错
  - 不污染现有构建、CI、文档和发布节奏
- 当前实际仓库路径：`C:\opencode\MediaPlayerNext`

## 5. 初始化目录计划

目标结构：

```text
MediaPlayerNext/
  apps/
    desktop/                    # Vite + React 前端
    subtitle-sidecar/           # Node.js 字幕 sidecar
  src-tauri/                    # Tauri 宿主入口
  crates/
    app-core/                   # 应用编排、命令注册、事件派发
    media-io/                   # 文件系统、归档读取、资源解析、预加载
    media-db/                   # SQLite 访问层
    media-thumb/                # 缩略图/全屏缓存
    media-playback/             # mpv/ffmpeg 控制
    shared-model/               # Rust DTO / 内部共享模型
  packages/
    contracts/                  # TS 合同、Zod schema、前端 DTO
    ui-shared/                  # 可选，后续抽公共前端工具
  docs/
    logs/
  config/
  scripts/
  .github/
```

### 为什么这样拆

- `apps/desktop`
  - 让前端继续按现有 React/Vite 心智开发
  - 不被 Tauri 目录结构绑死
- `src-tauri`
  - 只做宿主和桥接，不塞业务实现
- `crates/*`
  - 方便后续按能力拆分
  - 避免把所有 Rust 逻辑堆在一个 crate 中
- `apps/subtitle-sidecar`
  - 独立维护 Node 版本、模型依赖和打包策略

## 6. 初始化依赖计划

### 根目录

- Node `22.x`
- Rust stable + `rustup`
- Tauri CLI
- Windows `WebView2`

### `apps/desktop`

- `react`
- `react-dom`
- `vite`
- `typescript`
- `zod`
- `zustand`
- `three`（按需，未来接入现有 3D/可视化扩展时使用）
- `@tauri-apps/api`

### `src-tauri` / Rust

- `tauri`
- `tokio`
- `serde`
- `serde_json`
- `anyhow`
- `thiserror`
- `tracing`
- `tracing-subscriber`
- `rusqlite`
- `zip`（未来 zip 主链路 Rust 化时接入）
- `reqwest`（未来外部 AI 接口沿用）

### `apps/subtitle-sidecar`

- `typescript`
- `zod`
- `sherpa-onnx-node`
- 必要的 worker/process 管理依赖

## 7. 当前已经确认的技术决策

### 7.1 前端路线

- 保留 Web 前端，不切换到 Rust 原生 UI
- 保留 `Vite + React + three.js/WebGL`
- 原因：
  - 现有 UI 架构和 theme 系统可复用
  - 保留前端 tree shaking、HMR 和组件开发效率
  - shader/GLSL/Shadertoy 兼容性继续放在 Web 路线中处理更稳

### 7.2 宿主路线

- 使用 `Tauri 2 + Rust`
- 不再使用 Electron 的 preload/channel/handler 体系
- 仍然保留前后端桥接，但换成更轻的 `command + event`

### 7.3 字幕路线

- 字幕继续保留 `Node.js sidecar`
- 原因：
  - 当前字幕链路复杂，包含 `sherpa-onnx-node`、VAD、说话人处理、持久化
  - 风险高，不适合作为首批 Rust 迁移目标

### 7.4 播放器路线

- 基础模式：后续改走 WebView 原生媒体能力
- 高级模式：继续沿用 `mpv + ffmpeg`，只迁控制层到 Rust

### 7.5 数据库路线

- 继续使用 `SQLite`
- 未来迁的是访问层与重 I/O 串联方式，不是急于更换数据库类型

### 7.6 缩略图路线

- 初始化阶段仍使用 Node.js 侧 `sharp` 做依赖验证
- 当前仓仅验证 `sharp` 可用，不代表最终缩略图方案已经落定
- 长期方向：
  - Rust 负责接管缩略图整条流水线的调度、缓存、I/O
  - 是否完全替换 `sharp`，在后续实际迁移时再依据收益决定

### 7.7 压缩包路线

这是本轮讨论后新增确认的关键决策。

- 高频主链路围绕 `zip` 设计
- `rar/7z` 只在入库时做归一化，不进入后续高频查看主链路
- 日常查看、预加载、审核、删除、保存都以内部 `zip` 为核心格式

#### `zip` 路线

- 未来采用 `Rust 原生 zip` 直读 + 原生写回
- 重点覆盖：
  - 批量列目录
  - 连续查看
  - 预加载
  - 审核/编辑/删除后保存

#### `rar/7z` 路线

- 推荐仅在入库阶段归一化成内部 `zip`
- 优先继续沿用成熟外部工具而不是首批自研纯 Rust 解析
- 在当前项目语境下，推荐继续使用与旧项目一致的 `7z.exe` 归一化路线
- 原因：
  - 兼容性更稳
  - 低频路径不值得为“纯 Rust”投入过高成本
  - Rust 更应把精力放在高频 `zip` 主链路和重 I/O 流水线上

## 8. 迁移原则

- `先适配壳，再迁重后端`
- `先保 UI 完整性，再替换系统能力`
- `字幕独立，不阻塞主迁移`
- `第一阶段不追求命令面全量迁移，只迁最小可运行链路`
- `先等旧仓 theme 系统收束，再开始真实业务迁移`

## 9. 分阶段迁移计划

### 阶段 0：冻结边界

- 完成旧仓前端 theme/token/交互出口收口
- 清点所有直接 `window.mediaPlayerBackend/window.mediaPlayerWindow` 调用
- 准备把这些调用收口到 repository/adapter

### 阶段 1：新仓 bootstrap

- 建新仓
- 初始化 monorepo
- 跑通 `apps/desktop + src-tauri + apps/subtitle-sidecar`
- 实现最小桌面壳：窗口、启动页、日志、版本号、基础命令

### 阶段 2：前端壳迁移

- 把前端接到 Tauri
- 保持界面和 theme 不变
- 先替换基础宿主能力：
  - 窗口
  - dialog
  - clipboard
  - shell
  - app paths

### 阶段 3：桥接层改造

- 保留现有 repository 抽象，替换实现层
- 把 Electron 专属入口替成 Tauri adapter
- 不重写 UI 组件

### 阶段 4：Rust 核心能力迁移

- 迁移顺序：
  - `SQLite`
  - 扫描/入库
  - `zip` 主链路读取与写回
  - `rar/7z` 入库归一化
  - 缩略图/预加载
  - 媒体资源 resolve
- 当前旧仓参考：
  - `C:\opencode\MediaPlayer\electron\mediaLibraryDatabase.ts:15`
  - `C:\opencode\MediaPlayer\electron\services\file-system-read\librarySnapshotService.ts:104`
  - `C:\opencode\MediaPlayer\electron\fileSystemMediaReaders.ts:19`
  - `C:\opencode\MediaPlayer\electron\fileSystemThumbnailResolver.ts:1`
  - `C:\opencode\MediaPlayer\electron\zipArchiveHelpers.ts:60`

### 阶段 5：播放器与外部能力

- 基础模式改走 WebView 原生媒体能力
- 高级模式继续沿用 `mpv + ffmpeg`，只把控制层迁 Rust
- 外部 AI 接口直接迁 Rust HTTP 客户端

### 阶段 6：字幕 sidecar 接入

- Rust 宿主通过 `stdio / named pipe / local socket` 与 `apps/subtitle-sidecar` 通信
- 保留现有会话模型
- 首期不要求字幕重写为 Rust

### 阶段 7：打包与性能回归

- Windows 首发
- 验证：
  - 包体
  - 启动速度
  - 空闲内存
  - 入库速度
  - 翻页延迟
  - 缩略图命中率

## 10. 首期最小可运行范围

- 打开目录
- 打开 `zip` 图片集
- 文件列表/翻页
- 缩略图缓存
- 基础图片查看
- 调用 Rust 读取媒体资源
- 字幕 sidecar 仅完成启动/心跳/ping，不要求首期完整 UI 接入

## 11. 桥接策略

- `前端保持 repository 模式`
- `Tauri command` 替代 Electron query/command IPC
- `Tauri event` 替代库变化、任务进度、缩略图进度这类推送
- `subtitle-sidecar` 不直接暴露给前端，只通过 Rust 宿主管理

## 12. 合同策略

- 首期推荐：`packages/contracts` 继续维护 TS/Zod 合同
- Rust 侧只手写最小 DTO
- 原因：
  - 迁移初期命令面会频繁变动
  - 先不要引入复杂 codegen
- 等命令面稳定后，再评估 `JSON Schema/自定义代码生成`

## 13. 仓库初始化顺序

1. 新建仓库与基础 README、文档目录
2. 初始化 npm workspace
3. 初始化 `apps/desktop`
4. 初始化 `src-tauri`
5. 初始化 `apps/subtitle-sidecar`
6. 预留 `crates/app-core`、`crates/media-io`、`crates/media-db` 等目录
7. 跑通最小 `desktop <-> tauri command`
8. 接入前端骨架页面
9. 接入单个文件打开与图片显示
10. 再开始复制/迁移现有前端模块

## 14. 不建议首批就做的事

- 不要一开始迁 shader 可视化到 Rust
- 不要一开始重写字幕
- 不要一开始追求跨平台完全一致
- 不要一开始做全量 contracts codegen
- 不要一开始把所有 Electron channel 全搬过去
- 不要为了低频 `rar/7z` 路径一开始强追纯 Rust 解析实现

## 15. 里程碑定义

- `M1`：新仓可启动，React 页面在 Tauri 中运行
- `M2`：图片目录浏览可用
- `M3`：`zip` 查看 + 预加载 + 缩略图可用
- `M4`：SQLite 媒体库与导入可用
- `M5`：`mpv/ffmpeg` 高级播放接回
- `M6`：字幕 sidecar 接入完整可用
- `M7`：性能和包体达到替代门槛

## 16. 当前完成状态检查

以下状态基于当前仓库实际内容核对。

### 16.1 仓库与目录

- [x] 新仓已建立：`C:\opencode\MediaPlayerNext`
- [x] Git 已初始化
- [x] 已建立目录：
  - `C:\opencode\MediaPlayerNext\apps\desktop`
  - `C:\opencode\MediaPlayerNext\apps\subtitle-sidecar`
  - `C:\opencode\MediaPlayerNext\crates`
  - `C:\opencode\MediaPlayerNext\packages\contracts`
  - `C:\opencode\MediaPlayerNext\src-tauri`
  - `C:\opencode\MediaPlayerNext\docs`
  - `C:\opencode\MediaPlayerNext\docs\logs`
  - `C:\opencode\MediaPlayerNext\config`
  - `C:\opencode\MediaPlayerNext\scripts`
- [x] 已建立目录：
  - `C:\opencode\MediaPlayerNext\crates\app-core`
  - `C:\opencode\MediaPlayerNext\crates\media-io`
  - `C:\opencode\MediaPlayerNext\crates\media-db`
  - `C:\opencode\MediaPlayerNext\crates\media-thumb`
  - `C:\opencode\MediaPlayerNext\crates\media-playback`
  - `C:\opencode\MediaPlayerNext\crates\shared-model`
- [x] `.github/` 目录已创建

### 16.2 根工作区

- [x] npm workspace 已建立，见 `C:\opencode\MediaPlayerNext\package.json:1`
- [x] 已配置脚本：
  - `dev:web`
  - `build:web`
  - `tauri:dev`
  - `tauri:build`
  - `check`
- [x] 已安装 `@tauri-apps/cli`

### 16.3 前端 `apps/desktop`

- [x] 已初始化 `Vite + React + TypeScript`
- [x] 已接入最小 `desktop <-> tauri command` 示例
- [x] 当前页面可直接调用 Rust `greet` command 并显示返回结果
- [x] 已安装 `@tauri-apps/api`
- [ ] `zod` 尚未接入
- [ ] `zustand` 尚未接入
- [ ] `three` 尚未接入
- [ ] 未迁入任何旧仓 UI/theme/feature 代码

### 16.4 `src-tauri`

- [x] 已初始化 `Tauri 2 + Rust`
- [x] 已建立主程序与 runtime smoke check 二进制
- [x] 已接入 `rusqlite`
- [x] 已接入 `tauri`
- [x] 已接入 `serde` / `serde_json` / `anyhow`
- [x] 已配置 `default-run = "mediaplayernext"`
- [x] 当前仅保留 `crate-type = ["rlib"]`
- [ ] `tokio` 尚未接入
- [ ] `thiserror` 尚未接入
- [ ] `tracing` / `tracing-subscriber` 尚未接入
- [ ] `zip` crate 尚未接入
- [ ] `reqwest` 尚未接入

### 16.5 `apps/subtitle-sidecar`

- [x] 已建立 Node.js sidecar 基础骨架
- [x] 已安装 `sharp`
- [x] 已安装 `zod`
- [x] 已安装 `typescript` / `tsx`
- [x] 已提供 `sharp` smoke check
- [ ] `sherpa-onnx-node` 尚未接入
- [ ] 尚未实现最小 ping/heartbeat

### 16.6 本地运行时与绝对路径

- [x] 已建立绝对路径配置：`C:\opencode\MediaPlayerNext\config\local.paths.json`
- [x] 已记录：
  - `ffmpeg`
  - `ffprobe`
  - `mpv`
  - 当前 Electron 旧仓路径

### 16.7 已验证功能

- [x] `npm install` 可完成依赖安装
- [x] `npm run build:web` 可构建带最小 Tauri command 示例的 React 页面
- [x] `npm run tauri:dev` 可启动 Tauri 宿主并加载前端
- [x] `npm run check` 可完成基础 smoke check
- [x] `sharp` 可正常调用
- [x] `rusqlite` 可正常调用
- [x] `ffmpeg` 可通过绝对路径正常调用
- [x] `mpv` 可通过绝对路径正常调用

### 16.8 当前明确未做

- [x] 未迁移任何旧仓业务代码
- [x] 未迁移 theme 系统
- [x] 未迁移 repository/IPC 实现
- [x] 未迁移播放器真实逻辑
- [x] 未迁移字幕真实逻辑
- [x] 未开始 `zip` Rust 主链路实现
- [x] 未开始 `rar/7z` Rust 归一化实现

## 17. 下一步进入真实迁移前的前置条件

以下条件满足后，再开始真实业务迁移：

- 旧仓 theme 系统收束完成
- 旧仓前端直接 `window.*` 调用点梳理完成
- 新仓是否创建 `crates/*` 结构得到确认
- 新仓决定第一批迁移接口范围
- 压缩包路线继续按“`zip` 主链路 Rust 化，`rar/7z` 入库归一化用 `7z.exe`”执行

## 18. 新对话里可直接执行的任务目标

- 在当前新仓基础上继续补足缺失的初始化项
- 创建 `crates/*` 目录
- [已完成] 增加最小 `desktop <-> tauri command` 示例
- 增加字幕 sidecar 最小 ping/heartbeat
- 继续保持“不迁业务，只搭骨架”的边界
