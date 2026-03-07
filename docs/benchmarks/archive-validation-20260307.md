# 2026-03-07 归档索引验证记录（B4）

本记录用于确认 `B4` 阶段 zip 与归档索引主链路已经具备：

- zip 目录读取
- 图片页判断
- 页序排序
- entry 读取
- `archives` / `archive_entries` 落库
- `archive show` / `archive read-entry` 查询

本轮验证基于：

- `data/scan-validation/runs/run-20260307-b4-closeout/small/`
- `data/scan-validation/runs/run-20260307-b4-closeout/medium/`

## 基本信息

- 日期：`2026-03-07`
- 阶段：`B4`
- 验证入口：`scan run` / `archive index` / `archive show` / `archive read-entry`
- 验证目标：确认真实样本基线下的 zip 索引、空归档语义与 entry 读取能力

## small-fixture

### 数据集概况

- 数据集名称：`small-fixture-real`
- 扫描发现文件数：`23`
- 归档源数量：`4`

### 结果记录

- `archive index`：
  - `indexedArchives=4`
  - `indexedEntries=0`
  - `skippedNonPrimaryArchives=0`
- `archive show` 抽样结果：
  - 抽样范围内 `4` 个 archive source 均为 `empty` 状态
  - 当前未发现可供 `archive read-entry` 的图片页 entry

### 结果判断

- 结果符合预期：`small` 基线里的 zip/cbz 当前更像“归档文件存在但内部不包含可索引图片页”的空归档样本
- 这证明 `B4` 对“归档存在但 page_count=0”的语义已经能稳定表达，而不是把它们错误地当成损坏索引

## medium-fixture

### 数据集概况

- 数据集名称：`medium-fixture-real`
- 扫描发现文件数：`306`
- 归档源数量：`42`

### 结果记录

- `archive index`：
  - `indexedArchives=42`
  - `indexedEntries=1368`
  - `skippedNonPrimaryArchives=0`
- `archive show` 抽样结果：
  - 样本 source：`source_a4c2c49d8b46a4ef`
  - 文件名：`0001-chapter-alpha.zip`
  - `archiveStatus=indexed`
  - `pageCount=33`
  - `firstEntryPath=01.webp`
- `archive read-entry` 抽样结果：
  - `entryPath=01.webp`
  - `byteCount=324404`
  - `previewHex=524946462cf304005745425056503820`

### 结果判断

- 结果符合预期：`medium` 真实样本中已经能稳定索引出图片页，并且能够按指定 entry 读取出实际字节流
- 这说明 `B4` 的 zip 主链路已经不是“只能写索引，不能读 entry”，而是已经具备后续给缩略图和归档浏览复用的输入面

## 当前结论

- `B4` 计划内的 zip 高频主链路已闭合：通过
- 当前剩余工作不再是“让 B4 能成立”，而是把它继续衔接到 `B5` 的 asset / thumbnail 输入面

## 下一步动作

- 在 `B5` 中继续把 archive entry 与 `media_assets`、缩略图链路接通
- 后续继续复用 `data/scan-validation/runs/` 做归档回归，而不直接改动真实样本基线
