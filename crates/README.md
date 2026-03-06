# Rust Crates Skeleton

本目录用于承载后续 Rust 业务模块拆分，当前阶段只建立骨架，不放入真实迁移代码。

预留子模块：

- `app-core`：应用编排、命令注册、事件派发
- `media-io`：文件系统、归档读取、资源解析、预加载
- `media-db`：SQLite 访问层
- `media-thumb`：缩略图与缓存
- `media-playback`：`mpv/ffmpeg` 控制
- `shared-model`：Rust 内部共享模型

当前阶段要求：

- 仅保留目录与说明
- 不提前复制旧仓逻辑
- 不提前建立复杂 workspace 依赖关系
