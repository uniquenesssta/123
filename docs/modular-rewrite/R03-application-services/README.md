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
| R3-04 | Teams / Players Services | DONE |
| R3-05 | Lineups Service | DONE |
| R3-06 | Prediction Service | DONE |
| R3-07 | Research Service | DONE |
| R3-08 | Review / Postmatch / Analytics Services | DONE |
| R3-09 | Exchange / AI / Release Services | DONE |
| R3-10 | ApplicationService 兼容门面 | READY |

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

## R3-04 完成结果

- 已将旧 `player_catalog.rs` 中 35 个球队、球员、教练与实体引用 Application 职责迁入 `services/teams/`、`services/players/` 与对应 `use_cases/`；共 43 个 Service / Use Case Rust 文件。R3-05 已接手并删除原文件剩余的阵型、比赛、阵容与预设职责。
- `ApplicationService` / `ApplicationComposition` 已聚合 `TeamService` 与 `PlayerService`；公共方法、Tauri 命令、DTO、SQL、迁移、生产依赖和模型边界保持兼容。
- 6 个 Team / Player Ports 的具体适配按职责拆到 `composition/adapters/teams.rs` 与 `players.rs`；`port_registry.rs` 继续作为 Application 唯一直接导入 PostgreSQL crate 的组合根所有者。
- 首轮 Windows 编译真实暴露球队强制删除 SQLx transaction 的 non-Send 边界；现仅在组合适配器对 preview/force-delete 使用 `spawn_blocking + Handle::block_on`，保持既有 Tauri 隔离和事务语义，没有修改 SQL 或弱化强确认。
- Windows 2025 run `31258038424` / job `93104371481` 已通过 R3-04 专项、实体关系、球队强制清除、球队/球员管理、完整 architecture、保护资产、Application check、Application tests 33/33、workspace Clippy `-D warnings` 与 diff hygiene。
- 用户在 R3-04 当时已提供 clean 工作区、rustfmt、R3-04 专项、完整 architecture、Application check 与 33/33 Application tests 的本机通过结果；后续 R3-05 在继续保留 Teams / Players 权威 owner 的累计代码树上完成完整 `npm run verify:frontend`、完整 `npm run verify:rust`、workspace Clippy/tests 与 `tauri:dev`，补齐 R3-04 原先缺失的完整 frontend / Rust 回归。
- 2026-08-09 用户在当前累计代码树完成最终非破坏性 Windows `tauri:dev` 烟测：runtime JSONL 共 209 条且 209 条均为 `info`，无 `error`、`critical` 或 `operation_failed`；`update_team` 与 `update_player` 原值保存均完成，`list_coaches` 返回 10 条且 `read_coach` 成功，`list_entity_references` 对 team / player / coach 三类均成功返回 10 条。未执行强制删除、批量删除或数据库 reset。
- R3-04 原始验收缺口均已有真实证据，状态正式关闭为 `DONE`；在 R3-07 已完成的前提下，R3-08 Review / Postmatch / Analytics Services 开放为 `READY`。

详细记录见 [`R03-04-teams-players-services.md`](./R03-04-teams-players-services.md)。

## R3-05 完成结果

