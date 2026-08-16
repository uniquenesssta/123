# R06 Entity Catalog Persistence：执行记录索引

## 阶段状态

`IN_PROGRESS`

R6 按 Teams、Players、Coaches、Formations、Abilities、Dynamic Tags、Availability、Entity Matching、References、Deletion 与 Global Search 拆分 PostgreSQL Entity Catalog Adapter；不实施阵容提交、工作簿导入事务或 UI。

## 前置基线

- R5 阶段 R5-01～R5-06 已完成；阶段记录：[`../R05-competition-routing-persistence/R05-stage-completion.md`](../R05-competition-routing-persistence/R05-stage-completion.md)。
- R5 最终代码 merge commit：`acb0491003b365b3f775780d8c98ecfdf1e80104`；merged-stage canonical run `31884882480` / Windows job `95012628426` 为 `SUCCESS`。
- R5 最终 closeout HEAD `7512ee805fcba8cac3c8f334680f200d625808c0` 的 canonical Public Platform CI run `31886299959` / Windows job `95016342335` 为 `SUCCESS`；artifact `9247632165`，SHA-256 `3726104665e7102b57effb4ac5c58a5c9dfe7bde20218f7f672b8ef2ededf5c4`。R6 stage 与 R6-01 实施分支均从该 HEAD 精确建立。
- Team / Player Ports 与 Domain 公共对象保持冻结；0001–0046 migration 与模型保护资产继续冻结。

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R6-01 | Team Directory 与 Detail | DONE |
| R6-02 | Team Names 与 Profiles | DONE |
| R6-03 | Player Directory 与 Detail | VERIFYING |
| R6-04 | Player Names 与 Positions | BLOCKED |
| R6-05 | Team Periods 与 Availability | BLOCKED |
| R6-06 | Abilities 与 Dynamic Tags | BLOCKED |
| R6-07 | Coaches 与 Formation Usage | BLOCKED |
| R6-08 | Entity Matching 与 References | BLOCKED |
| R6-09 | Archive / Delete / Force Delete | BLOCKED |
| R6-10 | Global Name Search | BLOCKED |

## R6-01 当前事实

- 详细记录：[`R06-01-team-directory-and-detail.md`](R06-01-team-directory-and-detail.md)。
- Team `create/update/list/options/detail` persistence 已从旧 `player_catalog.rs` / `team_catalog.rs` 收敛到 `adapters/catalog/teams/{directory,detail}/`；typed Row、Domain Mapper、SQL reader 与 detail coordinator 分责。
- R6-02 的 Team Names/Profile writes 与 R6-09 deletion 保留原 owner，没有提前跨节点迁移。
- 旧 owner PostgreSQL 16 contract run `31888078703` / job `95020173411` 为 `SUCCESS`。
- 前两次 owner-switch run `31888247096` / `95020568846`、`31888389636` / `95020899291` 因 Rust module visibility/import 不完整 fail-fast，均未产生生产 owner 切换提交；最终 run `31888483586` / job `95021118552` 为 `SUCCESS`，生产切换提交为 `5056917bd5c0e81aaaa96d1ef70c7362ef874820`。
- 第一次 ownership/inventory run `31888670048` / job `95021554565` 在 R6-01 verifier、Domain inventory、模型保护均通过后，由旧 R4-02 Audit verifier 的 legacy create-team call-path 停止；仅更新 verifier owner 路径并保持同 transaction audit 原子性检查。
- 第二次 ownership/inventory run `31888800801` / job `95021851053` 为 `SUCCESS`：official inventory、R6-01 ownership、模型保护、完整 architecture、rustfmt、Persistence/Application check、Persistence unit tests 与同一 PostgreSQL contract 全部通过；verified HEAD `b2999e5dce982bddf2093f0596eec33a7dbd0333`。
- 临时 baseline/owner-switch/inventory workflow 与 helper 已从当前 branch tree 清理。
- 阶段级 hard gate 最终 run `31889555412`：Windows job `95023694944` 与 PostgreSQL/architecture job `95023694946` 均 `SUCCESS`；frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests、完整 architecture、模型保护、database baseline freeze 与同一 PG16 contract 已全部实际通过。
- hard gate 前五轮均因旧 verifier 仍绑定已迁移的 Team owner 路径而 fail-fast；仅推进 verifier call-path 到新 owner，原契约均保持。
- PR #32 clean head `58ec21b48b26ead4f2ee2fec15573c39a2f7ab8b` 的 canonical run `31890099520` 为 `SUCCESS`；squash merge commit `a54df5ca2695297ea5866a3efd74643239568825`。
- merged-stage canonical run `31892105125` / Windows job `95029805339` 为 `SUCCESS`；artifact `9249127034`，SHA-256 `972be4e32272e700da265f6e82ce96bc5e56aa356491bb217d6a1076e1b00087`。R6-01 正式 `DONE`，R6-02 开放为 `READY`。

## R6-02 当前事实

