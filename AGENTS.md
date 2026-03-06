# AGENTS.md - MediaPlayerNext 代理开发指南

本文档用于指导 AI 代理在 `C:\opencode\MediaPlayerNext` 仓库内开展工作。

## 1. 项目定位

- 仓库路径固定为 `C:\opencode\MediaPlayerNext`
- 当前阶段仅完成新仓初始化与运行时准备，不进行任何业务迁移
- 目标架构为：
  - 前端：`Vite + React`
  - 桌面宿主：`Tauri 2 + Rust`
  - 字幕链路：未来保留 `Node.js sidecar`
- 当前前端只需保持默认 Vite React 页面可运行
- 实际功能迁移必须等待旧项目 theme 系统收束后再开始

参考文档：

- `C:\opencode\MediaPlayerNext\docs\migration-plan.md`
- `C:\opencode\MediaPlayerNext\README.md`
- 运行时绝对路径配置：`C:\opencode\MediaPlayerNext\config\local.paths.json`

## 2. 当前阶段严格边界

### 允许做的事

- 维护新仓目录结构、构建脚本、开发脚手架
- 维护 Rust/Tauri、React、Node sidecar 的基础依赖
- 增加 smoke check、开发辅助脚本、基础文档
- 为未来迁移预留 contracts、crate、workspace 结构

### 当前禁止做的事

- 不复制 `C:\opencode\MediaPlayer\src` 的业务或 UI 代码
- 不复制 `C:\opencode\MediaPlayer\electron` 的 IPC、服务、主进程逻辑
- 不提前迁移 theme 系统、repository、播放器、缩略图、数据库 schema、字幕真实会话
- 不在当前阶段引入与迁移无关的大型新能力

## 3. 仓库结构

```text
C:\opencode\MediaPlayerNext
  apps\desktop              # React + Vite 前端
  apps\subtitle-sidecar     # Node.js 字幕 sidecar 骨架
  packages\contracts        # 未来共享合同
  src-tauri                  # Tauri 宿主与 Rust 入口
  docs                       # 计划与说明
  config                     # 本地绝对路径配置
  scripts                    # Windows/Rust 辅助脚本
```

## 4. 环境与工具

### 基础环境

- Node.js：`22.x`
- npm：`11.x`
- Rust：通过 `rustup` 安装的 `stable-x86_64-pc-windows-msvc`
- Windows 编译工具：Visual Studio 2022 Build Tools / Community MSVC

### 代理与网络

- 当前可用本机代理端口：`http://127.0.0.1:3066`
- 需要联网安装依赖时，优先显式设置：
  - `HTTP_PROXY=http://127.0.0.1:3066`
  - `HTTPS_PROXY=http://127.0.0.1:3066`
- 不要继续使用旧的 `2080` 端口

### 绝对路径约束

- 调用配置、脚本、运行时资源时，优先使用绝对路径
- 当前本地运行时资源记录于：`C:\opencode\MediaPlayerNext\config\local.paths.json`
- 已知外部资源：
  - `ffmpeg`：`C:\Tools\ffmpeg-2024-06-06-git-d55f5cba7b-full_build\bin\ffmpeg.exe`
  - `ffprobe`：`C:\Tools\ffmpeg-2024-06-06-git-d55f5cba7b-full_build\bin\ffprobe.exe`
  - `mpv`：`C:\opencode\mpv\mpv.exe`

## 5. 常用命令

在仓库根目录 `C:\opencode\MediaPlayerNext` 执行：

- 安装依赖：`npm install`
- 启动前端开发服务器：`npm run dev:web`
- 启动 Tauri 开发环境：`npm run tauri:dev`
- 构建前端：`npm run build:web`
- 运行基础 smoke check：`npm run check`

### 脚本说明

- `C:\opencode\MediaPlayerNext\scripts\run-tauri-dev.cmd`
  - 会先调用 `VsDevCmd.bat` 再执行 `tauri dev`
- `C:\opencode\MediaPlayerNext\scripts\run-tauri-build.cmd`
  - 会先调用 `VsDevCmd.bat` 再执行 `tauri build`
- `C:\opencode\MediaPlayerNext\scripts\run-cargo-with-msvc.cmd`
  - 用于在已注入 MSVC 环境后执行 Cargo
- `C:\opencode\MediaPlayerNext\scripts\check-runtimes.ps1`
  - 用于校验 `rusqlite`、`ffmpeg`、`mpv`

## 6. 当前已验证状态

- `npm run build:web` 可成功构建默认 React 页面
- `npm run tauri:dev` 可启动 Tauri 宿主并加载前端
- `npm run check` 已验证：
  - `sharp`
  - `rusqlite`
  - `ffmpeg`
  - `mpv`

## 7. Rust/Tauri 开发注意事项

- 当前 `C:\opencode\MediaPlayerNext\src-tauri\Cargo.toml` 仅保留 `crate-type = ["rlib"]`
- 未经明确需要，不要恢复 `staticlib` 或 `cdylib`
- 原因：当前阶段只做桌面宿主初始化，恢复额外 crate type 会显著增大 `target` 体积并拖慢编译
- 如果未来出现 FFI、移动端或原生嵌入需求，再按需补回

### 体积与缓存

- Rust 的 `C:\opencode\MediaPlayerNext\src-tauri\target` 目录会很大，属于正常现象
- 如果需要回收空间，可执行：
  - `C:\opencode\MediaPlayerNext\scripts\run-cargo-with-msvc.cmd clean --manifest-path C:\opencode\MediaPlayerNext\src-tauri\Cargo.toml`
- 清理后下次会重新编译，不属于故障

## 8. 代码与文档规范

- 与用户沟通使用中文
- 注释、文档、说明统一使用中文
- 新增代码优先使用 TypeScript / Rust，不要引入 `any`
- 保持当前阶段代码最小化，不为未来迁移过早抽象
- 优先可维护、可验证、可回退的实现

## 9. 工作流程

1. 先阅读：`C:\opencode\MediaPlayerNext\docs\migration-plan.md`
2. 确认是否属于“初始化阶段允许项”
3. 先改最小范围文件
4. 运行相关验证：
   - 前端改动至少执行 `npm run build:web`
   - Rust/运行时改动至少执行 `npm run check`
   - Tauri 宿主相关改动必要时执行 `npm run tauri:dev`
5. 在最终说明中明确：修改了什么、验证了什么、未做什么

## 10. 后续迁移原则

- 等待 `C:\opencode\MediaPlayer` 的 theme 系统收束完成后，再开始实际迁移
- 实际迁移顺序优先考虑：
  - 壳层与桥接层
  - 重 I/O 后端
  - 数据库与媒体资源解析
  - 播放控制
  - 字幕 sidecar 接入
- 当前仓库的首要目标是：保持初始化环境稳定、可复用、可直接继续开发
