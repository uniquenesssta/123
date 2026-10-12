# R9-07 Plain Chat Responses

状态：`DONE`（07自身精确Windows全SUCCESS；下述VERIFYING为实施历史）。用户2026-10-12“收尾06开始07”授权；原分支`rewrite/r9-research-ai-backend`。06精确代码d996a4d / Windows run38145691538、job114488432087全SUCCESS并DONE；07基线为仅文档收尾提交`9ddb27ac72af3fac2545ef4c1a1fb0968411e3b0`，源码与上述成功一致。07必须自身Windows通过，08～11仍BLOCKED，R9整体IN_PROGRESS。

### 1. 目标

将普通Responses的最小载荷构造和原始文本解码完整迁入唯一纯模块，保留两协议共用执行链与所有兼容政策。

### 2. 现状与来源

R0命令/类型及调用方清单保持。实际普通入口为src-tauri/src/commands/api_workspace.rs中send_api_workspace_message→execute_plain_text→execute_plain_text_with_sink。该入口准备Extraction operation、静态指令、会话历史和只读上下文；原client准备system消息并同时选择Responses/Chat协议；root response为Plain/Structured兼容响应共用文本回退和错误边界。Application预设、workspace持久化和请求注册表分别仍由其原owner承担。

### 3. 目标文件与目录

plain_chat/mod.rs只登记responses；responses/mod.rs只登记request/response并显式crate内导出build_request/extract_text。request.rs拥有Responses wire载荷；response.rs拥有Responses原始文本提取。无新公共API、DTO或通用协议框架。

### 4. 文件职责边界

请求owner只将既有model和借用的(role,content)序列转成原JSON；响应owner只从原Value提取未trim文本。client仍拥有两协议共用消息准备、校验、预算和异步工作流；root response仍拥有共用provider错误、refusal、跨协议回退、统一trim、元数据及用量。原Chat请求/文本具体实现留给08，不在07拆分。

### 5. 输入

请求接收&str model、Vec<(&str,&str)>消息，完全保留原顺序/角色/内容。解码接收&Value，支持原output_text及output/content容错形状，不读取配置、凭据或墙钟。

### 6. 输出

请求返回拥有字符串的serde_json::Value，顶层恰好model/input。解码返回原Option<String>，保留空白和换行；最终PlainTextGatewayResponse和GatewayAttempt由原共用边界构造，公开类型不变。

### 7. 允许依赖

仅原锁定serde_json1.0.150及std；沿用原serde1.0.228、Rust1.88 Windows工具链，无新增依赖或升级。纯协议模块通过显式crate内出口消费。

### 8. 禁止依赖

两个纯owner不依赖transport/reqwest/tokio/key_provider/sink/锁/文件/数据库/Application/Tauri/正式研究政策或GatewayConfig，不实现provider错误、预算、取消、重试或用量政策。

### 9. 状态所有权

没有新增State、缓存、静态或跨请求共享对象。原Gateway配置、transport、provider、Semaphore、CircuitBreaker和CancellationToken状态及生命周期保持；本次Vec为原请求的临时消息准备。

### 10. 副作用边界

新owner纯同步。原Shared Transport唯一HTTP出口、凭据provider、attempt_sink和Tauri请求注册/移除副作用保持，日志不新增问题正文、Schema或凭据。

### 11. 异常路径

HTTP provider错误先于refusal，兼容refusal先于文本，缺文本仍NoResult及原retry/recovery。解码保持非空顶层output_text优先；嵌套字符串及text/content别名按原顺序换行join、忽略未识别形状。原嵌套空白返回Some，随后共用trim为空产生NoResult，不改变为Chat回退；原output内native refusal不被兼容refusal扫描，保留旧NoResult行为。没有借拆分强化正式Schema、来源政策或改变容错。

### 12. 并发/异步/生命周期

校验→后台拒绝→熔断→预算→可取消许可→凭据→模型/重试→发送/取消→解析→attempt_sink顺序保持。原请求ID、offset、指纹、超时、model fallback、退避和远端取消不变；新增模块无async/监听器/定时器/挂起任务。

### 13. 兼容要求

普通Responses只发model/input，system/user/assistant content均input_text；不发送web search/tools/tool_choice/schema/token限制/background/store/reasoning/metadata等。profile配置键、model选择、endpoint、公共类型、错误文字、JSON Schema、数据格式、DB/UI/版本/依赖保持。正式P4仍只走Responses和严格正式响应链。Structured兼容路径继续通过原共用回退复用同一个Responses wire文本解码，不复制第二实现；它不调用普通问答业务政策。

