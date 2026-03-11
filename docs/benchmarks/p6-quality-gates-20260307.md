# 2026-03-07 P6 质量门禁自动化记录（P6-1）

本记录用于把 `docs/archive/root-plans/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md` 中 `P0 / P1 / P2` 门禁，落成可重复执行的脚本入口与结果产物。

本轮目标不是把所有治理问题一次清零，而是先把“该跑什么、结果落在哪里、失败如何定位”固定下来。

## 本轮新增入口

- `scripts/quality/run-rust-gates.ps1`
- `scripts/quality/run-rust-gates.cmd`
- `scripts/quality/run-coverage.ps1`
- `scripts/quality/run-release-verify.ps1`
- `scripts/quality/check-duplicate-deps.ps1`
- `scripts/quality/check-forbidden-edges.mjs`
- `deny.toml`
- `npm run check:quality`
- `npm run check:release`

## 运行口径

- 日期：`2026-03-07`
- 当前 commit：`cbd5b4f`
- 工作区状态：`dirty`（正在进行 `P6-1` 收口）
- 结果产物目录：`data/quality-gates/20260307-224945/rust-gates`
- Windows Rust 入口：`scripts/run-cargo-with-msvc.cmd`
- 质量工具版本：
  - `cargo-nextest 0.9.114`
  - `cargo-llvm-cov 0.8.4`
  - `cargo-deny 0.19.0`
  - `cargo-audit 0.22.1`
  - `cargo-udeps 0.1.56`

## 本轮执行入口

```bash
scripts\quality\run-rust-gates.cmd
powershell -NoProfile -ExecutionPolicy Bypass -File .\scripts\quality\run-rust-gates.ps1
```

## 本轮结果

| 门禁 | 优先级 | 结果 | 备注 |
|---|---|---|---|
| `fmt` | P0 | ✅ | 通过 |
| `clippy` | P0 | ✅ | `-D warnings` 通过 |
| `check` | P0 | ✅ | `--workspace --all-targets --locked` 通过 |
| `nextest` x3 | P0 | ✅ | `3/3` 通过 |
| `coverage` | P0 | ✅ | Line coverage `76.48%` |
| `cargo deny` | P0 | ✅ | `advisories / licenses / bans / sources` 通过 |
| `cargo audit` | P0 | ✅ | 未报 RustSec 漏洞 |
| `cargo udeps` | P2 | ✅ | 当前 workspace 未检出未使用依赖 |
| `forbidden edges` | P1 | ✅ | 当前 workspace 依赖方向无违规 |
| `tauri build` / release verify | P1 | ✅ | sidecar 构建、bundle、capability 检查通过 |
| `duplicate deps` | P2 | ❌ | 当前仍有 `63` 项重复依赖 |

## 关键产物

- 统一汇总：`data/quality-gates/20260307-224945/rust-gates/quality-gates-summary.json`
- 覆盖率汇总：`data/quality-gates/20260307-224945/rust-gates/coverage-summary.json`
- release verify 汇总：`data/quality-gates/20260307-224945/rust-gates/release-verify-summary.json`
- duplicate deps 汇总：`data/quality-gates/20260307-224945/rust-gates/duplicate-deps-summary.json`

## 当前判断

- `P6-1` 要求的统一脚本入口已经落地。
- `P0 / P1` 当前已能完整自动执行，并能把日志与 JSON 产物固定输出到 `data/quality-gates/<timestamp>/rust-gates`。
- `duplicate deps` 现在被单独收口成可重复门禁，当前基线是 `63` 项重复依赖，属于后续治理项，而不是脚本缺失。
- 本轮还顺带补了 `deny.toml` 与本地 crate `license` 字段，避免 `cargo deny` 继续停留在“默认配置噪音过大、无法形成稳定口径”的状态。

## 当前未收口项

- `duplicate deps` 仍未治理到 `0`，因此 `check:quality` 当前仍会以非零退出码结束。
- 这不再是自动化缺口，而是已被门禁稳定暴露出来的真实 P2 治理问题。

## 结论

- `P6-1` 已完成首轮收口。
- 后续进入 `P6-2` 时，已经可以建立在稳定的质量门禁脚本与结果产物之上继续推进。