- 已删除旧 `crates/application/src/player_catalog.rs`，其剩余 19 个阵型、比赛、阵容、阵容预设公开 Application 职责全部迁入 `services/lineups/` 与对应 `use_cases/lineups/`；共 23 个 Lineups Service / Use Case Rust 文件。
- `ApplicationService` / `ApplicationComposition` 已聚合唯一 `LineupService`；19 个既有公开方法名、参数、返回类型、Tauri 调用链与错误语义保持兼容。
- 沿用 `FormationPort`、`MatchCatalogPort`、`LineupPort`、`LineupPresetPort` 4 个既有 Ports，并由 `composition/adapters/lineups.rs` 负责具体持久化适配；Service / Use Case 不泄漏 PostgreSQL、SQLx、PgPool、PostgresStore 或 PersistenceStore。
- 完整 Rust 编译暴露 `MatchCatalogPort::read_match` 需要调用 persistence crate 既有 `read_match_exchange`，现仅将该方法从 `pub(crate)` 提升为 `pub async fn` 以形成合法 workspace 边界；方法体、SQL、参数、返回结构和数据库行为未改。
- 删除旧 Application owner 后确认受影响的 5 个历史验证器已改读当前权威 Teams / Players / Lineups owner，业务断言未删除或放宽；Domain inventory 已按最终源码重算，架构扫描覆盖 400 个 Rust 文件。
- clean 实施提交 `7e3fddeafcd32cc45e293fa9a7aeb05c7c66d4ec` 的 Public Platform CI run `31260698438` / job `93110942400` 已通过 architecture 与完整 Windows Automated：frontend、17 个截图回归视口、TypeScript、Vite、完整 Rust/Clippy/workspace tests、Tauri release 构建及 release 启动日志扫描均通过。artifact `9022970030`，大小 `14242839` 字节，SHA-256 `275e17a78db9d5205d49401a1a1d20ed91f08102594d2d04c339051165beb052`。
- 用户 Windows 本机在最终分支 HEAD 上通过 clean 工作区、rustfmt、R3-05 专项、完整 architecture、Application check/tests 33/33、完整 `verify:frontend`、完整 `verify:rust` 与 `tauri:dev`。workspace tests 无失败；18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试按安全设计保持 `ignored`，未记为已执行。
- 本机 runtime JSONL 共 280 条：274 条 `info`、6 条 `error`。其中 2 条为阵容预设名称为空、1 条为未选择球员的预期输入校验；另外 3 条为公开源码未分发 P4 模型运行时的预期失败。`bootstrap.connection_error=null`；两次预设保存成功，多次预设应用预检均 `can_apply=true`，`create_lineup_pair` 成功后 `list_lineups` 从 0 条变为 2 条，`read_match_lineup_chain` 从双方阵容缺失转为 `blocking_issues=[]`、`ready_for_model=true`。未发现 Lineups 持久化、SQL、migration、panic、连接或兼容性错误。
- R3-05 已正式关闭为 `DONE`；R3-06 Prediction Service 已进入 `IN_PROGRESS`。

详细记录见 [`R03-05-lineups-service.md`](./R03-05-lineups-service.md)。

## R3-06 当前结果

