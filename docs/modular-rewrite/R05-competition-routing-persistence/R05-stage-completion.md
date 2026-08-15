# R05 阶段完成记录

- 阶段状态：`DONE`
- R5 起点：`615dc952491d5e0e21d4979292cbf5170eedece6`
- R5 最终代码 merge commit：`acb0491003b365b3f775780d8c98ecfdf1e80104`
- 目标平台：Windows

## 1. 阶段目标与完成结论

R5-01 至 R5-06 均已 `DONE`。Competition、Season、Stage、Round、Rule Package、Competition Binding、Route Resolution 与 Model Run Identity PostgreSQL persistence 已从职责混合的旧模块收敛到 `crates/persistence-postgres/src/adapters/competition/` 与 `adapters/rules/` 的具名目录；查询、写入、typed Row、Domain Mapper 与多表 transaction 按职责拆分。

旧 `crates/persistence-postgres/src/competitions.rs` 与 `crates/persistence-postgres/src/routing.rs` 已删除，不保留双实现或长期转发壳。R5 不修改路由决策算法、模型算法、公共 Application Port、Tauri 命令/DTO、数据库 Schema、0001–0046 migration、配置、错误语义或生产依赖。

R5-06 PR #31 已使用 fixed clean HEAD `058884e82f3c584f87751dec3bb5f9b6531a151e` squash merge为 `acb0491003b365b3f775780d8c98ecfdf1e80104`；merged-stage canonical Public Platform CI run `31884882480` / Windows job `95012628426` 为 `SUCCESS`，因此 R5 阶段出口的源码与发布门禁均已满足。

## 2. 已完成节点索引

| 任务 ID | 实施记录 | 完成状态 | 最小/关键验证 |
|---|---|---|---|
| R5-01 | [`R05-01-competitions-repository.md`](R05-01-competitions-repository.md) | DONE | old-owner PG contract `31772436658`; Windows frontend `31773777077`; workspace Rust `31773900831`; PR `31775501711`; merged stage `31777130456` |
| R5-02 | [`R05-02-seasons-stages-and-rounds.md`](R05-02-seasons-stages-and-rounds.md) | DONE | hard gate `31794402789`; PR `31794818136`; merged stage `31796688330` |
| R5-03 | [`R05-03-rule-packages.md`](R05-03-rule-packages.md) | DONE | old/new PG contracts `31810196287` / `31811073556`; hard gate `31811535324`; PR `31812316772`; merged stage `31824523536` |
| R5-04 | [`R05-04-competition-bindings.md`](R05-04-competition-bindings.md) | DONE | old/new PG contracts `31862339200` / `31862713824`; hard gate `31862969585`; PR `31863321636`; merged stage `31864520861` |
| R5-05 | [`R05-05-route-resolution-reads.md`](R05-05-route-resolution-reads.md) | DONE | old/new PG contracts `31869022228` / `31869137395`; hard gate `31869727870`; PR `31873912016`; merged stage `31875035071` |
| R5-06 | [`R05-06-model-run-identity-reads.md`](R05-06-model-run-identity-reads.md) | DONE | isolated PG `31881761988`; source canonical `31881842368`; fixed-head push `31883239072`; PR `31883703954`; merged stage `31884882480` |

## 3. 实际新增文件总表

共 103 个新增文件。

