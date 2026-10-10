# R9-04 Formal Research Request 实施记录

状态：`VERIFYING`。用户2026-10-10“收尾03开始04”授权。uniquenesssta/123、0.23.0、唯一阶段分支rewrite/r9-research-ai-backend；03精确d58aa38/run37960534145全SUCCESS并DONE，04基线/回退bbc9031a41e21c3b4c90831a1d2ec1ce2fdcb1a4为五文档收尾 `[skip ci]`，源码同03成功。05～11BLOCKED。

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

## 完整实际文件清单

相对bbc9031基线，含untracked新增、`git diff --no-renames --name-status`：**9A / 9M / 0D**，无整文件移动/重命名。

新增（9）：

- `crates/research-gateway/src/budget.rs`
- `crates/research-gateway/src/request_fields.rs`
- `crates/research-gateway/src/formal_research/mod.rs`
- `crates/research-gateway/src/formal_research/request/mod.rs`
- `crates/research-gateway/src/formal_research/request/validation.rs`
- `crates/research-gateway/src/formal_research/request/payload.rs`
- `crates/research-gateway/src/formal_research/request/budget.rs`
- `crates/research-gateway/src/formal_research/request/tests.rs`
- `docs/modular-rewrite/R09-research-ai-backend/R09-04-formal-research-request.md`

修改（9）：

- `README.md`
- `architecture/domain-type-inventory.json`
- `crates/research-gateway/src/client.rs`
- `crates/research-gateway/src/lib.rs`
- `crates/research-gateway/tests/gateway_contract.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md`
- `docs/modular-rewrite/R09-research-ai-backend/README.md`
- `scripts/verify-api-compatible-transport.mjs`

删除：无；上述7旧定义已从client移除，无文件复制备份/转发壳。移动/重命名：无。没有修改原凭据、transport、resilience、response、validation.rs输出消费、config.rs、types.rs、UI/Tauri/Application/PG/Domain/公共契约/Schema/依赖或锁文件。

Domain扫描1113→1121、usageDigest `076f74ed50609cd9d6988b8d395b04d018c50f7438cc471b733d2fcd4ede4428`；365/300/sourceDigest `217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd`、Application402/43Ports、19状态/PG根35/46迁移与18保护资产/171命令保持。

## 新增行为测试与实际验证

原47 Gateway unit/21 contract保持，新13 unit进入原target：

- 输入4：精确字符/字节边界及来源不修改；空白/超长/非ASCII Schema名称与完整错误；31/32事实、空/重复/100/101字符及raw身份保持；三类Schema根错误与输入错误优先/Schema版本和指令非空。
- 载荷3：可信静态instructions与不可信嵌套context隔离、trace/match/cutoff/事实元数据；根Schema投影/嵌套$id保留与两次构造不修改来源；同步/后台、store、白名单/无filters、两Token别名、reasoning/search/tool/include原政策。
- 正式预算3：每模型初次+retry、相同fallback去重与不同fallback成本；NaN/正负无限/负值、daily→monthly→per-request优先及原缺席价格政策；ASCII/Unicode相同字符量/更多字节的金额阈值及严格超限。
- 共用政策3：精确价格优先、最长dash前缀/非dash拒绝、返回原配置借用；原完整预算错误/recovery/metadata；两Token别名替换、其他字段保留与非object不改。

新4 contract通过公开入口核对：正式错误优先/Chat隔离在key/IO前停止；预算在key/IO前拒绝且CircuitOpen仍优先于预算；resume共享请求校验先于response_id与后台守卫；Extraction→fallback两次attempt8/9保持上下文/Schema副本、嵌套$id、研究字段与原来源不修改。使用原mock及本项计数provider，无真实API/凭据调用、不修改全局环境，无新增test target。

