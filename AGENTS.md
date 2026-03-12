# AGENTS.md - MediaPlayerNext 代理开发指南

本文档用于指导 AI 代理在 `Z:\Playground\CurrentWorking\MediaPlayerNext` 仓库内开展工作。

## 1. 项目定位

- 仓库路径固定为 `Z:\Playground\CurrentWorking\MediaPlayerNext`
- 当前阶段已完成后端地基与 I1 首轮接线，开始进入“布局先行、样式链重建、面板逐步接回”的初始 UI 架构阶段
- 目标架构为：
  - 前端：`Vite + React`
  - 桌面宿主：`Tauri 2 + Rust`
  - 字幕链路：未来保留 `Node.js sidecar`
- 当前前端目标不再是维持默认 Vite 页面，而是逐步建立新的桌面 UI 壳层与面板布局
- 不再等待旧项目 theme 系统完全收束后才开始 UI 工作，改为分步迁移布局与内容

参考文档：

- `Z:\Playground\CurrentWorking\MediaPlayerNext\docs\README.md`
- `Z:\Playground\CurrentWorking\MediaPlayerNext\docs\ui\ui-definition.md`
- `Z:\Playground\CurrentWorking\MediaPlayerNext\docs\testing\tauri-e2e-strategy.md`
- `Z:\Playground\CurrentWorking\MediaPlayerNext\README.md`
- 运行时绝对路径配置：`Z:\Playground\CurrentWorking\MediaPlayerNext\config\local.paths.json`

## 2. 当前阶段严格边界

### 允许做的事

- 维护新仓目录结构、构建脚本、开发脚手架
- 维护 Rust/Tauri、React、Node sidecar 的基础依赖
- 增加 smoke check、开发辅助脚本、基础文档
- 继续推进 repository / adapter / protocol 边界
- 逐步建立新的 UI 布局骨架、基础样式基座与面板切片

### 当前禁止做的事

- 不复制 `Z:\Playground\CurrentWorking\MediaPlayerX\src` 的业务或 UI 代码
- 不复制 `Z:\Playground\CurrentWorking\MediaPlayerX\electron` 的 IPC、服务、主进程逻辑
- 不整包复制旧仓 theme 系统或旧仓页面代码
- 不把多个面板或整页 UI 一次性硬迁入新仓
- 不在样式基座未稳定前扩散大量页面级视觉细节
- 不在当前阶段引入与迁移无关的大型新能力

## 3. 仓库结构

