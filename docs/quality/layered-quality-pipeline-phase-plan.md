# 分层质量流水线实施计划

## 1. 文档定位

本文件用于把 `MediaPlayerNext` 当前已经存在的质量门禁，进一步收口成一套可执行的分层质量流水线（layered quality pipeline）。

本文件是执行型计划文档，按 phase 拆分工作；每个 phase 都明确：

- 开始前必须读取的文件
- todo 顺序
- 涉及文件
- 具体要做的内容
- 完成后需要回填的状态 check

本文件本身应包含在新对话中继续推进该任务所需的最小上下文，不依赖额外口头补充。

关联当前入口文档：

- `docs/quality/current-quality-process.md`

## 2. 为什么现在要做这件事

当前仓库已经不是“没有质量门禁”的状态，而是“门禁已经不少，但全量执行太重”。

截至 `data/quality-gates/20260312-074725/rust-gates/quality-gates-summary.json`，当前一次全量 `check:quality` 的重项包括：

- `tauri-build`：约 `160s`
- `coverage`：约 `82s`
- `udeps`：约 `64s`
- `clippy`：约 `34s`
- `check`：约 `28s`
- `nextest x3` 聚合：约 `22s`

这说明当前瓶颈不是单个测试文件，而是把多类重型 gate 串成了一条默认本地链路。

参考来源：

- `Z:\Playground\CurrentWorking\MediaPlayerX\修复方案.md`

上一次旧仓修复的直接对象是前端测试体系，但其中最有价值的方法论在当前仓库仍然成立：

- 先看真实耗时排名，不凭感觉优化
- 优先识别“结构性重项”而不是平均用力
- 把“每次都跑全量”改成“按层级、按触发条件运行”

因此本任务的目标不是简单再加脚本，而是把当前质量链路重构成分层流水线。

## 3. 当前范围与边界

### 3.1 本批要做的内容

- 冻结分层质量流水线的层级语义
- 冻结每层默认包含的 gate 范围
- 冻结默认命令命名与职责边界
- 明确哪些重门禁不再属于日常默认反馈链路
- 形成后续脚本拆分、命令调整、文档回填的 phase 计划

### 3.2 本批明确不做的内容

- 本文件阶段不直接重写全部现有 PowerShell 质量脚本
- 本文件阶段不直接落 CI workflow
- 本文件阶段不直接清理 `clippy` / `deny` / `duplicate-deps` 既有失败项
- 本文件阶段不直接调整桌面 E2E 用例结构

## 4. 当前相关现状

### 4.1 当前已存在入口

- `npm run check:quality`
  - 当前对应 `scripts/quality/run-rust-gates.ps1`
  - 仍是全量重型入口
- `npm run check:release`
  - 当前对应 `scripts/quality/run-release-verify.ps1`
- `npm run check:debt`
  - 当前对应 `scripts/quality/check-debt-delta.ps1`
- `npm run test:contracts`
- `npm run e2e:desktop`
- `npm run e2e:desktop:doctor`

### 4.2 当前核心问题

- `check:quality` 仍把快反馈与重门禁绑在一起
- `coverage`、`udeps`、`tauri-build` 这类重项没有被单独抬升到高层流水线
- 当前文档已经写了“按变更类型触发”，但默认命令结构还没有完全配合这个目标
- 新对话启动后，代理仍然容易直接跑最重入口，而不是先选合适层级

## 5. 分层质量流水线的固定语义（本计划冻结版）

### 5.1 `fast`

定位：本地日常快反馈，目标是尽快发现语法、编译、局部债务和明显边界错误。

固定原则：

- 不包含覆盖率
- 不包含 `udeps`
- 不包含 release build
- 不包含全量桌面 E2E

建议包含：

- `fmt --check`
- `cargo check`
- `debt-delta`
- `forbidden-edges`

### 5.2 `standard`

定位：本地提交前或 PR 前默认质量入口，覆盖主要功能正确性与静态检查，但仍避免最重项。

固定原则：

