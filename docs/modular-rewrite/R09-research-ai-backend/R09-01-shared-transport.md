# R9-01 Shared Transport 实施记录

状态：`VERIFYING`。2026-10-09用户明确“开始R9-01”，此前编号歧义已厘清。项目uniquenesssta/123、0.23.0，唯一阶段分支 `rewrite/r9-research-ai-backend`，起点/回退 `c72e559af4f29c9510daf2f9bf66b926dacb3013`；R8最终af3c98c/run37899786755源码已验，c72e559仅R8文档收尾。本项必须自身精确Windows，不继承R8成功。

## 实际来源与职责取舍

原client.rs同时持有传输trait/DTO、reqwest客户端与三HTTP操作/TLS/错误，以及网关协议/重试/并发/取消/预算/研究编排。提取本项真实shared transport，不提前重写后续credentials/retry/formal/plain/session/diagnostics。contract持原OpenAiTransport/TransportResponse；http持唯一ReqwestTransport、header、原send/body读、TLS Once及网络错误；response持纯字节JSON/空响应/解析错误。mod只登记和re-export，crate根保持原三个公开名字；不创建只转发的第二客户端或Application wrapper。

纯解码与HTTP IO有独立输入/失败和可验证边界，因此单列response；认证header/TLS/发送与错误同属该HTTP客户端职责，共置http，未机械为每个helper另建文件。GatewayAttemptSink/Noop、原payload组装/验证、provider响应语义/预算/circuit/Semaphore及三模式编排仍client；原凭据提供者与取消token不迁移。研究、普通问答、结构化分析、连接测试、恢复GET、远端取消空POST复用原接口，所有上游和原FakeTransport不需改名或替代实现。

## 公开契约、状态、异常与兼容

原三trait方法参数/借用/Duration/Send+Sync、TransportResponse status/provider_request_id/body及Clone/Debug保持；OpenAiResearchGateway/connection API和所有root导出名称保持。ReqwestTransport仍持一个可clone的reqwest::Client，原UA版本、禁止redirect、Bearer及application/json、POST JSON/GET/空POST、timeout/send/body读取保持；transport不添加tools/schema/token参数、重试/cancel registry或provider状态解释。

原TLS Once/ring install保持唯一；auth header失败仍原MissingCredential中文消息/恢复，不回显输入。reqwest is_timeout仍Timeout/原retryable/恢复，其余仍Network原格式；JSON empty严格Null、任意JSON Value原样保留，非2xx仍先解码再交给原Gateway provider错误解释。非法JSON仍原SchemaValidation/可重试/原消息和provider_status，原错误DTO无法携带request_id的边界保持，不擅自改变错误优先级/载荷或HTTP成功策略。

原Gateway trace/provider response ID、外层tokio::select、取消后的future drop、重试等待、后台poll/远端cancel/并发/circuit状态均完整保持；transport不新增State/监听器/持久化，不把生命周期模板套给全部函数。连接测试无新增重试/并发或取消政策。正式Responses、普通最小双协议、结构化原兼容fallback/字段、配置/profile/API Example/凭据落地/会话/UI不变；不升级外部协议。

## 完整实际文件清单

基线到当前工作树的git diff --no-renames与新增文件核对：5新增、9修改；无整文件移动/重命名/删除。

### 新增（5）

- `crates/research-gateway/src/transport/contract.rs`
- `crates/research-gateway/src/transport/http.rs`
- `crates/research-gateway/src/transport/mod.rs`
- `crates/research-gateway/src/transport/response.rs`
- `docs/modular-rewrite/R09-research-ai-backend/R09-01-shared-transport.md`

### 修改（9）

- `README.md`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `crates/research-gateway/src/client.rs`
- `crates/research-gateway/src/lib.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/09-R9-research-ai-backend.md`
- `docs/modular-rewrite/R09-research-ai-backend/README.md`
- `scripts/verify-api-compatible-transport.mjs`

### 移动/重命名

无。函数按职责提取，client仍持有其余原网关职责。

### 删除

无整文件删除；client内旧transport trait/DTO/adapter/TLS/network error实现移除，不保留双owner或兼容wrapper。

## 原测试与本项边界覆盖

原Gateway unit target的15项和gateway_contract16完整保持；原8新增unit定义在http/response已有同crate单测target，无新test runner/target/数据库/服务/依赖。源码预期Gateway **23**（15+8）/contract **16**，Application **126**/Persistence **175**保持；须自身Windows实跑，静态或R8结果不能代替。

