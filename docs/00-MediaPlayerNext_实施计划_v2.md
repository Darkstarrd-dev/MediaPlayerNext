# MediaPlayerNext 优化后的实施计划 v2

## 1. 文档定位

本文档用于替代现有 v1 迁移计划，目标不是“马上开始 UI 迁移”，而是把项目切换到 **后端先行、UI 后接** 的实施节奏。

当前判断：

- 旧仓 UI / theme / 交互出口仍在收尾，暂不适合开始大规模前端迁移。
- 但 Rust 后端中大量能力可以先做，并且这些能力一旦按正确边界实现，后续 UI 落地后可以直接接入，而不需要返工。
- 因此 v2 的核心策略是：
  - **前端迁移延后**
  - **Rust 核心能力前置**
  - **合同、协议、缓存、数据库、任务系统先稳定**
  - **Tauri 宿主保持极薄，尽量晚接入，避免 UI 未冻结时反复改桥接层**

---

## 2. v2 的核心目标

v2 不以“跑通一个能看的壳”为唯一目标，而以“把以后最难、最重、最容易拖慢 UI 集成的能力先独立做完”为主。

本轮目标分成两层：

### 2.1 当前阶段目标（UI 未完成前）

完成以下 **不依赖 UI** 的能力：

1. 后端合同与 DTO 稳定化
2. SQLite schema、migration、repository 落地
3. 扫描 / 入库 / 增量更新流水线
4. `zip` 主链路读取能力
5. `rar/7z -> zip` 归一化流水线
6. 缩略图主链路从 Node/sharp 收回 Rust
7. 媒体资源统一定位与自定义协议设计
8. `ffprobe / ffmpeg / mpv` 的后端适配层
9. 字幕 sidecar 宿主协议冻结
10. 基准测试、fixture、日志、错误码、任务状态系统

### 2.2 后续阶段目标（UI 冻结后）

在不推翻当前后端实现的前提下：

1. 将旧仓 UI 迁入 `apps/desktop`
2. 用 repository / adapter 方式接上新后端
3. 将图片、归档、缩略图、播放、字幕逐步接回
4. 做性能回归与打包收尾

---

## 3. 最终推荐路线（v2）

### 3.1 总路线保持不变

- 新仓独立维护：`MediaPlayerNext`
- 前端继续 `Vite + React + three.js/WebGL + 现有 theme 体系`
- 宿主采用 `Tauri 2 + Rust`
- 字幕保留 `Node.js sidecar`
- Windows 首发

### 3.2 对 v1 的关键升级

v1 的桥接策略以 `command + event` 为主。v2 调整为：

- `command`：控制面、请求-响应
- `channel`：高频进度、流式回传
- `custom protocol`：图片 / 缩略图 / 媒体二进制内容
- `event`：低频广播（库变化、任务完成、配置变化）

这意味着：

- 不再让大图片、缩略图字节流通过 JSON IPC 传输
- 不再把高频任务进度全部塞进 event
- UI 后续只需要拿 URL / ID / 状态，不需要自己拼 Blob

### 3.3 v2 的核心工程观念

1. **Library-first**：核心逻辑先写成纯 Rust crate，不依赖 Tauri 窗口。
2. **Contract-first**：前后端接口先定义，再做实现。
3. **Host-thin**：`src-tauri` 只负责注册命令、协议、sidecar、窗口，不承载业务实现。
4. **Backend-first**：在 UI 未冻结时，优先推进扫描、缩略图、数据库、归档等重能力。
5. **Fixture-driven**：所有核心模块都必须有样本库、金图、基准脚本。
6. **可集成性优先**：当前做的东西必须能在 UI 落地后直接对接，而不是临时 PoC。

---

## 4. 目标架构（v2）

```text
MediaPlayerNext/
  apps/
    desktop/                    # React/Vite 前端
    subtitle-sidecar/           # Node.js 字幕 sidecar
  src-tauri/                    # Tauri 宿主层（极薄）
  crates/
    app-core/                   # 应用服务编排、任务系统、命令入口适配
    media-io/                   # 文件系统、归档、媒体定位、解析
    media-db/                   # SQLite、migration、repository
    media-thumb/                # 缩略图生成、缓存、预热
    media-playback/             # ffprobe / ffmpeg / mpv 适配
    shared-model/               # Rust DTO、错误码、ID、任务模型
  packages/
    contracts/                  # TS/Zod 合同、前端 DTO、事件 / 命令模型
  docs/
    logs/
    fixtures/
    benchmarks/
  config/
  scripts/
```

