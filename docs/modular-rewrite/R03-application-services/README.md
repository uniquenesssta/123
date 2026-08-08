# R03 Application Services 重写：执行记录索引

## 阶段状态

`IN_PROGRESS`

R2 已完成并关闭。R3 只重写 Application 编排与 Ports/Services/Use Cases 边界，不修改具体 PostgreSQL SQL、Tauri DTO、前端状态或模型实现。

## 基线

- R3 分支：`new-C`
- R3 起点：`7cf906b8f98ab0fdcf89f80952bc8fb9cf21801f`
- R2 完成记录：[`../R02-domain-rewrite/R02-stage-completion.md`](../R02-domain-rewrite/R02-stage-completion.md)
- 目标平台：Windows

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R3-01 | Application Ports 设计 | DONE |
| R3-02 | Database Service | DONE |
| R3-03 | Competition / Rules Services | VERIFYING |
| R3-04 | Teams / Players Services | BLOCKED |
| R3-05 | Lineups Service | BLOCKED |
| R3-06 | Prediction Service | BLOCKED |
| R3-07 | Research Service | BLOCKED |
| R3-08 | Review / Postmatch / Analytics Services | BLOCKED |
| R3-09 | Exchange / AI / Release Services | BLOCKED |
| R3-10 | ApplicationService 兼容门面 | BLOCKED |

## R3-01 完成结果

- 已从真实源码扫描 PostgreSQL `232` 个公开异步方法，其中 `209` 个当前被 Application 调用；Application 对具体 PostgreSQL crate 的直接导入仍只有 `composition/port_registry.rs` 一处。
- 已建立 15 个 Port 职责域和 36 个最小能力 trait，不建立万能 Repository；Ports 禁止 SQLx、SQL Row、PgPool、PostgresStore、PersistenceError、裸 JSON Value 和 glob re-export。
- `football-model-api` 继续作为模型执行边界，不复制模型协议；R3-01 不切换现有业务流程，不修改公共 ApplicationService 行为。
- Windows 本机已通过 rustfmt、Application Ports、Domain 清单、完整 `verify:architecture`、`cargo check -p football-application`、workspace Clippy `-D warnings`、workspace tests 和完整 `npm run verify:frontend`。
- frontend 回归中发现并修复两处 R2 Domain 拆分后遗留的验证器旧路径：月度工作簿验证器与比赛阵容链验证器均已改读当前唯一职责文件；产品代码、数据库 SQL、Tauri、前端行为和模型保护资产未改变。
- 完整 frontend 最终通过 17 个截图回归视口、TypeScript 与 Vite production build；Vite 仅保留既有大 chunk warning。
- workspace tests 中 18 个真实 PostgreSQL 集成测试因未设置 `FOOTBALL_TEST_DATABASE_URL` 按既有显式设计保持 `ignored`，未记为已执行。

R3-01 已正式关闭为 `DONE`，详细记录见 [`R03-01-application-ports-设计.md`](./R03-01-application-ports-设计.md)。

## R3-02 完成结果

- 已删除旧 `crates/application/src/database.rs`，连接、迁移、恢复、health、statistics、reset 分别进入 `services/database/` 与 `use_cases/database/`，活动数据库状态由 `DatabaseService.session` 唯一持有。
- Tauri 数据库清空命令只委托 Application Database Service；具体 `PostgresStore` 生命周期与观测适配仍只位于 `composition/port_registry.rs`，Service / Use Case 不直接写 SQL 或依赖 SQLx。
- 实施期修复了 `active_store` 嵌套模块可见性、恢复数量返回值归一以及 reset 验证器对 rustfmt 链式调用的误判；没有增加 lint 抑制、跳过检查或放宽数据库强确认。
- 实施侧 Windows run `31244006019` / job `93069490517` 已通过 Database Service 专项、reset 契约、Application Ports、Domain 清单、确定性保护资产、完整 `verify:architecture`、`cargo fmt --check`、Application check/tests。
- 用户 Windows 本机已通过最小验证、完整 `npm run verify:frontend`、完整 `npm run verify:rust`；Application 单测 30/30 通过，workspace tests 无失败，18 个真实 PostgreSQL 集成测试因未设置专用 `FOOTBALL_TEST_DATABASE_URL` 继续安全保持 `ignored`。
- `npm run tauri:dev` 运行时烟测成功；上传 runtime JSONL 共 48 条且全部为 `info`，`bootstrap` 的 `connection_error=null`，原数据库上的教练、阵型、球队、阵容、Analytics 与 Postmatch 读取链均完成，没有迁移、连接、panic、error 或 critical。
- 本次 runtime log 实际写入 `F:\FOODBALL\logs`；从源码目录执行 `Get-ChildItem .\logs` 找不到目录是当前 runtime root discovery 的既有路径行为，不属于 Database Service 回归，本节点未修改运行日志目录策略。

R3-02 已正式关闭为 `DONE`。详细记录见 [`R03-02-database-service.md`](./R03-02-database-service.md)。

## R3-03 当前结果

- 已删除旧 `crates/application/src/competition.rs` 与 `crates/application/src/rule_packages.rs`，赛事层级和规则包/赛事绑定分别重写到 `services/competition/`、`services/rules/` 与对应 `use_cases/`；共 21 个 Service / Use Case Rust 文件。
- `ApplicationService` 聚合 `CompetitionService` 与 `RulesService`，既有同名公开方法只保留活动数据库会话获取和兼容委托；Tauri 7 个赛事/规则公共命令、参数与返回 DTO 保持不变。
- `ActiveDatabase` 在组合根实现 `CompetitionHierarchyPort`、`RulePackagePort`、`RuleRoutingPort`；具体 PostgreSQL 仍只位于 `composition/port_registry.rs`，未修改 SQL、迁移、Schema、依赖或模型保护资产。
- 数据库连接后的内置规则包注册已通过 RulesService；bootstrap 的赛事层级、规则包与绑定读取已通过 Competition/Rules Services。Prediction 路由预览与模型调用仍留给 R3-06，没有提前迁移。
- 新增 `verify:competition-rules-service` 并接入 architecture/frontend；R3-02 Database Service 验证器同步跟随新的 RulesService 初始化边界，没有降低生命周期、reset 或 PostgreSQL 隔离门禁。
- Windows 2025 严格实施验证 run `31248365735` / job `93080599447` 已通过 rustfmt、365 类型清单、R3-03 专项、完整 architecture、确定性模型保护指纹、Application check/tests 与 `git diff --check`；实施保护分支已同步到提交 `fc93103fb4327bbafea7b800984a971f5bf1f328`，临时 workflow 已删除。
- 用户 Windows 本机最小验证、完整 frontend、完整 Rust 与 `tauri:dev` 非破坏性赛事/规则运行时烟测尚未完成，因此 R3-03 保持 `VERIFYING`，R3-04 继续 `BLOCKED`。

详细记录见 [`R03-03-competition-rules-services.md`](./R03-03-competition-rules-services.md)。