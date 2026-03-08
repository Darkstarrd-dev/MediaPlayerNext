# 2026-03-08 P6-2 bad path 验证记录

本记录用于固定 `P6-2` 首轮 bad path 补强的验证口径，确保后续进入 `I1` 前，前端不需要再猜测 runtimes、DB fixture、custom protocol 与 subtitle sidecar 的错误表现。

## 本轮范围

- runtimes：缺失二进制、非零退出、空输出
- DB：非法 sqlite fixture 打开失败
- protocol：`thumb://` / `media://` / `archive://` 缺失资源返回 `404`
- sidecar：缺失入口、timeout、retryable crash、malformed payload、missing payload

## 代码入口

- `src-tauri/src/runtime_check.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/subtitle_sidecar.rs`
- `crates/media-db/tests/database_integration.rs`

## 固定样本

- `docs/fixtures/runtime-fixture/README.md`
- `docs/fixtures/sidecar-fixture/malformed-payload.response.txt`
- `docs/fixtures/sidecar-fixture/missing-payload.response.json`
- `packages/contracts/fixtures/app-error.not-found.sample.json`
- `packages/contracts/fixtures/app-error.timeout.sample.json`
- `packages/contracts/fixtures/app-error.invalid-argument.sample.json`
- `docs/fixtures/thumbnail-fixture/thumbnail-protocol-invalid-uri.expected.json`
- `docs/fixtures/playback-fixture/media-protocol-not-found.expected.json`
- `docs/fixtures/playback-fixture/archive-protocol-invalid-uri.expected.json`

## 本轮新增验证点

### runtimes bad path

- 缺失 runtime binary -> 返回 `runtime binary not found`
- runtime 非零退出 -> 返回 `runtime binary returned non-zero status`
- runtime 空输出 -> 返回 `runtime binary produced empty output`

### DB fixture bad path

- 非法 sqlite fixture -> `MediaDatabase::open(...)` 失败
- 错误信息包含 `not a database`

### protocol missing resource

- `media://asset/<missing>` -> `404` + `media asset not found`
- `media://asset/<existing>` 且磁盘文件缺失 -> `404` + `media file not found`
- `archive://entry/<missing>` -> `404` + `archive entry not found`
- `archive://entry/<existing>` 且 zip 文件缺失 -> `404` + `archive entry file not found`
- `thumb://cache` / `media://asset` / `archive://entry` -> `400 + INVALID_ARGUMENT`
- protocol 错误响应头已固定：`x-mediaplayernext-error-code`、`x-mediaplayernext-error-retriable`

### sidecar bad payload

- 缺失 sidecar 入口 -> 返回 `subtitle sidecar entry missing`
- malformed JSON -> 返回 `parse subtitle sidecar response`
- `ok=true` 但 payload 缺失 -> 返回 `response payload missing`
- 无响应 -> 返回包含 `timed out` 的错误
- 首次 crash 且可重试 -> `health()` 成功，`restartCount=1`

### error contract 对齐

- `apps/subtitle-sidecar/src/protocol.ts` 已把 `AppError.code` 从任意字符串收紧为稳定枚举
- `AppError.details` 已与 `packages/contracts` 口径对齐为可选字段
- `packages/contracts` 已补 `NOT_FOUND` / `TIMEOUT` / `INVALID_ARGUMENT` 三类 bad path fixture

## 验证命令

```bash
scripts/run-cargo-with-msvc.cmd test --workspace
npm run check
npm run test:contracts
```

## 本轮结果

- Rust workspace tests：通过
- sidecar check：通过
- runtime check：通过
- contracts fixtures parse：通过

## 当前判断

- `P6-2` 已经补上首轮最关键的 bad path 回归点，但阶段仍未结束。
- 当前还缺更完整的错误码/contract 收口，以及更系统的 DB upgrade 异常样本与 runtime 路径策略文档。
- 当前这轮已经足够支撑后续继续推进 `P6-2`，并减少 `I1` 接线时的临场判断。
