# MediaPlayerNext Rust/Tauri 工程审核方案与固定质量流程（v1）

> 目标：在保留你现有 Electron 审核思路的前提下，给 Rust/Tauri 迁移后的工程提供一套**等效、可执行、可持续对比版本变化**的审核方案；同时把开发期必须执行的固定流程（契约、测试、迁移、性能、发布）标准化。

---

# 一、设计原则

这份方案不追求“Rust 风格完全重写审核框架”，而是尽量保持与你现有 Electron 评估报告一致的阅读与治理习惯：

- 仍然保留 `P0 / P1 / P2` 分级门禁
- 仍然保留 `Go / No-Go` 发布结论
- 仍然保留 `方法口径 -> 门禁结果 -> 规模 -> 架构 -> 测试 -> 构建 -> 安全 -> 风险矩阵 -> 趋势对比` 的报告结构
- 仍然强调 **delta 管理**：不是一次性把所有治理问题清零，而是**不允许新增债务**

但 Rust/Tauri 工程与 Electron 工程的关注点并不完全相同，因此等效方案会把关注重点从：

- TS 类型债
- preload / IPC 暴露面
- Electron 主进程聚合热点

切换为：

- workspace / crate 边界
- command / channel / custom protocol / sidecar 契约
- `unsafe` / `unwrap` / `todo!` / `panic!` 债务
- DB migration / fixture 兼容性
- benchmark / 二进制体积 / 依赖图治理

---

# 二、Electron 方案到 Rust 方案的等效映射

| Electron 方案项 | Rust/Tauri 等效项 | 说明 |
|---|---|---|
| `format:check` | `cargo fmt --all --check` | 代码风格一致性 |
| `lint` | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Rust 静态检查主门禁 |
| `typecheck/build` | `cargo check` + `cargo build --release` | Rust 没有 TS typecheck，但 `cargo check` 等价承担编译级类型校验 |
| `test` | `cargo nextest run` / `cargo test` | Rust 测试主链路 |
| `coverage` | `cargo llvm-cov` | Rust 覆盖率主方案 |
| `flaky-index x3` | `nextest` 多轮复跑 + flaky 标记 | 保持“同一 commit 连跑 3 次”思路 |
| `audit(high/critical)` | `cargo deny` + `cargo audit` | RustSec / license / banned crates / sources |
| `madge=0` | `cargo metadata` + forbidden edges 检查 | Cargo 包图天然无包级循环，真正要守的是“禁止依赖方向” |
| `IPC 边界治理` | `command / channel / custom protocol / sidecar` 契约治理 | Tauri 宿主通信边界 |
| `ts-prune` | `cargo udeps` | 未使用依赖治理 |
| `type debt (any/ts-ignore)` | `unsafe / unwrap / expect / todo! / unimplemented! / panic! / #[allow]` 债务 | Rust 版本的“可维护性债务” |
| `jscpd` | `.rs/.toml/.ts` 文本重复率 | 仍保留 delta 守门 |
| `jsinspect-plus` | 逻辑重复热点清单 + 大函数/大文件 + 基准热点复盘 | Rust 生态没有同等普及的 AST 重复基线，改为热点治理 |
| `slot/i18n 治理脚本` | 能力(capabilities)/权限/契约导出/DB migration/fixture 校验脚本 | Rust/Tauri 特有治理点 |
| 构建产物检查 | `cargo tauri build` + sidecar/resource + 二进制体积变化 | 更关注打包结果与宿主边界 |

---

# 三、Rust 工程等效审核方案（报告模板）

下面这版可以直接作为你未来 Rust/Tauri 项目评估报告模板使用。

---

# MediaPlayerNext Rust 工程评估报告（v<版本号>）

