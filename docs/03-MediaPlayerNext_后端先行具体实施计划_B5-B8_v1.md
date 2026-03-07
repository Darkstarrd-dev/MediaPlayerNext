# MediaPlayerNext 后端先行具体实施计划 B5-B8 v1

## 1. 文档定位

本文件不是替代 `docs/00-MediaPlayerNext_实施计划_v2.md`，而是在 `docs/02-MediaPlayerNext_后端先行具体实施计划_B1-B4_v1.md` 已完成首版收口后，把后端先行阶段的后四个里程碑继续拆成可直接执行的实施清单。

四份文档的职责固定如下：

- `docs/00-MediaPlayerNext_实施计划_v2.md`
  - 负责总路线、工作包边界、接口策略与总体顺序
- `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
  - 负责质量门禁、测试要求、迁移流程、benchmark 与发布约束
- `docs/02-MediaPlayerNext_后端先行具体实施计划_B1-B4_v1.md`
  - 负责 `B1-B4` 的数据地基、扫描闭环与 zip 主链路
- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md`
  - 负责 `B5-B8` 的缩略图、归一化、播放后端与字幕 sidecar 宿主协议

本文件只覆盖 **后端先行阶段的后四个里程碑**：

- `B5`：Rust 缩略图主链路完成（普通图片 + zip 内页）
- `B6`：`rar/7z -> zip` 归一化完成
- `B7`：`ffprobe / ffmpeg / mpv` 适配完成，并补齐媒体定位与协议输入面
- `B8`：subtitle sidecar host 协议完成

不覆盖：

- UI 对接与页面迁移
- theme 系统迁入
- 复杂播放器 UI 与字幕 UI 联调

这样拆的原因是：`B1-B4` 已把数据库、扫描与 zip 主链路做成了可复用地基，接下来真正阻塞 UI 的重能力，已经转为缩略图、归一化、播放后端与 sidecar 宿主协议。

---

## 2. 已完成内容与当前进度

截至目前，`B1-B4` 已完成首版收口；`B5` 已进入 `B5-0` 资产输入面阶段，`B6-B8` 仍未开始，但其关键前置条件已具备。

### 2.1 已完成内容（承接 `B1-B4`）

- Rust workspace、共享模型、错误码、任务状态、CLI harness 已稳定
- SQLite schema、migration、repository 已稳定可测
- `scan add-library/run/resume/stats/diff` 已跑通首版链路
- 真实样本扫描验证、local 大变更验证、归档索引验证已建立记录
- zip 目录读取、页序排序、entry 读取、`archive_entries` 落库与 `archive read-entry` 已具备首版能力
- `small-fixture`、`medium-fixture`、`archive-fixture` 与 `runs` 临时复制工作流已建立

### 2.2 当前进度判断

- `B5`：进行中（`B5-0` 已接通资产输入面；`B5-1` 已完成 `media-thumb` crate 首版；`B5-2` 已补 `thumbnail.ensure/show`、`thumbnails` 表写回与 `thumb://` 协议；当前只剩 thumbnail golden 细化与真实样本 benchmark）
- `B6`：未开始（未接入 `7z` 运行时、未实现归一化流程，当前仅完成 zip 主链路）
- `B7`：未开始（`crates/media-playback` 仍是目录占位，`ffprobe`/`ffmpeg`/`mpv` 只停留在运行时校验层）
- `B8`：未开始（`apps/subtitle-sidecar` 仍是 placeholder，尚无真实 stdio 协议与宿主管理器）

### 2.3 当前最自然的下一步

1. 补 thumbnail golden JSON，固定 profile / cache layout / protocol 期望
2. 再基于真实样本补一轮 thumbnail benchmark 记录

---

## 3. 当前仓库基线

截至当前更新时，仓库实际状态如下：

- `crates/shared-model`、`crates/app-core`、`crates/media-db`、`crates/media-io` 已为真实 crate
- `crates/media-thumb` 已是 workspace 内真实 crate；`crates/media-playback` 仍只有 `README.md`
- `packages/contracts` 当前只落了 `errors` 与 `models` 首版；`commands/channels/events` 仍待补齐
- `apps/subtitle-sidecar` 当前只有 `sharp` 验证脚本与 placeholder 入口，不具备 sidecar 协议能力
- `config/local.paths.json` 当前仅记录 `ffmpeg`、`ffprobe`、`mpv` 与旧仓路径；尚无 `7z` 本地 override
- `src-tauri` 仍保持极薄，仅承载开发期 `backend_harness` 与 runtime smoke，不承载真实业务实现
- `docs/benchmarks/scan-validation-20260307.md` 与 `docs/benchmarks/archive-validation-20260307.md` 已形成首轮真实样本基线

因此接下来的首要目标，不再是“再补一个后端骨架”，而是把 `B4` 已经产出的 zip 与资产前置输入，继续推进成缩略图、归一化、播放与 sidecar 的稳定服务层。

---

## 4. 执行原则

### 4.1 总原则

1. **先资产化，后派生**
   - 先把 `source/archive_entry -> media_assets` 接稳，再接缩略图、协议与播放能力
2. **先 crate，后协议/宿主**
   - 先把逻辑沉到纯 Rust crate，再由 `src-tauri` 暴露命令、channel、custom protocol 或 sidecar 管理
3. **先文件/归档图片，后视频/rar/7z 边角**
   - 优先完成最常用、最可验证的图片与 zip 主链路
4. **先可缓存、可验证、可回放，后追求极限性能**
   - 当前阶段优先稳定缓存键、任务状态、fixture、golden 与 benchmark 基线
