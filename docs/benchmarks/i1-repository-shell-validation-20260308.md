# 2026-03-08 I1 repository shell 首轮验证记录

本记录用于固定 `I1` 第一轮“repository / adapter / app shell 骨架”验证结果。

## 本轮新增内容

- `apps/desktop/src/repositories/media-repository.ts`
- `apps/desktop/src/repositories/tauri-media-repository.ts`
- `apps/desktop/src/adapters/tauri/commands.ts`
- `apps/desktop/src/adapters/tauri/protocols.ts`
- `apps/desktop/src/app/AppShell.tsx`
- `apps/desktop/src/app/MediaRepositoryProvider.tsx`
- `docs/contracts/ui-dependency-matrix.md`

## 本轮固定口径

- React 组件不再直接依赖 `invoke`
- `runtime_smoke_check` 与 `subtitle.*` 先通过 repository 对外暴露
- `thumb://` / `media://` / `archive://` URL 统一从 adapter / repository 生成
- `library/scan/items/archive/thumbnail/playback` 仍以 repository 占位为主，等待 `src-tauri` command/channel 接线

## 本轮验证命令

```bash
npm run build:web
npm --workspace @mediaplayernext/desktop run lint
```

## 本轮验证结果

- `npm run build:web` 通过
- `npm --workspace @mediaplayernext/desktop run lint` 通过
- `apps/desktop/src/App.tsx` 已不再直接出现 `invoke(...)`
- app shell 已能通过 repository 调用 runtime diagnostics 与 subtitle host 能力

## 当前判断

- `I1` 已从 command demo 进入 repository shell 阶段
- 这仍不是“真实页面 UI 已开始迁移”，因为 theme/CSS/页面内部调用链仍未收口
- 但页面交互依赖矩阵已经足够支撑后续 `src-tauri` 命令接线与 repository 方法补实

## 下一步

- 优先补 `src-tauri` 的 `library/scan/items/archive/thumbnail` 最小 command/channel wiring
- 再让 `tauriMediaRepository` 把当前 planned 方法逐步替换为真实调用
