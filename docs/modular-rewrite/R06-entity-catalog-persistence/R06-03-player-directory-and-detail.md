# R06-03 — Player Directory 与 Detail

## 状态

`VERIFYING`

R6-03 从已验证的 R6-02 最终 closeout HEAD `4ce9eb405e897beedf8b122e6794d7c9141b660e` 精确建立实施分支 `agent/r6-03-player-directory-detail`。本节点只重写 Player Directory / Detail 的生产持久化 owner；R6-04 Names/Positions、R6-05 Team Periods/Availability、R6-06 Abilities/Dynamic Tags、R6-09 deletion、R6-10 global search 不提前迁移。

## 进入基线

- R6-02：`DONE`。
- R6-02 merged-stage canonical：run `31925219003` / Windows job `95111615032`，`SUCCESS`。
- R6-02 最终 closeout tree canonical：HEAD `4ce9eb405e897beedf8b122e6794d7c9141b660e`，Public Platform CI run `31926475831` / Windows job `95114758145`，`SUCCESS`；artifact `9258046111`，SHA-256 `76ae850a02080163086f0ec504e3a7c3cfed87ba7063e378b2c168298772cf7f`。

## 开工扫描与实际问题

R6-03 开工时，目标职责仍集中在 `crates/persistence-postgres/src/player_catalog.rs`：

- `create_player`：输入校验、名称规范化、Player INSERT、primary name INSERT、审计与事务提交混在同一方法。
- `update_player`：输入校验、名称规范化、旧 normalized-name 锁定、Player UPDATE、primary-name 切换、审计与事务提交混在同一方法。
- `list_players`：分页游标、NameSearch、球队过滤、状态过滤、多个 LATERAL 聚合、动态 Row mapping 与 page cursor 组装集中在单一方法。
- `read_player`：基础 Player、Names、Positions、Team Periods、Availability、Ability Profile/Observations、Dynamic Tags、External IDs 的多段查询和最终 `PlayerDetail` 聚合集中在单一方法。

直接上游公共边界保持冻结：Application `PlayerCatalogPort` 的 `create_player`、`update_player`、`list_players`、`read_player` 方法签名不变；Tauri catalog commands/DTO 映射不变。

## 已实施结构与职责边界

生产 owner 已切换到 `crates/persistence-postgres/src/adapters/catalog/players/`：

- `directory/`：`create_player`、`update_player`、`list_players`，并拆分 input policy、typed list row、list mapper；目录查询继续复用共享 `NameSearch`。
- `record/`：基础 `PlayerRecordRow` 与 `map_player_record` 独立承担数据库 Row → Domain record 映射。
- `detail/read_player.rs`：只负责 Detail 聚合编排。
- `detail/names/`、`positions/`、`team_periods/`、`availability/`、`abilities/profile/`、`abilities/observations/`、`external_ids/`：各自拥有单用途读取 SQL、typed Row/Mapper 或对应读取职责。
- Dynamic Tag 继续由既有独立 owner `crates/persistence-postgres/src/dynamic_tags.rs` 承担，没有被错误并入 R6-03。
- R6-04 Names/Positions writes、R6-05 Team Periods/Availability writes、R6-06 Abilities/Dynamic Tags writes 与 R6-09 deletion 继续留在既有 owner，没有提前跨节点迁移。

旧 `player_catalog.rs` 已退出 `create_player`、`update_player`、`list_players`、`read_player` 的生产 owner；未保留长期双实现或转发壳。

## 保持不变的契约

- 公共 `PlayerCatalogPort`、Tauri command/DTO、Domain shape、Schema、0001–0046 migration、配置、错误语义、日志等级和用户可观察行为不变。
- `create_player` / `update_player` 保留姓名不能为空、身高 120–230 cm、球员不存在等既有错误语义；名称规范化、primary name 切换、metadata merge 与 audit transaction 语义不变。
- `list_players` 保留 limit clamp、成对 pagination cursor、中文/英文/别名/重音/多关键词 NameSearch、team/status 过滤、稳定排序与 next cursor 语义。
- Detail 继续聚合 Names、Positions、Team Periods、Availability、Abilities、Dynamic Tags、External IDs；只移动读取 owner，不改变后续 R6 节点对应写入 owner。
- 默认战术角色/位置映射和历史 P4/运行引用不变。
- 未新增生产依赖，未修改模型保护区。

## 验证事实

