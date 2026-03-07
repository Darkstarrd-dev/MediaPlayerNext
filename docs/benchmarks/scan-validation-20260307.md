# 2026-03-07 扫描验证记录（真实样本基线）

本记录基于以下真实文件基线目录：

- `docs/fixtures/small-fixture/generated-placeholder/`
- `docs/fixtures/medium-fixture/generated-placeholder/scan-root/`
- `data/scan-validation/local-real-dir-mock/`

本轮实际测试不直接改动基线目录，而是先复制到：

- `data/scan-validation/runs/run-20260307-b/`

额外失败/恢复验证使用：

- `data/scan-validation/runs/run-20260307-fail/`

## 基本信息

- 日期：`2026-03-07`
- 阶段：`B3`
- 扫描入口：`scan run` / `scan resume`
- 验证目标：确认真实样本基线下的首扫、无变更重扫、删除+新增后重扫、失败后 resume 入口

## small-fixture

### 数据集概况

- 数据集类型：`small-fixture`
- 数据集名称：`small-fixture-real`
- 总文件数：`28`
- 图片数：`12`
- 音频数：`4`
- 视频数：`3`
- 归档数（zip/cbz）：`4`
- 无关文件数：`5`
- 路径覆盖：英文 / 中文 / 日文 / 空格 / 深层目录

### 验证场景

- [x] 首轮全量扫描
- [x] 第二轮无变更重扫
- [x] 删除部分文件后重扫
- [x] 新增部分文件后重扫
- [ ] 修改部分文件后重扫（可选）

### 结果记录

#### 首扫

- 发现文件数：`23`
- 插入/更新数：`23`
- 耗时：`3103.29 ms`
- 是否符合预期：符合；结果等于 `总文件数 - 无关文件数`

#### 重扫（无变更）

- `skipped_unchanged`：`23`
- 插入/更新数：`0`
- 耗时：`3243.03 ms`
- 是否符合预期：符合

#### 重扫（删除/新增后）

- tombstone 数：`1`
- 新增数：`1`（通过新增 `english/added-copy.jpg`）
- 插入/更新数：`1`
- 耗时：`3095.05 ms`
- 是否符合预期：符合；删除 `english/0001-page-alpha.jpg` 后出现 `exists=false` 历史记录，同时新文件被纳入 active 集合

#### 任务状态

- success 是否可观测：是
- failed 是否可观测：是（见本文末尾失败/恢复专项验证）
- resume 是否可调用：是；本轮返回 `discovered=23`、`skipped_unchanged=23`

## medium-fixture

### 数据集概况

- 数据集类型：`medium-fixture`
- 数据集名称：`medium-fixture-real`
- 总文件数：`330`
- 图片数：`210`
- 音频数：`30`
- 视频数：`24`
- 归档数（zip/cbz）：`42`
- 无关文件数：`24`
- 路径覆盖：英文 / 中文 / 日文 / 空格 / 深层目录

### 验证场景

- [x] 首轮全量扫描
- [x] 第二轮无变更重扫
- [x] 删除部分文件后重扫
- [x] 新增部分文件后重扫
- [ ] 修改部分文件后重扫（可选）

### 结果记录

#### 首扫

- 发现文件数：`306`
- 插入/更新数：`306`
- 耗时：`6092.31 ms`
- 是否符合预期：符合；结果等于 `总文件数 - 无关文件数`

#### 重扫（无变更）

- `skipped_unchanged`：`306`
- 插入/更新数：`0`
- 耗时：`10644.28 ms`
- 是否符合预期：符合

#### 重扫（删除/新增后）

- tombstone 数：`1`
- 新增数：`1`（通过新增 `english/added-copy.jpg`）
- 插入/更新数：`1`
- 耗时：`30184.26 ms`
- 是否符合预期：符合；删除 `english/0001-page-alpha.png` 后出现一条缺失历史记录，新文件进入 active 集合

#### 任务状态

- success 是否可观测：是
- failed 是否可观测：是（见本文末尾失败/恢复专项验证）
- resume 是否可调用：是；本轮返回 `discovered=306`、`skipped_unchanged=306`

## local-real-dir-mock

### 数据集概况

- 数据集类型：`local-real-dir`
- 数据集名称：`local-real-dir-mock-real`
- 总文件数：`520`
- 图片数：`350`
- 音频数：`45`
- 视频数：`40`
- 归档数（zip/cbz）：`55`
- 无关文件数：`30`
- 路径覆盖：英文 / 中文 / 日文 / 空格 / 深层目录

### 验证场景

- [x] 首轮全量扫描
- [x] 第二轮无变更重扫
- [x] 删除部分文件后重扫
- [x] 新增部分文件后重扫
- [ ] 修改部分文件后重扫（可选）