5. **先 URL 与协议设计，后 UI 对接**
   - 不让前端以后继续通过 command 取大块字节，也不让 UI 自己拼路径或 Blob

### 4.2 当前阶段禁止事项

- 不把缩略图、归一化、播放或 sidecar 业务逻辑直接堆进 `src-tauri`
- 不继续把 `sharp` 保留为长期主链路
- 不把 `7z.exe` 硬绑定到旧仓绝对路径
- 不在 `B5` 阶段提前做完整视频播放 UI 或字幕 UI
- 不为了“未来可能需要”而一次性扩写全部 `B5-B8` 数据字段

### 4.3 对 `B3/B4` 的继承要求

`B5-B8` 默认继承并复用以下既有基础：

- `B3` 的真实样本扫描验证工作流
- `B4` 的 zip 目录/页序/entry 读取/归档索引链路
- `docs/fixtures/archive-fixture/` 的归档 golden
- `data/scan-validation/runs/` 的临时测试目录工作流

说明：`B3` 的完整失败恢复仍未完全实现，但这不阻塞 `B5-B8` 开始；后续如在长任务链路上暴露问题，再按阶段回补。

---

## 5. 目标目录与职责落位

在 `B5-B8` 内，目录职责固定如下：

```text
MediaPlayerNext/
  Cargo.toml                       # B5/B7 时补充 workspace members
  crates/
    app-core/                      # use case 编排、ports、命令与协议解析入口
    media-io/                      # 文件系统、zip、rar/7z 归一化、媒体定位底层
    media-db/                      # SQLite、migration、repository
    media-thumb/                   # B5 起变成真实 crate：缩略图、缓存、profile、pipeline
    media-playback/                # B7 起变成真实 crate：ffprobe/ffmpeg/mpv 适配
    shared-model/                  # DTO、ID、错误码、任务状态、协议输入输出模型
  apps/
    subtitle-sidecar/              # B8 起补 stdio JSON 协议与最小 host 对话能力
  packages/
    contracts/                     # B5-B8 补 commands/channels/events 合同
  src-tauri/                       # B5-B8 起承担最小 protocol 注册与 sidecar 管理接线
  docs/
    fixtures/                      # 补 thumbnail / archive / playback / sidecar fixture 说明
    benchmarks/                    # 补 thumbnail / normalize / playback / sidecar benchmark 记录
```

说明：

- `media-thumb` 必须在 `B5` 开始时变成真实 crate，并加入 root workspace
- `media-playback` 必须在 `B7` 开始时变成真实 crate，并加入 root workspace
- `apps/subtitle-sidecar` 仍保留 Node 实现，但 `B8` 要把它从 placeholder 推进到可被 Rust 宿主管理的最小 sidecar
- `src-tauri` 在 `B5-B8` 会比 `B1-B4` 多一点集成职责，但仍必须保持“薄宿主”原则

---

## 6. 阶段拆分总览

| 阶段 | 目标 | 核心交付物 | 是否阻塞后续 |
|---|---|---|---|
| `B5` | 缩略图主链路 Rust 化 | `media-thumb`、资产输入面、缩略图缓存、`thumb://` | 是 |
| `B6` | `rar/7z -> zip` 归一化 | `7z` wrapper、归一化任务、归一化后复用 `B4` | 是 |
| `B7` | 播放后端与媒体协议 | `media-playback`、`ffprobe/ffmpeg/mpv` adapter、`media://` / `archive://` | 是 |
| `B8` | sidecar 宿主协议 | subtitle host wrapper、stdio 协议、heartbeat / restart | 否（但阻塞字幕 UI） |

推荐顺序仍以串行为主：`B5 -> B6 -> B7 -> B8`。

说明：

- `B8` 的消息协议可以提前草拟，但真实宿主接线建议放在 `B7` 之后
- `B6` 与 `B7` 都依赖 `B5` 先把资产与缩略图输入面稳定下来

---

## 7. B5：缩略图主链路 Rust 化

当前状态：进行中（已完成 `B5-0`、`B5-1` 与 `B5-2` 核心链路，主要缺文档基线）

## 7.1 阶段目标

把普通图片与 zip 内页缩略图主链路从 Node/sharp 收回 Rust，并建立稳定缓存与 `thumb://` 输入面，为后续 UI 列表、网格、详情页直接消费做准备。

## 7.2 范围

### 本阶段要做

1. 把 `crates/media-thumb` 变成真实 crate，并加入 workspace
2. 在 `shared-model` 与 `packages/contracts` 中冻结首批缩略图相关模型
3. 先把 `file source` 与 `archive_entry` 资产化到 `media_assets`
4. 在 `media-thumb` 中建立普通图片缩略图能力
5. 在 `media-thumb` 中建立 zip entry 缩略图能力
6. 建立缩略图 profile、cache key、磁盘落盘与 DB 记录
7. 建立 `thumbnail.ensure` / `thumbnail.show` 等开发期命令
8. 建立 `thumb://cache/<thumbnail_key>` 的协议输入面
9. 建立缩略图 fixture / golden / benchmark 基线

### 本阶段不做

- 不做视频首帧缩略图（放到 `B7`）
- 不做 `rar/7z` 归一化（放到 `B6`）
- 不做复杂预热调度与批量任务并发优化到极致
- 不把缩略图字节通过 JSON IPC 返回给前端

## 7.3 先行前置任务：资产输入面接通

`B5` 不是从缩略图编码函数直接开始，而是先完成下面这条链路：

1. 普通图片 `SourceRecord(kind=image)` 能映射到 `MediaAssetRecord(source_kind=file)`
2. `ArchiveEntryRecord` 能映射到 `MediaAssetRecord(source_kind=archive_entry)`
3. `app-core` 能稳定按 `asset_id` 解析到：
   - 普通文件路径
   - zip entry 路径

