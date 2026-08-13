# R05 Competition / Routing Persistence：执行记录索引

## 阶段状态

`READY`

R4 Persistence 基础设施已完成并通过阶段出口。R5 仅按 Competition、Season、Stage、Round、Rule Package、Binding、Route Resolution 与 Model Run Identity 拆分 PostgreSQL Adapter；不修改模型路由算法、前端赛事页面或历史 migration。

## 前置基线

- R4 `DONE`；完成记录：[`../R04-persistence-foundation/R04-stage-completion.md`](../R04-persistence-foundation/R04-stage-completion.md)。
- R5-01 开始时必须从 `rewrite/r4-persistence-foundation` 最终 closeout HEAD 独立建分支；该 HEAD 的最终 Public Platform CI 必须先为 `SUCCESS`。
- R3 Competition / Rules Ports 已冻结；0001–0046 migration 继续冻结。

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R5-01 | Competitions Repository | READY |
| R5-02 | Seasons / Stages / Rounds | BLOCKED |
| R5-03 | Rule Packages | BLOCKED |
| R5-04 | Competition Bindings | BLOCKED |
| R5-05 | Route Resolution Reads | BLOCKED |
| R5-06 | Model Run Identity Reads | BLOCKED |

## R5-01 进入条件

- 先读取 R5 任务书、R0 inventory 与当前 competition/routing/rule 持久化调用链。
- 先确认当前 R4 closeout HEAD 的 canonical Public Platform CI 已完成且为 `SUCCESS`，再创建 R5-01 独立实施分支。
- 先迁移/补齐 Competitions Repository 契约测试，再切换唯一 owner；不得提前实施 R5-02。

## 当前事实

- 本索引只开放 R5-01，没有创建或修改任何 R5 生产源码、SQL、migration、Port、DTO 或模型资产。
- R4 broad diagnostic 暴露的既有 P4 timestamp 精度问题不属于 R5-01，不得夹带修复。
