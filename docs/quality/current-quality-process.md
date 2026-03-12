# MediaPlayerNext 当前质量流程

## 1. 文档定位

本文件用于把 `docs/archive/root-plans/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md` 收口成当前仓库正在执行的质量流程入口。

当前目标不是重写一份新的历史方案，而是明确三件事：

- 当前仓库已经真正落地了哪些门禁
- 当前默认应该按什么顺序执行这些门禁
- 还有哪些关键缺口仍未进入强制流程

本文件是当前主动维护版本；历史 Rust/Tauri 审核方案继续留在归档目录作为来源参考。

当前分层质量流水线的执行型计划文档：

- `docs/quality/layered-quality-pipeline-phase-plan.md`

## 2. 当前适用范围

本仓不是纯 Rust 后端仓，当前质量流程同时覆盖：

- Rust workspace：`crates/*`、`src-tauri`
- 桌面前端：`apps/desktop`
- 外部契约：`packages/contracts`
- 桌面自动化验收：`tests/desktop-e2e`
- 打包与资源完整性：`scripts/quality/*`、`scripts/release/*`

因此当前流程不只看 `cargo fmt / clippy / test`，还要同时约束：

- contract fixture 与错误语义
- SQLite migration 与运行时路径回放
- Tauri host / sidecar / resources / capabilities
- 桌面闭环 E2E
- benchmark 与重复依赖这类 delta 治理项

## 3. 设计原则

- 继续保留 `P0 / P1 / P2` 分级门禁
- 继续保留 `Go / No-Go` 的发布口径
- 继续坚持 delta 管理，优先防止新增债务
- 优先让脚本与结果产物稳定可复跑，而不是把所有治理问题一次清零
- 文档、fixture、脚本与代码必须同批更新，不能让仓库处于“门禁已变、文档没变”的状态

## 4. 当前质量门禁分层

### 4.1 P0：发布阻断

P0 当前用于阻断“代码无法稳定构建、测试或通过核心安全检查”的问题。

当前已落地入口：

- `npm run check:quality`
  - 对应 `scripts/quality/run-rust-gates.ps1`
  - 当前默认对应 `standard` 层，会执行：
    - `cargo fmt --all --check`
    - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
    - `cargo check --workspace --all-targets --locked`
    - `cargo nextest run --workspace --all-features` 单轮
    - `debt-delta`
    - `duplicate-deps`
    - `forbidden-edges`
- `npm run check:quality:heavy`
  - 对应重层全量门禁，会追加：
    - `cargo nextest run --workspace --all-features` 三轮复跑
    - `cargo llvm-cov` 覆盖率
    - `cargo deny check advisories licenses bans sources`
    - `cargo audit`
    - `cargo +nightly udeps --workspace --all-targets`
- `npm run check:quality:legacy`
  - 保留旧全量链路（兼容入口）
  - 仍包含 `tauri-build` release verify gate
  - `Phase 2` 起脚本已支持分层参数：`-Layer fast|standard|heavy|legacy`
- `npm run test:contracts`
  - 校验 `packages/contracts` 中的 zod schema、fixture 与错误结构
- `npm run build:web`
  - 当前前端改动的最小构建验证
- `npm run check`
  - 当前运行时依赖检查入口，覆盖 `sharp / rusqlite / ffmpeg / mpv`

当前已纳入 P0 口径的仓库能力：

- Rust workspace 基础格式、lint、编译、覆盖率与安全门禁
- `packages/contracts` 的 fixture + schema 校验
- `crates/media-db` 中的 migration 与坏 fixture 集成测试
- `tests/desktop-e2e` 的最小桌面闭环能力已经可运行，但是否纳入每轮必跑仍按变更类型触发

### 4.2 P1：高风险回归

P1 当前用于阻断“构建虽能通过，但宿主边界、资源打包或桌面闭环明显漂移”的问题。

当前已落地入口：

- `npm run check:quality`
  - 默认 `standard` 层已包含 `forbidden-edges`
- `npm run check:quality:legacy`
  - 兼容层包含 `tauri-build`（release verify）
- `npm run check:release`
  - 对应 `scripts/quality/run-release-verify.ps1`
  - 当前会执行：
    - subtitle sidecar build
    - `scripts/run-tauri-build.cmd`
    - sidecar package verify
    - bundle / capability / icon / resource 存在性检查
- `npm run e2e:desktop:doctor`
  - 校验 `tauri-driver` 与 `msedgedriver` 前置条件
- `npm run e2e:desktop`
  - 当前桌面闭环自动验收入口

当前已纳入 P1 口径的仓库能力：

- workspace forbidden edges 校验：`scripts/quality/check-forbidden-edges.mjs`
- Tauri 打包、sidecar 资源、capability 文件存在性校验
- 数据库管理桌面 E2E：读取路径、目录切换、清除数据库确认流、运行时路径回落
- Tauri 自动化 E2E 基线：`tauri-driver + WebdriverIO`

