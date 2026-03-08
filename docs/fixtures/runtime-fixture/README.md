# runtime-fixture 说明

本目录用于固定 `P6-2` 阶段 runtimes 与 DB fixture 的坏路径口径。

当前首轮固定内容：

- 缺失 runtime binary
- runtime 非零退出
- runtime 空输出
- 非法 sqlite fixture
- 伪装成最新 schema 但缺表的 sqlite fixture
- 高于当前支持版本的 sqlite schema
- 缺少必需列的 upgrade / replay fixture
- 缺少必需索引的 replay fixture
- 缺少必需 foreign key 的 replay fixture
- 带 orphan foreign key row 的 replay fixture

说明：

- runtimes bad path 当前通过 `src-tauri/src/runtime_check.rs` 单元测试动态构造，不额外提交可执行样本。
- 非法 sqlite fixture 当前通过测试内写入非 sqlite 字节模拟，避免把损坏二进制样本直接放进仓库。
- 伪装最新 schema / 未来 schema 当前通过测试内动态写 `user_version` 模拟，不直接提交额外损坏数据库文件。
- 缺列 replay fixture 当前已补最小 SQL 样本：`crates/media-db/tests/fixtures/schema_v1_missing_exists_flag_fixture.sql`。
- 缺索引 replay fixture 当前已补最小 SQL 样本：`crates/media-db/tests/fixtures/schema_v2_missing_archive_entry_index_fixture.sql`。
- 缺 foreign key replay fixture 当前已补最小 SQL 样本：`crates/media-db/tests/fixtures/schema_v2_missing_thumbnail_foreign_key_fixture.sql`。
- orphan row replay fixture 当前已补最小 SQL 样本：`crates/media-db/tests/fixtures/schema_v2_orphan_thumbnail_asset_fixture.sql`。
- `MediaDatabase::open(...)` 当前会主动启用 SQLite `foreign_keys`，避免约束漂移只在更晚阶段才暴露。
- `MediaDatabase::open(...)` 当前还会执行 `foreign_key_check`，拦住已落盘的脏引用数据。
- 本目录当前先收口说明文档；后续如果需要固定更多 upgrade/replay 异常样本，再补真实 fixture 文件。
