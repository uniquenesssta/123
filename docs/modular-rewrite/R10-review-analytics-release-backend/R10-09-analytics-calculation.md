# R10-09 Analytics Calculation

状态：`VERIFYING`。用户2026-10-13“收尾08开始09”授权；uniquenesssta/123 v0.23.0，唯一分支rewrite/r10-review-analytics-release-backend。基线3fdb4435524af6a776256c3fa252ac3f685d06ab为08五文档[skip ci]收尾，源码同f834e31/Windows run38357781835全SUCCESS。01～08 DONE，09 VERIFYING，10～13 BLOCKED，R10 IN_PROGRESS。

### 1. 目标

收拢真实分析刷新与总览入口，以及原评估样本、快照、校准/漂移子记录、模型指标写入和总览投影，保持原公式及事务行为。

### 2. 现状与来源

App原analytics_overview单入口、jobs worker两条直接refresh调用；PG analytics.rs混合计算、数据质量、查询诊断、AI包及参数候选。只重写本计算边界，保留原analytics-engine纯计算和后续节点职责。

### 3. 目标文件与目录

App use_cases/analytics/calculation/{mod,overview,refresh,tests}.rs。PG adapters/analytics/calculation/{mod,repository,samples,mapping,metrics,projection,read,tests}.rs及上层analytics/mod.rs。参考目录按实际调用落实，不新增Port或外部命令。

### 4. 文件职责边界

App overview/refresh持有原Port调用边界；worker继续持有队列生命周期。PG repository唯一编排原分析写入事务；samples原筛选查询；mapping原Row字段解码；metrics借用同一事务写两个指标子记录；projection仅将原计算/样本身份/窗口转成持久化和总览值，不做统计公式或IO；read原快照及诊断读取。各mod只登记或显式导出，测试仅cfg(test)。

### 5. 输入

原AnalyticsRefreshRequest全部六字段：competition_id、window_start/end、bucket_count、baseline_size、current_size；原EvaluationSample、纯引擎返回的AnalyticsCalculation及已有质量/查询摘要。App不新增默认、验证、时间范围修复或窗口上限。

### 6. 输出

原AnalyticsOverview、analysis_snapshots/calibration_buckets/drift_snapshots/model_metrics及evaluation_samples冲突更新、原analytics_refreshed审计。无新增Domain、公共DTO、Result类型、模型字段或Schema。

### 7. 允许依赖

App原AnalyticsPort/ApplicationResult/Domain；PG原SQLx/chrono/Uuid/serde_json/QueryBuilder/PersistenceResult/write_audit_event和football_analytics_engine::calculate_analytics。read复用旧analytics的两个诊断读取，只将原私有方法可见性改为pub(crate)，正文不变。

### 8. 禁止依赖

App不引具体数据库、SQL、文件IO或公式。projection不承担IO，两个metrics函数不开始/提交事务。无新通用Repository、跨模块State、依赖、运行器、工作流、Cargo target或数据库专项入口；不自动接受候选/晋升参数，不改变P4/P7/参数/Profile/Schema/Golden和纯引擎。

### 9. 状态所有权

原数据库快照/指标/审计持有状态，调用内HashMap和具名窗口值只是临时投影。原19 state owner、43Ports/19Port文件保持，App425文件；PG直接模块owner由37增至38，仅登记实际新增analytics适配器。没有长期共享可变状态。

### 10. 副作用边界

原pool读样本→原纯计算→UUID→begin一次。事务内原500条分批样本upsert→原窗口推导→summary序列化→分析快照→校准桶→漂移finding→四字符串分组匹配的model_metrics upsert→原审计→commit一次。窗口推导仍在样本写后，序列化时点不提前；metrics借原tx。提交后仍self.analytics_overview().await，不把回读纳入事务或新增重试。

### 11. 异常路径

原Row字段try_get顺序、?传播及SQL/serde错误保持。任一样本/快照/子记录/模型指标/审计/提交失败停止后续，原未提交事务由SQLx释放；提交后overview失败不撤销已提交数据。总览先解码快照，解码失败不读诊断，随后质量100条、查询摘要；App六类PortError保持且无额外刷新/回读。

### 12. 并发/异步/生命周期

原两个worker刷新点已切计算入口，claim/parse/progress/cancel/complete/fail和全量刷新后质量→查询顺序逐字保持，生命周期归后续10。PG原样本(run_id,kickoff_time,version)及model_metrics复合冲突更新保持，不改成不可变追加；真实并发、唯一约束/事务回滚待最终新库。没有新增锁、超时、request ID或取消状态。

### 13. 兼容要求

