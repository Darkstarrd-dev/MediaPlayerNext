# MediaPlayerNext P6 收口与 I1 前置具体实施计划 v1

## 1. 文档定位

本文件不是替代 `docs/00-MediaPlayerNext_实施计划_v2.md`，而是在 `docs/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 已明确 UI 接入顺序后，把 `P6` 阶段继续拆成可直接执行的实施清单，用于在真实前端接入前把 benchmark、质量门禁、错误场景、接口文档、资源路径、可观测性与 contracts 再收紧一轮。

六份文档的职责固定如下：

- `docs/00-MediaPlayerNext_实施计划_v2.md`
  - 负责总路线、工作包边界、接口策略与总体顺序
- `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
  - 负责质量门禁、测试要求、迁移流程、benchmark 与发布约束
- `docs/02-MediaPlayerNext_后端先行具体实施计划_B1-B4_v1.md`
  - 负责 `B1-B4` 的数据地基、扫描闭环与 zip 主链路
- `docs/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md`
  - 负责 `B5-B8` 的缩略图、归一化、播放后端与字幕 sidecar 宿主协议
- `docs/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md`
  - 负责 `I1-I4` 的 repository / adapter、缩略图列表、zip 浏览与媒体库全链路 UI 接入
- `docs/05-MediaPlayerNext_P6收口与I1前置具体实施计划_v1.md`
  - 负责 `P6` 的 benchmark、门禁自动化、错误场景、接口文档、资源路径、可观测性与 contracts 收口

本文件只覆盖 **`P6` 阶段继续收口所需的六类工作**：

- `P6-0`：真实性能基线补齐
- `P6-1`：质量门禁自动化落地
- `P6-2`：错误场景补强
- `P6-3`：接口收口文档（为 `I1` 做准备）
- `P6-4`：发布前资源路径策略收口
- `P6-5`：可观测性与 contracts 再收紧

不覆盖：

- `I1-I4` 的实际前端页面开发
- `I5` 播放器接回
- `I6` 字幕接回
- `I7` 最终性能 / 包体 / 稳定性替代门槛收口
- theme 系统整包迁入

这样拆的原因是：当前仓库已经具备 `B1-B8` 首版闭环，继续直接做 UI 接入虽然能推进，但会把 benchmark 缺口、门禁自动化缺口、资源路径策略与接口口径漂移带到前端阶段，导致后续 `I1` 接线时再返工一次。

---

## 2. 已完成内容与当前进度

截至目前，`B1-B8` 已完成首版闭环，`I1-I4` 的实施文档也已建立，但 `P6` 当前仍处于“开始回归收口、尚未坐实基础”的状态。

### 2.1 已完成内容（承接 `B1-B8` 与现有 `P6`）

- `docs/benchmarks/backend-regression-20260307.md` 已形成 `B5-B8` 的统一回归收口记录
- `npm run check`、contracts 测试、Rust workspace 的 `fmt / clippy / test` 已形成一轮可复跑基线
- `thumb://` / `media://` / `archive://` / subtitle sidecar 协议已具备首版验证记录
- `docs/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md` 已明确 `I1-I4` 的顺序与边界

### 2.2 当前进度判断

- `P6-0`：已完成首轮（真实性能数字已补齐，见 `docs/benchmarks/p6-performance-baseline-20260307.md`）
- `P6-1`：已完成首轮（统一脚本、结果产物与首轮 gate run 已落地；`duplicate deps` 已按 baseline-delta 进入 P2 治理）
- `P6-2`：已完成首轮（runtimes / DB fixture replay / protocol / sidecar / command AppError 已完成首轮收口，可正式切入 `P6-3`）
- `P6-3`：进行中（后端边界版文档已落，`library/scan/items/archive/thumbnail` 首批 contracts 已开始实现）
- `P6-4`：未开始（开发态 / 打包态资源查找顺序仍未被单独收口为策略文档与校验脚本）
- `P6-5`：未开始（日志字段贯穿、外部进程统一日志格式与缺失域 contracts 仍待补齐）

### 2.3 当前最自然的下一步

