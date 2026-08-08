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
| R3-03 | Competition / Rules Services | DONE |
| R3-04 | Teams / Players Services | VERIFYING |
| R3-05 | Lineups Service | VERIFYING |
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

## R3-03 完成结果

- 已删除旧 `crates/application/src/competition.rs` 与 `crates/application/src/rule_packages.rs`，赛事层级和规则包/赛事绑定分别重写到 `services/competition/`、`services/rules/` 与对应 `use_cases/`；共 21 个 Service / Use Case Rust 文件。
- `ApplicationService` 聚合 `CompetitionService` 与 `RulesService`，既有同名公开方法只保留活动数据库会话获取和兼容委托；Tauri 7 个赛事/规则公共命令、参数与返回 DTO 保持不变。
- `ActiveDatabase` 在组合根实现 `CompetitionHierarchyPort`、`RulePackagePort`、`RuleRoutingPort`；具体 PostgreSQL 仍只位于 `composition/port_registry.rs`，未修改 SQL、迁移、Schema、依赖或模型实现与私有资产范围。
- 数据库连接后的内置规则包注册已通过 RulesService；bootstrap 的赛事层级、规则包与绑定读取已通过 Competition/Rules Services。Prediction 路由预览与模型调用仍留给 R3-06，没有提前迁移。
- 用户 Windows 本机已完成最小验证、完整 `verify:frontend`、完整 `verify:rust` 与 `tauri:dev`。Application 31/31 通过，workspace Clippy/tests 无失败；18 个真实 PostgreSQL 集成测试因未配置专用测试库继续 `ignored`。
- 本次 runtime JSONL 共 98 条，`bootstrap` 450 ms 完成且 `connection_error=null`；98 条中 97 条为 `info`，唯一 `error` 是阵容页未选择球员时点击添加触发的既有输入校验，与 Competition/Rules、数据库、panic 或基础设施无关。当前 bootstrap 只有在 CompetitionService hierarchy 与 RulesService catalog 均读取成功后才返回，因此已覆盖原数据库赛事层级、规则包与赛事绑定读取。
- 完整 frontend 中模型边界与 deterministic 保护资产门禁通过；用户单独运行旧版 `verify_protected_assets.mjs` 暴露历史 locale 排序差异。现已把基础验证器改为 ordinal path sort，并同步刷新其受保护指纹；workflow run `31249193592` 已验证直接入口与 deterministic wrapper 同时通过，临时 workflow 已删除。
- 保护资产直接入口与 deterministic wrapper 已在确定性修复 workflow run `31249193592` 的同一代码树通过；用户明确授权关闭节点。R3-03 已为 `DONE`，R3-04 Teams / Players Services 已开放为 `READY`。

详细记录见 [`R03-03-competition-rules-services.md`](./R03-03-competition-rules-services.md)。

## R3-04 当前结果

- 已将旧 `player_catalog.rs` 中 35 个球队、球员、教练与实体引用 Application 职责迁入 `services/teams/`、`services/players/` 与对应 `use_cases/`；共 43 个 Service / Use Case Rust 文件。R3-05 已接手并删除原文件剩余的阵型、比赛、阵容与预设职责。
- `ApplicationService` / `ApplicationComposition` 已聚合 `TeamService` 与 `PlayerService`；公共方法、Tauri 命令、DTO、SQL、迁移、生产依赖和模型边界保持兼容。
- 6 个 Team / Player Ports 的具体适配按职责拆到 `composition/adapters/teams.rs` 与 `players.rs`；`port_registry.rs` 继续作为 Application 唯一直接导入 PostgreSQL crate 的组合根所有者。
- 首轮 Windows 编译真实暴露球队强制删除 SQLx transaction 的 non-Send 边界；现仅在组合适配器对 preview/force-delete 使用 `spawn_blocking + Handle::block_on`，保持既有 Tauri 隔离和事务语义，没有修改 SQL 或弱化强确认。
- Windows 2025 run `31258038424` / job `93104371481` 已通过 R3-04 专项、实体关系、球队强制清除、球队/球员管理、完整 architecture、保护资产、Application check、Application tests 33/33、workspace Clippy `-D warnings` 与 diff hygiene。
- 用户随后在 `new-C` 提供 clean 工作区、rustfmt、R3-04 专项、完整 architecture、Application check 与 33/33 Application tests 的本机通过结果，并明确授权进入 R3-05；未提供完整 frontend / Rust 与非破坏性 runtime 烟测，因此 R3-04 状态仍保持 `VERIFYING`，不虚报为 `DONE`。

详细记录见 [`R03-04-teams-players-services.md`](./R03-04-teams-players-services.md)。

## R3-05 当前结果

- 已删除旧 `crates/application/src/player_catalog.rs`，其剩余 19 个阵型、比赛、阵容、阵容预设公开 Application 职责全部迁入 `services/lineups/` 与对应 `use_cases/lineups/`；共 23 个 Lineups Service / Use Case Rust 文件。
- `ApplicationService` / `ApplicationComposition` 已聚合唯一 `LineupService`；19 个既有公开方法名、参数、返回类型、Tauri 调用链与错误语义保持兼容。
- 沿用 `FormationPort`、`MatchCatalogPort`、`LineupPort`、`LineupPresetPort` 4 个既有 Ports，并由 `composition/adapters/lineups.rs` 负责具体持久化适配；Service / Use Case 不泄漏 PostgreSQL、SQLx、PgPool、PostgresStore 或 PersistenceStore。
- 完整 Rust 编译暴露 `MatchCatalogPort::read_match` 需要调用 persistence crate 既有 `read_match_exchange`，现仅将该方法从 `pub(crate)` 提升为 `pub async fn` 以形成合法 workspace 边界；方法体、SQL、参数、返回结构和数据库行为未改。
- 删除旧 Application owner 后确认受影响的 5 个历史验证器已改读当前权威 Teams / Players / Lineups owner，业务断言未删除或放宽；Domain inventory 已按最终源码重算，架构扫描覆盖 400 个 Rust 文件。
- clean 实施提交 `7e3fddeafcd32cc45e293fa9a7aeb05c7c66d4ec` 的 Public Platform CI run `31260698438` / job `93110942400` 已通过 architecture 与完整 Windows Automated：frontend、17 个截图回归视口、TypeScript、Vite、完整 Rust/Clippy/workspace tests、Tauri release 构建及 release 启动日志扫描均通过。artifact `9022970030`，大小 `14242839` 字节，SHA-256 `275e17a78db9d5205d49401a1a1d20ed91f08102594d2d04c339051165beb052`。
- R3-05 当前为 `VERIFYING`，等待用户 Windows 本机最小复核与非破坏性 runtime 烟测；R3-06 Prediction Service 继续 `BLOCKED`。

详细记录见 [`R03-05-lineups-service.md`](./R03-05-lineups-service.md)。
