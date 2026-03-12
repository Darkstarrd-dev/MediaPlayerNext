# Tauri 自动化 E2E 验收方案

## 1. 文档定位

本文件用于定义 `MediaPlayerNext` 的桌面端自动化 E2E（End-to-End）验收方案。

目标不是立刻把全仓所有桌面流程都接进自动化，而是先固定一套后续可持续扩展的验收路径，让代理在完成相关桌面任务后，可以明确询问用户是否需要执行自动 E2E 验收，而不是默认回退到手点窗口。

## 2. 当前结论

截至 `2026-03-13`，基于 Tauri 2 官方 WebDriver 文档，当前推荐基线固定为：

- 桌面 E2E 主方案：`tauri-driver + WebdriverIO`
- 当前优先目标平台：`Windows`
- Linux 后续可跟进同一路线
- macOS 不作为当前仓库默认官方验收基线

说明：

- Tauri 2 官方 WebDriver 桌面支持当前基于 `tauri-driver`
- 官方基线对桌面端主要覆盖 `Windows / Linux`
- 当前仓库主要开发环境就是 Windows，因此不为首轮方案再引入额外跨平台复杂度

## 3. 为什么不用“纯浏览器 E2E”替代

- 本仓库关键验收对象是 `Tauri 2 + Rust + React` 的桌面闭环，而不是单纯的 Vite 页面
- 很多关键流程依赖：
  - Tauri command
  - protocol handler
  - 本地文件路径
  - 桌面窗口行为
- 只跑浏览器侧 Playwright 无法覆盖真正的桌面宿主链路

结论：

- 浏览器级验收只能作为前端补充
- 桌面验收基线必须保留 Tauri WebDriver 路线

## 4. 方案选型

### 4.1 Runner 选择

首轮固定：

- WebDriver Server：`tauri-driver`
- Test Runner：`WebdriverIO`

原因：

- 有 Tauri 2 官方示例
- 与桌面 WebDriver 语义直接对齐
- 对应用窗口、元素交互、等待条件、截图更直接
- 在 Windows CI 与本地 Windows 环境里可执行性最高

### 4.2 目录建议

首轮建议新增独立测试工作区：

- `tests/desktop-e2e/`

建议结构：

```text
tests/desktop-e2e/
  package.json
  wdio.conf.mjs
  specs/
    settings-database.e2e.ts
  fixtures/
  utils/
```

说明：

- 不把 E2E 代码塞进 `apps/desktop`
- 让桌面验收依赖、fixture、测试工具与业务代码解耦

## 5. 首轮覆盖范围

### 5.1 必测类型

优先覆盖这几类桌面任务：

- 设置面板
- 导入面板
- destructive flow（破坏性流程）
- 前端 + Tauri command 的闭环
- 刷新后状态变化可验证的流程

### 5.2 首轮推荐用例

第一批建议最先落地：

1. `设置 -> 数据库管理 -> 清除数据库`
   - 打开设置
   - 切到数据库管理分页
   - 点击 `清除数据库`
   - 断言只出现 `4 小面板层` 确认框
   - 点击 `取消` 后无副作用
   - 点击 `确认清除` 后进入清理并 reload

2. `设置 -> 数据库管理 -> 读取当前路径`
   - 断言 SQL 路径与缩略图目录能读取并显示

3. `导入入口基础开关`
   - 打开 Header Logo 大面板
   - 断言导入面板状态块与基础按钮可见

4. `设置分页切换`
   - `界面设置` 与 `数据库管理` 可切换
   - 切换后主体内容正确刷新

## 6. 原生对话框处理原则

### 6.1 明确不直接自动化的对象

以下内容不作为首轮直接 WebDriver 自动化对象：

- 系统文件夹选择器
- 系统原生确认框
- OS 级拖拽文件选择窗口

原因：

- 这些对象稳定性差
- 不同平台实现差异大
- 容易把失败原因混入系统环境而不是应用逻辑

### 6.2 正确处理方式

对于 `选择 SQL 目录`、`选择缩略图目录` 这类流程，首轮一律走“测试替身（test double）”策略：

- 在测试模式下，为目录选择器提供固定返回值
- 业务 UI 仍走正常按钮链路
- 但底层 adapter 不真正弹系统目录选择器

当前仓库实现方式：

- 前端通过 `window.__MPNEXT_E2E__.directorySelections` 注入下一次目录选择结果
- `AppShell` 在真正调用 Tauri dialog plugin 前，会先消费这组 test double 队列
- 若未注入测试值，则仍回到正式的系统目录选择器链路

建议约束：

- 测试替身必须是 `dev / test only`
- 不允许影响正式构建
- 必须在文档中写明注入入口、启用条件与回退行为

### 6.3 运行时隔离

数据库管理 E2E 不允许直接污染开发中的真实数据库与缓存目录。

