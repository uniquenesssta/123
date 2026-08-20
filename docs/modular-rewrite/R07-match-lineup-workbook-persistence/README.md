# R07 Match Lineup Workbook Persistence：执行记录索引

## 阶段状态

`IN_PROGRESS`

R7 重写 Matches、Lineups、Presets 与 Workbook 持久化，重点保证双方阵容原子事务、截止时间、历史链路、批次账本、真实 XLSX 行/子记录 identity 与整批回滚；不实施工作簿解析算法本体或前端导入 UI。

## 前置基线

- R6 已正式 `DONE`；阶段记录：[`../R06-entity-catalog-persistence/R06-stage-completion.md`](../R06-entity-catalog-persistence/R06-stage-completion.md)。
- R7 唯一阶段分支：`rewrite/r7-match-lineup-workbook-persistence`。
- R7 分支精确起点：`a25da2bdd2b93d8146e46af60947a687e998f987`。
- R6 最终出口 run `32276040092`：Windows job `96143592950`、PostgreSQL 16 job `96143592734` 均 `SUCCESS`。
- 公共 Lineup/Match/Workbook Port、Domain DTO、0001–0046 migrations、模型保护资产和 UI 默认保持兼容；本阶段不新增生产依赖。

## 当前源码基线

R7 开始时相关 PostgreSQL 职责仍主要分布于：

- `crates/persistence-postgres/src/player_catalog.rs`：Match Catalog 与 Lineup 写读混合；R7-01 首先迁出 Match Catalog，Lineup 留给后续 R7 节点。
- `crates/persistence-postgres/src/lineup_chain.rs`：阵容时间窗口、validation、chain/history。
- `crates/persistence-postgres/src/team_lineup_presets.rs`：球队阵容预设。
- `crates/persistence-postgres/src/spreadsheet_exchange.rs`：Player/Team spreadsheet preview、batch ledger 与 commit 链路。
- `crates/persistence-postgres/src/monthly_workbooks.rs`：monthly team/player workbook persistence。
- `crates/persistence-postgres/src/match_exchange.rs`：match-lineup workbook/export/import persistence。

目标 owner 统一收敛到：

```text
crates/persistence-postgres/src/adapters/matches/
crates/persistence-postgres/src/adapters/lineups/
crates/persistence-postgres/src/adapters/workbooks/
```

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R7-01 | Match Catalog | VERIFYING |
| R7-02 | Lineup Pair Transaction | BLOCKED |
| R7-03 | Lineup Chain / History | BLOCKED |
| R7-04 | Team Lineup Presets | BLOCKED |
| R7-05 | Spreadsheet Batch Ledger | BLOCKED |
| R7-06 | Player Workbook | BLOCKED |
| R7-07 | Team Package | BLOCKED |
| R7-08 | Monthly Workbook | BLOCKED |
| R7-09 | Match Lineup Workbook | BLOCKED |
| R7-10 | Row / Subrecord Identity | BLOCKED |

## R7-01 READY 边界

- 只处理 Match Catalog 的创建、删除、列表、单场读取及其直接 validation/mapping/query owner。
- 目标目录：`crates/persistence-postgres/src/adapters/matches/catalog/`。
- 不提前迁移 `create_lineup`、`create_lineup_pair`、lineup chain/history、preset 或 workbook 事务。
- 新实现通过最小验证后切换唯一入口，并删除被 R7-01 替代的旧 Match Catalog 职责；不得保留转发壳或双实现。
- 必须保持比赛 external key、competition/season/stage/round scope、主客队校验、列表排序/limit、删除保护与审计语义不变。

## 阶段硬约束

- 双方阵容不得逐侧提交；R7-02 必须由单一 pair transaction 拥有双方写入原子性。
- actual 阵容不得进入赛前快照；截止时间与历史时点语义保持。
- Workbook preview 与 commit 分离；冲突未解决禁止提交；整批失败必须回滚。
- 真实 XLSX 与 PostgreSQL 链路必须验证，不得用 mock 替代要求的真实链路。
- 每个 Atomic Task 先通过最小验证，再运行阶段回归；失败即停，不进入下一节点。

## 记录规则

- R7-01 完成时创建 [`R07-01-match-catalog.md`](R07-01-match-catalog.md)，并把 R7-02 切到 `READY`。
- 后续节点按任务书依次创建对应记录；未完成的记录不提前创建为伪完成文件。
- 全部 R7-01～R7-10 完成且最终出口门禁通过后才创建 `R07-stage-completion.md`。

## R7-01 当前验证状态

- Match Catalog owner 已切换到 `adapters/matches/catalog/`；节点保持 `VERIFYING`，等待最小门禁、真实 PostgreSQL contract 与阶段回归。
