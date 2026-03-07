# 2026-03-07 归档归一化验证记录（B6）

本记录用于确认 `B6` 阶段 `rar/7z -> zip` 归一化主链路已经具备：

- `7z` wrapper 参数构造
- 归一化输出布局
- `rar/7z -> normalized.zip` 产物生成
- 归一化后复用 `archive index` 语义写回
- 失败任务记录与重试
- `archive.normalize` / `archive.normalize-status` 开发期命令输入面

本轮验证基于：

- `crates/media-io` 的 normalize 单元测试
- `crates/app-core` 的归一化应用层测试
- `config/local.paths.json` 中的 `sevenz` 开发期路径约定

说明：

- 当前仓库未提交真实 `rar/7z` 二进制样本
- 当前验证以 mock extractor + 动态生成 zip 产物为主，用于先固定状态语义、输出布局与重试行为
- 后续如需要更强回归，可再追加仓库外真实样本验证记录

## 基本信息

- 日期：`2026-03-07`
- 阶段：`B6`
- 验证入口：`archive.normalize` / `archive.normalize-status`
- 验证目标：确认 `rar/7z` 归一化后的 zip 产物可以无缝接回现有 `B4` 链路

## 验证场景

### 7z 归一化成功

- 输入类型：`SourceRecord(kind=archive, ext=7z)`
- 结果：
  - 可生成稳定的 `normalized.zip`
  - `archives.normalized_zip_path` 可写回
  - `archive_entries` 可按归一化 zip 写回
  - `archives.status=normalized`
  - `TaskRecord(task_type=normalize, state=completed)` 可查询

### rar 失败后重试

- 输入类型：`SourceRecord(kind=archive, ext=rar)`
- 结果：
  - 首次失败会记录：
    - `archives.status=failed`
    - `TaskRecord.state=failed`
    - `error_code=normalize_failed`
  - 第二次重试成功后：
    - 归一化 zip 可生成
    - `archives.status=normalized`
    - 可写回 `archive_entries`

### 输出布局与清理

- 输出布局：
  - `data/cache/normalized/<source_id>/<revision_hash>/normalized.zip`
- 重试策略：
  - 同一 `source_id + revision` 再次执行时，会先清理旧 work dir，再生成新结果

## 当前结果判断

- 结果符合预期：`B6` 首版已经完成从 `rar/7z` 输入、归一化 zip 产物、状态记录到重试恢复的最小闭环
- 当前链路已经能够把非 zip 归档纳入现有 `B4` 的索引与后续消费路径，而不是继续把复杂格式留给高频读取链路处理

## 当前缺口

- 还没有仓库内真实 `rar/7z` 二进制样本
- 还没有真实 `7z.exe` 环境下的 benchmark 记录
- `packing_zip` 细粒度状态当前未单独暴露为独立 archive status

## 下一步动作

- 进入 `B7`，开始播放后端与 `media://` / `archive://` 协议输入面
- 如后续需要，再补真实样本级别的 `rar/7z` benchmark
