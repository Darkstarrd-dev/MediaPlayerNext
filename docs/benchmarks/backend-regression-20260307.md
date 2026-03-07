# 2026-03-07 后端先行阶段回归收口记录（B5-B8）

本记录用于在 `B8` 完成后，对 `B5-B8` 的当前仓库状态做一轮统一回归收口。

它关注的不是单一功能点，而是确认：

- `B5` 缩略图链路仍然稳定
- `B6` 归一化链路仍然稳定
- `B7` 播放与协议输入面仍然稳定
- `B8` sidecar 宿主协议仍然稳定
- 根级检查、contracts、Rust workspace 测试与 runtime smoke 没有在阶段收口后被新的改动破坏

## 本轮回归命令

### Node / runtime

```bash
npm run check
npm run test --workspace @mediaplayernext/contracts
```

### Rust workspace

```bash
scripts/run-cargo-with-msvc.cmd fmt --all
scripts/run-cargo-with-msvc.cmd clippy --workspace --all-targets --all-features -- -D warnings
scripts/run-cargo-with-msvc.cmd test --workspace
```

## 本轮结果

### 根级检查

- `npm run check`：通过
- subtitle sidecar 协议 smoke：通过
- runtime smoke：通过
  - `sqlite_version=3.50.2`
  - `ffmpeg`：可调用
  - `ffprobe`：可调用
  - `mpv`：可调用

### contracts

- `npm run test --workspace @mediaplayernext/contracts`：通过
- 当前通过的 fixture / schema 校验共 `7` 项：
  - task progress
  - app error
  - media probe
  - playback session
  - subtitle health
  - subtitle session
  - subtitle progress

### Rust workspace

- `cargo fmt --all`：通过
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`：通过
- `cargo test --workspace`：通过

当前关键测试统计：

- `app-core`：`17` 项通过
- `media-db` integration：`8` 项通过
- `media-io`：`14` 项通过
- `media-playback`：`7` 项通过
- `media-thumb`：`8` 项通过
- `src-tauri` / sidecar / protocol：`10` 项通过
- `shared-model`：`4` 项通过

## 对 B5-B8 的当前判断

### `B5` 缩略图

- 当前仍保持闭环：`asset -> thumbnail.ensure -> thumbnails -> thumb://`
- 没有在后续 `B6-B8` 改动中退化

### `B6` 归一化

- 当前仍保持闭环：`rar/7z -> normalized.zip -> archive index`
- 失败重试语义仍可用

### `B7` 播放

- 当前仍保持闭环：`probe / ffmpeg wrapper / mpv session / media:// / archive://`
- runtime smoke 继续覆盖 `ffprobe` 与 `mpv`

### `B8` sidecar

- 当前仍保持闭环：`contracts -> Node sidecar -> Rust host wrapper -> subtitle.* command`
- 正常路径与异常路径都已有固定验证：
  - `ping / health / start_session / stop_session / get_progress / shutdown`
  - `export_srt` 占位错误
  - restart retry
  - timeout

## `P6-0` 首轮性能基线

缺失项已在 `docs/benchmarks/p6-performance-baseline-20260307.md` 补齐，本轮首轮结果如下：

- 普通图片缩略图冷生成：`avg 2980.124ms`，`median 2979.274ms`
- 普通图片缩略图热命中：`avg 19.595ms`，`median 14.472ms`
- zip 内页缩略图冷生成：`avg 4410.095ms`，`median 4394.651ms`
- `7z -> normalized.zip`：`avg 173.717ms`，`median 183.128ms`
- `ffprobe`：`avg 40.592ms`，`median 36.779ms`
- `ffmpeg` 单帧抽取：`avg 76.005ms`，`median 75.855ms`
- `mpv` 最小启动代理：`avg 6.228ms`，`median 3.991ms`
- sidecar `ping`：`avg 465.839ms`，`median 108.263ms`
- sidecar `health`：`avg 127.286ms`，`median 129.021ms`
- sidecar `restart`：`avg 136.634ms`，`median 136.249ms`

说明：

- sidecar `ping` 首轮存在明显冷启动离群值，因此当前更适合把中位数作为常态口径
- `mpv` 当前使用最小启动代理命令，而不是最终 UI 会话口径

## 本轮结论

- `B5-B8` 当前已经具备“阶段完成后仍能整体回归通过”的状态
- `P6-0` 已完成首轮真实性能补齐，后续可以切到 `P6-1` 质量门禁自动化
- 当前仓库已经可以从“逐阶段落地”切换到“等待 UI 收尾并准备 `I1` 接入”的节奏
