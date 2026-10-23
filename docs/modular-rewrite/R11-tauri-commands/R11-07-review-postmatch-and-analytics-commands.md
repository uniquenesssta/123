# R11-07 Review、Postmatch 与 Analytics Commands

状态：`DONE`（自身精确5b779b9 / Windows run39962382506完整SUCCESS；下文VERIFYING为实施历史）。

R11-01～06 DONE；07 Review/Postmatch/Analytics Commands已实施VERIFYING，08～10 BLOCKED，R11 IN_PROGRESS。用户2026-10-23“收尾06开始07”，继续唯一rewrite/r11-tauri-commands；基线859f12da03c04b4136be153d8b35f80015fce0a7为06五文档[skip ci]收尾，生产源码同自身完整Windows成功faa6f72。

### 1. 目标

为复盘、能力候选、赛后资料包/结算/证据/监控，以及分析/任务/AI资料包/参数生命周期命令建立唯一明确的传输职责，接通原真实Application/Ports/适配器链路。

### 2. 现状与来源

旧review.rs承载6条复盘/能力候选，postmatch.rs承载14条赛后/资料包，analytics.rs承载21条分析/后台任务/AI资料包/参数生命周期。它们是薄命令而非业务owner；本节点迁移41原完整函数，不重写已完成R10的业务、事务或模型算法。

### 3. 目标文件与目录

commands/review根4+abilities/2；commands/postmatch根3+evidence/2+monitoring/2+package/7；commands/analytics根2+jobs/4+ai_package/6+parameter_lifecycle/9。41命令文件、10纯mod.rs，共51新Rust；没有空DTO/mapper/error或无职责价值的coordinator。

任务书review目录示例补齐实际postmatch和analytics两个同级目录，保留3独立能力owner及13组，不把分析/后台队列放进复盘目录。不做仅为统一树形的跨能力嵌套，这是按实际职责执行的目录细化。

### 4. 文件职责边界

每命令仅原State/参数转换/原服务调用/错误投影；10mod只显式登记和导出函数与Tauri双宏。复杂资料包、XLSX、候选判定/结算/证据/监控、AI包解析、参数晋升/回滚和worker生命周期仍唯一原Application/Ports/PG/文件owner。

### 5. 输入

原Draft/Request、String/Option<String>/u32和State<AppState>保持，包括status/limit、review_id/match_review_id、package_id/expected_match_id、candidate_id/job_id、raw input/output_path与source_package_id。不改optional、serde或默认行为。

### 6. 输出

41原公开Result签名及Domain DTO保持，含复盘详情/候选、结算/证据/概览、资料包预检/确认/事实提交/生成/整体提交、后台任务/AI建议及参数就绪/影子/晋升/回滚。原同步AI响应预检与模板导出保持同步，其余39 async；不复制类型或硬编码结果。

### 7. 允许依赖

原Tauri State、bootstrap::AppState、commands::parse_uuid、原football_domain DTO及AppState.service。后台列表通过crate::commands::record_failed_background_jobs借用原根导入的platform共享投影；helper和其测试字节冻结。沿已完整Windows证明的锁定Tauri2.11.1/macros2.6.3/CLI2.11.4出口，未新增API或版本行为，无新的Context7问题。

### 8. 禁止依赖

命令不新增SQL/PG连接、文件I/O、模型计算、状态/服务实例、worker/锁/线程/时钟/事务/重试；无新DTO/依赖/迁移/runner/workflow/测试target/数据库专项入口，不在传输层重建原业务规则。

### 9. 状态所有权

AppState、ApplicationService/Arc、会话/工作器/问题日志/运行日志与State契约全部冻结，状态仍19；命令只借用同一State，无新共享状态、重复服务或日志实现。

### 10. 副作用边界

原Application和Ports/PG/文件I/O承担业务副作用；list_background_jobs仍先成功读取原limit，然后record_failed_background_jobs(&state.issue_log, &jobs)，最后Ok(jobs)，读取失败不投影。原平台Failed筛选、error_message、occurrence_key及去重行为/测试保持；不得在目录拆分时丢失问题日志。

### 11. 异常路径

原“复盘 ID”“当前比赛 ID”“比赛 ID”“资料包 ID”“任务 ID”“候选 ID”“来源分析包 ID”中文标签/空格、单项与Option UUID转换及前置顺序保持。list_ability_candidates仍Option.map(parse_uuid).transpose()?；资料包preview仍as_deref.map(parse_uuid).transpose()?；同步模板source_package_id仍原Option解析。status/limit/路径/Draft/Request原样传递，不trim路径、吞非法ID、夹紧limit、替换模式或增加数据库门槛；error.to_string()保持。

