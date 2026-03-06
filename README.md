# MediaPlayerNext

Rust + Tauri initialization workspace for the next desktop host.

Current stage only lands the new repository skeleton and runtime prerequisites:

- Keep frontend on React + Vite, still showing the default Vite React screen.
- Prepare Rust desktop host with Tauri 2.
- Prepare Node.js subtitle sidecar workspace with `sharp` runtime smoke check.
- Verify `rusqlite`, `sharp`, `ffmpeg`, and `mpv` can be called from the new repository.
- Do not migrate business code yet. Actual migration waits until the current project's theme system is fully converged.

Local absolute runtime paths are tracked in `C:\opencode\MediaPlayerNext\config\local.paths.json`.

Useful commands:

- `npm install`
- `npm run tauri:dev`
- `npm run check`