说明：如果没有这层资产输入，缩略图缓存 key、URL、后续 UI 对接都会失稳。

### 7.3.1 当前已完成的 `B5-0`

- `app-core` 已新增 `asset` 模块，补上：
  - `asset.ensure <library-id>`
  - `asset.resolve <asset-id>`
- 普通图片 `SourceRecord(kind=image)` 现在可稳定写成 `MediaAssetRecord(source_kind=file)`
- `ArchiveEntryRecord(media_kind=image)` 现在可稳定写成 `MediaAssetRecord(source_kind=archive_entry)`
- `app-core` 现在可按 `asset_id` 解析出：
  - 普通文件绝对路径
  - zip 归档路径 + entry 路径
- `media-db` 已补齐 `AssetRepository::get`、`ArchiveRepository::get`、`ArchiveEntryRepository::get`
- 当前还未开始：
  - `thumb://` 协议
  - 缩略图 fixture / golden / benchmark 文档

### 7.3.2 当前已完成的 `B5-1/B5-2` 首版接线

- `crates/media-thumb` 已加入 workspace，并可直接处理：
  - 普通文件图片缩略图
  - zip 内页图片缩略图
- `app-core` 已新增 `thumbnail` 模块，补上：
  - `thumbnail.ensure <asset-id> <profile>`
  - `thumbnail.show <thumbnail-key>`
- `media-db` 已补齐 `ThumbnailRepository::get`
- `thumbnail.ensure` 当前已完成：
  - `asset_id -> ThumbnailSource` 解析
  - `media-thumb` 调用
  - `thumbnails` 表写回
  - cache hit / miss 结果返回
- `src-tauri` 当前已完成：
  - `thumb://cache/<thumbnail_key>` 协议首版
  - 按 `thumbnail_key` 读取 DB 记录并返回实际图片字节
- 当前还未完成：
  - thumbnail golden JSON
  - 基于真实样本的 benchmark 记录

## 7.4 模块与文件计划

### `crates/media-thumb`

建议最小模块：

- `src/lib.rs`
- `src/profiles.rs`
- `src/source.rs`
- `src/cache.rs`
- `src/pipeline.rs`
- `src/service.rs`

职责建议：

- `profiles.rs`
  - 定义 `grid-sm` / `grid-md` / `detail-md` / `detail-lg`
- `source.rs`
  - 抽象 `ThumbnailSource::FilePath | ArchiveEntry`
- `cache.rs`
  - 生成 `thumbnail_key`、磁盘路径与原子写入流程
- `pipeline.rs`
  - 解码、方向修正、resize、编码
- `service.rs`
  - 负责编排 `asset -> source -> thumbnail record`

当前首版已落地：

- `src/profiles.rs`
  - 已定义 `grid-sm` / `grid-md` / `detail-md` / `detail-lg`
- `src/source.rs`
  - 已定义 `ThumbnailSource::FilePath | ArchiveEntry`
- `src/cache.rs`
  - 已实现 `thumbnail_key`、两级磁盘布局与原子写入
- `src/pipeline.rs`
  - 已实现普通图片与 zip entry 图片的最小缩略图生成与 lossless webp 编码
- `src/service.rs`
  - 已实现 `ensure_thumbnail`，支持文件图与 zip 内页图，且具备 cache hit / miss 语义

当前 `app-core` 已补首版编排：

- `thumbnail.ensure(asset_id, profile)`
  - 已接到 `media-thumb` 并会回写 `thumbnails` 表
- `thumbnail.get(thumbnail_key)`
  - 已可查询现有缩略图记录

### `shared-model` / `packages/contracts`

建议补首批模型：

- `ThumbnailProfile`
- `ThumbnailState`
- `ThumbnailSummary`
- `ThumbnailEnsureRequest`
- `ThumbnailEnsureResult`
- `ThumbnailProgressEvent`

### `app-core`

建议新增：

- `thumbnail.ensure(asset_id, profile)`
- `thumbnail.get(thumbnail_key)`
- `asset.resolve(asset_id)`
- `asset.ensure_for_library(library_id)`（可选，开发期命令优先）

## 7.5 数据与缓存策略

首版建议：

- 缩略图主键：`thumbnail_key = hash(source_identity + source_revision + profile + pipeline_version)`
- `source_identity`：
  - 普通图片：`source_id`
  - 归档页：`archive_entry_id`
- `source_revision`：
  - 普通图片：`fingerprint` 或 `mtime/size`
  - 归档页：优先使用 `archive_id + entry_path + crc32 + uncompressed_size`
- 磁盘布局：

```text
data/cache/thumbs/
  ab/
    cd/
      <thumbnail_key>.webp
```

- 落盘要求：
  - 先写临时文件
  - 成功后原子 rename
  - 再写 `thumbnails` 表

首版尽量复用现有 `thumbnails` 表；若后续发现 `pipeline_version`、失败重试信息或失效策略无法稳定表达，再单独补 migration。

## 7.6 交付物

- `media-thumb` 真实 crate
- 普通图片缩略图主链路
- zip 内页缩略图主链路
- `thumbnails` 落库逻辑
- `thumb://` 协议输入面
- 缩略图 golden / benchmark 记录

## 7.7 验收标准

- 能对普通图片生成稳定缩略图
- 能对 zip 内页生成稳定缩略图
- 二次生成能稳定命中磁盘缓存
- 缩略图 URL 可直接被未来 `<img>` 消费
- Rust 输出在尺寸、方向、裁切上达到可接受一致性

