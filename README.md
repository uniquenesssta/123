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

`verify:frontend` 包含公开模型边界、Node 调用链兼容、Windows 路径契约、TypeScript、静态契约、截图和 Vite 生产构建。TypeScript 与 Vite 使用当前 Node 执行包内 JavaScript CLI，不直接启动 Windows `.cmd` 包装器。Windows 验收器从 `.cargo/target-location.json` 解析实际 Cargo target，并支持相对于项目根目录的 `LogDirectory`；应用 runtime 日志继续写入项目根目录 `logs`。`verify:rust` 包含 Cargo.lock 一致性、格式检查、Clippy 与工作区测试。`verify_protected_assets.mjs` 校验模型公开边界文件指纹、保护目录精确集合以及私有 P4/P7 资产缺席状态。`verify_command_contract.mjs` 校验前端调用、Rust 命令定义和 `generate_handler!` 注册集合一致，并拒绝缺失、重复、孤立或未授权动态命令。`verify_database_baseline.mjs` 校验 0001–0046 迁移连续性、内容指纹、SQLx 迁移入口、PostgreSQL 集成测试集合和关键不可变约束。`run_database_baseline.mjs` 在静态门禁通过后执行被忽略的 PostgreSQL 集成测试，并拒绝数据库名不含 `test` 的连接。

## 模块化重写执行记录

