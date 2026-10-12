# R9-08 Plain Chat Chat Completions

状态：`DONE`（08自身精确Windows全SUCCESS；下述VERIFYING为实施历史）。用户2026-10-12“收尾07开始08”授权；保持`rewrite/r9-research-ai-backend`。07精确提交3dbfde3c7246a08e91ad8bde6dd324a774f5a4a9 / Windows run38155453893、job114517387791全SUCCESS并DONE。08基线`d065cff34d00abb64651e38f9c32c65b41bca715`为07仅文档收尾，源码与该成功一致；08须自身Windows验收，09～11 BLOCKED，R9 IN_PROGRESS。

### 1. 目标

将普通Chat Completions最小载荷和原始文本提取收敛到唯一纯协议模块，与07 Responses对称接入原共用执行链。

### 2. 现状与来源

R0命令/类型/调用方清单保持。普通入口src-tauri/src/commands/api_workspace.rs的send_api_workspace_message→execute_plain_text→execute_plain_text_with_sink不变，原DTO中的Extraction operation、静态指令、会话历史与只读上下文不变。此前client仍内联Chat JSON，root response仍持Chat文本函数；Plain与Structured兼容响应均消费同一原文本回退。

### 3. 目标文件与目录

plain_chat/chat_completions/{request,response}.rs，mod.rs仅登记和显式crate内导出；plain_chat/mod.rs仅登记两协议。crate根lib和全部公共exports不变。

### 4. 文件职责边界

request只构造原model/messages载荷；response只从首条choice提取未trim文本。client仍准备共用system/历史消息并选择两个纯请求owner；root response仍持provider/refusal优先级、跨协议回退、统一trim、元数据和用量。Structured请求的Schema/工具构造、正式P4链及Session/注册表/诊断留其原owner。

### 5. 输入

原model &str、Vec<(&str,&str)>消息；原&Value响应。没有新配置、参数、环境变量或协议版本。

### 6. 输出

拥有字符串的Value，顶层恰好model/messages，逐消息role/content且content为字符串；原Option<String>包含原空白与换行。最终PlainTextGatewayResponse与GatewayAttempt字段保持。

### 7. 允许依赖

原serde_json1.0.150及std；锁定serde1.0.228/Rust1.88 Windows环境保持。直接显式crate内导入两个owner，无新crate/依赖/通用协议抽象。

### 8. 禁止依赖

纯owner不依赖transport/reqwest/tokio/key_provider/sink/锁/文件/数据库/Application/Tauri/正式政策/GatewayConfig，不解析用量或provider错误，不新增finish_reason政策。

### 9. 状态所有权

无新增State/缓存/静态/跨请求状态。原Gateway配置、transport、provider、Semaphore、CircuitBreaker和CancellationToken继续唯一所有权；临时消息Vec与返回Value的生命周期保持。

### 10. 副作用边界

新owner纯同步，原HTTP/凭据/sink/注册与移除仍各自单入口；不新增文件、数据库或日志副作用，不记录问题正文或凭据。

### 11. 异常路径

仅choices数组首项，原非空legacy choice.text优先于message.content。message.content字符串包括空白/空字符串时原样Some；数组按字符串及text/content别名原顺序换行join，未知形状忽略。没有可识别首项才由共用边界回退Responses，已识别空白仍在统一trim后NoResult，不选第二choice、不按finish_reason拒绝。原provider→top/first-message refusal→文本顺序、错误类型/提示/recovery保持。

### 12. 并发/异步/生命周期

新owner无async/监听器/定时器。原校验→后台→熔断→预算→可取消许可→凭据→模型/重试→发送/解析/attempt顺序保持；请求ID、offset、指纹、超时、退避/模型切换/取消/许可释放不变。

### 13. 兼容要求

普通Chat不发input/web search/tools/tool_choice/schema/token限制/background/store/reasoning/metadata等字段。配置/profile/model/endpoint、public DTO/Schema/数据/DB/UI/版本/依赖/错误均保持。07 Responses文件字节不变；正式研究继续仅Responses及严格正式解析；Structured兼容仅通过原共用边界复用唯一Chat wire文本解码。

### 14. 实施步骤

先准备原表达式/函数的纯owner和原targets内8unit/4contract，再登记模块、切换两处直接导入并移除旧实现。两个既有兼容门禁读取新载荷与出口/解码，Domain使用清单与记录同步。未新增runner、workflow、target、测试框架或数据库专项入口。

### 15. 切换入口