1. 继续进入 `P6-2` 错误场景补强，把 runtimes / DB / protocol / sidecar 的坏路径提前固定成测试与文档
2. 再补接口收口文档、资源路径策略与 contracts，避免 UI 接线时仍靠猜测和临场判断

---

## 3. 当前仓库基线

截至当前更新时，仓库实际状态如下：

- `docs/benchmarks/backend-regression-20260307.md` 已明确还缺：
  - 缩略图冷 / 热命中耗时
  - zip 内页缩略图耗时
  - 真实 `7z.exe` 归一化耗时
  - 真实视频样本下的 `ffprobe / ffmpeg` 耗时
  - `mpv` 启动耗时
  - sidecar `ping / health / restart` 毫秒级统计
- `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md` 已明确 `P0 / P1 / P2` 门禁口径，但仓库内尚无统一的“全门禁执行脚本”与结果产物路径
- `apps/desktop` 仍是最小 Tauri command demo，尚未建立 `MediaRepository`，因此 `I1` 目前仍缺接口收口文档支撑
- `config/local.paths.example.json` 已覆盖 `ffmpeg` / `ffprobe` / `node` / `sevenz` / `mpv` 开发态路径，但打包态查找顺序仍未单独文档化
- `src-tauri` 当前能提供 runtime smoke、custom protocol 与 subtitle host wrapper，但日志字段与外部进程可观测性还没统一口径

因此接下来的首要目标，不再是“继续写新功能”，而是把已经存在的能力转换成：

- 可量化 benchmark
- 可自动复跑门禁
- 可预期 bad path 行为
- 可供前端直接依赖的接口文档
- 可持续验证的资源路径策略

---

## 4. 执行原则

### 4.1 总原则

1. **先把基础坐实，再接 UI**
   - 先把 benchmark、门禁、bad path、路径策略与 contracts 做稳，再进入 `I1`
2. **先自动化，后人工解释**
   - 能写成脚本 / 报告产物的，不依赖人工口头说明
3. **先坏路径，后新能力**
   - 当前阶段优先补缺失、异常与退化场景，不再开新后端能力
4. **先文档收口，后页面接线**
   - 先明确 `window.* -> command/channel/protocol`、`MediaRepository`、DTO 依赖关系，再写 UI
5. **先开发态与打包态统一，后发散资源接入**
   - 所有运行时资源必须在“开发态 / 打包态 / 本地 override”三者之间有稳定查找顺序

### 4.2 当前阶段禁止事项

- 不因为要补 benchmark 而顺手扩写新的业务能力
- 不为了“快接 UI”而跳过质量门禁自动化
- 不让前端继续猜测 contracts / DTO / URL 结构
- 不把资源路径策略继续散落在 README、脚本与代码里而不形成统一文档
- 不在 `src-tauri` 里堆页面级逻辑

### 4.3 对 `I1` 的直接服务要求

`P6` 的收口默认服务于 `I1`，因此必须确保：

- 前端在 `I1` 时只依赖稳定 contracts，不猜测返回结构
- `MediaRepository` 所需方法、参数、返回值与错误语义已有文档口径
- 开发态和打包态的资源查找顺序不会在 UI 接入后才首次暴露问题
- 关键 bad path 已具备可观测、可回放、可验证的基线

---

## 5. 目标目录与职责落位

在 `P6` 内，目录职责固定如下：

```text
MediaPlayerNext/
  scripts/
    bench/                         # benchmark 与 timing 脚本
    quality/                       # fmt/clippy/nextest/coverage/deny/audit/udeps 等质量门禁脚本
    release/                       # 资源路径、打包态校验与 tauri build 验证脚本
  docs/
    benchmarks/                    # benchmark 与 unified regression 结果
    runtime/                       # 资源路径策略文档
    contracts/                     # window 映射、repository 方法面、DTO 依赖矩阵文档
  packages/
    contracts/                     # 继续补齐 library/scan/items/archive/thumbnail 等域合同
  src-tauri/                       # runtime、sidecar、protocol、可观测性与打包校验接线
  crates/
    app-core/                      # bad path、ports、日志字段与错误语义补强
```

