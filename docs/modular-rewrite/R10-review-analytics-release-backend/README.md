# R10 Review / Postmatch / Analytics / Release 后端：执行记录索引

## 当前阶段状态

`IN_PROGRESS`。用户2026-10-12“开始R10-01”授权；从R9文档收尾HEAD `48618fed2d7ff7c2a4db831a037f227dc789f278` 建立唯一阶段分支 `rewrite/r10-review-analytics-release-backend`。01自身精确Windows run38229119264/job114743919870全SUCCESS并DONE；用户“收尾01开始02”授权02已实施VERIFYING，03～13 BLOCKED。

## 前置基线与依据

[R10任务书](../../football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md)、[总纲执行订正](../../football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md)、[R9完成记录](../R09-research-ai-backend/R09-stage-completion.md)。已验源码b50d8047a6dbf32775d70704afde61d99b897069 / tree7dd421459bbff3156d4c034fc01e9b74dd2985d5 / [Windows run38221009454](https://github.com/uniquenesssta/123/actions/runs/38221009454) / job114716987812 SUCCESS；R9收尾只改文档，不能替代R10自身验收。

## 任务状态

| 任务 | 责任 | 状态 |
|---|---|---|
| R10-01 | [Review Package Workflow](R10-01-review-package-workflow.md) | DONE · 自身精确Windows全SUCCESS |
| R10-02 | [Structured Match Facts](R10-02-structured-match-facts.md) | VERIFYING · 已实施，静态通过，自身Windows待验 |
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


## R10-01 精确 Windows 完成与02授权（2026-10-12）

精确提交 `838e39ea35355662c192769a358c56a06131a6ed` / tree `7f42a75620aa8eb44441670a69a379b0c1a7830b`，[Public Platform CI run38229119264](https://github.com/uniquenesssta/123/actions/runs/38229119264) / Windows job114743919870全SUCCESS，run于2026-10-12 17:44:15北京完成。实际Application146/146（新增8 workflow unit均成功）、Persistence179/179、Gateway112/112 unit和41/41 contract；frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/客户端启动及运行日志扫描全部通过。报告windows-acceptance-20261012-092001.json。

artifact11739056741，14,041,808字节，SHA-256 `2831911b9159911fb03ebc9bf894e03f00a4a01c73001b9ffddc8f9a289a1fa7`。01自身完整成功后标记DONE；用户2026-10-12“收尾01开始02”授权，02 Structured Match Facts READY，同一rewrite/r10-review-analytics-release-backend分支，03～13 BLOCKED，R10 IN_PROGRESS。本次仅五既有文档收尾[skip ci]，源码同上述成功提交；后续02必须取得自身完整Windows证据。

真实PG/历史保护四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4/P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验；ignored/公开unavailable stub/静态指纹不计这些真实项目PASS。此前01 VERIFYING及预期计数保留为实施历史，不替代当前DONE状态。


## R10-02 实际边界与继续位置

R10-02 Structured Match Facts 已实施 `VERIFYING`，基线ab43aee，继续唯一分支rewrite/r10-review-analytics-release-backend。Application真实事实阶段迁入facts coordinator，ReviewService与组合工作流直接复用；PG facts目录独占校验、事件身份、事务写入、查询/摘要、行映射和事实提交，旧review.rs的14职责及事实写入正文移除。原比赛锁、修订归属、SQL/审计/同事务上下文与十个公开方法签名保持，不改变报告/候选/参数人工门禁。新增8 PG unit待自身Windows，预期Persistence187 / Application146 / Gateway112 unit、41 contract；84/84静态、完整架构、19 Rust文件Rustfmt、等价与8破坏探针通过。01自身Windows已DONE，02 VERIFYING、03～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7回归与继承model.runs/0041删除风险仍最终新库待验。

事实事务借用边界与原完整准备上下文保持，原prepare_review_in_tx仍有真实球员/连续性/预测准备职责，留给03 Review Engine Adapter；无提前移除或新增计算。先核对02精确Windows完整结果，失败只修本唯一R10分支；成功且用户授权收尾后才推进03，无阶段完成记录。完整11A/19M/0D、职责迁移、原错误/SQL/风险与回退见 [02记录](R10-02-structured-match-facts.md)。
