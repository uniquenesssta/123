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
npm run tauri:dev
```

Windows 可使用：

```text
启动平台.bat
验证平台.bat
验收平台.bat
```

`verify:frontend` 包含公开模型边界、TypeScript、静态契约和 Vite 生产构建。`verify:rust` 包含 Cargo.lock 一致性、格式检查、Clippy 与工作区测试。`verify_protected_assets.mjs` 校验模型公开边界文件指纹、保护目录精确集合以及私有 P4/P7 资产缺席状态。`verify_command_contract.mjs` 校验前端调用、Rust 命令定义和 `generate_handler!` 注册集合一致，并拒绝缺失、重复、孤立或未授权动态命令。数据库集成测试必须连接名称包含 `test` 的专用 PostgreSQL 数据库。

## 模块化重写执行记录

- `new-A` 已从 `main` 基线提交 `db79995873460688c15abb3497bf1c61b73ffb18` 建立。
- R0-01 已冻结远端分支起点并建立 `docs/modular-rewrite/R00-baseline/` 节点记录；本节点未修改源码、配置、依赖、接口、数据、模型或运行行为。
- R0-02 已新增 `architecture/protected-assets.json` 和 `scripts/verify_protected_assets.mjs`，冻结 18 个公开模型边界及校验文件，聚合 SHA-256 为 `d2263a5ff09c8cf633a42b7bb35fffe3d42fb18648db4d12691817f51015c85c`；真实 P4/P7 私有资产继续禁止进入公开仓库。
- R0-02 已通过 Node 语法、基准夹具、受保护文件篡改失败、禁止资产失败和 CRLF 兼容验证；完整工作树、Windows 原生和 GitHub Actions 执行受当前环境或触发条件阻塞，详见节点记录。
- R0-03 已新增 `architecture/command-contract.json` 和 `scripts/verify_command_contract.mjs`，冻结前端 API、15 个 Rust 命令模块和 `generate_handler!` 中的 171 个公共命令；命令集合及定义映射已建立 SHA-256 门禁。
- R0-03 已通过 Node 语法、171 命令完整合成基准及缺失、重复、孤立、动态命令负向验证；未修改公共命令、DTO、生产源码、配置、依赖或运行行为，完整工作树、Windows 和 CI 执行限制详见节点记录。
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
- Windows实机全链路验收阶段 5 由 `验收平台.bat` 启动。

## 验证事实与限制

本公开包生成环境已通过公开模型边界、全部 Node 静态契约、TypeScript、Cargo.lock、发布就绪、迁移哈希和 UI 截图基线检查。Vite 构建因基线内携带的 Windows `node_modules` 缺少 Linux Rollup 原生可选包，且容器无法联网重装而被环境阻塞；最终包不包含 `node_modules`，GitHub Actions 会在 Ubuntu 上执行 `npm ci` 后重新构建。当前容器没有 Rust/Cargo 与 PostgreSQL，因此不会将 Rust 编译、Tauri 启动或真实数据库验证描述为通过。完整事实记录见 `docs/public-release/model-boundary/implementation.md`。