说明：

- `scripts/bench/` 是本阶段建议新增的 benchmark 入口目录
- `scripts/quality/` 是本阶段建议新增的门禁自动化入口目录
- `docs/contracts/` 与 `docs/runtime/` 是本阶段建议新增的执行文档目录
- `packages/contracts` 在本阶段继续补齐域合同，但不直接进入真实 UI 页面开发

---

## 6. 阶段拆分总览

| 子阶段 | 目标 | 核心交付物 | 是否阻塞后续 |
|---|---|---|---|
| `P6-0` | 真实性能基线补齐 | benchmark 脚本、真实性能数字、补齐 `backend-regression` 缺口 | 是 |
| `P6-1` | 质量门禁自动化 | quality scripts、统一执行入口、门禁结果产物 | 是 |
| `P6-2` | 错误场景补强 | runtimes / DB / protocol / sidecar bad path tests 与文档 | 是 |
| `P6-3` | 接口收口文档 | `window.*` 映射表、`MediaRepository` 方法清单、DTO 依赖矩阵、传输边界文档 | 是 |
| `P6-4` | 资源路径策略收口 | 开发态 / 打包态查找顺序文档与校验脚本 | 是 |
| `P6-5` | 可观测性与 contracts 再收紧 | 日志字段统一、外部进程日志格式、缺失域 contracts | 否（但强烈建议在 `I1` 前完成） |

推荐顺序仍以串行为主：`P6-0 -> P6-1 -> P6-2 -> P6-3 -> P6-4 -> P6-5`。

说明：

- `P6-0/P6-1/P6-2/P6-3/P6-4` 都直接阻塞 `I1`，因为没有这些基础，前端接入阶段会把隐性风险一起带进去
- `P6-5` 理论上可与 `I1` 并行，但强烈建议先完成大部分收口

---

## 7. P6-0：真实性能基线补齐

当前状态：已完成首轮

## 7.1 阶段目标

把 `docs/benchmarks/backend-regression-20260307.md` 已列出但尚未补齐的真实性能数字真正落到脚本、样本口径与结果记录上，避免后续性能讨论继续停留在“理论上应该不慢”。

## 7.2 范围

### 本阶段要做

1. 补普通图片缩略图冷生成耗时
2. 补普通图片缩略图热命中耗时
3. 补 zip 内页缩略图冷生成耗时
4. 补真实 `7z.exe` 环境下的归一化耗时
5. 补真实视频样本下的 `ffprobe` / `ffmpeg` 耗时
6. 补 `mpv` 会话启动耗时
7. 补 sidecar `ping / health / restart` 毫秒级统计

### 本阶段不做

- 不为了 benchmark 改写业务语义
- 不在当前阶段追求完整跨机器性能实验矩阵
- 不提交大体积真实媒体样本到仓库

## 7.3 模块与文件计划

建议新增：

- `scripts/bench/run-thumbnail-benchmark.ps1`
- `scripts/bench/run-normalize-benchmark.ps1`
- `scripts/bench/run-playback-benchmark.ps1`
- `scripts/bench/run-sidecar-benchmark.ps1`
- `docs/benchmarks/p6-performance-baseline-<date>.md`

记录口径建议：

- 样本路径来源
- 冷 / 热命中定义
- 运行次数、均值、中位数、最慢值
- 本机运行时路径与版本
- 当前 commit / 工作区状态

## 7.4 交付物

- benchmark 脚本
- `B5-B8` 缺失性能数字
- 更新后的 `backend-regression` 与专项 benchmark 记录

## 7.5 验收标准

- `backend-regression-20260307.md` 中列出的缺失性能项已全部有数字基线
- benchmark 可重复执行，不依赖手工复制粘贴步骤
- 结果文档能说明样本来源、运行口径与环境信息

## 7.6 本阶段必须补的测试/验证

- benchmark 脚本 smoke
- 样本缺失 / 运行时缺失时的错误提示验证
- 结果文件输出路径验证

## 7.7 本阶段验证命令

```bash
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\bench\run-thumbnail-benchmark.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\bench\run-normalize-benchmark.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\bench\run-playback-benchmark.ps1
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\bench\run-sidecar-benchmark.ps1
```

