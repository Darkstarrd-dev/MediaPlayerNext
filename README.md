# MediaPlayerNext

Rust + Tauri initialization workspace for the next desktop host.

Current stage only lands the new repository skeleton and runtime prerequisites:

- Keep frontend on React + Vite, still showing the default Vite React screen.
- Prepare Rust desktop host with Tauri 2.
- Prepare Node.js subtitle sidecar workspace with `stdio + JSON` host protocol smoke check.
- Verify subtitle sidecar protocol smoke plus `sharp`, `rusqlite`, `ffmpeg`, `ffprobe`, and `mpv` can be called from the new repository.
- Do not migrate business code yet. Actual migration waits until the current project's theme system is fully converged.

Local absolute runtime paths are tracked in `config/local.paths.json`.
Use `config/local.paths.example.json` as the template on a new machine.

Important local path rules:

- `config/local.paths.json` is local-only and ignored by git.
- Do not commit machine-specific path changes through that file.
- If a new runtime key becomes required, update `config/local.paths.example.json` and this README.

## New Machine Handoff

After cloning the repository, a new developer can take over with the following checklist.

### Prerequisites

- Node.js `22.x`
- npm `11.x`
- Rust `1.88.0` or newer via `rustup`
- Visual Studio 2022 MSVC/Build Tools
- WebView2 runtime on Windows

### Local Runtime Paths

Before running checks, copy `config/local.paths.example.json` to `config/local.paths.json`, then confirm it points to valid local binaries:

- `ffmpeg`
- `ffprobe`
- `node`
- `sevenz`
- `mpv`

If your machine uses different install paths, update that file first.

Current usage notes:

- `ffmpeg` and `ffprobe`
  - used by runtime smoke and `B7` playback metadata / frame extraction chain
- `node`
  - used by `B8` subtitle sidecar host wrapper; if omitted, the host falls back to `node` from `PATH`
- `sevenz`
  - used by `B6` archive normalization (`rar/7z -> zip`)
- `mpv`
  - used by runtime smoke and `B7` playback session bootstrap

### Proxy

If direct access to npm or GitHub fails, use the local proxy:

- `HTTP_PROXY=http://127.0.0.1:3066`
- `HTTPS_PROXY=http://127.0.0.1:3066`

### Bootstrap Commands

Run these commands in the repository root:

```bash
npm install
npm run build:web
npm run check
npm run tauri:dev
```

### What These Commands Verify

- `npm install`
  - installs workspace dependencies
- `npm run build:web`
  - builds the React frontend with the minimal Tauri command demo
- `npm run check`
  - verifies subtitle sidecar `stdio + JSON` smoke, `sharp`, `rusqlite`, `ffmpeg`, `ffprobe`, and `mpv`
- `npm run tauri:dev`
  - starts the Tauri host and loads the frontend

Useful commands:

- `npm install`
- `npm run build:web`
- `npm run tauri:dev`
- `npm run check`
- `npm run check:paths`
- `npm run check:sidecar-package`
- `npm run check:quality`
- `npm run check:release`

### 资源路径策略

`P6-4` freezes the current runtime path lookup order and subtitle sidecar bundle layout.

- strategy document
  - `docs/runtime/resource-path-strategy.md`
- verification script
  - `npm run check:paths`
- sidecar package verify
  - `npm run check:sidecar-package`
- output artifact
  - `data/resource-paths/<timestamp>/resource-paths-summary.json`

Current override variables:

- `MPNEXT_RUNTIME_FFMPEG_PATH`
- `MPNEXT_RUNTIME_FFPROBE_PATH`
- `MPNEXT_RUNTIME_MPV_PATH`
- `MPNEXT_RUNTIME_SEVENVZ_PATH`
- `MPNEXT_SUBTITLE_NODE_PATH`
- `MPNEXT_SUBTITLE_ENTRY_PATH`
- `MPNEXT_SUBTITLE_SESSIONS_ROOT`
- `MPNEXT_BACKEND_DB_PATH`
- `MPNEXT_BACKEND_THUMB_CACHE_ROOT`
- `MPNEXT_BACKEND_PLAYBACK_SESSIONS_ROOT`
- `MPNEXT_BACKEND_NORMALIZE_ROOT`

Use env override first when you need to validate an alternate local or packaged-like layout without editing `config/local.paths.json`.

Current packaged runtime policy:

- Node
  - `env override -> PATH node -> explicit failure`
