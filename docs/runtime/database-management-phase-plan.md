# 数据库管理分页实施计划

## 1. 文档定位

本文件用于指导 `设置 -> 数据库管理` 分页的首轮实施。

当前目标不是复刻旧仓全部设置系统，而是为 `MediaPlayerNext` 建立一条可执行、可分阶段验收的数据库管理链路，覆盖：

- 清除数据库
- 选择 SQL 目录
- 选择缩略图目录

本文件是执行型计划文档，按 phase 拆分工作；每个 phase 都明确：

- todo 顺序
- 涉及文件
- 具体要做的内容
- 完成后需要回填的状态 check

## 2. 当前范围与边界

### 2.1 本批要做的内容

- 在设置面板新增 `数据库管理` 分页
- 通过 `MediaRepository` 暴露数据库管理能力，不让组件直接 `invoke`
- 新增运行时信息读取能力：至少返回当前 SQL 路径和缩略图目录
- 新增运行时路径更新能力：支持设置 `database_dir` 和 `thumbnail_cache_dir`
- 新增“清除数据库”能力
- 保证清除数据库后的状态等价于“刚安装、首次打开 App”

### 2.2 本批明确不做的内容

- 不迁移旧仓完整设置系统、Zustand 结构、i18n 文案体系
- 不接复杂帮助文案、tooltip、批量确认流
- 不补测试框架大迁移；只做当前仓库最小验证
- 不在本批处理数据库以外的高级维护入口

## 3. 本批固定语义

### 3.1 清除数据库

`清除数据库` 在新仓里的定义固定为：

> 把应用恢复到“刚安装、首次打开”的状态，不保留任何用户数据或运行时状态。

因此清除动作不能只删 SQLite 文件，还必须清掉所有会影响“初始化状态”的本地持久化数据。

### 3.2 首轮需要清理的内容

首轮按“初始化状态”语义，至少需要清理：

- SQL 数据库文件（含 `-wal` / `-shm`）
- 缩略图缓存目录
- archive normalize 缓存目录
- playback sessions 缓存目录
- 运行时存储路径配置文件（如果本批新增）

说明：

- 这样清完后，数据库路径与缩略图目录会回落到默认解析规则
- 如果后续增加新的本地持久化目录，也必须追加到清理范围

### 3.3 SQL 目录切换

- UI 选择的是目录，不是具体 `.db` 文件
- 后端负责把目录标准化为新的数据库文件路径
- 若旧数据库文件存在有效内容，需要迁移：
  - 主文件
  - `-wal`
  - `-shm`

### 3.4 缩略图目录切换

- UI 选择的是目录
- 后端只切换路径并确保目录存在
- 首轮不迁移旧缩略图缓存

## 4. 当前现状判断

截至当前仓库状态，存在以下缺口：

- 设置面板只有 `界面设置` 单分页
- `MediaRepository` 没有数据库管理域
- 前端 adapter 没有数据库管理相关 commands
- Tauri 宿主没有：
  - `read_runtime_info_command`
  - `set_runtime_storage_paths_command`
  - `clear_database_command`
- Rust 没有运行时存储路径配置文件读写层
- 当前数据库路径与缩略图目录仍完全由环境变量 / dev 默认 / packaged 默认推导

结论：

- 这不是一个“只改设置 UI”的任务
- 必须按 `contracts -> repository -> tauri command -> runtime storage -> UI` 的顺序推进

## 5. Phase 总览

| Phase | 目标 | 状态 |
|---|---|---|
| `Phase 0` | 语义冻结与文件落点准备 | `done` |
| `Phase 1` | contracts / repository / adapter 扩展 | `done` |
| `Phase 2` | Rust runtime info 与 storage path 更新链路 | `done` |
| `Phase 3` | Rust 清除数据库与“初始化状态”恢复 | `done` |
| `Phase 4` | 设置面板新增数据库管理分页并接 3 个动作 | `done` |
| `Phase 5` | 验证、日志、文档回填 | `done（CLI 验证）` |

---

## 6. Phase 0：语义冻结与文件落点准备

### 6.1 todo 顺序

1. 冻结数据库管理分页的 3 个动作与语义
2. 冻结“清除数据库 = 初始化状态”的清理范围
3. 冻结运行时存储配置文件位置与回落顺序
4. 把本计划文档加入 docs 索引

### 6.2 涉及文件

- `docs/runtime/database-management-phase-plan.md`
- `docs/README.md`
- `docs/runtime/resource-path-strategy.md`

### 6.3 要做的内容

- 明确数据库管理分页首轮只做：
  - 清除数据库
  - 选择 SQL 目录
  - 选择缩略图目录
- 明确运行时存储配置文件建议位置：
  - 开发态：`data/runtime-storage-paths.json`
  - 打包态：`app local data dir/runtime-storage-paths.json`
