# R9-06 Source 与 Time Policy 实施记录

状态：`DONE`（精确d996a4d自身Windows全SUCCESS；下述VERIFYING为实施历史）。用户2026-10-11“收尾05开始06”授权。uniquenesssta/123、0.23.0，唯一阶段分支rewrite/r9-research-ai-backend。05精确aa70d2b/run38072987214/job114274071102全SUCCESS/DONE，五文档收尾基线/回退 `b6636fef9589e86ee59c173732a847fd92cb21ed`、tree `2cb6d893136938d065a317a48758aa2e5c63e7fb`；源码与05成功一致。用户已授权07 READY，08～11BLOCKED，R9整体IN_PROGRESS。

### 1. 目标

来源URL、域名允许/禁止、字段/内容限制与可选事实截止时间各有唯一纯规则owner，完整迁移正式响应消费链，保持政策兼容。

### 2. 现状与来源

05后root validation.rs仍持validate_url/domain_matches/validate_time；Schema还持禁用字段/内容的规范化和匹配。真实消费者仅正式Schema/Citation。配置校验、请求工具filters、Application事实审核与持久化分别属于既有owner，行为/接口未改。

### 3. 目标文件与目录

formal_research/policies/{source_policy,time_policy}.rs，mod仅登记和显式导出。Schema/Citation直接导入政策出口，crate根继续直接导出原公开Schema接口。

### 4. 文件职责边界

source_policy拥有原URL/domain与FactContentPolicy禁用字段/内容规则；time_policy拥有原可选UTC时间比较。FactContentPolicy仅为本次输出准备两个原小写列表，不是跨请求State、缓存或新配置。Schema保留值kind/实际载荷一致性，内容规则在形状校验之后原位置调用。

### 5. 输入

原&SourcePolicy、&str URL/字段、&ResearchValue；原field标签、Option<DateTime<Utc>>和原请求cutoff。借用本次输入，不读取墙钟、API或凭据。

### 6. 输出

原规范URL或Ok/原GatewayError。来源及内容错误继续SourcePolicy、未来时间继续SchemaValidation，原恢复建议和非retry语义保持；不改原响应/事实URL或raw_response。

### 7. 允许依赖

原url2.5.8、chrono0.4.45、std和既有Gateway类型/validation_error映射。未升级锁文件或增加crate/JSON Schema解释器。

### 8. 禁止依赖

政策不依赖transport/reqwest/tokio/key_provider/sink/锁/文件/数据库/Application/Tauri/UI，不做IO。Plain/Structured兼容路径不依赖正式政策，原容错/filters行为保持。

### 9. 状态所有权

原Gateway配置/transport/provider/许可/CircuitBreaker保持。新增具名内部FactContentPolicy只有本次校验的两个Vec，无静态、全局或持久状态，原配置不被修改。

### 10. 副作用边界

政策全部纯同步。HTTP/轮询/取消/attempt账本由原Gateway和sink执行，Application事实入库审核仍原owner；无重复来源治理副作用。

### 11. 异常路径

URL原解析→HTTPS→host→黑名单→白名单→fragment移除。匹配保留小写、配置trim、*.与www前缀、apex及点分子域；关闭https_only时保留原host/scheme政策，不擅自限制FTP等原可解析输入。禁字段/内容保留原小写substring、字符串列表空格join、raw term空白/空term行为。

### 12. 并发/异步/生命周期

政策无新async/监听器/锁/定时器；同步completed、后台完成、resume仍经05同一纯解析/校验入口。取消/预算/重试/许可/提供方状态原顺序保持。

### 13. 兼容要求

Schema match/schema/cutoff身份顺序、来源索引先于256上限、fact身份→禁字段→主体→状态→值形状→内容→published/observed/effective→引用原顺序保持。None及<=cutoff允许、纳秒>cutoff拒绝。公开SourcePolicy五字段、ValidationContext七字段、validate_research_output/root所有exports保持；类型/JSON Schema/配置/profile/UI/模型/迁移/版本不变。

### 14. 实施步骤

以05成功同源码文档收尾为基线，逐字迁3原函数；完整迁原禁词规范化/字段/内容块，FactContentPolicy为原两Vec具名owner。原82unit/29contract断言保留，仅原value校验测试调用适配新边界；新增8unit/2contract于原targets，原兼容门禁补政策接入/顺序/纯边界，7探针全部拒绝恢复。