原21公共Analytics facade入口、Port签名/组合适配器、PostgresStore公开签名、Domain/版本0.23.0、命令171、全部SQL/绑定/错误/JSON/日志/UI/配置/依赖/迁移保持。样本仍finalized+succeeded+available及原可选比赛/包含端点窗口；DISTINCT ON(run.id)，按run.id/review.created_at DESC/review.id DESC，不添加时序重排。四字符串分组大小写精确，重复组仍输入最后UUID对覆盖。显式窗口优先，缺边min/max，空样本快照窗口None，metrics缺边回落generated_at；倒置窗口不在投影修复。

### 14. 实施步骤

先核对08精确Windows并五文档收尾；从3fdb443同分支移原overview、提取实际refresh边界、切Service/worker；PG迁七原计算职责，窗口和总览分支提取纯投影，保留剩余29函数。补5App/8PG测试、原Analytics Service门禁和官方清单后，检查等价/负向/架构/保护/文档，推送原Windows CI。

### 15. 切换入口

AnalyticsService→calculation::analytics_overview；worker刷新和全量刷新→calculation::refresh_analytics；原AnalyticsPort组合适配器仍调同名PostgresStore公开方法，新repository/read实际实现。load_evaluation_samples、子记录、mapper与group规则各一处，根analytics无旧实现或空转发；AI包调用同名overview自动复用新owner。

### 16. 删除清单

删App analytics_overview/mod.rs及旧注册；原9行入口函数迁入具名calculation/overview.rs。PG root移出refresh_analytics/analytics_overview/load_evaluation_samples/sample_group_ids/insert_calibration_bucket/insert_drift_finding/evaluation_sample_from_row七职责与无用imports；保留AI包仍用DateTime/Utc/ANALYTICS_CALCULATION_VERSION，保留其余29函数。逻辑迁移1项；无整文件原样搬迁或根analytics删除，完整文件A/D以下用git --no-renames表示。

### 17. 最小验证

实际84/84既有JS源码检查、完整npm run verify:architecture、18实际Rust独立Rustfmt1.88 --check、15等价核对、11负向拒绝且所有bytes恢复、git diff --check与文档/链接核对通过。新增5App/8PG进入原unit target，预期Application176=171+5 / Persistence235=227+8 / Gateway112unit+41contract，必须本项精确Windows实跑，不记录本地动态PASS。

### 18. 阶段回归

沿用原Public Platform CI Windows Automated：frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy --workspace --all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动及日志扫描。没有非Windows Cargo/check/Clippy/build/test/frontendbuild/browser/loopback。现有46迁移/18PG静态契约不变，没有新框架或专项设施。

### 19. 失败停止条件

09自身精确完整Windows SUCCESS前VERIFYING，10～13BLOCKED，无阶段完成记录；失败只修同R10分支，不继承08PASS或减少测试。原Windows job实际启动后停止一切CI查询，提示约20～30分钟。

### 20. 回退点

3fdb4435524af6a776256c3fa252ac3f685d06ab（08文档收尾，源码同已验f834e31）。受控revert09完整代码/门禁/清单/记录，保留08收尾；不手工复制旧函数造成双owner。

### 21. 根README摘要记录

根README追加09唯一计算入口/事务边界、实际文件变化、新13测试、本地检查及自身Windows待验，链接本节点与索引；08DONE、10～13BLOCKED保持。

### 22. docs阶段详细记录

本23字段记录实际职责、迁移、兼容、事务/回读风险、A/M/D、源码证据和未执行项；索引/任务书/TESTING同步VERIFYING。08收尾另5M[skip ci]与09实现分开提交；先前08历史状态保留。

### 23. 完成标准

新计算职责和实际调用唯一、旧实现清理、源码门禁/记录完成；09自身精确Windows全SUCCESS才DONE。真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub、静态保护指纹不计这些实际PASS。

## 核对、测试与工具证据

15等价核对全部通过：root29保留函数体及两诊断可见性；5原样本/身份/子记录/mapper全文；原窗口推导；refresh逆窗口提取后完整原事务；overview原Some/None投影；overview逆投影后完整原读取；273个原生产SQL/错误/JSON字面量多重集；全部PG公开签名；原App总览只改名字/路径并删除旧入口；worker逆两调用后完整生命周期；Service逆一调用后全文；内部refresh单次原Port边界；其余节点/纯引擎/Ports/models/UI/contracts/deps/DB/workflow/targets/shared state冻结；365Domain/129声明文件/301映射不变；13新unit只进入原targets。Rust词法核对包含续行字符串，不误把SQL续行后的代码当字面量。

