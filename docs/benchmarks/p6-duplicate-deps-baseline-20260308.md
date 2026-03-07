# 2026-03-08 duplicate deps 基线治理记录（P2）

本记录用于把 `scripts/quality/check-duplicate-deps.ps1` 从“看到任何重复依赖就失败”的粗粒度口径，收口成符合 `P2` 治理目标的 baseline-delta 口径。

## 本轮调整

- 新增基线文件：`config/quality/duplicate-deps-baseline.json`
- duplicate deps 脚本现在会区分：
  - 真正的多版本 crate family
  - 同版本但在不同 host/target/build 上重复出现的 package header
- 门禁规则改为：
  - 不允许新增 duplicate family
  - 不允许在已有 family 上新增 version
  - 允许在现有基线内持续下降

## 当前基线

- 基线文件：`config/quality/duplicate-deps-baseline.json`
- 检查产物：`data/quality-gates/20260308-010134/duplicate-deps/duplicate-deps-summary.json`
- 当前 raw header 数：`56`
- 当前真正的多版本 crate family 数：`21`
- 当前同版本重复 header 数：`7`

## 本轮额外收敛

- 调整了工作区 `image` 依赖，改为 `default-features = false`
- 仅保留当前链路需要的 `jpeg` / `png` / `webp` feature
- 收敛结果：
  - raw duplicate header：`63 -> 56`
  - duplicate family：`23 -> 21`
  - 同版本重复 header：`9 -> 7`

本轮直接消掉的 family：

- `zune-core`
- `zune-jpeg`

## 当前结果

`duplicate-deps-summary.json` 当前显示：

- `passed = true`
- `baselineMatched = true`
- `debtRemaining = true`
- `addedFamilies = []`
- `addedVersions = []`

这表示：

- 当前重复依赖债务并没有被隐藏，仍然完整记录在 summary 中
- 但门禁现在只在“债务新增”时失败，符合 `P2` 的 delta 治理目标

## 当前保留的多版本 family

- `bitflags`
- `getrandom`
- `hashbrown`
- `indexmap`
- `phf` / `phf_codegen` / `phf_generator` / `phf_macros` / `phf_shared`
- `png`
- `rand` / `rand_chacha` / `rand_core`
- `siphasher`
- `syn`
- `thiserror` / `thiserror-impl`
- `windows-link` / `windows-sys` / `windows-targets` / `windows_x86_64_msvc`

## 当前判断

- 这批重复依赖大多来自 `tauri`、`wry`、`schemars`、`image` 与其转译依赖链，并不适合在初始化阶段为追求归零做大范围版本改造。
- 当前最重要的是把基线固定住，避免后续改动再继续引入新的版本分叉。
- 这轮已经验证：从工作区 direct deps 的 feature 裁剪入手，确实可以先拿到小而稳的收敛收益。
- 后续若要继续压缩这批 family，仍应优先从 `image` / `zip` 这类工作区直接依赖切入，而不是先动 `tauri` 宿主链路。

## 结论

- duplicate deps 现在已经进入可重复、可比较、可防回退的治理状态。
- `npm run check:quality` 已可在当前基线上完整通过，同时仍保留 debt 可见性。