| 文件 |
|---|
| `crates/persistence-postgres/src/adapters/competition/bindings/create_binding/insert_binding.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/create_binding/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/create_binding/transaction.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/create_binding/validation.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/ensure_type_default/find_existing_binding.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/ensure_type_default/insert_binding.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/ensure_type_default/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/ensure_type_default/transaction.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/list_bindings.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/package_route_metadata.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/read_binding.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/record_mapper.rs` |
| `crates/persistence-postgres/src/adapters/competition/bindings/record_row.rs` |
| `crates/persistence-postgres/src/adapters/competition/detail/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/detail/read_competition.rs` |
| `crates/persistence-postgres/src/adapters/competition/detail/record_mapper.rs` |
| `crates/persistence-postgres/src/adapters/competition/detail/record_row.rs` |
| `crates/persistence-postgres/src/adapters/competition/directory/create_competition.rs` |
| `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/deactivate_bindings.rs` |
| `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/delete_external_entity_ids.rs` |
| `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/lock_active_competition.rs` |
| `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/soft_delete_competition.rs` |
| `crates/persistence-postgres/src/adapters/competition/directory/delete_competition/transaction.rs` |
| `crates/persistence-postgres/src/adapters/competition/directory/list_competitions.rs` |
| `crates/persistence-postgres/src/adapters/competition/directory/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/rounds/create_round.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/rounds/list_rounds.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/rounds/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/rounds/read_round.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/rounds/record_mapper.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/rounds/record_row.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/seasons/create_season.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/seasons/list_seasons.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/seasons/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/seasons/read_season.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/seasons/record_mapper.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/seasons/record_row.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/stages/create_stage.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/stages/list_stages.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/stages/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/stages/read_stage.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/stages/record_mapper.rs` |
| `crates/persistence-postgres/src/adapters/competition/hierarchy/stages/record_row.rs` |
| `crates/persistence-postgres/src/adapters/competition/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/read_identity.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/record.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/record_row.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/definition.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/parameter_set.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/record.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/transaction.rs` |
| `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/version.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/context/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/context/record_mapper.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/context/record_row.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/context/resolve_context.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/context/season_context.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/context/stage_context.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/context/validate_scope.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/route/binding_candidate.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/route/explicit_rule_package.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/route/mod.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/route/record_mapper.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/route/record_row.rs` |
| `crates/persistence-postgres/src/adapters/competition/route_resolution/route/resolve_route.rs` |
| `crates/persistence-postgres/src/adapters/rules/mod.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/list_rule_packages.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/mod.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/record_mapper.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/record_row.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/register_rule_package/attach_competition_profile.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/register_rule_package/find_existing_package.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/register_rule_package/insert_rule_package.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/register_rule_package/mod.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/register_rule_package/transaction.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/source_documents/mod.rs` |
| `crates/persistence-postgres/src/adapters/rules/packages/source_documents/upsert_source_document.rs` |
| `crates/persistence-postgres/tests/competition_bindings_repository_contract.rs` |
| `crates/persistence-postgres/tests/competition_hierarchy_repository_contract.rs` |
| `crates/persistence-postgres/tests/competitions_repository_contract.rs` |
| `crates/persistence-postgres/tests/model_run_identity_repository_contract.rs` |
| `crates/persistence-postgres/tests/route_resolution_repository_contract.rs` |
| `crates/persistence-postgres/tests/rule_package_repository_contract.rs` |
| `docs/modular-rewrite/R05-competition-routing-persistence/R05-01-competitions-repository.md` |
| `docs/modular-rewrite/R05-competition-routing-persistence/R05-02-seasons-stages-and-rounds.md` |
| `docs/modular-rewrite/R05-competition-routing-persistence/R05-03-rule-packages.md` |
| `docs/modular-rewrite/R05-competition-routing-persistence/R05-04-competition-bindings.md` |
| `docs/modular-rewrite/R05-competition-routing-persistence/R05-05-route-resolution-reads.md` |
| `docs/modular-rewrite/R05-competition-routing-persistence/R05-06-model-run-identity-reads.md` |
| `docs/modular-rewrite/R05-competition-routing-persistence/R05-stage-completion.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/README.md` |
| `scripts/verify-competition-bindings.mjs` |
| `scripts/verify-competition-hierarchy.mjs` |
| `scripts/verify-competition-repository.mjs` |
| `scripts/verify-model-run-identity.mjs` |
| `scripts/verify-route-resolution.mjs` |
| `scripts/verify-rule-package-repository.mjs` |