### 结果记录

#### 首扫

- 发现文件数：`490`
- 插入/更新数：`490`
- 耗时：`6101.98 ms`
- 是否符合预期：符合；结果等于 `总文件数 - 无关文件数`

#### 重扫（无变更）

- `skipped_unchanged`：`490`
- 插入/更新数：`0`
- 耗时：`5887.09 ms`
- 是否符合预期：符合

#### 重扫（删除/新增后）

- tombstone 数：`1`
- 新增数：`1`（通过新增 `english/added-copy.jpg`）
- 插入/更新数：`1`
- 耗时：`5953.67 ms`
- 是否符合预期：符合；删除 `english/0001-page-alpha.png` 后出现一条缺失历史记录，新文件进入 active 集合

#### 任务状态

- success 是否可观测：是
- failed 是否可观测：是（见本文末尾失败/恢复专项验证）
- resume 是否可调用：是；本轮返回 `discovered=490`、`skipped_unchanged=490`

## 失败 / 恢复专项验证

本轮额外在 `run-20260307-fail/small/` 上做了单独验证：

- 先注册库根：`data/scan-validation/runs/run-20260307-fail/small`
- 再临时重命名目录，使 `scan run` 触发失败
- CLI 返回错误：`系统找不到指定的路径。 (os error 3)`
- 恢复目录名后执行 `scan resume`
- `resume` 成功返回：`discovered=23`、`insertedOrUpdated=23`

当前判断：

- 失败路径可被触发
- resume 入口可被调用
- 但当前 `resume` 仍然是“重新跑一轮完整扫描”，不是 checkpoint 续跑

## local 更大变更场景追加验证

本轮又基于：

- `data/scan-validation/runs/run-20260307-local-large/local/`

补做了一轮更大变更场景，目的是验证 `local-real-dir-mock` 在更接近真实批量变更时，扫描统计是否仍然稳定。

### 变更内容

- 删除图片文件：`10`
- 新增图片文件：`6`
- 修改已有文件时间戳：`5`
  - `2` 个 zip/cbz
  - `2` 个 mp4
  - `1` 个 wav

### 结果记录

- 首扫：
  - 发现文件数：`490`
  - 插入/更新数：`490`
  - 耗时：`16241.50 ms`
- 无变更重扫：
  - `skipped_unchanged`：`490`
  - 插入/更新数：`0`
  - 耗时：`6033.75 ms`
- 大变更后重扫：
  - 发现文件数：`486`
  - 插入/更新数：`11`
  - `skipped_unchanged`：`475`
  - tombstone 数：`10`
  - 耗时：`6117.14 ms`
- 扫描后统计：
  - `activeSourceCount=486`
  - `missingSourceCount=10`
  - `sourceCount=496`
- `resume`：
  - `discovered=486`
  - `skipped_unchanged=486`
  - `insertedOrUpdated=0`
  - 耗时：`35709.84 ms`

### 结果判断

- 结果符合预期：`10` 个删除对应 `10` 条 tombstone，`6 + 5` 个新增/修改对应 `11` 条 inserted_or_updated
- 说明当前 `B3` 的路径发现、增量跳过、批量 tombstone、基于 `mtime` 的最小变更识别，在更大本地样本变更下仍然成立
- 但也暴露出一个事实：`resume` 当前仍是整轮重跑，且在大目录下耗时明显偏高，后续不能把它当作真正的恢复能力

## 发现的问题

- 问题 1：`scan resume` 当前仅验证入口可用，本质仍复用完整 `run_scan`，不属于真正恢复语义
- 问题 2：中等规模样本的“删除+新增后重扫”耗时明显高于首扫，说明当前增量路径仍有优化空间
- 问题 3：当前扫描分类仍是扩展名级判断，本轮验证只能证明发现/分类/重扫语义可用，不能证明内容签名级正确性
- 问题 4：`local-real-dir-mock` 大变更场景下，`resume` 耗时达到 `35709.84 ms`，再次说明当前实现缺少真正的失败恢复与续跑优化

## 是否触发设计调整

- [ ] `SourceRecord` 字段
- [ ] 快速指纹规则
- [ ] 扫描统计字段
- [ ] `TaskRecord` 失败语义
- [ ] repository 接口
- [x] 其他：`B3` 结束前继续保留“resume 仍为最小语义”的状态说明，不把它误记为完整恢复能力

## 结论

- 当前结论：通过（B3 阶段真实样本首轮扫描验证通过）
- 下一步动作：将 `local` 大变更结果作为 B3 收尾证据之一，并在 `B4` 开始后加入 zip 索引链路回归
