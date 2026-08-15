# R06-01 — Team Directory 与 Detail

## 状态

`DONE`

生产 owner 已完成切换；专项 PostgreSQL 契约、ownership/inventory/architecture、阶段级 frontend/workspace Rust hard gate、clean PR canonical 与 merged-stage canonical 均已实际通过。R6-01 正式收口为 `DONE`，R6-02 可进入 `READY`。

## 基线与分支

- R5 最终 closeout HEAD：`7512ee805fcba8cac3c8f334680f200d625808c0`。
- 该 HEAD 的 canonical Public Platform CI：run `31886299959` / Windows job `95016342335`，`SUCCESS`；artifact `9247632165`，SHA-256 `3726104665e7102b57effb4ac5c58a5c9dfe7bde20218f7f672b8ef2ededf5c4`。
- R6 stage 分支：`rewrite/r6-entity-catalog-persistence`，从上述 HEAD 精确建立。
- R6-01 实施分支：`agent/r6-01-team-directory-detail`，从上述 HEAD 精确建立。

## 实际问题与任务边界

R6-01 开工前，Team Directory / Detail 能力分散在两个长期混合 owner：

- `crates/persistence-postgres/src/player_catalog.rs`：`create_team`、`list_team_options` 与 Team option/record 动态 Row mapping。
- `crates/persistence-postgres/src/team_catalog.rs`：`list_teams`、`read_team`、`update_team`，同时还包含 R6-02 的 Team Names/Profile 写入和 R6-09 的删除职责。

本节点只迁移 Team Directory 与 Detail：

- `create_team`
- `list_team_options`
- `list_teams`
- `read_team`
- `update_team`
- 上述读路径使用的 Team record/list/option/name/profile/squad/recent-match Row 与 Domain Mapper

明确未提前迁移：

- R6-02：`add_team_name`、`upsert_team_profile` 等 Team Names/Profile 写职责。
- R6-05：team/player periods 与 availability 的事实写入 owner。
- R6-07：coach/formation usage 的事实写入 owner。
- R6-09：archive/delete/force-delete。
- R6-10：global name search。

## 最终职责结构

```text
crates/persistence-postgres/src/adapters/catalog/
└─ teams/
   ├─ directory/
   │  ├─ create_team.rs
   │  ├─ update_team.rs
   │  ├─ list_team_options.rs
   │  ├─ option_row.rs
   │  ├─ option_mapper.rs
   │  ├─ list_teams.rs
   │  ├─ list_row.rs
   │  ├─ list_mapper.rs
   │  └─ name_policy.rs
   └─ detail/
      ├─ read_team.rs
      ├─ read_team_record.rs
      ├─ record_row.rs
      ├─ record_mapper.rs
      ├─ read_names.rs
      ├─ name_row.rs
      ├─ name_mapper.rs
      ├─ read_profile.rs
      ├─ profile_row.rs
      ├─ profile_mapper.rs
      ├─ read_squad.rs
      ├─ squad_row.rs
      ├─ squad_mapper.rs
      ├─ read_recent_matches.rs
      ├─ recent_match_row.rs
      └─ recent_match_mapper.rs
```

职责规则：

- `directory/` 只拥有 Team create/update/list/options 与 name normalization policy。
- `detail/read_team.rs` 仅编排详情读取，不嵌入 SQL。
- 每个 Detail SQL reader 只表达一个读取目的。
- Row 使用 `sqlx::FromRow` typed record；Domain Mapper 与 SQL 分离。
- 既有 player periods、coach periods、formation usage/resolution 继续调用原唯一 owner，不在 R6-01 复制事实写入或计算逻辑。

## 行为与兼容性

保持不变：

