# 足球模型平台公开外壳

当前版本 **0.23.0**。本仓库是平台、数据库、数据准备、路由、工作台与外部模型调用入口的公开版本；真实 P4/P7 预测引擎、参数、Profile、固定比赛、私有研究提示词及模型专用固定回归资产不随仓库分发。

## 公开边界

- 保留 `crates/model-api`、模型 ID、路由、规则包入口、预测页面和历史数据结构。
- 使用 `crates/model-stub` 注册外部模型入口。未接入 ModelProvider 时，预测明确返回“运行时未分发”，不会静默回退、生成伪结果或使用隐藏默认参数。
- 公开规则包只保存外部提供器标识与通用输入输出契约；参数生成、校准、晋升和真实运行由私有或独立部署的 ModelProvider 负责。
- 私有资产由 `.gitignore` 和 `scripts/verify-public-model-boundary.mjs` 双重阻断。

## 构建与验证

```powershell
npm ci
npm run verify:frontend
npm run verify:rust
node scripts/verify_protected_assets.mjs
node scripts/verify_command_contract.mjs
node scripts/verify_database_baseline.mjs
npm run tauri:dev
```

PostgreSQL 数据库基线必须使用名称包含 `test` 的专用、允许彻底清空的测试数据库：

```powershell
$env:FOOTBALL_TEST_DATABASE_URL="postgres://user:password@127.0.0.1:5432/football_test"
node scripts/run_database_baseline.mjs
```

Windows 可使用：

```text
启动平台.bat
验证平台.bat
验收平台.bat
```

`verify:frontend` 包含公开模型边界、TypeScript、静态契约和 Vite 生产构建。`verify:rust` 包含 Cargo.lock 一致性、格式检查、Clippy 与工作区测试。`verify_protected_assets.mjs` 校验模型公开边界文件指纹、保护目录精确集合以及私有 P4/P7 资产缺席状态。`verify_command_contract.mjs` 校验前端调用、Rust 命令定义和 `generate_handler!` 注册集合一致，并拒绝缺失、重复、孤立或未授权动态命令。`verify_database_baseline.mjs` 校验 0001–0046 迁移连续性、内容指纹、SQLx 迁移入口、PostgreSQL 集成测试集合和关键不可变约束。`run_database_baseline.mjs` 在静态门禁通过后执行被忽略的 PostgreSQL 集成测试，并拒绝数据库名不含 `test` 的连接。

## 模块化重写执行记录