5App测试覆盖完整六字段请求/纳秒/倒置窗口/0与usizeMAX原样传递、默认无界请求、完整含comparison/calibration/drift/诊断的响应、refresh六类PortError及overview六类PortError且无重试/额外调用。8PG测试覆盖空窗口与metrics fallback、乱序min/max/纳秒、显式倒置优先、单边推导/空边fallback、全部四大小写字符串维度、重复组最后实际UUID对且反转输入改变winner、无快照仍保留实际诊断、已保存全部指标/子记录/旧版本/诊断的逐字段投影。未改变纯引擎，也不新增复杂数值公式预期。

11破坏探针全部拒绝并按bytes恢复：旧root职责返回、worker绕计算、窗口移到分批写入前、删commit、500批次变499、latest review顺序反向、窗口min改max、分组缺snapshot_type、子记录逃逸原tx、质量100改10、保留AI时间导入缺失。源码门禁是静态边界检查，不替代PG故障/回滚或Rust实跑。

官方清单：Domain sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd；scan1244，rustUsageDigest b73aa7346035c274f5583c189878c2216d278d30d5d84c8e3992e5b1f7c3ab06。18保护资产聚合d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57；171命令；46迁移/18PG静态聚合d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93。PG直接owner38/state19，App425/43Ports/19Port文件。

实际命令：python scratch/r1009_static.py（84源码）与r1009_compare.py/r1009_probes.py（原链等价/静态破坏恢复）；npm run verify:architecture；原verify-analytics-service.mjs；verify-application-ports.mjs --refresh-source-scan；generate-domain-type-inventory.mjs；verify_protected_assets.mjs/verify_command_contract.mjs/verify_database_baseline.mjs；独立Rustfmt1.88与git diff --check。scratch r1009-static-checks.json/r1009-architecture.log/r1009-equivalence.json/r1009-negative-probes.json为可复建临时报告，不新增仓库验收框架。初次架构检查发现新PG登记缺失，已同步module-boundaries原owner清单/计数，随后84/84和完整架构通过；没有放宽任何边界。

Mermaid Chart已显示中文实际刷新/总览、样本→纯计算→同一事务→提交后回读链。使用既有稳定API，无新版本/未知库接口问题需重复Context7；Create State稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce保存实施与CI交接，不读附件Library、不另存git制品、无子代理。

## 全部实际文件清单

以基线3fdb443对照git diff --no-renames及未跟踪文件生成，含本记录和四文档同步；逻辑迁移上述旧总览→新具名总览一项，统计按实际A/D表示。

### 新增（14）

- `crates/application/src/use_cases/analytics/calculation/mod.rs`
- `crates/application/src/use_cases/analytics/calculation/overview.rs`
- `crates/application/src/use_cases/analytics/calculation/refresh.rs`
- `crates/application/src/use_cases/analytics/calculation/tests.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/mapping.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/metrics.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/mod.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/projection.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/read.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/repository.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/samples.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/tests.rs`
- `crates/persistence-postgres/src/adapters/analytics/mod.rs`
- `docs/modular-rewrite/R10-review-analytics-release-backend/R10-09-analytics-calculation.md`

### 修改（13）