### 4.3 P2：治理门禁

P2 当前用于暴露“不会立刻打断功能，但会持续侵蚀可维护性”的问题。

当前已落地入口：

- `npm run check:quality`
  - 已包含 `debt-delta baseline-delta`
  - 已包含 `duplicate deps baseline-delta`
- `npm run check:quality:heavy`
  - 已包含 `udeps`
- `npm run check:debt`
  - 对应 `scripts/quality/check-debt-delta.ps1`
- `docs/benchmarks/`
  - 当前用于沉淀 benchmark baseline、回归记录与质量门禁记录

当前已纳入 P2 口径的仓库能力：

- Rust debt baseline 治理：`config/quality/debt-baseline.json`
- duplicate deps 基线治理：`config/quality/duplicate-deps-baseline.json`
- `cargo +nightly udeps` 未使用依赖检查
- benchmark baseline 文档化沉淀
- fixture 目录分层与 replay 样本约定

## 5. 当前默认执行入口

### 5.1 日常最小入口

- 前端改动：`npm run build:web`
- 运行时 / Rust 改动：`npm run check`
- 契约改动：`npm run test:contracts`
- 桌面交互闭环改动：`npm run e2e:desktop` 或定向 `--spec`

### 5.2 当前统一质量入口

- Rust 质量门禁：`npm run check:quality`
- 发布级资源与打包校验：`npm run check:release`
- 桌面自动化前置检查：`npm run e2e:desktop:doctor`

补充说明：

- 当前 `npm run check:quality` 已切到 `standard` 层（`Phase 3` 已落地）
- 当前可直接执行分层命令：
  - `npm run check:quality:fast`
  - `npm run check:quality:standard`
  - `npm run check:quality:heavy`
  - `npm run check:quality:release`
  - `npm run check:quality:legacy`

### 5.3 结果产物目录

- Rust 质量门禁结果：`data/quality-gates/<timestamp>/rust-gates` 或 `rust-gates-<layer>`
- release verify 结果：`data/quality-gates/<timestamp>/release-verify`

分层脚本产物中，`quality-gates-summary.json` 会包含 `layer` 字段。

当前要求：执行质量门禁时，优先以脚本产物为准，不再以“人工记忆这一轮跑过哪些命令”为准。

### 5.4 分层入口推荐

- `fast`：本地日常快反馈，优先用于改动中的频繁自检
  - 命令：`npm run check:quality:fast`
- `standard`：提交前默认入口，平衡覆盖与耗时
  - 命令：`npm run check:quality` 或 `npm run check:quality:standard`
- `heavy`：高成本全量质量检查
  - 命令：`npm run check:quality:heavy`
- `release`：发布级链路（heavy + release verify + desktop e2e doctor + desktop e2e）
  - 命令：`npm run check:quality:release`
- `legacy`：旧入口兼容（不建议日常使用）
  - 命令：`npm run check:quality:legacy`

### 5.5 最新分层验证快照（2026-03-12 11:48）

- `fast`：通过
  - `data/quality-gates/20260312-101605/rust-gates-fast/quality-gates-summary.json`
- `standard`：通过
  - `data/quality-gates/20260312-102235/rust-gates-standard/quality-gates-summary.json`
- `heavy`：通过
  - `data/quality-gates/20260312-114524/rust-gates-heavy/quality-gates-summary.json`
- `release`：通过
  - `check:release` 通过：`data/quality-gates/20260312-114637/release-verify/release-verify-summary.json`
  - `desktop-e2e` 通过：`2 specs / 2 passed`

说明：

- 本轮已修复 `forbidden-edges` 元数据解析问题，`fast` 恢复通过
- 本轮已收敛 `standard` 与 `heavy` 层阻塞项
- `release` 相关阻塞已解除；当前分层命令可完整跑通

## 6. 变更类型与必跑动作

### 6.1 纯内部实现变更

- 必跑：相关单元/集成测试
- Rust 相关至少补：`npm run check:quality:fast`
- 若涉及前端：`npm run build:web`
- 若修改范围跨多个 crate 或存在较大重构：升级到 `npm run check:quality:standard`

### 6.2 契约变更

契约范围包括：`packages/contracts`、Tauri command 参数/返回、错误码、事件结构、sidecar I/O 语义。

- 必跑：`npm run test:contracts`
- 必跑：`npm run check:quality:standard`
- 若前端适配器有改动：`npm run build:web`
- 若桌面闭环有变化：补对应 `npm run e2e:desktop -- --spec=...`
- 必须同步更新：相关 contract 文档、fixture、错误语义说明

### 6.3 数据迁移变更

迁移范围包括：SQLite schema、runtime storage 结构、路径回落语义、数据库清除后默认值恢复。