### 4.1 模块依赖原则

- `shared-model` 只能放纯模型、错误码、ID、状态枚举
- `media-db` 不依赖 Tauri，不依赖前端
- `media-io` 不依赖 Tauri，只依赖 `shared-model` 和必要工具层
- `media-thumb` 调用 `media-io` 获取源，调用 `media-db` 写缓存记录
- `media-playback` 只做外部工具/进程适配，不做 UI 逻辑
- `app-core` 负责编排 use case，不做具体 I/O 细节
- `src-tauri` 只负责把 `app-core` 暴露成 Tauri command / channel / protocol

### 4.2 数据流原则

#### 控制流

前端 -> Tauri command -> `app-core` -> service / repository -> 返回 DTO

#### 流式状态

后端任务 -> channel -> 前端进度订阅

#### 媒体内容

前端 `<img>/<video>` -> `thumb://` / `media://` / `archive://` -> Rust 自定义协议返回字节流

---

## 5. 当前阶段的实施边界

在 UI 收尾完成前，**不做** 以下工作：

- 不做旧仓 UI 模块的大规模迁移
- 不做 theme 系统迁入
- 不做 Electron window API 的全面替换
- 不做面向页面的细碎交互联调
- 不做播放器 UI、字幕 UI 的复杂接线

在 UI 收尾完成前，**优先做** 以下工作：

- 纯后端服务
- 纯 Rust 核心 crate
- 合同定义
- 命令面设计
- 自定义协议设计
- 缓存 / DB / fixture / benchmark
- sidecar 协议
- 外部工具封装

---

## 6. 后续 UI 直接对接所需的接口策略

这是 v2 最重要的一部分：当前阶段做出来的后端，必须从一开始就按“给 UI 对接”的方式设计，而不是仅供 CLI 自测。

### 6.1 接口分层

#### A. Commands（请求-响应）

适合：

- 新建媒体库
- 开始扫描
- 查询列表
- 查询详情
- 请求播放会话
- 请求生成缩略图
- 获取任务状态
- 启动 / 停止字幕 sidecar

#### B. Channels（高频流）

适合：

- 扫描进度
- 入库进度
- 归一化进度
- 缩略图批量预热进度
- `ffmpeg` 转码/抽帧进度
- 长任务日志流

#### C. Events（低频广播）

适合：

- `library.changed`
- `scan.finished`
- `thumb.cache.invalidated`
- `settings.changed`
- `subtitle.sidecar.crashed`

#### D. Custom Protocols（媒体字节）

推荐预留以下协议：

- `thumb://cache/<thumb_key>`
- `media://asset/<asset_id>`
- `archive://entry/<entry_id>`

### 6.2 UI 未来推荐接法

未来 UI 不应该这样做：

- `invoke('get_thumbnail_bytes') -> Uint8Array -> Blob -> objectUrl`

未来 UI 应该这样做：

1. 列表查询接口返回 `thumbnailKey` 或可推导 URL
2. React 组件直接：
   - `<img src={thumbUrl} />`
3. 若缓存未命中：
   - `command('thumbnail.ensure')`
   - `channel('thumb_progress')`
   - 缩略图写盘完成后直接可显示

这样可以保证：

- UI 代码更薄
- 媒体数据不走 JSON IPC
- 后端缓存策略可独立演进

---

## 7. 合同与模型策略

## 7.1 当前阶段推荐策略

继续保留：

- `packages/contracts` 维护 TS / Zod 合同
- Rust 侧在 `shared-model` 中维护镜像 DTO

但 v2 补充两条要求：

1. **必须建立统一错误码模型**
2. **必须建立统一任务状态模型**

### 7.2 必须先冻结的模型

#### 通用 ID

- `LibraryId`
- `SourceId`
- `ArchiveId`
- `ArchiveEntryId`
- `AssetId`
- `ThumbnailKey`
- `TaskId`
- `PlaybackSessionId`
- `SubtitleSessionId`

#### 通用错误码

建议最小集：

