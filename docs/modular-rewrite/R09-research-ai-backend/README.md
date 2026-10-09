# R09 Research Gateway / AI Workspace 后端：执行记录索引

## 当前阶段状态

`READY`（仅前置交接，尚未实施）。唯一可进入的节点 R9-01 Shared Transport；R9-02～11 BLOCKED。尚未创建 R9 工作分支或修改 R9 代码。用户“收尾12开始13”中 R8-12 及 R8 阶段已收尾；R8 任务书仅有12项，“13”对应节点待厘清。

## 前置基线与依据

见 [R9任务书](../../football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md)、[总纲顶部执行订正](../../football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md)、[R8阶段完成记录](../R08-prediction-p4-orchestration/R08-stage-completion.md)。R8最终代码 `af3c98c31e28a332fe19ec47f4ed11c3a5a261ab` / [Windows run `37899786755`](https://github.com/uniquenesssta/123/actions/runs/37899786755) 全SUCCESS，Application126/Persistence175、17视口及Windows交付通过；R8文档收尾与此源码树一致，不能视为R9实现或其验收。

R8期间research-gateway、公开契约、配置/依赖及锁文件保持；原Gateway单测与contract在最终Windows通过，后续以原协议行为为冻结基线。真实 PostgreSQL、历史四项数据库验收、账本/并发/回滚、有效 XLSX、Windows Full、私有 P4/P7 Golden Master，以及继承的 model.runs/0041 历史删除风险继续最终封包新库待验；ignored、公开 unavailable stub 和静态保护指纹均不计真实执行 PASS。

## 任务状态

| 任务 | 责任 | 状态 |
|---|---|---|
| R9-01 | Shared Transport | READY · 仅前置交接，尚未实施 |
| R9-02 | Credentials 与 Redaction | BLOCKED |
| R9-03 | Retry、Circuit Breaker 与 Cancel | BLOCKED |
| R9-04 | Formal Research Request | BLOCKED |
| R9-05 | Formal Response、Schema 与 Citation | BLOCKED |
| R9-06 | Source 与 Time Policy | BLOCKED |
| R9-07 | Plain Chat Responses | BLOCKED |
| R9-08 | Plain Chat Chat Completions | BLOCKED |
| R9-09 | Session Persistence | BLOCKED |
| R9-10 | Cancellation Registry | BLOCKED |
| R9-11 | Runtime Diagnostics | BLOCKED |

## 实施范围与继续位置

厘清节点后再按完整任务书扫描当前Shared Transport真实职责/入口，建立唯一R9分支及已验起点，实施最小完整职责拆分。目录模板应服从真实职责；不以空转发替代拆分，不提前改凭据/重试/其他节点，也不升级外部协议或修改AI Workspace前端。

沿用现有单测/contract、Windows runner/workflow与最终新库入口；不新增持续回归基础设施，不执行Linux/macOS动态。每项必须自身精确Windows验收及实施记录才能DONE。此目录只有前置索引，尚无节点完成记录，不创建R9阶段完成记录。Create State仅记录足球模型项目0.23.0的交接状态。