- `ffmpeg/ffprobe/mpv/7z`
  - `env override -> explicit failure`
- this stage does not promise bundled Node or bundled external runtime binaries yet

Current sidecar package rule:

- `src-tauri/tauri.conf.json`
  - bundles `apps/subtitle-sidecar/dist/src/**/*` to resource path `sidecar/`
- Tauri subtitle commands
  - dev mode reads repo `apps/subtitle-sidecar/dist/src/index.js`
  - packaged mode reads bundled resource `sidecar/index.js`

Current Tauri protocol DB rule:

- `MPNEXT_BACKEND_DB_PATH`
  - always overrides
- dev mode
  - falls back to `data/mediaplayernext-dev.db`
- packaged mode
  - falls back to Tauri app local data `mediaplayernext.db`

### Quality Gates

`P6-1` adds a unified quality gate entry under `scripts/quality/`.

- `npm run check:quality`
  - default quality entry (now mapped to `standard` layer)
- `npm run check:quality:fast`
  - fast local feedback gate
- `npm run check:quality:standard`
  - standard pre-commit gate
- `npm run check:quality:heavy`
  - heavy full Rust quality gate
- `npm run check:quality:release`
  - release chain: `heavy + check:release + e2e:desktop:doctor + e2e:desktop`
- `npm run check:quality:legacy`
  - legacy compatibility entry (not recommended for daily use)
- `npm run check:release`
  - builds subtitle sidecar + Tauri bundle and verifies current release artifacts, including packaged sidecar resource presence
- `npm run check:module-boundaries`
- `npm run check:capabilities-drift`
- `npm run check:contract-drift`
- `npm run check:binary-size`
  - module size and boundary baseline gate

Layer highlights:

- `fast`
  - `fmt`, `check`, `module-boundaries`, `debt-delta`, `forbidden-edges`
- `standard`
  - `fast` + `clippy`, `nextest x1`, `duplicate-deps`
- `heavy`
  - `standard` + `nextest x3`, `coverage`, `deny`, `audit`, `udeps`
- `release`
  - `heavy` + `check:release` + desktop e2e doctor + desktop e2e

Artifacts:

- quality runs write to `data/quality-gates/<timestamp>/rust-gates` or `rust-gates-<layer>`
- each `quality-gates-summary.json` includes `layer`

Before the full quality gate can pass on a new machine, install the required Cargo subcommands:

- `cargo-nextest`
- `cargo-llvm-cov`
- `cargo-deny`
- `cargo-audit`
- `cargo-udeps`

Current validated tool versions for the Rust `1.88.0` project baseline:

- `cargo-nextest 0.9.114`
- `cargo-llvm-cov 0.8.4`
- `cargo-deny 0.19.0`
- `cargo-audit 0.22.1`
- `cargo-udeps 0.1.56`

### 当前待办（含最新进度）

#### 已完成进度（2026-03-12）

1. `AppShell.tsx` 单体继续拆分并显著降体积
   - 已从历史大文件基线持续下降到当前约 `398` 行（已低于 warning 线 `400`）
   - 已拆出并接入：
     - `use-app-shell-layout*`
     - `use-app-shell-workspace-cursor.ts`
     - `use-app-shell-workspace-selection.ts`
     - `use-app-shell-workspace-data.ts`
     - `use-app-shell-item-data.ts`
     - `use-app-shell-import-activities.ts`
     - `use-app-shell-import-controller.ts`
     - `use-app-shell-import-listeners.ts`
     - `use-app-shell-directory-picker.ts`
     - `use-app-shell-scan-state.ts`
     - `app-shell-sidebar-pane.tsx`
     - `app-shell-main-pane.tsx`
     - `app-shell-metadata-pane.tsx`
     - `app-shell-header.tsx`
     - `app-shell-settings-panel.tsx`
     - `app-shell-clear-database-dialog.tsx`
     - `app-shell-settings-ui-page.tsx`
     - `app-shell-settings-database-page.tsx`
     - `app-shell-settings-types.ts`
     - `use-app-shell-database-settings.ts`
     - `use-app-shell-view-state.ts`
     - `app-shell-workspace.tsx`
     - `app-shell-panels.tsx`
     - `use-app-shell-workspace-state.ts`
     - `use-app-shell-workspace-navigation.ts`
     - `use-app-shell-overlay-escape.ts`
     - `app-shell-import-batch.ts`
     - `use-app-shell-bootstrap-scan-snapshot.ts`