- `NOT_FOUND`
- `ALREADY_EXISTS`
- `UNSUPPORTED_FORMAT`
- `PERMISSION_DENIED`
- `INVALID_ARGUMENT`
- `IO_ERROR`
- `DB_ERROR`
- `EXTERNAL_TOOL_ERROR`
- `CANCELLED`
- `TIMEOUT`
- `INTERNAL_ERROR`

#### 通用任务状态

- `queued`
- `running`
- `completed`
- `failed`
- `cancelled`

#### 通用任务进度

```ts
{
  taskId: string;
  taskType: 'scan' | 'ingest' | 'normalize' | 'thumbnail' | 'ffmpeg' | 'subtitle';
  state: 'queued' | 'running' | 'completed' | 'failed' | 'cancelled';
  current: number;
  total?: number;
  message?: string;
  errorCode?: string;
}
```

### 7.3 建议的命令面命名

按域拆命令，而不是按旧 Electron channel 名字平移。

推荐命名空间：

- `library.*`
- `scan.*`
- `items.*`
- `archive.*`
- `thumbnail.*`
- `playback.*`
- `subtitle.*`
- `diagnostics.*`

### 7.4 后续 codegen 策略

- 当前阶段：手写合同 + snapshot tests
- 命令面稳定后：可引入 `tauri-specta` 输出部分 TypeScript bindings
- 不在当前阶段强推全仓 codegen

---

## 8. 当前阶段可先行开发的工作包（详细实施方案）

下面这些工作包均以“**不依赖 UI**、**后续可直接接入**”为前提设计。

---

## WP-01：共享模型与合同冻结

### 目标

把未来前后端对接会反复用到的基础模型先统一，避免后端先写、前端再猜。

### 实施内容

1. 在 `packages/contracts` 中建立：
   - `models/`
   - `commands/`
   - `channels/`
   - `events/`
   - `errors/`
2. 在 `crates/shared-model` 中建立 Rust 对应类型
3. 统一命名：ID、错误码、任务状态、分页结构、媒体摘要结构
4. 建立 snapshot / serde round-trip / zod parse 测试

### 交付物

- 稳定的 TS DTO
- 稳定的 Rust DTO
- 示例 JSON
- 错误码表
- 任务进度模型

### 验收标准

- 同一份示例数据能通过 Zod 校验与 Rust 反序列化
- 未来 UI 在不依赖后端实现的前提下即可开始写 repository 类型签名

---

## WP-02：SQLite schema 与 migration 体系

### 目标

先把媒体库的“数据地基”打稳，让扫描、缩略图、归档、播放元数据都有稳定落点。

### 推荐 schema（首版）

#### `libraries`

记录媒体库根目录与策略。

关键字段：

- `id`
- `root_path`
- `library_type`
- `scan_mode`
- `created_at`
- `updated_at`

#### `sources`

记录文件系统中的原始来源文件。

关键字段：

- `id`
- `library_id`
- `normalized_path`
- `file_name`
- `ext`
- `kind`（image / video / archive / audio / other）
- `size`
- `mtime_ms`
- `fingerprint`
- `exists`
- `last_seen_at`

#### `archives`

记录归档文件本身。

关键字段：

- `id`
- `source_id`
- `archive_type`（zip / rar / 7z）
- `normalized_zip_path`
- `page_count`
- `cover_entry_id`
- `status`

#### `archive_entries`

记录归档内部条目。

关键字段：

- `id`
- `archive_id`
- `entry_path`
- `entry_name`
- `page_index`
- `media_kind`
- `width`
- `height`
- `compressed_size`
- `uncompressed_size`
- `crc32`

#### `media_assets`

统一表示“可展示 / 可播放”的媒体对象。

关键字段：

- `id`
- `source_kind`（file / archive_entry / normalized_file）
- `source_ref_id`
- `mime`
- `width`
- `height`
- `duration_ms`
- `codec_info_json`
- `orientation`
- `created_at`

#### `thumbnails`

记录缩略图缓存。

关键字段：

- `thumbnail_key`
- `asset_id`
- `profile`
- `width`
- `height`
- `format`
- `disk_path`
- `byte_size`
- `state`
- `updated_at`

#### `tasks`

统一长任务状态。

关键字段：

- `id`
- `task_type`
- `state`
- `current`
- `total`
- `message`
- `error_code`
- `error_message`
- `started_at`
- `finished_at`

### 实施要求