> 评估日期：<YYYY-MM-DD>  
> 项目类型：Tauri 2 + Rust + React  
> 评估人：<姓名/角色>  
> 评估范围：规模/结构质量/契约与迁移/测试/覆盖率/构建与产物/安全与依赖/性能与体积/维护性/发布就绪  
> 评估基线：仓库 `<路径>`，版本 `<x.y.z>`，commit `<short-sha>`（工作区 <clean|dirty>）  
> 评估环境：OS `<平台>`；Rust `<version>`；Cargo `<version>`；Tauri `<version>`；Node `<version>`（若前端存在）

---

## 0. 结论摘要（Go/No-Go）

- **项目规模结论**：<小型/中型/大型>（Rust 业务代码 <LOC> LOC，<files> 文件，workspace <N> crates）。
- **功能复杂度结论**：<低/中/高>（<一句话依据：扫描/缩略图/归档/播放/字幕/DB/sidecar 等>）。
- **总体质量结论**：**<评级>**（<一句话解释>）。
- **发布建议**：<✅ Go / ❌ No-Go>。
- **阻断项（P0）**：<列出阻断项；若无写“无”>。
- **主要风险（Top 3）**：
  1) <风险 1>  
  2) <风险 2>  
  3) <风险 3>

---

## 1. 评估方法与口径

### 1.1 统计口径
- 业务代码范围：`src-tauri/src/**/*.rs` + `crates/**/src/**/*.rs`；排除 `tests/fixtures`、生成代码、`target`。
- 前端代码范围（若仍保留 Web UI）：`src/**/*.ts(x)`。
- 测试范围：单元测试、`tests/` 集成测试、doc tests、命令/协议适配测试、DB migration 测试、fixture/golden tests、benchmark（单列）。
- LOC 与规模度量：Rust 后端 / 前端 / 测试 / 基准分开统计。
- 结构指标：workspace crate 数、crate 依赖边界、未使用依赖、重复率、`unsafe` 面积、`unwrap/expect/todo!` 债务、二进制体积变化。
- 稳定性指标：Flaky Index（同一 commit 连跑 3 次）；必要时按测试组单独统计。

### 1.2 分级门禁定义（P0/P1/P2）

#### P0（发布阻断）
- `cargo fmt --check`
- `cargo clippy -D warnings`
- `cargo check --workspace --all-targets --locked`
- `cargo build --workspace --release --locked`
- `cargo nextest run` / `cargo test`
- `cargo test --doc`
- 覆盖率门禁
- `cargo deny check` / `cargo audit`
- 关键迁移与契约测试通过
- flaky-index 达标
- `cargo tauri build` 或等效发布构建通过

#### P1（高风险回归）
- crate 依赖方向无违规（forbidden edges = 0）
- Tauri command/channel/custom protocol/sidecar 契约导出与实现一致
- capabilities / 权限声明无扩大暴露
- DB migration 可从 `N-1 -> N` 成功升级
- 外部工具（ffmpeg/mpv/7z 等）版本、路径、可执行性有固定校验
- 关键资源打包（sidecar/resources/migrations）完整

#### P2（治理门禁）
- 文本重复率 delta 不增长
- 大函数 / 大文件数量 delta 不增长
- `unsafe` 债务 delta 不增长
- `unwrap/expect/todo!/panic!` 债务 delta 不增长
- benchmark 回归不超过阈值
- 二进制体积 delta 不超过阈值
- 未使用依赖 / 重复依赖持续收敛

### 1.3 验证命令与结果（模板）