## 7.8 本阶段涉及文件与目录

- `scripts/bench/`
- `docs/benchmarks/backend-regression-20260307.md`
- `docs/benchmarks/`
- `config/local.paths.json`
- `docs/fixtures/`

## 7.9 本阶段完成后必须更新的 check 项

- `docs/05-MediaPlayerNext_P6收口与I1前置具体实施计划_v1.md`
  - `P6-0` 当前状态
  - `P6-0` 完成情况
  - `P6-0` 完成定义
- `docs/benchmarks/backend-regression-20260307.md`
- `docs/benchmarks/` 下的专项 benchmark 记录
- `docs/logs/<当天日期>.md`

---

## 8. P6-1：质量门禁自动化

当前状态：进行中

## 8.1 阶段目标

把 `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md` 中 `P0 / P1 / P2` 门禁的关键命令真正落成可执行脚本与统一入口，不再依赖“人记得手动跑哪些命令”。

## 8.2 范围

### 本阶段要做

1. 补 `nextest` 多轮复跑流程
2. 补 coverage 流程
3. 补 `cargo deny` / `cargo audit`
4. 补 `cargo udeps`
5. 补 duplicate deps 检查
6. 补 forbidden edges 检查
7. 补 `cargo tauri build` 打包验证
8. 提供统一 quality 执行入口与结果产物目录

### 本阶段不做

- 不强行在当前阶段把所有门禁接入 CI 平台
- 不为了门禁自动化改写业务实现

## 8.3 模块与文件计划

建议新增：

- `scripts/quality/run-rust-gates.cmd`
- `scripts/quality/run-rust-gates.ps1`
- `scripts/quality/check-forbidden-edges.mjs`
- `scripts/quality/check-duplicate-deps.ps1`
- `scripts/quality/run-coverage.ps1`
- `scripts/quality/run-release-verify.ps1`
- `docs/benchmarks/p6-quality-gates-<date>.md`

建议在根 `package.json` 或独立脚本中补：

- `check:quality`
- `check:release`

## 8.4 交付物

- 一键执行的质量门禁脚本
- 门禁结果文档 / 产物目录
- forbidden edges / duplicate deps / release build 的固定口径

## 8.5 验收标准

- `fmt / clippy / nextest / coverage / deny / audit / udeps / duplicate deps / forbidden edges / tauri build` 都有明确执行入口
- 至少一轮完整门禁执行结果已形成记录
- 失败时能快速定位到是哪一类门禁失败

当前首轮结果：

- `P0 / P1` 当前已全部可自动执行并形成 JSON / log 产物
- `duplicate deps` 已从“绝对归零”改为 baseline-delta 治理，当前基线已收敛到 `21` 个多版本 crate family

## 8.6 本阶段必须补的测试/验证

- quality 脚本 smoke
- 工具缺失 / 不可执行时的失败提示验证
- release build 产物与 sidecar/resources 校验验证

## 8.7 本阶段验证命令

```bash
scripts\quality\run-rust-gates.cmd
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\quality\run-rust-gates.ps1
```

## 8.8 本阶段涉及文件与目录