1. schema 先覆盖主链路，不求一次性做满全部业务字段
2. migration 必须可重复执行
3. 所有字段命名优先稳定，不为短期方便做随意命名
4. repository 层不暴露 SQL 到上层

### 推荐实现方式

- `rusqlite` 作为主访问层
- 使用 `bundled` 策略保证桌面应用构建可控
- DB 写入采用串行写队列或明确事务边界，避免后续任务系统把 SQLite 打乱

### 交付物

- migration 文件
- repository traits
- repository 实现
- 数据库 smoke tests
- 基础 query benchmarks

### 验收标准

- 能初始化新库
- 能迁移旧 schema 版本
- 能插入 / 更新 / 查询 1000+ 样本记录且行为稳定

---

## WP-03：扫描 / 入库 / 增量更新流水线

### 目标

在完全没有 UI 的情况下，先把“媒体库从路径 -> 数据库”的核心闭环做出来。

### 核心原则

扫描服务不是简单 `walkDir + insert`，而是一个分阶段流水线。

### 建议流水线

#### Stage 1：发现文件

- 遍历 library root
- 过滤扩展名与隐藏路径策略
- 产出候选 `source`

#### Stage 2：快速指纹

为每个候选文件生成快速指纹：

- 归一化路径
- 文件大小
- 修改时间
- 必要时补充 hash

首版不强求全文件 hash，只在冲突、可疑变化、归档归一化后做补充。

#### Stage 3：媒体分类

根据扩展名和必要的头部信息，分为：

- image
- video
- archive
- audio
- other

#### Stage 4：深度检查

- 普通图片：读取基础元数据
- 视频：调用 `ffprobe`
- `zip`：列条目、排序、提取封面候选
- `rar/7z`：登记归一化任务，或在导入时直接归一化

#### Stage 5：DB upsert

- 更新 `sources`
- 新建 / 更新 `archives`
- 新建 / 更新 `archive_entries`
- 新建 / 更新 `media_assets`
- 写入任务进度

#### Stage 6：派生任务

- 缩略图预热
- 归一化任务
- 封面提取
- 视频首帧任务

#### Stage 7：收尾

- 标记本轮未出现的 source 为 `exists = false`
- 产出扫描统计
- 广播低频完成事件

### 增量更新要求

必须支持：

- 重扫时跳过未变化文件
- 删除文件后 tombstone / exists=false
- 扫描中断后可重试
- 扫描失败后记录错误而不是直接把全库搞脏

### 并发模型建议

- 文件遍历：单独生产者
- 分类/解析：有限并发 worker
- `zip` 检查：独立 worker 池
- `ffprobe`：独立 worker 池
- DB：单独写线程 / 明确事务提交点

### 当前阶段必须额外做的内容

为了让 UI 不参与也能推进，扫描服务要同时提供：

1. 纯 Rust service API
2. CLI harness
3. 假数据与真实样本测试

### 推荐 CLI 能力

建议增加一个仅用于开发的 CLI/二进制入口：

- `scan add-library <path>`
- `scan run <library-id>`
- `scan resume <task-id>`
- `scan stats <library-id>`
- `scan diff <library-id>`

### 交付物

- 扫描 service
- 入库 service
- 任务状态回传
- CLI harness
- fixture 扫描报告

### 验收标准

- 能对固定样本库完成首轮入库
- 二次重扫能跳过绝大部分未变化文件
- 扫描中断后可恢复
- 出错文件不会导致全任务崩溃

---

## WP-04：缩略图主链路 Rust 化（重点工作包）

### 目标

把缩略图主链路从 Node/sharp 收回 Rust，使其成为后续 UI 迁移时可以直接使用的稳定底层能力。

### 这项工作为什么要现在做

因为缩略图是典型的高频能力：

- 列表页
- 网格页
- 封面页
- 归档浏览
- 预加载

如果这条链路不先稳定，UI 迁移后会立刻遇到：

- IPC 压力
- Blob 拼装
- 缓存策略混乱
- Node 与 Rust 双实现并存

### 首版设计目标

#### 输入抽象统一

定义统一的 `ThumbnailSource`：

- `FilePath`
- `ZipEntry`
- `NormalizedArchiveEntry`
- `VideoFrame`

以后 UI 不关心源来自普通文件还是归档内页，只关心 `thumbnailKey` 和 URL。

#### 输出抽象统一

定义 `ThumbnailProfile`：

