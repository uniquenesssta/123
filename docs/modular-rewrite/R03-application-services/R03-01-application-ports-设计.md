# R03-01 Application Ports 设计实施记录

- 任务状态：`DONE`
- 分支：`new-C`
- 开始基线：`7cf906b8f98ab0fdcf89f80952bc8fb9cf21801f`
- 实施提交：`264a55baee0ff8fe0c33928fe8161a32367b6c84`
- 目标平台：Windows

## 1. 目标

在不迁移现有业务服务、不修改 PostgreSQL SQL 和公共 ApplicationService 行为的前提下，建立 R3 后续 Use Case/Service 重写所需的最小 Ports 契约。Port 必须按业务能力和事务边界拆分，不能形成覆盖全部数据库方法的万能 Repository。

## 2. 真实调用面基线

临时 Windows 扫描 workflow run `31237537087`、job `93052732745` 对 `crates/persistence-postgres/src/**/*.rs` 与 `crates/application/src/**/*.rs` 做了实际静态交叉扫描：

- PostgreSQL 公开 `pub async fn`：232 个。
- 当前被 Application 调用的方法：209 个。
- Application 直接导入 `football_persistence_postgres`：1 处，仅 `crates/application/src/composition/port_registry.rs`。
- 扫描覆盖数据库生命周期、Competition/Rules、Team/Player、Lineup、Prediction、Research、Review/Postmatch、Analytics、Exchange、AI Workspace、Release 等现有调用链。

该扫描只用于建立边界事实，不把 209 个方法复制成一个 Port。

## 3. Port 结构

`crates/application/src/ports/` 现在按职责拆分为 database、competition、rules、team、player、lineup、prediction、research、review、postmatch、analytics、exchange、ai_workspace、release 和 system；统一错误位于 `ports/error.rs`。

每个目录内的 trait 只描述相邻能力。例如 Team Catalog 与 Team Lifecycle 分离、Player Catalog/Signals/Coach/Entity Reference 分离、Lineup/Formation/Preset 分离、Analytics/Jobs/Parameter Lifecycle 分离。跨领域通用时钟和文件能力放在 system；模型执行继续复用 `football-model-api`，不重复定义模型协议。

## 4. 依赖规则

Ports 内禁止：

- `football_persistence_postgres`
- `sqlx` / SQL Row / PgPool
- `PostgresStore` / `PersistenceError`
- 万能 `*Repository` trait/type
- 用 `serde_json::Value` 代替明确契约
- glob re-export

Port 错误通过 `PortErrorKind + PortError + PortResult<T>` 表达基础设施无关失败类别；具体 PersistenceError 到 PortError/ApplicationError 的行为兼容映射留给各服务迁移任务实现。

## 5. 兼容性

R3-01 只增加端口契约、清单和静态门禁，不替换当前 `ApplicationService` 调用路径，不修改数据库 migration、SQL、Serde、Tauri 命令、前端状态、模型参数/Profile/fixture 或生产依赖。现有 concrete PostgreSQL import 仍暂时只允许组合根一处。

## 6. 已完成验证

实施环境已通过：

- `cargo fmt --all -- --check`
- `node scripts/verify-application-ports.mjs`
- `node scripts/generate-domain-type-inventory.mjs`
- `node scripts/verify-domain-type-inventory.mjs`
- `npm run verify:architecture`
- `cargo check --locked -p football-application`

其中 Application Ports 门禁结果为 15 个职责域、36 个最小 Port trait；真实调用面登记为 209/232，具体 PostgreSQL 导入仍仅位于组合根。Domain 类型清单为 365 个类型、365 个公共兼容类型、299 个 PostgreSQL 映射类型。

用户 Windows 10 本机在 `new-C`、干净工作树上已通过：

- `cargo fmt --all -- --check`
- `node scripts/verify-application-ports.mjs`
- `node scripts/verify-domain-type-inventory.mjs`
- `npm run verify:architecture`
- `cargo check --locked -p football-application`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo test --locked --workspace`
- `npm run verify:frontend`

完整 frontend 回归最终从架构、状态所有权、受保护导入、Domain 清单、Browser/Tauri/Application 组合根、公开模型边界、保护资产、球队/球员、阵容、工作区、数据库兼容、命令契约、截图回归、TypeScript 一直执行到 Vite production build，全部通过。Vite 仅报告既有的大 chunk warning，没有构建失败。

workspace tests 全部执行到结束；普通测试全部通过。`crates/persistence-postgres/tests/postgres_integration.rs` 中 18 个需要 `FOOTBALL_TEST_DATABASE_URL` 的真实 PostgreSQL 集成测试按既有显式设计保持 `ignored`，本节点未把它们伪装成已执行。

## 7. Frontend 阶段回归中的验证器兼容修复

完整 frontend 回归过程中发现两处 R2 Domain 拆分后遗留的验证器旧读取路径：

1. `scripts/verify-monthly-workbooks.mjs` 仍读取已删除的 `crates/domain/src/monthly_workbook.rs` 与 `crates/domain/src/spreadsheet.rs`。提交 `dee553026c03193e7f4298e0e4a963693b14b893` 将其切换到：
   - `crates/domain/src/exchange/monthly/contract.rs`
   - `crates/domain/src/exchange/spreadsheet/contract.rs`
2. `scripts/verify-match-lineup-chain.mjs` 仍读取已删除的 `crates/domain/src/exchange.rs`。提交 `55aab2cf49180fbd2798846926a6ce3beca4394f` 将其切换到当前唯一职责文件 `crates/domain/src/exchange/lineup.rs`。

两处修改都只修正静态验证器对模块化后真实源码位置的读取，不修改月度工作簿、比赛阵容、Domain 数据结构、数据库 SQL、Tauri 命令、前端行为、模型边界或保护资产。修复后用户本机完整 `npm run verify:frontend` 已通过。

## 8. 完成结论

R3-01 完成标准已满足：Application Ports 已建立唯一职责目录和静态门禁，公共 ApplicationService 行为保持不变，最小门禁、workspace Clippy、workspace tests 与完整 frontend 回归均通过。R3-01 正式关闭为 `DONE`，R3-02 Database Service 可进入 `READY`。

剩余风险仅为阶段级统一验收中既有的 18 个真实 PostgreSQL 集成测试尚未在本节点设置 `FOOTBALL_TEST_DATABASE_URL` 显式运行；该限制不被隐藏或视为通过。
