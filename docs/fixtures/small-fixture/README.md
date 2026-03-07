# small-fixture 说明

本目录用于承载 `B3` 阶段本地快速 smoke、单元/集成测试和 CLI 最小链路验证所需的数据集。

当前目录分层：

- `scan-smoke/`：当前已在使用的最小扫描快照样本
- `generated-placeholder/`：通过脚本生成的合规占位数据集

## 当前约定

- `generated-placeholder/` 中的文件是真实存在的占位文件，不是纯文件名清单
- 图片、音频、归档默认会生成可实际打开或可实际解包的最小文件
- 若本机 `ffmpeg` 可用，视频也会生成真实可读的小文件；否则回退为最小占位字节文件
- 多语言路径覆盖允许复用同一份底层文件，仅通过目录名和文件名差异覆盖英文、中文、日文与空格路径

## 生成方式

在仓库根目录执行：

```bash
npm run fixtures:scan-placeholders
```

执行后会更新：

- `docs/fixtures/small-fixture/generated-placeholder/`
- `docs/fixtures/medium-fixture/generated-placeholder/`
- `data/scan-validation/local-real-dir-mock/`

若已经准备了真实样本池：

```bash
npm run fixtures:fill-real
```

该命令会用 `docs/fixtures/realdatasmall/` 和 `docs/fixtures/realdata/` 中的真实文件填充三个基线目录。

## 替换真实文件的要求

- 后续进入真实扫描验证时，可直接用同类真实文件替换 `generated-placeholder/` 中的占位文件
- 优先保持目录层级、路径语言覆盖和文件类型比例不变
- 若替换 `data/scan-validation/local-real-dir-mock/`，无需提交到 Git，只记录验证结果

## 日常测试方式

- 不直接在基线目录上做删除/新增/改时间戳测试
- 先复制到 `data/scan-validation/runs/<run-name>/small/`
- 在临时副本上执行扫描测试，测试后删除临时目录