## 7.8 本阶段必须补的测试

- 普通图片缩略图生成 tests
- zip entry 缩略图生成 tests
- cache key 稳定性 tests
- cache hit / miss tests
- `thumbnails` repository integration tests
- `thumb://` 协议 smoke tests

## 7.9 本阶段验证命令

```bash
cargo test -p media-thumb
cargo test --workspace
cargo run --bin backend_harness -- thumbnail ensure <asset-id> <profile>
cargo run --bin backend_harness -- thumbnail show <thumbnail-key>
```

## 7.10 本阶段涉及文件与目录

优先会涉及：

- `Cargo.toml`
- `crates/media-thumb/`
- `crates/shared-model/src/media.rs`
- `crates/shared-model/src/records.rs`
- `crates/app-core/src/lib.rs`
- `crates/app-core/src/ports.rs`
- `crates/app-core/src/archive.rs`
- `crates/media-db/src/repositories.rs`
- `crates/media-db/tests/database_integration.rs`
- `packages/contracts/src/index.ts`
- `packages/contracts/src/models/`
- `packages/contracts/src/commands/`
- `packages/contracts/src/channels/`
- `src-tauri/src/bin/backend_harness.rs`
- `src-tauri/src/main.rs`
- `docs/fixtures/thumbnail-fixture/`
- `docs/benchmarks/`

按子阶段可再细分：

- `B5-0` 资产输入面：`shared-model`、`app-core`、`media-db`
- `B5-1` 缩略图 crate：`Cargo.toml`、`crates/media-thumb/`
- `B5-2` 协议与命令：`packages/contracts`、`src-tauri`、`backend_harness`
- `B5-3` fixture/golden/benchmark：`docs/fixtures/thumbnail-fixture/`、`docs/benchmarks/`

## 7.11 本阶段完成后必须更新的 check 项

实现完成后，至少同步更新以下检查项：

- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md`
  - `B5` 当前状态
  - `B5` 完成情况
  - `B5` 完成定义
- `docs/logs/<当天日期>.md`
- `docs/fixtures/README.md`
- `docs/fixtures/thumbnail-fixture/README.md`
- `docs/benchmarks/` 下的缩略图验证/benchmark 记录
- 若新增 contracts：
  - `packages/contracts/src/index.ts`
  - 对应 `tests` / `fixtures`
- 若新增 runtime / protocol：
  - `package.json`、`src-tauri/Cargo.toml`、相关 smoke/check 命令

## 7.12 用于新对话启动的最小提示

当后续要直接开始 `B5` 开发时，可在新对话中只给下面这段提示，避免一次加载过多上下文：

```text
请按 `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md` 的 `7. B5` 章节开始开发，只聚焦 B5。

先读取：
- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md` 中 `7. B5`
- `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
- `Cargo.toml`
- `crates/shared-model/src/records.rs`
- `crates/app-core/src/archive.rs`
- `crates/app-core/src/ports.rs`
- `crates/media-db/src/repositories.rs`
- `crates/media-thumb/README.md`

工作目标：
- 先实现 `B5-0` 的 `media_assets <- source/archive_entry` 资产输入面
- 再把 `crates/media-thumb` 变成真实 crate
- 改动完成后更新 `docs/03...`、当天日志、fixture/benchmark 文档

约束：
- 保持 `src-tauri` 极薄
- 不做 UI 迁移
- 不做视频首帧与 `rar/7z`
```

---

## 8. B6：`rar/7z -> zip` 归一化

当前状态：已完成（当前阶段要求的真实 crate、缩略图生成、`thumbnails` 表、磁盘缓存、`thumb://`、fixture / golden / validation 文档已具备）

## 8.1 阶段目标

把 `rar/7z` 纳入统一“低频归一化、后续复用 zip 主链路”的路线，让归档浏览与缩略图不需要直接承受 `rar/7z` 高频读取复杂度。

## 8.2 范围

### 本阶段要做

1. 在 `media-io` 中建立 `7z` 外部工具包装器
2. 冻结归一化任务与状态模型
3. 为 `rar/7z` 建立归一化输出 zip 的磁盘布局
4. 记录 `source archive -> normalized zip` 关系
5. 归一化成功后直接复用 `B4` 的 zip 索引链路
6. 建立失败可重试、可清理、可重新验证的任务语义
7. 建立 `archive.normalize` / `archive.normalize-status` 开发期命令
8. 补 `archive-fixture` 中的 rar / 7z / 损坏包样本说明与验证记录

### 本阶段不做

- 不做归一化后的写回原包
- 不做多归一化后端并存
- 不做生产环境完整打包资源策略收尾（只先把本地开发与宿主路径方案定稳）

## 8.3 工具与路径策略

当前仓库尚未为 `7z` 建立正式路径策略，因此 `B6` 必须同时处理：

1. 开发期 override：
   - 在 `config/local.paths.json` 中新增 `sevenz` 或等效字段
2. 宿主层统一入口：
   - 不允许继续依赖旧仓绝对路径
3. 后续生产路径：
   - 优先走应用资源目录；本文件阶段只需把接口与查找顺序定下来

## 8.4 建议的数据策略

首版优先复用现有 `archives` 表：

- `archive_type = rar / 7z`
- `normalized_zip_path` 记录归一化输出路径
- `status` 表达：
  - `queued`
  - `extracting`
  - `packing_zip`
  - `verifying`
  - `normalized`
  - `failed`

若首版实现后发现仅靠 `normalized_zip_path + status` 无法稳定表达 cleanup/retry/version，再单独补 migration 扩展字段。

## 8.5 模块与文件计划

建议在 `media-io` 中新增：

