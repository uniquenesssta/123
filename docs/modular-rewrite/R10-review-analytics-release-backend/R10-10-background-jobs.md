# R10-10 Background Jobs

状态：`DONE`（自身精确Windows全SUCCESS，以下VERIFYING为实施历史）。用户2026-10-13“收尾09开始10”授权；uniquenesssta/123 v0.23.0，唯一分支rewrite/r10-review-analytics-release-backend。基线4a6fa9b645dea38795912217ef966f8aa607816e为09五文档[skip ci]收尾，源码同已验fbfe83a/Windows run38398255939全SUCCESS。01～09DONE，10VERIFYING，11～13BLOCKED，R10IN_PROGRESS。

### 1. 目标

形成真实后台队列控制、异步启动、领取结算、任务执行/请求解码与共享PG队列持久化的唯一职责，保持原状态/取消/重试/历史行为。

### 2. 现状与来源

App原四独立目录仅持有各Port调用，worker混合spawn、领取结算loop、四任务执行与请求解码。PG root jobs.rs混合共享队列入队/控制/读取/领取/生命周期/Row/政策，分析、P4两类worker与启动恢复都实际复用公开方法。PG采用adapters/jobs共享归属而非虚构分析专用队列；P4业务编排和既有Ports不重写。

### 3. 目标文件与目录

App use_cases/analytics/jobs/{mod,commands,worker,coordinator,execution,request,tests}.rs。PG adapters/jobs/{mod,commands,read,claim,lifecycle,mapping,policy,tests}.rs。原PG root jobs.rs删除，四旧App入口目录删除，目录出口只登记/显式导出。

### 4. 文件职责边界

App commands统一四队列控制入口；worker唯一tokio::spawn启动边界；coordinator原领取/结算循环；execution原四任务和进度/取消编排；request原Null/空对象默认与serde解码。PG commands入队/取消/人工重试，read列表/单任务读取，claim原领取事务，lifecycle原运行进度/完成/失败和重启恢复，mapping Row解码，policy唯一支持类型/状态/失败归类。没有按函数机械拆文件，三生命周期事务共同维护运行状态与事件，纯政策仍在同一具名模块。

### 5. 输入

原EnqueueJobDraft全部字段、list limit、cancel/retry job_id；原领取types、后台job_type/payload/id、进度/消息/专用载荷及四类原结果。没有新request ID、配置、参数、超时、可运行类型或任务上限。

### 6. 输出

原BackgroundJob/JobStatus、AnalyticsJobResult、进度载荷、platform.jobs/job_events、原错误/提示/原public PostgresStore接口。没有新增Domain/公共DTO/Schema/通用Queue/Repository接口或后台长期State。

### 7. 允许依赖

App原AnalyticsPort/JobQueuePort、Domain/ApplicationResult/Error/serde_json与现有tokio::spawn；执行refresh继续使用09 calculation::refresh_analytics。PG原SQLx/Uuid/Domain/PersistenceResult/Error/serde_json。测试Fake只cfg(test)，Notify/timeout只同步原spawn的行为，使用已有tokio sync/time features，不引生产运行框架。

### 8. 禁止依赖

App不引SQL/具体PG、独立状态归类或统计公式。PG policy不IO，read/mapping不调度任务，worker不实现业务/SQL。无新共享可变State、自动候选接受/参数晋升、依赖/runner/workflow/target/migration或数据库专项；不更改纯review/analytics引擎及P4/P7/参数/Profile/Schema/Golden。

### 9. 状态所有权

原platform.jobs与job_events持有状态和幂等身份；worker/loop只有一次任务局部值，新增Fixture/State仅测试内。App426文件/43Ports/19Port文件；PG直接owner38和state19保持，lib jobs登记移到adapters jobs登记，清单以真实owner替换不改计数。原P4与数据库启动恢复复用同一公开PG实现，无新竞争所有者。

### 10. 副作用边界

App enqueue或retry原Port成功后才clone/spawn一次，返回原job；list/cancel不启动worker。原loop只领取refresh/data_quality/query_performance/full_refresh四类型；Some执行→成功complete或错误fail，结算错误按原策略忽略继续领取，None或领取错误结束worker。四类任务原进度/取消/业务顺序保持，全量仍refresh→quality→query且前一阶段副作用不因后续取消倒退。

