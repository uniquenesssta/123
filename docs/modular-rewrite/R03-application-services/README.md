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
| R3-02 | Database Service | VERIFYING |
| R3-03 | Competition / Rules Services | BLOCKED |
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

## R3-02 当前结果

- 已删除旧 `crates/application/src/database.rs`，连接、迁移、恢复、health、statistics、reset 分别进入 `services/database/` 与 `use_cases/database/`，活动数据库状态由 `DatabaseService.session` 唯一持有。
- Tauri 数据库清空命令已改为委托 Application Database Service；具体 `PostgresStore` 生命周期与观测适配仍只位于 `composition/port_registry.rs`，Service / Use Case 不直接写 SQL 或依赖 SQLx。
- 第一轮编译暴露并修复两个真实兼容问题：嵌套模块导致 `active_store` crate 内可见范围缩小，以及 interrupted API workspace recovery 的 `u64` 返回值未归一为 `()`；同时删除两个无调用的 crate-private 根别名，没有增加 lint 抑制或降低门禁。
- 实施侧 Windows run `31244006019` / job `93069490517` 已通过 Database Service 专项、reset 契约、Application Ports、Domain 清单、确定性保护资产、完整 `verify:architecture`、`cargo fmt --check`、`cargo check -p football-application` 和 `cargo test -p football-application`；成功生成实现支持提交 `cd754f79456b96b3e66ac45b119f61609346e06d`，临时 workflow 已删除。
- Windows 本机最小验证、完整 frontend、workspace Clippy/tests 与 `tauri:dev` 数据库运行时烟测尚未完成，因此 R3-02 保持 `VERIFYING`，R3-03 不开放。

详细记录见 [`R03-02-database-service.md`](./R03-02-database-service.md)。