- 详细记录：[`R06-02-team-names-and-profiles.md`](R06-02-team-names-and-profiles.md)。
- Team Names/Profile 写职责已收敛到 `adapters/catalog/teams/{names,profiles}/`；validation、normalization/input policy、typed Row、Domain Mapper、SQL/transaction owner 分责。
- R6-01 create/update 复用 `names/normalization.rs` 唯一名称规范化 owner；旧 `directory/name_policy.rs` 已删除。R6-09 deletion 仍保留原 owner，没有提前跨节点迁移。
- 旧 owner PostgreSQL 16 baseline run `31897078340` / job `95041920033` 为 `SUCCESS`；owner switch 最终 run `31897403363` / job `95042724731` 为 `SUCCESS`。
- 第一次 owner-switch run `31897301166` 仅因 rustfmt 精确差异 fail-fast；按 rustfmt 输出修复后全链通过，没有降低门禁。
- hard gate 首轮发现 official Domain inventory 漂移；refresh run `31897577673` / job `95043169681` 仅更新官方 inventory 并 `SUCCESS`。
- 第二轮 hard gate 的 Windows job 发现 entity-relationship verifier 仍绑定旧 Profile owner；只推进投影保护检查源到 R6-02 新 owner，原契约未弱化。
- 最终 hard gate run `31897727309`：Windows job `95043538718` 与 PostgreSQL/architecture job `95043538723` 均 `SUCCESS`；frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests、完整 architecture、模型保护、database baseline 与 PG16 contract 均已实际通过。
- clean PR canonical run `31898334326` / Windows job `95044999641` 为 `SUCCESS`；artifact `9250723192`，SHA-256 `15362b7164cc2ae0c7b177f4f4899b56678232595caa955d9602adea1eae2ee6`。
- PR #33 fixed head `f04923425f9d660f2ce1275da2b689ca681f3500` 已 squash merge 为 `334d5d86f08f5cd1adee5b23dc64d907aeb2eba2`。
- merged-stage canonical run `31925219003` / Windows job `95111615032` 为 `SUCCESS`；artifact `9257917686`，SHA-256 `ff29c7326ed23f3736fd433f2adde3aad587f9aa17506b2eedd96e913b7f2e9d`。R6-02 正式 `DONE`，R6-03 开放为 `READY`。

## R6-03 当前事实

- 详细记录：[`R06-03-player-directory-and-detail.md`](R06-03-player-directory-and-detail.md)。
- 实施分支 `agent/r6-03-player-directory-detail` 从 R6-02 最终 closeout HEAD `4ce9eb405e897beedf8b122e6794d7c9141b660e` 精确建立；该 HEAD 的 Public Platform CI run `31926475831` / Windows job `95114758145` 为 `SUCCESS`，artifact `9258046111`，SHA-256 `76ae850a02080163086f0ec504e3a7c3cfed87ba7063e378b2c168298772cf7f`。
- Player `create/update/list/detail` persistence 已从旧 `player_catalog.rs` 收敛到 `adapters/catalog/players/{directory,detail,record}/`；Directory、Detail coordinator、单用途读取模块、typed Row 与 Domain Mapper 分责。
- Dynamic Tag 保持既有独立 owner `dynamic_tags.rs`；R6-04 Names/Positions writes、R6-05 Team Periods/Availability writes、R6-06 Abilities/Dynamic Tags writes 与 R6-09 deletion 未提前迁移。
- 旧 owner PostgreSQL 16 contract run `31929515802` 为 `SUCCESS`；Player Directory owner-switch run `31929916934` 为 `SUCCESS`；Player Detail owner-switch run `31930614713` 为 `SUCCESS`。
- official Domain inventory refresh run `31932154320` 为 `SUCCESS`，只更新 `architecture/domain-type-inventory.json`；临时 refresh workflow 已清理。
- hard gate 多轮 fail-fast 均来自 legacy verifier 仍指向已迁移 Player read/list/mapper owner：Global Name Search、Player Role Inheritance、Database Reset、Entity Resource Center、Stage E1 Follow-up。每次只把原断言跟随到真实新 owner，未删除、跳过或放宽契约。
- 最终 stage hard gate run `31932648476`：Windows job `95129707185` 与 PostgreSQL/architecture job `95129707240` 均 `SUCCESS`；`npm run verify:frontend`、rustfmt、workspace Clippy `-D warnings`、workspace tests、R6-01/R6-02/R6-03 ownership、完整 architecture、模型保护、database baseline 与 PostgreSQL 16 contract 全部实际通过。
- 临时 R6-03 hard-gate workflow 与 inventory refresh workflow 已从实施分支清理。
- 当前状态保持 `VERIFYING`：clean PR canonical、squash merge 与 merged-stage canonical 尚未完成；在这些门禁完成前 R6-04 继续 `BLOCKED`。
- 用户现有 PostgreSQL 数据库真实数据写入/sample 验收与 Windows Full 人工交互验收未执行，不宣称通过。

## 当前边界与剩余门禁

- 公共 TeamCatalogPort / PlayerCatalogPort、Tauri command/DTO、Schema、0001–0046 migration、配置、错误语义、用户可观察行为、模型保护资产与生产依赖保持冻结。
- R6-01 与 R6-02 均已 `DONE`。
- R6-03 Player Directory 与 Detail 为当前唯一 `VERIFYING` 节点；R6-04～R6-10 继续 `BLOCKED`。
- R6-03 下一步是 clean PR canonical、squash merge 与 merged-stage canonical；全部完成后才能标记 `DONE` 并开放 R6-04。
- 每个节点完成时必须创建对应 `R06-xx` 实施记录并更新本索引；R6 完成时必须创建 `R06-stage-completion.md`。