build_plain_text_request_body的Chat arm直调新build_request；root response alias导入新extract_text供原extract_compatible_text使用。共用协调器只有消息准备/协议选择；没有旧Chat JSON、旧fn extract_chat_completions_text或转发方法。

### 16. 删除清单

移除client的原Chat JSON块和root response的原Chat文本函数。整文件删除：无；移动/重命名：无；无复制备份或重复owner。

### 17. 最小验证

本地83/83既有JS静态、完整architecture、Rustfmt1.88七变更Rust文件check、等价/8反向探针/diff及保护检查通过。原98unit/35contract保留，新增8unit/4contract；106/39是08自身Windows待验预期，未记动态PASS。

### 18. 阶段回归

原Public Platform CI/Windows Automated执行npm run verify:frontend、TypeScript/Vite/17视口、cargo fmt --all -- --check、cargo clippy --locked --workspace --all-targets -- -D warnings、cargo test --locked --workspace、Windows release/MSI/NSIS/启动/日志。遵守Windows唯一动态环境，未执行非Windows Cargo/check/Clippy/build/test/loopback或浏览器运行时。

### 19. 失败停止条件

08自身精确Windows全SUCCESS前VERIFYING，09～11 BLOCKED。静态/公开契约/保护失败先修，不抑制Clippy/跳测试/放宽门禁，不借07成功冒充08通过。

### 20. 回退点

d065cff34d00abb64651e38f9c32c65b41bca715；源码等于07成功3dbfde3。受控revert08提交与对应清单/门禁/记录，不手工覆盖文件。

### 21. 根 README 摘要记录

07 DONE、08 VERIFYING；记录两个独立协议owner和共用政策、实际文件/静态通过/Windows待验，引用本完整节点，不创建额外fix log或阶段完成记录。

### 22. docs 阶段节点详细记录

本记录含23职责字段、全部15真实文件、入口清理、兼容/测试/证据/取舍/回退/延期；阶段索引链接本文件并维护08 VERIFYING，任务书/TESTING同步。

### 23. 完成标准

实现、静态与记录完成，08自身精确Windows全SUCCESS后方可DONE。真实PG/Full/有效XLSX/私有固定回归继续最终封包新库待验，ignored/公共stub不计这些真实验收PASS。

## 完整实际文件清单

相对d065cff，含新文件，**4A / 11M / 0D**，15文件。移动/重命名：无。

新增（4）：

- crates/research-gateway/src/plain_chat/chat_completions/mod.rs
- crates/research-gateway/src/plain_chat/chat_completions/request.rs
- crates/research-gateway/src/plain_chat/chat_completions/response.rs
- docs/modular-rewrite/R09-research-ai-backend/R09-08-plain-chat-chat-completions.md

修改（11）：

- README.md
- architecture/domain-type-inventory.json
- crates/research-gateway/src/client.rs
- crates/research-gateway/src/plain_chat/mod.rs
- crates/research-gateway/src/response.rs
- crates/research-gateway/tests/gateway_contract.rs
- scripts/verify-api-compatible-transport.mjs
- scripts/verify-api-workspace.mjs
- docs/TESTING.md
- docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md
- docs/modular-rewrite/R09-research-ai-backend/README.md

删除：无。原lib、Responses四模块、配置/DTO/错误/预算/resilience/transport、全部原其他Gateway源码与测试、contracts/schemas、Application/Domain/Tauri/PG/UI、依赖/锁/版本/工作流/模型资产不变。

Domain扫描1138→1141，仅rustUsageDigest变为3cd4c687ac512bf476c270e38a2908b9b67eae248332e9ee96fb1d93965784b5和扫描数量更新。365/300/sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd、Application402/43Ports、19状态/35PG根保持。18保护聚合d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57、171命令、46迁移/18PG源码静态聚合d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

## 行为覆盖与证据

3请求unit：角色/顺序/Unicode/空白/转义和字符串content、两顶层字段/JSON样内容不被解释、借用释放后Value拥有内容。5解码unit：legacy优先/仅首choice、空legacy回退与message空白Some、数组混合形状/text-content优先及原join、缺失/无效形状不在纯owner回退Responses、Unicode/fences/JSON及finish_reason不影响原提取。