- `TeamCatalogPort` 公共方法、参数、返回类型与调用语义。
- Tauri command、DTO、前端 Team 页面与用户可观察行为。
- Team stable ID、player/team period、coach period、formation usage、历史比赛引用。
- 中文/别名检索继续复用共享 `NameSearch`；没有把名称规范化逻辑复制到 SQL。
- `list_teams` 的 country/team type 过滤、稳定 `(normalized_name, id)` cursor pagination 与半游标错误语义。
- Team detail 的 names/profile/current squad、中文本地化球员名、primary position/default tactical role、当前 availability、player/coach periods、recent matches、formation usage/resolution。
- `create_team` 业务写入与 `team_created` audit event 继续在同一 transaction 内提交。
- PostgreSQL Schema、0001–0046 migration、数据格式、配置、环境变量、公共错误类别/传播语义、日志等级。
- Cargo manifests、`Cargo.lock` 与生产依赖。
- `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 及模型保护资产。

## 契约冻结与 owner 切换

### 旧 owner PostgreSQL 16 contract

先创建 `crates/persistence-postgres/tests/team_directory_detail_repository_contract.rs`，在生产 owner 切换前对旧实现执行同一契约：

- run `31888078703` / job `95020173411`：`SUCCESS`。
- 覆盖 create、alias/中文名 option search、list filtering、稳定分页、半游标错误、profile/squad/availability 聚合、detail names/profile/squad/player periods/recent matches、update 与 missing entity 路径。

### owner-switch 失败与修复

- run `31888247096` / job `95020568846`：首次 generated module visibility/re-export 出现 Rust E0364/E0365/E0603；`cargo check` fail-fast，未提交生产切换。
- run `31888389636` / job `95020899291`：修复第一轮 visibility 后，mapper/row import visibility 仍不完整；再次在 `cargo check` fail-fast，未提交生产切换。
- run `31888483586` / job `95021118552`：修复后 `cargo fmt --check`、Persistence/Application `cargo check`、Persistence unit tests、同一 PostgreSQL 16 contract 全部 `SUCCESS`，随后生成生产切换提交 `5056917bd5c0e81aaaa96d1ef70c7362ef874820`。

失败 tree 均未作为生产 owner 提交；没有通过 `allow`、skip、mock 或放宽门禁掩盖错误。

## ownership / inventory / architecture gate

新增永久门禁 `scripts/verify-team-directory-detail.mjs`，锁定：

- target directory/entry 唯一 owner。
- legacy `team_catalog.rs` / `player_catalog.rs` 不再拥有 R6-01 methods/mappers。
- R6-02 Names/Profile writes 与 R6-09 delete 仍保留原 owner，防止提前跨节点迁移。
- typed Row/Mapper、单 SQL purpose、shared NameSearch、cursor error、localized squad/default role、detail coordinator 边界。
- PostgreSQL contract 关键覆盖项。

第一次 ownership/inventory workflow run `31888670048` / job `95021554565` 中：

- official Domain inventory generator：`SUCCESS`；365 个 Domain 类型。
- R6-01 ownership verifier：`SUCCESS`。
- Domain inventory drift：`SUCCESS`；365 public-compatible types、299 PostgreSQL mapping types。
- protected model assets：`SUCCESS`，聚合 SHA-256 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`。
- 完整 `verify:architecture` 在 R4-02 Audit verifier 因仍读取旧 `player_catalog.rs` 的 create-team call-path 而 fail-fast；后续步骤按硬门禁跳过。

修复只把 R4-02 verifier 的 `create_team` owner 指向 `adapters/catalog/teams/directory/create_team.rs`，并继续强制检查 `football.teams` 写入、统一 audit writer 与 `tx.commit()` 的同 transaction 顺序；没有删除或弱化审计原子性约束。

第二次 run `31888800801` / job `95021851053`：`SUCCESS`，实际通过：

- official Domain inventory refresh + drift verifier。
- R6-01 ownership verifier。
- protected model assets verifier。
- 完整 `npm run verify:architecture`。
- `cargo fmt --all -- --check`。
- `cargo check --locked -p football-persistence-postgres -p football-application`。
- Persistence unit tests。
- 同一 R6-01 PostgreSQL 16 contract。

该 run 提交正式 ownership/inventory wiring：`b2999e5dce982bddf2093f0596eec33a7dbd0333`；临时 baseline/owner-switch/inventory workflows 与 helper 均已从当前 branch tree 清理。

## 阶段级 hard gate

阶段级 gate 始终 fail-fast；只有确认真实 owner 已迁移且原契约语义仍需保持时才推进 verifier call-path，没有删除检查、扩大白名单或降低门禁。

前五轮失败均保留：run `31889047691` 停在旧 global-name-search owner 统计；run `31889120709` 停在旧 team-player-management detail source；run `31889255910` 停在旧 entity-relationships team-history source；run `31889343316` 停在旧 database-reset TeamRecord mapper；run `31889463684` 停在旧 entity-resource-center squad localization source。各次修复只将对应 verifier 指向 R6-01 新 owner，并保留原搜索、详情、历史、reset、localized-name 与审计契约。

