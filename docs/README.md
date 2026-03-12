# MediaPlayerNext 文档索引

本目录根层只保留这一份索引文档。

## 当前主动维护文档

- `docs/ui/ui-definition.md`
  - 当前 UI 定义、布局迁移、变量合同、切片与验收基线
  - 只记录 `MediaPlayerNext` 自身定义
- `docs/ui/u2-thumbnail-grid-phase-plan.md`
  - `U2` 缩略图网格分页、缩放与容器自适应的执行型计划文档
  - 用于后续新对话继续按 phase 推进实现
- `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md`
  - `U2` 基础版完成后的增强项计划文档
  - 覆盖滚轮翻页、ready-commit、gap snap、缩略图分辨率自适应
- `docs/quality/current-quality-process.md`
  - 当前质量门禁、执行入口、变更触发规则与缺口收口顺序
  - 作为当前版质量流程总入口（默认 `check:quality` 已对齐 `standard` 层）
- `docs/quality/layered-quality-pipeline-phase-plan.md`
  - 分层质量流水线执行计划，按 phase 推进命令分层与脚本重构
  - 作为后续质量命令改造的执行型文档
- `docs/ui/source-ui-known-data.md`
  - 源项目 UI 已知数据、锚点、变量与文档位置记录
  - 仅作迁移参考，不作为新项目定义

## 历史计划归档

- `docs/archive/root-plans/migration-plan.md`
  - 初始化阶段迁移总计划（历史版本）
- `docs/archive/root-plans/00-MediaPlayerNext_实施计划_v2.md`
  - 后端先行阶段总实施计划（历史版本）
- `docs/archive/root-plans/01-MediaPlayerNext_Rust_审核方案_与质量流程_v1.md`
  - Rust/Tauri 质量流程与审核方案（历史版本）
- `docs/archive/root-plans/02-MediaPlayerNext_后端先行具体实施计划_B1-B4_v1.md`
  - B1-B4 历史实施计划
- `docs/archive/root-plans/03-MediaPlayerNext_后端先行具体实施计划_B5-B8_v1.md`
  - B5-B8 历史实施计划
- `docs/archive/root-plans/04-MediaPlayerNext_UI接入具体实施计划_I1-I4_v1.md`
  - I1-I4 历史 UI 接入计划
- `docs/archive/root-plans/05-MediaPlayerNext_P6收口与I1前置具体实施计划_v1.md`
  - P6 收口历史计划

## 参考目录

- `docs/contracts/`
  - contracts、repository surface、transport boundary 等参考文档
- `docs/benchmarks/`
  - benchmark 与验证记录
- `docs/fixtures/`
  - fixtures 与 golden 说明
- `docs/runtime/`
  - 运行时路径与资源策略
  - 当前数据库管理分页实施计划：`docs/runtime/database-management-phase-plan.md`
- `docs/testing/`
  - 自动化测试与 E2E 验收策略
  - 当前 Tauri 自动化 E2E 方案：`docs/testing/tauri-e2e-strategy.md`
- `docs/quality/`
  - 当前质量流程、门禁分层、执行入口与后续收口顺序
  - 分层质量流水线计划：`docs/quality/layered-quality-pipeline-phase-plan.md`
- `docs/logs/`
  - 每日工作记录

## 当前文档策略

- 历史计划不再继续在根目录堆叠
- 新的 UI 迁移与布局重建以 `docs/ui/ui-definition.md` 为单一主动入口
- `U2` 缩略图分页与缩放的执行过程以 `docs/ui/u2-thumbnail-grid-phase-plan.md` 为当前任务入口
- `U2` 增强项推进以 `docs/ui/u2-thumbnail-grid-enhancement-phase-plan.md` 为后续任务入口
- 新的质量流程以 `docs/quality/current-quality-process.md` 为当前主动入口
- 若后续出现新的执行型文档，优先放到对应子目录，并在本索引补充链接