- `src/normalize/mod.rs`
- `src/normalize/sevenz.rs`
- `src/normalize/layout.rs`
- `src/normalize/service.rs`

`app-core` 建议新增：

- `archive.normalize(source_id)`
- `archive.normalize_status(task_id)`
- `archive.reindex_normalized(source_id)`

`packages/contracts` 建议补：

- `commands/archive.ts`
- `channels/normalize-progress.ts`
- `events/archive-events.ts`

## 8.6 交付物

- `7z` wrapper
- 归一化 service
- 归一化任务状态模型
- 归一化后接回 `B4` 索引链路
- rar / 7z fixture 与失败样本验证记录

## 8.7 验收标准

- 能把 `rar/7z` 稳定归一化成内部 zip
- 归一化成功后可继续进入 `archive index`
- 归一化失败能保留错误记录并支持重试
- 归一化产物可按策略清理，不污染工作目录

## 8.8 本阶段必须补的测试

- `7z` wrapper 参数与错误解析 tests
- 归一化输出布局 tests
- rar/7z -> zip integration tests
- 失败重试 tests
- 损坏包 / 密码包 / 边缘命名样本 tests（可分层实现）

## 8.9 本阶段验证命令

```bash
cargo test -p media-io normalize
cargo test --workspace
cargo run --bin backend_harness -- archive normalize <source-id>
cargo run --bin backend_harness -- archive normalize-status <task-id>
```

## 8.10 本阶段涉及文件与目录

优先会涉及：

- `config/local.paths.json`
- `crates/media-io/src/lib.rs`
- `crates/media-io/src/normalize/`
- `crates/app-core/src/lib.rs`
- `crates/app-core/src/archive.rs`
- `crates/app-core/src/ports.rs`
- `crates/media-db/src/migrations/`
- `crates/media-db/src/repositories.rs`
- `crates/media-db/tests/database_integration.rs`
- `packages/contracts/src/commands/archive.ts`
- `packages/contracts/src/channels/normalize-progress.ts`
- `packages/contracts/src/events/archive-events.ts`
- `src-tauri/src/bin/backend_harness.rs`
- `docs/fixtures/archive-fixture/`
- `docs/benchmarks/`

按子阶段可再细分：

- `B6-0` 工具路径与 wrapper：`config/local.paths.json`、`media-io/src/normalize/sevenz.rs`
- `B6-1` 任务与状态：`shared-model`、`contracts`、`app-core`
- `B6-2` 落库与重索引：`media-db`、`app-core/archive.rs`
- `B6-3` fixture/benchmark：`docs/fixtures/archive-fixture/`、`docs/benchmarks/`

## 8.11 本阶段完成后必须更新的 check 项

实现完成后，至少同步更新以下检查项：

- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md`
  - `B6` 当前状态
  - `B6` 完成情况
  - `B6` 完成定义
- `docs/logs/<当天日期>.md`
- `config/local.paths.json` 的 `7z`/`sevenz` 说明
- `docs/fixtures/archive-fixture/README.md`
- `docs/benchmarks/` 下的归一化验证/benchmark 记录
- 若新增 schema 字段或 migration：
  - `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md` 所要求的 migration fixture / upgrade tests

## 8.12 用于新对话启动的最小提示

当后续要直接开始 `B6` 开发时，可在新对话中只给下面这段提示：

```text
请按 `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md` 的 `8. B6` 章节开始开发，只聚焦 B6。

先读取：
- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md` 中 `8. B6`
- `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
- `config/local.paths.json`
- `crates/media-io/src/lib.rs`
- `crates/app-core/src/archive.rs`
- `crates/media-db/src/migrations/0002_init_archive_and_thumb.sql`
- `docs/fixtures/archive-fixture/README.md`

工作目标：
- 建立 `7z` wrapper 与 `rar/7z -> zip` 归一化链路
- 复用 `B4` 的 archive index 做归一化后重索引
- 改动完成后更新 `docs/03...`、当天日志、archive fixture/benchmark 文档

约束：
- 不依赖旧仓固定绝对路径
- 不做生产打包收尾
- 不改 UI
```

---

## 9. B7：播放后端适配与媒体协议输入面

当前状态：未开始

## 9.1 阶段目标

把 `ffprobe`、`ffmpeg`、`mpv` 做成稳定的后端服务层，并补齐 `media://` / `archive://` 协议输入面，使未来 UI 可以直接消费 URL，而不是通过 command 搬运字节。

## 9.2 范围

### 本阶段要做

1. 把 `crates/media-playback` 变成真实 crate，并加入 workspace
2. 建立 `ffprobe` adapter，回填 `media_assets` 的基础元数据
3. 建立 `ffmpeg` wrapper，用于视频首帧、抽帧与后续进度解析
4. 建立 `mpv` session manager，先支持最小会话控制
5. 建立 `media://asset/<asset_id>` 协议输入面
6. 建立 `archive://entry/<archive_entry_id>` 协议输入面
7. 补 `playback.*` / `items.*` / `events.*` 合同
8. 更新 runtime smoke，把 `ffprobe` 纳入固定校验

### 本阶段不做

- 不做完整播放器 UI
- 不做复杂播放列表、快捷键和多窗口控制
- 不做复杂转码编排平台

## 9.3 模块与文件计划

### `crates/media-playback`

建议最小模块：

- `src/lib.rs`
- `src/ffprobe.rs`
- `src/ffmpeg.rs`
- `src/mpv.rs`
- `src/session.rs`

### `shared-model` / `packages/contracts`

建议补：

- `MediaProbeSummary`
- `PlaybackSessionSummary`
- `PlaybackCommandRequest`
- `FfmpegProgressEvent`
- `MediaUrlSummary`