最终 hard gate run `31889555412` 全部 `SUCCESS`：Windows job `95023694944` 的 `npm run verify:frontend`、`cargo fmt --all -- --check`、workspace Clippy `-D warnings` 与 workspace tests 全部通过；Ubuntu job `95023694946` 的 R6-01 ownership、完整 architecture、模型保护、database baseline freeze 与同一 PostgreSQL 16 contract 全部通过。验证源码 HEAD 为 `051afcd9b08cdb57b862251ae4a66c9a6c5bd203`。

因此 R6-01 的最小验证、阶段 frontend、workspace Rust、architecture、模型保护、数据库静态 baseline 与真实 PostgreSQL 16 专项契约均已有成功证据。

## Clean PR 与 merged-stage canonical

- PR #32 以 clean head `58ec21b48b26ead4f2ee2fec15573c39a2f7ab8b` 进入 `rewrite/r6-entity-catalog-persistence`，PR canonical Public Platform CI run `31890099520` 为 `SUCCESS`。
- PR #32 已按 squash merge 合并，最终代码 merge commit 为 `a54df5ca2695297ea5866a3efd74643239568825`。
- merged-stage canonical Public Platform CI run `31892105125` / Windows job `95029805339` 为 `SUCCESS`；R1 architecture、Windows Automated acceptance 与 validation evidence upload 均通过。
- merged-stage artifact `9249127034`，SHA-256 `972be4e32272e700da265f6e82ce96bc5e56aa356491bb217d6a1076e1b00087`。

## 当前净变更清单（clean PR 前）

### 新增

- `crates/persistence-postgres/src/adapters/catalog/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/create_team.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/list_mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/list_row.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/list_team_options.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/list_teams.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/name_policy.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/option_mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/option_row.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/update_team.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/name_mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/name_row.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/profile_mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/profile_row.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_names.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_profile.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_recent_matches.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_squad.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_team.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_team_record.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/recent_match_mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/recent_match_row.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/record_mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/record_row.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/squad_mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/detail/squad_row.rs`
- `crates/persistence-postgres/tests/team_directory_detail_repository_contract.rs`
- `scripts/verify-team-directory-detail.mjs`
- `docs/modular-rewrite/R06-entity-catalog-persistence/R06-01-team-directory-and-detail.md`

### 修改

- `architecture/domain-type-inventory.json`
- `crates/persistence-postgres/src/adapters/mod.rs`
- `crates/persistence-postgres/src/player_catalog.rs`
- `crates/persistence-postgres/src/team_catalog.rs`
- `package.json`
- `scripts/verify-persistence-audit.mjs`
- `scripts/verify-global-name-search.mjs`
- `scripts/verify-team-player-management.mjs`
- `scripts/verify-entity-relationships.mjs`
- `scripts/verify-database-reset.mjs`
- `scripts/verify-entity-resource-center.mjs`
- `README.md`（本节点同步）
- `docs/modular-rewrite/R06-entity-catalog-persistence/README.md`（本节点同步）

### 移动/重命名

无。

### 删除

无长期生产文件。所有 R6-01 临时 workflow/helper 在进入 clean hard gate 前清理，不进入最终净 diff。

## 未执行项与剩余限制

R6-01 的代码、专项契约、阶段回归、clean PR 与 merged-stage canonical 已全部通过，节点正式 `DONE`。仍未执行且不被云端 Automated 替代的外部/人工验证：

- 未对用户现有 PostgreSQL 数据库执行写入或真实数据 sample 验收。
- 未执行 Windows Full 人工交互验收。

上述两项作为明确限制保留，不影响本 Atomic Task 的云端代码与契约收口结论。

## 回退点

- 节点起点：R5 final closeout `7512ee805fcba8cac3c8f334680f200d625808c0`。
- 当前可验证 owner-switch 核心提交：`5056917bd5c0e81aaaa96d1ef70c7362ef874820`。
- ownership/inventory verified HEAD：`b2999e5dce982bddf2093f0596eec33a7dbd0333`。
- stage hard-gate verified HEAD：`051afcd9b08cdb57b862251ae4a66c9a6c5bd203`。
- R6-01 final stage merge commit：`a54df5ca2695297ea5866a3efd74643239568825`。
- 如需回退，不复制旧 methods 形成双实现；通过 Git 回退到节点起点/最近通过门禁的原子提交并重跑 R6-01 PostgreSQL contract 与 architecture gate。