PG入队原幂等conflict返回与max_attempts夹紧1..20；取消/人工重试原pool写后read。领取单事务SELECT符合到期/类型/预算且未取消queued，priority DESC/available_at/created_at/id，FOR UPDATE SKIP LOCKED；更新running/attempts+1/started事件→commit后Row解码。空候选也提交返回None。progress、complete、fail各自原独立事务写状态和事件，恢复原pool更新；不合并或新增事务。

### 11. 异常路径

原请求Json错误在progress前传播；原未知job_type提示、六检查点“任务已取消”、Port六类错误与上层转换保持。领取错误结束当前loop，complete/fail返回错误原本忽略，不新增补偿/重试/日志。PG未知状态在Row其他字段前拒绝、原try_get与SQL/commit传播保持；提交后Row解码或read失败不回滚已提交状态。保留原retry零行InvalidState和缺任务提示。

### 12. 并发/异步/生命周期

原每成功enqueue/retry/start入口都会tokio::spawn，原句柄丢弃政策保持；没有新的单例队列或定时轮询。分析只领四类型，P4原worker只领原P4types；共用PG SKIP LOCKED与资格条件。原attempt预算、available_at未到期不领取、重启running按预算重新排队或failed政策保持。失败取消优先，其次attempts<max_attempts queued+30秒，否则failed；手动retry仅failed/cancelled且预算未尽，不归零attempts。原数据库并发/唯一键/回滚仍最终新库实验待验。

### 13. 兼容要求

原21Analytics facade/四外部命令、两Ports/组合适配、11公开PG签名、JobStatus五态、全部SQL/绑定/错误/JSON/配置/日志/UI/模型/0.23.0/迁移保持。进度0..99夹紧两处同规则、running缺行progress返回true、complete缺行no-op；完成时取消优先，取消保留progress且result和event payload空，否则100及原result。原fail读取无新增running限制，支持类型六个含P4且大小写/空白精确；不增状态别名或新政策。

### 14. 实施步骤

核对09精确Windows/五文档收尾后，从4a6fa9b同分支编组四原入口并切Service、提取原loop与执行/解码；PG按实际共享职责迁13原函数，失败归类和状态映射提取纯政策，11公开方法保持。移lib登记/更新真实owner清单，补19原unit/原Analytics门禁和官方扫描，再做等价、负向、架构/保护及记录。09自身窗口值/纯计算和后续11AI/12参数/13发布冻结。

### 15. 切换入口

Service直接jobs::{enqueue_analysis_job,list_background_jobs,cancel_background_job,retry_background_job}；三原启动点（Service start_job_worker、enqueue、retry）继续同一worker::spawn→coordinator::run_queue→execution::execute_job→request/helper与09 calculation。原JobQueuePort/P4 adapter/启动Database adapter保持原PostgresStore方法调用，现在实际实现来自adapters/jobs各owner。旧root/四旧目录无重复实现或兼容转发。

### 16. 删除清单

删四旧App入口mod.rs及注册，原函数更名迁到同一commands职责；删PG root jobs.rs和lib mod jobs，原13函数迁入多个共享队列owner，增加两个纯政策函数作为原片段唯一实现。实际5D，不保留备份或空壳；没有整文件原样搬迁，函数职责迁移按下方git --no-renames A/M/D清单记录。

### 17. 最小验证

实际84/84既有JS源码检查、完整npm run verify:architecture、18存在的变更Rust独立Rustfmt1.88 --check、17等价核对、12负向全部拒绝且所有bytes恢复、git diff --check/23字段/32路径/链接审核通过。新增13App/6PG unit进入原targets，预期Application189=176+13 / Persistence241=235+6 / Gateway112unit+41contract，必须本项精确Windows实跑，不记本地动态PASS。

### 18. 阶段回归

沿用原Public Platform CI Windows Automated：frontend/contracts/TypeScript/Vite/17视口、fmt/locked Clippy workspace all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动和日志扫描。原PG concurrent_workers_claim_once_and_restart_recovery_is_bounded及18PG静态契约保持原target和ignored安排，未新建验收入口或反复建库。无非WindowsCargo/check/Clippy/build/test/frontendbuild/browser/loopback。

### 19. 失败停止条件

10自身精确完整Windows SUCCESS前VERIFYING，11～13BLOCKED、R10IN_PROGRESS；不继承09PASS、不减少测试或抑制Clippy，无阶段完成记录。Windows job实际启动即停止一切CI查询并提示20～30分钟，失败只修同一R10分支。