- Atomic Task 1 已将 Prediction Core 的推演执行、比赛 readiness、stored-match formal/shadow execution、route preview、dry-run 与运行历史职责迁入 `services/prediction/`、`use_cases/prediction/`；旧 `crates/application/src/prediction.rs` 已删除。ApplicationService / Tauri 既有公开方法和返回语义保持兼容，模型调用继续只经过 `football-model-api`。
- Atomic Task 2A 已迁移 P4 horizon planning、freeze task list/read/events、freeze readiness、match/task workspace 只读职责；`resolve_p4_conflict`、联网 Research、Evidence/Fact 写入和 Research artifact 写入仍属于 R3-07，未提前混入 Prediction Service。
- 2A Windows hard gate run `31266144950` / job `93124468057` 已通过 Application Ports、完整 architecture、rustfmt、Application check 与 Application tests 33/33。
- 删除旧 Prediction 单文件后确认 3 个历史 frontend 验证器仍引用旧 owner，现已分别改读 readiness owner 或递归扫描当前完整 Prediction Service / Use Case 模块树；没有删除、跳过或放宽原有业务断言。
- 迁移后编译器确认的 unused imports 已直接清理；run `31266871976` / job `93126329974` 已通过 `cargo clippy --locked -p football-application --all-targets -- -D warnings`、Application Ports、architecture、rustfmt 与 Application tests。测试专用 `P4Horizon` / `is_p4_model` 已收敛至 `#[cfg(test)]`。
- Atomic Task 2A 已正式关闭为 `DONE`。最终验收提交 `443286b269cc6f34318bcf9ea60a86697f7a64a8` 的 Public Platform CI run `31268125289` / Windows Automated job `93129475772` 已通过 architecture、完整 frontend、17 个截图回归视口、TypeScript、Vite、Rust fmt、workspace Clippy `-D warnings`、workspace tests、Tauri Windows release 构建与 release runtime 日志验收；Application tests 33/33 通过。artifact `9025087726` 大小 `14245255` 字节，SHA-256 `f69a988b6832c5af18af661ea3e436ffeb48212d9a7f67c049676356376180ae`。18 个真实 PostgreSQL 集成测试因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 继续按既有安全设计保持 `ignored`，未记为已执行。所有 2A 临时 workflow / Python 脚本均已删除；R3-06 仍为 `IN_PROGRESS`，R3-07 仍为 `BLOCKED`，可继续 R3-06 下一 Atomic Task。
- Atomic Task 2B 已正式关闭为 `DONE`。实施提交 `0d691114e67116fb9f03e4cd0fb04c6a819d4254` 将 P4 freeze execution 迁入 `services/prediction` / `use_cases/prediction/execute_p4_freeze/`，通过独立 `P4FreezeExecutionPort` 访问路由事实、已有冻结快照与不可变快照写入；旧 `p4_orchestration.rs` 的 Research worker 与 OpenAI Research / Evidence / Fact / conflict mutation 继续留给 R3-07。专项 run `31289363055` / job `93183820380` 已通过 freeze verifier、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests。最终 clean-tree Public Platform CI run `31289854065` 已通过 architecture job `93185076242` 和 Windows Automated job `93185076247`，覆盖完整 frontend、17 个截图回归视口、TypeScript、Vite、Rust fmt、workspace Clippy/tests、Tauri Windows release 与 release runtime smoke；Application tests 33/33，runtime 为 7 条日志 / 3 个完成操作。artifact `9031315604` 大小 `14249898` 字节，SHA-256 `438a352f8d81cee9b044d9c1d9a36682f1df435fa513eb7768bbd875322e89bb`。18 个 PostgreSQL 集成测试因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 保持 `ignored`。2B 临时 workflow / Python 脚本均已清理；R3-06 保持 `IN_PROGRESS`，R3-07 保持 `BLOCKED`，可继续 R3-06 下一 Atomic Task。
- Atomic Task 2C 已正式关闭为 `DONE`。Windows hard gate run `31292152981` 已通过 Prediction Service、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests；公开 P4 Snapshot 写入/读取 API 已迁入 Prediction Service / `use_cases/prediction/p4_snapshot/`，公共 ApplicationService 契约保持兼容。clean-tree HEAD `5e60f0b16c907d38f1a827a8705dde59c626b045` 的 Public Platform CI run `31292509918` / Windows Automated job `93192012861` 已通过完整 frontend、17 个截图回归视口、TypeScript、Vite、Rust fmt、workspace Clippy/tests、Tauri Windows release 与 release runtime smoke；Application tests 33/33，runtime 为 7 条日志 / 3 个完成操作。artifact `9032172319` 大小 `14249704` 字节，SHA-256 `8a14b02d4687a75839497ed07b443e92be5960f49c1b98ad9ea5db6d0bae8450`。18 个 PostgreSQL 集成测试因未配置专用可写 `FOOTBALL_TEST_DATABASE_URL` 保持 `ignored`。终审确认旧 `p4_orchestration.rs` 剩余为 R3-07 Research worker，freeze 路径已委托 Prediction Service；`p4_workbench.rs` / `p4_persistence.rs` 剩余写职责均属于 Research。2C 临时 workflow / 脚本已清理；R3-06 已关闭为 `DONE`，R3-07 已开放为 `READY`。


## R3-07 当前结果

