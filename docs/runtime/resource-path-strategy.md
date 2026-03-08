# 资源路径策略

## 文档定位

本文件用于收口 `P6-4` 的资源路径策略，明确开发态、打包态与本地 override 的查找顺序，避免这些规则继续散落在脚本、README 与代码里。

## 当前目标

- 先固定首轮查找顺序
- 先为开发态与“打包前模拟验证”提供统一脚本
- 不在当前阶段承诺最终生产包内资源布局已经全部落地

## 总原则

1. 本地 override 优先于仓库内默认路径
2. 开发态优先依赖 `config/local.paths.json` 与仓库相对路径
3. 打包态若尚未内置对应资源，必须失败得足够明确，不能静默回退到不可预期位置
4. migrations 当前属于嵌入式资源，不走外部路径查找

## 资源分组与查找顺序

### 1. ffmpeg / ffprobe / mpv

适用面：

- runtime smoke
- playback probe/open

开发态首轮顺序：

1. 环境变量
   - `MPNEXT_RUNTIME_FFMPEG_PATH`
   - `MPNEXT_RUNTIME_FFPROBE_PATH`
   - `MPNEXT_RUNTIME_MPV_PATH`
2. `config/local.paths.json`
   - `ffmpeg`
   - `ffprobe`
   - `mpv`
3. 代码内默认路径（仅 backend harness 当前保留兜底）

当前说明：

- `scripts/check-runtimes.ps1` 现在优先读环境变量，再读 `config/local.paths.json`
- backend harness 的 `ffprobe/mpv` 现在也支持环境变量优先

打包态首轮策略：

- 先允许环境变量 override
- 未内置资源前，不承诺自动 bundle 路径查找
- 若路径不可用，应明确失败，不静默退回不存在的硬编码路径

### 2. sevenz

适用面：

- archive normalize

开发态首轮顺序：

1. `MPNEXT_RUNTIME_SEVENVZ_PATH`
2. `config/local.paths.json.sevenz`
3. backend harness 默认路径

打包态首轮策略：

- 先允许环境变量 override
- bundle 内置路径策略留待后续收口，但不能与开发态规则混写在同一处

### 3. subtitle sidecar node

适用面：

- subtitle host wrapper

开发态首轮顺序：

1. `MPNEXT_SUBTITLE_NODE_PATH`
2. `config/local.paths.json.node`
3. `PATH` 中的 `node`

打包态首轮策略：

1. `MPNEXT_SUBTITLE_NODE_PATH`
2. `PATH` 中的 `node`
3. 若仍不存在，则显式失败

说明：

- 当前仓库还没有把 Node runtime 打进发布包，因此打包态首轮策略仍以 override 或系统 Node 为主

### 4. subtitle sidecar entry

适用面：

- subtitle sidecar JS 入口

开发态首轮顺序：

1. `MPNEXT_SUBTITLE_ENTRY_PATH`
2. 仓库默认 `apps/subtitle-sidecar/dist/src/index.js`

打包态目标策略：

1. `MPNEXT_SUBTITLE_ENTRY_PATH`
2. Tauri bundle resource `sidecar/index.js`

当前说明：

- `src-tauri/tauri.conf.json` 已声明把 `apps/subtitle-sidecar/dist/**/*` 打进 bundle 资源目录
- Tauri command 链路现在会在非 dev 模式下解析 `sidecar/index.js`

### 5. subtitle sessions root

开发态顺序：

1. `MPNEXT_SUBTITLE_SESSIONS_ROOT`
2. 默认 `data/cache/subtitle/sessions`

打包态顺序：

1. `MPNEXT_SUBTITLE_SESSIONS_ROOT`
2. Tauri `app cache dir/subtitle/sessions`

### 6. backend cache / db roots

适用面：

- backend harness
- 后续 repository / adapter 验证

backend harness 当前顺序：

1. `MPNEXT_BACKEND_DB_PATH`
2. `MPNEXT_BACKEND_THUMB_CACHE_ROOT`
3. `MPNEXT_BACKEND_PLAYBACK_SESSIONS_ROOT`
4. `MPNEXT_BACKEND_NORMALIZE_ROOT`
5. 未设置时回落到仓库下 `data/` 默认路径

Tauri protocol host 当前顺序：

1. `MPNEXT_BACKEND_DB_PATH`
2. dev 模式回落到仓库 `data/mediaplayernext-dev.db`
3. 打包态回落到 Tauri `app local data dir/mediaplayernext.db`

当前说明：

- `thumb://` / `media://` / `archive://` 现在不再把打包态数据库路径硬编码回仓库根目录
- `thumb/playback/normalize` 的缓存根路径仍以 backend harness 默认值为主，尚未统一切到 Tauri app cache

### 7. migrations

当前策略：

- `media-db` migrations 为嵌入式资源
- 不走外部路径查找
- 因此开发态 / 打包态在 migrations 上不需要额外路径配置文件

## 当前代码对应关系

- `scripts/check-runtimes.ps1`
- `src-tauri/src/bin/backend_harness.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/subtitle_sidecar.rs`
- `src-tauri/tauri.conf.json`
- `config/local.paths.example.json`

## 当前校验入口

- `npm run check:paths`
- `npm run check:sidecar-package`
- 结果产物：`data/resource-paths/<timestamp>/resource-paths-summary.json`

其中 `npm run check:sidecar-package` 会验证：

- `src-tauri/tauri.conf.json` 是否声明 sidecar bundle resource
- `apps/subtitle-sidecar/dist/src/index.js` 是否存在
- `cargo tauri build` 产出的 `target/release` 下是否已出现 `sidecar/index.js` 的打包态产物

该脚本当前会验证：

- 本地 override / config / 默认路径的解析结果
- 关键开发态路径是否存在
- sidecar 入口与缓存目录默认值是否可解析
- 当前策略说明是否已形成结构化 summary

## 当前已知未完成项

- bundle 内置 `ffmpeg/ffprobe/mpv/7z` 路径尚未形成最终发布布局
- 还没有把 Node runtime 一并打进发布包，当前仍依赖 env override 或系统 `node`

因此 `P6-4` 当前状态应理解为：subtitle sidecar 的 bundle 入口已开始进入打包态闭环，但 runtimes 与 bundled Node 仍未最终收口。