- `scripts/quality/`
- `package.json`
- `docs/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
- `docs/benchmarks/`
- `src-tauri/Cargo.toml`

## 8.9 本阶段完成后必须更新的 check 项

- `docs/05-MediaPlayerNext_P6收口与I1前置具体实施计划_v1.md`
- `docs/benchmarks/` 下的 quality gates 记录
- `docs/logs/<当天日期>.md`
- 若新增 npm scripts：
  - `package.json`
  - `README.md`

---

## 9. P6-2：错误场景补强

当前状态：已完成首轮

## 9.1 阶段目标

继续补“坏路径”而不是新能力，把最容易在 UI 接入后才暴露的问题提前固定为测试、fixture 与文档。

## 9.2 范围

### 本阶段要做

1. 补 `ffprobe / ffmpeg / mpv / node / 7z` 缺失或版本异常场景
2. 补 DB 升级 / fixture 回放异常场景
3. 补 protocol handler 找不到资源场景
4. 补 sidecar crash / timeout / bad payload 场景
5. 补对应错误码、错误消息与文档说明

### 本阶段不做

- 不把所有异常都升级成重型恢复机制
- 不在当前阶段引入复杂熔断/熔毁设计

## 9.3 模块与文件计划

优先会涉及：

- `src-tauri/src/runtime_check.rs`
- `scripts/check-runtimes.ps1`
- `src-tauri/src/lib.rs`
- `src-tauri/src/subtitle_sidecar.rs`
- `crates/media-db/tests/`
- `packages/contracts/src/errors/`
- `docs/fixtures/sidecar-fixture/`
- `docs/benchmarks/`

建议新增：

- `docs/fixtures/runtime-fixture/`
- `docs/benchmarks/p6-bad-path-validation-<date>.md`

## 9.4 交付物

- runtimes bad path tests
- DB upgrade / fixture replay bad path tests
- protocol 404 / missing resource tests
- sidecar bad payload / timeout / crash tests
- 对应文档与 fixture 样本

## 9.5 验收标准

- 关键坏路径均有稳定错误码或稳定错误消息
- 关键坏路径都有可回放测试或 fixture 样本
- UI 在 `I1` 前已不需要再猜测这些异常应该怎样表现

当前首轮结果：

- runtimes 缺失 / 非零退出 / 空输出 bad path 已有单元测试
- DB 非法 fixture / 伪装最新 schema / future schema version / 缺列 replay fixture / 缺索引 replay fixture / 缺 foreign key replay fixture / orphan row replay fixture 已固定为可回放测试
- `thumb://` / `media://` / `archive://` 缺失资源场景已覆盖稳定 `404`
- sidecar 缺失入口 / malformed payload / missing payload / timeout / retry crash 已有测试
- `packages/contracts` 与 sidecar 协议已补一轮 `AppError` bad path fixture 对齐
- Tauri command 层已从 `String` 错误收口到统一 `AppError`

## 9.6 本阶段必须补的测试/验证

- runtime smoke bad path tests
- migration / upgrade failure tests
- protocol missing resource tests
- sidecar bad payload tests

## 9.7 本阶段验证命令

```bash
scripts/run-cargo-with-msvc.cmd test --workspace
npm run check
```

## 9.8 本阶段完成后必须更新的 check 项

- `docs/05-MediaPlayerNext_P6收口与I1前置具体实施计划_v1.md`
- `docs/benchmarks/` 下的 bad path validation 记录
- `docs/fixtures/` 下对应异常样本说明
- `docs/logs/<当天日期>.md`

---

## 10. P6-3：接口收口文档（为 I1 做准备）

当前状态：进行中

## 10.1 阶段目标

把 `I1` 前真正需要的前后端边界明确写成执行文档，确保前端接入时不是“看代码猜接口”，而是围绕稳定文档与 contracts 建 `MediaRepository`。

## 10.2 范围

### 本阶段要做

1. 先建不依赖旧 UI 最终定义的 `window.* -> 新仓 command/channel/protocol` 后端边界版映射表
2. 先建不依赖页面结构的 `MediaRepository` 方法清单骨架
3. 先建 transport 边界文档：哪些能力走 `command`，哪些走 `channel`，哪些直接走协议 URL
4. 把页面依赖的 DTO / URL / command 矩阵标记为 `P6-3` 补充件，等待旧仓 UI 定义收口后再补

### 本阶段不做

- 不直接开始写 React 页面
- 不补不存在的后端能力，只记录当前已具备与缺失能力
- 不在旧仓 UI 定义尚未收口时，硬写页面级精确映射与页面 DTO 依赖矩阵

## 10.3 模块与文件计划

建议新增：

- `docs/contracts/window-to-tauri-mapping.md`
- `docs/contracts/media-repository-surface.md`
- `docs/contracts/transport-boundary.md`
- `docs/contracts/i1-contract-gap-checklist.md`
- `docs/contracts/i1-library-scan-contract-draft.md`
- `docs/contracts/i1-items-archive-thumbnail-contract-draft.md`