- Atomic Task 1 已建立 ResearchService，并按 Artifact Catalog / Research Ledger 两个职责模块迁移原 `p4_persistence.rs` 的 7 个公开写入口；数据库初始化的内置 schema 注册也改经 ResearchService。旧 `p4_persistence.rs` 删除。
- Research Ports 新增 `ResearchEvidenceLedgerPort`，`ResearchArtifactPort` 补齐赛事配置版本与 run-event 返回记录能力；PostgreSQL 适配保持在 composition 层。
- `verify:research-service` 已接入 architecture / frontend。Windows hard gate run `31295528438` 负责 Research 专项、Ports、architecture、Application check/tests、workspace Clippy/tests。OpenAI Research、Fact Pipeline、Research worker 与人工 conflict mutation 尚未迁移，R3-07 保持 `IN_PROGRESS`。
- 正式 Public CI run `31295931710` 暴露 Database Service 验证器仍追踪旧 `register_p4_persistence_artifacts` owner；已将断言迁移到 ResearchService / Artifact Catalog / ResearchArtifactPort 权威链，保持原初始化语义门禁强度。修复门禁 run `31296085324` 通过后提交。
- 第二次正式 Public CI run `31296120912` 在 Database verifier 通过后暴露 Prediction verifier 仍直接读取已删除 `p4_persistence.rs`；已改为验证旧 owner 缺失并检查 Prediction `p4_snapshot` use case 继续承担公开快照职责，未放宽 Prediction/Research 边界。修复门禁 run `31296198981` 通过后提交。
- 第三次正式 Public CI run `31296232974` 发现 `p4_snapshot` 内部职责名为 `freeze/read`，并非公共 facade 方法名；验证器已改为检查真实内部入口及 Port 委托，公共 Application API 仍由 facade/service 断言。修复门禁 run `31296338770` 通过后提交。
- Atomic Task 1 已正式关闭为 `DONE`。正式 HEAD `ad53bd6c8cdbd93c9e58a38642ae4d22c7d32df7` 的 Public Platform CI run `31296372108` / Windows Automated job `93201993026` 已通过 architecture、完整 Windows automated acceptance 与 evidence upload；AT1 hard gate run `31295528438` 已通过 Research 专项、Ports、architecture、Application check/tests、workspace Clippy/tests。artifact `9033265259` 大小 `14252405` 字节，SHA-256 `730bc98ba24dca2955c7100975ed79fd10a66919fd74a713ec061bddd3b44b59`。临时施工与 verifier 修复 workflow / 脚本均已清理；R3-07 保持 `IN_PROGRESS`，下一 Atomic Task 尚未开始。
- Atomic Task 2 迁移 Fact Pipeline：删除旧 `fact_pipeline.rs`，公共 evidence processing 进入 ResearchService；Fact Pipeline 按职责拆为协调、实体、时间、来源、证据、冲突、路由、验证和共享类型模块，持久化只经 FactPipeline / Evidence Ledger / Artifact Ports。OpenAI Gateway、Research worker 与 conflict mutation 未提前迁移。
- Atomic Task 2 已正式关闭为 `DONE`。Windows hard gate run `31298524184` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean 提交 `1fb4c7b05573ef75eb48903eea25bc8b2072c9de`。正式 Public Platform CI run `31298887231` / Windows Automated job `93208343460` 全部 SUCCESS，validation evidence upload 成功；artifact `9034165649` 大小 `14273283` 字节，SHA-256 `761fa51e8f811d9fd85bf09b5ff798b9862613d7206185993dfff227cdaf160b`。AT2 临时施工文件已清理；R3-07 保持 `IN_PROGRESS`，下一 Atomic Task 为 OpenAI Research Gateway execution。
- Atomic Task 3 已正式关闭为 `DONE`。OpenAI Research Gateway execution 已删除旧 `openai_research.rs` 并按 artifacts / gateway / attempt audit / references / execution / validation 职责拆入 Research use cases；公共命令与错误语义保持不变，`ResearchGatewayAuditPort` 仅补齐 attempt-number offset 并由 `ActiveDatabase` 复用既有 PostgreSQL gateway records。迁移后无调用者的 Database transition-store 与 ResearchService Fact Pipeline 转发桥接已移除。Windows hard gate run `31312515543` / job `93242388772` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean 提交 `d818276bc93c1dfb8fd7c1c5fbab71a97e6cd5e2`；相同 clean tree 的验收提交 `ad12e8d8d574ee3df8c3c1e83cf7705e718d0c49` 的 Public Platform CI run `31313043256` / Windows Automated job `93243752378` 全部 SUCCESS，validation evidence upload 成功。artifact `9038223561` 大小 `14263377` 字节，SHA-256 `611f79b5b5789fce242d1429c1060415187f7f07f5253d03eb478bb7deed5bd3`。AT3 临时施工与 CI trigger 文件均已清理；R3-07 保持 `IN_PROGRESS`，下一 Atomic Task 为 P4 Research worker，manual conflict mutation 留待后续。
- Atomic Task 4 已迁移 P4 Research worker：Research 状态机、动态上下文、OpenAI Gateway 调用、run 恢复与 freeze handoff 拆入 `use_cases/research/p4_worker/`；ResearchService 只经既有 Ports 协作。根 `p4_orchestration.rs` 收敛为跨服务 dispatcher/worker loop，Prediction freeze 仍由 PredictionService 执行；`p4_workbench.rs` 人工冲突裁决保持后续边界，只复用 ResearchService 的成功收口。AT4 已正式关闭为 `DONE`：clean implementation `89a3c68ad50f7766d6db5d214c0fa5a39c1a6c72` 的同源码树验收提交 `75e76dea9a9ab7980f64374b4f27410ee221f8f9` 已通过 Public Platform CI run `31316144230` / Windows Automated job `93251603683`，validation evidence upload 成功；artifact `9039129007` 大小 `14264776` 字节，SHA-256 `fe3ef81501cb2c7d57302f8e03e9b0753f82138d8dc54356e39fc5d0ab31f68f`。Atomic Task 5 已正式关闭为 `DONE`：人工冲突裁决已迁入 ResearchService / `use_cases/research/p4_manual_conflict/`，公共 `resolve_p4_conflict` 与全部既有人工裁决语义保持不变。Windows hard gate run `31318631427` / job `93257903560` 已通过 Research/Database/Prediction 专项、Application Ports、完整 architecture、rustfmt、Application check/tests、workspace Clippy `-D warnings` 与 workspace tests，并生成 clean implementation `c1549738041c0e6d3cb046c16fa193c47a14553f`。该 clean HEAD 的 Public Platform CI run `31319176935` / Windows Automated job `93259283555` 已整体 `SUCCESS`，validation evidence upload 成功；artifact `9039991499` 大小 `14210568` 字节，SHA-256 `e48cf0649449fc74488a60ea560a43a224c3f49fe3f9723ad1e039cc06f79d75`。AT5 临时 workflow / generator / fix / marker 已清理；R3-07 五个 Atomic Tasks 全部完成并正式关闭为 `DONE`，R3-08 已在 R3-04 历史验证闭环后按阶段依赖开放为 `READY`。


