# thumbnail-fixture 说明

本目录用于承载 `B5` 阶段缩略图链路的 fixture、golden 与协议约定说明。

当前阶段约定：

- 仓库内优先保留轻量、稳定、可重复的文本基线与 golden 描述
- 大体积真实图片与归档样本，继续通过测试动态生成或在仓库外临时目录中验证
- 当前主链路只覆盖：
  - 普通文件图片
  - `zip/cbz` 内图片页

## 当前固定内容

- golden 文件：
  - `thumbnail-profiles.expected.json`
  - `thumbnail-cache-layout.expected.json`
  - `thumbnail-protocol.expected.json`
  - `thumbnail-protocol-invalid-uri.expected.json`

- 输入源类型：
  - `ThumbnailSource::FilePath`
  - `ThumbnailSource::ArchiveEntry`
- profile 集：
  - `grid-sm` -> `240x240`
  - `grid-md` -> `480x480`
  - `detail-md` -> `960x960`
  - `detail-lg` -> `1440x1440`
- 输出格式：`webp`
- cache key 规则：
  - `thumbnail_key = hash(source_identity + source_revision + profile + pipeline_version)`
- 磁盘缓存布局：
  - `data/cache/thumbs/<key[0..2]>/<key[2..4]>/<thumbnail_key>.webp`
- 协议输入面：
  - `thumb://cache/<thumbnail_key>`

## 当前验证方式

- `crates/media-thumb` 单元测试负责验证：
  - profile 尺寸
  - cache key 稳定性
  - 两级缓存路径
  - 普通图片缩略图生成
  - zip entry 缩略图生成
  - cache hit / miss 语义
- `src-tauri` 单元测试负责验证：
  - `thumb://cache/<thumbnail_key>` URI 解析
  - 协议成功返回图片字节
  - 缺失记录返回 `404`
  - 非法 URI 返回 `400 + INVALID_ARGUMENT`

## 后续计划

- 补真实样本验证记录，固定：
  - 普通图片输入
  - zip 内页输入
  - 不同 profile 的输出尺寸与字节范围
- 后续如开始支持视频首帧缩略图，应另开新小节，不直接混入本目录现有主链路
