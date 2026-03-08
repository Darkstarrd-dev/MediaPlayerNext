# 2026-03-08 P6 可观测性首轮验证记录（P6-5）

本记录用于把 `P6-5` 第一轮“日志字段与外部进程日志格式收口”固定成可复查结果。

## 本轮新增内容

- `crates/shared-model/src/observability.rs`
- `packages/contracts/src/models/observability.ts`
- `packages/contracts/fixtures/external-process-log.sample.json`

## 本轮统一口径

外部进程日志统一输出为一行 JSON，核心字段固定为：

- `event`
- `phase`
- `tool`
- `executable`
- `arguments`
- `commandLine`
- `exitCode`
- `durationMs`
- `ok`
- `context.taskId / assetId / sourceId / sessionId`
- `stderrExcerpt`

当前已接入首轮统一日志的链路：

- `runtime-check`
- `subtitle-sidecar`
- `ffprobe`
- `ffmpeg`
- `mpv`
- `sevenz`

其中当前已接入的业务上下文字段包括：

- subtitle sidecar
  - `assetId`
  - `sessionId`
- playback probe / mpv
  - `assetId`
  - `sessionId`（mpv 会话启动）
- archive normalize / sevenz
  - `taskId`
  - `sourceId`

## 本轮验证命令

```bash
npm run test --workspace @mediaplayernext/contracts
scripts/run-cargo-with-msvc.cmd test --workspace
npm run check
```

## 本轮验证结果

- contracts fixture / zod 校验通过
- Rust workspace tests 通过
- `npm run check` 通过
- `runtime-smoke-check` 已实际输出统一 JSON 日志
- playback / subtitle 事件样本已补到 fixtures，并进入 contracts test
- playback / subtitle request、`ffmpeg-progress`、`library.removed`、`scan.failed` 也已进入 contracts test
- 当前 `library/scan/items/archive/thumbnail/playback/subtitle` 这批已存在 command request 都已经有 fixture 覆盖

## 样例日志

```json
{"event":"external-process","phase":"completed","tool":"runtime-check","executable":"C:\\Tools\\ffmpeg-7.1.1-essentials_build\\bin\\ffmpeg.exe","arguments":["-version"],"commandLine":"C:\\Tools\\ffmpeg-7.1.1-essentials_build\\bin\\ffmpeg.exe -version","exitCode":0,"durationMs":31,"ok":true,"context":{}}
```

## 当前判断

- `P6-5` 已开始进入真实代码收口，不再只是文档口头约定。
- 外部进程日志字段已形成跨 Rust crate 与 contracts 的统一模型。
- 当前仍属于首轮，不代表所有业务链路都已经把 `task/source/asset/session` 全量贯穿。
- 但 `asset/session` 与 `task/source` 两组主键已经开始进入真实外部进程日志，而不是只存在于文档约定中。

## 下一步

- 继续把 `task/source/asset/session` 扩到更多后端链路
- 再收紧剩余 contracts 与事件模型