### R3-08 当前执行

- Atomic Task 1 — Review Core：`DONE`。
- Atomic Task 2 — Match Review Package：`DONE`。
- R3-08 整体已关闭为 `DONE`；Atomic Task 3 — Postmatch Service：`DONE`；Atomic Task 4 — Analytics：`DONE`；R3-09 Exchange / AI Workspace / Release Services 已开放为 `READY`。
- AT3 canonical 提交 `b551ac8acc4030f05d93100812b316469dc7ea83` 的 Public Platform CI run `31420385032` / Windows Automated job `93559423579` 已整体 `SUCCESS`，validation evidence upload 成功；artifact `9075930419` 大小 `14152359` 字节，SHA-256 `39fab48a3f2d62e1538276fb8f19fb811f1c72c79380de55897e12c82e39f582`。18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试继续按既有安全设计保持 `ignored`，未记为已执行。


### R3-08 Atomic Task 1 — Review Core

- 状态：`DONE`。
- 已将 `review.rs` 的 6 个 Review Core 公共职责迁入 `services/review/` 与 `use_cases/review/`，持久化适配进入 `composition/adapters/review.rs`；旧 `crates/application/src/review.rs` 已删除。
- Windows hard gate run `31328023642` 已通过 Review 专项、Application Ports、完整 architecture、保护资产、Application check/tests、完整 `verify:frontend`、完整 `verify:rust`、精确作用域和 clean-tree 检查。
- 首次正式 Public Platform CI run `31328591975` 在 Domain 类型清单漂移门禁停止；Windows Automated 因前置门禁失败被跳过。该失败未记为通过，AT1 未进入 DONE，AT2 未启动。
- 已确认漂移仅来自 Application Rust owner 文件集合变化，正在用既有确定性生成器刷新 `architecture/domain-type-inventory.json` 并重新执行完整门禁。
- 第二次正式 Public Platform CI run `31349803381` 已通过 Domain inventory、Domain 根出口及 R3-01～R3-07 既有专项门禁，随后因 `verify-review-service.mjs` 自身跨行字符串语法错误停止；Windows Automated 未执行。AT1 继续保持 `VERIFYING`，当前修复只针对验证器语法并增加 Node 22 `node --check`，AT2 未启动。
- R3-08 AT1 已正式关闭为 `DONE`：validator recovery run `31350028026` 全链 SUCCESS；正式 Public Platform CI run `31350677129` / job `93340716563` 全部 SUCCESS，artifact `9049246515`，SHA-256 `62f0bcfdb8f83ce0a715de58ee14f8a0a87b1f3192c938e841d65400b8ecb7ef`。下一 Atomic Task 为 Match Review Package，状态 `READY`；AT2 尚未实施。

