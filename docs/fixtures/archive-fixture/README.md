# archive-fixture 说明

本目录用于承载 `B4-B6` 阶段归档索引、归一化与相关 golden / snapshot 说明。

当前阶段约定：

- 仓库内优先保留轻量、稳定、可重复的 golden 结果
- 复杂或体积较大的真实归档样本，继续放在仓库外或 runs 临时目录中验证
- `rar/7z` 在 `B6` 已进入“低频归一化后再复用 zip 主链路”的路线

## 当前包含内容

- `zip-page-order.expected.json`：标准页序 golden
- `zip-edge-order.expected.json`：边缘命名页序 golden
- `zip-empty.expected.json`：空归档 golden
- `normalize-layout.expected.json`：归一化输出布局 golden
- `normalize-status.expected.json`：归一化状态与任务语义 golden

## 使用方式

- `media-io` 的归档测试会读取这些 golden 文件，对动态生成的 zip fixture 进行比对
- 这样可以把测试重点放在“页序规则是否稳定”“空归档语义是否稳定”，而不是依赖仓库内二进制大文件
- `B6` 当前不把真实 `rar/7z` 二进制样本直接纳入 Git，而是先固定：
  - 归一化输出布局
  - 归一化状态语义
  - mock extractor 驱动下的回归验证

## 后续计划

- 如后续需要固定损坏包 / 密码包样本，可在本目录补最小二进制 fixture 或外部样本说明
- 如后续需要更强 `B6` 回归，可补仓库外真实 `rar/7z` 验证记录