当前仓库实现方式：

- `wdio.conf.mjs` 会为每次桌面 E2E 运行创建独立临时目录
- `wdio.conf.mjs` 采用“优先复用 env runtime context，缺失时才创建”的策略，避免 worker 进程各自创建不同 session root
- 通过以下环境变量把 Tauri 宿主重定向到隔离路径：
  - `MPNEXT_RUNTIME_STORAGE_CONFIG_PATH`
  - `MPNEXT_RUNTIME_DEFAULT_DATA_DIR`
  - `MPNEXT_RUNTIME_DEFAULT_CACHE_DIR`
  - `MPNEXT_BACKEND_NORMALIZE_ROOT`
  - `MPNEXT_BACKEND_PLAYBACK_SESSIONS_ROOT`
- `clear_database_command` 在 E2E 中清掉的是隔离环境，不会误删日常开发数据

补充稳定性约束：

- `tauri-driver` 只在 `onPrepare` 启动一次，`onComplete` 统一关闭
- 不在 `beforeSession/afterSession` 按 worker 启停，避免重复抢占 `127.0.0.1:4444`

## 7. 首轮实施顺序

### Phase A：E2E 基础脚手架

- [x] 新增 `tests/desktop-e2e`
- [x] 接入 `WebdriverIO`
- [x] 配置 `tauri-driver`
- [x] 固定本地运行命令与 CI 命令

### Phase B：数据库管理验收用例

- [x] 补 `设置 -> 数据库管理` 基础定位器
- [x] 接 `清除数据库` 确认流用例
- [x] 接 runtime info 显示用例

### Phase C：目录选择测试替身

- [x] 给目录选择器补测试模式注入
- [x] 接 SQL / 缩略图目录切换用例

### Phase D：纳入默认验收流程

- 对相关桌面任务，代理完成实现后主动询问：
  - 是否需要继续执行自动 E2E 验收
- 若用户确认，则优先跑对应最小 E2E 集合，而不是要求用户手点

## 8. 建议命令约定

当前仓库已接入的统一脚本名：

- `npm run e2e:desktop`
  - 运行桌面 E2E 全量验收
- `npm run e2e:desktop:headed`
  - 需要可视窗口时运行
- `npm run e2e:desktop:doctor`
  - 检查 `tauri-driver` 与 `msedgedriver` 前置条件
- `npm run check:quality:release`
  - 分层质量流水线的发布级入口
  - 当前串联：`check:quality:heavy` + `check:release` + `e2e:desktop:doctor` + `e2e:desktop`

补充：

- 当前也可通过 `npm run e2e:desktop -- --spec=./specs/shell-smoke.e2e.mjs` 定向跑某个 spec
- 当前首轮内置 smoke 用例为：`tests/desktop-e2e/specs/shell-smoke.e2e.mjs`
- 当前数据库管理主用例为：`tests/desktop-e2e/specs/settings-database.e2e.mjs`

注意：

- 数据库管理页已经具备完整自动验收与目录选择 test double
- 当前 Windows 本地前置条件已经过验证：
  - `tauri-driver.exe`
  - 与本机 Edge `145.0.3800.97` 匹配的 `msedgedriver.exe`

## 9. 触发规则

从现在开始，以下任务完成后，代理应在最终说明里主动询问用户是否需要自动 E2E 验收：

- Tauri 桌面 UI 交互任务
- 设置面板相关任务
- 导入 / 拖拽 / 粘贴等用户流程任务
- 带确认步骤的 destructive flow
- 前端 + Tauri command + 本地状态的闭环任务

默认提问模板建议为：

> 这批改动属于桌面交互闭环；如果你要，我下一步可以继续做自动 E2E 验收。

要求：

- 只在“相关任务”完成后提示
- 提示要简短，不抢占主结论
- 若仓库尚未落地 E2E 脚手架，要明确说明“当前是方案已定、脚手架待接入”

## 10. 当前落地状态

截至本文件创建时：

- E2E 方案：`已确定`
- 官方技术路线核对：`已完成`
- 仓库脚手架：`Phase A 已接入`
- 自动 E2E 命令：`已接入`
- AGENTS 工作流提示：`已同步纳入`
- `doctor` 前置检查：`已通过`
- `shell smoke`：`已通过`
- `Phase B` 数据库管理验收：`已通过`
- `Phase C` 目录选择 test double：`已通过`
- 当前桌面 E2E 总状态：`4 specs / 6 cases passed`

## 11. 当前不做的事

- 不在本次仅文档任务中直接接入 WebdriverIO 依赖
- 不在本次仅文档任务中修改 Tauri 宿主以接测试替身
- 不在当前阶段引入额外第三方商用 E2E 服务
- 不为了 E2E 先行改动正式运行时行为
