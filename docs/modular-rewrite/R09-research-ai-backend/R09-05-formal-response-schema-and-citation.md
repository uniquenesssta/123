# R9-05 Formal Response、Schema 与 Citation 实施记录

状态：`VERIFYING`。用户2026-10-11“收尾04开始05”授权。uniquenesssta/123、0.23.0，唯一阶段分支rewrite/r9-research-ai-backend；04修复cb3d2ca/run38055651688/Windows job114223464446全SUCCESS并DONE，五文档收尾基线/回退 `53e49f86f56f5766d85a0c8f938b47abf2a4ce34`、tree `3e6739375846c3eb8765e2a2c60d74aebe056827`，源码与04成功相同。06～11BLOCKED，R9整体IN_PROGRESS。

### 1. 目标

正式Responses严格解析、原ResearchOutput解码/Schema语义和引用关联各归唯一职责；普通问答/协作兼容解析不依赖正式研究私有实现。

### 2. 现状与来源

原response.rs混合正式结构解析、Plain/Structured兼容解析、引用/用量wire解码和提供方错误；validation.rs混合公开输出校验、事实值/主体/缺失、来源关联及URL/时间政策，client还实现parse_and_validate。基线原Gateway60 unit/25 contract已由04精确Windows验证。

### 3. 目标文件与目录

formal_research/response/{parse,validation}.rs持正式解析与纯组合；schema/{decode,validation}.rs持严格类型解码与公开输出校验；citations/validation.rs持来源索引/事实引用关联。共用引用wire、响应字段/用量和事实错误映射分别归根citations.rs、response_fields.rs、validation_error.rs；真实共用调用方决定位置，不让兼容问答依赖正式目录。

### 4. 文件职责边界

各mod仅登记/显式export。response协调函数只调用严格解析和带原上下文的输出校验；Schema不解析HTTP/轮询，引用wire不执行URL白黑名单或事实政策。原validate_url/domain_matches/validate_time继续root validation.rs，政策实现留06，不提前开始下一节点。

### 5. 输入

原TransportResponse/status/provider_request_id/Value；原借用GatewayRequest/SourcePolicy、ResearchOutput和公开ValidationContext，后者所有七个字段与类型保持。引用关联借用citations/sources/policy，避免引用模块依赖Schema上下文所有者。

### 6. 输出

原GatewayResponse/ResearchOutput/ValidationContext、原GatewayError或Ok；原raw_response/provider metadata/usage/search counts/citation位置和来源顺序不变，无新公开接口或DTO。

### 7. 允许依赖

原serde1.0.228/serde_json1.0.150、chrono0.4.45、url2.5.8、std和现有Gateway类型。正式Schema与引用调用原URL/时间政策和具名错误映射，不引入JSON Schema解释器或新依赖。

### 8. 禁止依赖

新纯职责不认识transport/ApiKey/provider/tokio/锁/sink/数据库/Application/Tauri/UI；不执行真实API、凭据读取、日志或持久化。普通问答及协作仍调用原兼容解析，保持原结构化降级和容错来源政策。

### 9. 状态所有权

无新生产跨请求状态、缓存或配置副本；Gateway仍持原config/transport/provider/Semaphore与CircuitBreaker。所有纯响应函数借用本次输入或消费本次Value，无新的生产State。

### 10. 副作用边界

HTTP和后台状态/轮询/取消留原client/transport；解析、Schema和引用校验不写事实库。失败attempt的原始响应、provider ID/status、response_id与error仍由原Gateway/sink记录。

### 11. 异常路径

HTTP错误先于身份/结构，结构错误先于refusal，refusal先于严格JSON解码；正式只接受一个output_text，不使用顶层Plain fallback，不吞掉非法引用。Schema保持match/schema/cutoff、原来源索引先于256上限、事实/主体/值/时间/引用及缺失覆盖原顺序；原SchemaValidation/SourcePolicy两类恢复建议与wire错误分开保留。

### 12. 并发/异步/生命周期

新生产模块全部纯同步，无新async/Mutex/监听器。同步completed、后台poll完成和resume三路各传其实际response/completed到同一parse_and_validate；原取消、超时、许可scope、retry/fallback、attempt与远端后台生命周期保持。

### 13. 兼容要求

正式Responses边界、五层deny_unknown_fields、256原子事实/120 ASCII fact_key、主体七类型/200字符、值六kind、原验证状态/缺失覆盖保持。RootContext及公开函数签名/名字不变，类型/JSON Schema/配置/profile/UI/模型/依赖/锁/迁移/版本均不变。原Plain/Structured代码、容错引用解析、用量别名/饱和及提供方错误保持，不修改外部协议。

### 14. 实施步骤

