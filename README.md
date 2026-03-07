# MediaPlayerNext

Rust + Tauri initialization workspace for the next desktop host.

Current stage only lands the new repository skeleton and runtime prerequisites:

- Keep frontend on React + Vite, still showing the default Vite React screen.
- Prepare Rust desktop host with Tauri 2.
- Prepare Node.js subtitle sidecar workspace with `sharp` runtime smoke check.
- Verify `rusqlite`, `sharp`, `ffmpeg`, and `mpv` can be called from the new repository.
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
- `sevenz`
- `mpv`

If your machine uses different install paths, update that file first.

Current usage notes:

- `ffmpeg` and `ffprobe`
  - used by runtime smoke and `B7` playback metadata / frame extraction chain
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
  - verifies `sharp`, `rusqlite`, `ffmpeg`, `ffprobe`, and `mpv`
- `npm run tauri:dev`
  - starts the Tauri host and loads the frontend

Useful commands:

- `npm install`
- `npm run build:web`
- `npm run tauri:dev`
- `npm run check`
