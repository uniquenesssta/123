# 公开模型边界处理记录

## 处理目标

以用户提供的 `fdd.zip` 为唯一代码基线，生成可重新初始化为公开 Git 仓库的平台源码包。公开包保留应用、数据库、工作台、路由、P4/P7 模型标识与调用入口，但不分发真实预测引擎、模型参数、Profile、固定比赛、私有研究提示词或模型专用固定回归资产。

## 基线与工作区

- 基线来源：`/mnt/data/fdd.zip`
- 隔离工作区：`/mnt/data/fdd-public-work`
- 工作分支：`public-sanitize`
- 基线提交：`b265d6885cfdf9366e059ff2d4eb1026078ec9f7`
- 实施前已检查工作区状态；基线提交后工作区干净。
- 未向原压缩包或用户私有仓库写入内容。

## 已删除的私有资产

- `crates/model-p4/`
- `crates/model-p7/`
- `src-tauri/resources/defaults/`
- `src-tauri/resources/research/p4_*`
- `contracts/p4-*`
- `schemas/p4-*`
- `scripts/verify-p4-*`
- `docs/P4_INTEGRATION.md`

同时删除了仅服务于已移除私有实现、且无法在公开边界内继续成立的陈旧验证脚本。公开包的 `.gitignore` 继续阻断上述路径和未来同类资产。

## 新增的公开模型边界

### 模型 API 与 Stub

新增 `crates/model-stub/`，通过 `football-model-api` 注册原有 P4/P7 入口标识。Stub 只描述外部提供器能力，不包含算法或参数；调用预测时返回 `ModelError::Unavailable`，不会：

- 回退到隐藏默认模型；
- 生成模拟概率冒充正式结果；
- 从公开仓库加载私有参数；
- 静默忽略模型运行失败。

`crates/model-api/src/lib.rs` 新增明确的 `Unavailable(String)` 错误语义。

### 应用层模型壳

新增：

- `crates/application/src/model_shell/mod.rs`
- `crates/application/src/model_shell/fixtures.rs`

该模块集中负责公开模型入口标识、通用演示输入和外部提供器参数信封。`ApplicationService` 改为注册 `PublicModelStub`，不再直接依赖真实 P4/P7 crate，也不再通过 `include_str!` 编译私有参数和固定比赛。

### 通用契约与资源

新增：

- `contracts/model-provider-boundary-contract.json`
- `contracts/model-integration-contract.json`
- `contracts/model-persistence-contract.json`
- `contracts/research-gateway-contract.json`
- `contracts/fact-pipeline-contract.json`
- `contracts/model-orchestration-contract.json`
- `contracts/model-workbench-contract.json`
- `schemas/research-output.schema.json`
- `schemas/evidence.schema.json`
- `schemas/prematch-snapshot.schema.json`
- `src-tauri/resources/research/public_evidence_routes.json`
- `src-tauri/resources/research/public_source_policy.json`
- `src-tauri/resources/research/public_research_prompt.txt`

这些文件只描述公开平台与外部提供器之间的通用边界。

## 直接依赖与数据流调整

- 根 `Cargo.toml` 移除两个私有模型 workspace member，新增 `crates/model-stub`。
- `crates/application/Cargo.toml` 移除真实模型 path dependency，新增 Stub dependency。
- `Cargo.lock` 同步为 11 个本地 workspace 包，并更新完整性契约。
- 推演、规则包、发布验收和模型注册改为外部提供器语义。
- 规则包不再内置模型数学常量、固定矩阵拓扑、校准样本信息或私有参数版本。
- 概率快照改为接受提供器定义的非空、唯一概率链，不再要求固定链名称或固定单元数量。
- 参数调优候选生成在公开包中明确返回不可用，避免对不存在的私有参数执行伪调优。
- 本地基础进球率准备、时间前推算法和固定校准逻辑已从公开持久化层移除。
- 赛后结算、参数生命周期和发布验收保留不可变账本及人工审核路径，但只记录外部提供器状态。