### 20. 回退点

4a6fa9b645dea38795912217ef966f8aa607816e（09纯文档收尾，源码同已验fbfe83a）。受控revert本10代码/清单/守卫/记录，保留09完成；不手工复制旧jobs.rs导致双owner。

### 21. 根README摘要记录

根README记录09DONE、10唯一启动/领取结算/执行及共享PG队列、原事务/状态/19新unit/实际源码检查与自身Windows待验，14A/13M/5D，链接本记录/阶段索引。

### 22. docs阶段详细记录

本23字段与32实际A/M/D清单，记录实际共享P4与恢复依赖、四入口编组取舍/原生命周期政策、等价/测试/负向/保护/回退及未执行项；index/任务书/TESTING同步10VERIFYING。09五文档收尾另[skip ci]提交4a6fa9b，与10实现分开，无新增阶段完成记录。

### 23. 完成标准

唯一职责/真实入口/旧实现清理/源码核对与文档完成；10自身完整Windows SUCCESS才能DONE。真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开unavailable stub/static指纹不计这些真实PASS。

## 等价、边界测试与验证证据

17等价核对全通过：PG11原完整函数体；原失败取消/预算分支；fail逆归类提取后完整事务；原五态/未知error；mapper逆状态提取后原字段/错误顺序；70个生产SQL/错误/JSON字面量多重集；11公开签名；旧owner删除且15实际函数唯一；原execute四类型/六progress取消/结果及parser完整；coordinator原loop仅从owned port到borrowed port参数适配；worker逆loop提取原spawn完整；四命令仅名/路径改变旧文件删除；Service逆四调用全文；P4/启动composition/09计算/其他节点/引擎/模型/Ports/UI/DB/contracts/deps/workflow/targets/state冻结；365Domain/129声明/301映射保持；43Ports/19文件/426App扫描；19unit只登记原targets。

13App unit覆盖Null/空对象默认与部分请求/纳秒/未知字段原忽略、非法refresh载荷在progress前失败、fullrefresh三个进度/载荷/业务顺序/原结果、三独立任务原checkpoint/完整专用结果、每个六取消checkpoint阻断后续、progress/三业务阶段错误原传播并停后续、未知类型在Port前拒绝、loop只领四types并按原id完成、执行失败记原字符串且结算错误后继续、None/领取失败结束无执行/结算、完整enqueue/retry请求/回传/成功后实际spawn、list0/u32MAX/cancelnil透传及原job、四命令六类PortError不spawn。Fixture全cfg(test)，spawn测试用Notify和已有timeout等待真实worker达到空队列，不阻塞生产代码或新建调度框架。

6PG unit覆盖六原支持类型与空白/大小写拒绝、五原状态、无修剪/别名的未知状态错误、取消压过任意预算、严格小于最大attempt才重试、到达/超过预算terminal且不加入新校验。极端/负数只测原纯比较，不将fixture视为合法数据库记录。原真实PG并发领取/available_at/重启预算回归完整保留，真实执行仍待最终新库。

12负向全拒绝并bytes恢复：旧PGroot复活、分析领取混P4、结算错误政策改?、enqueue写前启动、取消提示改变、空对象默认改变、删SKIP LOCKED、预算<放宽<=、领取提交移到started事件前、progress夹紧改100、取消失败改queued、人工retry放开succeeded。现有Analytics Service守卫同步真实新owner、18职责源承载21公开入口，而非硬凑一入口一目录；09两refresh检查跟随原执行函数迁移，09窗口类型守卫保留。

官方清单：App426/43Ports/19Port文件；PG直接38/state19；Domain365/129声明文件/301映射，scan1252，sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd，rustUsageDigest b80bce0a8341a13a7a4022538da1df240223a6ca36e287eab92d6c0011e1d89e。18保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57；171命令；46迁移/18PG静态aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

实际命令：原84 JS源码脚本（scratch/r1010_static.py清单）、npm run verify:architecture、verify-analytics-service.mjs、verify-application-ports.mjs --refresh-source-scan、generate-domain-type-inventory.mjs、verify_protected_assets.mjs/verify_command_contract.mjs/verify_database_baseline.mjs、独立Rustfmt1.88 --edition2021 --config skip_children=true --check实际18Rust路径、git diff --check；scratch/r1010_compare.py/r1010_probes.py是静态逆切/破坏恢复核对。r1010-static-checks.json/r1010-architecture.log/r1010-equivalence.json/r1010-negative-probes.json为可复建临时报告，不入repo或新增验收框架。Rust unit和DB仍按上述Windows/最终新库真实执行，不把源码检查当动态PASS。

