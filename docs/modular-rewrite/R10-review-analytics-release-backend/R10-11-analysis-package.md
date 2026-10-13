# R10-11 Analysis Package

状态：`VERIFYING`。用户2026-10-13“收尾10开始11”授权；uniquenesssta/123 v0.23.0，唯一rewrite/r10-review-analytics-release-backend。基线5a19c712f9a4002e05ffc98310d5b14c561b6661为10五doc[skip ci]收尾，源码同已验3365b2b/Windows run38411713492全SUCCESS。01～10DONE、11VERIFYING、12～13BLOCKED、R10IN_PROGRESS。

### 1. 目标

将分析包导出、回包预检/导入和人工建议审核的实际入口与PG职责收拢为唯一模块，保留原资料包格式、身份、待审与人工候选政策。

### 2. 现状与来源

App六原独立目录、PG root analytics混合资料包/质量/参数职责，原analysis-package库已持有ZIP格式、摘要、验证和文件读写。保留该库并只重写App/PG适配调用；没有把文件安全验证或格式规则复制到新owner。

### 3. 目标文件与目录

App既有use_cases/analytics/ai_package/{mod,exports,response,suggestions,tests}.rs；PG adapters/analytics/analysis_package/{mod,data,exports,imports,decisions,suggestions,mapping,policy,tests}.rs。任务参考analysis-package目录落到现有合法Rust标识ai_package，不另建并行模块或为目录外观改名。

### 4. 文件职责边界

App exports编排原模板写出/分析数据→写包→记录，response原预览和解析后导入，suggestions原列表与明确人工决定。PG data只读取资料包六类数据，exports记录原导出元数据，imports原导入事务，decisions原人工建议/候选事务，suggestions原列表/单条读取，mapping Row字段，policy原阻断/严重度/待审/能力值及JSON字段规则。mod只登记或导出，测试只cfg。

### 5. 输入

原output_path/source_package_id/input_path/status/limit、AiSuggestionDecisionDraft全部字段；原AiAnalysisPackageData、preview/manifest/suggestions/warnings/blocking_errors。无新配置、字段、默认、版本或任务类型。

### 6. 输出

原Summary/Preview/SuggestionRecord、两种v1 ZIP、导出元数据、导入账本/pending建议、人工决定与可能的新pending能力候选/审计。AbilitySuggestionValues仅本次调用内部三标量投影，非公共DTO或长期State。

### 7. 允许依赖

App原AnalyticsPort、Domain、ApplicationResult/Error、原analysis-package API与std fs/Path；PG原SQLx、serde_json、chrono、Uuid、PersistenceResult/Error和write_audit_event。data实际仍需DateTime/Utc，exports实际仍需ANALYTICS_CALCULATION_VERSION，原编译依赖守卫随职责迁入实际owner。

### 8. 禁止依赖

不新增Port、wrapper、通用Repository/State、框架/依赖/runner/workflow/target/migration或DB专项。原ZIP库、模型/公式/参数/Profile/Schema/Golden保护区不动，不自动接受AI建议、正式写能力观察或晋升参数。

### 9. 状态所有权

原PG导出/导入/建议/候选表持有状态与身份。新增局部能力值和测试Fixture非共享生产State；PG直接owner38/state19保持，只有既有analytics模块登记子模块。App424/43Ports/19Portfiles保持原边界。

### 10. 副作用边界

分析导出原六数据顺序读→原库写ZIP→Port记录元数据；模板原库生成bytes→fs.write；预览仅读并返回DTO，导入另读同路径后交PG。PG导入先阻断错误门禁、再单tx查response_id重复和可选source属于本库→导入账本/原warnings→逐建议pending→审计commit→全pending列表limit500回读。人工决定另单tx锁建议→pending门禁→原决定分支；仅接受ability_update核验球员/维度、读取原值并插新pending候选→建议状态/链接/说明→审计commit→单条回读。两个事务不合并。

### 11. 异常路径

原文件/ZIP/JSON/Port错误包装与?传播保持。数据读失败不创建文件，写失败不记元数据；元数据失败保留已写文件，不新增回删或重试。预览不做导入；格式解析失败不调Port。PG阻断错误在begin前、重复/来源错误/写入/审计失败按原事务释放；提交后回读失败不撤销已提交记录。pending状态检查仍先于payload getter，能力数值规则仍在身份核验后。

### 12. 并发/异步/生命周期

原同response_id预查/数据库唯一约束、建议FOR UPDATE锁/仅pending决定保持，无新锁、取消State、超时、句柄、轮询或补偿。此文件IO沿用原同步接口；真正DB并发/回滚仍最终新库验收，静态比较不冒充实跑。

### 13. 兼容要求

六App入口签名/原21facade/AnalyticsPort/组合适配、全部PG公共签名/SQL/绑定/错误/JSON/配置/日志/UI/0.23.0/依赖迁移保持。库完整冻结：manifest/成员名、SHA按原精确bytes与BTreeMap、v1格式/Schema/未知字段、路径/重复项/大小/建议验证均不改。严重度仅空白回落info，其他字节不trim；list原可选status、limit1..1000、createdDESC/idDESC，导入后仍回读所有pending500而非仅本response。

