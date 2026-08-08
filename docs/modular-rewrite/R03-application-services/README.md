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
| R3-01 | Application Ports 设计 | VERIFYING |
| R3-02 | Database Service | BLOCKED |
| R3-03 | Competition / Rules Services | BLOCKED |
| R3-04 | Teams / Players Services | BLOCKED |
| R3-05 | Lineups Service | BLOCKED |
| R3-06 | Prediction Service | BLOCKED |
| R3-07 | Research Service | BLOCKED |
| R3-08 | Review / Postmatch / Analytics Services | BLOCKED |
| R3-09 | Exchange / AI / Release Services | BLOCKED |
| R3-10 | ApplicationService 兼容门面 | BLOCKED |

## R3-01 当前结果

- 已从真实源码扫描 PostgreSQL `232` 个公开异步方法，其中 `209` 个当前被 Application 调用；Application 对具体 PostgreSQL crate 的直接导入仍只有 `composition/port_registry.rs` 一处。
- 已建立 15 个 Port 职责域和 36 个最小能力 trait，不建立万能 Repository。
- Ports 只使用 Domain、Application、自有 DTO、`football-model-api` 与标准基础类型；禁止 SQL Row、SQLx、PgPool、PostgresStore、PersistenceError 和裸 JSON Value 穿透。
- `football-model-api` 继续作为模型执行边界，不复制一套模型协议。
- 当前 concrete PostgreSQL 导入仍由 R1 组合根暂时持有，后续 R3-02 至 R3-09 按服务迁移逐步接入适配器；R3-01 不改业务行为。
- `verify:application-ports` 已接入 `verify:architecture`，锁定职责目录、trait 集合和过渡导入唯一所有者。
- 用户 Windows 本机已通过 `cargo fmt --all -- --check`、Application Ports、Domain 类型清单、完整 `verify:architecture`、`cargo check --locked -p football-application`、workspace Clippy `-D warnings` 和完整 workspace tests。
- workspace tests 中 18 个真实 PostgreSQL 集成测试因未设置 `FOOTBALL_TEST_DATABASE_URL` 按既有显式设计保持 `ignored`；没有将其记为已执行。
- `npm run verify:frontend` 第一次在旧 `monthly_workbook.rs` 路径停止，提交 `dee553026c03193e7f4298e0e4a963693b14b893` 修复后月度工作簿专项已通过。
- 第二次 frontend 回归继续推进到 `verify-match-lineup-chain.mjs`，其仍读取已删除的 `crates/domain/src/exchange.rs`。提交 `55aab2cf49180fbd2798846926a6ce3beca4394f` 已切换到当前唯一职责文件 `crates/domain/src/exchange/lineup.rs`；仍未修改产品行为。

## R3-01 剩余门禁

当前只剩用户 Windows 本机拉取最新 `new-C` 后重新运行：

- `npm run verify:frontend`

完整 frontend 回归通过后，R3-01 才能改为 `DONE` 并将 R3-02 改为 `READY`。详细记录见 [`R03-01-application-ports-设计.md`](./R03-01-application-ports-设计.md)。
