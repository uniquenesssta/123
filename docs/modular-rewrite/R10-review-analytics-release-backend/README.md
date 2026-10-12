# R10 Review / Postmatch / Analytics / Release 后端：执行记录索引

## 当前阶段状态

`IN_PROGRESS`。用户2026-10-12“开始R10-01”授权；从R9文档收尾HEAD `48618fed2d7ff7c2a4db831a037f227dc789f278` 建立唯一阶段分支 `rewrite/r10-review-analytics-release-backend`。01已实施 `VERIFYING`，必须自身精确Windows完整SUCCESS后才能DONE；02～13 BLOCKED。

## 前置基线与依据

[R10任务书](../../football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md)、[总纲执行订正](../../football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md)、[R9完成记录](../R09-research-ai-backend/R09-stage-completion.md)。已验源码b50d8047a6dbf32775d70704afde61d99b897069 / tree7dd421459bbff3156d4c034fc01e9b74dd2985d5 / [Windows run38221009454](https://github.com/uniquenesssta/123/actions/runs/38221009454) / job114716987812 SUCCESS；R9收尾只改文档，不能替代R10自身验收。

## 任务状态

| 任务 | 责任 | 状态 |
|---|---|---|
| R10-01 | [Review Package Workflow](R10-01-review-package-workflow.md) | VERIFYING · 已实施，静态通过，自身Windows待验 |
| R10-02 | Structured Match Facts | BLOCKED |
| R10-03 | Review Engine Adapter | BLOCKED |
| R10-04 | Ability Candidate Transaction | BLOCKED |
| R10-05 | Postmatch Settlement | BLOCKED |
| R10-06 | Evidence Scoring | BLOCKED |
| R10-07 | Provider Score | BLOCKED |
| R10-08 | Drift Monitoring | BLOCKED |
| R10-09 | Analytics Calculation | BLOCKED |
| R10-10 | Background Jobs | BLOCKED |
| R10-11 | Analysis Package | BLOCKED |
| R10-12 | Parameter Lifecycle | BLOCKED |
| R10-13 | Release Acceptance | BLOCKED |

## R10-01 实际边界与验证

R10-01 Review Package Workflow 已实施、`VERIFYING`；唯一阶段分支 `rewrite/r10-review-analytics-release-backend`，从 R9 收尾 `48618fed2d7ff7c2a4db831a037f227dc789f278` 建立。Application workflow 独立持有编排、身份门禁与人工确认元数据，ReviewService 五入口已切换；PG 十个工作流方法迁入对应适配器，SQL、审计及事务正文逐字保持，旧两 owner 删除。原 Domain 八状态/九步界面、171 命令、Ports/DTO/错误/模型保护区保持；不自动接受候选或晋升参数。新增 8 项原 Application unit 待自身 Windows，预期 Application146 / Persistence179 / Gateway112 unit、41 contract。84/84 既有源码门禁、完整架构、Rustfmt、等价复核与 8 破坏探针通过；CI 开始后停止查询。R10 IN_PROGRESS，02～13 BLOCKED，真实 PG/历史四项/账本并发回滚/有效 XLSX/Windows Full/私有 P4/P7 固定回归及继承 model.runs/0041 删除风险仍最终封包新库待验。

采用现有 package/workflow 子目录按职责递归拆分，而非为参考路径机械迁移整个资料包。export/preview/workbook仍承担其原职责；workflow独占五个生命周期编排、三项纯身份门禁和确认元数据，PG适配器独占工作流持久化。Domain状态权限继续唯一权威，事实提交原阵容→事实→状态三个Port操作保持各自原事务，未声称改成全包单事务。SQL条件保留幂等结算和单活动包约束。

无新runner/workflow/target/数据库专项入口、依赖或迁移；无R10阶段完成记录。ignored/公共unavailable stub/静态指纹不能记为真实新库项目PASS。
