# R11-08 AI Workspace 与 OpenAI Profiles Commands

状态：`VERIFYING`（首轮Clippy失败已修复；修复自身完整Windows待验，下文首轮记录保留历史）。

R11-01～07 DONE；08 AI Workspace/OpenAI Profiles Commands已实施VERIFYING，09～10 BLOCKED，R11 IN_PROGRESS。用户2026-10-23“收尾07开始08”，继续唯一rewrite/r11-tauri-commands；基线b7c2eb132a3803b8aa514b685ed097966b4e843f为07五文档[skip ci]收尾，生产源码同自身完整Windows成功5b779b9。

### 1. 目标

将AI问答和兼容API配置的13条公开命令接到唯一具名传输目录，清楚区分发送编排、校验/会话/上下文/网关/回答投影；原真实服务、取消与凭据所有者保持，不添加新问答功能或绕过现有政策。

### 2. 现状与来源

旧api_workspace.rs含6命令、原发送DTO及6私有helper，单一发送函数混合校验/会话读写/上下文/网关/日志/回答组装；旧openai.rs含7配置/示例解析命令。原Application会话/取消租约、Gateway双协议、OpenAiProfileStore及Windows凭据链已有唯一owner，本节点不重做它们。

### 3. 目标文件与目录

commands/ai_workspace根2（预设/取消）、sessions3、profiles7、send/command1，共13命令；4纯mod、8发送支持职责（request/validation/session/metadata/content/gateway/response/workflow）与4测试文件，共29新Rust。commands.rs为唯一既有Rust修改，30格式文件。不是将旧435行整体装入新coordinator。

### 4. 文件职责边界

12简单命令保留完整正文；send/command只借原State并把原DTO交workflow。request唯一保存原公开DTO；validation只输入校验和规范化，内部ValidatedMessage<'a>借用原字符串、携原预设/UUID结果，是有实际用途的无serde私有校验结果，不是新wire DTO/共享State。session只原会话绑定和创建，metadata/content纯处理元数据/受限上下文，gateway持原usage/请求/脱敏日志/执行租约，response纯映射原回答草稿；workflow只按原顺序连接这些阶段和原消息/上下文服务。

### 5. 输入

13原公开签名保持；SendApiWorkspaceCommand的derive/全部字段/serde(default)/optional/原始字符串与未知字段兼容精确不变。原DTO通过send与根显式导出，发送命令直接使用crate::commands::SendApiWorkspaceCommand兼容路径；不新增公开请求ID或字段。配置Draft/ProfileId与解析preferred_protocol原样交原存储/解析器。

### 6. 输出

原SessionDetail/SessionRecord/Preset、ProfileSummary/ProfilesState/TestResult/ApiExampleParseResult及String错误保持。原assistant草稿内容/response_id/model_id/token计数、末次attempt费用（None仍显式null）、plain_text/legacy:false、空citations/attachments与无提案/文件数组保持。

### 7. 允许依赖

原Tauri State、AppState、UUID helper，原Application预设/会话/用量与AiWorkspaceRequestLease，原Domain DTO、Gateway纯文本类型和OpenAiProfileSummary、serde/json/Uuid；没有新增外部API/版本行为。沿已完整Windows证明Tauri2.11.1/macros2.6.3/CLI2.11.4双宏出口，无新的Context7问题。

### 8. 禁止依赖

命令不新增PG池/SQL/新凭据存储/模型/服务实例、取消map/锁/令牌/worker/线程/重试/时钟；纯转换不I/O或await。无依赖/迁移/runner/workflow/target/数据库专项入口。AI问答仍禁联网工具、写库提案、附件/文件生成；P4正式研究边界保持。

### 9. 状态所有权

AppState、Application注册表/lease、OpenAiProfileStore、Windows凭据和原日志owner全部字节冻结；状态仍19。state-ownership只将旧openai命令单文件writer展开为实际save/set_active/delete/clear_key/test五文件，owner/scope/forbidden及其余状态精确不变。内部ValidatedMessage只随本次调用存在，无持久或跨请求共享状态。

### 10. 副作用边界