### 12. 并发/异步/生命周期

41完整正文和公开async签名等价；同步2/异步39及原await顺序保持，不改变队列启动、领取/结算、取消/重试和监听器生命周期。请求/过期结果归原UI调用方；资料包锁/确认/幂等、不可变结算、人工裁决、参数影子验证/晋升/回滚仍原底层owner。静态等价不替代真实PG事务/并发证明。

### 13. 兼容要求

171公共名称/参数/返回/全局注册顺序、前端、contracts/schemas、配置/错误/日志、模型、依赖/锁/workflow、版本0.23.0保持；公共集合SHA256 dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669。

13物理组保持，review6/postmatch14/analytics21各组命令顺序/前端字段保持，其他10组精确不变；只同步3个根目录和逐命令定义、module-boundaries与Domain使用位置，定义映射SHA e4271c64e385f45e10f765d20af92a8a0d572dd698741a40cb9b7473fe403e05。内部清单确有位置变化，不声称字节冻结。

### 14. 实施步骤

fresh核06精确完整Windows、全部步骤/日志/artifact并五文档[skip ci]收尾。原Review/Analytics守卫先增加真实传输/纯出口/签名/原UUID参数/调用形状约束，缺新目录时确认失败；迁移41原完整函数，接通3目录、删除3旧owner，更新7既有守卫受影响读者。静态、等价、隔离探针和保护检查通过后沿原Windows流程。

### 15. 切换入口

commands.rs与bootstrap/command_registry.rs字节不变，原mod review/postmatch/analytics直接解析新目录，不加旧转发。10mod只显式导出函数及__cmd__/__tauri_command_name_双宏；后台列表从子目录导入原commands根的共享helper，根alias仍有真实调用，不新增日志owner或unused导入。

### 16. 删除清单

删除commands/review.rs、postmatch.rs、analytics.rs。原41完整函数按职责迁移，整文件逐字移动/重命名：无；无重复命令、旧转发、复制状态/DTO或被替代业务实现残留。

### 17. 最小验证

最终82/82原只读源码、完整npm run verify:architecture、51新Rust独立Rustfmt1.88 --check、154等价、隔离26破坏拒绝/8守卫恢复正向、主目录222源码/清单/守卫SHA256不变、18保护资产/171命令13组/46迁移18PG静态及git diff --check通过。

154含41正文/41完整公开签名/41原AppState、冻结业务/Ports/模型/UI/依赖/状态/其他命令/01～06、公共契约/10原组、3组wire与7守卫逆变换。新增/迁移Rust单测0，原targets及测试正文保持；预期Desktop46/App230/PG259/Gateway112unit+41contract须07自身精确Windows确认，静态核对和探针不计动态测试。

### 18. 阶段回归

沿原Public Platform CI完整frontend/contracts/TypeScript/Vite/17视口、fmt/locked Clippy workspace all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动与日志。82源码排除原Windows进程与五浏览器入口；未执行非Windows Cargo/check/Clippy/build/test/前端构建/浏览器/loopback。

### 19. 失败停止条件

07自身完整Windows成功前VERIFYING，08～10 BLOCKED；保护/公共契约/最小验证失败继续修复同一R11分支。Windows job首次实际in_progress后停止全部CI查询，提示约20～30分钟并等用户返回；不继承06成功。

### 20. 回退点

859f12da03c04b4136be153d8b35f80015fce0a7，生产源码同06完整Windows成功faa6f72。按节点原子提交受控revert命令/清单/守卫/记录，不硬reset/手工复制旧owner/改历史数据；回退源码亦须自身Windows。

### 21. 根 README 摘要记录

同步41条职责/三个旧owner退出、问题投影保持、171兼容/13组、静态结果与07自身待验，链接本记录；01～06 DONE、07 VERIFYING、08～10 BLOCKED、R11 IN_PROGRESS，无阶段完成记录。

### 22. docs 阶段节点详细记录

本记录、索引、任务书、README及TESTING同步，01～06节点记录实施中字节冻结。Review/Analytics守卫原全部业务断言逆变换保持，仅补新传输块及Review实际定义读者；参数生命周期/资料包/赛后结算/Stage A只同步正式定义map读者，Tauri bootstrap仅同步真实后台投影import位置，不关闭检查。正式命令验证器字节冻结。