源码预期Gateway **60** / contract **25**、Application **126** / Persistence **175**，自身Windows尚待验。实际本地PASS为83/83现有静态、完整verify:architecture、Rustfmt1.88源码format/check、原兼容传输/Domain/命令/数据库静态/18保护资产与git diff --check；无非Windows编译/单测/客户端动态。保护聚合 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`、PG静态聚合 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`保持。

7破坏探针分别移除正式校验接入、31事实改32、不可信标志反转、根$id投影丢失、fallback去重反转、Token别名清理丢失、最长价格前缀改最短，原增强兼容传输验证器全部拒绝；所有源码逐字恢复、原门禁及Domain清单通过。无新回归框架/runner/workflow/数据库入口。

等价核对迁出7函数：校验完整tokens，构造/预算/估算/定价体在仅还原接收参数后tokens保持，两共享helper完整tokens保持；剩余client反向恢复入口路由后全部tokens保持，247条原生产literal、原47unit所有source/21contract及35原具名函数集合保持。公共lib导出保持，仅新增私有模块登记，生产政策变化=[]。

报告：`/workspace/scratch/eb298ad5cdcb/r904-static-checks.json`、`r904-architecture.log`、`r904-equivalence.json`、`r904-negative-probes.json`、`r904-review.json`。本地没有运行Clippy，Clippy及Rust运行以本项Windows为准。

## 工具、取舍与继续位置

Context7按Cargo.lock serde_json1.0.150核对Value clone/Map remove，但仅返回latest；已额外读取官方serde-rs/json v1.0.150的src/value/mod.rs和src/map.rs确认Clone/持有Map/Vec及remove实现，原clone/根remove原样迁移，无API升级或preserve_order feature变化。Mermaid Chart已展示正式执行/恢复、纯校验/构造/预算、共享政策及原IO链路。Create State仅保存本轮已核实足球稳定model96f03270-c236-46de-99fa-db85d2fbf4ce，摘要不替代Git及CI。

实际偏差仅参考目录按职责具体化：定价/预算错误与Token别名是真实跨调用方共用政策，不能放在正式目录让Plain/连接依赖正式私有实现，也不复制。原model_for_operation仍唯一Gateway路由；预算参数primary_model只是借用既有确定性选择，不增加IO/状态或改变拒绝优先级。响应解析/后台状态/实际成本和Plain预算继续原owner，后续05/07按任务处理，不把本项变成整个Gateway重写。

沿用Public Platform CI，确认自身精确head/Windows job开始后立即停止轮询，预计20～30分钟。下一轮先核实04自身结果，成功才收尾，失败只修相关完整链路；05～11仍BLOCKED，R9整体IN_PROGRESS。真实PG/历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master与继承model.runs/0041历史删除风险仍最终封包新库待验；ignored与公共unavailable stub不计PASS。


## 首轮 Windows 失败与修复（2026-10-10）