| 类别 | 实际命令 | 结果 | 关键信息 |
|---|---|---|---|
| 版本基线 | `git rev-parse --short HEAD` + `git status --porcelain` | <✅/❌> | commit `<sha>`；工作区 <clean/dirty> |
| baseline-clean | `git diff --quiet` / 自定义脚本 | <✅/❌> | <输出摘要> |
| 格式 | `cargo fmt --all --check` | <✅/❌> | <输出摘要> |
| lint | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | <✅/❌> | warnings `<N>` |
| 编译检查 | `cargo check --workspace --all-targets --locked` | <✅/❌> | <输出摘要> |
| 发布构建 | `cargo build --workspace --release --locked` | <✅/❌> | <输出摘要> |
| 特性矩阵 | `cargo hack check --feature-powerset --depth <N>` | <✅/❌> | 组合 `<N>` |
| 测试（主链路） | `cargo nextest run --workspace --all-features -P ci` | <✅/❌> | `<pass/fail/skip>` |
| 文档测试 | `cargo test --workspace --doc` | <✅/❌> | `<pass/fail>` |
| 覆盖率 | `cargo llvm-cov nextest --workspace --lcov` | <✅/❌> | Lines / Branches / Regions |
| 稳定性复跑 | `<测试命令> x3` | <✅/❌> | `<3/3 或 x/3>` |
| 安全/许可证/重复依赖 | `cargo deny check advisories licenses bans sources` | <✅/❌> | advisories / license / bans / sources |
| 安全（补充） | `cargo audit` | <✅/❌> | vulnerabilities `<N>` |
| 未使用依赖 | `cargo +nightly udeps --workspace --all-targets` | <✅/❌> | <输出摘要> |
| 重复依赖 | `cargo tree -d` | <✅/❌> | duplicate crates `<N>` |
| 架构边界 | `cargo metadata --format-version 1` + 自定义 forbidden-edges 脚本 | <✅/❌> | violations `<N>` |
| 契约导出 | `<contract export script>` | <✅/❌> | commands/types/events `<N>` |
| 能力/权限 | `<capability verify script>` | <✅/❌> | 增量权限 `<N>` |
| migration | `<migration smoke/integration tests>` | <✅/❌> | up/down / fixture upgrade |
| 文本重复 | `jscpd crates src-tauri src` | <✅/⚠️/❌> | 重复率 `<x.xx%>` |
| 债务扫描 | `rg 'unwrap!|expect\(|todo!|unimplemented!|panic!|unsafe'` + 脚本 | <✅/⚠️/❌> | delta `<N>` |
| 二进制体积 | `cargo bloat --release -n 20 --crates` | <✅/⚠️/❌> | Top crates / Top funcs |
| 打包构建 | `cargo tauri build` | <✅/❌> | bundle / sidecar / resources |

---

## 2. 质量门禁验证结果

| 检查项 | 优先级 | 结果 | 关键数字 | 证据 |
|---|---|---|---:|---|
| fmt | P0 | <✅/❌> | - | `cargo fmt --all --check` |
| clippy | P0 | <✅/❌> | warnings `<N>` | `cargo clippy ... -D warnings` |
| check | P0 | <✅/❌> | - | `cargo check ...` |
| release build | P0 | <✅/❌> | - | `cargo build --release ...` |
| tests | P0 | <✅/❌> | `<数字>` | `cargo nextest run` |
| doc tests | P0 | <✅/❌> | `<数字>` | `cargo test --doc` |
| coverage | P0 | <✅/❌> | `<数字>` | `cargo llvm-cov ...` |
| flaky-index（3次） | P0 | <✅/❌> | `<x/3>` | `<复跑命令>` |
| security | P0 | <✅/❌> | advisories `<N>` | `cargo deny` + `cargo audit` |
| contract/migration | P0 | <✅/❌> | `<数字>` | `<脚本>` |
| architecture-boundary | P1 | <✅/❌> | violations `<N>` | `cargo metadata + script` |
| capabilities drift | P1 | <✅/❌> | delta `<N>` | `<脚本>` |
| sidecar/resources package | P1 | <✅/❌> | `<数字>` | `cargo tauri build` |
| duplication-delta | P2 | <✅/⚠️/❌> | `<率与 delta>` | `jscpd` |
| debt-delta | P2 | <✅/⚠️/❌> | `unsafe/unwrap/todo/panic` | `<脚本>` |
| binary-size-delta | P2 | <✅/⚠️/❌> | `<MB / top crates>` | `cargo bloat` |
| benchmark-delta | P2 | <✅/⚠️/❌> | `<回归百分比>` | `criterion` 报表 |