- 明确数据库路径解析顺序改为：
  - env override
  - runtime storage config
  - dev / packaged 默认值

### 6.4 phase 完成后状态 check

- [x] `清除数据库` 的语义已经固定成“初始化状态”
- [x] 运行时存储路径配置文件位置已固定
- [x] docs 索引可找到本计划文档

---

## 7. Phase 1：contracts / repository / adapter 扩展

### 7.1 todo 顺序

1. 在 contracts 定义最小数据库管理 DTO
2. 扩展桌面端 command adapter
3. 扩展 `MediaRepository` 数据库管理域
4. 更新 repository surface / dependency 文档（如需要）

### 7.2 涉及文件

- `packages/contracts/src/`（新增或补充 runtime/database 相关 model）
- `packages/contracts/src/index.ts`
- `apps/desktop/src/adapters/tauri/commands.ts`
- `apps/desktop/src/repositories/media-repository.ts`
- `apps/desktop/src/repositories/tauri-media-repository.ts`
- `docs/contracts/media-repository-surface.md`

### 7.3 要做的内容

建议新增最小类型：

```ts
export interface RuntimeInfo {
  databasePath: string
  thumbnailCachePath: string
}

export interface SetRuntimeStoragePathsInput {
  databaseDir?: string
  thumbnailCacheDir?: string
}
```

建议在 `MediaRepository` 增加：

```ts
database: {
  readRuntimeInfo(): Promise<RuntimeInfo>
  setStoragePaths(input: SetRuntimeStoragePathsInput): Promise<RuntimeInfo>
  clear(): Promise<void>
}
```

### 7.4 phase 完成后状态 check

- [x] contracts 已有数据库管理最小 DTO
- [x] adapter 已能调用 3 个数据库管理 command
- [x] `MediaRepository` 已暴露 `database.*`
- [x] 组件层不需要直接 `invoke`

---

## 8. Phase 2：Rust runtime info 与 storage path 更新链路

### 8.1 todo 顺序

1. 新增 runtime storage 配置结构与文件读写层
2. 新增当前运行时信息读取 command
3. 新增设置运行时存储路径 command
4. 修改 command environment 的路径解析顺序
5. 验证开发态和打包态默认回落不被破坏

### 8.2 涉及文件

- `src-tauri/src/lib.rs`
- `src-tauri/src/`（建议新增如 `runtime_storage.rs`）
- `docs/runtime/resource-path-strategy.md`

### 8.3 要做的内容

建议新增 Rust 侧能力：

- `read_runtime_info_command`
  - 返回当前：
    - `database_path`
    - `thumbnail_cache_path`
- `set_runtime_storage_paths_command`
  - 接受：
    - `database_dir`
    - `thumbnail_cache_dir`
  - 返回更新后的 runtime info

建议新增配置文件结构：

```json
{
  "database_dir": "...",
  "thumbnail_cache_dir": "..."
}
```

建议新增路径解析逻辑：

- database path
  - `MPNEXT_BACKEND_DB_PATH`
  - runtime storage config 中的 `database_dir`
  - dev default / packaged default
- thumbnail cache root
  - `MPNEXT_BACKEND_THUMB_CACHE_ROOT`
  - runtime storage config 中的 `thumbnail_cache_dir`
  - dev default / packaged default

SQL 目录切换时要处理：

- 新目录标准化
- 目标数据库文件路径生成
- 如果旧 DB 文件存在：迁移 `db / -wal / -shm`
- 写回 runtime storage config

缩略图目录切换时要处理：

- 新目录标准化
- `create_dir_all`
- 写回 runtime storage config
- 不迁移旧缩略图缓存

### 8.4 phase 完成后状态 check

- [x] 宿主可返回当前 SQL 路径与缩略图路径
- [x] 宿主可持久化设置 `database_dir`
- [x] 宿主可持久化设置 `thumbnail_cache_dir`
- [x] 切换 SQL 目录时已处理 `db / -wal / -shm` 迁移
- [x] 路径解析顺序已切到 `env -> config -> default`

---

## 9. Phase 3：Rust 清除数据库与“初始化状态”恢复

### 9.1 todo 顺序

1. 先冻结清理范围
2. 新增 `clear_database_command`
3. 实现数据库文件与缓存目录清理
4. 清理 runtime storage 配置文件
5. 清理后重新回落到默认路径
6. 返回成功后由前端 reload

### 9.2 涉及文件

- `src-tauri/src/lib.rs`
- `src-tauri/src/`（建议与 runtime storage / cleanup 逻辑拆模块）
- `docs/runtime/resource-path-strategy.md`

### 9.3 要做的内容

清理范围按“初始化状态”固定为：

