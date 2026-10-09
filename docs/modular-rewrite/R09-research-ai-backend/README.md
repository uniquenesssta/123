# R09 Research Gateway / AI Workspace 后端：执行记录索引

## 当前阶段状态

`IN_PROGRESS`。用户已明确“开始R9-01”，节点映射已厘清。唯一阶段分支 `rewrite/r9-research-ai-backend`，起点 `c72e559af4f29c9510daf2f9bf66b926dacb3013`。01 Shared Transport自身Windows全SUCCESS、DONE；用户“收尾01开始02”授权02，已自身Windows全SUCCESS、DONE；用户“收尾02开始03”授权03 READY，04～11 BLOCKED，不创建阶段完成记录。

## 前置基线与依据

见 [R9任务书](../../football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md)、[总纲顶部执行订正](../../football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md)、[R8阶段完成记录](../R08-prediction-p4-orchestration/R08-stage-completion.md)。R8最终代码 `af3c98c31e28a332fe19ec47f4ed11c3a5a261ab` / [Windows run `37899786755`](https://github.com/uniquenesssta/123/actions/runs/37899786755) 全SUCCESS，Application126/Persistence175、17视口及Windows交付通过；R8文档收尾与此源码树一致，不能视为R9实现或其验收。

R8期间research-gateway、公开契约、配置/依赖及锁文件保持；原Gateway单测与contract在最终Windows通过，后续以原协议行为为冻结基线。真实 PostgreSQL、历史四项数据库验收、账本/并发/回滚、有效 XLSX、Windows Full、私有 P4/P7 Golden Master，以及继承的 model.runs/0041 历史删除风险继续最终封包新库待验；ignored、公开 unavailable stub 和静态保护指纹均不计真实执行 PASS。

## 任务状态

| 任务 | 责任 | 状态 |
|---|---|---|
| R9-01 | Shared Transport | DONE · [完成记录](R09-01-shared-transport.md) · run37933341557 SUCCESS |
| R9-02 | Credentials 与 Redaction | DONE · [完成记录](R09-02-credentials-and-redaction.md) · run37954536795 SUCCESS |
| R9-03 | Retry、Circuit Breaker 与 Cancel | READY · 用户已授权 |
| R9-04 | Formal Research Request | BLOCKED |
| R9-05 | Formal Response、Schema 与 Citation | BLOCKED |
| R9-06 | Source 与 Time Policy | BLOCKED |
| R9-07 | Plain Chat Responses | BLOCKED |
| R9-08 | Plain Chat Chat Completions | BLOCKED |
| R9-09 | Session Persistence | BLOCKED |
| R9-10 | Cancellation Registry | BLOCKED |
| R9-11 | Runtime Diagnostics | BLOCKED |

## 实施范围与继续位置

原client的三个公共传输类型与HTTP职责已切换transport唯一owner；Formal/Plain/Structured/连接测试/恢复GET/取消空POST继续同一原接口，不升级协议、不修改AI Workspace前端或凭据/重试政策。详见01记录；01自身Windows全SUCCESS，02已自身精确Windows通过并DONE，下一步03重试/熔断/取消职责拆分。

沿用现有单测/contract、Windows runner/workflow与最终新库入口；不新增持续回归基础设施，不执行Linux/macOS动态。每项必须自身精确Windows验收及实施记录才能DONE。已创建01实施记录；其状态已DONE，不创建R9阶段完成记录。Create State仅记录足球模型项目0.23.0的交接状态。


## R9-01 实施与门禁（2026-10-09，DONE）

从c72e559已验源码文档基线建立唯一R9分支，原client将TransportResponse/OpenAiTransport迁入transport/contract；原ReqwestTransport/headers/send/三种请求/rustls Once/error转换由http唯一持有；纯JSON/空响应/解析错误由response持有，mod只登记/re-export。crate根仍导出原三个名字，Application/Tauri/原mock调用方不需变化。原GatewayAttemptSink、协议/重试/并发/取消/研究与会话职责保持原owner，无额外全局状态或转发层。

原Gateway单测target新增8项：原三HTTP动作在有界127.0.0.1夹具核对方法/URL/头/body/429原样返回；禁止跟随307；响应体等待仍超时及原恢复；非法URL原Network/无provider metadata；认证头及非法字符拒绝；任意JSON/status透传；空body Null/原metadata；whitespace/截断/非法UTF8/零字符原SchemaValidation错误。仅WindowsCI实际运行loopback/单测，不调用真实API/不新增target、runner、依赖或协议字段；源码预期Gateway23/contract16，Application126/Persistence175保持。

83/83现有静态、完整architecture、18保护资产/171命令/46迁移/18PG静态、Rustfmt1.88源码check及diff通过；6破坏探针全部拒绝恢复。8原HTTP函数逐token保持，decode重内联后原execute保持、剩余client全体token与265生产literal保持。清单只登记OpenAiTransport实际owner和Domain扫描1099→1103/使用摘要，43Ports/365Domain/300映射/声明摘要保持。Context7只给latest示例，另核对官方reqwest0.13.4准确版本timeout范围；不升级API/依赖。Mermaid已更新原入口与策略/共享IO边界，取消不是transport新增state，连接测试不强加执行重试。

本项自身Windows fmt/Clippy/workspace/前端/17视口/release/MSI/NSIS/启动已全部通过，精确证据见下段；此前CI启动后停止轮询。详细A/M/D、等价/测试/报告/订正与回退见01记录。真实PG、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共stub不计PASS。

## 01 收尾与02授权

精确实现 `f3da62d4b84235044c7483399f6f8aa145d9438d` / [Windows run `37933341557`](https://github.com/uniquenesssta/123/actions/runs/37933341557) / job `113829199317` 全 SUCCESS（2026-10-09 21:24北京时间完成）；Gateway **23/23**、contract **16/16**、Application **126/126**、Persistence **175/175**，本项8个新增测试全部通过；前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动全部通过。证据artifact `11617684270`，14,032,903字节，SHA-256 `a2178dd81182ec5839db1e0d53b57cc0a40c00802a3f553c159164124ae19c48`。

五份文档同步01 DONE/02 READY，源码与验证器保持，提交 `[skip ci]` 复用精确源码结果。02已实施，下一步核实自身Windows；03～11仍BLOCKED，R9整体仍IN_PROGRESS。

## R9-02 实施与门禁（VERIFYING）

基线b80f09e；9A/9M/1D，credentials目录唯一持密钥生命周期/提供者/目标操作/WindowsIO/解码/错误/纯脱敏，旧单文件及解析器重复脱敏删除。原六公共接口、cfg native8函数、provider/错误/配置政策保持；失败副本/UTF16清理及已提取compatible key在嵌套模板替换为四项明确安全变化。原测试保留、新12unit预期35/contract16；83静态/完整架构/format/18保护资产/171命令/46迁移18PG静态及8探针PASS，自身Windows待验。详见02记录；03～11BLOCKED，CI开始后停止轮询。真实PostgreSQL、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共unavailable stub不计PASS。

## R9-02 首轮失败修复

R9-02 首轮 `98d542f` / [Windows run `37944445186`](https://github.com/uniquenesssta/123/actions/runs/37944445186) 在新增 provider Debug 测试被 Clippy `uninlined_format_args` 拦截；前端/类型/构建和17视口已通过，Rust单测/打包/启动未执行。已改为内联参数，提供者生产实现、原测试断言与35项测试数量保持；既有源码卫生检查增加该回归检查，原写法/换行/尾逗号三探针均拒绝并恢复。83/83静态、完整架构、Rustfmt与保护资产通过；仍VERIFYING，须修复提交自身Windows确认，03～11BLOCKED。

修复相对98d542f仅8M、0A/0D/0移动，生产职责/依赖与配置无变化，报告及完整文件见02记录。已有精确01成功不替代02验收，R9仍IN_PROGRESS。


## R9-02 精确 Windows 收尾（2026-10-10，DONE）

精确修复 `05055d1fd90e9a19ee4bc791e769018ca700dd14` / [Windows run `37954536795`](https://github.com/uniquenesssta/123/actions/runs/37954536795) / job `113901577714` 全 SUCCESS，2026-10-10 00:15:32北京时间完成。Gateway **35/35**、contract **16/16**、Application **126/126**、Persistence **175/175**，新增12项及原凭据/示例/传输测试全部通过；前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动7条记录/3完成操作均通过。证据artifact `11628233815`，14,037,356字节，SHA-256 `9a738defde297f2ff10bc0218885efa72d9e7f2c9ad4ff999bcf237cafc0e45a`。

首轮 Clippy 失败已由该修复自身精确 Windows 关闭。当前02 DONE；用户“收尾02开始03”授权同一R9分支03 READY，04～11 BLOCKED，R9整体IN_PROGRESS。本次只同步既有五份文档，源码/验证器/依赖保持，使用 `[skip ci]` 复用上述精确源码结果；此前VERIFYING叙述为实施历史。真实PG、历史四项/账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored和公共stub不计PASS。