---

## 3. 项目规模评估

### 3.1 实际业务行数（LOC）
- Rust 核心源码：<files> 文件 / <loc> 行
- Tauri 宿主层：<files> 文件 / <loc> 行
- Web 前端：<files> 文件 / <loc> 行
- 核心业务合计：<files> 文件 / **<loc> 行**
- 测试代码：<files> 文件 / <loc> 行；测试/业务比：**<ratio>%**
- workspace crate 数：<N>

### 3.2 模块与文件规模观察
- >1000 行 Rust 文件 <N> 个，>150 行函数 <N> 个。
- Top 大文件（不含测试）：
  - `<path>`：<lines>
  - `<path>`：<lines>
  - `<path>`：<lines>
  - `<path>`：<lines>
  - `<path>`：<lines>

---

## 4. 结构与架构质量评估

### 4.1 分层结构与边界
- 目标分层：`contracts -> core domain -> adapters -> app shell (tauri)`。
- 核心原则：内层不依赖外层；`core` 不依赖 `tauri`、`ffmpeg` CLI、UI 或 Web 类型。
- 风险热点：扫描编排、缩略图流水线、归档读取、数据库仓储、播放器/字幕 sidecar 适配层。

### 4.2 结构健康度指标（本轮）
- crate 依赖违规：<结果>（<数字>）。
- 未使用依赖：<结果>（<数字>）。
- 重复依赖：<结果>（<数字>）。
- 文本重复率：<结果>（<x.xx%>）。
- 大函数/大文件：<结果>（<数字>）。

---

## 5. 契约、迁移与宿主边界专项

### 5.1 契约治理
- command 数：<N>
- channel 流定义：<N>
- custom protocol：<N>
- sidecar 协议：<N>
- 契约导出方式：<手写 / codegen / specta>
- 本轮新增 breaking changes：<N>

### 5.2 数据迁移治理
- SQLite schema 版本：`<N>`
- migration 数：`<N>`
- `N-1 -> N` 升级验证：<✅/❌>
- 历史 fixture 回放：<✅/❌>
- 回滚策略：<说明>

### 5.3 宿主权限与暴露面
- capabilities 变更：<N>
- 新增 plugin 权限：<N>
- 新增 sidecar / resources：<N>
- 风险说明：<一句话>

---

## 6. 测试质量与稳定性评估

### 6.1 测试结果
- 单元测试：<数字>
- 集成测试：<数字>
- 文档测试：<数字>
- 契约/协议测试：<数字>
- migration 测试：<数字>
- fixture/golden tests：<数字>
- 覆盖率汇总：Lines `<x.xx%>` / Branches `<x.xx%>` / Regions `<x.xx%>`

### 6.2 稳定性结论（Flaky Index）
- 同一 commit 连跑 3 次：<x/3> 通过
- 结论：<稳定/不稳定>
- 发布影响：<说明>

---

## 7. 构建、产物与发布质量评估

### 7.1 Cargo 构建
- `cargo check`：<通过/失败>
- `cargo build --release`：<通过/失败>
- feature matrix：<通过/失败>

### 7.2 Tauri 打包
- `cargo tauri build`：<通过/失败>
- sidecar/resources/migrations 打包：<完整/缺失>
- Windows 首发产物：<NSIS/MSI/...>

### 7.3 二进制与体积
- 可执行体积：<数字>
- 主要体积来源 Top N：
  - `<crate/function>`：<size>
  - `<crate/function>`：<size>
  - `<crate/function>`：<size>

---

## 8. 安全与依赖健康评估

### 8.1 安全
- advisories：<N>
- ignored advisories：<N>
- license exceptions：<N>
- banned crates：<N>
- `unsafe` 使用面：<N>