补充件（等待旧仓 UI 定义收口后再补）：

- `docs/contracts/ui-dependency-matrix.md`

## 10.4 交付物

- 映射表
- `MediaRepository` 方法面文档
- transport 边界文档

补充件：

- 页面依赖矩阵（后补）

## 10.5 验收标准

- 前端在 `I1` 时已经可以只靠文档与 contracts 写 repository 抽象
- 能清楚区分 `command` / `channel` / `thumb://` / `media://` / `archive://` 的使用边界
- 已形成首版 repository / adapter 边界文档，不再要求前端先读 Rust 代码猜接法

补充验收（等待旧仓 UI 定义收口后再补）：

- 能清楚看到哪些页面依赖哪些 DTO / URL / command

## 10.6 本阶段必须补的测试/验证

- 文档与现有 command / protocol 对照复核
- 与 `packages/contracts/src/index.ts` 的导出项交叉检查
- 与 `apps/desktop/src/App.tsx` 当前最小桥接状态交叉检查，避免文档提前假设不存在的 UI 能力

## 10.7 本阶段验证命令

```bash
npm run test --workspace @mediaplayernext/contracts
scripts/run-cargo-with-msvc.cmd test --workspace
```

## 10.8 本阶段完成后必须更新的 check 项

- `docs/05-MediaPlayerNext_P6收口与I1前置具体实施计划_v1.md`
- `docs/contracts/`
- `docs/logs/<当天日期>.md`

---

## 11. P6-4：发布前资源路径策略收口

当前状态：未开始

## 11.1 阶段目标

把 `node`、`7z`、sidecar、migrations、runtime binaries 的开发态 / 打包态 / 本地 override 查找顺序收口成统一策略与校验脚本，避免 UI 接入后才首次暴露打包路径问题。

## 11.2 范围

### 本阶段要做

1. 明确 `node` 的开发态 / 打包态 / 本地 override 查找顺序
2. 明确 `7z` 的开发态 / 打包态 / 本地 override 查找顺序
3. 明确 subtitle sidecar 入口的开发态 / 打包态查找顺序
4. 明确 migrations、runtime binaries 的开发态 / 打包态查找顺序
5. 建立对应校验脚本与策略文档

### 本阶段不做

- 不在当前阶段完成最终生产打包资源清单
- 不引入与 Windows 首发无关的跨平台资源策略

## 11.3 模块与文件计划

建议新增：

- `docs/runtime/resource-path-strategy.md`
- `scripts/release/verify-resource-paths.ps1`
- `scripts/release/verify-sidecar-package.ps1`

优先会涉及：

- `config/local.paths.example.json`
- `README.md`
- `src-tauri/src/subtitle_sidecar.rs`
- `scripts/check-runtimes.ps1`
- `src-tauri/Cargo.toml`

## 11.4 交付物

- 资源路径策略文档
- 资源路径校验脚本
- README 与 example config 同步说明

## 11.5 验收标准

- 开发态 / 打包态 / 本地 override 的查找顺序清晰可查
- sidecar、runtimes、migrations 的路径策略不再散落在多个文件里靠隐式约定维护
- 至少一轮打包态校验或模拟验证已形成记录

## 11.6 本阶段必须补的测试/验证

- 资源路径脚本 smoke
- 打包验证 smoke
- 运行时缺失时的失败提示验证

## 11.7 本阶段验证命令

```bash
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\release\verify-resource-paths.ps1
scripts/run-cargo-with-msvc.cmd tauri build
```

## 11.8 本阶段完成后必须更新的 check 项

- `docs/05-MediaPlayerNext_P6收口与I1前置具体实施计划_v1.md`
- `docs/runtime/resource-path-strategy.md`
- `README.md`
- `config/local.paths.example.json`
- `docs/logs/<当天日期>.md`

---

## 12. P6-5：可观测性与 contracts 再收紧

当前状态：未开始

## 12.1 阶段目标

把日志字段、外部进程日志格式与缺失域 contracts 进一步统一，让后续 `I1` 的 repository 与页面状态层可以稳定依赖类型、错误与日志上下文。