原Service/Ports/PG负责会话/消息事务；配置仍原Store与Windows Credential Manager，Gateway仍原execute_plain_text。gateway仅汇集原请求/日志/租约边界，日志只有ID/协议/端点/字符数/耗时/状态等原字段，不能添加问题正文/原始响应/Schema。普通回答仍append_api_workspace_assistant_bundle(assistant_message, vec![], vec![])，不产生提案或文件。

### 11. 异常路径

原先预设→空配置→空问题→拒附件→请求UUID→比赛UUID→实体规范化/UUID→实体成对/类型的优先级保持。活动会话仍依次检查status、profile、preset、match和entity，原中文错误/空格不变。取消仍trim/空ID错误并委托原registry.cancel；会话/比赛/实体可选UUID仍map/transpose，非法Some不吞错，raw DTO保持到原规范化边界。

### 12. 并发/异步/生命周期

workflow在校验后、profile/gateway和会话副作用前登记原唯一lease。准备失败/中止沿原Drop清理；成功移交gateway，execute_plain_text仍借registration.cancellation()，await返回后先drop(registration)，再match成功/失败并记录完成/取消/错误。无新token/map/重复owner，原Application8项取消生命周期覆盖和全部测试保持；GUI请求/过期结果/监听器保持。source等价不当作实机并发验证。

### 13. 兼容要求

171命令名称/全局注册顺序/参数/返回DTO、前端/contracts/schemas、配置格式/历史数据/错误/日志、0.23.0、依赖/锁/workflow/模型保持。公共集合SHA256 dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669。

物理组13→12：api_workspace6+openai7归ai_workspace13，保持原两组相对命令顺序和前端字段，其他11组精确不变；bootstrap注册171字节冻结。定义映射SHA7e86fb076698f679f57d690ea174a43e061b8bc65048823cdeed654c0315c64c；module-boundaries同步新root，state只writer路径、Domain只实际使用位置刷新。历史v3/Profile契约artifact原路径保留，由原验证器显式映射到真实新制品，不改契约或迁移哈希。

### 14. 实施步骤

fresh核07精确完整Windows/步骤/日志/artifact并五文档[skip ci]收尾；先在既有AI Workspace门禁补传输/流程约束，缺新目录时确认失败。按原语句分出真实职责，保留12薄命令正文，接新入口并删除2旧owner，补原Tauri target内纯边界测试及实际reader；等价、格式、架构、隔离探针/保护通过后沿原Windows验收。

### 15. 切换入口

commands.rs用ai_workspace替换api_workspace并退休openai模块/导出，Rustfmt按新名排根声明/导出；bootstrap/command_registry.rs原171注册不动。4mod只显式导出每命令/双宏及原Send DTO，send/mod只登记8支持职责、command和cfg(test)模块，不承载流程。原公开DTO有真实命令使用，避免测试专用或未使用生产re-export。

### 16. 删除清单

删除commands/api_workspace.rs与commands/openai.rs，13命令/原DTO/6helper均唯一新owner，无旧转发/重复逻辑/State/取消表/凭据层。整文件逐字移动/重命名：无；按原职责迁移并组装真实阶段。

### 17. 最小验证

最终82/82原只读源码、完整npm run verify:architecture、30现存变更Rust独立Rustfmt1.88 --check、61等价、隔离30破坏拒绝/6守卫恢复正向、主247源码/清单/守卫SHA256不变、18保护资产/171命令12组/46迁移18PG静态及git diff --check通过。

61包括13完整签名/12薄正文/原DTO/6helper、校验/会话/网关/回答片段和展开后完整发送流程逐token等价、全生产字符串多重集、冻结范围/11组/State writer及5原守卫逆变换。新增12 Tauri unit，原单测迁移0，预期Desktop58（46+12）/App230/PG259/Gateway112unit+41contract须08自身Windows；这些静态检查不计动态测试。

### 18. 阶段回归

原Public Platform CI执行完整frontend/contracts/TypeScript/Vite/17视口、fmt/locked Clippy workspace all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动与日志；12新unit只在既有target由Windows执行。82源码排除原Windows进程入口和五浏览器入口；未执行非Windows Cargo/check/Clippy/build/test/前端构建/浏览器/loopback。

### 19. 失败停止条件

