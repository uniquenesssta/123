# R09 Research Gateway / AI Workspace 后端：执行记录索引

## 当前阶段状态

`IN_PROGRESS`。用户已明确“开始R9-01”，节点映射已厘清。唯一阶段分支 `rewrite/r9-research-ai-backend`，起点 `c72e559af4f29c9510daf2f9bf66b926dacb3013`。01 Shared Transport已实施、VERIFYING，须自身精确Windows CI；02～11 BLOCKED，不创建阶段完成记录。

## 前置基线与依据

见 [R9任务书](../../football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md)、[总纲顶部执行订正](../../football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md)、[R8阶段完成记录](../R08-prediction-p4-orchestration/R08-stage-completion.md)。R8最终代码 `af3c98c31e28a332fe19ec47f4ed11c3a5a261ab` / [Windows run `37899786755`](https://github.com/uniquenesssta/123/actions/runs/37899786755) 全SUCCESS，Application126/Persistence175、17视口及Windows交付通过；R8文档收尾与此源码树一致，不能视为R9实现或其验收。

R8期间research-gateway、公开契约、配置/依赖及锁文件保持；原Gateway单测与contract在最终Windows通过，后续以原协议行为为冻结基线。真实 PostgreSQL、历史四项数据库验收、账本/并发/回滚、有效 XLSX、Windows Full、私有 P4/P7 Golden Master，以及继承的 model.runs/0041 历史删除风险继续最终封包新库待验；ignored、公开 unavailable stub 和静态保护指纹均不计真实执行 PASS。

## 任务状态

| 任务 | 责任 | 状态 |
|---|---|---|
| R9-01 | Shared Transport | VERIFYING · [实施记录](R09-01-shared-transport.md) · 待自身精确Windows |
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

原client的三个公共传输类型与HTTP职责已切换transport唯一owner；Formal/Plain/Structured/连接测试/恢复GET/取消空POST继续同一原接口，不升级协议、不修改AI Workspace前端或凭据/重试政策。详见01记录；下一步核实01精确Windows，成功才收尾并开放02。

沿用现有单测/contract、Windows runner/workflow与最终新库入口；不新增持续回归基础设施，不执行Linux/macOS动态。每项必须自身精确Windows验收及实施记录才能DONE。已创建01实施记录；其状态仍VERIFYING，不创建R9阶段完成记录。Create State仅记录足球模型项目0.23.0的交接状态。


## R9-01 实施与门禁（2026-10-09，VERIFYING）

从c72e559已验源码文档基线建立唯一R9分支，原client将TransportResponse/OpenAiTransport迁入transport/contract；原ReqwestTransport/headers/send/三种请求/rustls Once/error转换由http唯一持有；纯JSON/空响应/解析错误由response持有，mod只登记/re-export。crate根仍导出原三个名字，Application/Tauri/原mock调用方不需变化。原GatewayAttemptSink、协议/重试/并发/取消/研究与会话职责保持原owner，无额外全局状态或转发层。

原Gateway单测target新增8项：原三HTTP动作在有界127.0.0.1夹具核对方法/URL/头/body/429原样返回；禁止跟随307；响应体等待仍超时及原恢复；非法URL原Network/无provider metadata；认证头及非法字符拒绝；任意JSON/status透传；空body Null/原metadata；whitespace/截断/非法UTF8/零字符原SchemaValidation错误。仅WindowsCI实际运行loopback/单测，不调用真实API/不新增target、runner、依赖或协议字段；源码预期Gateway23/contract16，Application126/Persistence175保持。

83/83现有静态、完整architecture、18保护资产/171命令/46迁移/18PG静态、Rustfmt1.88源码check及diff通过；6破坏探针全部拒绝恢复。8原HTTP函数逐token保持，decode重内联后原execute保持、剩余client全体token与265生产literal保持。清单只登记OpenAiTransport实际owner和Domain扫描1099→1103/使用摘要，43Ports/365Domain/300映射/声明摘要保持。Context7只给latest示例，另核对官方reqwest0.13.4准确版本timeout范围；不升级API/依赖。Mermaid已更新原入口与策略/共享IO边界，取消不是transport新增state，连接测试不强加执行重试。

本项须自身Windows fmt/Clippy/workspace/前端/17视口/release/MSI/NSIS/启动；不继承R8 PASS或源码预期，CI开始后停止轮询。详细A/M/D、等价/测试/报告/订正与回退见01记录。真实PG、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共stub不计PASS。