### `app-core`

建议新增：

- `playback.probe(asset_id)`
- `playback.open(asset_id)`
- `playback.status(session_id)`
- `playback.pause(session_id)` / `playback.seek(session_id)` / `playback.stop(session_id)`
- `media.resolve(asset_id)`
- `archive.resolve_entry(entry_id)`

## 9.4 协议面策略

`B7` 必须把下面三类入口分清：

1. `command`
   - 返回元数据、状态、URL、session id
2. `channel`
   - 返回高频进度（`ffmpeg`、长任务日志）
3. `custom protocol`
   - 返回实际媒体字节

推荐协议：

- `thumb://cache/<thumbnail_key>`（承接 `B5`）
- `media://asset/<asset_id>`
- `archive://entry/<archive_entry_id>`

要求：

- 协议 handler 只做参数解析与响应包装
- 实际寻址与权限判断放在 `app-core`
- 后续 UI 不再直接依赖磁盘路径

## 9.5 交付物

- `media-playback` 真实 crate
- `ffprobe` / `ffmpeg` / `mpv` adapter
- `media://` / `archive://` 协议输入面
- 播放会话 DTO 与最小命令面
- 视频首帧缩略图输入面（供后续扩展）

## 9.6 验收标准

- 能读取基础媒体元数据并更新 `media_assets`
- 能执行最小抽帧任务并返回进度
- 能建立本地 `mpv` 会话并执行基本控制
- `<img>` / `<video>` 未来可以直接消费 `thumb://` / `media://` / `archive://`

## 9.7 本阶段必须补的测试

- `ffprobe` 输出解析 tests
- `ffmpeg` 进度解析 tests
- `mpv` 会话 smoke tests
- `media://` / `archive://` 协议 smoke tests
- `media_assets` 元数据回填 tests

## 9.8 本阶段验证命令

```bash
cargo test -p media-playback
cargo test --workspace
cargo run --bin backend_harness -- playback probe <asset-id>
cargo run --bin backend_harness -- playback open <asset-id>
cargo run --bin backend_harness -- playback status <session-id>
```

## 9.9 本阶段涉及文件与目录

优先会涉及：

- `Cargo.toml`
- `config/local.paths.json`
- `src-tauri/src/runtime_check.rs`
- `scripts/check-runtimes.ps1`
- `crates/media-playback/`
- `crates/shared-model/src/media.rs`
- `crates/shared-model/src/records.rs`
- `crates/app-core/src/lib.rs`
- `crates/app-core/src/ports.rs`
- `crates/media-db/src/repositories.rs`
- `packages/contracts/src/commands/`
- `packages/contracts/src/channels/`
- `packages/contracts/src/events/`
- `src-tauri/src/main.rs`
- `src-tauri/src/bin/backend_harness.rs`
- `docs/fixtures/playback-fixture/`
- `docs/benchmarks/`

按子阶段可再细分：

- `B7-0` crate 启动：`Cargo.toml`、`crates/media-playback/`
- `B7-1` ffprobe/ffmpeg：`media-playback/src/ffprobe.rs`、`media-playback/src/ffmpeg.rs`
- `B7-2` mpv：`media-playback/src/mpv.rs`、`media-playback/src/session.rs`
- `B7-3` 协议：`src-tauri`、`app-core`、`contracts`

## 9.10 本阶段完成后必须更新的 check 项

实现完成后，至少同步更新以下检查项：

- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md`
  - `B7` 当前状态
  - `B7` 完成情况
  - `B7` 完成定义
- `docs/logs/<当天日期>.md`
- `src-tauri/src/runtime_check.rs` 与 `scripts/check-runtimes.ps1`
- `docs/fixtures/playback-fixture/README.md`
- `docs/benchmarks/` 下的 playback / ffprobe / ffmpeg / mpv 验证记录
- 若新增协议：
  - `packages/contracts/src/index.ts`
  - `commands/channels/events` 导出与测试

## 9.11 用于新对话启动的最小提示

当后续要直接开始 `B7` 开发时，可在新对话中只给下面这段提示：

```text
请按 `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md` 的 `9. B7` 章节开始开发，只聚焦 B7。

先读取：
- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md` 中 `9. B7`
- `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
- `config/local.paths.json`
- `src-tauri/src/runtime_check.rs`
- `scripts/check-runtimes.ps1`
- `crates/media-playback/README.md`
- `crates/shared-model/src/records.rs`
- `packages/contracts/src/index.ts`

工作目标：
- 把 `crates/media-playback` 变成真实 crate
- 建 `ffprobe/ffmpeg/mpv` 最小 adapter
- 建 `media://` / `archive://` 输入面
- 改动完成后更新 `docs/03...`、当天日志、playback fixture/benchmark 文档

