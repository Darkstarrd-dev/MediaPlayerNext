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

- `HTTP_PROXY=http://127.0.0.1:2080`
- `HTTPS_PROXY=http://127.0.0.1:2080`

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

Layer highlights:

- `fast`
  - `fmt`, `check`, `debt-delta`, `forbidden-edges`
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

### Recommended Next Implementation Items

After layered quality pipeline `Phase 0~5` completion, current recommended implementation priorities are:

1. Stabilize existing failing gates:
   - `clippy`
   - `deny`
   - `duplicate-deps`
2. Add missing gates:
   - `capabilities drift`
   - `contract drift`
3. Land CI workflows for layered gates (`fast/standard/heavy/release`)
4. Extend release-level verification:
   - installer/upgrade replay
   - signing/offline smoke