08自身完整Windows成功前VERIFYING，09～10 BLOCKED；保护/契约/最小验证失败继续修复同一分支，不能继承07成功。实际Windows job首次in_progress后停止全部CI查询，告知约20～30分钟并等用户返回。

### 20. 回退点

b7c2eb132a3803b8aa514b685ed097966b4e843f，生产源码同07完整Windows成功5b779b9。按节点提交受控revert命令/清单/守卫/测试/记录，不硬reset、手工复制旧owner或动历史数据/凭据；回退源码亦须自身Windows。

### 21. 根 README 摘要记录

同步13公开命令/发送职责/原租约与凭据保持、171兼容/12组、12单测预期/静态结果与08自身待验，链接本记录；01～07 DONE、08 VERIFYING、09～10 BLOCKED、R11 IN_PROGRESS，不创建阶段完成记录。

### 22. docs 阶段节点详细记录

本记录、索引、任务书、README与TESTING同步，01～07节点记录实施中字节冻结。5既有守卫原全部断言逆变换保持：AI Workspace补真实传输/流程/纯出口/测试声明，其他reader改读实际命令/发送职责；v3和Profile旧artifact显式映射新制品，Profile额外核实际5writer，正式command verifier冻结。未删除功能或关闭必要检查。

首次scratch guard生成脚本变量复用，误将JS写入response_tests.rs；Rustfmt拦截、未推送。已修正显式目标路径、恢复原12测试并重跑82源码/完整架构/30格式/61等价/隔离检查；核对器还纠正了网关片段末尾匹配与Rustfmt根排序口径，没有为此更改生产语义。后续探针只在repo外独立副本，不在交付工作区做破坏性检查。

30探针覆盖旧owner/根登记/双层宏/纯出口、原参数/await/取消/签名、附件/UUID/实体/会话绑定、登记/历史/上下文、原令牌与drop、日志事件/泄露、历史/上下文/标题边界、费用/无提案、测试cfg/配置writer/注册。全部拒绝，6守卫恢复正向，主247指纹不变。

### 23. 完成标准

13唯一命令/完整发送阶段、旧owner清理、原target测试与记录/静态验证完成；08 VERIFYING，09～10未开始，R11未完成。真实API提供方/用户凭据/会话PG/取消端到端，以及历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归与继承model.runs/0041删除风险仍最终封包待验；ignored、公开unavailable stub和静态指纹不计真实PASS。

## 实际能力与单测

| 位置 | 能力 | 数量 |
|---|---|---:|
| 根 / sessions / profiles / send/command | 预设取消 / 会话 / 配置 / 发送 | 2 / 3 / 7 / 1 命令 |
| send/request、validation、session、metadata、content、gateway、response、workflow | 原wire与校验、绑定、纯内容、网关边界、回答与编排 | 8 职责 |
| send/request_tests | 默认/未知字段/原字符串、必填与错误wire类型 | 2 unit |
| send/validation_tests | 校验优先级、ID/实体、元数据、Unicode标题 | 4 unit |
| send/content_tests | 先过滤取末20、Unicode 20000/40000边界、只读指令 | 4 unit |
| send/response_tests | 原回答字段/用量/空集合、末次费用/显式null | 2 unit |

## 实际验证与未执行范围

原82read-only node、npm run verify:architecture、node scripts/verify_protected_assets.mjs、node scripts/verify_command_contract.mjs、node scripts/verify_database_baseline.mjs、30文件独立Rustfmt1.88 --edition 2021 --config skip_children=true --check、scratch/r1108_compare.py和隔离副本r1108_probes.py、git diff --check实际通过。报告r1108-static.json、r1108-architecture.log、r1108-equivalence.json、r1108-negative.json、r1108-primary-fingerprints.json在repo外scratch，可从源码复建；不新增持续runner。

Domain365/129声明源/301映射、scan1464→1491；sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，usageDigestcc2ff0a54510e70b3a690cf891594405f95634c23410c918f2cb50f3a99a67d8。App422/43Ports19files/PG直接38/State19不变；保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57与DB aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