- `new-A` 已从 `main` 基线提交 `db79995873460688c15abb3497bf1c61b73ffb18` 建立。
- R0-01 已冻结远端分支起点并建立 `docs/modular-rewrite/R00-baseline/` 节点记录。
- R0-02 已新增 `architecture/protected-assets.json` 和 `scripts/verify_protected_assets.mjs`，冻结 18 个公开模型边界及校验文件，聚合 SHA-256 为 `d2263a5ff09c8cf633a42b7bb35fffe3d42fb18648db4d12691817f51015c85c`。
- R0-03 已新增 `architecture/command-contract.json` 和 `scripts/verify_command_contract.mjs`，冻结 171 个公共命令、15 个 Rust 命令模块和前端调用边界。
- R0-04 已新增数据库静态基线与安全执行入口，冻结 0001–0046 共 46 个迁移，聚合 SHA-256 为 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`。真实 PostgreSQL 验证按用户要求留到最终统一验证。
- R0-05 workflow run `30910130867` 中，Linux `npm ci` 通过；Chromium 启动失败。Rust locked metadata 通过，但 `cargo fmt --check` 失败，Clippy 与 workspace tests 未执行。
- R0-06 workflow run `30912862564` 建立 Windows 基线。Windows release 构建和 RuntimeOnly startup 通过；startup report 为 PASS，7 条记录、3 个完成操作、0 个无效行、0 个运行时错误。
- R0-06 精确 Automated 暴露 Windows Node 调用问题及验收路径契约问题。
- R0-06.1 新增 `scripts/process/execution-context.mjs`、`scripts/process/node-package-cli.mjs` 和 `scripts/verify-node-process-compatibility.mjs`，关闭目录联接依赖同步与 `.cmd` 子进程调用缺口。Windows workflow run `30919764753` 中完整 frontend 通过，Automated 到达 Rust 阶段。
- R0-06.2 新增 `scripts/windows/acceptance-paths.psm1` 与 `scripts/verify-windows-path-contract.mjs`，支持根入口既有 `LogDirectory` 参数，并按 Cargo target 登记文件查找 release EXE。
- R0-06.2 Windows workflow run `30922384735` 中，完整 frontend、release 构建与 RuntimeOnly runner 均实际通过。release EXE 从项目根目录 `.cargo-target\release` 启动；startup report 为 PASS，7 条记录、3 个完成操作、0 个无效行、0 个运行时错误。
- R0-06.2 证据 artifact 为 `8898312587`，SHA-256 为 `d6ed06066aab354686f86938ec7c55f2c1f740e11a37e42a6a1b5edbbd53df63`。临时 workflow 的 job 最终因产品验证结束后的一条辅助中文日志精确匹配未命中而显示 failure；证据文件已复核，未将 workflow 总体描述为通过。
- R0-07 使用 Rust 1.88.0 rustfmt 对 42 个已诊断 Rust 文件进行纯格式规范化，实施提交为 `9e7be511ae2d97a0782fee1a2bea5e25d910d10d`；未触碰模型保护文件、依赖、锁文件、迁移或公共接口。
- R0-07 精确 workflow run `30961535208` 中，Cargo.lock 门禁、Cargo target 准备和 `cargo fmt --all -- --check` 通过。完整 `npm run verify:rust` 随后在 16 个 Clippy 错误处以退出码 `101` 结束，workspace tests 因 fail-fast 未执行。
- R0-07 精确验证 artifact 为 `8913160029`，SHA-256 为 `47712408cb9fbd37088f42cab92e71565b0c982d5c0492a78fb6c4ef2e53ad49`。
- R0-08 以 11 个 Rust 文件白名单关闭 Clippy 门禁，实施提交为 `919d62a2eaf95ade5ba1efa18924a9d578ef3f63`；没有添加 `allow`、放宽 `-D warnings`、修改依赖、迁移、模型边界或公共接口。
- R0-08 workflow run `30965687503` 中，持久化库测试 73/73、Tauri runtime log 专项 7/7、`cargo clippy --locked --workspace --all-targets -- -D warnings` 均通过。表格库完整测试为 11/12，通过替代验证确认除既有空白行用例外其余 11 项通过。
- R0-08 精确 workflow run `30966064295` 中，Cargo.lock、rustfmt 和 Clippy 连续通过；workspace tests 首个失败为 `openai_research::tests::built_in_gateway_is_strict_and_has_no_secret`，因此完整 `npm run verify:rust` 以退出码 `101` 结束。
- R0-08 最终应用 artifact 为 `8914718704`，SHA-256 为 `b8e75726c6ad53bdb4932ceb0bb3d35ff4554f306179178e6a566187723c6c60`；精确验证 artifact 为 `8914844238`，SHA-256 为 `05ee24344468b9613bf18c139ff7d3aabecb92e005f93afb1f9037ed7f21cede`。
- R0-09 先以 `--no-fail-fast` 完整扫描 workspace tests，确认失败集合只有 OpenAI 提示词大小写断言和表格稀疏测试工作簿两项。
- R0-09 仅修正两个测试契约，共新增 2 行；实施提交为 `50daa258af8ac8e09f8e4f5f428249fe670f2dd2`，未改变生产逻辑、公共接口、数据、配置、依赖、迁移或模型保护边界。
- R0-09 workflow run `30967448070` 中两个专项测试、185 个 workspace 测试以及精确 `npm run verify:rust` 全部通过；18 个 PostgreSQL 集成测试保持忽略并按用户要求留到最终统一验证。
- R0-09 最终 artifact 为 `8915431192`，SHA-256 为 `a745bded71179bb6542d3a06b5c65f61cdf48845b8f88193dc7ef0ac5c8fcadc`；临时 workflow、patch 与 Draft PR #6 均已清理或关闭。
- 用户于 2026-08-05 明确将目标平台收敛为 Windows，Linux Chromium 不再属于交付或阶段门禁；PostgreSQL 实跑、Windows Full 与用户本机实机验收统一延期到最终验收。R00 已完成并开放 R1，详见 `docs/modular-rewrite/R00-baseline/R00-stage-completion.md`。
- R0-01 至 R0-06.2 未修改前端或 Rust 业务源码；R0-07 只改变 Rust 排版；R0-08 仅实施行为等价的私有参数收束、Copy/借用修复、无效私有代码清理和测试编译补全；R0-09 只修改测试构造与断言。
- 当前执行环境未建立本地 Git 工作树，用户设备上的未提交与未跟踪文件不可见；远端分支操作不会覆盖这些本地内容。

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

R0-06.1 已关闭 Windows 目录联接依赖同步和 `.cmd` 子进程调用缺口；R0-06.2 已关闭 `LogDirectory` 参数和 Cargo release 查找路径缺口。Windows 完整 frontend、release 构建与 RuntimeOnly startup 均已有真实通过证据。R0-07 已关闭 Rust `cargo fmt --check` 阻塞；R0-08 已关闭 workspace all-targets Clippy `-D warnings` 阻塞；R0-09 已关闭全部已枚举 workspace tests 失败，精确 `npm run verify:rust` 现已通过。

R00 阶段已按 Windows-only 目标范围标记为 **DONE**。Linux Chromium 历史失败仅保留为非目标平台记录，不再阻塞后续重写。

PostgreSQL 实跑、Windows Full 和用户本机 Windows 10/11 实机验收尚未执行，统一保留到最终验收，不得在后续阶段描述为已通过。另保留 1 个 moderate npm vulnerability 和 Vite 大 chunk 警告。

已创建 `R00-stage-completion.md` 并进入 R1。下一唯一 READY 任务为 `R1-01 模块边界契约`；R1 状态见 `docs/modular-rewrite/R01-architecture-composition/README.md`。