- 作为后续默认 `check:quality` 的候选层
- 覆盖编译、lint、主测试链路与主要 P1/P2 边界
- 不默认带 `coverage` / `udeps` / release build

建议包含：

- `fast` 全部内容
- `clippy`
- `nextest` 单轮
- `duplicate-deps`
- 必要时补 `test:contracts`

### 5.3 `heavy`

定位：手动全量质量检查、主分支 nightly 或阶段性收口使用。

固定原则：

- 承担当前最重的真实性质量门禁
- 不要求成为每次本地修改后的默认入口

建议包含：

- `standard` 全部内容
- `nextest` 三轮 flaky replay
- `coverage`
- `deny`
- `audit`
- `udeps`

### 5.4 `release`

定位：发布前或宿主边界变更后的高成本验收链路。

固定原则：

- 明确包含打包、资源、sidecar、能力文件、桌面闭环验证
- 后续可继续接入安装/升级/签名与离线 smoke

建议包含：

- `heavy` 的必要子集
- `check:release`
- 桌面 E2E doctor
- 关键桌面 E2E

## 6. 后续命令口径（建议冻结）

建议后续统一为：

- `npm run check:quality:fast`
- `npm run check:quality:standard`
- `npm run check:quality:heavy`
- `npm run check:quality:release`

建议默认别名：

- `npm run check:quality` -> `npm run check:quality:standard`

保留现有直连入口：

- `npm run check:debt`
- `npm run check:release`
- `npm run test:contracts`
- `npm run e2e:desktop`

## 7. 新对话启动时的最小上下文

如果在新对话里继续推进本计划，应先接受以下上下文为当前事实：

1. 当前质量总入口文档是：`docs/quality/current-quality-process.md`
2. 当前分层流水线实施文档是：`docs/quality/layered-quality-pipeline-phase-plan.md`
3. 当前全量 Rust 质量入口脚本是：`scripts/quality/run-rust-gates.ps1`
4. 当前 release 校验入口脚本是：`scripts/quality/run-release-verify.ps1`
5. 当前已落地的 P2 delta 门禁包括：
   - `scripts/quality/check-debt-delta.ps1`
   - `scripts/quality/check-duplicate-deps.ps1`
6. 当前 `check:quality` 的主要重项时长，以上述 `20260312-074725` 产物为准
7. 当前目标不是再新增一个“总文档”，而是把命令、脚本和文档入口真正分层

## 8. 建议文件落点

- `docs/quality/layered-quality-pipeline-phase-plan.md`
  - 本实施文档，执行中持续回填
- `docs/quality/current-quality-process.md`
  - 当前质量入口文档，需要在分层命令落地后同步改写
- `docs/README.md`
  - 补充质量实施文档入口
- `package.json`
  - 新增分层命令
- `scripts/quality/run-rust-gates.ps1`
  - 后续从“全量重入口”拆成可组合层级
- `docs/logs/<当天日期>.md`
  - 持续回填推进状态

## 9. Phase 总览

| Phase | 目标 | 状态 |
|---|---|---|
| `Phase 0` | 冻结分层语义并建立实施文档 | `done` |
| `Phase 1` | 冻结命令层级与 gate 归属表 | `done` |
| `Phase 2` | 重构质量脚本为可组合层级 | `done` |
| `Phase 3` | 落库分层命令与默认别名 | `done` |
| `Phase 4` | 同步触发策略、文档入口与结果产物说明 | `done` |
| `Phase 5` | 验证、日志、文档回填 | `done` |

---

## 10. Phase 0：冻结分层语义并建立实施文档

### 10.1 开始前读取文件

- `docs/quality/current-quality-process.md`
- `docs/README.md`
- `Z:\Playground\CurrentWorking\MediaPlayerX\修复方案.md`
- `data/quality-gates/20260312-074725/rust-gates/quality-gates-summary.json`

### 10.2 todo 顺序

1. 冻结分层流水线的目标与不做项
2. 冻结 `fast / standard / heavy / release` 语义
3. 冻结建议命令命名
4. 把本计划文档加入 docs 索引与质量入口文档