### 8.2 依赖健康
- duplicate deps：<N>
- unused deps：<N>
- direct deps：<N>
- outdated 关键依赖：<N>

---

## 9. 长期稳定性指标（趋势）

### 9.1 热点（90 天）
- 变更次数 Top N：
  - `<path>`：<count>
  - `<path>`：<count>
  - `<path>`：<count>

### 9.2 债务趋势
- `unsafe`：<N>
- `unwrap/expect`：<N>
- `todo!/unimplemented!`：<N>
- `panic!`（非测试）：<N>
- `#[allow(...)]`：<N>

### 9.3 性能趋势
- benchmark 回归项：<N>
- 二进制体积变化：<数字>

---

## 10. 发布就绪度评估

- 结论：**<Go/No-Go>**。
- 发布前建议：
  1) <建议 1>  
  2) <建议 2>  
  3) <建议 3>  
  4) <建议 4>

---

## 11. 风险矩阵与治理闭环

| 风险 | 严重度 | 概率 | 证据 | 当前状态 | 建议 | Owner | 截止 |
|---|---|---|---|---|---|---|---|
| <风险1> | <高/中/低> | <高/中/低> | <证据> | <状态> | <建议> | <团队> | <日期> |
| <风险2> | <高/中/低> | <高/中/低> | <证据> | <状态> | <建议> | <团队> | <日期> |
| <风险3> | <高/中/低> | <高/中/低> | <证据> | <状态> | <建议> | <团队> | <日期> |

### 11.1 治理闭环状态
- 新增问题：<N>
- 已关闭问题：<N>
- 接受风险：<N>（复核日期 <日期>）

---

## 12. 对比上版变化（v<prev> -> v<curr>）

| 项 | v<prev> | v<curr> | 变化解读 |
|---|---|---|---|
| commit | `<sha>` | `<sha>` | <说明> |
| 工作区状态 | <clean/dirty> | <clean/dirty> | <说明> |
| Rust LOC | <数字> | <数字> | <说明> |
| crates 数 | <数字> | <数字> | <说明> |
| tests | <状态> | <状态> | <说明> |
| coverage | <状态> | <状态> | <说明> |
| contract tests | <状态> | <状态> | <说明> |
| migration tests | <状态> | <状态> | <说明> |
| `unsafe` 债务 | <状态> | <状态> | <说明> |
| duplication | <状态> | <状态> | <说明> |
| binary size | <状态> | <状态> | <说明> |
| benchmark | <状态> | <状态> | <说明> |
| 综合评级 | <评级> | <评级> | <说明> |

---

## 13. 附录：本轮执行命令清单（建议基线）

```bash
# baseline
 git rev-parse --short HEAD
 git status --porcelain

# format / lint / build
 cargo fmt --all --check
 cargo clippy --workspace --all-targets --all-features -- -D warnings
 cargo check --workspace --all-targets --locked
 cargo build --workspace --release --locked

# feature matrix
 cargo hack check --workspace --feature-powerset --depth 2 --no-dev-deps

# tests
 cargo nextest run --workspace --all-features -P ci
 cargo test --workspace --doc
 cargo llvm-cov nextest --workspace --all-features --lcov --output-path lcov.info

# security / deps
 cargo deny check advisories licenses bans sources
 cargo audit
 cargo tree -d
 cargo +nightly udeps --workspace --all-targets

# architecture / debt / size
 cargo metadata --format-version 1 > target/metadata.json
 cargo bloat --release -n 20 --crates
 cargo tauri build
```

---

# 四、开发中必须执行的固定流程（契约、测试、迁移、发布）

这部分不是“建议”，而是建议你作为 Rust 迁移期的**固定开发流程**落进仓库（PR 模板、CI、开发文档、脚本）里的内容。

---

## 4.1 变更类型分级

每个需求或修复在开始前必须归类到以下类型之一：