约束：
- 不做播放器 UI
- 不在 `src-tauri` 写业务逻辑
- 不把媒体大字节重新塞进 JSON IPC
```

---

## 10. B8：subtitle sidecar 宿主协议

当前状态：未开始

## 10.1 阶段目标

在不迁移真实字幕业务本体的前提下，先把 Rust 宿主如何启动、探活、重启、管理 Node sidecar 的协议固定下来，避免以后 UI 接回时再边写宿主边定协议。

## 10.2 范围

### 本阶段要做

1. 把 `apps/subtitle-sidecar` 从 placeholder 推进为最小 stdio JSON 服务
2. 冻结 sidecar 消息模型与消息 framing 规则
3. 在 `app-core` 中定义 sidecar host ports 与 use case
4. 在 `src-tauri` 中建立最小 sidecar host wrapper
5. 建立心跳、异常退出、重启与日志收集语义
6. 建立 `subtitle.ping` / `subtitle.health` / `subtitle.start-session` 等开发期命令
7. 补 sidecar 协议 fixture、转录样本与日志记录

### 本阶段不做

- 不迁移真实字幕识别 / 对齐 / 导出业务实现
- 不做复杂会话编排 UI
- 不做高级 transport（named pipe / websocket）

## 10.3 协议策略

首版固定：

- transport：`stdio`
- 消息体：JSON
- framing：一行一个 JSON message（newline-delimited JSON）

首批必须冻结的消息：

- `ping`
- `health`
- `start_session`
- `stop_session`
- `get_progress`
- `export_srt`
- `shutdown`

建议在 `packages/contracts` 中增加：

- `commands/subtitle.ts`
- `channels/subtitle-progress.ts`
- `events/subtitle-events.ts`

## 10.4 模块与文件计划

### `apps/subtitle-sidecar`

建议最小模块：

- `src/index.ts`：stdio loop
- `src/protocol.ts`：消息 schema
- `src/handlers/ping.ts`
- `src/handlers/health.ts`

### Rust 侧

建议新增：

- `app-core::subtitle_host`
- `SubtitleHostPort`
- `SubtitleHostSummary`
- `SubtitleSessionSummary`

`src-tauri` 只负责：

- 找到 sidecar 入口
- 启动/停止进程
- 连接 stdio
- 把消息转发给 `app-core` 侧 host 编排

## 10.5 交付物

- sidecar 消息协议定义
- 最小 Node sidecar
- Rust host wrapper
- 心跳与重启逻辑
- sidecar smoke / restart 验证记录

## 10.6 验收标准

- Rust 宿主能稳定启动 Node sidecar
- 能完成 ping / health / restart
- sidecar 异常退出时宿主能留下稳定错误信息
- 协议模型可直接给后续 UI / 日志系统复用

## 10.7 本阶段必须补的测试

- 协议 schema tests
- stdio framing tests
- host restart tests
- sidecar crash / heartbeat timeout tests

## 10.8 本阶段验证命令

```bash
npm run check --workspace @mediaplayernext/subtitle-sidecar
cargo test --workspace
cargo run --bin backend_harness -- subtitle ping
cargo run --bin backend_harness -- subtitle health
```

## 10.9 本阶段涉及文件与目录

优先会涉及：

- `apps/subtitle-sidecar/package.json`
- `apps/subtitle-sidecar/src/index.ts`
- `apps/subtitle-sidecar/src/`
- `apps/subtitle-sidecar/scripts/check.ts`
- `packages/contracts/src/commands/`
- `packages/contracts/src/channels/`
- `packages/contracts/src/events/`
- `crates/app-core/src/lib.rs`
- `crates/app-core/src/ports.rs`
- `src-tauri/src/main.rs`
- `src-tauri/src/bin/backend_harness.rs`
- `docs/fixtures/sidecar-fixture/`
- `docs/benchmarks/`

按子阶段可再细分：

- `B8-0` 协议冻结：`contracts`、`apps/subtitle-sidecar/src/protocol.ts`
- `B8-1` sidecar 最小服务：`apps/subtitle-sidecar/src/index.ts`、`handlers/`
- `B8-2` Rust host：`app-core`、`src-tauri`
- `B8-3` fixture/benchmark：`docs/fixtures/sidecar-fixture/`、`docs/benchmarks/`

## 10.10 本阶段完成后必须更新的 check 项

实现完成后，至少同步更新以下检查项：

- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md`
  - `B8` 当前状态
  - `B8` 完成情况
  - `B8` 完成定义
- `docs/logs/<当天日期>.md`
- `apps/subtitle-sidecar/package.json` 与 sidecar `check`/smoke 脚本
- `docs/fixtures/sidecar-fixture/README.md`
- `docs/benchmarks/` 下的 sidecar 验证记录
- `packages/contracts/src/index.ts` 与 subtitle 相关 commands/channels/events

## 10.11 用于新对话启动的最小提示

当后续要直接开始 `B8` 开发时，可在新对话中只给下面这段提示：

```text
请按 `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md` 的 `10. B8` 章节开始开发，只聚焦 B8。

先读取：
- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md` 中 `10. B8`
- `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
- `apps/subtitle-sidecar/package.json`
- `apps/subtitle-sidecar/src/index.ts`
- `packages/contracts/src/index.ts`
- `crates/app-core/src/ports.rs`
- `src-tauri/src/bin/backend_harness.rs`

工作目标：
- 冻结 subtitle sidecar 的 `stdio + JSON` 协议
- 把 `apps/subtitle-sidecar` 从 placeholder 推进到最小服务
- 建 Rust host wrapper 与 ping/health/restart
- 改动完成后更新 `docs/03...`、当天日志、sidecar fixture/benchmark 文档

约束：
- 不迁移真实字幕业务
- 不做高级 transport
- 保持 `src-tauri` 极薄
```

---

## 11. 阶段间依赖关系

### `B4 -> B5`

- 没有 zip entry stream、`archive_entries` 与真实样本归档验证，缩略图无法稳定支持 zip 内页

### `B5 -> B6`

- 没有缩略图缓存、资产输入面与 `thumb://`，归一化后很难直接验证结果是否可被 UI 消费

### `B5/B6 -> B7`

- 没有稳定 `media_assets` 与 URL 解析面，`ffprobe`、视频首帧与 `media://` / `archive://` 难以保持统一入口

### `B7 -> B8`

- 没有稳定的外部进程 wrapper、日志与错误处理模式，sidecar 宿主协议很容易重复造轮子

