# R03-01 Application Ports 设计实施记录

- 任务状态：`VERIFYING`
- 分支：`new-C`
- 开始基线：`7cf906b8f98ab0fdcf89f80952bc8fb9cf21801f`
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

## 6. 验证要求

实施树必须先通过：

- `cargo fmt --all -- --check`
- `node scripts/verify-application-ports.mjs`
- `npm run verify:architecture`
- `cargo check --locked -p football-application`

随后由用户 Windows 本机执行阶段最小门禁；通过前任务保持 `VERIFYING`。
