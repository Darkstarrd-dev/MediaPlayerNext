# medium-fixture 说明

本目录用于承载 `B3` 扫描验证和首轮 benchmark 所需的中等规模负载数据集。

当前约束：

- 仓库内先保留说明文档、目录规范、验证口径
- 如样本体积过大，不强制把完整负载文件纳入 Git
- 若后续补充实际样本，优先保持可复现、可统计、可替换

## 1. 目标

`medium-fixture` 的作用不是替代 `small-fixture`，而是验证：

- 首轮全量扫描在中等规模下是否稳定
- 重扫时 `skipped_unchanged` 是否可信
- 删除/新增文件后，tombstone 与重扫结果是否正确
- 图片、音频、视频、归档、无关文件混合时，分类是否符合预期
- 中文、英文、日文、空格路径是否会影响扫描结果

## 2. 数据集最低要求

### 2.1 文件规模

- 总文件数：300~1000
- 建议首版目标：500 左右

### 2.2 类型分布

- 图片：40%~70%
- 音频：5%~20%
- 视频：5%~20%
- 归档：10%~30%（当前以 `zip/cbz` 为主）
- 无关文件：5%~15%

### 2.3 路径与命名分布

至少覆盖：

- 英文路径
- 中文路径
- 日文路径
- 空格路径
- 连续编号文件名
- 非连续编号文件名
- 长文件名
- 至少 3 层目录深度

## 3. 文件内容要求

- 不能全部使用占位文件
- 至少一部分必须是真实可解析媒体文件
- 当前阶段优先保证以下样本真实可用：
  - 图片
  - 音频
  - 视频
  - zip/cbz

说明：

- `B3` 目标是验证扫描发现、分类、重扫与 tombstone 语义
- 因此部分样本可以保留为轻量文件
- 但不能让整个数据集退化成“只有文件名和后缀”的假样本集合

## 4. 推荐目录结构

```text
docs/fixtures/medium-fixture/
  README.md
  manifest.example.json
  reports/
  sample-layout/
```

说明：

- `manifest.example.json`：记录数据集规模和类型分布示例
- `reports/`：记录扫描验证结果
- `sample-layout/`：仅用于说明目录分布，不要求放真实大文件

## 5. 验证场景要求

每轮 `medium-fixture` 验证至少覆盖：

1. 首轮全量扫描
2. 第二轮无变更重扫
3. 删除部分文件后重扫
4. 新增部分文件后重扫

可选补充：

5. 修改部分文件时间戳后重扫
6. 修改部分文件内容或大小后重扫

## 6. 输出要求

每轮验证至少记录：

- 数据集规模
- 类型分布
- 路径分布（中/英/日/空格）
- 首扫结果
- 重扫结果
- tombstone 结果
- 发现的问题
- 是否触发模型/字段/策略调整

## 7. 当前状态

- 目录规范：已建立
- 验证口径：已建立
- 真实样本：待补（当前可先用 `generated-placeholder/` 占位）
- 扫描报告：待补

## 8. 占位数据生成

为避免 `medium-fixture` 长期停留在纯文档状态，当前已约定先通过脚本生成一套可扫描的占位数据集：

```bash
npm run fixtures:scan-placeholders
```

如需明确重建现有目录，才使用：

```bash
npm run fixtures:scan-placeholders:force
```

若 `docs/fixtures/realdata/` 与 `docs/fixtures/realdatasmall/` 已经就位，则应改用：

```bash
npm run fixtures:fill-real
```

该命令会把：

- `docs/fixtures/realdatasmall/` 填充到 `small-fixture` 基线目录
- `docs/fixtures/realdata/` 填充到 `medium-fixture` 基线目录
- `docs/fixtures/realdata/` 同步填充到 `data/scan-validation/local-real-dir-mock/`

生成结果位于：

- `docs/fixtures/medium-fixture/generated-placeholder/scan-root/`

若目录下存在：

- `docs/fixtures/medium-fixture/generated-placeholder/realdata/`

则脚本会优先从 `realdata/` 中按类型随机抽样，复制到 `scan-root/` 对应位置；不足的类型再回退到脚本内置最小样本。

生成完成后：

- `realdata/` 仅作为一次性样本池使用，会被自动清理
- 最终保留用于扫描验证的目录只有 `scan-root/`
- 后续再次执行脚本时，如果没有新的 `realdata/` 样本池，现有 `scan-root/` 会被保留，不会被脚本自动重置

使用约束：

- 该目录用于当前阶段的结构验证、扫描命名覆盖和路径覆盖
- 实际扫描时应以 `scan-root/` 作为库根目录，不应直接把 `realdata/` 一并纳入扫描根
- 该目录不纳入 Git，按本地生成产物处理
- `realdata/` 中当前优先识别并替换的类型是：图片、音频、视频、`zip/cbz`、其他文件
- 不在当前扫描主链路内的扩展名会被当作“其他文件”来源，不会改变当前 B3 的分类边界
- 后续可用同类真实文件逐步替换，占位文件本身不是最终验证终点
- 替换时优先保持类型分布、目录层级、中英日与空格路径覆盖不变

## 9. local-real-dir-mock 同步规则

对 `data/scan-validation/local-real-dir-mock/` 也执行同样的策略：

- 若存在 `data/scan-validation/local-real-dir-mock/realdata/`，脚本会优先按类型抽样生成最终 mock 目录
- 生成完成后 `realdata/` 会被自动清理，不保留在最终扫描根中
- 若后续没有新的 `realdata/`，现有 mock 目录会被保留

## 10. runs 临时目录

日常扫描测试不直接修改基线目录，而是统一复制到：

- `data/scan-validation/runs/<run-name>/small/`
- `data/scan-validation/runs/<run-name>/medium/`
- `data/scan-validation/runs/<run-name>/local/`

生成命令：

```bash
npm run fixtures:create-run -- <run-name>
```

测试结束后，直接删除对应 `run-name` 目录即可。