- `grid-sm`
- `grid-md`
- `detail-md`
- `detail-lg`

所有缩略图都按照 `source + profile + version` 生成稳定 key。

#### 缓存策略统一

建议：

- `thumbnail_key = hash(source_identity + source_revision + profile + pipeline_version)`
- 文件落盘使用两级目录散列
- 先写临时文件，再原子 rename
- DB 记录状态，磁盘保存实体

### 推荐磁盘布局

```text
cache/
  thumbs/
    ab/
      cd/
        <thumbnail_key>.webp
```

### 处理流程

#### 普通图片

1. 读取源文件
2. 自动方向修正
3. resize / crop
4. 编码成目标格式
5. 落盘并写 DB

#### `zip` 内页

1. 打开 zip entry 流
2. 解码图片
3. 后续流程同普通图片

#### 视频

1. 调 `ffmpeg` 抽首帧或指定时间点
2. 进入同一套 resize / encode / cache 流程

### 当前阶段的实施建议

#### 第 1 步：建立 Rust 缩略图服务骨架

- `media-thumb::service`
- `media-thumb::pipeline`
- `media-thumb::cache`
- `media-thumb::profiles`

#### 第 2 步：先支持图片文件与 zip entry

- 普通图片
- `zip` 内图片

这是最有价值、最可控的起点。

#### 第 3 步：补视频首帧

通过 `ffmpeg` 抽帧，不在首版引入更重的视频解码栈。

#### 第 4 步：建立 parity 测试

当前 Node/sharp 不应继续做主链路，但可以暂时作为“对照实现”：

- 对固定样本生成金图
- 与 Rust 输出做视觉 / 尺寸 / 方向一致性比对
- 在确认结果可接受后，再移除 sharp 主职责

### 未来 UI 接口建议

建议 UI 不请求“缩略图字节”，只请求：

- `thumbnail.ensure({ assetIds, profile })`
- `thumbnail.getUrl({ thumbnailKey })`

更理想的做法是列表接口直接返回 `thumbnailUrl`。

### 交付物

- Rust 缩略图 service
- 缩略图 cache 管理器
- profile 体系
- 金图对比测试
- cache key 规范文档

### 验收标准

- 能对普通图片与 zip 内图片生成稳定缩略图
- 二次请求命中磁盘缓存
- 缩略图 URL 可被前端 `<img>` 直接消费
- Rust 输出在方向、尺寸、裁切上达到可接受一致性

---

## WP-05：`zip` 主链路与 `rar/7z` 归一化

### 目标

让归档浏览在后端层先具备可用基础，且统一走“高频 zip、低频归一化”的路线。

### 实施原则

- 高频查看主链路：`zip`
- `rar/7z`：入库时归一化，不进入高频查看主链路
- 首版优先把“读”和“规范化”做稳，写回可放后面

### `zip` 首版能力

- 列目录
- 判断图片页
- 页序排序
- 读取 entry 流
- 提取封面候选
- 与 `media_assets`、`archive_entries` 建立映射

### `rar/7z` 首版能力

- 统一走外部 `7z` 工具
- 入库时输出内部 `zip`
- 记录 `source archive -> normalized zip` 关系
- 失败可重试、可清理

### 强制要求

不要继续依赖旧项目绝对路径上的 `7z.exe`。

v2 应改为：

- 将固定版本 `7z.exe + 7z.dll` 随应用资源或开发配置统一管理
- 在 `config/local.paths.json` 中只保留本地开发 override
- 生产环境走内置资源路径

### 归一化任务建议

任务状态建议拆开：

- `queued`
- `extracting`
- `packing_zip`
- `verifying`
- `completed`
- `failed`

### 交付物

- `zip` 读取服务
- 归一化服务
- `7z` 外部工具包装器
- 归一化任务模型
- 样本归档测试

### 验收标准

- 能稳定读取 zip 图片集
- 能把 rar/7z 归一化为内部 zip
- 归一化失败能留错误记录并支持重试

---

## WP-06：媒体定位与自定义协议

### 目标

在 UI 迁入前先把“媒体如何被读取”设计成稳定 URL 体系，避免未来所有页面都在前端拼数据流。

### 设计原则

- URL 是媒体主入口
- command 返回元数据和 URL，而不是大块字节
- 协议处理器在 Rust 中统一鉴权、寻址、缓存、错误转换