```text
Z:\Playground\CurrentWorking\MediaPlayerNext
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


### 绝对路径约束

- 调用配置、脚本、运行时资源时，优先使用绝对路径
- 当前本地运行时资源记录于：`Z:\Playground\CurrentWorking\MediaPlayerNext\config\local.paths.json`
- 已知外部资源：
  - `ffmpeg`：`C:\Tools\ffmpeg-7.1.1-essentials_build\bin\ffmpeg.exe`
  - `ffprobe`：`C:\Tools\ffmpeg-7.1.1-essentials_build\bin\ffprobe.exe`
  - `mpv`：`Z:\Playground\CurrentWorking\mpv\mpv.exe`

## 5. 常用命令

在仓库根目录 `Z:\Playground\CurrentWorking\MediaPlayerNext` 执行：

- 安装依赖：`npm install`
- 启动前端开发服务器：`npm run dev:web`
- 启动 Tauri 开发环境：`npm run tauri:dev`
- 构建前端：`npm run build:web`
- 运行基础 smoke check：`npm run check`

### Tauri 自动化 E2E 约定

- 当前仓库的桌面自动化 E2E 基线方案固定为：`tauri-driver + WebdriverIO`
- 方案说明文档：`Z:\Playground\CurrentWorking\MediaPlayerNext\docs\testing\tauri-e2e-strategy.md`
- 当前阶段优先目标平台：`Windows`
- 当前已接入 `Phase A` 脚手架，可使用：
  - `npm run e2e:desktop:doctor`
  - `npm run e2e:desktop`
  - `npm run e2e:desktop:headed`
- 与系统文件夹选择器、系统级原生弹窗有关的流程，首轮默认走测试替身方案，不把 OS 对话框本身作为主验收对象

### 脚本说明

- `Z:\Playground\CurrentWorking\MediaPlayerNext\scripts\run-tauri-dev.cmd`
  - 会先调用 `VsDevCmd.bat` 再执行 `tauri dev`
- `Z:\Playground\CurrentWorking\MediaPlayerNext\scripts\run-tauri-build.cmd`
  - 会先调用 `VsDevCmd.bat` 再执行 `tauri build`
- `Z:\Playground\CurrentWorking\MediaPlayerNext\scripts\run-cargo-with-msvc.cmd`
  - 用于在已注入 MSVC 环境后执行 Cargo
- `Z:\Playground\CurrentWorking\MediaPlayerNext\scripts\check-runtimes.ps1`
  - 用于校验 `rusqlite`、`ffmpeg`、`mpv`

## 6. 当前已验证状态

- `npm run build:web` 可成功构建当前桌面壳层页面
- `npm run tauri:dev` 可启动 Tauri 宿主并加载前端
- `npm run check` 已验证：
  - `sharp`
  - `rusqlite`
  - `ffmpeg`
  - `mpv`

## 7. Rust/Tauri 开发注意事项

- 当前 `Z:\Playground\CurrentWorking\MediaPlayerNext\src-tauri\Cargo.toml` 仅保留 `crate-type = ["rlib"]`
- 未经明确需要，不要恢复 `staticlib` 或 `cdylib`
- 原因：当前阶段只做桌面宿主初始化，恢复额外 crate type 会显著增大 `target` 体积并拖慢编译
- 如果未来出现 FFI、移动端或原生嵌入需求，再按需补回

### 体积与缓存

- Rust 的 `Z:\Playground\CurrentWorking\MediaPlayerNext\src-tauri\target` 目录会很大，属于正常现象
- 如果需要回收空间，可执行：
  - `Z:\Playground\CurrentWorking\MediaPlayerNext\scripts\run-cargo-with-msvc.cmd clean --manifest-path Z:\Playground\CurrentWorking\MediaPlayerNext\src-tauri\Cargo.toml`
- 清理后下次会重新编译，不属于故障

## 8. 代码与文档规范

- 与用户沟通使用中文
- 注释、文档、说明统一使用中文
- 新增代码优先使用 TypeScript / Rust，不要引入 `any`
- 保持当前阶段代码最小化，不为未来迁移过早抽象
- 优先可维护、可验证、可回退的实现

### 文档同步要求

- 只要任务已经改变当前阶段判断、UI 定义、运行时策略、实施计划或当日日志，就必须同步更新相关文档，不能拖到任务全部结束后再一次性补写
- 执行过程中若存在分 phase 的实施计划文档，必须随着实际进度回填该文档中的 phase 状态、todo 完成情况和 check 状态
- 当前默认需要优先关注并按需更新的文档包括：
  - `Z:\Playground\CurrentWorking\MediaPlayerNext\docs\ui\ui-definition.md`
  - `Z:\Playground\CurrentWorking\MediaPlayerNext\docs\logs\<当天日期>.md`
  - 当前任务对应的执行型文档，例如 `Z:\Playground\CurrentWorking\MediaPlayerNext\docs\runtime\database-management-phase-plan.md`
  - 若任务改变了自动化验收策略，还要同步 `Z:\Playground\CurrentWorking\MediaPlayerNext\docs\testing\tauri-e2e-strategy.md`
  - 若新增了新的主动维护文档，还要同步更新 `Z:\Playground\CurrentWorking\MediaPlayerNext\docs\README.md`
- 文档更新要求与代码改动保持同一批次完成，避免出现“代码已变、文档仍停留旧状态”的情况

## 9. 工作流程

1. 先阅读：`Z:\Playground\CurrentWorking\MediaPlayerNext\docs\README.md`
2. 再阅读：`Z:\Playground\CurrentWorking\MediaPlayerNext\docs\ui\ui-definition.md`
3. 确认当前任务是否属于已定义 UI 切片或其直接支撑项
4. 先改最小范围文件
5. 在执行过程中同步更新相关文档与 phase/check 状态，而不是全部完成后再一次性补文档
6. 运行相关验证：
   - 前端改动至少执行 `npm run build:web`
   - Rust/运行时改动至少执行 `npm run check`
   - Tauri 宿主相关改动必要时执行 `npm run tauri:dev`
7. 若任务属于桌面交互闭环（如设置面板、导入流程、带确认步骤的 destructive flow、前端 + Tauri command 状态闭环），在最终说明末尾追加一句简短提示：是否需要继续执行自动 E2E 验收
8. 在最终说明中明确：修改了什么、验证了什么、未做什么

## 10. 后续迁移原则

- 以 `docs/ui/ui-definition.md` 为准，按布局、样式基座、面板内容逐步迁移与验收
- 实际迁移顺序优先考虑：
  - 壳层与桥接层
  - 布局与样式基座
  - 面板内容切片
  - 重 I/O 后端
  - 数据库与媒体资源解析
  - 播放控制
  - 字幕 sidecar 接入
- 当前仓库的首要目标是：保持后端边界稳定，同时把新的 UI 架构以小步可验收的方式建立起来