2. `runtime_storage` 已完成首轮结构拆分（Rust 侧）
   - 原 `src-tauri/src/runtime_storage.rs` 已拆为目录模块：
     - `src-tauri/src/runtime_storage/mod.rs`
     - `src-tauri/src/runtime_storage/config.rs`
     - `src-tauri/src/runtime_storage/defaults.rs`
     - `src-tauri/src/runtime_storage/filesystem.rs`
   - 对外行为保持一致，仅做结构拆分与测试写法收敛
   - `module-boundaries` 冻结债文件数已从 `8` 降到 `7`
   - `debt-delta` 总计数已从 `411` 降到 `396`
3. `src-tauri/src/lib.rs` 已完成一轮协议层拆分
   - 新增 `src-tauri/src/app_protocol.rs`，收口 `thumb/media/archive` 协议响应与错误响应构建
   - `lib.rs` 当前行数已进一步降到 `1187`（较 baseline `2104` 持续下降）
   - 本轮未改命令语义，仅做适配层结构拆分
4. `lib.rs` 的 Tauri 命令已开始按领域拆分
   - 新增 `src-tauri/src/tauri_commands.rs`（greet + runtime storage 相关命令）
   - 新增 `src-tauri/src/tauri_subtitle_commands.rs`（subtitle 相关命令）
   - `lib.rs` 的 `tauriCommandAnnotations` 已从 `31` 降到 `21`
5. `lib.rs` 命令域拆分第二轮已完成（workspace/scan/media/playback）
   - 新增 `src-tauri/src/tauri_workspace_commands.rs`
   - 新增 `src-tauri/src/tauri_scan_commands.rs`
   - 新增 `src-tauri/src/tauri_media_commands.rs`
   - 新增 `src-tauri/src/tauri_playback_commands.rs`
   - `lib.rs` 当前行数已进一步降到 `397`，`tauriCommandAnnotations` 已降到 `0`
6. baseline 与 frozen 记录已完成同步清理
   - `config/quality/module-boundaries-baseline.json` 已移除失效项 `src-tauri/src/runtime_storage.rs`
   - `module-boundaries` 的 `missingFrozenFileCount` 已从 `1` 归零到 `0`
   - `frozenDebtFileCount` 已从 `7` 降到 `6`
7. `subtitle_sidecar` 已完成目录化拆分并退出 frozen debt
   - `src-tauri/src/subtitle_sidecar.rs` 已拆为：
     - `src-tauri/src/subtitle_sidecar/mod.rs`
     - `src-tauri/src/subtitle_sidecar/paths.rs`
     - `src-tauri/src/subtitle_sidecar/wire.rs`
     - `src-tauri/src/subtitle_sidecar/tests.rs`
   - `debt-delta totalOccurrenceCount` 已从 `392` 继续降到 `367`
8. `archive.rs` 已完成第一轮归档规范化逻辑下沉
    - 新增 `crates/app-core/src/archive_normalize.rs`
    - `archive` 领域规范化主流程与 ID 生成辅助已迁到新模块
    - `crates/app-core/src/archive.rs` 行数已从 `1139` 降到 `901`
    - frozen debt 仍在，但体量已进入持续下降区间
9. `archive.rs` 已完成第二轮读取/定位链路拆分
   - 新增 `crates/app-core/src/archive_resolve.rs`
   - `read/resolve` 相关结构与函数从 `archive.rs` 下沉到独立模块
   - `crates/app-core/src/archive.rs` 行数已从 `901` 继续降到 `812`
   - `debt-delta` 保持 `baselineMatched=true`，未新增 debt path
10. `asset.rs` 已完成首轮解析链路拆分
     - 新增 `crates/app-core/src/asset_resolve.rs`
     - `resolve_asset` 主链路已下沉，`asset.rs` 通过 re-export 保持原 API
     - `crates/app-core/src/asset.rs` 行数已从 `895` 降到 `826`
     - `debt-delta` 保持 `baselineMatched=true`，未新增 debt path
11. `archive.rs` 已完成第三轮索引/快照链路拆分
     - 新增 `crates/app-core/src/archive_index.rs`
     - `index_library_archives` 与 `archive_snapshot` 已下沉并通过 re-export 保持 API
     - `crates/app-core/src/archive.rs` 行数已从 `812` 继续降到 `678`
