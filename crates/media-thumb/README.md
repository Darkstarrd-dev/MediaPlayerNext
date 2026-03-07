# media-thumb

负责缩略图派生链路的 Rust crate。

当前已落地的首批能力：

- 缩略图 profile 定义
- `file path` / `archive entry` 两类输入源模型
- 缓存 key 与两级磁盘布局
- 普通图片与 zip 内页的最小缩略图生成

当前还未落地：

- 与 `thumbnails` 表的正式写回接线
- `thumb://` 协议
- benchmark / golden / fixture 文档