### 15. 切换入口

Schema直接调用FactContentPolicy/validate_time，Citation三个实际URL消费处直接调用唯一validate_url。root lib删除mod validation，正式mod登记policies；无旧API/方法转发或第二实现。

### 16. 删除清单

整文件crates/research-gateway/src/validation.rs删除。Schema原禁词准备/字段匹配/内容匹配移除并切入source_policy；值形状仍原Schema owner。没有复制备份、移动/重命名或空出口壳。

### 17. 最小验证

本地实际83/83既有JS静态、完整npm run verify:architecture、Rustfmt1.88八变更Rust文件check、diff、保护资产/171命令/46迁移18PG静态、等价与7探针PASS。Gateway90/contract31、Application126/Persistence175只是源码预期，06自身Windows待验。

### 18. 阶段回归

复用原Public Platform CI/Windows Automated前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace tests、Windows release/MSI/NSIS/启动与日志扫描，无新runner/workflow/target/回归框架/DB。非Windows不执行Cargo/编译/单测/loopback/浏览器运行时；真实延期见下。

### 19. 失败停止条件

06自身Windows未完整SUCCESS前VERIFYING、07～11BLOCKED。静态/保护/公开契约失败先修，不抑制Clippy、跳测试或放宽门禁，不能继承05成功冒充06运行通过。

### 20. 回退点

b6636fef9589e86ee59c173732a847fd92cb21ed源码等于05精确aa70d2b成功。受控revert06源码及匹配清单/门禁/文档，不手工复制或覆盖用户更改。

### 21. 根 README 摘要记录

05 DONE/06 VERIFYING、来源/时间与形状/关联责任、真实文件清单和静态证据、Windows待验及最终新库边界。

### 22. docs 阶段节点详细记录

本记录保留23职责/契约字段、17真实文件、原行为等价与新增测试/工具/取舍/延期/回退。阶段索引06 VERIFYING、07～11BLOCKED，不创建R9阶段完成记录。

### 23. 完成标准

实现、静态和记录已完成；06自身精确Windows完整SUCCESS才DONE。ignored、公开stub和指纹不能代替真实PG/Full/私有运行。

## 完整实际文件清单

相对b6636fe，含untracked，初版f714acb为4A/11M/1D、16文件；含三轮修复当前实际 **4A / 12M / 1D**，17文件；移动/重命名：无。

新增（4）：

- crates/research-gateway/src/formal_research/policies/mod.rs
- crates/research-gateway/src/formal_research/policies/source_policy.rs
- crates/research-gateway/src/formal_research/policies/time_policy.rs
- docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md

修改（12）：

- README.md
- architecture/domain-type-inventory.json
- crates/research-gateway/src/formal_research/citations/validation.rs
- crates/research-gateway/src/formal_research/mod.rs
- crates/research-gateway/src/formal_research/schema/validation.rs
- crates/research-gateway/src/lib.rs
- crates/research-gateway/tests/gateway_contract.rs
- scripts/verify-api-compatible-transport.mjs
- scripts/verify-rust-source-hygiene.mjs
- docs/TESTING.md
- docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md
- docs/modular-rewrite/R09-research-ai-backend/README.md

删除（1）：crates/research-gateway/src/validation.rs。原client/response/request/config/DTO/errors/transport/resilience/credentials、Application/Domain/Tauri/PG/UI、工作流/版本/依赖/锁/公共契约/Schema/模型不变。

Domain扫描1132→1134，usageDigest `33b0d1b4baf260934d849a561f30f3c48f8061cc3e40214552486c22fc177d3f`；365/300/sourceDigest `217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd`、Application402/43Ports、19状态/PG根35保持。18保护聚合 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`、46迁移18PG静态聚合 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`保持。

## 行为覆盖与证据

新增6 Source unit：域名大小写/前缀/apex/多级子域/点分边界；原parse/HTTPS/host/block/allow优先及非retry/恢复建议；fragment移除与原path/query/port/credentials规范化、查询差异/配置不改；空白名单/关闭HTTPS原scheme政策与黑名单仍生效；禁字段/内容case/substring/字符串列表join/配置不改；raw term空白与空term原行为。

