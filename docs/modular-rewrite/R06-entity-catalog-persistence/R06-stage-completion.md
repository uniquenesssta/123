# R06 阶段完成记录

- 阶段状态：`DONE`
- R6 起点：`7512ee805fcba8cac3c8f334680f200d625808c0`
- R6 最终生产代码：`809cfb429ec31c165616e65e1b6169f928ee4dcb`
- 最终出口验证：run `32276040092` / Windows job `96143592950` / PostgreSQL 16 job `96143592734`
- 目标平台：Windows + PostgreSQL 16

## 1. 阶段目标与完成结论

R6-01 至 R6-10 均已 `DONE`。Teams、Players、Coaches、Formations、Abilities、Dynamic Tags、Availability、Entity Matching、References、Deletion 与 Global Search persistence 已收敛到 `crates/persistence-postgres/src/adapters/catalog/`。

阶段出口残留的 Team Detail player-period projection、Position reference list 与 season-team membership option read 已进入对应 catalog owner；旧 `entity_catalog.rs` 已删除。`player_catalog.rs` 只保留 R7 Match/Lineup 范围职责。

公共 Port/DTO、Tauri 命令、Schema、0001–0046 migrations、配置、错误语义、UI、模型保护资产与生产依赖保持兼容。最终出口 run `32276040092` 的 Windows job `96143592950` 与 PostgreSQL 16 job `96143592734` 均 `SUCCESS`，R6 满足出口条件。

## 2. 已完成节点索引

| 任务 | 范围 | 记录 | 状态 |
|---|---|---|---|
| R6-01 | Team Directory 与 Detail | [`R06-01-team-directory-and-detail.md`](R06-01-team-directory-and-detail.md) | DONE |
| R6-02 | Team Names 与 Profiles | [`R06-02-team-names-and-profiles.md`](R06-02-team-names-and-profiles.md) | DONE |
| R6-03 | Player Directory 与 Detail | [`R06-03-player-directory-and-detail.md`](R06-03-player-directory-and-detail.md) | DONE |
| R6-04 | Player Names 与 Positions | [`R06-04-player-names-and-positions.md`](R06-04-player-names-and-positions.md) | DONE |
| R6-05 | Team Periods 与 Availability | [`R06-05-team-periods-and-availability.md`](R06-05-team-periods-and-availability.md) | DONE |
| R6-06 | Abilities 与 Dynamic Tags | [`R06-06-abilities-and-dynamic-tags.md`](R06-06-abilities-and-dynamic-tags.md) | DONE |
| R6-07 | Coaches 与 Formation Usage | [`R06-07-coaches-and-formation-usage.md`](R06-07-coaches-and-formation-usage.md) | DONE |
| R6-08 | Entity Matching 与 References | [`R06-08-entity-matching-and-references.md`](R06-08-entity-matching-and-references.md) | DONE |
| R6-09 | Archive / Delete / Force Delete | [`R06-09-archive-delete-and-force-delete.md`](R06-09-archive-delete-and-force-delete.md) | DONE |
| R6-10 | Global Name Search | [`R06-10-global-name-search.md`](R06-10-global-name-search.md) | DONE |

## 3. 最终职责边界

- `teams/`：Team directory/detail、names、profiles 与 player-period projection。
- `players/`：Player directory/detail、names、positions、team periods、position reference 与 season-team membership read。
- `availability/`、`abilities/`、`dynamic_tags/`：Player signals。
- `coaches/`、`formations/`：Coach 与 formation persistence。
- `entity_matching/`、`references/`：实体匹配与引用。
- `deletion/`：preflight、archive、safe/permanent/force delete。
- `global_search/`：统一名称搜索。
- `player_catalog.rs` 剩余 Match/Lineup 职责归 R7。

## 4. 最终验证

- Windows job `96143592950`：`verify:architecture` + 官方 `windows-acceptance.ps1 -Mode Automated`，覆盖 frontend、Rust、release build 与 runtime smoke。
- PostgreSQL 16 job `96143592734`：architecture、protected assets、database baseline、Domain inventory 与 R6-01～R6-10 的 12 个 retained contracts。
- 残留 owner 前置门禁已通过完整 architecture、97/97 Persistence unit tests 与 3 个 focused PostgreSQL contracts；生产收尾 `809cfb429ec31c165616e65e1b6169f928ee4dcb`。

## 5. 兼容与保护结论