26探针覆盖三旧owner恢复、注册/根宏/子宏/纯出口、3类Option ID吞错、标签/资料包UUID、路径/Draft/status、两个同步入口、后台投影丢失/错误owner/提前执行/limit、原错误、晋升/回滚调用、监控签名与边界owner。全部在repo外独立源码副本运行，8原守卫恢复正向，主222文件SHA256保持，未修改生产目录做破坏探针。无本节点生产验证失败或未恢复探针。

### 23. 完成标准

41唯一传输入口、旧owner清理、记录与静态验证完成；07 VERIFYING，08～10未开始，R11未完成。真实PG/历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。

## 实际能力归属

| 目录 | 命令数 | 能力 |
|---|---:|---|
| review根 / abilities | 4 / 2 | 复盘生成/列表/读取与能力候选 |
| postmatch根 / evidence / monitoring / package | 3 / 2 / 2 / 7 | 结算、证据判定、监控与完整资料包流程 |
| analytics根 / jobs / ai_package / parameter_lifecycle | 2 / 4 / 6 / 9 | 概览/质量、后台任务、AI包与参数生命周期 |

## 实际验证与未执行范围

原82read-only node入口、npm run verify:architecture、node scripts/verify_protected_assets.mjs、node scripts/verify_command_contract.mjs、node scripts/verify_database_baseline.mjs、51文件独立Rustfmt1.88 --edition 2021 --config skip_children=true --check、scratch/r1107_compare.py、隔离副本r1107_probes.py及git diff --check实际通过。报告r1107-static.json、r1107-architecture.log、r1107-equivalence.json、r1107-negative.json和r1107-primary-fingerprints.json在repo外scratch，可从源码复建，不新增持续runner。

Domain365/129声明源/301映射，scan1416→1464；sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，usageDigest2f57057c21c018f83ae397f8311147d54326487442e8600636c3fe693fa77fe6。App422/43Ports19files/PG直接38/state19不变；保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57与DB静态aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

未运行真实PG资料包确认/事实提交/结算/能力与证据裁决/参数晋升回滚、有效XLSX或Windows Full；原R10底层代码/测试targets保持，实际效果仍最终新库待验。Mermaid Chart按实际三个命令目录→原业务/Ports/PG/文件与平台问题投影更新中文图；Create State沿足球稳定UUID96f03270-c236-46de-99fa-db85d2fbf4ce续接；无新API版本问题或子代理。

## 全部实际文件清单

基于859f12da03c04b4136be153d8b35f80015fce0a7，no-renames口径52A/14M/3D，共69路径；51新Rust（41命令、10纯出口），既有Rust修改无；整文件逐字移动/重命名无，41完整函数按实际职责迁移。

### 新增（52）

