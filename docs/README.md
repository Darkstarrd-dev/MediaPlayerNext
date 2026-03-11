# MediaPlayerNext 文档索引

本目录根层只保留这一份索引文档。

## 当前主动维护文档

- `docs/ui/ui-definition.md`
  - 当前 UI 定义、布局迁移、变量合同、切片与验收基线
  - 只记录 `MediaPlayerNext` 自身定义
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
- `docs/logs/`
  - 每日工作记录

## 当前文档策略

- 历史计划不再继续在根目录堆叠
- 新的 UI 迁移与布局重建以 `docs/ui/ui-definition.md` 为单一主动入口
- 若后续出现新的执行型文档，优先放到对应子目录，并在本索引补充链接
