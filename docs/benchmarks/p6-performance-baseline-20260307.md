# 2026-03-07 P6 真实性能基线记录（P6-0）

本记录用于补齐 `docs/benchmarks/backend-regression-20260307.md` 中仍缺失的真实性能数字。

本轮不改业务语义，只补：

- 普通图片缩略图冷 / 热命中耗时
- zip 内页缩略图冷生成耗时
- 真实 `7z.exe` 归一化耗时
- 真实视频样本下的 `ffprobe` / `ffmpeg` 耗时
- `mpv` 最小启动代理耗时
- sidecar `ping / health / restart` 毫秒级统计

## 本轮执行入口

```bash
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\bench\run-thumbnail-benchmark.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\bench\run-normalize-benchmark.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\bench\run-playback-benchmark.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\bench\run-sidecar-benchmark.ps1
```

## 环境口径

- 日期：`2026-03-07`
- 当前 commit：`d9bbe69`
- 工作区状态：`dirty`（正在进行 `P6-0` 实施）
- 运行时：
  - `ffmpeg`：`C:\Tools\ffmpeg-7.1.1-essentials_build\bin\ffmpeg.exe`
  - `ffprobe`：`C:\Tools\ffmpeg-7.1.1-essentials_build\bin\ffprobe.exe`
  - `7z`：`C:\Program Files\7-Zip\7z.exe`
  - `mpv`：`Z:\Playground\CurrentWorking\mpv\mpv.exe`
  - `node`：`v22.13.1`
- 结果输出目录（本地、未入库）：
  - `data/benchmarks/20260307-195908/thumbnail`
  - `data/benchmarks/20260307-200018/normalize`
  - `data/benchmarks/20260307-205505/playback`
  - `data/benchmarks/20260307-205206/sidecar`

## 样本口径

### 缩略图

- 样本根目录：`docs/fixtures/medium-fixture/generated-placeholder/scan-root`
- 普通图片样本：`english/0001-page-alpha.png`
- zip 内页样本：`english/0001-chapter-alpha.zip` 内 `01.webp`
- profile：`grid-sm`
- 冷 / 热定义：
  - 冷：每轮删除同一 `thumbnail_key` 对应磁盘缓存后再执行 `thumbnail ensure`
  - 热：先预热一次，再连续执行 `thumbnail ensure`

### 归一化

- 基线样本：从 `docs/fixtures/small-fixture/generated-placeholder/` 抽 4 个真实图片文件
- 先用本机 `7z.exe` 打成单个 `benchmark-input.7z`
- 每轮删除 `data/cache/normalized` 对应输出目录后再执行 `archive normalize`

### 播放

- 视频样本：`docs/fixtures/small-fixture/generated-placeholder/english/0001-clip-alpha.mp4`
- `ffprobe`：直接读取 JSON 元数据
- `ffmpeg`：抽取 `0s` 单帧到本地 `frame.png`
- `mpv`：当前阶段使用最小启动代理命令
  - `mpv --no-config --really-quiet --vo=null --ao=null --frames=1 <sample>`
  - 说明：当前 `B7` 仍未接 `mpv` IPC，本轮先固定“最小真实进程启动 + 单帧退出”口径，而不是 UI 会话口径

### sidecar

- `ping` / `health`：使用 `apps/subtitle-sidecar/dist/src/index.js`
- `restart`：使用一次性 `retry-sidecar.mjs`，首启故意 `exit 2`，由 Rust host wrapper 重试一次

## 结果

### 缩略图

| 项目 | runs | avg(ms) | median(ms) | min(ms) | max(ms) |
|---|---:|---:|---:|---:|---:|
| 普通图片冷生成 | 5 | 2980.124 | 2979.274 | 2950.778 | 3016.582 |
| 普通图片热命中 | 5 | 19.595 | 14.472 | 13.612 | 39.124 |
| zip 内页冷生成 | 5 | 4410.095 | 4394.651 | 4370.109 | 4468.656 |

补充：

- 普通图片输出：`164x240`，`28620 bytes`
- zip 内页输出：`171x240`，`72250 bytes`

### 归一化

| 项目 | runs | avg(ms) | median(ms) | min(ms) | max(ms) |
|---|---:|---:|---:|---:|---:|
| `7z -> normalized.zip` | 3 | 173.717 | 183.128 | 164.882 | 183.128 |

补充：

- 输入样本：4 张图片组成的单个 `benchmark-input.7z`
- 结果：`extractedFileCount=4`，`indexedEntries=4`

### 播放

| 项目 | runs | avg(ms) | median(ms) | min(ms) | max(ms) |
|---|---:|---:|---:|---:|---:|
| `ffprobe` 元数据读取 | 5 | 40.592 | 36.779 | 35.652 | 56.339 |
| `ffmpeg` 单帧抽取 | 5 | 76.005 | 75.855 | 73.330 | 78.725 |
| `mpv` 最小启动代理 | 5 | 6.228 | 3.991 | 3.363 | 15.939 |

补充：

- `ffprobe` 结果确认样本包含视频流、音频流与附带封面流
- `ffmpeg` 输出：`frame.png`，`267101 bytes`

### sidecar

| 项目 | runs | avg(ms) | median(ms) | min(ms) | max(ms) |
|---|---:|---:|---:|---:|---:|
| `subtitle ping` | 5 | 465.839 | 108.263 | 107.547 | 1886.357 |
| `subtitle health` | 5 | 127.286 | 129.021 | 120.338 | 133.110 |
| `subtitle restart` | 5 | 136.634 | 136.249 | 127.404 | 147.410 |

补充：

- `ping` 首轮存在明显冷启动离群值：`1886.357ms`
- `restart` 场景已确认 `restartCount=1` 且保留首次失败信息

## 判断

- `P6-0` 所要求的缺失项已完成首轮补齐
- 缩略图链路当前最重的是冷生成，热命中已明显进入毫秒级
- `7z` 归一化首轮耗时远低于缩略图冷生成，当前更像低频后台任务，而不是前台瓶颈
- `ffprobe` / `ffmpeg` 当前样本耗时已稳定在双位数毫秒级
- sidecar 常态 `health` / `restart` 在 `127-147ms` 区间，`ping` 首轮冷启动需要单独看待，不应直接拿平均值当常态口径

## 当前限制

- 本轮只做单机首轮基线，不做跨机器对照
- `mpv` 当前仍是最小启动代理口径，不是未来 UI 会话口径
- sidecar `ping` 平均值受首轮冷启动影响较大，后续如要做门禁阈值，应优先使用中位数或拆分冷 / 热启动口径