- `README.md`
- `architecture/application-port-inventory.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `crates/application/src/services/analytics/service.rs`
- `crates/application/src/use_cases/analytics/jobs/worker.rs`
- `crates/application/src/use_cases/analytics/mod.rs`
- `crates/persistence-postgres/src/adapters/mod.rs`
- `crates/persistence-postgres/src/analytics.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md`
- `docs/modular-rewrite/R10-review-analytics-release-backend/README.md`
- `scripts/verify-analytics-service.mjs`

### 删除（1）

- `crates/application/src/use_cases/analytics/analytics_overview/mod.rs`

Git整文件原样重命名：无。旧9行总览职责改名/归属为逻辑迁移，其他为函数或纯投影片段提取；不按文件数量机械切分。实际合计28路径，18存在的变更Rust文件独立格式检查通过；删除文件不纳入Rustfmt。


## 首轮 Windows type_complexity 修复（2026-10-13）

首轮精确提交656a97cb466c480ed50125a60b3af1d288597f29 / tree4c6333eef2ebad1e3cf5ceda760845f232188ff4，[Public Platform CI run38371099454](https://github.com/uniquenesssta/123/actions/runs/38371099454) / job115208003236完整结束FAILURE。唯一错误在PG calculation/projection.rs:13，analysis_windows返回两个Option<DateTime<Utc>>及两个DateTime<Utc>的四项元组触发clippy::type_complexity，-D warnings使编译停止。2026-10-13 16:47:00北京记录Rust链失败；不是窗口数值单测失败，workspace单测/打包/启动尚未完成。前端/contracts/TypeScript/Vite/17视口和Rustfmt实际通过，不能据此标记09DONE。

失败报告windows-acceptance-20261013-084347.txt；artifact11799147265，672bytes，sha256 f4f8cd93df7fe95ab7c79c325591cf394b3d0bdef21381c5e0580247977ce227。修复基线本首轮656a97c，不重做08收尾或进入10。

现在同一projection定义pub(super) AnalysisWindows，四原字段window_start/end为Option<DateTime<Utc>>，metric_window_start/end为DateTime<Utc>；Debug/PartialEq/Eq仅支持现有值断言。analysis_windows原局部推导代码和显式优先/minmax/空边fallback不变，返回四字段具名值。repository在原分批写后同一时点解构该值，原变量继续原快照/子记录/metrics绑定。结构只在计算模块内可见，没有Domain/公共API/DTO/Schema/长期State或新的wrapper/file。没有allow/expect、阈值配置、工具链或依赖变化。

原tests中8处窗口期望元组改为同四值的具名结构，保持空/乱序/纳秒/显式倒置/单边边界与原assert_eq，不改其余函数、数据、容差、断言或测试数量。原5App/8PG新unit数量保持；预期App176/PG235/Gateway112unit+41contract仍须此修复提交自身Windows执行。

实际84/84原JS源码、完整npm run verify:architecture、3实际修改Rust独立Rustfmt1.88 --check、22等价（原15核对+本次7项类型形状逆切）、5新负向及git diff --check通过。7项修复核对为：window逆返回形状后完整原函数一致；其他projection函数保持；结构精确四原字段/内部可见性；repository逆具名解构/类型导入后全文一致；tests逆8处具名期望/导入后全文一致；无lint抑制/工具链依赖配置变化；只改这3个Rust文件。原29保留函数、273生产字面量多重集、SQL/绑定/公开签名/纯引擎/原worker和Service逆切及原保护区保持。

原Analytics Service门禁新增窗口具名返回、四原字段类型和值映射、真实事务解构及无type_complexity抑制检查。5探针：恢复多行复杂元组、恢复单行复杂元组、把window_start映为window_end、加入allow(type_complexity)、调用方漏metric_window_end；全部被拒绝且按bytes恢复。此前11生产破坏探针为初始实施历史，本修复不重复无变化探针；现22等价重新确认整条原链。scratch r1009fix1-static-checks.json/r1009fix1-architecture.log/r1009fix1-equivalence.json/r1009fix1-negative-probes.json可复建，未增加验收框架/runner/workflow/target/数据库入口。

官方Domain365/129声明文件/301映射、scan1244/sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持。由于原3个Rust文件变化，rustUsageDigest刷新为f80276941dc555a7864c61f2f714de4e5819a4c3ebe048f0a58db90796d4075a；没有新增Domain类型。App425/43Ports/19Port文件、PG直接owner38/state19保持，18保护/171命令/46迁移和18PG静态契约不变。

已从原.github/workflows/ci.yml确认Rust/Clippy1.88.0。Context7返回当前Rust Clippy源码说明复杂度按嵌套类型计分，未提供1.88专属索引；再核对[官方rust-1.88.0标签的类型复杂度实现](https://github.com/rust-lang/rust-clippy/blob/rust-1.88.0/clippy_lints/src/types/type_complexity.rs)，该版本按语法中的Path/Tup/嵌套类型访问并与阈值比较。采用简单具名Path返回和分别声明字段，结合1.88实际CI诊断修正，不将master文档冒充版本证据。原Mermaid复杂链继续准确，本次类型形状改动不改变图中拓扑或事务边界，无子代理。Create State按稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce保存本失败/修复/新CI交接。

本轮相对656a97c为10M/0A/0D，移动/重命名无。受控revert本修复到656a97c仅回退类型/原期望形状/守卫/使用摘要/记录，会恢复已知Clippy失败；回退整个09则按第20节基线3fdb443。09仍VERIFYING，10～13BLOCKED、R10IN_PROGRESS；修复后的原Windows job实际启动即停止一切CI查询并提示20～30分钟。未执行非Windows Cargo/check/Clippy/build/test/frontendbuild/browser/loopback；真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验，ignored/公共stub/static指纹不计真实PASS。

### 本修复全部修改文件（10）

- `README.md`
- `architecture/domain-type-inventory.json`
- `crates/persistence-postgres/src/adapters/analytics/calculation/projection.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/repository.rs`
- `crates/persistence-postgres/src/adapters/analytics/calculation/tests.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md`
- `docs/modular-rewrite/R10-review-analytics-release-backend/R10-09-analytics-calculation.md`
- `docs/modular-rewrite/R10-review-analytics-release-backend/README.md`
- `scripts/verify-analytics-service.mjs`
