# R06 Entity Catalog Persistence：执行记录索引

## 阶段状态

`IN_PROGRESS`

R6 按 Teams、Players、Coaches、Formations、Abilities、Dynamic Tags、Availability、Entity Matching、References、Deletion 与 Global Search 拆分 PostgreSQL Entity Catalog Adapter；不实施阵容提交、工作簿导入事务或 UI。

## 前置基线

- R5 阶段 R5-01～R5-06 已完成；阶段记录：[`../R05-competition-routing-persistence/R05-stage-completion.md`](../R05-competition-routing-persistence/R05-stage-completion.md)。
- R5 最终代码 merge commit：`acb0491003b365b3f775780d8c98ecfdf1e80104`；merged-stage canonical run `31884882480` / Windows job `95012628426` 为 `SUCCESS`。
- 本 README 与 R5 closeout 文档位于同一 closeout 提交；该提交的 canonical Public Platform CI 成功后，才作为 R6-01 源码实施基线。
- Team / Player Ports 与 Domain 公共对象保持冻结；0001–0046 migration 与模型保护资产继续冻结。

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R6-01 | Team Directory 与 Detail | READY |
| R6-02 | Team Names 与 Profiles | BLOCKED |
| R6-03 | Player Directory 与 Detail | BLOCKED |
| R6-04 | Player Names 与 Positions | BLOCKED |
| R6-05 | Team Periods 与 Availability | BLOCKED |
| R6-06 | Abilities 与 Dynamic Tags | BLOCKED |
| R6-07 | Coaches 与 Formation Usage | BLOCKED |
| R6-08 | Entity Matching 与 References | BLOCKED |
| R6-09 | Archive / Delete / Force Delete | BLOCKED |
| R6-10 | Global Name Search | BLOCKED |

## 当前边界

- R6 尚未修改任何生产源码、Schema、migration、配置、依赖或用户数据。
- R6-01 的目标目录为 `crates/persistence-postgres/src/adapters/catalog/teams/directory/` 与 `detail/`；在 closeout canonical baseline 成功前不得开始源码实施。
- 每个节点完成时必须创建对应 `R06-xx` 实施记录并更新本索引；R6 完成时必须创建 `R06-stage-completion.md`。
