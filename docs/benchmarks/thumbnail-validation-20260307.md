# 2026-03-07 缩略图链路验证记录（B5）

本记录用于确认 `B5` 阶段缩略图主链路已经具备：

- `asset_id -> ThumbnailSource` 解析
- 普通图片缩略图生成
- zip 内页缩略图生成
- `thumbnails` 表写回
- 磁盘缓存命中
- `thumb://cache/<thumbnail_key>` 协议返回字节

本轮验证基于：

- `crates/media-thumb` 单元测试中的动态生成图片与 zip fixture
- `crates/app-core` 缩略图用例测试
- `src-tauri` 协议 smoke 测试

## 基本信息

- 日期：`2026-03-07`
- 阶段：`B5`
- 验证入口：`thumbnail.ensure` / `thumbnail.show` / `thumb://cache/<thumbnail_key>`
- 验证目标：确认缩略图核心后端链路已经闭合，且后续前端可以直接消费协议 URL

## 验证场景

### 普通图片资产

- 输入类型：`MediaAssetRecord(source_kind=file)`
- 结果：
  - 可解析为 `ThumbnailSource::FilePath`
  - 可生成 `grid-sm` 缩略图
  - 输出尺寸不超过 `240x240`
  - 可写入 `thumbnails` 表
  - 第二次生成可命中磁盘缓存

### zip 内页资产

- 输入类型：`MediaAssetRecord(source_kind=archive_entry)`
- 结果：
  - 可解析为 `ThumbnailSource::ArchiveEntry`
  - 可从 zip entry 读取实际图片字节
  - 可生成 `grid-sm` 缩略图
  - 输出尺寸不超过 `240x240`
  - 可写入 `thumbnails` 表

### 协议访问

- 输入形式：`thumb://cache/<thumbnail_key>`
- 结果：
  - 可按 `thumbnail_key` 查询 DB 记录
  - 可读取磁盘缓存并返回 `image/webp`
  - 当记录缺失时返回 `404`

## 当前结果判断

- 结果符合预期：`B5` 的核心后端链路已经不再是分散能力，而是形成了完整路径：
  - `asset`
  - `thumbnail.ensure`
  - `thumbnails` 表
  - 磁盘缓存
  - `thumb://`
- 当前剩余工作已从“能否跑通”转为“如何沉淀 fixture / golden / benchmark 基线”

## 当前缺口

- 还没有基于真实样本目录的批量验证记录
- 还没有 profile 维度的字节大小、耗时对比表

## 下一步动作

- 已补 `docs/fixtures/thumbnail-fixture/` 最小 golden 文件
- 下一步补基于真实样本的一轮 thumbnail validation
- 补 profile 级别的输出尺寸、字节大小与耗时基线