- 中文/英文/历史名/别名/重音/多关键词检索与稳定分页保持。
- 默认战术角色与位置映射保持。
- 删除引用不破坏历史 P4/运行引用；force purge 保持审计与显式确认。
- 同源 ID matching 规则保持。
- 模型保护区与 0001–0046 migrations 未修改；无新增生产依赖。

## 6. 实际新增文件

共 241 项。

| 文件 |
|---|
| `AGENTS.md` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/list.rs` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/dimensions/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/observations/add.rs` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/observations/input_policy.rs` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/observations/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/observations/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/abilities/observations/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/availability/add_player_availability.rs` |
| `crates/persistence-postgres/src/adapters/catalog/availability/input_policy.rs` |
| `crates/persistence-postgres/src/adapters/catalog/availability/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/availability/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/availability/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/detail/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/detail/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/directory/create.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/directory/list.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/directory/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/existence.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/mapping/coach.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/mapping/external_id.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/mapping/listing.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/mapping/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/mapping/name.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/mapping/period.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/names/add.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/names/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/normalization.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/team_periods/add.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/team_periods/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/team_periods/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/coaches/validation.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/archive/bulk.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/archive/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/archive/write.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/bulk_delete.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/delete_write.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/force_delete/counts.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/force_delete/execute.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/force_delete/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/force_delete/operation.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/ids.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/preflight/check.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/preflight/labels.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/preflight/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/preflight/references.rs` |
| `crates/persistence-postgres/src/adapters/catalog/deletion/safe_delete.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/scoring.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/list.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/definitions/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/add.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/input_policy.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/list.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/dynamic_tags/tags/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/entity_matching/existence.rs` |
| `crates/persistence-postgres/src/adapters/catalog/entity_matching/external_id.rs` |
| `crates/persistence-postgres/src/adapters/catalog/entity_matching/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/entity_matching/name_candidates.rs` |
| `crates/persistence-postgres/src/adapters/catalog/entity_matching/normalization.rs` |
| `crates/persistence-postgres/src/adapters/catalog/entity_matching/outcome.rs` |
| `crates/persistence-postgres/src/adapters/catalog/entity_matching/resolve.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/constants.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/directory/list.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/directory/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/directory/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/resolution/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/resolution/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/resolution/resolve.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/grouping.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/key.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/preparation.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/probability.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/save.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/tests.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/validation.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/window.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/window/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/formations/usage/write.rs` |
| `crates/persistence-postgres/src/adapters/catalog/global_search/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/global_search/normalization.rs` |
| `crates/persistence-postgres/src/adapters/catalog/global_search/query.rs` |
| `crates/persistence-postgres/src/adapters/catalog/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/observations/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/profile/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/profile/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/profile/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/abilities/profile/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/availability/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/availability/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/external_ids/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/external_ids/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/external_ids/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/external_ids/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/names/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/names/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/positions/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/positions/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/read_player.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/directory/create_player.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/directory/input_policy.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/directory/list_mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/directory/list_players.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/directory/list_row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/directory/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/directory/update_player.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/names/add_player_name.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/names/input_policy.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/names/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/names/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/names/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/normalization.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/positions/assign_player_position.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/positions/input_policy.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/positions/list_positions.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/positions/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/positions/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/positions/reference_mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/positions/reference_row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/positions/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/record/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/record/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/record/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/team_periods/add_player_team_period.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/team_periods/input_policy.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/team_periods/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/team_periods/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/team_periods/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/team_periods/season_memberships/list.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/team_periods/season_memberships/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/team_periods/season_memberships/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/team_periods/season_memberships/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/players/value_mapping.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/directory/list.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/directory/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/directory/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/directory/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/entity_type.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/external_ids/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/external_ids/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/external_ids/validation.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/external_ids/write.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/providers/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/providers/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/providers/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/providers/validation.rs` |
| `crates/persistence-postgres/src/adapters/catalog/references/providers/write.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/name_mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/name_row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/player_periods/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/player_periods/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/player_periods/read.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/player_periods/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/profile_mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/profile_row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_names.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_profile.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_recent_matches.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_squad.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_team.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/read_team_record.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/recent_match_mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/recent_match_row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/record_mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/record_row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/squad_mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/detail/squad_row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/directory/create_team.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/directory/list_mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/directory/list_row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/directory/list_team_options.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/directory/list_teams.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/directory/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/directory/option_mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/directory/option_row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/directory/update_team.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/names/add_team_name.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/names/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/names/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/names/normalization.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/names/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/names/validation.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/profiles/input_policy.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/profiles/mapper.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/profiles/mod.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/profiles/row.rs` |
| `crates/persistence-postgres/src/adapters/catalog/teams/profiles/upsert_team_profile.rs` |
| `crates/persistence-postgres/tests/coach_formation_repository_contract.rs` |
| `crates/persistence-postgres/tests/entity_deletion_repository_contract.rs` |
| `crates/persistence-postgres/tests/entity_matching_references_repository_contract.rs` |
| `crates/persistence-postgres/tests/entity_permanent_delete_repository_contract.rs` |
| `crates/persistence-postgres/tests/global_name_search_repository_contract.rs` |
| `crates/persistence-postgres/tests/player_abilities_dynamic_tags_repository_contract.rs` |
| `crates/persistence-postgres/tests/player_directory_detail_repository_contract.rs` |
| `crates/persistence-postgres/tests/player_names_positions_repository_contract.rs` |
| `crates/persistence-postgres/tests/player_team_periods_availability_repository_contract.rs` |
| `crates/persistence-postgres/tests/team_directory_detail_repository_contract.rs` |
| `crates/persistence-postgres/tests/team_force_delete_repository_contract.rs` |
| `crates/persistence-postgres/tests/team_names_profiles_repository_contract.rs` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-01-team-directory-and-detail.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-02-team-names-and-profiles.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-03-player-directory-and-detail.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-04-player-names-and-positions.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-05-team-periods-and-availability.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-06-abilities-and-dynamic-tags.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-07-coaches-and-formation-usage.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-08-entity-matching-and-references.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-09-archive-delete-and-force-delete.md` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/R06-10-global-name-search.md` |
| `scripts/verify-coach-formation-persistence.mjs` |
| `scripts/verify-entity-deletion-persistence.mjs` |
| `scripts/verify-entity-matching-references-persistence.mjs` |
| `scripts/verify-player-abilities-dynamic-tags.mjs` |
| `scripts/verify-player-directory-detail.mjs` |
| `scripts/verify-player-names-positions.mjs` |
| `scripts/verify-player-team-periods-availability.mjs` |
| `scripts/verify-team-directory-detail.mjs` |
| `scripts/verify-team-names-profiles.mjs` |