## 4. 实际修改文件总表

共 8 个修改文件。

| 文件 |
|---|
| `README.md` |
| `architecture/domain-type-inventory.json` |
| `crates/persistence-postgres/src/adapters/mod.rs` |
| `crates/persistence-postgres/src/lib.rs` |
| `crates/persistence-postgres/src/model_runs.rs` |
| `docs/modular-rewrite/R05-competition-routing-persistence/README.md` |
| `package.json` |
| `scripts/verify-persistence-mapping.mjs` |

## 5. 实际移动或重命名文件总表

无。

## 6. 实际删除文件总表

共 2 个删除文件。

| 文件 |
|---|
| `crates/persistence-postgres/src/competitions.rs` |
| `crates/persistence-postgres/src/routing.rs` |

## 7. 最终目录与职责边界

- `adapters/competition/directory/`：Competition create/list/delete；soft delete 的锁定、binding 停用、external ID 清理与 competition soft-delete 各自独立，transaction 只编排。
- `adapters/competition/detail/`：Competition detail SELECT、typed Row 与 Domain Mapper。
- `adapters/competition/hierarchy/seasons/`、`stages/`、`rounds/`：各层级独立 create/read/list、typed Row 与 mapper。
- `adapters/rules/packages/`：Rule Package register/list、source document upsert、typed Row/mapper 与 registration transaction。
- `adapters/competition/bindings/`：Binding list/detail/create/type-default、package route metadata、typed Row/mapper 与具名 transaction。
- `adapters/competition/route_resolution/context/`：Competition hierarchy context read 与 scope validation；`route/`：explicit package、binding candidate、result row/mapper 与 route read coordinator。
- `adapters/competition/model_run_identity/`：单次 Model Run identity read、typed Row/record mapper；`registration/`：definition/version/parameter-set 与 model registration transaction。
- `model_runs.rs` 只编排 run document 本体并委托 identity owner；不再直接拥有 identity JOIN。
- `competitions.rs` / `routing.rs` 已从最终树删除。

## 8. 最终调用流、数据流和状态所有权

Competition / Rules Application Port -> Application composition adapter -> `PostgresStore` -> R5 具名 adapter query/command -> typed SQLx Row -> Domain Mapper -> Domain / route result。

Model Run detail 的 identity 路径为 `model_runs::read_run -> read_model_run_identity -> typed Row -> identity record`；Rule Package registration 通过 crate 内 `register_model_in_tx` 进入 `model_run_identity/registration/` 唯一事务边界。

数据库继续是 Competition、hierarchy、binding 与 model identity 的事实源；R5 未新增进程内业务缓存、Repository 全局工作流状态或第二状态 owner。

## 9. 公共接口、DTO、Schema、数据与配置变化

- Application Competition / Rules Ports：名称、参数、返回类型和调用语义未改变。
- Tauri 公共命令、DTO 与前端调用契约：未改变。
- PostgreSQL Schema、0001–0046 migration 与持久化格式：未改变。
- Competition/Season/Stage/Round ID、Binding 关系与历史运行引用：未改变。
- 配置键、默认值、环境变量、日志等级和生产依赖：未改变。
- `ModelRegistration` 公共导出、`PostgresStore::register_model` 调用语义及 `read_run` identity JSON 字段：未改变。
- 路由决策算法与模型算法：未改变。

## 10. 保持不变的兼容行为