### R3-08 Atomic Task 2 — Match Review Package

- 状态：`DONE`。旧 `match_review_package.rs` 已按导出、预检、生命周期、共享规则/XLSX I/O 拆入 Review Service/Use Case/Ports；7 个公共入口保持兼容，Postmatch/Analytics 未提前迁移。
- 初始 `31359686297` 已通过专项、Application Rust、完整 frontend/Rust 与 scope；fresh-checkout `31360521486` 暴露 Domain inventory 未固化及 R3-01 冻结 WorkflowPort 误删。最终 recovery `31362128833` 已恢复 37-Port 契约、将 AT2 低层状态接口明确为 `MatchReviewPackageStatePort`，并通过专项、完整 architecture、保护资产、Application Rust、完整 frontend/Rust、scope、clean commit 与 clean-tree。canonical 提交 `23997cde7d3ba46db34fdd2e0577075555391986` 的 Public Platform CI run `31406717077` / Windows Automated job `93514736356` 已 `SUCCESS`，AT2 已正式标记 `DONE`。
- 在最终关闭前，canonical run `31377744818` 曾因 `verify-match-event-facts.mjs` 仍读取已删除旧 owner 而 `ENOENT` 失败；修复提交 `23997cde7d3ba46db34fdd2e0577075555391986` 只迁移 verifier 读取路径到 `use_cases/review/package/shared.rs`，原事件事实断言未弱化。PR clean CI run `31404194850` / job `93506381047` 与 canonical run `31406717077` / job `93514736356` 均完整 `SUCCESS`；canonical evidence artifact `9070811986` 大小 `14156296` 字节。18 个 PostgreSQL 集成测试因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 保持 `ignored`，未记为已执行。
- AT2 canonical run `31363234205`：architecture PASS，Windows Automated 因旧 Stage-A verifier 读取已删除 `match_review_package.rs` 失败；当时只迁移该验证器 authoritative source 路径，随后 run `31377744818` 又暴露 `verify-match-event-facts.mjs` 的第二处旧 owner 路径，并由提交 `23997cde7d3ba46db34fdd2e0577075555391986` 修复；最终 canonical run `31406717077` 全链 `SUCCESS`。

### R3-08 Atomic Task 3 — Postmatch Service

