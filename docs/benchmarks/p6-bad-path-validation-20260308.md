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
- fixture 伪装成最新 schema 版本但缺少必需表 -> 返回 `required table missing for current schema`
- database `user_version` 高于当前支持版本 -> 返回 `is newer than supported`
- v1 fixture 缺少必需列（如 `sources.exists_flag`）-> 返回 `required column missing for current schema`
- v2 replay fixture 缺少必需索引（如 `idx_archive_entries_archive_path`）-> 返回 `required index missing for current schema`
- v2 replay fixture 缺少必需 foreign key（如 `thumbnails.asset_id -> media_assets.id`）-> 返回 `required foreign key missing for current schema`
- v2 replay fixture 带有 orphan row（如 thumbnail 指向不存在 asset）-> 返回 `foreign key integrity violation for current schema`

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

## 补充更新：DB upgrade 守卫

- `crates/media-db/src/migrations.rs` 现在会在迁移前拒绝比当前仓库更高的 schema version
- 迁移完成后会校验当前 version 所需的关键表与关键列是否真实存在
- 迁移完成后会校验当前 version 所需的关键表、关键列与关键索引是否真实存在
- 打开连接时已显式启用 `PRAGMA foreign_keys = ON`
- schema 校验结束前会执行 `PRAGMA foreign_key_check`
- 当前已固定两类异常：
  - fixture 把 `user_version` 写成最新版本，但实际缺少 `archives/archive_entries/thumbnails`
  - database `user_version` 已漂到未来版本

## 补充更新：DB replay fixture 列漂移

- 新增测试 fixture：`crates/media-db/tests/fixtures/schema_v1_missing_exists_flag_fixture.sql`
- 当前已额外固定一类 replay 异常：
  - v1 fixture 表面上仍像旧版本，但 `sources.exists_flag` 这类核心列已经丢失
- 当前结果：`MediaDatabase::open(...)` 会在迁移完成后的 schema 校验阶段直接失败，而不是把问题拖到更后面的 repository 查询

## 补充更新：DB replay fixture 索引漂移

- 新增测试 fixture：`crates/media-db/tests/fixtures/schema_v2_missing_archive_entry_index_fixture.sql`
- 当前已额外固定一类 replay 异常：
  - v2 fixture 缺少 `idx_archive_entries_archive_path` 这类关键索引
- 当前结果：`MediaDatabase::open(...)` 会在 schema 校验阶段直接失败，而不是等 archive entry 查询/替换路径在性能或唯一性语义上悄悄漂移

## 补充更新：DB replay fixture foreign key 漂移

- 新增测试 fixture：`crates/media-db/tests/fixtures/schema_v2_missing_thumbnail_foreign_key_fixture.sql`
- 当前已额外固定两类约束语义：
  - 打开连接后 `foreign_keys` pragma 必须为 `1`
  - v2 fixture 缺少 `thumbnails.asset_id -> media_assets.id` 这类关键 foreign key 时，打开数据库应立即失败
- 当前结果：`MediaDatabase::open(...)` 现在会更早拦住“表、列、索引看起来都还在，但引用完整性已经漂移”的 replay 问题

## 补充更新：DB replay fixture orphan rows

- 新增测试 fixture：`crates/media-db/tests/fixtures/schema_v2_orphan_thumbnail_asset_fixture.sql`
- 当前已额外固定一类数据层 replay 异常：
  - schema 本身仍完整，但数据里已经存在 orphan foreign key row
- 当前结果：`MediaDatabase::open(...)` 会在 `foreign_key_check` 阶段直接失败，而不是等仓库查询或 UI 接线时才发现脏引用

## 当前判断

- `P6-2` 已经补上首轮最关键的 bad path 回归点，但阶段仍未结束。
- 当前还缺更完整的错误码/contract 收口，以及更系统的 DB upgrade 异常样本与 runtime 路径策略文档。
- 当前这轮已经足够支撑后续继续推进 `P6-2`，并减少 `I1` 接线时的临场判断。