### 10.3 要做的内容

- 把本文件作为分层质量流水线的执行入口
- 明确当前要解决的问题是“全量默认链路过重”，不是“门禁数量不够”
- 明确后续对话应先读本文件再推进，不再从历史归档重新拼背景

### 10.4 phase 完成后状态 check

- [x] 分层流水线目标、边界与不做项已冻结
- [x] `fast / standard / heavy / release` 语义已写清楚
- [x] docs 索引可找到本计划文档
- [x] 当前质量入口文档已能链接到本计划文档

---

## 11. Phase 1：冻结命令层级与 gate 归属表

### 11.1 开始前读取文件

- `docs/quality/layered-quality-pipeline-phase-plan.md`
- `docs/quality/current-quality-process.md`
- `scripts/quality/run-rust-gates.ps1`
- `scripts/quality/run-release-verify.ps1`
- `package.json`

### 11.2 todo 顺序

1. 列出当前所有 gate 与直连入口
2. 给每个 gate 标记推荐层级：`fast / standard / heavy / release`
3. 冻结默认别名与兼容关系
4. 冻结哪些 gate 仍保留独立直连命令

### 11.3 要做的内容

至少要形成一张 gate 归属表，覆盖：

- `fmt`
- `clippy`
- `check`
- `nextest x1`
- `nextest x3`
- `coverage`
- `deny`
- `audit`
- `udeps`
- `debt-delta`
- `duplicate-deps`
- `forbidden-edges`
- `check:release`
- `test:contracts`
- `e2e:desktop:doctor`
- `e2e:desktop`

同时要明确：

- `check:quality` 后续默认落到哪一层
- 哪些只在 `heavy` 或 `release` 层运行
- 哪些任务仍适合保留单独入口

本轮冻结结果如下：

| gate / 入口 | fast | standard | heavy | release | legacy 兼容 | 说明 |
|---|---|---|---|---|---|---|
| `fmt` | ✓ | ✓ | ✓ | ✓ | ✓ | 基础格式门禁 |
| `check` | ✓ | ✓ | ✓ | ✓ | ✓ | 编译级基础反馈 |
| `clippy` |  | ✓ | ✓ | ✓ | ✓ | 放到标准层起步 |
| `nextest x1` |  | ✓ |  | ✓ |  | 标准层单轮测试 |
| `nextest x3` |  |  | ✓ | ✓ | ✓ | 重层稳定性复跑 |
| `debt-delta` | ✓ | ✓ | ✓ | ✓ | ✓ | P2 delta 防回退 |
| `duplicate-deps` |  | ✓ | ✓ | ✓ | ✓ | 标准层即可暴露 |
| `forbidden-edges` | ✓ | ✓ | ✓ | ✓ | ✓ | 架构边界快速反馈 |
| `coverage` |  |  | ✓ | ✓ | ✓ | 重层执行 |
| `deny` |  |  | ✓ | ✓ | ✓ | 重层执行 |
| `audit` |  |  | ✓ | ✓ | ✓ | 重层执行 |
| `udeps` |  |  | ✓ | ✓ | ✓ | 重层执行 |
| `check:release` |  |  |  | ✓ | ✓ | 宿主打包与资源校验 |
| `test:contracts` |  | ✓ | ✓ | ✓ |  | 仍保留直连入口 |
| `e2e:desktop:doctor` |  |  |  | ✓ |  | 发布层前置校验 |
| `e2e:desktop` |  |  |  | ✓ |  | 发布层关键闭环 |

默认别名冻结结果：

- 目标态：`check:quality` -> `standard`（在 `Phase 3` 切换）
- 兼容态：`check:quality` 暂时保持 legacy 全量入口（本轮不改 package 脚本）

保留直连入口冻结结果：

- `check:debt`
- `check:release`
- `test:contracts`
- `e2e:desktop`
- `e2e:desktop:doctor`

### 11.4 phase 完成后状态 check

- [x] 已有完整的 gate 分层归属表
- [x] 已冻结 `check:quality` 的默认目标层
- [x] 已冻结保留直连入口的清单
- [x] 文档已明确 `heavy` 与 `release` 的边界