已以04成功同源码收尾作为基线，按真实职责迁出17个原函数；从原严格输出语句抽出单一decode seam，原语句与错误映射保持。原1项正式响应测试/4项公开校验测试迁到实际owner，原其余55单测/25契约保持；22新unit/4新contract沿用原targets。既有兼容传输门禁补唯一owner、三路实际响应/严格边界及原政策守卫，六破坏探针拒绝恢复。

### 15. 切换入口

client直接导入formal_research::response::parse_and_validate，三路直调，无旧方法转发；crate根直接从formal_research::schema导出原validate_research_output/ValidationContext。正式和兼容各直调用唯一wire引用/用量实现，提供方错误仍原response.rs。

### 16. 删除清单

旧client::parse_and_validate、root response正式解析/七wire helper、root validation公开校验/三语义helper/两引用helper/两错误函数定义已移除。无整文件删除、复制备份或重命名；root response保留真正的兼容解析，root validation保留真正的三项政策，不是转发壳。

### 17. 最小验证

本地实际83/83现有JS源码门禁、完整npm verify:architecture、Rustfmt1.88源码check（17个变更Rust源）、18保护资产/171命令/46迁移18PG静态、diff、等价与6探针均PASS。动态源码预期Gateway82/contract29、Application126/Persistence175，尚需05自身Windows，不能记为运行PASS。

### 18. 阶段回归

复用原Public Platform CI/Windows Automated完整frontend/contracts/TypeScript/Vite/17视口、fmt/Clippy -D warnings/workspace tests、Windows release/MSI/NSIS/启动。当地不执行Linux/macOS Cargo/编译/测试/loopback/客户端动态；PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归仍最终新库待验。

### 19. 失败停止条件

05自身Windows未完整SUCCESS前保持VERIFYING；06～11BLOCKED。保护资产/公开契约或原静态门禁失败先修，不抑制Clippy、跳测试、放宽门禁或继承04结果。

### 20. 回退点

53e49f86f56f5766d85a0c8f938b47abf2a4ce34仅五文档收尾，源代码等于04精确cb3d2ca成功。受控revert本项及匹配清单/门禁/文档，不手工复制第二实现或覆盖用户更改。

### 21. 根 README 摘要记录

记录04 DONE/05 VERIFYING、响应/Schema/引用职责与真实共用边界、文件总数、原兼容行为、实际静态与Windows待验。

### 22. docs 阶段节点详细记录

本记录含所有24文件与真实A/M/D、契约/等价/新行为覆盖/工具证据/失败尝试/回退及延期。阶段索引05 VERIFYING、06～11BLOCKED，不创建R9阶段完成记录。

### 23. 完成标准

实现/静态/记录已完成，须05自身精确Windows完整SUCCESS后才能DONE；最终新库/Full/私有等延期保持显式待验，不以ignored或公开stub代替实跑。

## 完整实际文件清单

相对53e49f8基线含untracked的实际 **12A / 12M / 0D**，24文件；没有移动/重命名。

新增（12）：

- crates/research-gateway/src/citations.rs
- crates/research-gateway/src/response_fields.rs
- crates/research-gateway/src/validation_error.rs
- crates/research-gateway/src/formal_research/response/mod.rs
- crates/research-gateway/src/formal_research/response/parse.rs
- crates/research-gateway/src/formal_research/response/validation.rs
- crates/research-gateway/src/formal_research/schema/mod.rs
- crates/research-gateway/src/formal_research/schema/decode.rs
- crates/research-gateway/src/formal_research/schema/validation.rs
- crates/research-gateway/src/formal_research/citations/mod.rs
- crates/research-gateway/src/formal_research/citations/validation.rs
- docs/modular-rewrite/R09-research-ai-backend/R09-05-formal-response-schema-and-citation.md

修改（12）：

- README.md
- architecture/domain-type-inventory.json
- crates/research-gateway/src/client.rs
- crates/research-gateway/src/lib.rs
- crates/research-gateway/src/response.rs
- crates/research-gateway/src/validation.rs
- crates/research-gateway/src/formal_research/mod.rs
- crates/research-gateway/tests/gateway_contract.rs
- scripts/verify-api-compatible-transport.mjs
- docs/TESTING.md
- docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md
- docs/modular-rewrite/R09-research-ai-backend/README.md

删除：无；旧17定义已迁到真实唯一owner。配置/请求04/凭据/transport/resilience/types.rs、Application/Domain/Tauri/PG/UI、所有公共契约/Schema/依赖/锁/迁移保持。临时反向探针修改types.rs已逐字恢复，不在实际清单。

