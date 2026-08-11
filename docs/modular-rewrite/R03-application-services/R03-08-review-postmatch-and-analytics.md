# R3-08 Review / Postmatch / Analytics Services

## 状态

`DONE`

## Atomic Task 1 — Review Core

状态：`DONE`

### 已实施

- 将 `generate_match_review`、`list_reviewable_matches`、`list_match_reviews`、`read_match_review`、`list_ability_candidates`、`decide_ability_candidate` 六个既有公共 Application 职责迁入 `services/review/` 与 `use_cases/review/`。
- `ActiveDatabase` 通过 `composition/adapters/review.rs` 实现 `MatchReviewPort`；Service / Use Case 不直接依赖 PostgreSQL、SQLx、PgPool 或 `PersistenceStore`。
- 删除旧 `crates/application/src/review.rs`，不保留空转发层。
- `ApplicationService`、`ApplicationComposition` 增加唯一 `ReviewService` owner；Tauri Review 命令、公共方法名/参数/返回 DTO 保持不变。
- Match Review Package、Postmatch、Analytics 未在 AT1 修改。

### 验证事实

- Windows AT1 hard gate run `31328023642`：Review 专项、Application Ports、完整 architecture、模型保护资产、rustfmt、Application check/tests、完整 frontend、完整 Rust、精确作用域与 clean tree 均通过。
- clean Review Core tree 已形成正式 Atomic commit `6059d7a79f34bb31a5b81b56a1c15af0a30db11a`。
- 首次 Public Platform CI run `31328591975` 在 `verify:architecture` 停止，明确错误为 Domain 类型与契约清单漂移；Windows Automated 未执行，因此 AT1 保持 `VERIFYING`。
- 根因：Review owner 拆分改变了 Application Rust 文件/调用面，但 `architecture/domain-type-inventory.json` 仍保存拆分前的 `rustUsageDigest`、扫描文件数量及外部调用者路径。已按既有生成器刷新清单；恢复 run `31348056577` 的 inventory、Review 专项、Application Ports、完整 architecture、保护资产、完整 frontend/Rust、scope 与 clean tree 均通过。
- 刷新后的正式提交 `586ad2196be7218e366cf1715fbbf69d87d91f01` 触发第二次 Public Platform CI run `31349803381`。Domain inventory、Domain 根出口、Application Ports、Database、Competition/Rules、Teams/Players、Lineups、Prediction、Research 门禁均已通过；随后 `scripts/verify-review-service.mjs` 在 Node 22 clean checkout 中因普通字符串被错误写成跨行文本而触发 `SyntaxError: Invalid or unexpected token`，Windows Automated 再次因前置架构门禁失败被跳过。
- 第二次失败定位为 Review 专项验证器自身语法缺陷，不是 Review Core 业务实现失败。修复仅将失败输出改为显式 `\n` 拼接，并在恢复门禁增加 `node --check scripts/verify-review-service.mjs`；不删除、不放宽任何验证断言。

- 第三次正式 Public Platform CI run `31350677129` / Windows Automated job `93340716563` 已整体 `SUCCESS`；architecture、Windows automated acceptance、validation evidence upload 与 post-job 均成功。artifact `9049246515`，大小 `14276530` 字节，SHA-256 `62f0bcfdb8f83ce0a715de58ee14f8a0a87b1f3192c938e841d65400b8ecb7ef`。正式修复提交 `bac0ca3c1192b8da919b409b06aa6e454ceca87e`。

### 未改变

