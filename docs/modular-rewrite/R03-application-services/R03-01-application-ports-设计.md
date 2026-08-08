# R03-01 Application Ports 设计实施记录

- 任务状态：`VERIFYING`
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

用户 Windows 10 本机已在 `new-C`、干净工作树上再次通过：

- `cargo fmt --all -- --check`
- `node scripts/verify-application-ports.mjs`
- `node scripts/verify-domain-type-inventory.mjs`
- `npm run verify:architecture`
- `cargo check --locked -p football-application`
- `cargo clippy --locked --workspace --all-targets -- -D warnings`
- `cargo test --locked --workspace`

本机 `cargo check` 与完整 workspace Clippy 均成功完成。workspace tests 全部已执行到结束；普通测试全部通过，`crates/persistence-postgres/tests/postgres_integration.rs` 中 18 个需要 `FOOTBALL_TEST_DATABASE_URL` 的真实 PostgreSQL 集成测试按既有显式设计保持 `ignored`，本节点未把它们伪装成已执行。

## 7. Frontend 阶段回归与验证器兼容修复

本机 `npm run verify:frontend` 已两次实际进入完整验证链。两次失败都发生在 R2 Domain 拆分后遗留的验证器旧读取路径，不是产品行为、Port 编译、公共契约或模型保护边界失败。

第一次失败：

```text
verify-monthly-workbooks.mjs
ENOENT: crates/domain/src/monthly_workbook.rs
```

`scripts/verify-monthly-workbooks.mjs` 原来仍读取已删除的 `crates/domain/src/monthly_workbook.rs` 与 `crates/domain/src/spreadsheet.rs`。提交 `dee553026c03193e7f4298e0e4a963693b14b893` 已将其切到当前唯一职责来源：

- `crates/domain/src/exchange/monthly/contract.rs`
- `crates/domain/src/exchange/spreadsheet/contract.rs`

用户拉取后第二次执行 `npm run verify:frontend`，月度工作簿专项已明确通过；随后新的首个失败点为：

```text
verify-match-lineup-chain.mjs
ENOENT: crates/domain/src/exchange.rs
```

`scripts/verify-match-lineup-chain.mjs` 仍读取已被 R2-07 删除的旧 `crates/domain/src/exchange.rs`。比赛阵容导入格式常量当前唯一来源为 `crates/domain/src/exchange/lineup.rs`，因此提交 `55aab2cf49180fbd2798846926a6ce3beca4394f` 仅将验证器读取路径切换到该职责文件。

以上修复均只修正静态验证器对模块化后真实源码位置的读取，不修改月度工作簿、比赛阵容、Domain 数据结构、数据库 SQL、Tauri 命令、前端行为、模型边界或保护资产。

## 8. 剩余关闭条件

当前只剩用户 Windows 本机拉取最新 `new-C` 后重新执行：

- `npm run verify:frontend`

该命令完整通过前，R3-01 保持 `VERIFYING`，R3-02 不提前开放。完整通过后再将本记录和阶段索引收口为 `DONE / READY`。