- 状态：`DONE`。
- 7 个 Postmatch 公共职责按独立 Use Case 迁入 `services/postmatch/`、`use_cases/postmatch/`，旧 `crates/application/src/postmatch.rs` 删除；两个既有 Postmatch Ports 对齐真实查询参数并由 `composition/adapters/postmatch.rs` 实现。
- 结算继续复用 AT2 `MatchReviewPackageStatePort` 维护 `review_created -> settled` 状态机，没有新增重复 workflow owner；Analytics 未修改。
- 旧 Postmatch Application owner 的全仓引用已扫描；Domain inventory 由既有确定性生成器刷新，Review Service、Stage-A 与 Postmatch Settlement verifier 仅迁移到新 authoritative owner，原门禁不弱化。
- Windows hard gate：run `31419165233`：Postmatch 专项、Application Ports、完整 architecture、保护资产、rustfmt、Application check/tests、完整 frontend 与完整 Rust 回归均 `SUCCESS`。
- canonical 提交 `b551ac8acc4030f05d93100812b316469dc7ea83` 的 Public Platform CI run `31420385032` / Windows Automated job `93559423579` 已整体 `SUCCESS`；artifact `9075930419` 大小 `14152359` 字节，SHA-256 `39fab48a3f2d62e1538276fb8f19fb811f1c72c79380de55897e12c82e39f582`。AT3 已正式关闭为 `DONE`；该时点随后开放 Analytics AT4。


### R3-08 Atomic Task 4 — Analytics Service

- 状态：`DONE`。
- 删除旧 `crates/application/src/analytics.rs`，建立唯一 `AnalyticsService`、三个分责 Analytics Ports、composition adapters，并将 21 个公共 Analytics 用例全部拆入独立目录；旧 owner 与聚合型 `jobs.rs` / `ai_package.rs` / Parameter Lifecycle 多用例文件不再作为公共用例 owner。
- Stage Regression run `31461057749`：完整 frontend、workspace Rust/Clippy/tests、protected assets、最终 clean-tree 全部 `SUCCESS`。
- clean publication commit `fcf5f3df29b477baf7e1c3aeebcf5ed6f459b8a8` 的 PR #16 CI run `31461760837` 与 canonical CI run `31463538968` / job `93691679605` 均整体 `SUCCESS`；canonical artifact `9091238488`，大小 `14092031` 字节，SHA-256 `ae08e7eeccb09f4b625885716391f88de2ad4ce3902d59608c4978bc63f29a72`。
- 18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试按既有安全设计保持 `ignored`，未记为已执行；未执行破坏性数据库验证。
- R3-08 已正式关闭为 `DONE`；下一任务 R3-09 Exchange / AI Workspace / Release Services 为 `READY`。


### R3-09 当前执行