1. **纯内部实现变更**  
   不改契约、不改 schema、不改 sidecar 协议、不改 capabilities。

2. **契约变更**  
   改 command 参数/返回、channel 消息、custom protocol、sidecar 输入输出。

3. **数据迁移变更**  
   改数据库 schema、索引、缓存布局、缩略图 key 规则。

4. **高频路径变更**  
   扫描、缩略图、解压、元数据读取、播放控制、列表加载。

5. **安全/权限变更**  
   改 capabilities、文件系统暴露面、sidecar、新增外部二进制。

不同类型必须触发不同的固定动作，不能“写完代码再看”。

---

## 4.2 契约优先流程（强制）

凡是会被前端、sidecar、外部工具调用的后端能力，都必须先完成契约。

### 固定步骤
1. 在 `crates/contracts`（或等效目录）定义：
   - Request / Response DTO
   - Error DTO（稳定错误码）
   - Event / Channel 消息结构
   - 协议版本号（必要时）

2. 契约必须满足：
   - `Serialize + Deserialize`
   - 禁止把 `anyhow::Error` / 原始字符串直接作为跨边界错误返回
   - 错误结构必须稳定，例如：`code/message/retriable/details`
   - 不允许前端依赖 Rust 内部枚举顺序或 `Display` 文本

3. 若使用 codegen：
   - Rust 为单一事实源
   - 导出绑定文件纳入测试，而不是靠开发者手动更新

4. 每个契约变更必须附带：
   - 一条 golden fixture
   - 一条 backward compatibility fixture（若涉及已有数据/历史调用）
   - 一条错误路径 fixture

### 契约门禁
- 契约导出文件有 diff 但实现没有对应测试：不允许合并
- breaking contract 变更没有版本说明：不允许合并
- error code 新增/删除没有 changelog：不允许合并

---

## 4.3 测试金字塔（固定要求）

### A. 单元测试（必须）
覆盖：
- 纯函数
- 解析逻辑
- path / archive / mime 分类
- cache key
- 指纹与去重规则
- error mapping

要求：
- 新增核心逻辑必须有单元测试
- bug fix 必须先补失败测试再修复

### B. 集成测试（必须）
覆盖：
- 扫描目录 -> 入库 -> 查询结果
- ZIP 打开 -> 页访问 -> 缩略图生成
- DB migration `N-1 -> N`
- 播放器适配器与假进程/假输出
- sidecar 协议回放

要求：
- 所有涉及 IO / DB / 归档 / 子进程的能力至少有 1 条集成测试

### C. 契约测试（必须）
覆盖：
- command 入参与返回 JSON 结构
- channel 消息结构
- custom protocol 返回头、状态码、范围请求（若支持）
- sidecar stdout/stderr/exit code 协议

要求：
- 每个公开能力至少 1 条成功路径 + 1 条错误路径

### D. Fixture / Golden Tests（必须）
覆盖：
- 历史 DB 文件
- 历史缩略图缓存键
- 历史契约 JSON
- 典型 zip/rar/7z 样本
- 特殊文件名/编码/超长路径样本

要求：
- 任何影响兼容性的改动必须更新 fixture
- fixture 更新必须解释为什么是“预期变化”

### E. Benchmark（高频路径必须）
覆盖：
- 扫描目录
- 缩略图生成
- ZIP 首次打开与连续翻页
- 元数据读取
- 批量入库

要求：
- 高频路径改动必须附 benchmark 对比
- 超阈值回归禁止直接合并，除非明确接受风险

### F. 文档测试（建议强制）
覆盖：
- 对外 API 示例
- 协议示例
- 关键 crate 的 README 示例

### G. UI 对接前 smoke tests（必须）
虽然 UI 尚未迁移完成，但后端先行阶段也必须保留：
- CLI harness
- 假前端调用脚本
- 最小 Tauri 壳层 smoke test

避免“后端看似完成，等 UI 接入才第一次真正跑通”。

