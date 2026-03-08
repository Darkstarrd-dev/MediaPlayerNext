# runtime-fixture 说明

本目录用于固定 `P6-2` 阶段 runtimes 与 DB fixture 的坏路径口径。

当前首轮固定内容：

- 缺失 runtime binary
- runtime 非零退出
- runtime 空输出
- 非法 sqlite fixture

说明：

- runtimes bad path 当前通过 `src-tauri/src/runtime_check.rs` 单元测试动态构造，不额外提交可执行样本。
- 非法 sqlite fixture 当前通过测试内写入非 sqlite 字节模拟，避免把损坏二进制样本直接放进仓库。
- 本目录当前先收口说明文档；后续如果需要固定更多 upgrade/replay 异常样本，再补真实 fixture 文件。