新增2 Time unit：三字段None/截止前/相等/纳秒后及原错误恢复；不同RFC3339偏移同UTC瞬间与纳秒精度，不读墙钟。新增2 contract：12类同步政策失败/优先（block/allow/HTTPS/禁字段/禁内容/三时间/值形状优先/主体优先/身份优先/来源索引先于上限），每类一条attempt、一load/一请求、原raw/provider status-ID/response-ID/error、零额外retry/fallback和原失败用量；后台queued与resume各三情形（等UTC cutoff和fragment关联成功/纳秒后失败/内容失败），原事实/引用/来源URL与raw保持，一次GET正确URL/凭据次数/POST次数/原用量。沿用现有mock，不调用真实API/数据库/凭据/实际等待，无全局环境修改。

等价报告：3原URL/domain/time函数tokens逐字保持；禁词准备、字段/内容块tokens保持；恢复迁移块后完整原Schema协调函数、值形状、原82unit/29contract断言/其余Gateway文件、所有公开出口及Context保持；657生产literal相同、policy_changes=[]。7探针（点分域名边界、fragment、时间相等、字段入口、值形状/内容优先、黑名单优先、raw term不得trim）均拒绝恢复。

报告 `/workspace/scratch/eb298ad5cdcb/r906-static-checks.json`、`r906-architecture.log`、`r906-equivalence.json`、`r906-negative-probes.json`、`r906-review.json`。本地实跑为源码/静态，不是Rust编译/动态测试。90/31预期不能计06 Windows PASS。

## 工具与偏差

Context7先resolve /servo/rust-url再查询fragment规则，文档未提供2.5.8版本；另读官方servo/rust-url **v2.5.8** url/src/lib.rs（blob f1558682b6287d0cba1cd93d84cee6cf6cbeaf44）核对set_fragment(None)原实现，与代码结合，未以main文档冒充锁版本。Mermaid Chart展示真实正式响应→Schema/引用→来源/内容/时间政策及兼容共用解码；Create State保存前确认足球稳定model96f03270-c236-46de-99fa-db85d2fbf4ce，合成摘要不替代Git/CI/Windows约束。

临时格式工具首次缺PATH/driver库，定位既有Rustfmt1.88并指定其本地LD_LIBRARY_PATH后format/check通过，无仓库工具链修改。等价辅助脚本首次误取函数返回类型Self为构造块终点，修正token范围后证据通过，生产代码未改。无编译失败被掩盖或新门禁框架。

比参考目录多迁Schema禁字段/内容块是完整SourcePolicy消费边界必需；FactContentPolicy只命名原两Vec准备职责，不新增策略/状态/泛化配置。配置合法性、工具filters、Application审核治理不迁出其原owner。原HTTPS关闭及空禁词等行为按已验证基线保留，没有借重构改变规则。

同一分支推送原Windows CI，确认本项精确head/Windows job开始即停止轮询，预计20～30分钟；06 VERIFYING、07～11BLOCKED。下轮先查自身结果，失败先修相关链路。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7 Golden Master及继承model.runs/0041删除风险保持最终封包新库待验，ignored与公共unavailable stub不计PASS。

## 首轮 Windows 失败修复（2026-10-11）