---

## 4.4 数据库与迁移固定流程（强制）

任何 SQLite schema 变更必须执行：

1. 新 migration 文件
2. migration 命名和编号规范
3. 升级测试：空库 -> 最新版
4. 升级测试：旧库 fixture -> 最新版
5. 兼容验证：旧数据仍可查询
6. 索引验证：关键查询 explain / benchmark 不退化
7. 失败回滚策略说明

### 额外要求
- 禁止静默重建库来绕过 migration
- 禁止在没有 fixture 的情况下修改 schema
- 禁止把“重扫即可修复”当作默认迁移策略

---

## 4.5 高频路径固定流程（强制）

对以下链路实行“变更必须带 benchmark”的规则：

- thumbnail
- scan/import
- zip read / page decode
- search/query
- playlist load / media open

### 每次改动必须提交
- baseline 数据
- 变更后数据
- 样本数据集说明
- 结论：改善 / 持平 / 回归
- 若回归：是否接受、为什么

建议阈值：
- P50 回归 > 10%：必须解释
- P95 回归 > 15%：默认拦截
- 内存峰值上升 > 15%：默认拦截

---

## 4.6 Rust 专项债务治理（固定脚本）

建议把以下内容做成 CI 脚本，按 delta 守门：

- `unsafe`
- `unwrap(`
- `expect(`
- `todo!`
- `unimplemented!`
- `panic!`（排除测试）
- `#[allow(...)]`
- `dbg!`

### 规则
- 新增 `unsafe`：必须附安全说明与测试说明
- 新增 `unwrap/expect`：必须说明“理论上不会失败”的前提，最好改成 error path
- 新增 `todo!/unimplemented!`：默认不允许进入主分支
- 新增 `panic!`：默认只允许测试代码
- 新增 `#[allow(clippy::...)]`：必须写原因，且优先局部而不是全局

---

## 4.7 架构边界固定流程（强制）

建议 workspace 至少拆成：

- `crates/contracts`：DTO / error codes / exported schema
- `crates/core-*`：纯业务逻辑，不依赖 tauri
- `crates/infra-*`：sqlite / fs / zip / subprocess / thumbnail / metadata
- `apps/tauri-shell`：Tauri 宿主、commands、capabilities、resources

### 边界规则
- `core-*` 不得依赖 `tauri`
- `core-*` 不得依赖前端类型
- `contracts` 不得依赖 `tauri`
- command 层不得直接写 SQL，必须经 repository / service
- UI 不得直接访问数据库文件、缓存目录、sidecar 私有协议

### 检查方式
- `cargo metadata` + 自定义 forbidden-edges 脚本
- 每个 crate 在 `Cargo.toml` 中声明角色
- PR 中若新增依赖方向，必须说明原因

---

## 4.8 Tauri 宿主边界固定流程（强制）

### 对 command
- 只做控制面，不传大块二进制
- 参数和返回值必须有稳定契约
- 错误必须映射为稳定错误码

### 对 channel
- 只用于流式/进度/批量状态
- 必须定义消息枚举
- 必须有终止/取消/错误语义

### 对 custom protocol
- 只用于图片、视频、缩略图等媒体字节
- 必须测试路径编码、中文、空格、范围请求、非法路径

### 对 sidecar
- 必须固定版本
- 必须固定路径和存在性校验
- stdout/stderr 协议必须可测试
- 退出码必须有映射

### 对 capabilities
- 新增权限必须单独列入 PR
- 默认 deny，按窗口/页面最小授权

---

## 4.9 PR 固定清单（建议直接做成模板）

每个 PR 必填：

1. 变更类型：内部 / 契约 / migration / 高频路径 / 权限
2. 是否改契约：是/否
3. 是否改 schema：是/否
4. 是否改 sidecar / resources：是/否
5. 新增测试：单元 / 集成 / 契约 / fixture / benchmark
6. 是否有 breaking change：是/否
7. 是否有性能影响：提升 / 无明显影响 / 可能回归
8. 是否新增 `unsafe/unwrap/todo!/panic!`：是/否
9. 是否更新文档 / fixtures / 导出绑定：是/否