拒绝只标rejected，接受其他类型只标accepted不生成参数；接受能力建议仅新增待审候选，不直接影响正式能力。decided_by仅None回落desktop-user，空白原样；proposed0..100、confidence缺/错类型默认0.5再0..1、sample缺/错整数类型默认0再0..i32MAX，原字段字符串不修剪、UUID错误保持。

### 14. 实施步骤

核实10全Windows并五doc收尾，从5a19c71编组六App原函数并切Service/删旧目录。PG迁15资料包职责、剩余14根函数完整保留；提取四原纯政策段，能力值采用内部具名结构并按原位置解构，payload从owned移borrowed后移除额外&避免needless_borrow。更新原Analytics守卫/官方扫描、补19原unit，完成等价/破坏恢复/架构/保护/文档后推送原Windows。

### 15. 切换入口

Service直接ai_package六具名函数，由exports/response/suggestions唯一实现；原AnalyticsPort adapter仍调同名PostgresStore，新PG模块实际实现。构包继续调用09 analytics_overview；导入/决定继续调用新建议read，原JSON/helper仅一处，无根转发。原analysis-package库调用路径与职责保持。

### 16. 删除清单

删除六App旧mod.rs及旧注册，原函数按三职责更名编组。root analytics移出15职责及无用AI/DateTime/版本imports，保留质量/COPY/参数与14原函数；DateTime和版本改由实际data/exports持有。实际6D，无整文件原样搬迁/根删除/备份，A/M/D按下方no-renames口径。

### 17. 最小验证

实际84/84原JS、完整npm run verify:architecture、17存在的变更Rust独立Rustfmt1.88 --check、18等价、13负向全拒绝bytes恢复、gitdiffcheck/23字段/31路径/links审核通过。新增11App/8PG原unit，预期Application200=189+11 / Persistence249=241+8 / Gateway112unit+41contract，须自身精确Windows，不记本地动态PASS。

### 18. 阶段回归

沿用原Public Platform CI Windows Automated：前端/contracts/TypeScript/Vite/17视口、fmt/lockedClippy all-targets-Dwarnings/workspace、release/MSI/NSIS/启动/日志。原analysis-package unit与18PG静态契约/targets安排保持，不新增设施；无非WindowsCargo/check/Clippy/build/test/frontendbuild/browser/loopback。

### 19. 失败停止条件

11自身完整Windows SUCCESS前VERIFYING、12～13BLOCKED，不继承10PASS、不减少测试或压警告，无阶段完成记录。Windows job实际启动即停止所有CI查询，提示20～30分钟；失败只修同R10分支。

### 20. 回退点

5a19c712f9a4002e05ffc98310d5b14c561b6661（10纯doc收尾，源码同已验3365b2b）。受控revert11整套代码/门禁/扫描/记录，不手工复制旧职责造成双owner。

### 21. 根README摘要记录

根README记录10DONE、11三App职责与PG资料包/原人工事务、19新unit/源码验证及自身Windows待验；14A/11M/6D，链接本节点/索引，12～13BLOCKED。

### 22. docs阶段详细记录

本23字段/31路径记录实际ai_package标识取舍、库冻结、元数据失败文件保留/提交后回读风险、人工候选链、测试/等价/负向/保护/回退和未执行项。index/任务书/TESTING同步；10收尾另五doc5a19c71[skip ci]，与11实现分开。

### 23. 完成标准

唯一职责/真实入口/旧实现清理/源码门禁及记录完成；11自身精确完整Windows SUCCESS才能DONE。真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验，ignored/公开stub/static指纹不计这些真实PASS。

## 等价、测试与工具证据

18等价全通过：root14保留函数体与owner集合；13原PG完整data/export/read/mapper/JSON函数；阻断/严重度两原规则；import逆两提取后完整原事务；pending原门禁；能力三字段原规则与getter顺序（只owned→borrowed参数适配）；内部值结构精确类型/字段及原值投影；decision逆两个提取/字段后完整原锁/候选事务；240生产SQL/错误/JSON字面量多重集；所有PG公共签名；六App原签名/body只改名/路径且旧owner删除；Service逆六调用全文；原ZIP库/格式/Schema/验证及引擎/模型/后台/其他节点/Ports/UI/DB/deps/workflow/targets/state冻结；365Domain/129声明/301映射；43Ports/19/424扫描；19unit只登记原targets。

11App测试覆盖实际模板ZIP/可选source/版本/无阻断/空建议警告，同文件preview→import完整DTO与Port结果不筛选，缺失/坏ZIP先于Port失败，实际导出后记录完全同Summary/两个相同数据包内容hash相同但packageID不同，六类build错误不创建文件，模板Io与包AnalysisPackage写错边界和无元数据，六类metadata错误保留原文件，六类import错误无重试，statusNone/空白/空串与limit0/max透传，人工Accept/Reject/None或空白reviewer/note保持，建议两入口六类PortError无其他动作。真实文件测试只随原Windows unit运行，不在非Windows跑。