- 当前 SQL 数据库文件
- 当前 SQL `-wal`
- 当前 SQL `-shm`
- 当前缩略图缓存目录
- 当前 normalize 缓存目录
- 当前 playback sessions 目录
- runtime storage config 文件

清理完成后的要求：

- 下次读取 runtime info 时回落到默认路径
- 当前主界面 reload 后表现等价于首次打开
- 不残留此前选择过的数据库目录或缩略图目录设置

### 9.4 phase 完成后状态 check

- [x] `clear_database_command` 已可调用
- [x] SQL 文件与缓存目录已全部清掉
- [x] runtime storage config 已删除
- [x] reload 后应用回到初始化状态

---

## 10. Phase 4：设置面板数据库管理分页 UI 接线

### 10.1 todo 顺序

1. 给设置面板增加分页状态
2. 左侧新增 `数据库管理` 分页按钮
3. 渲染数据库管理分页主体
4. 接 `readRuntimeInfo`
5. 接 `选择 SQL 目录`
6. 接 `选择缩略图目录`
7. 接 `清除数据库`
8. 补 pending / success / error 展示

### 10.2 涉及文件

- `apps/desktop/src/app/AppShell.tsx`
- `apps/desktop/src/App.css`
- `apps/desktop/src/app/use-media-repository.ts`
- `apps/desktop/src/app/ImportTaskPanel.tsx`（通常不需要改，除非要复用状态块）

### 10.3 要做的内容

UI 首轮建议结构：

- 左侧分页按钮：
  - `界面设置`
  - `数据库管理`
- `数据库管理` 分页主体：
  1. `清除数据库`
     - 按钮
     - pending 文案
     - error 文案
     - 成功后 `window.location.reload()`
  2. `SQL 目录`
     - 只读路径显示
     - `选择 SQL 目录` 按钮
  3. `缩略图目录`
     - 只读路径显示
     - `选择缩略图目录` 按钮

交互建议：

- 目录选择继续复用当前 Tauri dialog plugin
- `清除数据库` 先弹 `confirm`
- 目录选择成功后立即写入后端并刷新显示值
- 目录写入成功后显示简短提示，如“目录已保存”

### 10.4 phase 完成后状态 check

- [x] 设置面板已存在 `数据库管理` 分页
- [x] SQL 路径可显示、可修改
- [x] 缩略图目录可显示、可修改
- [x] 清除数据库按钮已接通并带确认
- [x] 成功清除后会 reload

---

## 11. Phase 5：验证、日志、文档回填

### 11.1 todo 顺序

1. 跑前端构建
2. 跑运行时检查
3. 手动验证 3 个动作
4. 回填 UI 文档
5. 回填日志

### 11.2 涉及文件

- `docs/ui/ui-definition.md`
- `docs/logs/20260311.md` 或当日日志
- 如有必要：`docs/runtime/resource-path-strategy.md`

### 11.3 要做的内容

最小验证清单：

1. `npm run build:web`
2. `npm run check`
3. 手动联调：
   - 打开设置面板 -> 数据库管理分页
   - 选择 SQL 目录，重启后仍生效
   - 选择缩略图目录，重启后仍生效
   - 清除数据库，reload 后回到初始化状态

### 11.4 phase 完成后状态 check

- [x] 构建通过
- [x] runtime check 通过
- [ ] 3 个动作已手动验证
- [x] 文档与日志已同步

补充说明：

- 当前已在 CLI 环境完成 `npm run build:web`、`npm run check`、Rust 单元测试与 Tauri 启动尝试
- `npm run tauri:dev` 本轮被现有 `1420` 端口占用阻塞，未在交互界面内完成最终手动点验

---

## 12. 推荐执行顺序

严格按下面顺序推进，不建议跳 phase：

1. `Phase 1` 先补 `contracts / repository / adapter`
2. `Phase 2` 再补 runtime info / storage path
3. `Phase 3` 再补 clear database
4. `Phase 4` 最后接设置 UI
5. `Phase 5` 做验证与文档回填

原因：

- 若先做 UI，会被后端能力面卡住
- 若先做 clear database，但 runtime storage 还没建立，后面一定返工
- 只有把 runtime path 与 reset 语义先立住，设置分页才不会变成一次性假 UI

## 13. 当前执行状态面板

### 13.1 总状态

- `Phase 0`：`done`
- `Phase 1`：`done`
- `Phase 2`：`done`
- `Phase 3`：`done`
- `Phase 4`：`done`
- `Phase 5`：`done（CLI 验证）`

### 13.2 当前下一步

当前批次代码、文档与 CLI 验证已经完成。

仍待进入交互环境补的最后一项是：

- 打开 `设置 -> 数据库管理`，手动点验 `选择 SQL 目录 / 选择缩略图目录 / 清除数据库` 三个 UI 动作