Domain扫描1121→1132，usageDigest `7dca548fa0c39319eb8f393a24a1823e8ffa705046dc89d86b1596dc6e4d13a9`；365/300/sourceDigest `217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd`、Application402/43Ports、19状态/PG根35保持。保护聚合 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`、PG静态聚合 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`保持。

## 行为覆盖与真实结果

原60 unit/25 contract保持，原1个正式解析测试和4个公开校验测试迁入生产owner，仅测试所在模块变化。

新增22 unit：正式解析5（严格envelope/不走Plain fallback、content/text/重复输出与错误优先、refusal/HTTP优先、引用顺序/首来源元数据/原始响应/用量、错误引用与搜索来源先于解码）；严格decode2（五层未知字段、必需字段/类型/时间/文本与围栏拒绝）；输出Schema8（身份优先、256/257原子事实、120 ASCII/raw身份/重复key、缺失覆盖/状态/冲突、七主体/Unicode200、六value kind与有限数、原验证状态/禁用文本映射、三事实时间前/相等/纳秒后）；引用关联2（fragment规范化及引用覆盖来源原次序、四需来源状态/两免来源状态/完整清单关联/规范化重复）；共用wire引用3（位置/标题/域名/optional offset原类型政策、URL缺席与非法host/HTTP不擅加政策、原必需字符串错误/空URL）；共用用量2（双协议alias原优先与饱和、缺席/无效主字段原零默认不擅自fallback）。

新增4 contract：七类正式响应失败保留一条attempt/provider status-ID/response_id/完整raw/error、零额外retry/fallback及原失败用量；正式坏引用拒绝但原协作容错/标准化保持；六provider状态守卫先于Schema解码；后台queued与resume两路分别验证成功/错误match同一校验、完整raw/citations/sources/usage/provider ID和一次GET/实际URL/凭据次数。均沿用原mock接口与contract文件，不调用真实API/数据库/凭据、不改全局环境；后台GET夹具直接completed，无实际定时等待。

源码预期 **Gateway82 / contract29 / Application126 / Persistence175**，本项Windows待验。源码/格式不能冒充编译或单测PASS。本地实际PASS为83现有静态/完整architecture/Rustfmt/保护/命令/DB静态/diff；报告 `/workspace/scratch/eb298ad5cdcb/r905-static-checks.json`、`r905-architecture.log`、`r905-equivalence.json`、`r905-negative-probes.json`、`r905-review.json`。

等价证据：17原函数按仅恢复调用路由/引用输入参数/严格decode表达式后tokens保持；剩余client与Plain/Structured/提供方函数全部tokens保持，三URL/domain/time政策原函数tokens保持；ValidationContext完整字段、原60单测/25契约、657条Gateway生产literal及原所有公开exports保持，policy_changes=[]。原types.rs字节保持，没有新JSON Schema或外部API。

6破坏探针：混用完成/恢复实际响应、重复output_text判定反转、ResearchOutput去掉deny_unknown_fields、256上限改257、引用membership不用规范URL、用量饱和改普通加法。全部拒绝且所有源恢复。

## 工具、偏差与继续位置

Context7核对serde严格解码，但所选官方文档集未提供1.0.228版本；另读官方serde-rs/serde **v1.0.228** serde_derive/src/de/struct_.rs确认deny_unknown_fields生成路径，结合原types五处标注，不以latest冒充锁版本。serde_json1.0.150/chrono0.4.45/url2.5.8未升级，原调用体保持。Mermaid Chart已展示真实正式解析→共用wire/严格decode→Schema/引用→原URL/时间及返回链；Create State保存前核实稳定足球model96f03270-c236-46de-99fa-db85d2fbf4ce，合成摘要误述跨平台不得替代本项目Windows约束或Git/CI。

最初等价工具把rustfmt排序后的pub use位置变化当成不等价，已改为比较同一公开出口及其余完整tokens，确认接口保持；首个响应路由破坏探针暴露仅总数检查可接受混用变量，已在原门禁分别限制resume/sync/background实际参数后六项全拒绝恢复。没有运行编译后错误被掩盖，也没有放宽既有门禁。

引用索引改为显式借用citations/sources/policy，使引用关联不依赖Schema上下文所有者；唯一严格decode只是抽出原语句，错误与Typed DTO原样。真实共用wire/错误映射放在根具名职责，避免Plain/协作绕入Formal或复制实现。root validation继续真实URL/domain/time实现，06后续按任务处理。

同一R9分支原Windows CI确认05精确head/Windows job开始即停止轮询，预计20～30分钟；05仍VERIFYING、06～11BLOCKED。下一轮先核实05自身结果，成功才收尾，失败只修完整相关链路。真实PG/历史四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4P7 Golden Master及继承model.runs/0041历史删除风险仍最终封包新库待验，ignored与公共unavailable stub不计PASS。
