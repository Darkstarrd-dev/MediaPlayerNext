# 缩略图编码格式对比参考（2026-03-13）

## 目标

- 为 `MediaPlayerNext` 缩略图链路确定“可落地的最优组合”：编码格式 + 分辨率 + 画质。
- 约束前提：缩略图是即时生成路径，优先关注生成吞吐、磁盘占用、页面首屏可用时间。

## 当前已知事实

- 当前生产链路（2026-03-13 补更）：
  - `JPEG 输入`：`jpeg-decoder::Decoder::scale` 先做分数级缩放解码（目标边长约 `2x`）
  - `统一缩放`：`fast_image_resize`（SIMD）做 Lanczos3 重采样
  - `输出编码`：`image::codecs::jpeg::JpegEncoder`（按 profile 质量档）
  - 代码：`crates/media-thumb/src/pipeline.rs`
- 当前 profile 分辨率是离散档位，不是原图全分辨率直接编码。
  - `grid-sm=240`、`grid-md=480`、`detail-md=960`、`detail-lg=1440`
  - 代码：`crates/media-thumb/src/profiles.rs`
- 全屏查看不依赖缩略图，走源图读取链路。
- 缓存落盘扩展名已切到 `.jpg`，并通过 pipeline version 升级触发新旧缓存隔离。
  - 代码：`crates/media-thumb/src/cache.rs`、`crates/app-core/src/thumbnail.rs`

## 已完成测试与可用结论

### 1) 与源项目 `sharp-thumb-bench` 的同参数口径对齐

- 脚本：`scripts/bench/run-rust-thumb-bench.mjs`
- 产物：`bench-user-data/rust-thumb-bench/results.md`
- 口径：同输入目录、同 width/quality 序列、同 rounds=3 median。

可用结论：

- 该测试只能用于“当前 Rust lossless 路径吞吐观察”。
- 不能用于“有损质量策略决策”，因为 Rust 侧 quality 参数不生效。

### 2) Rust 内部格式候选（`webp-lossless` vs `jpeg`）

- 脚本：`scripts/bench/run-rust-thumb-format-bench.mjs`
- 产物：`bench-user-data/rust-thumb-format-bench/results.md`
- 口径：同输入、同 width 序列、同 rounds=3 median。

可用结论：

- 在当前 Rust 实现下，`jpeg@q70~q80` 在多数宽度下更快，且体积显著小于 `webp-lossless`。
- 该结论可用于“当前实现的默认格式临时选择”。

### 3) 有损同参数测试（`webp` vs `jpeg`，q40/q50/q60）

- 脚本：`scripts/bench/run-lossy-codec-quality-bench.mjs`
- 产物：`bench-user-data/lossy-codec-quality-bench/results.md`
- 口径：
  - 输入：`Z:/PureBenchFolder/zip/test`（80 张）
  - 宽度：`512`
  - 编码：`sharp.webp({quality})` vs `sharp.jpeg({quality})`
  - 质量档：`40/50/60`
  - 轮次：`3`（取 median）

可用结论：

- 在该样本与参数下，`webp` 体积更小（约为 `jpeg` 的 `0.79~0.83`）。
- 在该样本与参数下，`webp` 编码更慢（约为 `jpeg` 的 `1.51~1.54` 倍耗时）。

限制：

- 该测试是“同 quality 参数”，不是“同感知画质”。
- 因此可用于速度/体积趋势判断，但不能直接作为最终画质等价决策。

### 4) WebP 输入主场景下的 JPEG 优先对比（同源项目口径）

- 脚本：`scripts/bench/run-webp-input-jpeg-priority-bench.mjs`
- 命令：`npm run bench:thumbnail:webp-input-jpeg`
- 产物：`bench-user-data/webp-input-jpeg-priority-bench/results.md`
- 对比对象：
  - `MediaPlayerX` 基线：`sharp + webp(quality)`（读取 `results.md`）
  - `MediaPlayerNext`：`rust + jpeg`
  - `MediaPlayerNext` 控制组：`sharp + jpeg`
- 口径：
  - 输入仍为 `Z:/PureBenchFolder/zip/test`（80 张 `.webp`）
  - `width` 序列与 `quality` 序列与源项目一致
  - `rounds=3` 取中位数

可用结论：

- 在 `webp` 输入主场景下，`MediaPlayerNext` 的 `rust+jpeg` 与 `sharp+jpeg` 均显著快于 `MediaPlayerX` 的 `sharp+webp` 基线（同参数口径）。
- 在“目标链路首版落地”后（scaled decode + SIMD resize + JPEG 编码链），`rust+jpeg` 在同仓 `sharp+jpeg` 控制组上转为稳定领先：
  - `width=512,q50`：`rust+jpeg 727.79ms` vs `sharp+jpeg 865.05ms`（`0.84x`）
  - `width=768,q50`：`rust+jpeg 755.95ms` vs `sharp+jpeg 1053.26ms`（`0.72x`）
  - `quality=50,width=512`：`rust+jpeg 736.21ms` vs `sharp+jpeg 870.26ms`（`0.85x`）

限制：

- 当前缩放链路仍是“分数级 scaled decode + 最终 Lanczos3 重采样”的折中方案，不是 libjpeg-turbo 全链路实现。
- 本轮对比未补 `SSIM/PSNR`，所以“默认质量档位”的最终决策仍需等画质桶对齐。

## 当前不可直接采信的对比

- `jpeg(有损)` vs `webp-lossless(无损)` 的结果不能直接作为“格式终局结论”。
- 原因：损失模型不同，不属于同一画质约束条件。

## 正确的对比方式（最终决策口径）

必须满足“同等质量约束 + 同等尺寸约束 + 同等并发约束 + 同等样本集”。

### A. 编码候选（同类比较）

- `JPEG lossy`：q60/q70/q80
- `WebP lossy`：q50/q60/q70/q80
- `WebP lossless`：仅作上界参考，不参与“默认即时缩略图”候选

### B. 分辨率候选

- 按 UI 实际可见边长与 `DPR` 计算目标边长（优先）。
- 离散 profile 仅作为 fallback，不应强行高档。

### C. 评价指标

- 生成吞吐：`img/s`（cold 与 warm 分开）
- 端到端体验：7x7 首屏从触发到可见缩略图 ready 的耗时
- 平均体积：`KB/图`
- 画质指标：`SSIM` / `PSNR`（相对原图）
- 在画质指标下建立“等画质桶”后再做格式决策

### D. 决策规则（建议）

- 在 `SSIM >= 目标阈值` 前提下，优先最小化首屏 ready 时间。
- 若首屏差异不显著（<5%），优先体积更小方案。

## 下一步实施建议

1. 继续补齐“分阶段耗时采样”（decode/resize/encode）并入基准结果，防止后续回归时只能看总耗时。
2. 新增统一基准脚本：同一批样本一次性产出 `JPEG/WebP lossy` 全矩阵结果（包含 `SSIM/PSNR`）。
3. 在 UI 真实链路做 `7x7` 端到端复测（包含调度开销），将“纯编码基准”与“业务体验基准”分开汇报。

## 当前可执行命令

- `npm run bench:thumbnail:rust`
- `npm run bench:thumbnail:format`
- `npm run bench:thumbnail:webp-input-jpeg`