1. 三HTTP操作：有界127.0.0.1单次夹具分别核对POST JSON/GET/空POST、完整URL query、原UA/Bearer/Content-Type及原body/无body，HTTP429/请求ID/JSON完整透传。
2. 307带Location原样返回，禁止跟随到第二请求。
3. 发送响应头后延迟body：请求timeout覆盖body读取，完整原Timeout/恢复匹配。
4. 非法URL保持Network/retryable/原恢复，无provider metadata或测试credential泄露。
5. 原认证头值及非法零字符拒绝，完整原MissingCredential错误/恢复。
6. 各2xx/4xx/5xx与object/array/number/null原样透传，transport不实施provider策略。
7. 严格空body Null、原HTTP状态/可选请求ID保持。
8. whitespace、截断JSON、非法UTF8、零字符仍完整原SchemaValidation消息/恢复/状态。

loopback仅测试夹具，原std线程、accept/read/write各5秒上限，测试请求5秒，body延迟2秒/请求500毫秒；不触达真实提供器。测试只在本项WindowsCI执行；本地非Windows没有启动夹具/网络测试。原协议contract中成功/错误/并发/取消等场景保留，同源client token等价覆盖全部原workflow，不写镜像生产实现的空测试。

## 实际验证与待验

实际静态PASS：83/83原前端源码验证（排除原5浏览器项）；完整npm run verify:architecture；原API compatible transport、Domain、源码卫生；18保护资产指纹/私有缺席、171命令、46迁移/18原PG静态；Rustfmt1.88.0源码format/check；git diff --check。六探针改变redirect、timeout、HTTP方法、空body、错误类别、root导出均被原增强API compatible verifier拒绝并恢复。

等价核对：原client剔除本项职责并仅补传输import后的全体tokens保持；new/headers/post_json/get_json/post_empty/TLS install/map_reqwest_error/network_error八原函数逐token保持；response解码重内联后的execute等价；trait/DTO tokens完整保持，265生产literal含错误/HTTP/协议/hash均保持。无生产功能变化或原测试删除。报告：`/workspace/scratch/eb298ad5cdcb/r901-static-checks.json`、`r901-architecture.log`、`r901-negative-probes.json`、`r901-equivalence.json`。

清单只将OpenAiTransport owner设实际transport/contract，Domain扫描1099→1103、rustUsageDigest `21761727c797739ac862976e35399f8c0291bc27c02a2a509b7616f11cd97981`；365Domain/300映射/sourceDigest `217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd`保持。Application402/43Ports、171命令、PG根35及46迁移保持。整个crates/application、persistence-postgres、domain、model-api/P4/P7、contracts/schemas、src/src-tauri、依赖锁文件/workflow均无本项源码修改。

Windows待验：原前端contract/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace（含8新测试）、release/MSI/NSIS/启动。沿用Public Platform CI，确认本项精确head/Windows job开始后停止轮询；01VERIFYING、02～11BLOCKED，不提前创建R9阶段完成记录。真实PG、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共stub不计PASS。

## 文档、工具核对与订正

根README/TESTING、R9任务书和阶段索引已同步实际实施/待验及当前唯一分支；此前13映射歧义已由本次明确指令关闭，R8收尾历史保留。Mermaid Chart已展示原各入口及策略/共享IO真实边界；图不把连接测试强行串入执行重试。结束时Create State只保存足球稳定model96f03270-c236-46de-99fa-db85d2fbf4ce，Git与记录仍为实际依据。

Context7以实际锁定reqwest0.13.4查询，但仅返回latest示例；已另读[官方0.13.4 RequestBuilder文档](https://docs.rs/reqwest/0.13.4/reqwest/struct.RequestBuilder.html)核对原timeout从连接开始至body完成，不以latest替代精确版本；原API/依赖保持。初次静态脚本误用moduleContract.backend而实际根为rust，已按原清单结构修正并通过，未放宽owner条件。

原scratch formatter/归档截断，无法加载；恢复官方同版formatter文件并核对官方SHA（rustc archive b049fd57fce274d10013e2cf0e05f215f68f6580865abc52178f66ae9bf43fd8、rustfmt archive55f4c150a865cae19cfa5f5f2b5eec84ed98dc3a1c1c406d4323c63d896c3f1d），只运行格式化静态check，没有Linux/macOS Cargo/编译/单测或客户端动态。该操作不改变仓库依赖/运行时或用户安装要求。

## 回退与继续位置

回退基线 `c72e559af4f29c9510daf2f9bf66b926dacb3013`；受控revert本项源码、原门禁/清单和文档，恢复唯一owner/公开export，不复制旧实现、不覆盖用户改动或变更历史数据。下一步核实01自身精确Windows全SUCCESS后收尾01，02前置开放；失败则只修完整相关链路，不越级进入02。真实PG、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共stub不计PASS。
