# R9-06 Source 与 Time Policy 实施记录

状态：`VERIFYING`。用户2026-10-11“收尾05开始06”授权。uniquenesssta/123、0.23.0，唯一阶段分支rewrite/r9-research-ai-backend。05精确aa70d2b/run38072987214/job114274071102全SUCCESS/DONE，五文档收尾基线/回退 `b6636fef9589e86ee59c173732a847fd92cb21ed`、tree `2cb6d893136938d065a317a48758aa2e5c63e7fb`；源码与05成功一致。07～11BLOCKED，R9整体IN_PROGRESS。

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

本记录保留23职责/契约字段、16真实文件、原行为等价与新增测试/工具/取舍/延期/回退。阶段索引06 VERIFYING、07～11BLOCKED，不创建R9阶段完成记录。

### 23. 完成标准

实现、静态和记录已完成；06自身精确Windows完整SUCCESS才DONE。ignored、公开stub和指纹不能代替真实PG/Full/私有运行。

## 完整实际文件清单

相对b6636fe，含untracked，实际 **4A / 11M / 1D**，16文件；移动/重命名：无。

新增（4）：

- crates/research-gateway/src/formal_research/policies/mod.rs
- crates/research-gateway/src/formal_research/policies/source_policy.rs
- crates/research-gateway/src/formal_research/policies/time_policy.rs
- docs/modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md

修改（11）：

- README.md
- architecture/domain-type-inventory.json
- crates/research-gateway/src/formal_research/citations/validation.rs
- crates/research-gateway/src/formal_research/mod.rs
- crates/research-gateway/src/formal_research/schema/validation.rs
- crates/research-gateway/src/lib.rs
- crates/research-gateway/tests/gateway_contract.rs
- scripts/verify-api-compatible-transport.mjs
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