- `new-A` 已从 `main` 基线提交 `db79995873460688c15abb3497bf1c61b73ffb18` 建立。
- R0-01 已冻结远端分支起点并建立 `docs/modular-rewrite/R00-baseline/` 节点记录；本节点未修改源码、配置、依赖、接口、数据、模型或运行行为。
- R0-02 已新增 `architecture/protected-assets.json` 和 `scripts/verify_protected_assets.mjs`，冻结 18 个公开模型边界及校验文件，聚合 SHA-256 为 `d2263a5ff09c8cf633a42b7bb35fffe3d42fb18648db4d12691817f51015c85c`；真实 P4/P7 私有资产继续禁止进入公开仓库。
- R0-02 已通过 Node 语法、基准夹具、受保护文件篡改失败、禁止资产失败和 CRLF 兼容验证；完整工作树、Windows 原生和 GitHub Actions 执行限制详见节点记录。
- R0-03 已新增 `architecture/command-contract.json` 和 `scripts/verify_command_contract.mjs`，冻结前端 API、15 个 Rust 命令模块和 `generate_handler!` 中的 171 个公共命令；命令集合及定义映射已建立 SHA-256 门禁。
- R0-03 已通过 Node 语法、171 命令完整合成基准及缺失、重复、孤立、动态命令负向验证；未修改公共命令、DTO、生产源码、配置、依赖或运行行为。
- R0-04 已新增 `architecture/database-baseline.json`、`scripts/verify_database_baseline.mjs` 和 `scripts/run_database_baseline.mjs`，冻结 0001–0046 共 46 个迁移，迁移聚合 SHA-256 为 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`。
- 数据库公共接口、迁移集合和不可变约束继续以 `main` 基线保持。用户于 2026-08-04 明确要求将真实 PostgreSQL 迁移幂等和 18 个集成测试推迟到最终统一验证；这些未执行项不会被描述为通过。
- R0-05 已通过关闭且未合并的 Draft PR #1 触发 GitHub Actions workflow run `30910130867`。`npm ci` 通过，但存在 1 个 moderate npm audit 警告；`npm run verify:frontend` 失败于 GitHub Ubuntu Chromium 未开放调试端口，未进入 UI 截图业务断言。
- R0-05 Rust job 已通过 Rust 1.88.0/Tauri Linux 依赖安装、公开模型边界、Cargo.lock 和 locked metadata；`cargo fmt --all -- --check` 失败，因此 Clippy 与 workspace tests 未执行。Actions 没有按 package script 单入口直接执行完整 `npm run verify:rust`。
- R0-05 只记录现有基线，没有格式化生产源码、替换截图工具、修改依赖、弱化测试或修改正式 CI。
- R0-06 已在 GitHub-hosted Windows Server 2025 建立 Windows 基线。主要 workflow run 为 `30912862564`，证据 artifact 为 `8894874465`，SHA-256 为 `9aacf759cd33bf4c01676465fbefe6bbe657fd7fae037e25a9429d162ea92e76`。
- R0-06 精确 Automated 未通过：依赖同步阶段报告通过，但后续前端专项脚本无法解析 `typescript` 包；单独诊断还确认 `verify-frontend.mjs` 直接启动 Windows `tsc.cmd` 会返回 `spawnSync EINVAL`。
- R0-06 在 runner 临时适配中显式安装依赖并直接执行 TypeScript、Vite、Cargo.lock 门禁和 Tauri release 构建，成功生成 EXE、MSI、NSIS。临时配置和目录联接没有提交到仓库。
- R0-06 RuntimeOnly startup 通过并生成本次独立证据：runtime JSONL 共 7 条记录，`bootstrap`、`read_workspace_state`、`save_workspace_state` 三个操作完成，0 个无效行、0 个运行时错误；startup acceptance report 状态为 PASS。
- R0-06 Full 未执行：需要专用 PostgreSQL 和人工 GUI 业务操作，数据库运行验证按用户要求留到最终统一验证。用户本机 Windows 10/11 也尚未实机复核。
- R0-06 临时 Windows workflow 已删除，Draft PR #1 已关闭且未合并；生产源码、公共接口、依赖、配置、数据库迁移、模型边界和用户可观察行为均未改变。
- 当前执行环境未建立本地 Git 工作树，用户设备上的未提交与未跟踪文件不可见；本次远端分支操作不会覆盖这些本地内容。

## 0.23.0 变更记录

- 删除真实 P4/P7 模型 crate、参数、Profile、固定比赛、模型专用契约、Schema、研究资源和验证脚本。
- 新增外部 `ModelProvider` Stub、通用模型边界契约、公开研究资源和明确不可用错误语义。
- 移除公开代码中的固定模型矩阵拓扑、固定比分单元数、校准常量、时间前推参数算法和自动参数候选生成。
- 保留赛事推演、模型路由、规则包、快照、复盘、分析和数据库入口；未连接外部提供器时不会执行预测。
- 新增 GitHub Actions 自动验证，减少本机重复下载和手工测试。

## 历史兼容记录

以下标题用于保留原项目的可追溯历史契约；不表示本次公开拆分重新执行了所有历史验收。

## 0.22.0 变更记录

接入点 H 的不可变赛后结算、证据评分和监控工作流保留。既有 UI 不重复重构；公开版本只调整模型执行边界。

## 0.19.0 变更记录

工作区 UI、双层导航和页面状态保持历史兼容。

## 0.15.0 变更记录

实体关系、球队与球员管理链路保持历史兼容。

## 0.14.0 变更记录

API 协作工作台、OpenAI Profile 与运行日志链路保持历史兼容。

## 0.13.5 变更记录

API 传输与诊断契约保持历史兼容。

## 0.13.4 变更记录

球队资料、导入和历史记录能力保持历史兼容。

## 0.13.3 变更记录

球队与球员管理修复保持历史兼容。

## 0.13.2 变更记录

球队与球员管理基础链路保持历史兼容。

## 历史能力索引

- 默认战术角色全链路：Excel、档案、阵容、复盘与来源审计。
- 球队完整资料包与 P4 输入就绪度历史工作流；历史数据记录包含 1248 条世界杯球员俱乐部关系增量补录。
- 强制删除全部资料使用 `football.force_purge` 审计边界；永久删除预检会处理陈旧标签页。
- 导入行子记录身份修复保留同一物理行内多实体身份。
- Windows 实机全链路验收阶段 5 由 `验收平台.bat` 启动。

## 验证事实与限制

R0-01 至 R0-06 均已完成各自的基线记录，但 R00 阶段出口仍为 **BLOCKED**。已通过的关键事实包括：保护资产和命令契约静态门禁、46 个迁移静态冻结、Linux 依赖安装、Windows LF 保真 Cargo.lock 校验、17 个 Windows UI 截图视口、Tauri Windows release 构建及 RuntimeOnly startup。

仍未关闭的硬缺口包括：Linux Chromium 启动失败、Rust `cargo fmt --check` 失败、Clippy/workspace tests 未执行、精确 Windows Automated 失败、PostgreSQL 实跑和 Windows Full 未执行、用户本机 Windows 10/11 未实机验收。现有 Windows 根入口还存在未声明参数、`.cmd` 子进程兼容和 Cargo target 查找路径问题。另保留 1 个 moderate npm vulnerability、Vite 大 chunk 和 2 个 Rust dead-code 警告。

因此未创建 `R00-stage-completion.md`，`R1-01` 未进入 READY。完整事实记录见 `docs/public-release/model-boundary/implementation.md` 和 `docs/modular-rewrite/R00-baseline/`。