## 数据库兼容性

历史迁移编号和文件名保持不变，避免破坏已有数据库的迁移顺序。涉及模型运行时、快照、研究与工作台的迁移正文已改为通用外部提供器契约，不再种入真实模型参数或固定概率拓扑。

`contracts/release-readiness-contract.json` 已重新冻结当前 46 条迁移的文件清单与 SHA-256，用于检测后续漂移。

## 前端行为

- 保留赛事推演、P4 研究工作台、历史、规则包和临时演练入口。
- 页面明确显示“外部模型未捆绑”。
- 数据库可用只表示数据链可用，不再错误显示模型已可正式运行。
- 规则包区域改为“外部提供器规则入口”。
- 连通检查使用通用提供器入口，不再显示私有回归资产或内部校准信息。
- 删除公开页面中仅用于过滤已移除私有回归包的陈旧条件。

## 自动验证与启动入口

新增：

- `scripts/verify-public-model-boundary.mjs`
- `scripts/verify-frontend.mjs`
- `.github/workflows/ci.yml`
- `启动平台.bat`
- `验证平台.bat`
- `验收平台.bat`

GitHub Actions 分为两个任务：

1. Ubuntu + Node 22：`npm ci` 后执行完整前端、边界、TypeScript 和 Vite 验证；
2. Rust 1.88：执行公开边界、Cargo.lock、`cargo metadata --locked`、格式、Clippy 和工作区测试。

真实 PostgreSQL 集成测试继续标记为显式 ignored，必须连接名称包含 `test` 的专用数据库后人工运行，CI 不会触碰生产数据库。

## 实际执行的验证

已执行并通过：

- `node scripts/verify-public-model-boundary.mjs`
- 统一前端验证编排中的全部 Node 静态契约检查
- `node_modules/.bin/tsc --noEmit`
- `node scripts/verify-cargo-lock.mjs`
- `node scripts/verify-release-acceptance.mjs`
- `node scripts/verify-release-readiness.mjs`
- 17 个既有 UI 截图基线检查
- 所有 JSON 文件解析检查
- 所有 Rust `include_str!` 目标存在性检查
- Cargo 本地 path dependency 存在性检查
- `git diff --check`
- 发布包私有路径、依赖缓存、构建产物和 Git 历史扫描

前端统一验证已运行到 Vite 生产构建阶段。Vite 构建未在当前容器完成：`fdd.zip` 携带的是 Windows `node_modules`，其中只有 Windows Rollup 原生可选包；当前 Linux 容器缺少对应原生包，且容器 DNS 无法访问 npm registry 重新安装。TypeScript 和全部构建前门禁已经通过。最终公开包不包含该 `node_modules`，GitHub Actions 会在 Ubuntu 上通过 `npm ci` 安装正确平台依赖后重新执行 Vite 构建。

## 未执行的验证与剩余风险

当前容器没有 `cargo`、`rustc`、Docker 或 PostgreSQL，因此以下验证没有在生成环境执行：

- `cargo metadata --locked`
- `cargo fmt --all -- --check`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo test --locked --workspace`
- Tauri 桌面端真实启动与打包
- PostgreSQL 46 条迁移的真实空库执行
- ignored PostgreSQL 集成测试

替代验证包括 Cargo.lock 结构和哈希检查、workspace/path dependency 静态检查、Rust 源码卫生门禁、迁移连续性和 SHA-256 冻结、Tauri 命令前后端一致性检查。Rust 编译、桌面运行和真实数据库行为仍需由新仓库 GitHub Actions及专用测试数据库验证。

## 发布包边界

最终压缩包不包含：

- `.git/`
- `node_modules/`
- `dist/`
- `target/` 或 `.cargo-target/`
- 日志、缓存、备份和临时文件
- 本记录开头列出的私有模型资产

解压后可直接作为新仓库根目录执行 `git init -b main`。