4contract：完整model/messages载荷、store和原Auto联网配置不泄露、完整raw/provider/response-ID/指纹/offset/缓存别名用量/成本/attempt；9类首choice/legacy/message/数组/Responses回退/空白NoResult/元数据默认或空字符串/queued接受/缺ID；5类provider/refusal/NoResult原优先及原提示/recovery/失败证据；正常限流重试→model_not_found切fallback三attempt和取消后停止重试/fallback、保留未消费响应、释放许可/新token恢复两attempt。四个契约合计17个矩阵/执行情形，并核对模型切换时原消息/两字段及连续attempt号。

沿用FakeTransport、CountingKeyProvider、FormalResponseAttemptSink、CancelAfterAttemptSink及07既有PlainTextGatewayRequest夹具；仅新增合法Chat协议/HTTPS端点配置helper，无重复请求DTO/框架。取消分支沿用原60秒退避配置和2秒timeout：必须立即取消；恢复请求的model_unavailable立即切模，不等待60秒。正常重试使用原1ms退避。未访问真实API/数据库/系统凭据。原共用校验、后台拒绝和预算的五类07 contract仍执行，未另复制同政策测试。

等价：完整client/common response逆迁移后与基线字节一致、Chat JSON表达式tokens与完整解码函数一致、原lib/公开exports/其余Gateway源码及测试一致、原contract全文前缀一致；526种生产字符串878次出现多重集一致，policy_changes=[]。8探针（tools泄露、content改数组、首choice改末项、嵌套trim、跨协议回退顺序、根重复decoder、请求入口缺失、取消后队列断言丢失）均拒绝且完全恢复，正常门禁最后通过。

实际命令：`python /workspace/scratch/eb298ad5cdcb/r908_static.py`（83/83）、`npm run verify:architecture`、Rustfmt1.88七文件`--check --edition 2021 --config skip_children=true`、`python /workspace/scratch/eb298ad5cdcb/r908_equivalence.py`、`python /workspace/scratch/eb298ad5cdcb/r908_probes.py`、`git diff --check`。报告`/workspace/scratch/eb298ad5cdcb/r908-static-checks.json`、`r908-architecture.log`、`r908-equivalence.json`、`r908-negative-probes.json`、`r908-review.json`。106unit/39contract、Application126/Persistence175仅当前源码预期，尚未记08 Windows PASS。

## 取舍、工具与剩余验收

07已核实Context7 /serde-rs/json及官方serde-rs/json v1.0.150 src/macros.rs的原json!借用插值→拥有Value行为，证据继续适用；08迁移原表达式和Value提取函数，没有新库/API/版本事实需另查，不升级依赖。Mermaid Chart展示实际两请求模块→transport→共用兼容边界→两文本模块→原回退/用量/attempt；Create State先确认足球稳定模型96f03270-c236-46de-99fa-db85d2fbf4ce。

两个原门禁同步新Chat owner；07预算夹具守卫限定到原07函数，避免追加08测试后越界扫描，不删除原断言。为保留真实共用边界，没有把兼容refusal/provider/预算/取消/用量迁成两个副本；没有提前实施09 Session Persistence、10注册表或11诊断。

08同一分支推送原Windows CI，确认精确head/Windows job开始即停止轮询，预计20～30分钟。07 DONE、08 VERIFYING、09～11 BLOCKED。真实PG、历史四项、账本并发回滚、有效XLSX、Windows Full、私有P4/P7固定回归和继承model.runs/0041历史删除风险继续最终封包新库待验，ignored/公共unavailable stub/静态指纹不计真实PASS。

## R9-08 完成与09授权（2026-10-12）

精确08提交7fd7e1e0eeec5f8b1e3a6a516a32ba21e9f067f9 / treea029512756ee7524367ff1c3daa4ba0ca293688c，[Windows run38191340582](https://github.com/uniquenesssta/123/actions/runs/38191340582) / job114624551223全SUCCESS，2026-10-12 09:45:37北京完成。Gateway106/106 unit、39/39 contract、Application126/126、Persistence175/175；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7记录、3完成操作）。

证据artifact11722500937，14,041,703字节，SHA-256 791a0a3414641bdbc70b524c69be0ef16e4b3e57a93078b2b4809bcd2844ff29；报告windows-acceptance-20261012-011909.json及同名.txt。08新增8unit/4contract全部执行通过。

08 DONE；用户2026-10-12“收尾08开始09”授权09 READY，同一rewrite/r9-research-ai-backend分支，10～11 BLOCKED、R9 IN_PROGRESS。本次五既有文档[skip ci]收尾，源码与上述成功一致，先前VERIFYING为实施历史；未创建R9阶段完成记录。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored与公共stub不计PASS。
