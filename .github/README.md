# GitHub 配置

当前目录已接入首版 CI / PR / release 相关配置：

- `workflows/quality-fast.yml`
  - `main` push 与 PR 默认快反馈门禁
- `workflows/quality-standard.yml`
  - PR 与手动触发的标准层门禁
- `workflows/release-verify.yml`
  - 手动触发的发布级校验（heavy + release verify）
- `pull_request_template.md`
  - PR 最小验证与文档同步检查清单

后续若新增工作流或模板，需要同步更新：

- `docs/quality/current-quality-process.md`
- `docs/logs/<当天日期>.md`