Mermaid Chart已显示中文真实命令→spawn→领取结算→执行/取消及共享PG事务链，原启动与P4共享调用保持。使用原tokio/SQLx/serde稳定接口和已有features，无新版本/未知API事实需重复Context7；Create State按稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce保存实施与CI交接，不用Library读附件，不另存Git制品，无子代理。

## 全部实际文件清单

以基线4a6fa9b对照git diff --no-renames及未跟踪文件生成。无整文件原样重命名；四App函数逻辑迁入commands、PG root13函数按共享职责提取，Git相似度识别不改变实际A/D清单口径。

### 新增（14）

- `crates/application/src/use_cases/analytics/jobs/commands.rs`
- `crates/application/src/use_cases/analytics/jobs/coordinator.rs`
- `crates/application/src/use_cases/analytics/jobs/execution.rs`
- `crates/application/src/use_cases/analytics/jobs/request.rs`
- `crates/application/src/use_cases/analytics/jobs/tests.rs`
- `crates/persistence-postgres/src/adapters/jobs/claim.rs`
- `crates/persistence-postgres/src/adapters/jobs/commands.rs`
- `crates/persistence-postgres/src/adapters/jobs/lifecycle.rs`
- `crates/persistence-postgres/src/adapters/jobs/mapping.rs`
- `crates/persistence-postgres/src/adapters/jobs/mod.rs`
- `crates/persistence-postgres/src/adapters/jobs/policy.rs`
- `crates/persistence-postgres/src/adapters/jobs/read.rs`
- `crates/persistence-postgres/src/adapters/jobs/tests.rs`
- `docs/modular-rewrite/R10-review-analytics-release-backend/R10-10-background-jobs.md`

### 修改（13）

- `README.md`
- `architecture/application-port-inventory.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `crates/application/src/services/analytics/service.rs`
- `crates/application/src/use_cases/analytics/jobs/mod.rs`
- `crates/application/src/use_cases/analytics/jobs/worker.rs`
- `crates/persistence-postgres/src/adapters/mod.rs`
- `crates/persistence-postgres/src/lib.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md`
- `docs/modular-rewrite/R10-review-analytics-release-backend/README.md`
- `scripts/verify-analytics-service.mjs`

### 删除（5）

- `crates/application/src/use_cases/analytics/jobs/cancel_background_job/mod.rs`
- `crates/application/src/use_cases/analytics/jobs/enqueue_analysis_job/mod.rs`
- `crates/application/src/use_cases/analytics/jobs/list_background_jobs/mod.rs`
- `crates/application/src/use_cases/analytics/jobs/retry_background_job/mod.rs`
- `crates/persistence-postgres/src/jobs.rs`

实际合计32路径，18存在的变更Rust文件独立格式检查通过；删除文件不纳入Rustfmt。职责编排/函数迁移：四App入口与13PG原函数；整文件原样移动/重命名：无。


## R10-10 精确 Windows 完成与11授权（2026-10-13）

精确提交 `3365b2bb5b6396fe010aee311c819bdd6b1198d8` / tree `1b4c59ba5d786bba1da11c13c2bc5ddd50e069e8`，[Public Platform CI run38411713492](https://github.com/uniquenesssta/123/actions/runs/38411713492) / Windows job115345476640全SUCCESS，run于2026-10-13 22:43:53北京时间完成。实际Application189/189、Persistence241/241、Gateway112/112 unit及41/41 contract；本项新增13App/6PG unit全部通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志扫描均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-141620.json及同名.txt。

证据artifact11824026576，14,051,930字节，SHA-256 `ec237a2d6899402531ad616aab6c7fba11cbb6d1098c3d3e868eafe75e8a030f`。10自身完整SUCCESS后DONE；用户2026-10-13“收尾10开始11”授权11 Analysis Package READY，同一唯一rewrite/r10-review-analytics-release-backend分支，12～13BLOCKED、R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验3365b2b，11须自身精确完整Windows，无阶段完成记录。此前10 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开unavailable stub/static指纹不计真实PASS。