### PostgreSQL 16 与 owner switch

- 旧 owner PostgreSQL 16 contract run `31929515802`：`SUCCESS`。
- Player Directory owner switch run `31929916934`：`SUCCESS`。
- Player Detail owner switch run `31930614713`：`SUCCESS`。
- PostgreSQL contract：`crates/persistence-postgres/tests/player_directory_detail_repository_contract.rs`。
- R6-03 ownership verifier：`scripts/verify-player-directory-detail.mjs`。

### Official inventory 与 legacy verifier 跟随 owner 迁移

- 首轮 hard gate `31930748342` fail-fast：R6-03 verifier 错把 Dynamic Tag 视为 `player_catalog.rs` owner，同时 official Domain type inventory 已因真实源码迁移发生漂移；未降低门禁。
- Domain inventory refresh run `31932154320` 为 `SUCCESS`，只刷新 `architecture/domain-type-inventory.json`；生成提交 `d016ac312e2da4b4d443833950f7811439633305`。临时 refresh workflow 随后已删除。
- hard gate `31932204170`：PostgreSQL/architecture job 成功；Windows `verify:frontend` 发现 `verify-global-name-search.mjs` 仍绑定旧 Player list owner，随后仅将原有 7 个 NameSearch 接入检查跟随到新 Directory owner。
- hard gate `31932294193`：Windows 继续发现 `verify-player-role-inheritance.mjs` 把当前 Player Directory 默认角色投影绑定在旧 owner；只迁移当前目录投影检查，历史阵容 `lineup.captured_at::date` 检查仍留在 `player_catalog.rs`。
- hard gate `31932368637`：Windows 继续发现 `verify-database-reset.mjs` 仍查找旧 `player_record_from_row`；只将 PlayerRecord 映射契约跟随到 `record/mapper.rs::map_player_record`。
- hard gate `31932458011`：数据库 reset 验证已通过，随后 `verify-entity-resource-center.mjs` 发现 Player localized-name SQL/mapper 仍绑定旧 owner；只将中文名查询/映射检查分别跟随到 `directory/list_players.rs` 与 `directory/list_mapper.rs`。
- hard gate `31932529241`：上述验证均通过，随后 `verify-stage-e1-followup.mjs` 发现 alternate-name 查询仍绑定旧 owner；只将原有英文/别名查询断言跟随到 `directory/list_players.rs`。
- 所有这些修复均为 verifier owner-path 更新；原契约阈值、语义断言和失败条件未删除、跳过或放宽。

### 最终阶段 hard gate

最终 stage hard gate run `31932648476` 全部通过：

- Windows job `95129707185`：`SUCCESS`。
  - `npm run verify:frontend`：`SUCCESS`；包括 Domain inventory、全局 NameSearch、角色继承、数据库 reset、Entity Resource Center、Stage E1 follow-up、截图回归、TypeScript/Vite 等现有聚合门禁。
  - `cargo fmt --all -- --check`：`SUCCESS`。
  - `cargo clippy --locked --workspace --all-targets -- -D warnings`：`SUCCESS`。
  - `cargo test --locked --workspace`：`SUCCESS`。
- PostgreSQL/architecture job `95129707240`：`SUCCESS`。
  - R6-01 / R6-02 retained ownership、R6-03 ownership：`SUCCESS`。
  - `npm run verify:architecture`：`SUCCESS`。
  - protected assets 与 database baseline freeze：`SUCCESS`。
  - PostgreSQL 16 `player_directory_detail_repository_contract`：`SUCCESS`。
- 模型保护资产聚合 SHA-256 继续为 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`。
- 临时 `r6-03-hard-gate.yml` 已在 hard gate 成功后从实施分支删除；临时 inventory refresh workflow 也已删除。

## 当前未执行与剩余门禁

- 用户现有 PostgreSQL 数据库真实数据写入/sample 验收：未执行。
- Windows Full 人工交互验收：未执行；现有 Windows 自动化/截图/静态门禁通过不等价于人工 Full 验收。
- clean PR canonical、squash merge、merged-stage canonical：尚未完成，因此当前状态保持 `VERIFYING`，不得提前标记 `DONE`。

## 回退点

- R6-03 节点起点：`4ce9eb405e897beedf8b122e6794d7c9141b660e`。
- 所有实施通过 Git 提交回退，不复制旧实现、不保留长期双 owner 或转发壳。