---

## 4.10 CI 固定流水线（推荐）

### PR 必跑
1. `cargo fmt --check`
2. `cargo clippy -D warnings`
3. `cargo check --workspace --all-targets --locked`
4. `cargo nextest run --workspace --all-features -P ci`
5. `cargo test --doc`
6. 契约导出校验
7. forbidden-edges 校验
8. 债务 delta 校验
9. migration smoke tests
10. `cargo deny check advisories licenses bans sources`

### main 分支必跑
1. PR 必跑全部内容
2. `cargo llvm-cov nextest`
3. feature matrix
4. `cargo audit`
5. `cargo +nightly udeps`
6. `cargo tree -d`
7. benchmark compare
8. `cargo tauri build`
9. 二进制体积 diff

### release 分支必跑
1. main 全量内容
2. 打包签名 / 资源完整性校验
3. sidecar 可执行性校验
4. 离线安装/升级 smoke tests
5. `N-1 -> N` 真升级回放
6. 风险矩阵与 release note 生成

---

# 五、建议的默认阈值（初版）

## P0
- clippy warnings：0
- test failures：0
- doc test failures：0
- security advisories（未豁免）：0
- migration failures：0
- contract export drift：0
- packaging failures：0

## P1
- forbidden dependency edges：0
- capabilities 非预期扩大：0
- sidecar/resource 缺失：0
- 历史 fixture 不兼容：0

## P2
- 重复率 delta：<= 0
- 大文件数 delta：<= 0
- `unsafe` delta：<= 0
- `unwrap/expect` delta：<= 0
- `todo!/unimplemented!` delta：<= 0
- 二进制体积 delta：默认 <= +5%
- benchmark 回归：P95 默认 <= +15%

---

# 六、你这个项目最该优先落地的 8 个质量动作

1. 建立 `contracts` crate，统一 DTO / error code / exported schema
2. 建立 `forbidden-edges` 架构校验脚本
3. 建立 `debt-delta` 脚本，统计 `unsafe/unwrap/todo/panic`
4. 建立 `migration fixtures` 与 `历史契约 fixtures`
5. 建立 `thumbnail / scan / zip` 的 benchmark 基线
6. 建立 `cargo deny` 与 `cargo audit` 的双层安全门禁
7. 建立 `cargo tauri build` 的资源与 sidecar 完整性校验
8. 把 PR 模板与 CI 必跑链路一起落库，不等到 UI 接入后再补

---

# 七、实施建议（按阶段）

## Phase A：后端先行阶段（现在就能做）
- contracts crate
- error code 规范
- fixture 目录规范
- migration harness
- benchmark harness
- debt-delta / forbidden-edges / contract-export 脚本
- thumbnail / scan / zip / db 的单元 + 集成测试

## Phase B：UI 对接前阶段
- Tauri command/channel/custom protocol 最小壳层
- 假前端 smoke tests
- capabilities 初版
- sidecar 协议测试

## Phase C：发布前阶段
- 完整覆盖率
- 打包构建
- 体积基线
- 风险矩阵
- Go / No-Go 评估报告

---

# 八、最终建议

对你这个项目，Rust 工程审核方案不要只把 Electron 的 `lint/build/test` 换成 Cargo 命令，而要真正把审核焦点改成：

- 契约是否稳定
- crate 边界是否正确
- migration 是否可回放
- 高频路径是否有 benchmark
- sidecar / resources / capabilities 是否受控
- `unsafe/unwrap/todo` 债务是否按 delta 受管

真正决定 Rust 迁移后质量上限的，不是“代码能不能编过”，而是这几条固定流程能不能在开发期持续执行。