---

## 12. Phase 2：重构质量脚本为可组合层级

### 12.1 开始前读取文件

- `docs/quality/layered-quality-pipeline-phase-plan.md`
- `scripts/quality/run-rust-gates.ps1`
- `scripts/quality/common.ps1`
- `scripts/quality/check-debt-delta.ps1`
- `scripts/quality/check-duplicate-deps.ps1`

### 12.2 todo 顺序

1. 识别 `run-rust-gates.ps1` 中可复用的 gate 片段
2. 冻结脚本拆分方式：按层拆，还是按 gate 集合参数化
3. 确保所有层级都继续输出统一 summary 与 duration
4. 保留兼容入口，避免现有命令瞬间失效

### 12.3 要做的内容

建议方向：

- 优先做参数化，而不是复制四份脚本
- 让 summary 中能显式标记当前运行层级
- 保持结果产物仍落到 `data/quality-gates/<timestamp>/...`

需要特别注意：

- `nextest x1` 与 `nextest x3` 需要有明确分离
- `coverage` 不应继续默认绑定在快反馈链路
- `tauri-build` / release verify 不应混在普通 Rust gate 中默认执行

本轮落地结果：

- 已选择“按 gate 集合参数化”方案，而不是复制多份脚本
- `scripts/quality/run-rust-gates.ps1` 新增 `-Layer` 参数：
  - `legacy`
  - `fast`
  - `standard`
  - `heavy`
- summary 已新增 `layer` 字段，结果目录按层输出：
  - 例如 `rust-gates-fast`、`rust-gates-standard`
- `legacy` 仍保留旧全量链路（含 `tauri-build`）用于兼容
- `fast / standard / heavy` 已按冻结归属集执行，且：
  - `fast`、`standard` 不再带 `coverage / udeps / tauri-build`
  - `standard` 使用 `nextest x1`
  - `heavy` 使用 `nextest x3`

### 12.4 phase 完成后状态 check

- [x] 质量脚本已支持按层级选择 gate 集合
- [x] 各层仍有统一 summary 产物
- [x] `fast` 与 `standard` 不再误带 `coverage / udeps / tauri-build`
- [x] 旧入口仍有明确兼容路径

---

## 13. Phase 3：落库分层命令与默认别名

### 13.1 开始前读取文件

- `docs/quality/layered-quality-pipeline-phase-plan.md`
- `package.json`
- `docs/quality/current-quality-process.md`

### 13.2 todo 顺序

1. 在 `package.json` 新增分层命令
2. 把 `check:quality` 重定向到默认层
3. 保留原有专项命令
4. 明确哪些命令属于手动高成本入口

### 13.3 要做的内容

至少需要落地：

- `check:quality:fast`
- `check:quality:standard`
- `check:quality:heavy`
- `check:quality:release`
- `check:quality` -> `check:quality:standard`

本轮落地结果：

- `package.json` 已新增：
  - `check:quality:fast`
  - `check:quality:standard`
  - `check:quality:heavy`
  - `check:quality:release`
  - `check:quality:legacy`
- `check:quality` 已切到 `check:quality:standard`
- 原有专项命令保持不变：
  - `check:debt`
  - `check:release`
  - `test:contracts`
  - `e2e:desktop`
  - `e2e:desktop:doctor`
- `check:quality:release` 当前定义为高成本链路：
  - `check:quality:heavy`
  - `check:release`
  - `e2e:desktop:doctor`
  - `e2e:desktop`

### 13.4 phase 完成后状态 check

- [x] `package.json` 已有四层质量命令
- [x] `check:quality` 已变成默认标准层
- [x] 原有专项命令仍可直接运行
- [x] 文档中的命令口径与实际脚本一致

---

## 14. Phase 4：同步触发策略、文档入口与结果产物说明

### 14.1 开始前读取文件

- `docs/quality/layered-quality-pipeline-phase-plan.md`
- `docs/quality/current-quality-process.md`
- `docs/README.md`
- `docs/testing/tauri-e2e-strategy.md`