未发送真实用户API请求/操作用户Windows密钥，也未实跑会话PG/取消UI端到端；原真实服务/网关/凭据/Lease链路保留接通，实际效果仍待验。Mermaid Chart已更新校验→原lease→profile/session→用户消息/只读上下文→Gateway/脱敏日志→释放lease→回答事务及失败Drop的中文实际链路；Create State沿足球稳定UUID96f03270-c236-46de-99fa-db85d2fbf4ce续接，无子代理。

## 首轮全部实际文件清单（ffbaedc）

基于b7c2eb132a3803b8aa514b685ed097966b4e843f，no-renames口径30A/14M/2D，共46路径。29新Rust（13命令、4纯出口、8发送职责、4测试文件），既有Rust修改仅commands.rs；整文件逐字移动/重命名无，按真实职责迁移/组合原完整语句。

### 新增（30）

- `docs/modular-rewrite/R11-tauri-commands/R11-08-ai-workspace-and-openai-profiles-commands.md`
- `src-tauri/src/commands/ai_workspace/cancel_api_workspace_request.rs`
- `src-tauri/src/commands/ai_workspace/list_api_workspace_presets.rs`
- `src-tauri/src/commands/ai_workspace/mod.rs`
- `src-tauri/src/commands/ai_workspace/profiles/clear_openai_profile_key.rs`
- `src-tauri/src/commands/ai_workspace/profiles/delete_openai_profile.rs`
- `src-tauri/src/commands/ai_workspace/profiles/list_openai_profiles.rs`
- `src-tauri/src/commands/ai_workspace/profiles/mod.rs`
- `src-tauri/src/commands/ai_workspace/profiles/parse_openai_api_example.rs`
- `src-tauri/src/commands/ai_workspace/profiles/save_openai_profile.rs`
- `src-tauri/src/commands/ai_workspace/profiles/set_active_openai_profile.rs`
- `src-tauri/src/commands/ai_workspace/profiles/test_openai_profile.rs`
- `src-tauri/src/commands/ai_workspace/send/command.rs`
- `src-tauri/src/commands/ai_workspace/send/content.rs`
- `src-tauri/src/commands/ai_workspace/send/content_tests.rs`
- `src-tauri/src/commands/ai_workspace/send/gateway.rs`
- `src-tauri/src/commands/ai_workspace/send/metadata.rs`
- `src-tauri/src/commands/ai_workspace/send/mod.rs`
- `src-tauri/src/commands/ai_workspace/send/request.rs`
- `src-tauri/src/commands/ai_workspace/send/request_tests.rs`
- `src-tauri/src/commands/ai_workspace/send/response.rs`
- `src-tauri/src/commands/ai_workspace/send/response_tests.rs`
- `src-tauri/src/commands/ai_workspace/send/session.rs`
- `src-tauri/src/commands/ai_workspace/send/validation.rs`
- `src-tauri/src/commands/ai_workspace/send/validation_tests.rs`
- `src-tauri/src/commands/ai_workspace/send/workflow.rs`
- `src-tauri/src/commands/ai_workspace/sessions/archive_api_workspace_session.rs`
- `src-tauri/src/commands/ai_workspace/sessions/list_api_workspace_sessions.rs`
- `src-tauri/src/commands/ai_workspace/sessions/mod.rs`
- `src-tauri/src/commands/ai_workspace/sessions/read_api_workspace_session.rs`

### 修改（14）