## 7. 实际修改文件

共 21 项。

| 文件 |
|---|
| `README.md` |
| `architecture/domain-type-inventory.json` |
| `crates/persistence-postgres/src/adapters/mod.rs` |
| `crates/persistence-postgres/src/lib.rs` |
| `crates/persistence-postgres/src/player_catalog.rs` |
| `docs/modular-rewrite/R06-entity-catalog-persistence/README.md` |
| `package.json` |
| `scripts/verify-database-reset.mjs` |
| `scripts/verify-entity-deletion.mjs` |
| `scripts/verify-entity-relationships.mjs` |
| `scripts/verify-entity-resource-center.mjs` |
| `scripts/verify-force-team-delete.mjs` |
| `scripts/verify-formation-usage.mjs` |
| `scripts/verify-global-name-search.mjs` |
| `scripts/verify-match-event-facts.mjs` |
| `scripts/verify-match-review-package.mjs` |
| `scripts/verify-persistence-audit.mjs` |
| `scripts/verify-player-role-inheritance.mjs` |
| `scripts/verify-stage-e1-followup.mjs` |
| `scripts/verify-stage-e2-lineup-presets.mjs` |
| `scripts/verify-team-player-management.mjs` |

## 8. 实际移动或重命名文件

共 3 项。

| 文件 |
|---|
| `crates/persistence-postgres/src/team_force_delete.rs -> crates/persistence-postgres/src/adapters/catalog/deletion/force_delete/targets.rs` |
| `crates/persistence-postgres/src/dynamic_tags.rs -> crates/persistence-postgres/src/adapters/catalog/dynamic_tags/contribution/calculate.rs` |
| `crates/persistence-postgres/src/name_search.rs -> crates/persistence-postgres/src/adapters/catalog/global_search/predicate.rs` |

## 9. 实际删除文件

共 3 项。

| 文件 |
|---|
| `crates/persistence-postgres/src/entity_catalog.rs` |
| `crates/persistence-postgres/src/formation_catalog.rs` |
| `crates/persistence-postgres/src/team_catalog.rs` |

## 10. 阶段出口与下一阶段

R6 正式关闭为 `DONE`。R7 可从 R6 closeout HEAD 建立唯一阶段分支；Match/Lineup persistence 不再计入 R6。
