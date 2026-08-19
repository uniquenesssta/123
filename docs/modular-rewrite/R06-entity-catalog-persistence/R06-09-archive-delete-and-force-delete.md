# R6-09 Archive / Delete / Force Delete

## 状态

`IN_PROGRESS`

## Atomic Task 1 — Preflight / Reference Counts / Archive

**状态：DONE**

- 基线：`3bc7ca89ef0341b53985246602218c2ba6fbb094`。
- `check_entity_deletion`、team/player/coach reference counts、entity label 与 `bulk_archive_entities` 已从 `entity_catalog.rs` 迁入 `adapters/catalog/deletion/{preflight,archive}/`；`entity_catalog.rs` 仅保留 team-player-period read。
- 公共 Port/DTO/Schema/迁移/错误/归档行为/历史 P4 保护均未改变；普通永久 Delete 与 Force Delete 未在 AT1 修改。
- Minimum Gate：R6-09 AT1 ownership、历史 Entity Deletion / Entity Relationships、R6-07/R6-08 retained ownership、完整 architecture、protected assets、rustfmt、Persistence check/unit tests、R6-09 contract compile、Application check 与 diff hygiene 全部 PASS。
- PostgreSQL 16 真实 contract：run `32219150263` / job `95967398924` 为 `SUCCESS`。
- Windows Stage Regression 首轮 run `32219150263` / job `95967398932` 因历史 `verify-match-review-package.mjs` 仍读取旧 `entity_catalog.rs` fail-fast；第一次 retry run `32220406479` / job `95969707027` 又发现 `verify-stage-e2-lineup-presets.mjs` 仍绑定旧 owner。两处只将原 reference-count 断言跟随到 `adapters/catalog/deletion/preflight/references.rs`，未删除、跳过或放宽门禁。
- 最终 Windows retry run `32220581584` / job `95970175840` 为 `SUCCESS`：AT1 current-head verifier、`npm run verify:frontend`、rustfmt、workspace Clippy `-D warnings`、workspace tests 与 `git diff --check` 全部实际通过。AT1 据此关闭为 `DONE`。
- 新增：`adapters/catalog/deletion/{mod.rs,ids.rs,preflight/*,archive/*}`、`entity_deletion_repository_contract.rs`、`verify-entity-deletion-persistence.mjs`、本记录。
- 修改：`entity_catalog.rs`、catalog `mod.rs`、相关 retained verifier、`package.json`、Domain inventory、根/阶段 README。
- 删除/移动：AT1 无生产文件删除；临时 retry workflow、failure/run marker 已由 cleanup commit `0fa05a3b67eb21009b5a56e00cb5b365fd0e66b5` 清理。
- 回退点：上述基线提交。

## Atomic Task 2 — Safe Permanent Delete

**状态：DONE**

- `bulk_delete_teams`、`bulk_delete_players` 与 `delete_player` 已迁入 `adapters/catalog/deletion/{bulk_delete,safe_delete,delete_write}.rs`；bulk/safe-delete 协调器 SQL-free，事务、行锁、最终竞态复检、external-ID 清理与审计写入由 `delete_write.rs` 唯一持有。
- 旧 `team_catalog.rs` 已删除；`player_catalog.rs` 不再持有 `delete_player`。Team Force Delete 继续保留 `team_force_delete.rs`，未提前进入 AT3。
- 公共 Port/DTO/Schema/迁移、无引用永久删除条件、错误语义、历史引用保护与用户可观察行为保持不变；无新增生产依赖。
- AT2 生产提交 `eca85a92d30cdf83e6b9f3f1b341874a0e9fba01`：Minimum Gate、AT1/AT2 focused PostgreSQL 16 contracts、完整 Windows Stage Regression 与 R6-01～R6-09 retained PostgreSQL 16 contracts全部通过。AT2 已独立关闭；AT3 Team Force Delete 为本节点剩余职责。
