# R10 Review / Postmatch / Analytics / Release 后端：执行记录索引

## 当前阶段状态

`READY`（仅前置文档）。R9全部01～11与Windows Automated阶段出口已DONE，用户当前“收尾11”，未授权开始R10实现。当前工作分支仍rewrite/r9-research-ai-backend；未创建第二分支。收到开始R10-01指令后，从R9纯文档收尾HEAD建立本阶段唯一分支。

## 前置基线与依据

[R10任务书](../../football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md)、[总纲执行订正](../../football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md)、[R9完成记录](../R09-research-ai-backend/R09-stage-completion.md)。已验源码b50d8047a6dbf32775d70704afde61d99b897069 / tree7dd421459bbff3156d4c034fc01e9b74dd2985d5 / [Windows run38221009454](https://github.com/uniquenesssta/123/actions/runs/38221009454) / job114716987812全SUCCESS；本轮收尾文档复用其源码，不替代R10自身验收。

## 任务状态

| 任务 | 责任 | 状态 |
|---|---|---|
| R10-01 | Review Package Workflow | READY · 未实施 |
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

## R10-01 继续位置

目标为Review Package Workflow，任务书参考目录use_cases/review/package-workflow；现有实际owner在Application services/review/service.rs及use_cases/review/package/{export,preview,lifecycle,shared,workbook}，原Review Ports/composition与PG调用继续当前实现。开始时须完整核对实际九步状态机/包身份/事实写入/候选人工接受/原事务/全部调用方，再按职责决定目录与切换，不能仅为模板路径机械改名。

未创建R10-01实施记录、测试或阶段完成记录，不写入尚未发生的A/M/D/验证。保持纯计算公式/模型资产、171命令/DTO/config/error/log/UI以及历史与幂等语义；不得自动接受候选或晋升参数，不使合成fixture混入真实统计。

仅Windows动态；沿用原Rust unit/contract/Windows runner/workflow/数据库入口，不新增持续回归体系。真实PG/历史保护四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored/公开stub/静态指纹不是这些真实项目PASS。