## 12.2 范围

### 本阶段要做

1. 让 `task id / asset id / source id / session id` 尽量贯穿日志
2. 统一外部进程 `command line / duration / exit code` 日志格式
3. 把还缺的域合同继续补齐到 `packages/contracts`
4. 保证 `I1` 时前端只接 contracts，不猜后端返回结构

### 本阶段不做

- 不在当前阶段做完整日志平台接入
- 不做全量 codegen 改造

## 12.3 模块与文件计划

优先会涉及：

- `crates/app-core/`
- `crates/media-io/`
- `crates/media-playback/`
- `src-tauri/src/`
- `packages/contracts/src/index.ts`
- `packages/contracts/src/commands/`
- `packages/contracts/src/channels/`
- `packages/contracts/src/events/`
- `packages/contracts/src/models/`

建议新增或补齐：

- `commands/library.ts`
- `commands/scan.ts`
- `commands/items.ts`
- `commands/archive.ts`
- `commands/thumbnail.ts`
- `channels/scan-progress.ts`
- `channels/thumbnail-progress.ts`
- `events/library-events.ts`

## 12.4 交付物

- 统一日志字段口径
- 外部进程日志格式约定
- 补齐后的缺失域 contracts

## 12.5 验收标准

- 关键链路日志中能关联 `task/source/asset/session`
- 外部进程日志具备统一字段，不再各模块各写各的
- `packages/contracts` 已覆盖 `I1` 所需主域，不需要前端继续猜 DTO

## 12.6 本阶段必须补的测试/验证

- contracts zod / fixture tests
- 日志字段 smoke（可从测试输出与最小验证脚本开始）
- 外部进程日志格式复核

## 12.7 本阶段验证命令

```bash
npm run test --workspace @mediaplayernext/contracts
scripts/run-cargo-with-msvc.cmd test --workspace
npm run check
```

## 12.8 本阶段完成后必须更新的 check 项

- `docs/05-MediaPlayerNext_P6收口与I1前置具体实施计划_v1.md`
- `packages/contracts/src/index.ts` 与对应 `tests` / `fixtures`
- `docs/logs/<当天日期>.md`
- `docs/benchmarks/` 下的 contracts / observability 验证记录

---

## 13. 阶段间依赖关系

### `P6-0 -> I1`

- 没有真实性能基线，前端接入后很难区分是页面层慢还是后端本身已慢

### `P6-1 -> I1`

- 没有门禁自动化，前端接入开始后容易在更频繁的改动里把既有后端能力悄悄破坏

### `P6-2 -> I1`

- 没有 bad path 基线，前端接线时会被迫临场定义错误行为

### `P6-3 -> I1`

- 没有接口收口文档，`MediaRepository` 很容易退化成“直接看代码猜命令”

### `P6-4 -> I1/I7`

- 没有路径策略收口，开发态可用并不代表打包态可用

### `P6-5 -> I1`

- 没有 contracts 与可观测性再收紧，后续 UI 状态与错误诊断都容易出现漂移

因此当前阶段不建议跳过 `P6` 直接大规模开始 `I1` 页面接入。

---

## 14. fixture、golden、benchmark 计划

## 14.1 fixture 目录建议

```text
docs/fixtures/
  thumbnail-fixture/
  archive-fixture/
  playback-fixture/
  sidecar-fixture/
  runtime-fixture/
```

### `runtime-fixture`

- runtimes 缺失 / 版本异常样本说明
- protocol 资源缺失样本说明
- DB upgrade / replay 异常样本说明
- 当前完成情况：未开始

## 14.2 当前阶段要产出的记录

- 真实性能 benchmark 记录
- quality gates 记录
- bad path validation 记录
- resource path strategy / release verify 记录
- contracts / observability 验证记录

## 14.3 benchmark / validation 计划

`P6` 至少记录以下基线：

- 缩略图冷 / 热命中耗时
- zip 内页缩略图耗时
- `7z` 归一化耗时
- `ffprobe` / `ffmpeg` / `mpv` 耗时
- sidecar `ping / health / restart` 耗时
- quality gates 统一执行记录
- bad path 统一验证记录