- `README.md`
- `architecture/command-contract.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `architecture/state-ownership.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/README.md`
- `scripts/verify-ai-workspace-service.mjs`
- `scripts/verify-api-runtime-diagnostics.mjs`
- `scripts/verify-api-workspace-research-control.mjs`
- `scripts/verify-api-workspace.mjs`
- `scripts/verify-openai-profile-ui.mjs`
- `src-tauri/src/commands.rs`

### 删除（2）

- `src-tauri/src/commands/api_workspace.rs`
- `src-tauri/src/commands/openai.rs`


## R11-08 首轮Clippy失败修复（2026-10-23，VERIFYING）

精确首轮ffbaedc11049e040e42282aa476b79d680be49c1 / [Windows run39971471528](https://github.com/uniquenesssta/123/actions/runs/39971471528) / job120474983650为FAILURE，run于2026-10-23 22:46:44北京时间完成。完整日志唯一Rust错误为send/gateway.rs:32的clippy::uninlined_format_args；lib与lib test均因此停止。架构及前端契约/类型/生产构建/17视口已通过；工作区单测（含新增12项）、Windows release/MSI/NSIS/启动没有完成，不能以首轮前端成功或静态检查替代。

根因：职责提取前format!参数含session.id字段访问；提取后变成session_id普通变量，原混合表达式变为可全部内联的变量，触发项目-D warnings下的Clippy门禁。生产只将format!("ai-chat:{}:{}", session_id, request_id)改为format!("ai-chat:{session_id}:{request_id}")；Display顺序/分隔符/UUID与request_id值不变，不修改trace政策、日志字段、发送/租约/配置/数据/DTO/错误或行为。

既有verify-rust-source-hygiene.mjs新增该路径的普通参数形式拒绝断言；先对首轮源码执行并确认失败，修复后通过。没有allow属性、降级告警或放宽门禁。原unit及12项新增unit所有字节保持，无新增/删除/迁移测试；预期Desktop58/Application230/Persistence259/Gateway112unit+41contract仍须本修复自身精确Windows确认。

实际82/82原只读源码、完整npm run verify:architecture、gateway.rs独立Rustfmt1.88 --edition 2021 --config skip_children=true --check、git diff --check通过。沿原61等价核对，仅对这一处已声明等价格式表达式做规范化；原13签名/12薄正文/DTO/6helper/完整发送展开及其他生产字符串保持。fix相对ffbaedc仅一处Rust替换，业务/模型/UI/依赖/所有原测试/命令契约/边界/19状态清单字节冻结；Domain仅rustUsageDigest刷新为ded94c69cc93a5fc02d563e94103be51fe41aacb973cfc53e109cd7c84b5e98a，365/129/301及scan1491/sourceDigest保持。

3破坏探针（首轮写法、多行/尾逗号写法、allow掩盖）仅在repo外隔离副本执行，全部拒绝；2既有守卫恢复正向，主248源码/清单/守卫SHA256保持。验证器的继承等价执行使用隔离命名空间，避免临时变量覆盖后续路径；首次scratch路径变量被覆盖只导致副本路径校验失败，未改生产目录，修正后全部通过。报告r1108f-static.json、r1108f-architecture.log、r1108f-equivalence.json、r1108f-review.json、r1108f-primary-fingerprints.json在repo外scratch。

Context7查询官方Clippy文档核对uninlined_format_args与默认allow-mixed-uninlined-format-args；实际项目Rust1.88.0和该版本Windows诊断/建议为对应版本证据。没有升级依赖或新增API。此为单表达式修复，架构/真实数据流不变，沿08既有Mermaid图；Create State保存本次失败、修复与待验状态。

原08最初61等价/全生产literal保持叙述属于首轮实施历史；本修复当前允许变化的是格式表达式字面写法，输出内容保持，其余等价检查仍严格执行。首轮46路径清单保留其提交事实；当前修复相对ffbaedc为8M/0A/0D，累计相对08基线b7c2eb1为30A/15M/2D，共47路径（仅额外加入原verify-rust-source-hygiene.mjs）。不把首轮清单误称为当前累计46。

08仍VERIFYING、09～10 BLOCKED，修复沿原完整Windows CI；实际Windows job首次in_progress后停止全部CI查询，提示约20～30分钟并等用户返回。真实API/用户凭据/会话PG/取消端到端、历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归和继承model.runs/0041删除风险仍最终封包待验；ignored/公开stub/静态hash不是这些真实PASS。回退可受控revert本修复提交；ffbaedc是已知失败点，最近同阶段已验前置源码仍07成功5b779b9（docs基线b7c2eb1），不得把回退到ffbaedc称为可验收完成。

### 本次修复全部修改（8）

- `README.md`
- `architecture/domain-type-inventory.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/R11-08-ai-workspace-and-openai-profiles-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/README.md`
- `scripts/verify-rust-source-hygiene.mjs`
- `src-tauri/src/commands/ai_workspace/send/gateway.rs`

新增、删除、移动/重命名：无。