- Atomic Task 1 — Match Lineup / AI Match Package Exchange：`DONE`。
- Atomic Task 2 — Spreadsheet Exchange：`DONE`。旧 `crates/application/src/spreadsheet.rs` 已删除，16 个 Spreadsheet 公共入口迁入唯一 `ExchangeService` 与独立 Use Cases；facade / service / adapters 已按 Match Lineup / Spreadsheet 职责拆分。
- Atomic Task 3 — AI Workspace：`DONE`。旧 `crates/application/src/api_workspace.rs` 已删除，12 个既有 ApplicationService 入口迁入唯一 `AiWorkspaceService` 与独立 Use Cases；Session / Context / Operation / Presets / Attachments 已分责，Apply Operation 进一步拆为 orchestration / dispatch / payload / metadata。
- Atomic Task 4 — Release：`DONE`。旧 `release_acceptance.rs` 已删除，3 个 Release Acceptance 公共入口已迁入唯一 `ReleaseService` 与职责拆分 Use Cases；严格 hard gate 与最终 clean Public Platform CI 均通过，PR #20 已合并。`R3-09` 正式关闭，`R3-10` 开放为 `READY`。
- AT2 Windows hard gate run `31512091398` / job `93848182746` 已通过最终 rustfmt、官方 Domain inventory、完整 architecture、workspace Clippy `-D warnings` 与 workspace tests；Application tests 35/35、Domain Serde 17/17、Spreadsheet IO 12/12 等均无失败。18 个 PostgreSQL 集成测试因未配置专用 `FOOTBALL_TEST_DATABASE_URL` 保持 `ignored`，未记为已执行。
- 最终 clean Public Platform CI run `31513432237` / Windows Automated job `93852598090` 已在正式只读 workflow 与 clean HEAD `8719008ba63241d313fe02ce4328ee8d0a727e9d` 上整体 `SUCCESS`；artifact `9110944721` 大小 `14040458` 字节，SHA-256 `f0628be97bbfb13a765da3e4799bd7e8abf549352e391cb79ef1111cb9522a9f`。AT2 已关闭为 `DONE`。
- AT3 hard gate run `31553249625` / job `93980269528` 已通过官方 Domain inventory、完整 architecture、37-Port 契约、新 AI Workspace 专项、Application check、Application tests 33/33、Clippy `-D warnings` 与 formatter scope；临时 workflow 已自删除。Ports 裸 JSON 阻塞通过显式序列化结果边界修复，历史验证器旧 owner 仅迁移读取源，不弱化断言。最终 clean Public Platform CI run `31553867879` / Windows Automated job `93982110497` 已整体 `SUCCESS`；artifact `9125791161` 大小 `13985122` 字节，SHA-256 `f5662c9e11d1fbd9da12e39efce9b44645144fce33bf6fe674011c2b7c1af0a6`。PR #19 已合并，merge commit `3d925671ccd424e25965ba9409189f32f920bc4f`；AT3 已关闭为 `DONE`，AT4 `READY`，R3-10 继续 `BLOCKED`。


## R3-09 AT4 执行记录

- AT4 Release 已完成源码迁移并进入 `VERIFYING`：旧 `release_acceptance.rs` owner 已删除，3 个公共入口由唯一 `ReleaseService` 编排，运行验收按校验、五组运行事实检查模块、汇总和报告哈希拆分。
- `ReleaseAcceptancePort` 已对齐 runtime-facts / persist / list / read 真实能力并由 `ActiveDatabase` 适配；公共 ApplicationService/Tauri 契约、Persistence SQL/migration、前端产品行为、模型边界和依赖保持不变。
- 最终严格 hard gate run `31568170298` / job `94024298468` 已 `SUCCESS`：Release 专项、历史发布契约、完整 architecture、官方 inventory、rustfmt、Application check、33/33 tests 与 Clippy `-D warnings` 全部通过。18 个专用 PostgreSQL 集成测试未执行且未记为通过。
- 临时验证 workflow 已清理；PR #20 保持 Draft / Open / 未合并，等待最终 clean Public Platform CI 后再决定正式关闭。R3-09 继续 `IN_PROGRESS`，R3-10 继续 `BLOCKED`。
- 首轮最终 clean Public Platform CI run `31568684129` / job `94025845936` 中独立 architecture 已通过，Windows Automated 在完整 frontend 的 deterministic protected-assets 门禁停止：AT4 为迁移旧 Release owner 而合法修改了受保护的 `scripts/verify-public-model-boundary.mjs` 权威扫描路径，但 `architecture/protected-assets.json` 尚未同步该验证器的新指纹；这不是 Release 业务、编译或契约失败，且该轮未被记为通过。
- 按仓库既有 `chore(verify): refresh public-boundary fingerprint` 机制，只刷新该受保护验证器的 Git blob / fingerprint 与聚合 SHA；refresh workflow 在提交前实际执行 `verify-protected-assets-deterministic.mjs` 并通过。新 blob 为 `2e5adc250dba986b3b0441c7f26e762f8b9ef6f4`，fingerprint 为 `13f9ea8c98624e156208c836f837875f2a54104be15e6115f5e7f547d4f491e0`，聚合 SHA-256 为 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`；保护文件集合、禁止私有资产规则和验证逻辑未放宽。