### 14. 实施步骤

迁移原Responses JSON表达式和完整extract_responses_text函数，补原targets内8unit/4contract，再切换两个直接导入。共用client和response可逆还原后与基线逐字一致，其余Gateway源码及原90unit/31contract保持。Domain使用扫描、两个既有兼容门禁和记录同步，无新runner/target/回归框架/DB入口。

### 15. 切换入口

原build_plain_text_request_body的Responses match arm直调新build_request；root response以显式alias导入新extract_text供原extract_compatible_text使用。不存在旧Responses JSON内联、旧fn extract_responses_text或转发方法；共用协调器保留是两个协议的实际边界。

### 16. 删除清单

移除client中的旧Responses JSON职责块和root response的旧Responses文本函数实现；整文件删除：无。移动/重命名：无。没有备份文件、重复owner或空转发壳。

### 17. 最小验证

实际83/83既有JS静态、完整npm run verify:architecture、Rustfmt1.88八变更Rust文件check、diff、等价和7反向探针通过。18保护资产、171命令、46迁移和18PG测试源码保持。Gateway98unit/35contract、Application126/Persistence175是本项Windows待验证预期，不记为本地动态PASS。

### 18. 阶段回归

推送同一分支复用原Public Platform CI/Windows Automated：npm run verify:frontend、TypeScript/Vite/17视口、cargo fmt --all -- --check、cargo clippy --locked --workspace --all-targets -- -D warnings、cargo test --locked --workspace及Windows release/MSI/NSIS/启动/日志。非Windows未运行Cargo/check/Clippy/build/test/loopback/浏览器运行时，遵守总纲Windows唯一动态环境。

### 19. 失败停止条件

自身精确Windows未完整SUCCESS前07 VERIFYING、08～11BLOCKED。静态/公开契约/保护失败先修，不放宽门禁或抑制Clippy；06成功不能代替07验收。

### 20. 回退点

`9ddb27ac72af3fac2545ef4c1a1fb0968411e3b0`，其代码等于06最终成功d996a4d。受控revert07原子提交及同步清单/门禁/记录，禁止手工覆盖用户文件。

### 21. 根 README 摘要记录

06 DONE、07 VERIFYING，明确实际新增/修改/删除、纯协议与共用政策边界、静态证据和Windows待验；不创建阶段完成记录。

### 22. docs 阶段节点详细记录

本记录含23职责字段、全部16真实文件、入口清理/行为/依赖/测试/证据/工具/取舍/回退/延期。阶段索引07 VERIFYING并链接本文件，任务书及TESTING同步。

### 23. 完成标准

实现、静态及记录完成，自身精确Windows全SUCCESS后方可DONE。真实数据库/Full/有效XLSX/私有固定回归保持最终封包新库待验，ignored和公共unavailable stub不计PASS。

## 完整实际文件清单

相对9ddb27a，含未跟踪新文件，实际 **5A / 11M / 0D**，16文件。移动/重命名：无。

新增（5）：

- crates/research-gateway/src/plain_chat/mod.rs
- crates/research-gateway/src/plain_chat/responses/mod.rs
- crates/research-gateway/src/plain_chat/responses/request.rs
- crates/research-gateway/src/plain_chat/responses/response.rs
- docs/modular-rewrite/R09-research-ai-backend/R09-07-plain-chat-responses.md

修改（11）：

- README.md
- architecture/domain-type-inventory.json
- crates/research-gateway/src/client.rs
- crates/research-gateway/src/lib.rs
- crates/research-gateway/src/response.rs
- crates/research-gateway/tests/gateway_contract.rs
- scripts/verify-api-compatible-transport.mjs
- scripts/verify-api-workspace.mjs
- docs/TESTING.md
- docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md
- docs/modular-rewrite/R09-research-ai-backend/README.md

删除：无。Cargo/锁/版本、公开contracts/schemas、配置/DTO/错误、Application/Domain/Tauri/PG/UI、工作流与模型资产均不变。

Domain扫描1134→1138，仅rustUsageDigest变为`0b8ddb0218f0a794dda9ec20befd39849c6953f47f21c0703ba65f67ab6da700`及扫描数量更新；365/300/sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd、Application402/43Ports、19状态/35PG根保持。18保护聚合d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57，46迁移/18PG源码静态聚合d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

## 行为覆盖与验证证据

新增3请求unit覆盖角色/顺序/Unicode/空白/转义、顶层两字段及JSON样用户文本原样、借用消息释放后Value拥有内容；5解码unit覆盖顶层优先/raw空白、混合嵌套顺序/别名/未识别形状、顶层空白回退和嵌套空白Some、缺失/非数组/Chat不在该owner回退、Unicode/fences/JSON文本原样。