用户“未通过”指向本项首次代码，不收尾04或开始05。精确提交 `16b5d4e06442466e45553e0beebe668fba4bbe70`、tree `26569fb830bb7c0a2e768fac3955d8de074857e8`、同一R9分支，[run `38021521371`](https://github.com/uniquenesssta/123/actions/runs/38021521371) / [Windows job `114123347337`](https://github.com/uniquenesssta/123/actions/runs/38021521371/job/114123347337) **FAILURE**，完成2026-10-10 11:54:24北京。日志实际：

- 原架构、frontend/contracts/TypeScript/Vite与17视口通过，fmt/Clippy通过；Application126、Persistence175、Gateway60通过。
- gateway_contract实际24 PASS / 1 FAIL；新增 `formal_request_validation_and_protocol_errors_stop_before_credentials_and_io` 在原第1283行Gateway构造器unwrap收到InvalidConfiguration：“兼容 API请求端点与所选协议不一致”。该契约尚未到达身份/Schema/正式协议拒绝断言，不能算断言成功。
- release/MSI/NSIS/启动未执行，workspace整体失败；原18 broad PG仍ignored，真实数据库不记PASS。
- 验收记录 `D:\a\123\123\logs\windows-acceptance-20261010-034410.txt`；失败证据artifact `11657834137` / `windows-automated-delivery-evidence-16b5d4e06442466e45553e0beebe668fba4bbe70`，671 bytes，SHA-256 `a1c5979bb89af09306cb22042957b2fbedf0a53bb62b473c2f91ee131d21961f`。仅失败过程证据，不含成功交付。

根因：新测试从原Responses config()克隆后仅改api_protocol，显式request_endpoint仍为 `/v1/responses`；原GatewayConfig::validate先检查URL协议后缀，在进入execute前按正确政策拒绝。原3处Chat契约的端点配置均正确，审计本项4个新增契约没有第二处协议切换遗漏。修复在构造器前补该测试的显式 `/v1/chat/completions` 地址；保留原配置校验及正式协议守卫，不修改生产策略，不删除/放宽断言或用Clippy抑制。

原 `verify-api-compatible-transport.mjs` 增加该错误优先级契约的配置顺序、唯一端点赋值及身份/Schema/正式协议错误、零provider/IO断言守卫。缺失端点、错用Responses端点、构造器后才设置端点3破坏探针均被该守卫拒绝，并逐字恢复后通过。没有新runner/target/workflow/持续回归框架或数据库设施。

相对失败head实际 **8M / 0A / 0D**，无重命名，完整文件：

- `crates/research-gateway/tests/gateway_contract.rs`：仅新增一行有效Chat端点，原21契约及新增4项所有断言/测试数量不变。
- `scripts/verify-api-compatible-transport.mjs`：扩展原R9-04守卫。
- `architecture/domain-type-inventory.json`：用原生成器只刷新usageDigest为 `0b91bb6cd84c76569b6b6e4a67c70f3e690994372350566b67619fadbde9b90c`；365/300、1121扫描和sourceDigest保持。
- `README.md`：同步首轮实际失败及修复状态。
- `docs/TESTING.md`：同步精确Windows和本次验证边界。
- `docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md`：04仍VERIFYING，05～11BLOCKED。
- `docs/modular-rewrite/R09-research-ai-backend/README.md`：准确阶段状态和继续位置。
- `docs/modular-rewrite/R09-research-ai-backend/R09-04-formal-research-request.md`：完整根因/修复/清单/验证/回退记录。

所有生产Rust文件相对16b5d4e逐字保持。相对整个04基线bbc9031再次复核7迁出函数、其余client重路由tokens、247原literal、47原unit/21原contract及公开exports保持，生产policy_changes=[]。测试文件对失败head的精确差异仅上述一行，保留全部原断言及预计60/25/126/175数量。

本次实际本地PASS：83/83原静态门禁、完整npm verify:architecture、Rustfmt1.88源码check、18保护资产/171命令/46迁移18PG静态、diff、3破坏探针及等价。报告 `/workspace/scratch/eb298ad5cdcb/r904fix-static-checks.json`、`r904fix-architecture.log`、`r904fix-equivalence.json`、`r904fix-source-preservation.json`、`r904fix-negative-probes.json`。未运行Linux/macOS Cargo/编译/测试/loopback/客户端动态；修复自身Windows尚未完整通过，首轮局部PASS不能替代修复精确结果。

本次是既有测试配置遗漏，无第三方API/版本/架构变化；依据精确CI错误和原config校验修复，未重复查询Context7或绘制无变化架构。收尾保存前重新核实Create State稳定足球model；记录以Git/CI为准，模型合成摘要不替代精确证据。

修复回退点16b5d4e；整个04回退点仍bbc9031。使用同一R9分支/原Public Platform CI，核实修复精确head及Windows job开始后停止轮询，预计20～30分钟。04仍VERIFYING，05～11BLOCKED；下一轮核实修复自身结果，成功才收尾，失败继续修相关链路。真实PG/历史四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041历史删除风险仍最终封包新库待验，ignored与公共stub不计PASS。
