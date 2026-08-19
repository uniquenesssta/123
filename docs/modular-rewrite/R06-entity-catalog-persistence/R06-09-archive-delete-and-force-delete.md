# R6-09 Archive / Delete / Force Delete

## 状态

`IN_PROGRESS`

## Atomic Task 1 — Preflight / Reference Counts / Archive

- 基线：`3bc7ca89ef0341b53985246602218c2ba6fbb094`。
- `check_entity_deletion`、team/player/coach reference counts、entity label 与 `bulk_archive_entities` 已从 `entity_catalog.rs` 迁入 `adapters/catalog/deletion/{preflight,archive}/`；`entity_catalog.rs` 仅保留 team-player-period read。
- 公共 Port/DTO/Schema/迁移/错误/归档行为/历史 P4 保护均未改变；普通永久 Delete 与 Force Delete 未在 AT1 修改。
- Minimum Gate：R6-09 AT1 ownership、历史 Entity Deletion / Entity Relationships、R6-07/R6-08 retained ownership、完整 architecture、protected assets、rustfmt、Persistence check/unit tests、R6-09 contract compile、Application check 与 diff hygiene 全部 PASS。
- Stage Regression 与 PostgreSQL 16 真实 contract 由本 workflow 后续独立 jobs 执行；在成功前 AT1 不关闭。
- 新增：`adapters/catalog/deletion/{mod.rs,ids.rs,preflight/*,archive/*}`、`entity_deletion_repository_contract.rs`、`verify-entity-deletion-persistence.mjs`、本记录。
- 修改：`entity_catalog.rs`、catalog `mod.rs`、相关 retained verifier、`package.json`、Domain inventory、根/阶段 README。
- 删除/移动：AT1 无生产文件删除；临时 workflow/helper 在提交时清理。
- 回退点：上述基线提交。
