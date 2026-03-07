# archive-fixture 说明

本目录用于承载 `B4` 阶段 zip 与归档索引相关的固定 fixture、golden 与 snapshot 说明。

当前阶段约定：

- 仓库内优先保留轻量、稳定、可重复的 golden 结果
- 复杂或体积较大的真实归档样本，继续放在仓库外或 runs 临时目录中验证
- `rar/7z` 暂不纳入当前主链路，本目录当前只围绕 `zip/cbz` 维护

## 当前包含内容

- `zip-page-order.expected.json`：标准页序 golden
- `zip-edge-order.expected.json`：边缘命名页序 golden
- `zip-empty.expected.json`：空归档 golden

## 使用方式

- `media-io` 的归档测试会读取这些 golden 文件，对动态生成的 zip fixture 进行比对
- 这样可以把测试重点放在“页序规则是否稳定”“空归档语义是否稳定”，而不是依赖仓库内二进制大文件

## 后续计划

- 如后续需要固定损坏包样本，可在本目录补最小二进制 fixture 或外部样本说明
- 等 `B4` 更稳定后，再决定是否补仓库内更完整的 archive golden 集