因此当前阶段不建议跳过 `B5` 直接做完整播放或字幕 UI。

---

## 12. fixture、golden、benchmark 计划

## 12.1 fixture 目录建议

```text
docs/fixtures/
  small-fixture/
  medium-fixture/
  archive-fixture/
  thumbnail-fixture/
  playback-fixture/
  sidecar-fixture/
```

### `thumbnail-fixture`

- 普通图片
- 方向异常图片
- 超长图 / 超宽图
- zip 内页样本
- 当前完成情况：未开始

### `playback-fixture`

- 小体积视频
- 小体积音频
- `ffprobe` 结果快照样本
- `ffmpeg` 进度样本
- 当前完成情况：未开始

### `sidecar-fixture`

- sidecar 请求/响应 transcript
- crash / timeout 样本
- 当前完成情况：未开始

## 12.2 当前阶段要产出的 golden

- 缩略图金图
- 归一化结果 manifest
- `ffprobe` 结果 snapshot
- `ffmpeg` 进度 snapshot
- sidecar transcript snapshot

说明：

- B5 起开始正式建立缩略图金图
- B6 起把 rar/7z 归一化结果也纳入 golden/manifest
- B7/B8 的 snapshot 更偏协议与元数据，不要求大二进制全部入仓

## 12.3 benchmark 计划

`B5-B8` 至少记录以下基线：

- 普通图片缩略图冷生成耗时
- 普通图片缩略图热命中耗时
- zip 内页缩略图冷生成耗时
- 归一化耗时（rar / 7z 分开）
- `ffprobe` 元数据读取耗时
- `ffmpeg` 抽帧耗时
- `mpv` 会话启动耗时
- sidecar `ping/health/restart` 耗时

---

## 13. 与 `src-tauri` 的关系

在 `B5-B8` 阶段，`src-tauri` 会比 `B1-B4` 多承担一些集成职责，但仍然不是主业务开发面。

允许的改动：

- 注册 `thumb://` / `media://` / `archive://` 协议
- 注册 `thumbnail.*` / `playback.*` / `subtitle.*` 最小命令与 channel
- 维持与扩展 runtime checks
- 启动与管理 subtitle sidecar

不允许的改动：

- 在 protocol handler 里直接写图片解码、zip 解包、`ffprobe` 解析逻辑
- 在 Tauri command 里直接写数据库、归一化或播放器业务逻辑
- 为了调试方便把大块字节重新塞回 JSON IPC

`src-tauri` 在当前阶段的职责是：

- 把 crate 层能力暴露成命令、channel、protocol、sidecar 管理
- 不成为后端真实业务实现的承载层

---

## 14. 每阶段完成定义（Definition of Done）

## `B5` 完成定义

- `media-thumb` 变成真实 crate
- 普通图片与 zip entry 缩略图可生成
- `thumbnails` 表与磁盘缓存可工作
- `thumb://` 输入面可用

当前状态：未开始

## `B6` 完成定义

- `7z` wrapper 可工作
- `rar/7z` 可归一化成 zip
- 归一化产物可接入 `archive index`
- 失败可重试、可清理

当前状态：未开始

## `B7` 完成定义

- `media-playback` 变成真实 crate
- `ffprobe` / `ffmpeg` / `mpv` 最小能力可用
- `media://` / `archive://` 输入面可用
- 视频元数据与抽帧链路可验证

当前状态：未开始

## `B8` 完成定义

- subtitle sidecar 消息协议冻结
- Node sidecar 最小服务可工作
- Rust 宿主能 ping / health / restart
- 协议与宿主错误可稳定观测

当前状态：未开始

---

## 15. 当前建议的实际执行顺序

如果从当前仓库立即继续开工，建议严格按以下顺序落地：

### 第 5 批：`B5-0` 到 `B5`

1. 把 `media_assets` 输入面接通（普通图片 + archive entry）
2. 把 `crates/media-thumb` 变成真实 crate
3. 建普通图片缩略图
4. 建 zip entry 缩略图
5. 建 `thumb://` 协议与最小命令面

### 第 6 批：`B6`

1. 建 `7z` wrapper
2. 建归一化任务模型
3. 建 `rar/7z -> zip` 归一化链路
4. 复用 `B4` 做归档索引与回归

### 第 7 批：`B7`

1. 把 `crates/media-playback` 变成真实 crate
2. 建 `ffprobe` adapter
3. 建 `ffmpeg` 抽帧与进度解析
4. 建 `mpv` session manager
5. 建 `media://` / `archive://` 协议

### 第 8 批：`B8`

1. 冻结 sidecar 协议
2. 把 `apps/subtitle-sidecar` 从 placeholder 推进为最小服务
3. 建 Rust host wrapper 与 heartbeat/restart

完成以上四批后，再进入 UI 接入阶段或单独编写 UI 对接实施文档。

---

## 16. 最终执行结论

当前仓库在 `B1-B4` 已完成首版后，后端先行阶段的正确落地方式不是：

- 直接开始 UI 大迁移
- 直接开始播放器 UI 或字幕 UI
- 直接把缩略图与媒体字节继续塞进 JSON IPC

而是：

1. 先完成 `B5` 缩略图与资产输入面
2. 再完成 `B6` 归一化
3. 再完成 `B7` 播放后端与媒体协议
4. 再完成 `B8` 字幕 sidecar 宿主协议

完成这四步后，后续 UI 才会真正具备：

- 直接消费 URL 的能力
- 不依赖旧 Node/sharp 主链路的缩略图能力
- 可复用的媒体元数据、播放与 sidecar 宿主能力
- 明确、稳定、可测试的命令 / channel / protocol / sidecar 边界