12. `asset.rs` 已完成第二轮目录构建/快照链路拆分
     - 新增 `crates/app-core/src/asset_catalog.rs`
     - `ensure_media_assets_for_library` 与 `asset_snapshot_for_library` 已下沉并保持 API 不变
     - `crates/app-core/src/asset.rs` 行数已从 `826` 继续降到 `657`
13. `playback.rs` 已完成运行时链路拆分并退出 frozen debt
     - 新增 `crates/app-core/src/playback_runtime.rs`
     - `resolve_media_asset_path` 对外 API 保持兼容，`probe/open` 内部链路下沉
      - `crates/app-core/src/playback.rs` 行数已从 `618` 降到 `472`，降到 hardMax 以内
14. `archive/asset` 已完成测试支撑下沉并继续降体积
      - 新增测试支撑文件：`crates/app-core/src/archive/archive_test_support.rs`、`crates/app-core/src/asset/asset_test_support.rs`
      - `archive.rs` 行数已从 `678` 继续降到 `463`
      - `asset.rs` 行数已从 `657` 继续降到 `472`
15. `scan/thumbnail` 已完成测试支撑下沉并退出 frozen debt
      - 新增测试支撑文件：`crates/app-core/src/scan/scan_test_support.rs`、`crates/app-core/src/thumbnail/thumbnail_test_support.rs`
      - `scan.rs` 行数已从 `677` 降到 `476`
      - `thumbnail.rs` 行数已从 `611` 降到 `398`
      - `module-boundaries` 中 `frozenDebtFileCount` 已降到 `0`
16. 本轮质量门禁结果
        - `npm run build:web`：通过
        - `npm run check`：通过
        - `npm run check:module-boundaries`：通过
        - `npm run check:debt`：通过
        - `npm run check:quality:fast`：通过
17. `subtitle_sidecar` 测试文件 warning 已压线
       - `src-tauri/src/subtitle_sidecar/tests.rs` 已从 `403` 收敛到 `400`
       - `module-boundaries` 的 warning 文件已从 `5` 降到 `4`
18. app-core warning 已完成清零
       - `archive.rs`：`463 -> 449`
       - `asset.rs`：`472 -> 447`
       - `scan.rs`：`476 -> 441`
       - `playback.rs`：通过测试支撑下沉降到 `272`
       - 新增 `crates/app-core/src/playback/playback_test_support.rs` 与 `crates/app-core/src/playback_test_support.rs`
       - `module-boundaries` 最新结果：`warningCount=0`、`frozenDebtFileCount=0`
19. 最新快速质量门禁（20260312-224200）
       - `check:quality:fast` 全部通过（`5/5`）
       - `debt-delta`：`baselineMatched=true`、`addedEntries=[]`
       - `totalOccurrenceCount` 进一步降到 `270`
20. 质量缺口收口：`capabilities-drift / contract-drift` 已落地
       - 新增 `scripts/quality/check-capabilities-drift.ps1` + `config/quality/capabilities-baseline.json`
       - 新增 `scripts/quality/check-contract-drift.mjs` + `config/quality/contract-drift-baseline.json`
       - `run-rust-gates.ps1` 分层门禁已接入两项 drift gate（P1）
21. `.github` 首版工作流与 PR 模板已落库
       - `quality-fast.yml`（main push/PR）
       - `quality-standard.yml`（PR + workflow_dispatch）
       - `release-verify.yml`（workflow_dispatch）
       - `pull_request_template.md`
22. benchmark 与发布评审文档模板已补齐
       - `docs/benchmarks/benchmark-threshold-policy.md`
       - `docs/quality/release-go-no-go-template.md`
23. binary size 基线脚本已落地
       - 新增 `scripts/quality/check-binary-size.ps1`
       - 新增 `config/quality/binary-size-baseline.json`
       - 新增命令 `npm run check:binary-size`
       - 当前验证：`mediaplayernext.exe` = `10665984 bytes`（阈值 `268435456 bytes`）

#### 待拆分 / 待完成

1. 继续压缩 top-large `rust-domain` 文件到 `targetLines=350`
   - `crates/app-core/src/archive.rs`（449 行）
   - `crates/app-core/src/asset.rs`（447 行）
   - `crates/app-core/src/scan.rs`（441 行）
2. 完善发布增强验收
   - installer/upgrade replay
   - signing/offline smoke
3. 补 benchmark compare + threshold 与 `cargo bloat` 自动化门禁