用户“未通过”后核实：精确初版 `f714acbcf382a42840c55abb9c26d5f0c593dc2e` / tree `a3f8c1d34b84cbac9481b05b260fe241c7af5005`，[run38108158888](https://github.com/uniquenesssta/123/actions/runs/38108158888) 与 [job114377958564](https://github.com/uniquenesssta/123/actions/runs/38108158888/job/114377958564) 均FAILURE。2026-10-11 11:28:24北京在Rust门禁停止，source_policy.rs:77 `uninlined_format_args` 被 `-D warnings` 提升为错误。前端/contracts/TypeScript/Vite/17视口已通过，Rust工作区测试、release/MSI/NSIS/启动尚未执行；90/31不能记为PASS。

原表达式格式参数为fact.field_key成员访问；抽取字段策略后成为field_key普通变量，保留原占位写法因此触发Clippy。改为 `"字段{field_key}属于禁止进入模型的预测、盘口或推荐内容"` 内嵌捕获，原Display输出、错误类别/恢复建议/非retry和策略不变。没有allow、门禁放宽或依赖变更。检查迁移相关format调用，其余已捕获变量或保持原成员表达式，无其他同根点。

原scripts/verify-rust-source-hygiene.mjs增加具名路径回归守卫，旧源码被该守卫拒绝，改后通过；复用已有入口，不新增runner/framework/target。原Gateway90unit/31contract源码全保持，既有同步12类政策错误契约含home_ODDS完整提示断言。修复仅一个生产format表达式，全部其他Gateway src/tests字节保持，原API/DTO/配置/来源时间/错误顺序/用量/raw/模型/迁移/UI不变。初版657生产literal等价记录属于初版；本轮只有该format模板字面量改变，输出完全相同，literal总数保持。

本轮相对f714acb实际 **0A / 8M / 0D**，移动/重命名无：

- README.md
- architecture/domain-type-inventory.json
- crates/research-gateway/src/formal_research/policies/source_policy.rs
- scripts/verify-rust-source-hygiene.mjs
- docs/TESTING.md
- docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md
- docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md
- docs/modular-rewrite/R09-research-ai-backend/README.md

原节点4A/11M/1D、16文件，含新增卫生门禁修改后为 **4A/12M/1D、17文件**，上方完整清单已同步。Domain扫描1134、365/300与sourceDigest保持，当前usageDigest `d52e1a00455c4b29c48382a6c591f0d65aafde4a4da093bb7c6bb769fcaf3cbd`；只该摘要更新，其他inventory字段不变。

实际验证：83/83既有JS静态、完整npm run verify:architecture、Rustfmt1.88 source_policy源码check、差异/语义等价、原缺陷拒绝/修复接受及18保护/171命令/DB静态PASS。报告 `/workspace/scratch/eb298ad5cdcb/r906f-static-checks.json`、`r906f-architecture.log`、`r906f-equivalence.json`、`r906f-review.json`。非Windows不执行Cargo/编译/Clippy/单测；修复自身Windows必须复验，静态不能关闭Windows失败。

Context7 resolve/query /rust-lang/rust-clippy未提供1.88.0版本目录，另读官方rust-lang/rust-clippy **rust-1.88.0** clippy_lints/src/format_args.rs（blob `8a3f8e1c5874d45ca2c67cbe4e0c49fcec9233c3`）核对原lint与内嵌示例。简单格式修复未改架构，原Mermaid图仍适用；Create State本轮核实稳定足球model后保存真实失败/修复/待验。

原分支推送修复自身Windows，精确开始后停止轮询，预计20～30分钟；06仍VERIFYING、07～11BLOCKED，05 DONE。回退仅受控revert该格式/门禁/摘要/文档修复；完整06回退点仍b6636fe。真实PG/历史四项/账本并发回滚/XLSX/Windows Full/私有固定回归及继承删除风险保持最终封包新库待验。

## 第二轮 Windows 单测失败修复（2026-10-11）

用户再次“未通过”后核实精确修复提交 `3981b9d836343df1fa71fbe60e722b26c6e85436` / tree `6d7b692e483d067b09aa50b5ecddb42bc865f478`，[run38109741446](https://github.com/uniquenesssta/123/actions/runs/38109741446) / [job114382644048](https://github.com/uniquenesssta/123/actions/runs/38109741446/job/114382644048) 均FAILURE，2026-10-11 12:03:19北京停止。前端/contracts/TypeScript/Vite/17视口及Rust fmt/Clippy -D warnings已通过；Application126/126、Persistence175/175单测通过，Gateway89/90。失败为新增domain_rules_keep_case_prefix_subdomains_and_dot_boundaries在source_policy.rs:135对apex执行expect，实际SourcePolicy错误“来源域名不在当前赛事允许列表：example.com”。Gateway contract31、后续workspace/release/MSI/NSIS/客户端启动未完成，不能标记全链路SUCCESS；ignored数据库测试不计真实数据库通过。日志记录D:\a\123\123\logs\windows-acceptance-20261011-035310.txt。

原domain_matches先trim配置、去*.和仅小写www.前缀，最后to_lowercase，和05基线完全一致。新增成功夹具误将“ WWW.EXAMPLE.COM ”当成example.com；实际保留前缀后得到www.example.com，URL host规范后为example.com，因此拒绝正确。本轮只修原具名unit中的夹具和覆盖：成功矩阵包括EXAMPLE.COM主体大小写、仅小写www.与*.www.；新增WWW/WwW及*.WWW配置对apex/普通子域的完整错误、类别/非retry断言，同时验证sub.www.example.com仍按点分子域成功。原成功、子域边界和错误断言保留，不改生产政策迁就测试。

原scripts/verify-api-compatible-transport.mjs补前缀去除/小写先后及具名测试矩阵守卫。三个反向探针：本次失败原夹具、提前小写生产规则、移除大写前缀拒绝断言，均被针对性拒绝并恢复，最终正向门禁通过。不新增测试target/runner/workflow/framework，不跳过测试、抑制Clippy或放宽门禁。测试数量仍90unit/31contract，原82unit/29contract以及其他Gateway源码/测试字节保持；本轮所有Gateway生产字节与3981b9d完全相同。

本轮相对3981b9d实际 **0A / 8M / 0D**，移动/重命名无：

- README.md
- architecture/domain-type-inventory.json
- crates/research-gateway/src/formal_research/policies/source_policy.rs
- scripts/verify-api-compatible-transport.mjs
- docs/TESTING.md
- docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md
- docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md
- docs/modular-rewrite/R09-research-ai-backend/README.md

06全量仍 **4A/12M/1D、17文件**。Domain清单仅rustUsageDigest因测试行偏移变为 `c6b0e2980fbdb43f4f4a7feecb0df38fa8ccf98c503eda723c93f314bc593510`；1134扫描、365/300和声明sourceDigest不变。生产接口/DTO/配置/错误/来源与时间策略、原同步/后台/恢复链路、用量/raw、依赖/模型/迁移/DB/UI/Windows流程保持。

本轮实际83/83既有JS静态、完整npm run verify:architecture、Rustfmt1.88 check、diff/生产字节等价与18保护/171命令/46迁移18PG静态通过。报告 `/workspace/scratch/eb298ad5cdcb/r906f2-static-checks.json`、`r906f2-architecture.log`、`r906f2-equivalence.json`、`r906f2-review.json`。静态不能关闭Windows运行失败，90/31全过仍是修复自身Windows待验；非Windows不执行Cargo/编译/Clippy/单测/浏览器运行时。

本次为项目内原字符串规则与新增夹具不一致，不涉及陌生库、新API或版本/编译问题；不另查Context7，之前锁版本证据保持。未改架构，原Mermaid图保持；Create State本轮核实稳定足球model96f03270-c236-46de-99fa-db85d2fbf4ce后保存实际失败/修复/待验。

同一rewrite/r9-research-ai-backend分支推送原Windows CI，精确修复自身Windows job开始即停止轮询，预计20～30分钟；05 DONE、06 VERIFYING、07～11BLOCKED。本轮回退仅受控revert该测试/门禁/摘要/文档修复；整体06回退仍b6636fe。最终新库真实PG/历史四项/账本并发回滚/XLSX/Full/私有固定回归及继承删除风险继续显式待验。

## 第三轮 Windows 契约失败修复（2026-10-11）

用户再次“未通过”后核实精确43dba5f7aa9bf2fc1180ccbaa27369c89e10ec10 / tree e105256715eb62f10425446806ba74523098fc0b，[run38135480448](https://github.com/uniquenesssta/123/actions/runs/38135480448) / [job114458351000](https://github.com/uniquenesssta/123/actions/runs/38135480448/job/114458351000) 均FAILURE。2026-10-11 19:35:14北京在gateway_contract.rs:1917的queued_and_resumed_policies_keep_original_urls_cutoff_and_content_validation失败，实际来源首项https://example.com/team-news?q=one#citation，误期望#search。该轮前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings、Gateway90/90和Application126/126、Persistence175/175单测通过；Gateway contract30/31，后续workspace/release/MSI/NSIS/启动未完成。前两项修复已通过，不代表06完整SUCCESS；ignored数据库测试不计真实数据库PASS。日志windows-acceptance-20261011-112634.txt。

正式解析原先即使用完整原URL为BTreeMap键，合并搜索来源与引用，into_values输出按URL排序；#citation、#search原片段不同，形成两条原始WebSource，#citation排在#search之前。Citation政策只以移除fragment后的URL做成员关联，不修改响应来源列表。既有05单测formal_citations_preserve_order_first_source_metadata_usage_and_raw_response已覆盖原排序，本轮实际Windows已通过；新增06契约却错误假定来源首项就是搜索项，之前复核漏掉该排序。

只把该一处索引断言替换为来源URL完整Vec相等断言，精确保留查询/片段和#citation→#search顺序及两条来源；不只改索引或用contains弱化集合验证。该契约的两种入口×三种情形逐项复核，原raw_response、事实/引用URL、UTC cutoff、两类错误、cached用量/provider request ID、凭据加载/POST/GET次数与实际GET地址保持；其他契约/夹具、全部Gateway src与90unit均字节不变，仍90unit/31contract。正式URL键与规范成员关联各留原owner，不改变生产行为。

原scripts/verify-api-compatible-transport.mjs补完整URL键/搜索与引用合并/原排序出口，以及该具名契约完整来源列表和原分支断言守卫。四反向探针（旧失败首项断言、预期列表倒序、列表漏引用、生产wire键误去fragment）均被针对性拒绝并恢复，最终正向通过。没有allow、跳测试、门禁放宽或新runner/target/workflow/framework/DB入口。

本轮相对43dba5f实际 **0A / 8M / 0D**，移动/重命名无：

- README.md
- architecture/domain-type-inventory.json
- crates/research-gateway/tests/gateway_contract.rs
- scripts/verify-api-compatible-transport.mjs
- docs/TESTING.md
- docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md
- docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md
- docs/modular-rewrite/R09-research-ai-backend/README.md

06全量仍 **4A/12M/1D、17文件**。Domain清单仅rustUsageDigest随契约测试行偏移变为c3aece9b4b2beec64c5434d549d9596532b310b9e88541fda2220ba2c1845bc0；1134扫描、365/300和声明sourceDigest不变。原生产API/DTO/Schema/SourcePolicy配置、错误/来源时间政策/同步后台恢复/用量raw、依赖/模型/迁移/DB/UI/Windows流程保持。

实际本轮83/83既有JS静态、完整npm run verify:architecture、Rustfmt1.88 check、diff/生产与其余契约字节等价、四探针及18保护/171命令/46迁移18PG静态通过；报告/workspace/scratch/eb298ad5cdcb/r906f3-static-checks.json、r906f3-architecture.log、r906f3-equivalence.json、r906f3-review.json。静态不能关闭Windows契约失败，修复自身90unit/31contract全过及完整Windows仍待验；非Windows不执行Cargo/编译/Clippy/单测/浏览器运行时。

本次是既有项目来源列表语义与新增断言不一致，原实现及已通过单测提供证据，无新版本/API/编译问题，不另查Context7；原架构/Mermaid图保持。Create State本轮先核实足球稳定model96f03270-c236-46de-99fa-db85d2fbf4ce再保存实际失败/修复/待验。五既有文档同步，当前阶段索引仍05 DONE、06 VERIFYING、07～11BLOCKED、R9 IN_PROGRESS，无阶段完成记录。

原rewrite/r9-research-ai-backend分支推送修复自身Windows CI，精确job开始即停止轮询，预计20～30分钟。回退仅受控revert本轮测试/门禁/摘要/文档修复，整体06仍b6636fe。最终新库真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险继续显式待验。

## R9-06 完成记录（2026-10-12）

精确修复提交d996a4d4bea34d53486b14a642fa1f655478bde7 / tree2584ce620b3d9f53595f8d4d49da0b3f5ae42df0，[Windows run38145691538](https://github.com/uniquenesssta/123/actions/runs/38145691538) / job114488432087均SUCCESS，2026-10-11 22:30:11北京完成。Gateway90/90、contract31/31、Application126/126、Persistence175/175；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7条记录、3个完成操作）。

证据artifact11705243220，14,039,496字节，SHA-256 4d11fab6a5732281e1cbe5b3112ebdf2314d20f9d2d99d13141cb25e9ca499d8；报告windows-acceptance-20261011-140603.json、同名.txt。初轮format捕获、第二轮域名前缀夹具、第三轮来源排序断言三个失败均由该精确修复自身Windows关闭；新增8unit/2contract全通过。

06 DONE；用户2026-10-12“收尾06开始07”授权07 READY，同一rewrite/r9-research-ai-backend分支，08～11 BLOCKED、R9 IN_PROGRESS。本次只同步五既有文档，使用[skip ci]复用上述同源码Windows成功；此前VERIFYING为实施历史，无R9阶段完成记录。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored和公共stub不计PASS。
