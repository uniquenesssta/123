# 阶段 0：模块化重写基线

状态：`IN_PROGRESS`

工作分支：`refactor/modular-rewrite-stage0`

基线分支：`main`

基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`

开始日期：2026-08-04

## 阶段目标

阶段 0 只负责冻结公开仓库的真实基线、建立影响边界、确认不可改变的兼容契约，并形成后续模块化重写的可验证入口。本阶段不实施业务功能重写，不调整公共接口、数据库结构、配置默认值、错误语义、权限、安全策略或用户可观察行为。

## Atomic Task

| 节点 | 内容 | 状态 |
|---|---|---|
| S0-01 | 规则、仓库、工具链与源码规模基线 | `DONE` |
| S0-02 | 公开模型边界与私有资产隔离基线 | `PENDING` |
| S0-03 | 公共接口、命令、契约与数据流基线 | `PENDING` |
| S0-04 | PostgreSQL 迁移、持久化与数据兼容基线 | `PENDING` |
| S0-05 | 验证矩阵、阶段门禁与回退基线 | `PENDING` |

阶段 1 在 S0-01 至 S0-05 全部完成并通过阶段门禁前保持阻塞。

## S0-01 完成记录

### 已执行

- 读取并应用根目录 `ALL_AI_CODE.md` 与 `AI_PROJECT_RULES.md`。
- 以 `main` 的唯一初始提交作为公开仓库基线。
- 检查根工作区、前端、Tauri、Rust crate、脚本、契约、Schema 和数据库迁移结构。
- 确认当前公开仓库由 11 个 Rust workspace member 组成。
- 确认当前阶段不包含用户未提交修改，不修改任何业务源码、配置、依赖或数据库迁移。

### 基线规模

| 项目 | 数量 |
|---|---:|
| Rust workspace member | 11 |
| Rust 源文件 | 123 |
| TypeScript/TSX 文件 | 37 |
| CSS 文件 | 9 |
| PostgreSQL 迁移 | 46 |
| Node 验证脚本 | 66 |
| JSON 契约 | 37 |
| JSON Schema | 15 |
| 包含 Rust 测试的文件 | 51 |

统计范围为基线提交中的 `src/`、`src-tauri/`、`crates/`、`scripts/`、`contracts/`、`schemas/` 与 `crates/persistence-postgres/migrations/`。

### 当前高风险聚合点

以下文件仅作为后续影响分析入口，不代表阶段 0 已决定拆分方式：

| 文件 | 行数 | 当前风险信号 |
|---|---:|---|
| `src/main.ts` | 8550 | 前端启动、状态、事件、渲染与工作流高度集中 |
| `crates/persistence-postgres/src/monthly_workbooks.rs` | 3330 | 月度工作簿持久化流程集中 |
| `src/styles/layout.css` | 3006 | 全局布局规则集中，跨页面回归风险高 |
| `src/types.ts` | 3003 | 多领域类型集中，变更影响面不清晰 |
| `crates/persistence-postgres/src/spreadsheet_exchange.rs` | 2989 | 导入导出与持久化边界集中 |
| `crates/persistence-postgres/tests/postgres_integration.rs` | 2879 | 多条数据库链路共享单一集成测试文件 |
| `crates/persistence-postgres/src/player_catalog.rs` | 2638 | 球员目录查询与写入职责集中 |
| `src/styles/app.css` | 2618 | 应用级视觉规则与局部组件规则耦合风险 |
| `crates/persistence-postgres/src/review.rs` | 2077 | 复盘持久化与聚合逻辑集中 |
| `crates/research-gateway/src/client.rs` | 1912 | 外部传输、协议、错误和生命周期集中 |
| `crates/domain/src/lib.rs` | 1854 | 多领域公开类型集中在 crate 根模块 |

文件大小只是风险信号。后续只有在调用链、状态所有权、依赖方向和测试边界确认后才允许实施拆分。

### 必须保持不变

- 公开仓库不得包含真实 P4/P7 引擎、参数、Profile、固定比赛、私有研究提示词或模型专用固定回归资产。
- `football-model-stub` 必须继续显式返回模型不可用错误，不得静默回退或生成伪预测。
- 现有 Tauri 命令、公共 DTO、数据库格式、46 条历史迁移及兼容路径不得在阶段 0 改动。
- 非模型平台能力、球队与球员数据、赛事、阵容、复盘、Excel、研究网关和历史记录行为保持现状。
- 不新增生产依赖，不执行无关格式化、重命名或目录迁移。

## 验证记录

| 验证 | 结果 |
|---|---|
| `node scripts/verify-public-model-boundary.mjs` | 通过 |
| `node scripts/verify-cargo-lock.mjs` | 通过 |
| `node scripts/verify-rust-source-hygiene.mjs` | 通过，检查 122 个 Rust 文件 |
| `node scripts/verify-release-readiness.mjs` | 通过，确认版本 0.23.0、46 条迁移和公开模型边界 |
| `node scripts/verify-release-acceptance.mjs` | 通过 |
| `node scripts/verify-cargo-lock-sync.mjs` | 未运行完成：当前执行容器不存在 `cargo`，进程返回 `spawnSync cargo ENOENT` |
| Rust 格式、Clippy、workspace tests | 未运行：当前执行容器不存在 Rust/Cargo |
| PostgreSQL 集成测试 | 未运行：当前执行环境没有专用测试数据库 |
| Vite 生产构建 | 本节点未重复运行，交由现有 GitHub Actions 与后续 S0-05 门禁确认 |

未完成验证不得被视为通过。Rust、数据库和完整前端结果将在阶段 0 门禁中以实际 CI 或可复现日志确认。

## 本节点变更范围

- 新增本阶段记录文件。
- 根 README 增加阶段 0 状态索引。
- 未修改源码、Cargo/NPM 依赖、配置、契约、Schema、迁移或用户行为。

## 下一节点

S0-02 将冻结公开模型边界：核对模型入口、Stub、规则包、公开契约、忽略规则和边界验证脚本，形成允许项、禁止项、调用入口和失败语义清单。