### 14.2 todo 顺序

1. 把当前质量入口文档改成分层说明
2. 把按变更类型触发的策略和分层命令对齐
3. 补充产物目录与阅读方式说明
4. 更新 docs 索引与需要的新入口

### 14.3 要做的内容

需要明确：

- 日常本地改动默认先跑哪一层
- 哪些情况下升级到 `heavy`
- 哪些情况下必须跑 `release`
- 是否需要把桌面 E2E 放进 `release` 的固定闭环里

本轮落地结果：

- `docs/quality/current-quality-process.md` 已切到分层入口口径：
  - 增加 `5.4 分层入口推荐`
  - 把 `6.*` 变更类型触发动作改成分层命令
  - 新增 `6.6 发布前收口`
- 结果产物说明已补齐：
  - `rust-gates` 与 `rust-gates-<layer>`
  - `quality-gates-summary.json.layer`
- `docs/testing/tauri-e2e-strategy.md` 已同步：
  - 发布级入口可通过 `npm run check:quality:release` 串联 desktop E2E
- docs 索引仍以 `docs/README.md` 为单入口，`current-quality-process` 与 `layered-quality-pipeline-phase-plan` 可直接定位

### 14.4 phase 完成后状态 check

- [x] 当前质量入口文档已变成分层说明
- [x] 按变更类型触发规则已与新命令一致
- [x] docs 索引已能定位到分层计划与当前入口
- [x] 结果产物说明已覆盖多层流水线场景

---

## 15. Phase 5：验证、日志、文档回填

### 15.1 开始前读取文件

- `docs/quality/layered-quality-pipeline-phase-plan.md`
- `docs/logs/<当天日期>.md`
- 本轮实际变更涉及的脚本与文档

### 15.2 todo 顺序

1. 逐层执行最小验证
2. 记录每层实际耗时与结果产物
3. 回填本计划文档状态
4. 回填当日日志与当前质量入口文档

### 15.3 要做的内容

至少应验证：

- `fast`
- `standard`
- 至少一次 `heavy` 或其关键重项组合
- 若涉及 release 入口变更，则补 `release` 层验证

本轮验证矩阵结果：

| 层级 | 命令 | 结果 | 关键结论 | 产物 |
|---|---|---|---|---|
| fast | `npm run check:quality:fast` | 通过 | 4/4 通过 | `data/quality-gates/20260312-084055/rust-gates-fast/quality-gates-summary.json` |
| standard | `npm run check:quality:standard` | 失败 | 失败项：`clippy`、`duplicate-deps` | `data/quality-gates/20260312-084112/rust-gates-standard/quality-gates-summary.json` |
| heavy | `npm run check:quality:heavy` | 失败 | 失败项：`clippy`、`deny`、`duplicate-deps` | `data/quality-gates/20260312-084137/rust-gates-heavy/quality-gates-summary.json` |
| release | `npm run check:quality:release` | 失败 | 在 `heavy` 阶段即失败，后续 `check:release/e2e` 未触发 | `data/quality-gates/20260312-084417/rust-gates-heavy/quality-gates-summary.json` |

补充说明：

- `npm run check:quality` 默认别名已验证指向 `standard` 层
- 产物文件均包含 `layer` 字段，可用于后续自动汇总

### 15.4 phase 完成后状态 check

- [x] 分层命令至少完成一轮最小验证
- [x] 各层 summary 产物可落盘并可读
- [x] 本计划文档 phase 状态已回填
- [x] 当日日志已记录本轮落地结果

## 16. 当前建议的推进顺序

后续继续执行时，建议优先顺序固定为：

1. 先收敛既有失败项（`clippy / deny / duplicate-deps`）
2. 再推进下一批质量缺口（`业务 benchmark compare / Go-No-Go 自动化`）
3. 最后补发布级闭环细节（安装/升级/签名与离线 smoke）

这样可以先把“流程定义 + 命令入口 + 触发策略”完整闭环，再进入质量债务治理。