- PostgreSQL SQL、migration、Schema、数据格式。
- Tauri DTO/命令签名与前端状态。
- 错误语义、日志等级和用户可观察业务行为。
- `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 与模型保护资产。
- 生产依赖。

### AT1 关闭

- Review Core 6 个公开职责已完成迁移，两个正式 CI 暴露的验证缺口均已修复且未放宽门禁。
- AT1 正式关闭为 `DONE`。下一 Atomic Task：Match Review Package，状态 `READY`；Postmatch 与 Analytics 继续保持未修改。

## Atomic Task 2 — Match Review Package

状态：`DONE`

### 已实施

- 保持 `export_match_review_package`、`read_match_review_package_workflow`、`preview_match_review_package`、`confirm_match_review_package`、`commit_match_review_package_facts`、`generate_match_review_from_package`、`commit_match_review_package` 七个公共 ApplicationService/Tauri 入口及参数/返回 DTO 不变。
- 删除旧 `crates/application/src/match_review_package.rs`，不保留转发壳；唯一 `ReviewService` 继续作为 Review / Match Review Package 应用层 owner。
- 拆分 `use_cases/review/package/`：`export` 负责导出组装，`preview` 负责导入预检，`lifecycle` 负责确认/事实写入/正式复盘状态推进，`shared` 负责唯一共享快照与身份规则，`workbook` 负责 XLSX 路径、阻塞 I/O 与 SHA256。
- 保留 R3-01 冻结 `MatchReviewWorkflowPort` 原签名；AT2 新增 `MatchReviewPackageSourcePort`、`MatchReviewPackageStatePort`、`MatchReviewPackageFactsPort`。`StatePort` 只负责 package 工作流状态持久化，避免与冻结 WorkflowPort 形成重复 owner。实际阵容写入复用 `LineupPort`，正式复盘生成复用 AT1 `MatchReviewPort`。
- PostgreSQL 具体实现集中于 `composition/adapters/review.rs`；Service / Use Case 不依赖 SQLx、PostgresStore、PersistenceStore。
- `verify:review-service` 与 `verify:match-review-package` 已更新到新 authoritative owner；九步状态机、SHA 绑定、重复复检、结构化事件、数据库约束与 Tauri/frontend 入口断言未弱化。

### 验证事实

- 初始 Windows hard gate `31359686297`：AT2 专项、Application Ports、施工树 architecture、Application Rust、完整 `verify:frontend`、完整 `verify:rust`、精确 scope 与 clean commit 均通过；提交后 final-tree 因 Domain inventory 未固化而失败，因此未记为 DONE。
- fresh-checkout 诊断 `31360521486`：Review/Package verifier 与 clean tree 通过；architecture 暴露 Domain inventory 漂移，同时确认 R3-01 冻结 `MatchReviewWorkflowPort` 在 AT2 初始实现中被误删，37-Port 契约下降为 36。
- scope 诊断 `31361978957` 明确 recovery 合法变化路径；没有越界到 Postmatch/Analytics/Tauri/PostgreSQL/Domain/模型/依赖。
- 最终 recovery `31362128833`：冻结 WorkflowPort 恢复、AT2 状态接口改名、inventory 固化；Review/Package 专项、37-Port、完整 architecture、保护资产、Application Rust、完整 frontend/Rust、精确 scope、clean commit、clean-tree 全部 `SUCCESS`。

### 未改变

- `postmatch.rs`、`analytics.rs`。
- Tauri 命令/DTO、PostgreSQL SQL/migration/Schema、Domain 契约。
- 模型保护区、生产依赖、错误语义和用户可观察业务行为。

### 后续门禁

- canonical `rewrite/r3-08-review-postmatch-analytics` 提交 `23997cde7d3ba46db34fdd2e0577075555391986` 的 Public Platform CI run `31406717077`、Windows Automated job `93514736356` 与 evidence upload 已全部成功；关闭记录已同步，AT2 可正式标记 `DONE` 并开放下一 Atomic Task。

### Canonical CI 第一次失败与 Stage-A 验证器恢复

- canonical run `31363234205` 的 `verify:architecture` 已通过；Windows Automated 在 frontend 聚合门禁执行 `verify-stage-a-architecture.mjs` 时因该旧验证器仍读取已删除 `match_review_package.rs` 而失败。
- `verify-stage-a-architecture.mjs` 只迁移读取路径到 `use_cases/review/package/preview.rs` + `lifecycle.rs`；强类型工作流动作、状态权威、Tauri 薄层、前端能力 DTO、PostgreSQL 状态迁移等原断言保持。
- canonical 发布期间连接器曾产生瞬时 README-only 异常提交；随后采用 non-force fast-forward 将完整已验证 publication tree 恢复。生产源码未受这些文档异常提交影响，异常 CI 不作为通过证据。
- 随后的 canonical run `31377744818` 再次通过 architecture，但完整 Windows acceptance 在 `verify-match-event-facts.mjs` 读取已删除的旧 `crates/application/src/match_review_package.rs` 时因 `ENOENT` 失败；该失败明确属于重构后 verifier 路径未同步，不是 Match Review Package 业务实现失败。
- 修复提交 `23997cde7d3ba46db34fdd2e0577075555391986` 只把 `verify-match-event-facts.mjs` 的应用层读取源迁移到 `use_cases/review/package/shared.rs`；OwnGoal 身份校验及其余事件事实断言保持原样，未恢复旧 owner、未增加转发壳、未修改业务代码或公共契约。
- PR clean CI run `31404194850` / Windows Automated job `93506381047` 已完整 `SUCCESS`；artifact `9069809674` 大小 `14156070` 字节，SHA-256 `0109171ab7791609e0c92dcdeba6797f651e1ea20175d9967a89f3bc7d256c37`。
- canonical Public Platform CI run `31406717077` / Windows Automated job `93514736356` 同样完整 `SUCCESS`，architecture、完整 frontend、17 个截图回归视口、TypeScript/Vite、Rust fmt、workspace Clippy `-D warnings`、workspace tests、Windows release 构建、release runtime 启动与 validation evidence upload 均通过；artifact `9070811986`（`windows-automated-delivery-evidence-23997cde7d3ba46db34fdd2e0577075555391986`）大小 `14156296` 字节。
- 18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试按既有安全设计保持 `ignored`，未记为已执行；本次未执行破坏性数据库验证。

### AT2 关闭

- Match Review Package 的 7 个公开职责已完成模块化迁移，旧 owner 已删除且未恢复兼容壳。
- canonical Public Platform CI、Windows Automated 与 evidence upload 已满足关闭条件，验证器路径迁移未弱化任何断言。
- AT2 正式关闭为 `DONE`。R3-08 整体继续 `IN_PROGRESS`；Postmatch 与 Analytics 仍未修改，下一 Atomic Task 可按任务书开放。

## Atomic Task 3 — Postmatch Service

状态：`DONE`

### 已实施

- 保持 `postmatch_settlement_readiness`、`settle_postmatch_review`、`list_postmatch_settlements`、`list_evidence_scoring_items`、`decide_evidence_scoring_item`、`refresh_postmatch_monitoring`、`postmatch_overview` 七个公共 ApplicationService/Tauri 入口及参数/返回 DTO 不变。
- 删除旧 `crates/application/src/postmatch.rs`，不保留转发壳；新增唯一 `PostmatchService`，每个公开职责进入独立 `use_cases/postmatch/<use-case>/`。
- 继续使用 R3-01 的 `PostmatchSettlementPort` / `PostmatchMonitoringPort`，把原先与真实公共/持久化 API 不一致的 limit/status/overview 参数对齐；没有新增万能 Repository 或重复 Postmatch Port。
- `settle_postmatch_review` 继续先检查 Match Review Package workflow 的 `SettleReview` action，再写 settlement，最后推进 package workflow 到 `settled`；状态读取/推进复用 AT2 `MatchReviewPackageStatePort`，没有第二状态 owner。
- PostgreSQL 具体 Postmatch 调用只由 `composition/adapters/postmatch.rs` 适配；Service / Use Case 不依赖 PostgreSQL、SQLx、PostgresStore 或 PersistenceStore。
- 全仓旧 owner 扫描确认受影响验证器为 Postmatch Settlement、Review Service 阶段保护和 Stage-A workflow 三处；均只迁移 authoritative 读取位置并保留原断言。Domain type inventory 由既有生成器刷新。
- 新增 `verify-postmatch-service.mjs`，锁定模块边界、7 个公共入口、2 个既有 Ports、资料包状态机和 composition，并接入 `verify:architecture` 与完整 frontend 聚合验证。

### 未改变

- `crates/application/src/analytics.rs` 及 Analytics Service 范围。
- PostgreSQL SQL/migration/Schema、Tauri 命令/DTO、Domain 类型与数据格式。
- 模型保护区、生产依赖、错误语义、日志等级和用户可观察业务行为。

### 验证事实

- Windows hard gate：run `31419165233`：Postmatch 专项、Application Ports、完整 architecture、保护资产、rustfmt、Application check/tests、完整 frontend 与完整 Rust 回归均 `SUCCESS`。
- 18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试未配置专用测试库时必须保持 `ignored`，不得记为已执行。

### AT3 关闭

- canonical 提交 `b551ac8acc4030f05d93100812b316469dc7ea83` 的 Public Platform CI run `31420385032` / Windows Automated job `93559423579` 已整体 `SUCCESS`；architecture、完整 Windows automated acceptance 与 validation evidence upload 均成功。
- evidence artifact `9075930419`（`windows-automated-delivery-evidence-b551ac8acc4030f05d93100812b316469dc7ea83`）大小 `14152359` 字节，SHA-256 `39fab48a3f2d62e1538276fb8f19fb811f1c72c79380de55897e12c82e39f582`。
- 18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试按既有安全设计继续保持 `ignored`，未记为已执行；本 Atomic Task 未执行破坏性数据库验证。
- AT3 Postmatch Service 正式关闭为 `DONE`；该时点 Atomic Task 4 — Analytics 随即开放。AT4 后续已完成并通过正式验收，R3-08 的最终当前状态为 `DONE`。


## Atomic Task 4 — Analytics Service

状态：`DONE`

### 已实施

- 删除旧 `crates/application/src/analytics.rs`，不保留转发壳；`AnalyticsService` 成为唯一应用层 owner。
- 21 个公共 Analytics 用例全部按独立目录落入 `use_cases/analytics/<use-case>/`；AI Analysis Package、后台任务、Data Quality 与 Parameter Lifecycle 按职责分组，公共用例不再共享聚合型 owner。
- `AnalyticsPort`、`JobQueuePort`、`ParameterLifecyclePort` 分别承载 Analytics 数据、后台队列和参数生命周期事务边界；具体 PostgreSQL 调用只存在于 composition adapters。
- `ApplicationService` 公共方法名、参数和返回 DTO 保持兼容；Tauri、Domain 类型、PostgreSQL SQL/migration/Schema、模型保护区与生产依赖未改变。
- 受 owner 拆分影响的 Analytics / Parameter Lifecycle / Review verifier 与 deterministic Domain inventory 已迁移到新的 authoritative module tree，原断言未删除或放宽。

### 验证事实

- Stage Regression run `31461057749` 已整体 `SUCCESS`：完整 `npm run verify:frontend`、完整 `npm run verify:rust`、workspace Clippy `-D warnings`、workspace tests、模型保护资产和最终 clean-tree 均通过。
- clean publication commit `fcf5f3df29b477baf7e1c3aeebcf5ed6f459b8a8` 相对 AT3 canonical 为单一原子实现提交，临时 AT4 workflow 不在 publication tree。
- PR #16 Public Platform CI run `31461760837` 完整 `SUCCESS`，clean publication tree 的 Windows Automated 与 evidence upload 通过。
- canonical Public Platform CI run `31463538968` / Windows Automated job `93691679605` 完整 `SUCCESS`；architecture、Windows automated acceptance 与 validation evidence upload 均成功。artifact `9091238488`（`windows-automated-delivery-evidence-fcf5f3df29b477baf7e1c3aeebcf5ed6f459b8a8`）大小 `14092031` 字节，SHA-256 `ae08e7eeccb09f4b625885716391f88de2ad4ce3902d59608c4978bc63f29a72`。
- 18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试按既有安全设计保持 `ignored`，未记为已执行；本 Atomic Task 未执行破坏性数据库验证。

### AT4 与 R3-08 关闭

- Analytics 的公共职责、状态/副作用边界与持久化适配已完成模块化迁移，旧 owner 已删除且没有兼容转发壳。
- AT1 Review Core、AT2 Match Review Package、AT3 Postmatch、AT4 Analytics 均为 `DONE`，R3-08 正式关闭为 `DONE`。
- 根据 R3 任务书顺序，下一节点 R3-09 Exchange / AI Workspace / Release Services 开放为 `READY`。
