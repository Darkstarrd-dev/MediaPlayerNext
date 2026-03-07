# MediaPlayerNext 后端先行具体实施计划 B1-B4 v1

## 1. 文档定位

本文件不是替代 `docs/00-MediaPlayerNext_实施计划_v2.md`，而是把 v2 总纲转成当前仓库可直接执行的实施清单。

三份文档的职责固定如下：

- `docs/00-MediaPlayerNext_实施计划_v2.md`
  - 负责总路线、工作包边界、总体顺序与里程碑
- `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
  - 负责质量门禁、测试要求、迁移流程、CI/PR 约束
- `docs/02-MediaPlayerNext_后端先行具体实施计划_B1-B4_v1.md`
  - 负责把当前阶段拆成可执行的阶段、目录、交付物、验收标准与验证命令

本文件只覆盖 **后端先行阶段的前四个里程碑**：

- `B1`：共享模型、错误码、任务模型、CLI 骨架完成
- `B2`：SQLite schema + migration + repository 完成
- `B3`：扫描 / 入库 / 增量更新完成（首版）
- `B4`：zip 列目录与归档索引完成

不覆盖：

- `B5` 缩略图主链路 Rust 化
- `B6` `rar/7z` 归一化
- `B7` 播放链路
- `B8` 字幕 sidecar 宿主协议
- UI 对接与页面迁移

这样拆的原因是：当前仓库仍处于 bootstrap 状态，必须先把数据地基、扫描闭环与 zip 索引做稳，后续缩略图和 UI 才不会返工。

---

## 2. 已完成内容与当前进度

截至目前，`B1` 与 `B2` 已完成，`B3` 已进入收尾阶段，`B4` 尚未开始。

### 2.1 已完成内容

#### `B1` 已完成

- 根 `Cargo.toml` workspace 已建立
- `crates/shared-model`、`crates/app-core`、`crates/media-db`、`crates/media-io` 已变成真实 crate
- `shared-model` 已落首批 ID、错误码、任务状态、任务进度、分页与记录模型
- `packages/contracts` 已建立 `src/fixtures/tests` 初版结构，并有 Zod 校验测试
- 开发期 CLI harness 已落到 `src-tauri/src/bin/backend_harness.rs`
- `docs/fixtures/` 与 `docs/benchmarks/` 目录已建立

#### `B2` 已完成

- `media-db` 已具备数据库入口、migration runner、repository 首版实现
- `0001_init_core.sql` 与 `0002_init_archive_and_thumb.sql` 已落库
- 空库初始化、重复迁移、`N-1 -> N` 升级、事务回滚、1000+ 记录写入测试已存在

#### `B3` 已完成的部分

- 文件发现、路径归一化、扩展名分类已落地
- `scan add-library`、`scan run`、`scan resume`、`scan stats`、`scan diff` 已可通过 CLI 调用
- 首扫、重扫、未变化跳过、`exists=false` tombstone 已具备最小语义
- 扫描失败会写入 `failed` 任务状态
- `small-fixture` 扫描 smoke 样本与快照 fixture 已建立

### 2.2 当前进度判断

- `B1`：完成
- `B2`：完成
- `B3`：进行中（已完成主链路，仍需补更完整的失败恢复和结果固化）
- `B4`：未开始

### 2.3 当前最自然的下一步

1. 收尾 `B3` 剩余内容，补更完整的失败恢复与扫描结果固化
2. 开始 `B4`，推进 zip 目录、页序、entry 读取与归档索引落库

---

## 3. 当前仓库基线

截至当前更新时，仓库实际状态如下：

- `src-tauri` 已能运行最小 Tauri 宿主，并已接入开发期 `backend_harness`
- `crates/app-core`、`crates/media-io`、`crates/media-db`、`crates/shared-model` 已为真实 crate；`media-thumb`、`media-playback` 仍未启动
- `packages/contracts` 已建立 `models/errors/fixtures/tests` 初版结构，`commands/channels/events` 仍待补全
- `rusqlite` migration 与 repository 首版已建立，数据库 smoke tests 已可运行
- `sharp` 仅作为 Node sidecar 依赖验证，不是业务主链路
- 扫描 CLI 已能跑通 `add-library/run/resume/stats/diff` 最小链路

因此当前阶段的首要目标，已经从“把纯 Rust 能力沉到 `crates/*`”推进为“继续补齐 B3 剩余语义，并准备切入 B4 的 zip 主链路”。

---

## 4. 执行原则

### 4.1 总原则

1. **先 crate，后宿主**
   - 先把逻辑写进纯 Rust crate，再由 `src-tauri` 暴露
2. **先 contracts，后实现**
   - 先冻结 DTO / 错误码 / 任务状态，再写 service
3. **先落库，后派生**
   - 先把 `source/archive/asset/task` 落到稳定 schema，再接缩略图等派生能力
4. **先 CLI/测试，后 UI**
   - 当前阶段所有能力必须能在无 UI 环境下验证
5. **先最小主链路，后扩展边角**
   - 先覆盖图片与 zip 主链路，不提前做 `rar/7z`、视频首帧、复杂 UI

### 4.2 当前阶段禁止事项

- 不把业务实现继续堆进 `src-tauri`
- 不提前迁旧仓 UI/theme
- 不让前端通过 command 获取大块媒体字节
- 不把 `sharp` 继续推进为长期缩略图主链路
- 不为了“看起来完整”而同时开做 `B1-B8`

### 4.3 质量门禁继承关系

本文件所有阶段默认继承 `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md` 中的要求，尤其是：

- 契约优先流程
- migration 固定流程
- 高频路径 benchmark 要求
- `unsafe/unwrap/todo!/panic!` 债务 delta 守门
- command/channel/custom protocol/sidecar 契约测试要求

---

## 5. 目标目录与职责落位

在 `B1-B4` 内，目录职责固定如下：

```text
MediaPlayerNext/
  Cargo.toml                       # 根 workspace（本阶段新增）
  apps/
    desktop/                       # 现阶段只保留最小壳，不承载迁移实现
    subtitle-sidecar/              # 现阶段只保留 sharp 与后续 sidecar 骨架
  crates/
    app-core/                      # use case 编排、ports、CLI 调用入口适配
    media-io/                      # 文件发现、zip 读取、路径归一化、媒体分类
    media-db/                      # SQLite 连接、migration、repository 实现
    shared-model/                  # DTO、ID、错误码、任务状态
  src-tauri/                       # 极薄宿主；本阶段只做最小接线或不改动
  packages/
    contracts/                     # TS/Zod 合同与示例 JSON
  docs/
    fixtures/                      # 本阶段新增，放样本与 snapshot 说明
    benchmarks/                    # 本阶段新增，放 benchmark 说明与结果基线
```

说明：

- `media-thumb` 在本文件阶段内仍只做设计准备，不进入真实开发
- `media-playback` 在本文件阶段内不落真实代码
- `src-tauri` 在 `B1-B4` 不是核心施工面，只允许做最小 wiring

---

## 6. 阶段拆分总览

| 阶段 | 目标 | 核心交付物 | 是否阻塞后续 |
|---|---|---|---|
| `B1` | 地基冻结 | workspace、shared-model、contracts、CLI 骨架、tracing | 是（已完成） |
| `B2` | 数据地基 | migration、repository、DB smoke tests | 是（已完成） |
| `B3` | 路径到数据库闭环 | 扫描/入库最小闭环、任务状态、fixture 扫描报告 | 是（进行中） |
| `B4` | zip 与归档索引 | zip 目录/排序/entry 读取、archive_entries 落库 | 是 |

执行顺序必须串行推进：`B1 -> B2 -> B3 -> B4`。

原则上不允许跳过 `B2/B3` 直接做缩略图，因为缩略图的 key、缓存、预热、URL 全部依赖稳定的 `asset/archive/task` 模型。

---

## 7. B1：地基冻结

当前状态：已完成

## 7.1 阶段目标

把当前“有目录、无实现”的 bootstrap 仓库，升级为可承载真实 Rust 后端开发的 workspace。

## 7.2 范围

### 本阶段要做

1. 建立根 `Cargo.toml` workspace
   - 完成情况：已完成
2. 把以下目录变成真实 crate：
   - `crates/shared-model`
   - `crates/app-core`
   - `crates/media-db`
   - `crates/media-io`
   - 完成情况：已完成
3. 在 `shared-model` 中冻结首批共享模型
   - 完成情况：已完成
4. 在 `packages/contracts` 中建立 TS/Zod 合同目录结构
   - 完成情况：已完成（当前已落 `src/errors`、`src/models`、`fixtures`、`tests`，`commands/channels/events` 仍可后补）
5. 建立开发期 CLI harness 骨架
   - 完成情况：已完成
6. 接入 `thiserror`、`tokio`、`tracing`、`tracing-subscriber`
   - 完成情况：已完成
7. 建立最小 fixture 与 benchmark 目录结构
   - 完成情况：已完成

### 本阶段不做

- 不实现真实扫描业务
- 不实现真实 zip 读取
- 不实现 Tauri command/channel/protocol 全接线
- 不实现缩略图 service

## 7.3 目录与文件计划

### Rust workspace

- 根目录新增 `Cargo.toml`
- `members` 建议包含：
  - `src-tauri`
  - `crates/shared-model`
  - `crates/app-core`
  - `crates/media-db`
  - `crates/media-io`
- 视情况补 `workspace.dependencies`，统一 `serde`、`thiserror`、`tracing`、`tokio`

### `crates/shared-model`

建议首批模块：

- `ids.rs`
- `errors.rs`
- `tasks.rs`
- `pagination.rs`
- `media.rs`
- `lib.rs`

首批必须冻结的类型：

- `LibraryId`
- `SourceId`
- `ArchiveId`
- `ArchiveEntryId`
- `AssetId`
- `ThumbnailKey`
- `TaskId`
- `PlaybackSessionId`
- `SubtitleSessionId`
- `AppErrorCode`
- `TaskState`
- `TaskProgress`

### `packages/contracts`

建议建立：

```text
packages/contracts/
  src/
    models/
    commands/
    channels/
    events/
    errors/
  fixtures/
  tests/
```

首批合同只冻结通用层，不急于冻结全部业务命令。

### CLI harness

首版建议放在 `src-tauri/src/bin/` 或 `crates/app-core/src/bin/`，但必须满足：

- 只作为开发入口
- 所有业务逻辑调用 `app-core`
- 不在 CLI 二进制中直接写数据库/扫描/zip 逻辑

若首版为减小改动继续放在 `src-tauri/src/bin/`，则后续业务实现仍必须沉到 `crates/*`。

## 7.4 交付物

- 可编译的 Rust workspace
- `shared-model` 初版
- `packages/contracts` 初版目录与首批模型
- `AppErrorCode` / `TaskState` / `TaskProgress` 首版
- CLI harness 骨架
- `docs/fixtures/` 与 `docs/benchmarks/` 目录

## 7.5 验收标准

- workspace 可 `cargo check --workspace`
- `shared-model` 与 `packages/contracts` 能通过基础序列化/校验测试
- 同一份示例 JSON 可通过 Zod parse 与 Rust 反序列化
- CLI harness 至少能打印帮助、读取基础配置、调用空 use case

## 7.6 本阶段必须补的测试

- `shared-model` serde round-trip tests
- `packages/contracts` zod parse tests
- JSON fixture snapshot tests
- CLI 启动 smoke test

## 7.7 本阶段验证命令

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo check --workspace --all-targets --locked
cargo test --workspace
```

---

## 8. B2：数据地基

当前状态：已完成

## 8.1 阶段目标

把 SQLite schema、migration、repository 做成可重复执行、可测试、可被后续扫描与 zip 直接复用的数据底座。

## 8.2 范围

### 本阶段要做

1. 在 `media-db` 中建立连接层、migration runner、repository 实现
   - 完成情况：已完成
2. 冻结首版 schema
   - 完成情况：已完成
3. 明确 repository ports 与实现分工
   - 完成情况：已完成
4. 建立数据库 smoke tests 与基础 query benchmarks
   - 完成情况：smoke tests 已完成，benchmark 仍待后补
5. 建立历史 fixture 升级测试框架
   - 完成情况：已完成

### 本阶段不做

- 不做复杂搜索与排序优化
- 不做播放器/字幕相关 schema
- 不做缩略图生成，仅预留 `thumbnails` 表

## 8.3 schema 拆分建议

为了降低首轮返工与 migration 风险，建议首版拆成以下 migration：

### `0001_init_core`

- `libraries`
- `sources`
- `media_assets`
- `tasks`
- 完成情况：已完成

### `0002_init_archive_and_thumb`

- `archives`
- `archive_entries`
- `thumbnails`
- 完成情况：已完成（当前以预留表结构为主，业务链路尚未启用）

说明：

- 不要把所有未来字段一次塞满
- 但字段命名要按最终稳定模型命名，不接受临时命名

## 8.4 repository 边界建议

### ports 所在层

repository traits 建议定义在 `app-core::ports`，例如：

- `LibraryRepository`
- `SourceRepository`
- `ArchiveRepository`
- `AssetRepository`
- `TaskRepository`
- `ThumbnailRepository`

### 实现所在层

在 `media-db` 中实现上述 traits，负责：

- 连接管理
- migration 执行
- SQL 语句与事务边界
- 行到 DTO 的映射

### 明确禁止

- command 层直接写 SQL
- 上层服务直接持有裸 `rusqlite::Connection`
- 通过“删库重建”绕过 migration

## 8.5 交付物

- `media-db` 真实 crate
- migration runner
- migration SQL 文件
- repository traits 与实现
- 空库初始化测试
- `N-1 -> N` 升级测试框架
- 基础 query benchmark 脚本

## 8.6 验收标准

- 能初始化新库
- 能重复执行 migration 而不报错
- 能在测试中插入 / 更新 / 查询 1000+ 样本记录
- 上层 service 不暴露 SQL 细节

## 8.7 本阶段必须补的测试

- migration smoke tests
- repository integration tests
- transaction rollback tests
- schema fixture upgrade tests

## 8.8 本阶段验证命令

```bash
cargo test -p media-db
cargo test --workspace
```

必要时补充：

```bash
cargo bench -p media-db
```

---

## 9. B3：扫描 / 入库最小闭环

当前状态：进行中

## 9.1 阶段目标

在无 UI 环境下，把“媒体库路径 -> source/archive/asset/task 落库”的最小主链路跑通。

## 9.2 范围

### 本阶段要做

1. 建立文件发现与路径归一化
   - 完成情况：已完成
2. 建立快速指纹规则
   - 完成情况：已完成首版（基于 path + size + mtime 的快速指纹）
3. 建立扩展名级媒体分类
   - 完成情况：已完成
4. 建立扫描 service 与入库 service
   - 完成情况：已完成首版
5. 建立扫描任务状态写入与读取
   - 完成情况：已完成首版，支持 running/completed/failed 查询语义
6. 建立 CLI 命令：
   - `scan add-library <path>`
   - `scan run <library-id>`
   - `scan resume <library-id>`
   - `scan stats <library-id>`
   - `scan diff <library-id>`
   - 完成情况：已完成
7. 输出 fixture 扫描报告
   - 完成情况：已完成首版 fixture 与 snapshot，对固定 smoke 样本可比对

### 本阶段不做

- 不做完整 `ffprobe` 深度元数据
- 不做 `rar/7z` 归一化
- 不做缩略图批量预热
- 不做复杂并发优化到极致

## 9.3 首版流水线切分

### Stage 1：发现文件

- 遍历 `library root`
- 跳过隐藏目录与不支持扩展名
- 产出候选 `source`
- 完成情况：已完成

### Stage 2：快速指纹

- `normalized_path`
- `size`
- `mtime_ms`
- 需要时补充轻量 hash
- 完成情况：已完成首版；当前未引入额外文件内容 hash

### Stage 3：媒体分类

- `image`
- `archive`
- `video`
- `audio`
- `other`
- 完成情况：已完成扩展名级分类

说明：B3 首版允许只对 `image/archive` 做完整主链路，`video/audio` 先完成分类与占位落库。

### Stage 4：首轮深度检查

- 普通图片：读取基础尺寸/方向（若代价可控）
- `zip`：只登记“待进一步解析”或调用 B4 的目录能力
- 完成情况：未完成，当前仍停留在扫描分类与落库前置阶段

### Stage 5：DB upsert

- 更新 `sources`
- 创建/更新 `media_assets`
- 创建/更新 `archives`（若当前已识别归档）
- 写入 `tasks`
- 完成情况：`sources` 与 `tasks` 已接通；`media_assets` / `archives` 的扫描侧联动仍待补全

### Stage 6：收尾

- 标记 `exists = false`
- 产出扫描统计
- 写入日志
- 完成情况：`exists = false`、统计、snapshot 已完成首版；更完整日志与失败恢复仍待补充

## 9.4 并发与事务策略

首版建议：

- 遍历：单生产者
- 解析：有限 worker
- DB：单写入器或显式批事务

原因：当前阶段优先稳定性与可回放性，不优先追求极限吞吐。

## 9.5 交付物

- `media-io` 的文件发现与分类能力
- `app-core` 扫描 use case
- CLI 扫描入口
- 扫描任务状态持久化
- `small-fixture` 扫描报告

## 9.6 验收标准

- 能对固定样本目录完成首轮入库
- 二次重扫能跳过绝大多数未变化文件
- 删除文件后能标记 `exists = false`
- 出错文件不会导致整轮扫描崩溃

## 9.7 本阶段必须补的测试

- 路径归一化测试
- 扩展名分类测试
- 首扫/重扫 integration tests
- tombstone 测试
- 扫描中断恢复测试（首版可做最小恢复语义）
- 完成情况：前四项已完成；扫描中断恢复已具备 `failed + resume` 最小语义，但仍未实现完整恢复策略

## 9.8 本阶段验证命令

```bash
cargo test -p media-io
cargo test -p app-core
cargo run --bin <scan-cli> -- scan run <library-id>
cargo run --bin <scan-cli> -- scan resume <library-id>
cargo run --bin <scan-cli> -- scan diff <library-id>
```

若 CLI 二进制仍位于 `src-tauri`，则命令按实际 bin 名替换。

## 9.9 扫描验证实施计划

`B3` 不应等到 `B4` 全部完成后，才第一次使用实际负载做扫描验证。

当前推荐策略是：

1. 在 `B3` 阶段先完成 **小到中等规模** 的真实负载扫描验证
2. 在 `B4` 进行中，再补 **包含 zip 主链路** 的实际负载验证
3. 在 `B4` 完成后，再做一次阶段性回归并固化 benchmark 基线

这样安排的原因是：

- `B3` 最容易暴露的是扫描模型、指纹策略、重扫语义、tombstone 语义、任务状态模型的问题
- 如果等到 `B4` 甚至 `B5` 才第一次跑真实负载，返工会同时牵连扫描、数据库、归档索引和缩略图依赖面
- 但在 `B4` 尚未开始前，也不适合过早把最终性能结论写死，因为 zip 主链路负载还未纳入

### `B3` 阶段当前必须做的扫描验证

#### 验证目标

- 验证首扫正确性
- 验证重扫时的 `skipped_unchanged` 是否可信
- 验证文件删除后的 `exists = false` / tombstone 是否正确
- 验证任务状态在成功、失败、resume 时是否可观测
- 验证扫描结果快照是否可复现、可比对
- 验证图片、音频、视频、归档在当前扫描阶段是否能被正确发现与分类

#### 验证方式

1. `small-fixture`
   - 用于本地快速 smoke、单元/集成测试、CLI 验证
2. `medium-fixture`
   - 用于中等规模扫描回归与首轮 benchmark
3. 一份本地真实目录（非提交到仓库）
   - 用于验证扫描结果与真实文件分布是否一致

说明：

- 当前扫描阶段的分类仍以后缀级判断为主，因此数据集必须覆盖最终目标媒体类型，避免后缀策略只围绕图片和 zip 调整
- `B3` 当前不要求所有样本都已接入深度解析，但必须至少能正确发现、分类、落库

#### 当前阶段应记录的指标

- 首扫发现文件数
- 首扫插入/更新数
- 重扫跳过数 `skipped_unchanged`
- 删除后 tombstone 数
- 成功任务数 / 失败任务数
- 首扫耗时
- 重扫耗时

#### 触发设计调整的条件

- 首扫发现数量与样本实际数量明显不符
- 重扫时未变化文件无法稳定跳过
- 删除文件后不能正确写入 `exists = false`
- 任务失败时没有稳定错误码或错误信息
- CLI 输出与快照内容无法稳定复现
- 图片、音频、视频、归档的分类结果与样本预期不一致
- 含中文、英文、日文、空格路径的样本在扫描中出现遗漏或归一化错误

出现以上情况时，应优先在 `B3` 调整以下内容，而不是带着问题进入 `B4`：

- `SourceRecord` 字段
- 快速指纹规则
- 扫描统计字段
- `TaskRecord` 失败语义
- `SourceRepository` / `TaskRepository` 接口设计

### `B4` 阶段再追加的扫描验证

在 `B4` 期间，扫描验证重点切换为：

- zip 文件在扫描阶段是否被稳定识别
- `archives` / `archive_entries` 是否与 `sources` 关系正确
- zip 索引与扫描主链路是否存在字段或状态冲突

此阶段如发现问题，优先调整：

- `archives` / `archive_entries` schema
- 扫描阶段归档落库策略
- zip 封面候选与页序策略

## 9.10 扫描负载数据集要求

扫描验证不能只依赖极小样本。当前阶段至少需要三层数据集。

### A. `small-fixture` 要求

用途：本地快速 smoke、测试回放、CLI 命令验证。

最低要求：

- 总文件数：10~50
- 至少包含：
  - 普通图片
  - 音频文件
  - 视频文件
  - zip / cbz
  - 无关文件（应被忽略）
  - 嵌套目录
- 至少覆盖：
  - 英文路径
  - 中文路径
  - 日文路径
  - 空格路径
  - 删除后可用于 tombstone 的样本

文件内容要求：

- 需要是真实存在的文件，不能只放文件名清单
- 在 `B3` 阶段允许部分样本使用轻量占位文件，只要文件真实存在且能稳定体现 `size/mtime/path` 变化即可
- 对 `small-fixture` 的多语言路径覆盖，允许复用同一份底层文件内容，仅通过不同目录和文件名覆盖中/英/日/空格路径；当前阶段重点是验证路径、扫描、分类和重扫语义，不要求每个语言样本都对应不同媒体内容
- 但至少应保留少量真实可打开的图片、音频、视频、zip/cbz 样本，为后续阶段过渡做准备

### B. `medium-fixture` 要求

用途：扫描回归、重扫验证、首轮 benchmark。

最低要求：

- 总文件数：300~1000
- 图片占比：40%~70%
- 音频占比：5%~20%
- 视频占比：5%~20%
- 归档占比：10%~30%（当前以 `zip/cbz` 为主）
- 无关文件：5%~15%
- 目录层级：至少 3 层
- 文件名分布：
  - 连续编号
  - 非连续编号
  - 带空格
  - 带中文
  - 带日文
  - 长文件名

必须能覆盖以下行为：

- 首轮全量扫描
- 第二轮无变更重扫
- 删除部分文件后重扫
- 新增部分文件后重扫

文件内容要求：

- 不能全部使用占位文件
- 至少一部分必须是真实可解析媒体文件，尤其要包括：
  - 真实图片
  - 真实音频
  - 真实视频
  - 真实 zip/cbz
- 目标是让 `B3` 阶段就能验证“扫描发现与分类”是否适配真实媒体分布，而不是只验证文件名和后缀规则

### C. 本地真实目录要求（不提交仓库）

用途：验证扫描模型是否适配真实使用环境。

最低要求：

- 文件规模：至少 1 个真实媒体目录，建议 500+ 文件
- 文件类型：至少包含图片、音频、视频与 zip/cbz，允许混合少量无关文件
- 路径特征：
  - 中文路径
  - 日文路径（若本机目录中存在）
  - 英文路径
  - 空格路径
  - 较深目录层级
- 变更场景：
  - 先扫描一次
  - 再删除/新增部分文件
  - 再执行重扫

说明：

- 本地真实目录只用于验证，不应提交仓库
- 若包含隐私或版权敏感内容，应只记录统计结果与结论，不记录原始路径清单

### 当前阶段对扫描负载数据集的落地要求

1. 仓库内必须至少维护：
   - `small-fixture`
   - `medium-fixture` 方案说明（可先文档化，后补样本）
   - 当前补充要求：允许先通过脚本生成 `generated-placeholder/` 占位数据集，用于提前建立路径分布、类型分布和扫描回归骨架；后续再用同类真实文件逐步替换
   - 当前进一步补充：当 `docs/fixtures/realdatasmall/` 与 `docs/fixtures/realdata/` 准备完成后，应通过一次性脚本把真实文件填充到三个基线目录；日常测试不直接修改基线目录，而是复制到 `data/scan-validation/runs/` 下的临时目录执行
2. 在 `B3` 结束前，至少完成：
   - `small-fixture` 全链路验证
   - 一轮 `medium-fixture` 扫描验证
   - 一轮本地真实目录扫描验证记录
3. 验证结果必须至少输出：
   - 样本规模
   - 类型分布（图片/音频/视频/归档/无关文件）
   - 首扫结果
   - 重扫结果
   - 发现的问题
   - 是否触发模型/字段/策略调整

---

## 10. B4：zip 与归档索引

当前状态：未开始

## 10.1 阶段目标

把 zip 高频主链路的“列目录、判图、排序、entry 读取、归档索引落库”做成稳定底层能力，为 B5 缩略图与未来 zip 浏览打地基。

## 10.2 范围

### 本阶段要做

1. 在 `media-io` 中建立 zip 目录读取能力
   - 完成情况：未开始
2. 建立图片页判断规则
   - 完成情况：未开始
3. 建立页序排序规则
   - 完成情况：未开始
4. 建立 zip entry stream/read API
   - 完成情况：未开始
5. 建立封面候选提取
   - 完成情况：未开始
6. 建立 `archive_entries` 落库
   - 完成情况：未开始
7. 建立归档页序 snapshot tests
   - 完成情况：未开始

### 本阶段不做

- 不做 zip 写回
- 不做 `rar/7z` 归一化
- 不做视频帧抽取
- 不做缩略图生成

## 10.3 `media-io` 建议模块

- `archive/mod.rs`
- `archive/zip_reader.rs`
- `archive/zip_index.rs`
- `archive/page_sort.rs`
- `archive/entry_stream.rs`

## 10.4 索引落库策略

对 zip 文件首版至少要写入：

- `archives`
  - `archive_type`
  - `page_count`
  - `cover_entry_id`
  - `status`
- `archive_entries`
  - `entry_path`
  - `entry_name`
  - `page_index`
  - `media_kind`
  - `compressed_size`
  - `uncompressed_size`
  - `crc32`

如果在当前阶段就能低成本拿到宽高，可一起写入；否则字段先允许为空。

## 10.5 交付物

- `media-io` zip 读取服务
- `app-core` archive index use case
- zip 页序 snapshot
- 归档 fixture 测试
- `archive_entries` 落库逻辑

## 10.6 验收标准

- 能稳定列出 zip 图片集条目
- 页序在固定 fixture 上稳定
- 能读取指定 entry 数据流
- 能把归档索引写入数据库并供后续查询

## 10.7 本阶段必须补的测试

- zip 目录读取 tests
- 页序排序 tests
- zip entry 读取 integration tests
- 损坏 zip / 空 zip / 边缘命名样本 tests
- 归档索引落库 tests

## 10.8 本阶段验证命令

```bash
cargo test -p media-io archive
cargo test --workspace
```

必要时补充：

```bash
cargo bench -p media-io
```

---

## 11. 阶段间依赖关系

### `B1 -> B2`

- 没有 `shared-model/contracts/errors/tasks`，数据库 schema 会反复改名

### `B2 -> B3`

- 没有稳定 schema/repository，扫描只能做成临时 insert 脚本

### `B3 -> B4`

- 没有 `sources/assets/tasks` 的基本落库，zip 索引无法稳定纳入主数据模型

### `B4 -> B5`

- 没有 zip entry stream 与 `archive_entries`，zip 内页缩略图无法按最终形态实现

因此当前阶段不建议跳过 `B2/B3` 直接开发缩略图。

---

## 12. fixture、golden、benchmark 计划

## 12.1 fixture 目录建议

```text
docs/fixtures/
  small-fixture/
  medium-fixture/
  archive-fixture/
```

### `small-fixture`

- 几十个普通图片
- 1~2 个 zip
- 用于单元/集成测试与本地调试
- 当前完成情况：已完成首版 `scan-smoke` 样本与快照；并补充脚本生成的 `generated-placeholder/` 占位数据集

### `medium-fixture`

- 几百到一千级文件
- 用于扫描回归和 benchmark
- 当前完成情况：已补方案说明，并补充脚本生成的 `generated-placeholder/` 占位数据集；后续再按验证结果逐步替换为真实文件

### `archive-fixture`

- 正常 zip
- 空 zip
- 损坏 zip
- 边缘命名 zip
- 后续补 rar/7z 样本
- 当前完成情况：未开始

## 12.2 当前阶段要产出的 golden

- `shared-model` / contracts JSON fixtures
- 扫描结果 snapshot
- zip 页序 snapshot

当前完成情况：

- `shared-model` / contracts JSON fixtures：已完成
- 扫描结果 snapshot：已完成首版（`small-fixture/scan-smoke.expected.json`）
- zip 页序 snapshot：未开始

说明：缩略图金图放到 `B5` 再正式建立。

## 12.3 benchmark 计划

`B1-B4` 至少记录以下基线：

- 首次扫描耗时
- 二次重扫耗时
- zip 目录读取耗时
- zip 连续 entry 读取耗时
- 关键 query 耗时

---

## 13. 与 `src-tauri` 的关系

在 `B1-B4` 阶段，`src-tauri` 不是主开发面。

允许的改动：

- 把 `app-core` 接到最小开发命令或 smoke 命令
- 维持现有 runtime checks
- 仅做必要依赖接线

不允许的改动：

- 把扫描、repository、zip 逻辑直接塞进 `src-tauri`
- 为了调试方便在 Tauri command 中直接读库/扫目录/读 zip
- 提前做复杂 event/channel/protocol 全接线

`src-tauri` 在当前阶段的职责是：

- 证明宿主可加载后端能力
- 不成为后端真实实现的承载层

---

## 14. 每阶段完成定义（Definition of Done）

## `B1` 完成定义

- workspace 建好
- 首批 crate 可编译
- 通用 ID / 错误码 / 任务模型冻结
- contracts 初版与 fixture 初版存在
- CLI harness 可运行

当前状态：已达成

## `B2` 完成定义

- migration 可执行
- repository 可用
- 空库初始化与升级测试通过
- 1000+ 样本记录 smoke test 通过

当前状态：已达成

## `B3` 完成定义

- 首扫/重扫/tombstone 路径可用
- 扫描任务状态可查询
- CLI 能输出基础统计
- fixture 扫描结果可快照比对

当前状态：大部分已达成；仍待补“更完整失败恢复”后可视为阶段完成

## `B4` 完成定义

- zip 条目可列出
- 页序稳定
- entry 可读取
- `archive_entries` 可落库
- 为 `B5` 提供稳定 zip entry 输入面

当前状态：未开始

---

## 15. 当前建议的实际执行顺序

如果从当前仓库立即开工，建议严格按以下顺序落地：

### 第 1 批：`B1-1` 到 `B1-3`

1. 建根 workspace
2. 把 `shared-model` 变成真实 crate
3. 把 `packages/contracts` 变成真实合同包

完成情况：已完成

### 第 2 批：`B1-4` 到 `B2`

1. 接入 `thiserror` / `tokio` / `tracing`
2. 建 `media-db`
3. 建 migration runner
4. 建 repository 首版

完成情况：已完成

### 第 3 批：`B3`

1. 建文件发现
2. 建分类与快速指纹
3. 建扫描 CLI
4. 跑通首轮入库

完成情况：已完成并继续扩展到重扫、tombstone、failed、resume、snapshot

### 第 4 批：`B4`

1. 建 zip 列目录
2. 建页序排序
3. 建 entry 读取
4. 建归档索引落库

完成情况：未开始

完成以上四批后，再单独编写 `B5-B8` 的具体实施计划或在新文档中补续篇。

---

## 16. 本文件对应的后续文档建议

当 `B1-B4` 接近完成时，新增下一份文档：

- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md`

该文档再覆盖：

- `B5` 缩略图主链路 Rust 化
- `B6` 归一化
- `B7` 播放后端适配
- `B8` sidecar 宿主协议

这样做的目的，是避免在地基阶段就把后续重能力细节一起写死。

---

## 17. 最终执行结论

当前仓库后端先行阶段的正确落地方式不是：

- 直接开始做缩略图
- 直接开始扩 `src-tauri`
- 直接开始 UI 接线

而是：

1. 先完成 `B1` 地基冻结
2. 再完成 `B2` 数据地基
3. 再完成 `B3` 路径到数据库闭环
4. 再完成 `B4` zip 与归档索引

完成这四步后，`B5` 缩略图 Rust 化才会真正具备稳定、可测试、可直接接 UI 的实现条件。