- `docs/modular-rewrite/R11-tauri-commands/R11-07-review-postmatch-and-analytics-commands.md`
- `src-tauri/src/commands/analytics/ai_package/decide_ai_analysis_suggestion.rs`
- `src-tauri/src/commands/analytics/ai_package/export_ai_analysis_package.rs`
- `src-tauri/src/commands/analytics/ai_package/export_ai_analysis_response_template.rs`
- `src-tauri/src/commands/analytics/ai_package/import_ai_analysis_response.rs`
- `src-tauri/src/commands/analytics/ai_package/list_ai_analysis_suggestions.rs`
- `src-tauri/src/commands/analytics/ai_package/mod.rs`
- `src-tauri/src/commands/analytics/ai_package/preview_ai_analysis_response.rs`
- `src-tauri/src/commands/analytics/analytics_overview.rs`
- `src-tauri/src/commands/analytics/decide_data_quality_finding.rs`
- `src-tauri/src/commands/analytics/jobs/cancel_background_job.rs`
- `src-tauri/src/commands/analytics/jobs/enqueue_analysis_job.rs`
- `src-tauri/src/commands/analytics/jobs/list_background_jobs.rs`
- `src-tauri/src/commands/analytics/jobs/mod.rs`
- `src-tauri/src/commands/analytics/jobs/retry_background_job.rs`
- `src-tauri/src/commands/analytics/mod.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/decide_parameter_tuning_candidate.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/generate_parameter_tuning_candidate.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/list_parameter_promotion_decisions.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/list_parameter_shadow_validations.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/list_parameter_tuning_candidates.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/mod.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/parameter_lifecycle_readiness.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/promote_parameter_candidate.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/rollback_parameter_candidate.rs`
- `src-tauri/src/commands/analytics/parameter_lifecycle/run_parameter_shadow_validation.rs`
- `src-tauri/src/commands/postmatch/evidence/decide_evidence_scoring_item.rs`
- `src-tauri/src/commands/postmatch/evidence/list_evidence_scoring_items.rs`
- `src-tauri/src/commands/postmatch/evidence/mod.rs`
- `src-tauri/src/commands/postmatch/list_postmatch_settlements.rs`
- `src-tauri/src/commands/postmatch/mod.rs`
- `src-tauri/src/commands/postmatch/monitoring/mod.rs`
- `src-tauri/src/commands/postmatch/monitoring/postmatch_overview.rs`
- `src-tauri/src/commands/postmatch/monitoring/refresh_postmatch_monitoring.rs`
- `src-tauri/src/commands/postmatch/package/commit_match_review_package.rs`
- `src-tauri/src/commands/postmatch/package/commit_match_review_package_facts.rs`
- `src-tauri/src/commands/postmatch/package/confirm_match_review_package.rs`
- `src-tauri/src/commands/postmatch/package/export_match_review_package.rs`
- `src-tauri/src/commands/postmatch/package/generate_match_review_from_package.rs`
- `src-tauri/src/commands/postmatch/package/mod.rs`
- `src-tauri/src/commands/postmatch/package/preview_match_review_package.rs`
- `src-tauri/src/commands/postmatch/package/read_match_review_package_workflow.rs`
- `src-tauri/src/commands/postmatch/postmatch_settlement_readiness.rs`
- `src-tauri/src/commands/postmatch/settle_postmatch_review.rs`
- `src-tauri/src/commands/review/abilities/decide_ability_candidate.rs`
- `src-tauri/src/commands/review/abilities/list_ability_candidates.rs`
- `src-tauri/src/commands/review/abilities/mod.rs`
- `src-tauri/src/commands/review/generate_match_review.rs`
- `src-tauri/src/commands/review/list_match_reviews.rs`
- `src-tauri/src/commands/review/list_reviewable_matches.rs`
- `src-tauri/src/commands/review/mod.rs`
- `src-tauri/src/commands/review/read_match_review.rs`

### 修改（14）

- `README.md`
- `architecture/command-contract.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/README.md`
- `scripts/verify-analytics-service.mjs`
- `scripts/verify-match-review-package.mjs`
- `scripts/verify-parameter-lifecycle.mjs`
- `scripts/verify-postmatch-settlement.mjs`
- `scripts/verify-review-service.mjs`
- `scripts/verify-stage-a-architecture.mjs`
- `scripts/verify-tauri-bootstrap.mjs`

### 删除（3）

- `src-tauri/src/commands/analytics.rs`
- `src-tauri/src/commands/postmatch.rs`
- `src-tauri/src/commands/review.rs`


## R11-07 精确 Windows 完成与08授权（2026-10-23）

精确提交 `5b779b9dfb8aa113c29a64f6714f6514c5a62db8` / tree `cf123387b38ba1eb1be64e995e22668cb684443b`，[Public Platform CI run39962382506](https://github.com/uniquenesssta/123/actions/runs/39962382506) / Windows job120443361023完整SUCCESS，run于2026-10-23 22:02:43北京时间完成。实际Desktop46/46、Application230/230、Persistence259/259、Gateway112/112 unit和41/41 contract；完整frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动与日志均通过（7条记录、3个完成操作）。新增/迁移单测0，154等价与26隔离探针不计动态测试。

报告logs/windows-acceptance-20261023-133701.json及同名.txt；artifact12507405698，14,053,924字节，SHA-256 `76a48e55a30cf7f237f864c1abd55e4986837d67a727a9b770a3b379148daffc`，查询时未过期。已核对精确head/branch、全部job步骤、完整日志与artifact。用户2026-10-23“收尾07开始08”，07 DONE，08 AI Workspace/OpenAI Profiles Commands READY，09～10 BLOCKED，R11 IN_PROGRESS。此次仅5份既有文档[skip ci]收尾，源码沿上述自身完整Windows证据，不重复同源码构建；历史VERIFYING/预期计数保留，08须自身完整Windows。

真实PG/历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。