- Competition CRUD、层级读取与现有 binding 语义保持。
- Route specificity/fallback、有效期、模型过滤、explicit package、reason/error 语义保持。
- Model identity 的 model/version/parameter/rule-package/binding nullable 语义保持。
- 模型版本/参数版本冲突错误保持既有语义。
- 历史 migration 内容冻结，R5 未执行任何 migration 改写。
- `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 与关联受保护资产未作为 R5 修改范围。

## 11. 旧实现、重复实现和临时路径清理结果

- `crates/persistence-postgres/src/competitions.rs`：删除。
- `crates/persistence-postgres/src/routing.rs`：删除。
- R5-01～R5-06 的 transient PostgreSQL / inventory / diagnostic workflow 均在对应验证后清理，最终 stage tree 不保留临时 gate。
- 未保留 `old/new/legacy/copy/final/v2` 长期业务目录或无退出计划兼容层。
- 所有生产入口在各节点内完成唯一 owner 切换；失败验证 tree 未合并到 stage 分支。

## 12. 阶段级验证与真实结果

- R5-01：Competition Repository PostgreSQL 16 contract `31772436658` / job `94680947724` `SUCCESS`；Windows frontend `31773777077` / job `94684879652` `SUCCESS`；workspace Rust `31773900831` / job `94685243308` `SUCCESS`；PR #26 `31775501711` 与 merged-stage `31777130456` 均 `SUCCESS`。
- R5-02：最终 hard gate `31794402789` / job `94748190282` `SUCCESS`；PR `31794818136` / job `94749470982` 与 merged-stage `31796688330` / job `94755204665` `SUCCESS`。
- R5-03：old/new Rule Package PG contracts `31810196287` / `31811073556` 均 `SUCCESS`；hard gate `31811535324`、PR `31812316772`、merged-stage `31824523536` 均 `SUCCESS`。
- R5-04：old/new Binding PG contracts `31862339200` / `31862713824` 均 `SUCCESS`；hard gate `31862969585`、PR `31863321636`、merged-stage `31864520861` 均 `SUCCESS`。
- R5-05：old/new Route Resolution PG contracts `31869022228` / `31869137395` 均 `SUCCESS`；hard gate `31869727870`、PR #30 `31873912016`、merged-stage `31875035071` 均 `SUCCESS`。
- R5-06：isolated PostgreSQL 16 run `31881761988` 的 identity job `95005200186` 与 Rule Package regression job `95005200191` 均 `SUCCESS`；clean source canonical `31881842368` / job `95005434692` `SUCCESS`；fixed clean HEAD `058884e82f3c584f87751dec3bb5f9b6531a151e` 的 push canonical `31883239072` / `95008635990` 与 PR #31 canonical `31883703954` / `95009730520` 均 `SUCCESS`。
- PR #31 squash merge `acb0491003b365b3f775780d8c98ecfdf1e80104` 后 merged-stage canonical Public Platform CI `31884882480` / Windows job `95012628426`：`SUCCESS`；artifact `9247274708`，大小 `13936932` 字节，SHA-256 `4c6b107922787a732537a83b58b005bb364ff10298146c87f0dff40f8bdf5a26`。
- canonical Windows gate 实际覆盖 architecture verifier、公开/受保护模型边界、frontend verification、Cargo.lock/rustfmt/workspace Clippy `-D warnings`、workspace tests、Tauri build、release binary startup 与 runtime error scan。

## 13. 未执行验证、环境阻塞和剩余风险

- R5 没有在用户现有 PostgreSQL 数据库执行写入、destructive reset 或 broad 18 项 ignored `postgres_integration.rs` 全量回归；各节点使用 GitHub Actions 临时 PostgreSQL 16 专用测试库执行 domain-specific contract。历史数据库兼容由冻结的 0001–0046 migration、既有 schema contract 与节点 PG contracts 覆盖，但本阶段没有把“用户真实现有 DB sample”描述为已执行。
- R5-01 Ubuntu frontend diagnostic 曾因 Chromium `DevToolsActivePort` 环境阻塞，Ubuntu workspace Rust 曾因 `glib-2.0/gobject-2.0` 缺失阻塞；相同目标源码的 Windows frontend 与 Windows workspace Rust 已分别实际通过。
- R5-05 Ubuntu 专项 gate 同样受 runner `glib-2.0` 系统库缺失阻塞；相同源码的 canonical Windows gate 实际通过。
- Windows `Full` 人工交互验收与用户本机数据库验收未由云端 Automated 替代；本阶段没有将其虚报为通过。
- 未发现需要新增生产依赖、Schema migration 或兼容层的剩余风险。

## 14. 根 README、阶段 README 与架构文档同步

- 根 `README.md` 已记录 R5-06 与 R5 阶段实际实现、兼容边界、关键验证和完成记录链接。
- 本阶段 `README.md` 已将 R5-01～R5-06 全部标记为 `DONE`，并记录 PR #31、merged-stage canonical 与阶段完成记录。
- `architecture/domain-type-inventory.json` 仅使用项目官方 generator 在节点变更后刷新；没有手工放宽 drift gate。
- 本阶段未改变 `docs/ARCHITECTURE.md` 所述系统层级或公共架构契约，因此没有为无实际架构变化制造额外文档改写。

## 15. 阶段回退点与回退步骤

R5 起点为已验证 R4 closeout `615dc952491d5e0e21d4979292cbf5170eedece6`。R5 最终代码 merge commit 为 `acb0491003b365b3f775780d8c98ecfdf1e80104`。

如需整体回退 R5，应通过 Git 回退到 `615dc952491d5e0e21d4979292cbf5170eedece6` 或对 R5 的阶段提交执行受控 revert，并重新运行数据库/路由/模型 identity 契约与 canonical Windows gate；不得通过复制已删除的 `competitions.rs` / `routing.rs` 恢复双实现。

## 16. 出口门禁逐项结论

- `docs/modular-rewrite/R05-competition-routing-persistence/README.md` 完整索引 R5-01～R5-06：`PASS`。
- `R05-stage-completion.md` 已创建并包含真实文件总表、验证、限制、回退与订正：`PASS`。
- 赛事与路由持久化旧模块 `competitions.rs` / `routing.rs` 删除：`PASS`。
- Competition / Rules 相关 PostgreSQL Ports 由新 Adapter 唯一实现且无双 owner：`PASS`。
- 路由结果与 Model Run identity 契约保持：`PASS`。
- 模型保护资产、Schema/migration、生产依赖无未批准变化：`PASS`。
- R6 可进入初始化/唯一 `READY` 节点：`PASS`；R6-01 为下一入口。

## 17. 下一阶段唯一 READY 任务

`R6-01 — Team Directory 与 Detail`。

目标目录：

```text
crates/persistence-postgres/src/adapters/catalog/teams/directory/
crates/persistence-postgres/src/adapters/catalog/teams/detail/
```

R6 阶段索引已初始化于 `docs/modular-rewrite/R06-entity-catalog-persistence/README.md`。R6-01 不得在本次 closeout 文档 tree 的 canonical Public Platform CI 成功前开始生产源码实施。

## 18. 订正记录

- R5-06 名称原为 Model Run Identity Reads，但 R5-05 后 `routing.rs` 唯一残余职责还包含模型注册 identity 写入。为满足“唯一 Adapter owner + 删除旧 routing module”的阶段出口，R5-06 在同一 identity 边界内同步迁移 `ModelRegistration` 与 registration transaction；未扩展到模型/路由算法。
- R5-06 首轮 PostgreSQL workflow 让两个 ignored contract 复用同一数据库，第二项因重复 migration 出现 `relation "settings" already exists`；随后改为隔离 PostgreSQL 16 service database，最终双 job 实际通过。该失败保留在节点记录中，没有弱化或跳过测试。
- R5-05 PR 首轮 canonical Clippy 因未使用 typed `RouteRow.competition_kind` 失败；只删除未消费 Row 字段与冗余 SELECT 返回列，保留 WHERE 过滤与 route 语义，失败 tree 未合并。
- 本阶段所有环境阻塞与失败门禁均保留事实记录；没有把未执行、被阻塞或失败的验证描述为通过。
