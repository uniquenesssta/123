# R9：Research Gateway 与 AI Workspace 后端重写——独立执行任务书

> 文档编号：`R09`  
> 前置阶段：`R8`  
> 后续阶段：`R10`  
> 本文档是唯一执行依据之一；必须与 `00-总体架构与前23节.md` 同时适用。

## 当前执行订正（2026-10-09）

R8 的12个节点及 Windows Automated 阶段出口已DONE，最终代码 `af3c98c31e28a332fe19ec47f4ed11c3a5a261ab` / [Windows run `37899786755`](https://github.com/uniquenesssta/123/actions/runs/37899786755) 全 SUCCESS。实际来源/文件/门禁/回退见 [R8阶段完成记录](../modular-rewrite/R08-prediction-p4-orchestration/R08-stage-completion.md)；继承真实PG/XLSX/Full/私有固定回归及删除风险仍最终新库待验，不继承为PASS。

执行总纲顶部订正：只在Windows动态验证，复用原targets/contract/workflow/数据库入口，不新增持续回归体系；R9不升级外部协议、不修改AI Workspace前端。R8期间 `crates/research-gateway/`、公开契约、依赖与配置保持，最终CI原Gateway测试/contract通过，原协议行为作为下一节点基线。

用户已明确“开始R9-01”，编号映射已厘清。从R8文档收尾 `c72e559af4f29c9510daf2f9bf66b926dacb3013` 建立唯一分支 `rewrite/r9-research-ai-backend`；01共享传输自身精确Windows全SUCCESS、DONE，用户“收尾01开始02”授权02，已自身精确Windows全SUCCESS、DONE；用户“收尾02开始03”授权03，已自身Windows全SUCCESS并DONE；用户“收尾03开始04”授权04，已实施VERIFYING，05～11 BLOCKED，精确状态见 [阶段索引](../modular-rewrite/R09-research-ai-backend/README.md)。必须取得01自身Windows CI，R8成功不能替代。适用总纲顶部职责/生命周期/既有验证订正，真实验证延期边界继续保留。

## 1. 阶段目标

- 把正式研究与普通问答拆成独立协议和工作流。
- 统一 transport、凭据脱敏、重试、熔断、取消和诊断。

## 2. 本阶段解决的实际问题

- 正式研究和普通聊天若共享过多逻辑，容易发生工具、schema、web search 或 token 字段串线。

## 3. 前置输入与进入条件

- R8 完成。
- OpenAI 配置和当前协议行为已冻结。

## 4. 明确范围

### 4.1 纳入范围

- Shared transport。
- Credentials/redaction。
- Retry/circuit breaker/cancel。
- Formal Responses research。
- Plain Responses/Chat Completions。
- Session persistence。
- Cancellation registry。
- Runtime diagnostics。

### 4.2 排除范围

- AI Workspace 前端。
- 外部协议升级。

## 5. 当前实现来源与扫描重点

- `crates/research-gateway/`。
- application/openai_research、api_workspace。
- src-tauri openai/workspace stores。

## 6. 目标目录总览

```text
crates/research-gateway/src/transport/
crates/research-gateway/src/credentials/
crates/research-gateway/src/formal_research/
crates/research-gateway/src/plain_chat/
crates/application/src/use_cases/ai_workspace/
crates/persistence-postgres/src/adapters/ai_workspace/
```

## 7. 目录与文件边界规则

- 正式研究和普通问答有独立 request/response 类型。
- 凭据只通过 Secret Provider 进入 transport。
- 诊断只能记录脱敏元数据。

## 8. 数据流、调用流与依赖方向

```text
formal request -> Responses transport + schema + citations
plain chat -> selected plain protocol -> text response
cancel action -> cancellation registry -> transport abort -> late result discard
```

## 9. 状态所有权与事务边界

- 会话持久化由 AI Workspace Repository 持有。
- 单次请求取消令牌由 cancellation registry 以 request ID 唯一持有。

## 10. 公共契约与兼容要求

- 正式研究只走 Responses。
- 普通问答不带 web search/tools/schema/token fields。
- 现有配置键与 profile 行为不变。

## 11. 风险与禁止事项

- 密钥不得进入日志、错误正文或序列化快照。
- 不得在协议失败时从正式研究静默降级到普通聊天。

## 本阶段 docs 实施记录目录

本阶段使用固定目录：

```text
docs/modular-rewrite/R09-research-ai-backend/
├─ README.md
├─ R09-01-shared-transport.md
├─ R09-02-credentials-and-redaction.md
├─ R09-03-retry-circuit-breaker-and-cancel.md
├─ R09-04-formal-research-request.md
├─ R09-05-formal-response-schema-and-citation.md
├─ R09-06-source-and-time-policy.md
├─ R09-07-plain-chat-responses.md
├─ R09-08-plain-chat-chat-completions.md
├─ R09-09-session-persistence.md
├─ R09-10-cancellation-registry.md
├─ R09-11-runtime-diagnostics.md
└─ R09-stage-completion.md
```

执行要求：

- 第一个节点进入 `READY` 前创建本目录和 `README.md`。
- 每完成一个节点，立即创建对应记录文件，不得等到阶段结束后集中补写。
- 每个记录必须基于真实 `git diff --name-status`、实际目录树和真实验证结果填写。
- 每次节点状态变化都同步更新本阶段 `README.md`。
- 所有节点完成后创建阶段完成记录；该文件缺失时，本阶段不得通过出口门禁。
- 根 `README.md` 只保存摘要和记录链接，详细变更以本目录为准。


## 全阶段强制执行规则

1. 本阶段只允许修改本阶段明确列出的目标模块，不得夹带无关重命名、格式化、依赖升级或功能变化。
2. 模型保护区 `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 以及关联参数、Profile、Schema、fixture、Golden Master 不得修改。
3. 任何原文件出现第二个独立职责时，必须把该职责模块升级为目录：原职责迁入具名文件，新职责进入新的具名文件；不得继续向原文件追加。
4. 不得使用 `old`、`new`、`legacy`、`copy`、`final`、`v2` 作为长期文件名或目录名。所谓旧职责和新职责必须使用真实业务语义命名。
5. 每条公共 Tauri 命令、DTO 字段、数据库格式、配置键、错误语义、日志等级和用户可观察行为默认保持兼容。
6. 每个业务状态只能有一个所有者；View 不拥有业务状态，API 不拥有页面状态，Repository 不拥有工作流状态。
7. 新旧实现只能在单个 Atomic Task 的受控切换窗口内短暂共存；任务结束前必须切换唯一入口并删除旧实现。
8. 不得新增生产依赖。确有必要时，必须单独提交依赖评估，不得混入业务任务。
9. 每个 Atomic Task 必须先通过最小验证，再运行阶段回归；硬性验证失败立即停止，不得进入下一任务。
10. 实际源码、配置、接口或行为发生变化时，同步更新根目录 `README.md`，只记录实际完成和实际验证结果。
11. 每个节点完成时必须创建 `docs/modular-rewrite/R09-research-ai-backend/<task-record>.md` 并更新阶段 `README.md`；阶段完成时必须创建 `R09-stage-completion.md`。缺少记录不得标记为 `DONE`。

## 原计划阶段摘要（保留用于追溯）

## R9：Research Gateway 与 AI Workspace 重写

Atomic Tasks：

- R9-01 shared transport
- R9-02 credentials and redaction
- R9-03 retry/circuit breaker/cancel
- R9-04 formal research request
- R9-05 formal response/schema/citation
- R9-06 source and time policy
- R9-07 plain chat Responses
- R9-08 plain chat Chat Completions
- R9-09 session persistence
- R9-10 cancellation registry
- R9-11 runtime diagnostics

关键验收：

- 正式研究仅 Responses。
- 普通问答无 web search/tools/schema/token fields。
- 密钥不落日志。
- 请求取消可追踪。
- 正式和普通链路互不降级。

---

# Atomic Tasks

## R9-01 Shared Transport

状态：`DONE`（用户明确开始R9-01；实际共享传输已实施，静态通过，须自身精确Windows）。

### 1. 目标

- 收敛原共享HTTP契约、请求发送、响应解码职责，保持正式/普通/分析/连接/恢复/取消各原入口。

### 2. 现状与来源

- 原client.rs混合TransportResponse、OpenAiTransport、ReqwestTransport/rustls/错误与业务网关。提取该实际责任，不提前迁移协议、凭据、重试/熔断/取消、会话或诊断。

### 3. 目标文件与目录

- `crates/research-gateway/src/transport/{mod,contract,http,response}.rs`；原crate根显式导出原三个公共名字。

### 4. 文件职责边界

- contract：原trait/DTO；http：唯一HTTP客户端/认证头/三操作/超时/字节读取/网络错误/原TLS Once；response：纯JSON/空body/解析错误；mod仅登记导出。原Gateway策略、GatewayAttemptSink及provider错误解释仍client，不创建空转发。

### 5. 输入

- 原借用ApiKey/URL/JSON与Duration；无新增字段或提供器协议。

### 6. 输出

- 原status/provider_request_id/Value或完整原GatewayError；transport不解释provider业务状态。

### 7. 允许依赖

- 原reqwest0.13.4/rustls0.23.42/async-trait/serde_json/std及crate内原类型；不升级/新增。

### 8. 禁止依赖

- 领域/持久化/Tauri/UI；不向transport放协议组装、Token/tools/schema、凭据读取或Gateway重试状态。

### 9. 状态所有权

- 原ReqwestTransport持同一cloneable Client；原TLS Once唯一；Gateway持原Semaphore/circuit，原凭据/取消owner保持，无新全局state。

### 10. 副作用边界

- 三原HTTP操作共用http/execute，原headers/timeout/send/body读及错误保持；response纯解码不IO。

### 11. 异常路径

- 原MissingCredential/Network/Timeout/SchemaValidation完整消息/恢复/provider_status保持。JSON错误仍先于Gateway对HTTP状态的provider解释；空body保留Null，原invalid x-request-id处理保持。

### 12. 并发/异步/生命周期

- 原请求trace/id、外层tokio::select/超时/future drop/重试等待/远端取消保持；transport不新增请求registry/UI监听器。连接测试不擅自新增重试、并发或token取消策略。

### 13. 兼容要求

- 正式研究只Responses，普通双协议最小文本且不带tools/web search/schema/token fields；结构化/连接测试原字段/端点/配置/profile保持。禁止重定向、原UA/Bearer/Content-Type、GET/两POST与原body字节保持。

### 14. 实施步骤

- 核对起点/原真实调用及R0清单，提取唯一owner并切换root exports、架构owner；原Gateway target新增8边界测试，原contract16保持。同步原验证器和使用清单；源码等价与静态门禁/探针通过后推送本项Windows。

### 15. 切换入口

- 同一公开root类型由transport导出；原所有生产/mock调用方直接消费原接口，无旧wrapper或新增side path。

### 16. 删除清单

- client中旧trait/DTO/Reqwest/TLS/网络错误职责移除。无整文件删除或重命名、无双实现。

### 17. 最小验证

- 已通过：原API兼容传输/源码卫生/Domain/保护/命令/DB静态与Rustfmt、等价8函数/265literal及完整client、六破坏探针。Rust编译/单测/loopback实际执行待本项Windows。

### 18. 阶段回归

- 已通过83现有源码/完整architecture；五浏览器项、Rust fmt/Clippy/workspace、前端构建/17视口/Windows交付待原Public Platform CI。没有非Windows动态、新入口/runner/target/数据库/依赖。

### 19. 失败停止条件

- 保护/API/硬门禁失败须修复，01不得DONE或启动02；本项精确Windows未取得前保持VERIFYING。

### 20. 回退点

- `c72e559af4f29c9510daf2f9bf66b926dacb3013`；受控revert同步唯一owner/原测试/门禁/清单/记录，不复制旧实现或变更历史库。

### 21. 根 README 摘要记录

- 已记录实际职责/接口保持、静态/Windows待验、继续位置及继承真实延期。

### 22. docs 阶段节点详细记录

- 已创建 [01实施记录](../modular-rewrite/R09-research-ai-backend/R09-01-shared-transport.md)，完整A/M/D、测试/等价/探针/报告/取舍/回退；阶段索引01DONE、02已授权、03～11BLOCKED。

### 23. 完成标准

- 实际职责与旧实现清理、兼容、静态及记录已完成；必须自身精确Windows成功后才收尾01，不继承R8 PASS。真实PG、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共stub不计PASS。

---

### 本次完成证据

精确实现 `f3da62d4b84235044c7483399f6f8aa145d9438d` / [Windows run `37933341557`](https://github.com/uniquenesssta/123/actions/runs/37933341557) / job `113829199317` 全 SUCCESS（2026-10-09 21:24北京时间完成）；Gateway **23/23**、contract **16/16**、Application **126/126**、Persistence **175/175**，本项8个新增测试全部通过；前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动全部通过。证据artifact `11617684270`，14,032,903字节，SHA-256 `a2178dd81182ec5839db1e0d53b57cc0a40c00802a3f553c159164124ae19c48`。

仅五文档 `[skip ci]` 收尾，无源码/门禁/清单变化；用户授权02，真实延期仍保留。

## R9-02 Credentials 与 Redaction

状态：`DONE`（精确修复05055d1 / Windows run37954536795全SUCCESS，首轮Clippy失败已关闭）。

### 1. 目标

- 将实际凭据生命周期、来源选择、档案目标、平台IO、编码与持久化脱敏拆为唯一owner，补齐四处安全边界；不推进后续节点。

### 2. 现状与来源

- 原Gateway credentials.rs与api_example.rs；只扫描Application/Tauri profile/workspace真实调用，本轮其源码无改动。

### 3. 目标文件与目录

- credentials/{mod,key,provider,store,windows,blob,error,redaction}.rs；实际清单见02记录。

### 4. 文件职责边界

- mod只显式导出；key生命周期，provider来源，store目标/三公开操作，windows平台IO，blob纯编码，error原共享错误，redaction纯持久化模板。

### 5. 输入

- CredentialConfig/原String key及target；示例endpoint/JSON与借用的瞬时提取key。

### 6. 输出

- 原ApiKey/ApiKeyProvider/Default与三个公开函数、原GatewayError、canonical sanitized模板；公共DTO与名称保持。

### 7. 允许依赖

- 原async_trait/zeroize1.9.0/windows-sys0.61/serde_json；只复用锁定依赖。

### 8. 禁止依赖

- 无新增runner/workflow/target/DB/依赖；纯key/blob/redaction不得环境/文件/网络或后台IO，不跨到协议或UI。

### 9. 状态所有权

- ApiKey唯一String所有权、Drop与Debug掩码；临时Zeroizing。无新跨模块State或第二provider；原profiles状态/rollback不动。

### 10. 副作用边界

- Windows适配器唯一原native调用；Default provider唯一原环境读取；目标及key错误在nativeIO前停止，blob/redaction纯函数。

### 11. 异常路径

- 原category/完整中文提示/recovery/provider metadata保持；107生产literal保持；normalized/save早返/UTF16临时清理修复。

### 12. 并发/异步/生命周期

- 原async load/Send+Sync与外层取消原样；ApiKey与临时缓冲销毁清零，无新线程/registry/监听。测试不修改全局环境。

### 13. 兼容要求

- 正式Responses与普通问答最小字段、双协议/profile/配置/DTO/UI保持；有意只修改含已提取凭据的模板文本/对象键。

### 14. 实施步骤

- 按真实调用/原四凭据测试与三示例测试核对，切唯一目录出口；删除旧owner，更新原artifact/owner门禁，补12项原unit测试并验证。

### 15. 切换入口

- 原mod credentials继续解析为目录；lib/client逐字保持；api_example仅从credentials出口一次调用统一sanitized_api_example。

### 16. 删除清单

- 旧credentials.rs及api_example四重复脱敏helper/placeholder常量；无forwarding shell、整文件移动或重复状态。

### 17. 最小验证

- 83/83原静态、完整architecture、原API/Domain/源码卫生/保护资产/命令/数据库静态、Rustfmt1.88和diffPASS。

### 18. 阶段回归

- 自身Windows原前端/contracts/TS/Vite/17视口、fmt/Clippy/workspace与35/16/126/175预期、release/MSI/NSIS/启动待验；本地非Windows无动态。

### 19. 失败停止条件

- 保护指纹/公共契约未批准变化、任一门禁失败或用户修改冲突硬停；首轮清单漂移用官方生成器修正后全量83/架构复验，未放宽。

### 20. 回退点

- 基线b80f09e6488af18435c8e281395ddf7962c48eb5；受控revert源码/实际清单/门禁/文档，不复制双实现或更改历史数据。

### 21. 根 README 摘要记录

- 已同步01 DONE/02VERIFYING、9A/9M/1D、12原target新测试与待验/安全取舍/继续位置。

### 22. docs 阶段节点详细记录

- 已创建[02实施记录](../modular-rewrite/R09-research-ai-backend/R09-02-credentials-and-redaction.md)，含完整清单、边界/四项差异/12测试/8探针/等价/报告/工具/回退；阶段索引02VERIFYING/03～11BLOCKED。

### 23. 完成标准

- 实际唯一职责与旧实现清理、兼容、静态及文档已完成；必须自身精确Windows成功才DONE，不能继承01或源码预期。真实PostgreSQL、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共unavailable stub不计PASS。

### 首轮失败与修复证据

R9-02 首轮 `98d542f` / [Windows run `37944445186`](https://github.com/uniquenesssta/123/actions/runs/37944445186) 在新增 provider Debug 测试被 Clippy `uninlined_format_args` 拦截；前端/类型/构建和17视口已通过，Rust单测/打包/启动未执行。已改为内联参数，提供者生产实现、原测试断言与35项测试数量保持；既有源码卫生检查增加该回归检查，原写法/换行/尾逗号三探针均拒绝并恢复。83/83静态、完整架构、Rustfmt与保护资产通过；仍VERIFYING，须修复提交自身Windows确认，03～11BLOCKED。

修复仅8M：单个测试format参数、原源码卫生门禁、Domain使用摘要及五份现有文档；生产职责/原错误/协议保持，无A/D/移动。原生Windows单测/打包/启动未完成，修复CI開始后停止轮询，不提前DONE/03。

---


## R9-02 精确 Windows 收尾（2026-10-10，DONE）

精确修复 `05055d1fd90e9a19ee4bc791e769018ca700dd14` / [Windows run `37954536795`](https://github.com/uniquenesssta/123/actions/runs/37954536795) / job `113901577714` 全 SUCCESS，2026-10-10 00:15:32北京时间完成。Gateway **35/35**、contract **16/16**、Application **126/126**、Persistence **175/175**，新增12项及原凭据/示例/传输测试全部通过；前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动7条记录/3完成操作均通过。证据artifact `11628233815`，14,037,356字节，SHA-256 `9a738defde297f2ff10bc0218885efa72d9e7f2c9ad4ff999bcf237cafc0e45a`。

首轮 Clippy 失败已由该修复自身精确 Windows 关闭。当前02 DONE；用户“收尾02开始03”授权同一R9分支03 READY，04～11 BLOCKED，R9整体IN_PROGRESS。本次只同步既有五份文档，源码/验证器/依赖保持，使用 `[skip ci]` 复用上述精确源码结果；此前VERIFYING叙述为实施历史。真实PG、历史四项/账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored和公共stub不计PASS。

## R9-03 Retry、Circuit Breaker 与 Cancel

状态：`DONE`（d58aa38 / Windows run37960534145完整SUCCESS）。

### 1. 目标

- 已把原重试预算/等待、熔断状态与本地取消令牌归到唯一resilience职责；协议编排留原client，不改业务策略。

### 2. 现状与来源

- 原client.rs混合三协议、三个重复退避select、CircuitState/三状态方法、取消错误；原cancellation.rs只持本地令牌。
- 原GatewayAttemptSink、Application/Tauri Registry、Responses远端取消/轮询继续原owner；R9-10另处理Registry。

### 3. 目标文件与目录

- `resilience/{retry,circuit_breaker,cancellation}.rs` 与仅登记/显式export的mod.rs；旧cancellation.rs删除。
- 按总纲顶部订正使用三个实际职责文件，不为单一可独立职责机械创建空子目录。

### 4. 文件职责边界

- retry持原次数上限、指数退避与可取消等待，不持模型/错误重试资格/协议/账本。
- CircuitBreaker持单一Mutex<CircuitState>及check/success/failure；配置仍唯一GatewayConfig，不克隆第二政策对象。
- CancellationToken持原Arc<AtomicBool+Notify>共享克隆及原Cancelled错误；不持网络响应ID/Registry。

### 5. 输入

- 原max_retries、retry_base_delay_ms/retry_index；GatewayError及借用CircuitBreakerConfig；原CancellationToken。

### 6. 输出

- 原每模型max_retries+1预算、原Duration/Ok或Cancelled、原CircuitOpen/Ok及同一令牌公共接口。

### 7. 允许依赖

- 已锁定std、tokio1.52.3 sync/time/macros与现有GatewayError/Config；依赖/feature/锁文件保持。

### 8. 禁止依赖

- resilience不调用transport、key_provider、HTTP、账本/数据库、Domain预测、Tauri、UI或Responses远端取消。

### 9. 状态所有权

- CircuitBreaker实例唯一持consecutive_failures/open_until，三协议共用；令牌原Arc生命周期唯一持cancelled/notify。
- 原architecture/state-ownership登记这两个实际owner，配置仍GatewayConfig，Semaphore仍client原请求scope。

### 10. 副作用边界

- retry仅原tokio sleep及取消等待；circuit仅原锁内内存状态；token仅原SeqCst原子位/notify_waiters。
- 所有网络调用、远端cancel/poll与attempt sink仍原client/transport，不新增IO、日志或持久化事件。

### 11. 异常路径

- 原四类Network/Timeout/RateLimit/ProviderUnavailable计数，与retryable位独立；其余15类不计数且不清零。
- 到阈值设置原open_seconds窗口，下一入口拒绝；到期或在途成功清零；不新增半开单探针政策。
- 原sink.record错误先于熔断状态更新；原Cancelled/CircuitOpen完整错误、metadata与优先级保持。

### 12. 并发/异步/生命周期

- 原三个入口只在请求开始check；在途失败可能打开熔断但当次原重试继续，不增加每次retry检查。
- 原select分支顺序、公平选择、网络future丢弃及OwnedSemaphorePermit请求scope保持。
- 令牌先创建Notified再查取消位；SeqCst/notify_waiters/幂等cancel、克隆共享及独立token保持，丢弃等待不取消token。

### 13. 兼容要求

- 正式研究只Responses；Plain双协议仍不发送tools/web/schema/token字段。
- Structured自动web-search兼容回退、模型fallback资格/去重、每模型预算、attempt_offset/饱和及事件顺序全部保持。
- 原公共Gateway/CancellationToken API、DTO、Schema、配置键/profile、错误文案/日志/UI/版本0.23.0保持。

### 14. 实施步骤

1. 已核实02修复自身Windows全SUCCESS，五文档收尾59af3f2，用户授权03；原35 unit/16 contract为基线。
2. 已迁移三职责及唯一export，原生产体和整个client剩余逻辑完成重路由等价核对。
3. 原unit新增12、原contract新增5；原测试保持，无新target/runner/workflow/数据库。
4. 原兼容传输验证器补唯一owner/三入口/政策/唤醒顺序，6破坏探针拒绝恢复。
5. 静态/保护验证及记录完成；提交自身WindowsCI开始后停止轮询，待下次核实精确结果。

### 15. 切换入口

- crate根继续公开CancellationToken原名；client直接持CircuitBreaker并调用方法，三个等待共用wait_retry。
- 不留下Gateway转发方法、旧取消入口、第二状态或第二退避公式；远端取消不是本地token副作用。

### 16. 删除清单

- 删除旧cancellation.rs；client移除CircuitState、check_circuit/record_success/record_failure、retry_delay/cancelled_error和三重复退避select。
- 原其他client方法和原contract16测试/9具名helper与实现逐token保留；不删除行为/断言。

### 17. 最小验证

- 实际83/83原静态检查、完整verify:architecture、Rustfmt1.88源check、171命令、18保护资产、46迁移/18PG静态及diff PASS。
- 生产3熔断体/原token全部/退避与错误/重路由剩余client/251literal等价，6探针拒绝恢复。
- 源码预期Gateway47/contract21、Application126/Persistence175，新增12+5须本项Windows实际运行，不能记为已PASS。

### 18. 阶段回归

- 原Public Platform CI：完整frontend/contracts/TypeScript/Vite/17视口、fmt/Clippy -D warnings/workspace tests、Windows release/MSI/NSIS/启动。
- 本地仅源码静态，未执行Linux/macOS Cargo/编译/单测/loopback/浏览器动态；真实PG/历史四项/账本并发回滚/有效XLSX/Full/私有固定回归仍最终新库待验。

### 19. 失败停止条件

- 自身Windows失败只修相关链路，03不得DONE/启动04；公共契约/保护资产/静态门禁失败先修复，不放宽或略过。

### 20. 回退点

- 03基线59af3f2ec65030a28150a3042d3b0673d65a48a8，源码等于02精确05055d1/run37954536795成功；受控revert03整体并恢复原唯一owner，不复制双实现、不覆盖用户修改。

### 21. 根 README 摘要记录

- 已同步02DONE/03VERIFYING、5A/10M/1D、职责/等价/12+5测试及实际静态/Windows待验边界。

### 22. docs 阶段节点详细记录

- 已创建03实施记录和准确全文件清单、报告/工具/回退；阶段索引03VERIFYING、04～11BLOCKED，R9整体IN_PROGRESS。

### 23. 完成标准

- 代码/静态/文档已完成；仍须本项精确Windows完整SUCCESS才能03DONE。
- 未执行真实PG等延期项目不冒充PASS，阶段不创建完成记录或提前启动04。

详见 [03实施记录](../modular-rewrite/R09-research-ai-backend/R09-03-retry-circuit-breaker-and-cancel.md)，含全部文件清单/测试/报告/工具与回退。

---


## R9-03 精确 Windows 收尾（2026-10-10，DONE）

精确实现 `d58aa38f88c2b59e3ee3b6f568b54d5430843b29` / [Windows run `37960534145`](https://github.com/uniquenesssta/123/actions/runs/37960534145) / job `113921936420` 全 SUCCESS，2026-10-10 01:05:49北京时间完成。Gateway **47/47**、contract **21/21**、Application **126/126**、Persistence **175/175**，新增12单测/5契约及原测试全部通过；完整前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动验收均通过。artifact `11632067812`，14,041,285字节，SHA-256 `71b4a13385cb2f77cb773e0181f2d1987d533148220b657317ad802507450d5b`。

03 DONE；用户“收尾03开始04”授权同一R9分支04 READY，05～11 BLOCKED，R9整体IN_PROGRESS。本次仅既有五文档 `[skip ci]` 收尾，同源码树复用上述精确Windows；此前VERIFYING是实施历史。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险继续最终新库待验，ignored及公共stub不计PASS。

## R9-04 Formal Research Request

状态：`VERIFYING`（03自身Windows全SUCCESS并DONE，用户已授权04；职责/静态/记录完成，待本项Windows）。

### 1. 目标

- 已将正式研究请求的输入校验、Responses载荷和请求前预算检查归到唯一职责；协议和执行生命周期留原Gateway。

### 2. 现状与来源

- 原client.rs混合三协议编排、正式请求身份/Schema校验、载荷/上下文、预算前检/估算及共用定价/Token字段投影。
- 以03精确成功代码及五文档收尾bbc9031为基线；原Gateway47 unit/21 contract保持。

### 3. 目标文件与目录

- `formal_research/request/{validation,payload,budget}.rs` 分别持校验、载荷和预算；tests.rs仅原unit target行为测试/夹具。
- formal_research/mod与request/mod仅登记/显式export；共用预算政策归src/budget.rs，Token字段投影归request_fields.rs。
- 依总纲顶部订正按真实职责共置函数，不机械逐函数建目录；shared helper必须支持正式与原非正式调用方。

### 4. 文件职责边界

- validation持原trace/match/schema/fact字段及严格Schema根校验，无IO/状态；execute与resume各直接复用一次。
- payload持正式JSON载荷、可信静态instructions/不可信动态输入、Schema副本根字段投影及研究web-search字段。
- 正式budget持原用量前检、主模型/去重fallback与每模型重试预算的完整上界检查；不持真实账本或定价缓存。
- 共用budget只持原精确/最长dash前缀价格查找及BudgetExceeded映射；request_fields只持共享Token别名投影，两者不认识正式协议编排。

### 5. 输入

- 原借用GatewayConfig/GatewayRequest及本次模型；预算使用原Gateway model_for_operation选择的primary_model。
- 路由、配置、请求 DTO、Schema和预算用量来源不变，无新的公开输入类型。

### 6. 输出

- 原Value载荷、原GatewayError或Ok；配置/请求不会被修改，价格仍借用原配置中的ModelPricing。

### 7. 允许依赖

- 已锁定serde_json1.0.150、现有std及Gateway类型；预算复用03 attempt_limit和唯一共享定价，载荷复用唯一Token字段投影。

### 8. 禁止依赖

- 请求职责不依赖HTTP/transport、ApiKey/provider、tokio/取消/锁、GatewayAttemptSink、数据库/Domain预测、Tauri或UI。
- 不将正式构造器用于普通聊天，不把正式请求特有策略放入shared HTTP transport。

### 9. 状态所有权

- 新职责全是借用输入的纯处理，无新跨请求状态/配置副本；Gateway仍唯一持config/transport/provider/Semaphore及03 CircuitBreaker。
- 原19状态、Application402/43Ports/PG根35保持，不登记不存在的新State。

### 10. 副作用边界

- 只构造/校验本次JSON和金额；原凭据读取、许可获取、网络、sink/响应解析/轮询/取消继续原Gateway/transport。
- 不新增API调用、日志字段、持久化事件、Schema文件或真实凭据测试。

### 11. 异常路径

- 原身份错误优先于Schema；根不是object与type/additionalProperties错误分别保留原完整错误。
- 原正式执行顺序：请求校验→Responses协议→熔断→预算→许可→凭据→构造→IO；resume仍请求校验→response_id→background/store→凭据/轮询。
- 无效预算NaN/无限/负值先拒绝；原daily/monthly/per-request优先、完整BudgetExceeded/recovery与无provider metadata保持。

### 12. 并发/异步/生命周期

- 新模块无async/锁/全局变量；借用config/request，仅clone本次provider_schema副本，不修改来源或共享缓存。
- 原attempt offset/饱和、retry/fallback、sink先于熔断、select/OwnedSemaphorePermit及远端取消/后台响应生命周期保持。

### 13. 兼容要求

- 正式Research/Extraction仍Responses，原研究/提取路由不变，普通聊天双协议最小载荷保持；不升级外部API。
- Schema仅移除副本根$schema/$id，嵌套原样；原动态context_is_untrusted、strict/name/metadata/reasoning/store/background/domain filters/tool/include/token字段不变。
- trace64/match200/fact100为字符、Schema名称64为字节且ASCII；31事实及raw精确去重、原trim只检查非空不规范化政策保持。
- 原formal静态指令不擅加Plain的30000字符限制；原缺少价格filter_map政策不擅改，预算公式/路由/profile/DTO/配置/错误/UI/日志/版本0.23.0保持。

### 14. 实施步骤

1. 已核实03自身精确Windows成功，5文档收尾bbc9031，用户“收尾03开始04”已授权。
2. 已按三个正式请求职责及两个实际共用政策拆分，入口直调；迁出7函数及其余client重路由等价核对。
3. 原unit新增13、原contract新增4；原47/21测试保留，无新target/runner/workflow/数据库/依赖。
4. 原兼容传输验证器增加请求owner/入口顺序/政策守卫，7破坏探针全拒绝恢复；完整静态/格式/保护验证通过。
5. 同步实施记录与准确文件清单；提交后确认自身WindowsCI开始即停止轮询，仍VERIFYING。

### 15. 切换入口

- client从formal_research::request直接导入校验/构造/预算，无旧Gateway转发壳；execute/resume共用一个原校验器。
- 原所有成本调用直接共用budget::pricing_for_model，连接测试与正式构造共用request_fields::apply_token_limit；旧helper全部移除。

### 16. 删除清单

- client删除validate_gateway_request/build_request_body/check_budget/estimate_request_ceiling/pricing_for_model/budget_error/apply_token_limit定义；原公开入口未删除。
- 无旧Rust文件整文件删除或改名，无第二构造器/校验器/价格公式；响应/Schema消费验证留05、source/time留06，不提前推进。

### 17. 最小验证

- 本地实际83/83现有源码门禁、完整npm verify:architecture、Rustfmt1.88源码format/check、18保护资产/171命令/46迁移18PG静态、diff及等价通过。
- 7生产函数/整个剩余client重路由tokens、247生产literal、47原unit源及21原contract保持；7探针拒绝恢复。
- 源码预期Gateway60/contract25、Application126/Persistence175，新增13+4须本项Windows实际运行，不能计源码预期为PASS。

### 18. 阶段回归

- 原Public Platform CI/Windows Automated复用完整frontend/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace tests、release/MSI/NSIS/启动。
- 本地不执行Linux/macOS Cargo/编译/单测/loopback/浏览器动态；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归继续最终新库待验。

### 19. 失败停止条件

- 自身Windows未完整SUCCESS前04保持VERIFYING、05～11BLOCKED；失败只修相关链路，保护资产/公开契约/静态门禁失败先修，不放宽门禁或抑制Clippy。

### 20. 回退点

- 04基线bbc9031a41e21c3b4c90831a1d2ec1ce2fdcb1a4，源码等于03精确d58aa38/run37960534145全SUCCESS；受控revert本项，恢复原唯一owner及清单，不复制双实现或覆盖用户修改。

### 21. 根 README 摘要记录

- 已记录03DONE/04VERIFYING、9A/9M/0D、请求职责/兼容、13+4测试、实际静态和Windows待验。

### 22. docs 阶段节点详细记录

- 已创建04实施记录，包含全18文件/测试/等价/报告/工具/偏差/回退；阶段索引04VERIFYING、05～11BLOCKED，R9整体IN_PROGRESS，无阶段完成记录。

### 23. 完成标准

- 实现/静态/文档已完成，须04自身精确Windows完整SUCCESS后才能DONE；延期真实数据库/Full等不能冒充PASS或从03继承。

详见 [04实施记录](../modular-rewrite/R09-research-ai-backend/R09-04-formal-research-request.md)，含全部18文件清单、测试/报告/工具/取舍/回退。

---

## R9-05 Formal Response、Schema 与 Citation

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Formal Response、Schema 与 Citation 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/research-gateway/`。
- application/openai_research、api_workspace。
- src-tauri openai/workspace stores。

### 3. 目标文件与目录

```text
crates/research-gateway/src/formal_research/response/
crates/research-gateway/src/formal_research/schema/
crates/research-gateway/src/formal_research/citations/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 正式研究只走 Responses。
- 普通问答不带 web search/tools/schema/token fields。
- 现有配置键与 profile 行为不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R9-05 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R9-05 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R09-research-ai-backend/R09-05-formal-response-schema-and-citation.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R09-research-ai-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R09-05-formal-response-schema-and-citation.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R9-06 Source 与 Time Policy

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Source 与 Time Policy 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/research-gateway/`。
- application/openai_research、api_workspace。
- src-tauri openai/workspace stores。

### 3. 目标文件与目录

```text
crates/research-gateway/src/formal_research/policies/source_policy.rs
crates/research-gateway/src/formal_research/policies/time_policy.rs
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 正式研究只走 Responses。
- 普通问答不带 web search/tools/schema/token fields。
- 现有配置键与 profile 行为不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R9-06 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R9-06 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R09-research-ai-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R09-06-source-and-time-policy.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R9-07 Plain Chat Responses

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Plain Chat Responses 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/research-gateway/`。
- application/openai_research、api_workspace。
- src-tauri openai/workspace stores。

### 3. 目标文件与目录

```text
crates/research-gateway/src/plain_chat/responses/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 正式研究只走 Responses。
- 普通问答不带 web search/tools/schema/token fields。
- 现有配置键与 profile 行为不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R9-07 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R9-07 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R09-research-ai-backend/R09-07-plain-chat-responses.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R09-research-ai-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R09-07-plain-chat-responses.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R9-08 Plain Chat Chat Completions

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Plain Chat Chat Completions 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/research-gateway/`。
- application/openai_research、api_workspace。
- src-tauri openai/workspace stores。

### 3. 目标文件与目录

```text
crates/research-gateway/src/plain_chat/chat_completions/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 正式研究只走 Responses。
- 普通问答不带 web search/tools/schema/token fields。
- 现有配置键与 profile 行为不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R9-08 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R9-08 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R09-research-ai-backend/R09-08-plain-chat-chat-completions.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R09-research-ai-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R09-08-plain-chat-chat-completions.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R9-09 Session Persistence

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Session Persistence 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/research-gateway/`。
- application/openai_research、api_workspace。
- src-tauri openai/workspace stores。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/ai_workspace/sessions/
crates/persistence-postgres/src/adapters/ai_workspace/sessions/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 正式研究只走 Responses。
- 普通问答不带 web search/tools/schema/token fields。
- 现有配置键与 profile 行为不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R9-09 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R9-09 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R09-research-ai-backend/R09-09-session-persistence.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R09-research-ai-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R09-09-session-persistence.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R9-10 Cancellation Registry

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Cancellation Registry 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/research-gateway/`。
- application/openai_research、api_workspace。
- src-tauri openai/workspace stores。

### 3. 目标文件与目录

```text
crates/application/src/use_cases/ai_workspace/cancellation/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 正式研究只走 Responses。
- 普通问答不带 web search/tools/schema/token fields。
- 现有配置键与 profile 行为不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R9-10 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R9-10 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R09-research-ai-backend/R09-10-cancellation-registry.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R09-research-ai-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R09-10-cancellation-registry.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

## R9-11 Runtime Diagnostics

状态：`BLOCKED`（仅当上一任务与本任务前置门禁通过后改为 `READY`）

### 1. 目标

- 完成 Runtime Diagnostics 的完全重写，并将该能力收敛到唯一、可递归拆分的模块目录。

### 2. 现状与来源

- `crates/research-gateway/`。
- application/openai_research、api_workspace。
- src-tauri openai/workspace stores。

### 3. 目标文件与目录

```text
crates/research-gateway/src/diagnostics/
```

### 4. 文件职责边界

- 每个文件只承担一个可用一句话描述的职责。
- 目录出口文件只负责显式导出。
- 协调器只编排，不实现数据访问、UI 渲染或领域计算。

### 5. 输入

- 无。

### 6. 输出

- 稳定的模块公开接口、可独立测试的实现和对应契约测试。

### 7. 允许依赖

- 无。

### 8. 禁止依赖

- 无。

### 9. 状态所有权

- 该任务不新增跨模块共享状态；需要状态时由目标模块内具名 State/Coordinator 唯一持有。

### 10. 副作用边界

- 所有 I/O、副作用和外部调用必须集中在明确命名的 adapter/transport/repository/workflow 文件。

### 11. 异常路径

- 保持现有错误码、错误类型和用户可见提示语义；新增内部错误必须在边界映射为既有公共错误。

### 12. 并发/异步/生命周期

- 所有异步请求必须具备请求 ID、取消或过期结果丢弃策略；销毁时解除监听器、定时器和挂起回调。

### 13. 兼容要求

- 正式研究只走 Responses。
- 普通问答不带 web search/tools/schema/token fields。
- 现有配置键与 profile 行为不变。

### 14. 实施步骤

1. 读取 R0 生成的文件、命令、类型和调用方清单，确认本任务准确影响范围。
2. 为目标目录创建清晰的 `mod.rs`/`index.ts` 出口，出口只 re-export，不承载业务逻辑。
3. 先迁移或补齐契约测试，再实现新文件。
4. 按职责逐文件实现；发现单文件再次出现第二职责时立即递归升级为子目录。
5. 接入上游和下游，确保跨层只经过公开接口。
6. 切换唯一入口，删除旧职责实现、重复类型、重复状态和重复样式。
7. 运行最小验证、阶段回归和保护资产验证。
8. 更新 README 并创建可回退原子提交。

### 15. 切换入口

- 在新实现通过最小验证后切换唯一调用入口；切换完成后立即运行契约验证。

### 16. 删除清单

- 删除被本任务替代的旧职责实现、重复出口、重复测试和临时转发。

### 17. 最小验证

- 相关 crate/feature 单元测试通过。
- TypeScript/Rust 编译或类型检查通过。
- 架构边界脚本通过。
- 模型保护资产指纹通过。

### 18. 阶段回归

- `npm run verify:frontend`。
- `cargo fmt --all -- --check`。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `cargo test --locked --workspace`。

### 19. 失败停止条件

- 任何保护资产指纹变化。
- 公共契约出现未批准变化。
- 最小验证失败。
- 发现用户未提交修改与目标文件重叠且无法安全合并。

### 20. 回退点

- 回退到 R9-11 开始前的已验证提交；不得手工复制旧文件恢复。

### 21. 根 README 摘要记录

- 记录 R9-11 实际创建、移动、删除的文件。
- 记录执行过的命令、结果、未执行项与剩余风险。

### 22. docs 阶段节点详细记录

- 创建 `docs/modular-rewrite/R09-research-ai-backend/R09-11-runtime-diagnostics.md`。
- 记录本节点实际做了什么、为何修改、修改前后职责、行为和依赖变化。
- 分别列出全部新增、修改、移动/重命名和删除文件；没有对应类型时明确写“无”。
- 文件清单必须与本节点真实 `git diff --name-status` 和最终工作区一致。
- 记录公共接口、DTO、Schema、数据格式、配置、错误语义、日志、UI 行为和模型保护资产是否变化。
- 记录实际执行的验证命令、环境、结果和报告路径；未执行项必须写明原因、替代验证和剩余风险。
- 记录入口切换、旧实现清理、关键设计决策、计划偏差和回退方法。
- 更新 `docs/modular-rewrite/R09-research-ai-backend/README.md` 中本任务的状态、记录链接和门禁结果。
- 节点记录及阶段索引未完成时，本任务只能停留在 `VERIFYING`，不得改为 `DONE`。

### 23. 完成标准

- 目标职责已由唯一新模块承担。
- 旧入口和旧实现已删除。
- 最小验证与阶段回归均通过。
- README 与实际状态一致。
- `R09-11-runtime-diagnostics.md` 已创建并与实际变更、验证结果一致。
- 阶段 `README.md` 已更新本任务状态和记录链接。

---

# 阶段级验证矩阵

| 验证层级 | 必须执行 | 通过条件 |
|---|---|---|
| 协议 | formal/plain contract tests | 链路不串线 |
| 安全 | secret redaction tests | 零明文泄漏 |
| 取消 | concurrency tests | 仅取消目标请求 |
| 会话 | persistence integration | 顺序与归档一致 |

# 阶段出口门禁

- `docs/modular-rewrite/R09-research-ai-backend/README.md` 已完整索引全部节点记录。
- `R09-stage-completion.md` 已创建并确认本阶段真实变更、验证、限制和回退点。
- 正式研究与普通问答后端完全分离。
- 取消、重试和诊断可追踪。
- R10 可进入 READY。

# 阶段提交与回退

- 阶段内每个可独立验收的任务保留原子提交；阶段完成提交建议为 `rewrite: complete R9 research-ai-backend`。
- 只允许回退到最近一个通过全部门禁的提交。
- 不得通过保留双实现代替可回退提交。

# 阶段完成记录要求

本阶段所有 Atomic Task 完成并通过阶段回归后，创建：

```text
docs/modular-rewrite/R09-research-ai-backend/R09-stage-completion.md
```

必须使用以下结构：

```text
# R09 阶段完成记录

## 1. 阶段目标与完成结论
## 2. 已完成节点索引
| 任务 ID | 实施记录 | 完成状态 | 最小验证 |

## 3. 实际新增文件总表
## 4. 实际修改文件总表
## 5. 实际移动或重命名文件总表
## 6. 实际删除文件总表
## 7. 最终目录与职责边界
## 8. 最终调用流、数据流和状态所有权
## 9. 公共接口、DTO、Schema、数据与配置变化
## 10. 保持不变的兼容行为
## 11. 旧实现、重复实现和临时路径清理结果
## 12. 阶段级验证与真实结果
## 13. 未执行验证、环境阻塞和剩余风险
## 14. 根 README、阶段 README 与架构文档同步
## 15. 阶段回退点与回退步骤
## 16. 出口门禁逐项结论
## 17. 下一阶段唯一 READY 任务
## 18. 订正记录
```

阶段完成记录必须引用本阶段每个节点记录，不得只重复任务书中的计划。缺少任何节点记录、真实文件总表、验证结果或回退信息时，本阶段不得标记为 `DONE`。