- 必跑：Rust migration 相关测试
- 必跑：`npm run check:quality:standard`
- 若涉及设置页/路径管理闭环：`npm run e2e:desktop -- --spec=./specs/settings-database.e2e.mjs`
- 必须同步更新：`docs/runtime/` 下对应策略文档与当日日志

### 6.4 高频路径变更

高频路径包括：scan / thumbnail / archive / playback / sidecar。

- 必跑：`npm run check:quality:standard`
- 必须补 benchmark 或 validation 记录到 `docs/benchmarks/`
- 当前至少要求记录 baseline、样本口径、结果变化与风险判断
- 若涉及性能基线或回归判断：升级到 `npm run check:quality:heavy`
- 若变更影响桌面观感或状态流，还要补最小 UI / E2E 验证

### 6.5 安全 / 权限 / 宿主边界变更

范围包括：capabilities、bundle resources、external binaries、sidecar 打包、resource path。

- 必跑：`npm run check:quality:heavy`
- 必跑：`npm run check:release`
- 若改动影响桌面闭环：补最小 `desktop-e2e`
- 必须同步更新：相关 runtime / testing / contracts 文档

### 6.6 发布前收口

- 必跑：`npm run check:quality:release`
- 若发布范围不涉及桌面交互闭环，可评估降级为：
  - `npm run check:quality:heavy`
  - `npm run check:release`
- 发布说明必须引用本轮质量产物目录，避免口头结论

## 7. 当前已明确落地的能力

### 7.1 契约与错误语义

- 当前对外契约主入口是 `packages/contracts`
- 已有 zod schema 测试与 sample fixtures
- 已有稳定错误结构，不再直接以裸字符串充当跨边界错误

### 7.2 迁移与 fixture replay

- `crates/media-db` 已有空库初始化、`N-1 -> N` 升级、坏 fixture 失败用例
- runtime storage 与数据库目录切换已纳入桌面 E2E 验收

### 7.3 架构边界与发布验证

- 已有 forbidden edges 脚本
- 已有 Tauri build、sidecar package、resource path 验证脚本
- 已有 `data/quality-gates` 统一结果产物目录

### 7.4 Rust debt delta

- 已有 `scripts/quality/check-debt-delta.ps1`
- 当前扫描范围固定为 `crates/**/*.rs` 与 `src-tauri/**/*.rs`，并排除 `target`
- 当前按 baseline-delta 管理：
  - `unsafe`
  - `unwrap(`
  - `expect(`
  - `todo!`
  - `unimplemented!`
  - `panic!`
  - `#[allow(...)]`
  - `dbg!`
- 当前基线文件：`config/quality/debt-baseline.json`

### 7.5 桌面闭环验收

- 已固定 `tauri-driver + WebdriverIO`
- 已接入数据库管理主用例与 shell smoke 用例
- 当前相关桌面任务完成后，应提示是否继续执行自动 E2E 验收

## 8. 当前仍未收口的关键缺口

以下项目已经明确是缺口，但尚未形成当前仓库的强制门禁：

- 没有 CI / PR 模板 / release workflow 真正落库
- 没有 capabilities drift 校验，当前只有存在性与打包完整性检查
- 没有 contract drift 校验，当前 `packages/contracts` 与 Rust command 仍存在手工漂移风险
- benchmark 还没有 compare + threshold gate
- 还没有 binary size / `cargo bloat` / Go-No-Go 报告模板的当前版落地
- 还没有离线安装、升级回放、签名等 release 级闭环

## 9. 当前阶段推进顺序

分层质量流水线的具体 phase 推进与 check 回填，以 `docs/quality/layered-quality-pipeline-phase-plan.md` 为准。

### 9.1 第一阶段：统一口径

- 建立 `docs/quality/current-quality-process.md`
- 把 `docs/README.md` 中的 quality 入口补齐
- 让代理和开发者都以当前版文档为统一入口，而不是继续直接翻历史归档

### 9.2 第二阶段：补强制门禁

优先补以下脚本化能力：

- debt-delta
- capabilities drift
- contract drift
- 独立 migration gate

### 9.3 第三阶段：补发布级闭环

- CI workflow
- PR 模板
- benchmark compare + threshold
- binary size gate
- Go / No-Go 报告模板
- release 安装、升级、签名与离线 smoke

## 10. 当前执行结论

当前仓库已经不是“只有历史方案，没有实际流程”的状态。

更准确的判断是：

- P0 / P1 的首轮脚本化地基已经落地
- 契约、migration、release verify、desktop E2E 都已经各自形成最小闭环
- 当前最大的缺口是“统一当前文档入口”和“少数尚未脚本化的强制门禁”

因此从本文件开始，后续质量流程变更应优先更新当前文档，而不是继续只追加到归档计划里。