8PG测试覆盖阻断错误原顺序/全角分号、warnings/空建议不新阻断、blank-only严重度及其他原字节、仅精确pending可决定、必需字符串类型/不trim、UUID含nil原解析/错误、原能力三值夹紧/default/整数类型策略、必需numeric proposed值先于可选default。只测现有规则，不新增统计公式或复杂算术预期，fixture不算真实数据库执行。

13负向全拒绝bytes恢复：旧App/PG owner返回、元数据先于写ZIP、阻断guard在begin后、来源不存在仍接受、导入自动accepted、severity不trim判空、pending门禁改accepted、能力值移到identity核验前、候选自动accepted、list上限变化、实际data时间import/exports版本import缺失。原Analytics Service守卫同步15实际职责源承载21facade，09时间/版本依赖检查跟随真正资料包owner，原09/10门禁保持。

官方扫描：App424/43Ports/19Portfiles，PG直接38/state19；Domain365/129声明/301映射、scan1259，sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd，rustUsageDigest ae838daa05f042312d2d0d8de616d9e398fbabeda00ef3683a38b045251cfc7f。18保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57；171命令；46迁移/18PGstatic aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

实际命令：原84JS清单（scratch/r1011_static.py）、npm run verify:architecture、verify-analytics-service.mjs、verify-application-ports.mjs --refresh-source-scan、generate-domain-type-inventory.mjs、verify_protected_assets.mjs/verify_command_contract.mjs/verify_database_baseline.mjs、独立Rustfmt1.88 --edition2021 --config skip_children=true --check实际17Rust、gitdiffcheck；scratch/r1011_compare.py/r1011_probes.py静态逆切/破坏恢复。r1011-static-checks.json/architecture.log/equivalence.json/negative-probes.json为可复建临时报告，不入repo或新建验收框架。

Mermaid Chart中文图明确预览只返回DTO、导入和独立人工决定事务、待审能力候选与各提交后回读。稳定原API/依赖无新版本或陌生接口需重复Context7；Create State稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce保存本实施和CI交接。附件不通过Library读取，Git制品不另存，无子代理。

## 全部实际文件清单

以5a19c71基线git diff --no-renames及未跟踪文件生成。六App函数逻辑迁入三文件，十五PG职责迁出root；没有整文件原样移动，A/D表示实际创建与退出路径。

### 新增（14）

- `crates/application/src/use_cases/analytics/ai_package/exports.rs`
- `crates/application/src/use_cases/analytics/ai_package/response.rs`
- `crates/application/src/use_cases/analytics/ai_package/suggestions.rs`
- `crates/application/src/use_cases/analytics/ai_package/tests.rs`
- `crates/persistence-postgres/src/adapters/analytics/analysis_package/data.rs`
- `crates/persistence-postgres/src/adapters/analytics/analysis_package/decisions.rs`
- `crates/persistence-postgres/src/adapters/analytics/analysis_package/exports.rs`
- `crates/persistence-postgres/src/adapters/analytics/analysis_package/imports.rs`
- `crates/persistence-postgres/src/adapters/analytics/analysis_package/mapping.rs`
- `crates/persistence-postgres/src/adapters/analytics/analysis_package/mod.rs`
- `crates/persistence-postgres/src/adapters/analytics/analysis_package/policy.rs`
- `crates/persistence-postgres/src/adapters/analytics/analysis_package/suggestions.rs`
- `crates/persistence-postgres/src/adapters/analytics/analysis_package/tests.rs`
- `docs/modular-rewrite/R10-review-analytics-release-backend/R10-11-analysis-package.md`

### 修改（11）

- `README.md`
- `architecture/application-port-inventory.json`
- `architecture/domain-type-inventory.json`
- `crates/application/src/services/analytics/service.rs`
- `crates/application/src/use_cases/analytics/ai_package/mod.rs`
- `crates/persistence-postgres/src/adapters/analytics/mod.rs`
- `crates/persistence-postgres/src/analytics.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md`
- `docs/modular-rewrite/R10-review-analytics-release-backend/README.md`
- `scripts/verify-analytics-service.mjs`

### 删除（6）

- `crates/application/src/use_cases/analytics/ai_package/decide_ai_analysis_suggestion/mod.rs`
- `crates/application/src/use_cases/analytics/ai_package/export_ai_analysis_package/mod.rs`
- `crates/application/src/use_cases/analytics/ai_package/export_ai_analysis_response_template/mod.rs`
- `crates/application/src/use_cases/analytics/ai_package/import_ai_analysis_response/mod.rs`
- `crates/application/src/use_cases/analytics/ai_package/list_ai_analysis_suggestions/mod.rs`
- `crates/application/src/use_cases/analytics/ai_package/preview_ai_analysis_response/mod.rs`

实际合计31路径，17存在的变更Rust独立格式检查通过，删除文件不纳入Rustfmt。函数职责迁移：六App/十五PG；整文件原样移动/重命名：无。