### 推荐协议

#### `thumb://cache/<thumbnail_key>`

返回缩略图文件

#### `media://asset/<asset_id>`

返回普通图片 / 视频文件资源

#### `archive://entry/<archive_entry_id>`

返回归档内页内容

### 实施要求

1. 协议 handler 只做轻逻辑，核心寻址走 `app-core`
2. 错误统一映射为可观测状态
3. 后续页面不要直接感知源文件路径

### 交付物

- URI scheme handler
- URL 生成器
- URL -> source resolver
- 协议级 smoke tests

### 验收标准

- `<img>` 可以直接加载缩略图与归档内页
- 无需通过 IPC 中转字节

---

## WP-07：播放相关后端适配

### 目标

在 UI 还没接回前，先把播放相关的后端能力做成稳定服务层。

### 范围划分

#### `ffprobe`

负责：

- 基础媒体元数据
- 时长 / 分辨率 / 编码信息
- 流信息提取

#### `ffmpeg`

负责：

- 抽帧
- 转码任务
- 进度输出解析

#### `mpv`

负责：

- 播放会话控制
- seek / pause / stop
- 状态查询

### 当前阶段怎么做

1. 建立 process wrapper
2. 建立 stdout / stderr / IPC 解析器
3. 定义播放会话 DTO
4. 做 integration tests

### 当前阶段不做

- 不做播放器 UI
- 不做快捷键层
- 不做复杂播放列表交互

### 交付物

- `media-playback` service
- `ffprobe` adapter
- `ffmpeg` adapter
- `mpv` session manager

### 验收标准

- 能读取视频元数据
- 能执行抽帧并汇报进度
- 能控制本地 mpv 会话完成基本播放控制

---

## WP-08：字幕 sidecar 协议冻结

### 目标

字幕实现本体暂不迁移，但宿主如何管理它，必须先定下来。

### v2 推荐策略

- 首版 transport 用 `stdio`
- 保持消息体 JSON 化
- 后续如确有必要再升级到 named pipe

### 必须先定义的消息

- `ping`
- `health`
- `start_session`
- `stop_session`
- `get_progress`
- `export_srt`
- `shutdown`

### 宿主职责

- 启动进程
- 心跳检测
- 异常退出拉起
- 日志收集
- 会话生命周期管理

### 当前阶段交付物

- sidecar host wrapper
- 消息协议定义
- 最小 ping/heartbeat

### 验收标准

- Rust 宿主能启动 Node sidecar
- 能完成 ping / health / restart

---

## WP-09：开发辅助能力（必须做）

### 目标

保证后端先行开发不是“黑盒推进”。

### 必须补齐的内容

#### 1. 日志

统一 `tracing`：

- task id
- library id
- source id
- asset id
- 外部进程 command line
- 耗时

#### 2. Fixture 数据集

建议至少准备三套：

- `small-fixture`：几十个文件，方便单元与本地调试
- `medium-fixture`：几百到一千级，做功能与性能回归
- `archive-fixture`：zip / rar / 7z / 损坏包 / 边缘命名样本

#### 3. Golden Outputs

- 缩略图金图
- 扫描结果 JSON snapshot
- 归档页序 snapshot
- `ffprobe` 结果 snapshot

#### 4. Benchmark 脚本

记录以下指标：

- 首次扫描耗时
- 重扫耗时
- 缩略图冷生成耗时
- 缩略图热命中耗时
- zip 连续读取耗时

### 验收标准

- 每个核心模块都能在无 UI 环境下验证正确性与性能趋势

---

## 9. 建议的开发顺序（当前阶段）

### P0：地基冻结

1. `shared-model`
2. `packages/contracts`
3. 错误码与任务状态
4. CLI harness 骨架
5. `tracing` 基础设施

### P1：数据库与扫描闭环

1. migration
2. repository
3. 文件发现与快速指纹
4. 首轮入库
5. 增量重扫

### P2：zip 与归档信息

1. zip 列目录
2. zip 内页排序
3. 封面候选提取
4. `archive_entries` 落库

### P3：缩略图 Rust 化

1. 普通图片缩略图
2. zip 内页缩略图
3. 缓存落盘
4. 自定义协议 `thumb://`
5. parity 测试

### P4：归一化与视频链路

