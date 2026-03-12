# 2026-03-12 Rust debt-delta 基线记录（P2）

本记录用于把 `scripts/quality/check-debt-delta.ps1` 从“文档里提到要治理”推进到“仓库里已有可执行 baseline-delta 门禁”的状态。

## 本轮新增

- 新增脚本：`scripts/quality/check-debt-delta.ps1`
- 新增基线文件：`config/quality/debt-baseline.json`
- 新增直接入口：`npm run check:debt`
- 统一质量入口 `npm run check:quality` 已接入 `debt-delta`

## 当前口径

扫描范围固定为：

- `crates/**/*.rs`
- `src-tauri/**/*.rs`
- 排除 `target`

当前统计的 Rust debt 类型：

- `unsafe`
- `unwrap(`
- `expect(`
- `todo!`
- `unimplemented!`
- `panic!`
- `#[allow(...)]`
- `dbg!`

门禁规则：

- 不允许在任意文件上新增某类 debt 次数
- 允许在现有基线上持续下降
- 当前仍保留 debt 可见性，不伪装成“已经清零”

## 首轮基线（脚本落地时）

- 基线文件：`config/quality/debt-baseline.json`
- 首轮检查产物：`data/quality-gates/20260312-074716/debt-delta/debt-delta-summary.json`
- 扫描 Rust 文件数：`53`
- 当前 debt 总次数：`557`

按类别汇总：

- `expect(`：`542`
- `panic!`：`9`
- `#[allow(...)]`：`6`
- `unsafe`：`0`
- `unwrap(`：`0`
- `todo!`：`0`
- `unimplemented!`：`0`
- `dbg!`：`0`

## 本轮追加收敛（首批 expect 治理）

本轮在 `crates/media-db/tests/database_integration.rs` 做了第一批收敛：

- 将多条 happy-path 集成测试改为返回 `Result<()>`
- 用 `?` 与 `context(...)` 替代大量 `.expect(...)`
- 保留了原本用于坏路径断言的失败分支结构，不把错误路径语义抹平

对应产物：

- 收敛后检查产物：`data/quality-gates/20260312-080519/debt-delta/debt-delta-summary.json`
- 收敛后 debt 总次数：`502`
- `expect(`：`542 -> 487`（`-55`）
- `crates/media-db/tests/database_integration.rs`：`79 -> 24`

为防止回弹，本轮已刷新 baseline：

- 刷新后基线：`config/quality/debt-baseline.json`
- `updatedAt`：`2026-03-12T08:06:08`

## 本轮继续收敛（第二批 expect 治理）

本轮继续在低风险区推进，优先处理 `src-tauri/src/lib.rs` 的测试模块：

- 把 protocol 相关测试统一改为 `Result<()>`
- 用 `?` 与 `anyhow::Context` 替换 `.expect(...)`
- 保留失败路径断言语义，不改变业务分支判断
- 顺手移除 `workspace_root()` 中的 1 处 `expect`

对应产物：

- 收敛检查产物：`data/quality-gates/20260312-081240/debt-delta/debt-delta-summary.json`
- 收敛后 debt 总次数：`502 -> 411`（`-91`）
- `expect(`：`487 -> 396`（`-91`）
- `src-tauri/src/lib.rs`：`96 -> 5`

为防止回弹，本轮再次刷新 baseline：

- 刷新后基线：`config/quality/debt-baseline.json`
- `updatedAt`：`2026-03-12T08:13:02`
- 刷新后验证产物：`data/quality-gates/20260312-081309/debt-delta/debt-delta-summary.json`

## 当前判断

- 当前 debt 主体不是 `unsafe` 或 `unwrap`，而是大量 `expect(`。
- `panic!` 当前只出现在 `crates/app-core/src/asset.rs` 与 `crates/media-db/tests/database_integration.rs`。
- `#[allow(...)]` 当前规模很小，已经适合继续做定点收敛。
- 这说明当前仓库的 Rust debt 结构和原始担心并不完全一致：真正首先需要收敛的是 `expect(`，而不是先从 `unsafe` 开刀。
- 经过本轮第一批收敛后，`expect(` 依然是绝对主体，下一批应优先继续在测试与易回归路径做小步替换。
- 第二批完成后，`expect(` 依旧是主体，但规模已经进入可持续分批收敛区间。

## 与统一质量门禁的关系

- `npm run check:debt` 当前已通过
- `npm run check:quality` 当前已接入 `debt-delta`
- `2026-03-12` 这一轮全量质量门禁仍然失败，但失败项是已有的：
  - `clippy`
  - `deny`
  - `duplicate-deps`
- `debt-delta` 本身已通过，不属于本轮失败来源

## 结论

- Rust debt-delta 已进入可重复、可比较、可防回退的治理状态。
- 后续继续做质量流程补强时，可以先围绕 `expect(` 做分批收敛，再补 `capabilities drift` 与 `contract drift`。