新增4contract共18情形：真实公共执行入口的最小载荷/强制Web Search和store配置不泄露/完整attempt证据/用量缓存和成本；7类容错文本/跨协议回退/queued接受/元数据默认/用量别名及嵌套空白NoResult；5类provider/refusal/NoResult优先及原错误/恢复/提供方元数据/失败attempt；5类非法role、缺trace与后台优先、后台、非有限用量、正数预算耗尽在凭据/发送/attempt前停止。沿用既有FakeTransport、CountingKeyProvider、FormalResponseAttemptSink，不调用真实API、数据库或系统凭据。

预算夹具预审发现0预算会先被配置校验拒绝，已改为合法10美元预算和已用10美元，再验证原工作流超限；后台夹具均明确store=true。Web Search模式直接按现有ResponsesWebSearch变体使用。辅助计数首次只统计tokio契约，已纳入3个原同步契约，实际源码98/35。均为提交前预审修正，不是Windows已执行结果。

等价报告：完整client/root response逆迁移字节一致、载荷表达式tokens及完整解码函数一致、全部原公开exports和其余Gateway源码/测试一致、原contract全文前缀一致。526种生产字符串的878次出现多重集一致（本次按所有Gateway src中#[cfg(test)]前的字符串统计），policy_changes=[]。7探针：tools泄露、input_text类型漂移、嵌套trim、回退顺序、根重复owner、请求入口缺失、非法零预算夹具，均拒绝且完整恢复；正常门禁最后通过。

实际命令：`python /workspace/scratch/eb298ad5cdcb/r907_static.py`（83/83）、`npm run verify:architecture`、Rustfmt1.88 `--check --edition 2021 --config skip_children=true`八文件、`python /workspace/scratch/eb298ad5cdcb/r907_equivalence.py`、`python /workspace/scratch/eb298ad5cdcb/r907_probes.py`、`git diff --check`。报告`/workspace/scratch/eb298ad5cdcb/r907-static-checks.json`、`r907-architecture.log`、`r907-equivalence.json`、`r907-negative-probes.json`、`r907-review.json`。

## 工具、取舍与剩余验收

Context7先resolve /serde-rs/json，再查询json!借用字符串/数组的Value所有权；未给1.0.150准确版本，因此另读官方serde-rs/json **v1.0.150 src/macros.rs**，原插值调用to_value(&$other).unwrap()，不以主分支文档冒充锁版本。Mermaid Chart展示真实共享执行→两个请求分支→transport→共用响应→两个解码分支→原回退/attempt关系。Create State按足球稳定模型96f03270-c236-46de-99fa-db85d2fbf4ce保存交接，合成上下文不替代Git/CI结果。

最小完整拆分仅迁Responses专属载荷和文本，不为共用拒绝/用量/重试复制实现，不提前实施08。新增第二个现有门禁读取迁后载荷，避免只检查旧client而漏检新字段。没有新增runner、target、测试框架、DB入口或依赖。

同一分支推送原Windows CI，确认本项精确head/Windows job开始即停止轮询，预计20～30分钟；07 VERIFYING、08～11BLOCKED。真实PG、历史四项、账本并发回滚、有效XLSX、Windows Full、私有P4/P7固定回归，以及继承model.runs/0041删除风险继续最终封包新库待验；ignored、公共stub及静态指纹不计这些真实项目PASS。

## R9-07 完成与08授权（2026-10-12）

精确07提交3dbfde3c7246a08e91ad8bde6dd324a774f5a4a9 / tree38d8ce7f358eb45e8ce73e1414013b46f7462ced，[Windows run38155453893](https://github.com/uniquenesssta/123/actions/runs/38155453893) / job114517387791全SUCCESS，2026-10-12 00:54:23北京完成。Gateway98/98 unit、35/35 contract、Application126/126、Persistence175/175；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过。

证据artifact11709500697，14,040,090字节，SHA-256 10c5237fa3aa7713212cfdcd7647b927bd61d60ed5f82878ce03b190c16f8c01；报告windows-acceptance-20261011-162740.json及同名.txt。07新增8unit/4contract全部执行通过。

07 DONE；用户2026-10-12“收尾07开始08”授权08 READY，同一rewrite/r9-research-ai-backend分支，09～11 BLOCKED、R9 IN_PROGRESS。本次五既有文档[skip ci]收尾，源码树与上述成功一致，先前VERIFYING叙述为实施历史；未创建R9阶段完成记录。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored和公共stub不计PASS。