1. `7z` 包装器
2. `rar/7z -> zip` 归一化
3. `ffprobe` 元数据
4. 视频首帧缩略图

### P5：播放与 sidecar 宿主协议

1. `mpv` session manager
2. `ffmpeg` 进度解析
3. subtitle sidecar host wrapper

### P6：等待 UI 收尾，同时持续做回归

1. benchmark
2. 日志/可观测性
3. 错误场景处理
4. 文档与示例

---

## 10. UI 收尾完成后的对接方案

当旧仓 UI 收尾完成、开始真实迁移时，建议按下面顺序接入，而不是一下子全量接。

### Step 1：前端先做 repository/adapter 收口

先把旧仓里直接 `window.*` 调用继续往 repository / adapter 收口。

目标：

- UI 组件不直接知道 Electron / Tauri
- 只依赖 `MediaRepository` 抽象

### Step 2：新仓接入 contracts

在 `apps/desktop`：

- 引入 `packages/contracts`
- 建立 `tauriMediaRepository`
- 实现 command / channel / event 对接

### Step 3：最先接的页面

推荐顺序：

1. 媒体库选择 / 列表
2. 缩略图网格
3. zip 图片浏览
4. 基础图片查看
5. 视频详情 / 元数据
6. 播放器控制
7. 字幕

### Step 4：逐步替换旧 Electron 接口

- 每替换一个 repository 能力，就删一段旧桥接
- 不做“大爆炸式”替换

### Step 5：最后才接复杂 UI

- 高级播放 UI
- 字幕编辑 / 同步
- 复杂批量操作
- 可视化扩展

---

## 11. 里程碑（v2）

### 后端先行阶段

- `B1`：共享模型、错误码、任务模型、CLI 骨架完成
- `B2`：SQLite schema + migration + repository 完成
- `B3`：扫描 / 入库 / 增量更新完成
- `B4`：zip 列目录与归档索引完成
- `B5`：Rust 缩略图主链路完成（普通图片 + zip 内页）
- `B6`：`rar/7z` 归一化完成
- `B7`：`ffprobe / ffmpeg / mpv` 适配完成
- `B8`：subtitle sidecar host 协议完成

### UI 接入阶段

- `I1`：repository / adapter 接上
- `I2`：缩略图列表页可用
- `I3`：zip 浏览可用
- `I4`：媒体库全链路可用
- `I5`：播放器接回
- `I6`：字幕接回
- `I7`：性能 / 包体 / 稳定性达到替代门槛

---

## 12. 当前阶段不建议做的事（v2）

- 不要先迁 theme
- 不要先迁所有页面
- 不要把大量业务放进 `src-tauri`
- 不要让前端通过 command 取大块图片字节
- 不要把所有进度都塞进 event
- 不要继续把 `7z.exe` 绑死在旧仓路径
- 不要把 sharp 继续当成长期主链路
- 不要在命令面未稳定时就强推全仓 codegen

---

## 13. 推荐的立即执行清单

如果现在就开始做，建议按下面顺序落地：

### 第一批（马上可做）

1. 建 `shared-model` 基础类型
2. 建 `packages/contracts` 初版
3. 接入 `thiserror`、`tracing`、`tokio`
4. 建 migration 体系
5. 建 CLI harness

### 第二批（最有价值）

1. 做扫描 / 入库首版
2. 做 zip 列目录首版
3. 做缩略图 Rust service 首版
4. 做 `thumb://` 协议

### 第三批（补强）

1. 做 `rar/7z` 归一化
2. 做 `ffprobe` 与视频首帧
3. 做 `mpv` wrapper
4. 做 subtitle sidecar ping/heartbeat

---

## 14. v2 的最终结论

在当前“UI 还未落地、但后端可以先做”的阶段，最优策略不是急着开始 UI 迁移，而是：

- 先把 Rust 后端做成 **可独立运行、可测试、可 benchmark、可通过合同直接接 UI** 的服务层
- 优先完成扫描、入库、zip、缩略图、缓存、协议、播放适配、sidecar 管理
- 等 UI 收尾后，再把前端以 repository/adapter 方式接过来

这样做的收益是：

1. UI 不会被后端不稳定拖慢
2. 后端能提前消化最重、最难、最耗时的能力
3. UI 接入时只需要“接接口”，而不是边迁边设计后端
4. 缩略图、扫描、归档这类高频能力能从第一天就按最终形态设计