---

## 15. 与 `src-tauri` 的关系

在 `P6` 阶段，`src-tauri` 仍然承担接线与宿主职责，但重点从“新增能力”转为“验证、bad path、资源路径与可观测性收口”。

允许的改动：

- 维持与扩展 runtime checks
- 维持与扩展 custom protocol / sidecar 的 bad path 测试
- 补充资源路径与打包态查找顺序接线
- 补充外部进程日志字段

不允许的改动：

- 为了做 benchmark 或 bad path 而顺手增加新的业务实现
- 在 protocol handler 里开始写页面级逻辑
- 把前端尚未定义的 UI 状态强行塞进宿主层

`src-tauri` 在当前阶段的职责是：

- 为 `I1` 前提供稳定、可验证、可打包的宿主边界
- 不成为前端页面接入逻辑的替代层

---

## 16. 每阶段完成定义（Definition of Done）

## `P6-0` 完成定义

- `backend-regression` 列出的缺失性能项已全部有基线数字
- benchmark 脚本可重复执行

当前状态：已完成首轮

## `P6-1` 完成定义

- 关键质量门禁已落成统一脚本 / 流程
- 至少一轮完整门禁记录已形成

当前状态：未开始

## `P6-2` 完成定义

- 关键 runtimes / DB / protocol / sidecar bad path 已有测试与文档

当前状态：未开始

## `P6-3` 完成定义

- 已形成 `window.*` 映射表、`MediaRepository` 方法清单、DTO 依赖矩阵与传输边界文档

当前状态：未开始

## `P6-4` 完成定义

- 资源路径策略文档与校验脚本已具备
- 开发态 / 打包态查找顺序已明确

当前状态：未开始

## `P6-5` 完成定义

- 关键日志字段已统一
- 外部进程日志格式已统一
- `I1` 所需缺失域 contracts 已补齐

当前状态：未开始

---

## 17. 当前建议的实际执行顺序

如果从当前仓库立即继续开工，建议严格按以下顺序落地：

### 第 13 批：`P6-0`

1. 补 benchmark 脚本
2. 跑真实性能样本
3. 把数字回写到 benchmark 文档

### 第 14 批：`P6-1`

1. 建质量门禁脚本
2. 补 forbidden edges / duplicate deps / release verify
3. 固化一轮完整门禁记录

### 第 15 批：`P6-2`

1. 补 runtimes bad path
2. 补 DB / protocol / sidecar bad path
3. 形成统一 validation 记录

### 第 16 批：`P6-3`

1. 建 `window.*` 映射表
2. 建 `MediaRepository` 方法清单
3. 建 DTO / URL / command 依赖矩阵

### 第 17 批：`P6-4`

1. 建资源路径策略文档
2. 建资源路径与打包校验脚本
3. 同步 README 与 example config

### 第 18 批：`P6-5`

1. 统一日志字段
2. 统一外部进程日志格式
3. 补齐 `I1` 所需缺失域 contracts

完成以上六批后，再进入 `I1` 的 repository / adapter 实施。

---

## 18. 最终执行结论

当前仓库在 `B1-B8` 已完成首版、`I1-I4` 也已规划完成后，继续往前的正确落地方式不是：

- 立即开始大规模 UI 页面迁移
- 立即开始播放器或字幕 UI
- 让前端在缺少 benchmark、门禁、路径策略与接口文档的前提下边接边猜

而是：

1. 先完成 `P6-0` 真实性能基线
2. 再完成 `P6-1` 质量门禁自动化
3. 再完成 `P6-2` 错误场景补强
4. 再完成 `P6-3` 接口收口文档
5. 再完成 `P6-4` 资源路径策略收口
6. 再完成 `P6-5` 可观测性与 contracts 收紧

完成这六步后，后续 `I1` 才会真正具备：

- 建立在真实 benchmark 之上的接入判断能力
- 建立在自动化门禁之上的回归保护能力
- 建立在明确 bad path 与路径策略之上的稳定宿主基础
- 建立在稳定 contracts 与接口文档之上的 repository / adapter 收口能力
