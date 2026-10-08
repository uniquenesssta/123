# R08-07 Run Persistence 完成记录

## 状态与基线

2026-10-08，用户“开始07”后实施。当前状态 **DONE**：精确 `58b390a` / Windows run `37796909083` 全 SUCCESS；本文实施时的预期/待验描述保留为历史，当前验收见末节。08 已获用户启动授权，09～12 BLOCKED。唯一阶段分支 `rewrite/r8-prediction-p4-orchestration`。起点/受控回退点 `eadb49d51722f50349509a0402c5a925402a53dc`，这是06文档收尾，其生产源码为已通过 `0969331` / Windows run `37752995641` 的树。当前状态见 [索引](README.md)，范围依据 [任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md)。

## 实际来源与职责变化

实际旧 owner 为 `crates/persistence-postgres/src/model_runs.rs`（622行），不是 R3 已删除的 Application prediction.rs。保存、输入审计/快照前检、模块/比分明细、历史读取与隐藏事务原先混在该文件，现在全部迁出并删除旧文件，不保留转发壳。

| 文件 | 唯一职责 |
|---|---|
| runs/mod.rs | 私有模块登记和原 ModelRunListItem 导出 |
| runs/write.rs | PostgresStore::save_successful_run 原保存事务与完成审计 |
| runs/input.rs | 原审计/manifest hash 和 feature snapshot 的纯前检 |
| runs/details.rs | 借用同一事务写模块与比分，原数值检查 |
| runs/read.rs | 原 DTO、list_recent_runs、read_run 投影与共享身份读取 |
| runs/visibility.rs | hide_run_from_history 原更新与审计事务 |

上游仍是 `use_cases/prediction/execute_prediction/mod.rs` 经原 PredictionExecutionPort 和 composition adapter 调用 PostgresStore。正式保存/影子 Uuid::nil、失败传播和结果组装保持同一个执行用例。该分支只有十余行连贯编排，另建空 Application wrapper 无实际职责收益，故不照模板创建转发目录。R5-06 共享身份、mapping/audit owner 原样复用；R8-08及其后的账本/事实/工作台/冻结不提前迁移。

保存顺序保持：运行 UUID → 原完整 input hash → 原审计 → summary → 可选数据库比赛 ID → snapshot 前检 → 拒绝无比赛的 snapshot → 开启一次事务 → 插入/复用 runtime snapshot → succeeded run → 模块/比分 → completion audit → commit → 返回 UUID。快照冲突仍 ON CONFLICT，先精确 ID，再 match_key/snapshot_type/input hash 和 legacy/runtime source 复用。details 只借用该事务，不开池、不自行提交；SQL/明细/审计失败沿原 SQLx 事务 drop 回滚，不自动重试。

历史保持 limit 1～500、created_at DESC/ID DESC、succeeded 和未隐藏过滤、比分排序、名称 fallback 与原共享 identity 字段。隐藏 reason trim/空白默认、首次 hidden_at COALESCE、每次成功隐藏一条审计、同一事务保持；不删除 run，不改输入/输出身份，隐藏后仍可 read_run。

## 全部文件清单

新增（7）：

- `crates/persistence-postgres/src/adapters/prediction/runs/mod.rs`
- `crates/persistence-postgres/src/adapters/prediction/runs/write.rs`
- `crates/persistence-postgres/src/adapters/prediction/runs/input.rs`
- `crates/persistence-postgres/src/adapters/prediction/runs/details.rs`
- `crates/persistence-postgres/src/adapters/prediction/runs/read.rs`
- `crates/persistence-postgres/src/adapters/prediction/runs/visibility.rs`
- `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-07-run-persistence.md`

修改（15）：

- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `crates/application/src/use_cases/prediction/tests.rs`
- `crates/persistence-postgres/src/adapters/mod.rs`
- `crates/persistence-postgres/src/adapters/prediction/mod.rs`
- `crates/persistence-postgres/src/lib.rs`
- `crates/persistence-postgres/tests/model_run_identity_repository_contract.rs`
- `scripts/verify-history-scoreline-ui.mjs`
- `scripts/verify-model-run-identity.mjs`
- `scripts/verify-persistence-mapping.mjs`
- `scripts/verify-prediction-service.mjs`
- `README.md`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md`
- `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md`

移动/重命名：无。内容按职责提取，不是整文件搬迁；按 `git diff --no-renames --name-status` 核对。

删除（1）：`crates/persistence-postgres/src/model_runs.rs`。

## 兼容与清单

九项原生产函数体（save/list/hide/read、save_model_details、required_i64/f64、prepared_run_input_audit、prepared_feature_snapshot）、原一项测试、全部 SQL literals 和四个公开方法签名，去空白/格式逗号比较一致；私有 helper 可见性只扩至同目录兄弟模块。原公开 PostgresStore 方法、ModelRunListItem 根导出/字段/Serde、Domain/Model API、Application Port/Service、Tauri DTO/171命令、Schema/数据格式/配置、错误类型/中文提示/优先级、日志、UI 和模型算法/参数/保护资产没有变化。输入审计负数/字符串 score 按原 as_u64 语义视为缺失，null manifest 与原错误优先级均保留，不借重构修行为。

Postgres 直接模块清单移除已删除 model_runs，计数36→35；runs 位于既有 prediction 下。原 generator 刷新 Domain 使用方，扫描1046→1051，365类型/300映射及 sourceDigest `217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd` 保持，usageDigest `57653c49ff4d861f79dbb439e83e066899455c6aeb98f94cc5f4474253c484d2`。Application 文件390/43 Ports及其契约不变。18保护资产、Cargo/npm manifests/locks、0001～0046 migration、数据库基线和生产依赖保持。没有新 runner/workflow/test target/数据库设施。

## 测试与实际验证

原 Persistence 审计测试保留并迁移；新增七项进入原 target：

- legacy_audit_preserves_fallback_shape_hash_and_missing_values
- audited_input_preserves_trimmed_identity_levels_and_optional_score
- audit_field_errors_keep_priority_and_null_manifest_semantics
- feature_snapshot_keeps_original_identity_window_and_schema
- feature_snapshot_rejects_missing_metadata_and_changed_identity
- feature_snapshot_quality_keeps_closed_unit_interval_and_type_checks
- model_detail_scalars_preserve_integer_numeric_and_missing_errors

原 Application failed_run_save_is_propagated_without_success_or_automatic_retry 扩六类 PortErrorKind，保留原错误 kind/message，精确 scope→route→supports→predict→save 顺序与一次保存，不增加测试数量。预期 Persistence **148**（141+7）/Application **92**，尚未执行 Rust tests；不能把静态解析或预期数量写成 PASS。

原 ignored PG model_run_identity_repository_contract target 保留 full/nullable 夹具并扩公开 save/read/history/hide：第二比分 rank32768或概率2.0失败后五类写入共同为零；两次成功 run UUID 不同且只一个 snapshot；完整载荷/hash/审计/路由/耗时与明细/默认比分分类读回；每个成功一条 completion audit；history topscore/name/limit；重复隐藏首次时间不变、原因 trim、两条审计、列表排除而原记录可读；不存在 run 拒绝和输入不可变 trigger。现有18 broad数量不变，这些新增 PG 断言未实跑，只在本项 Windows编译并最终封包新库执行。

| 实际本地检查 | 结果 |
|---|---|
| 原 verify-frontend 列表逐项执行83个源码检查 | 83/83 PASS；5浏览器项留Windows |
| npm run verify:architecture | 完整现有门禁 PASS |
| 同版本 Rustfmt 1.88.0，changed Rust files --edition2021 --config skip_children=true --check | PASS |
| node scripts/verify_protected_assets.mjs | 18项 PASS，摘要 d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57 |
| node scripts/verify_command_contract.mjs | 171命令 PASS |
| node scripts/verify_database_baseline.mjs | 46连续迁移/18原PG静态契约 PASS，迁移摘要 d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93 |
| git diff --check | PASS |
| 九函数/原测试/SQL/四签名比较 | 等价 PASS，不等同真实PG结果 |

报告 `/workspace/scratch/0c69084e7a7f/r807-source-checks.json`、`/workspace/scratch/0c69084e7a7f/r807-negative-probes.json`，仓库原入口可重现。六项临时破坏探针：提前 commit、放宽snapshot来源、绕过manifest hash、放宽quality上界、漏hidden过滤、影子正式保存；均被增强的原 verify-prediction-service 拒绝，逐一恢复后健康门禁 PASS，不提交临时探针。其余三项原 verifier 只更新实际 owner 路径，原断言保留。

本地 Rustfmt 共享运行库缺损，已有同版本归档提取曾被截断；以完整内容恢复并核对文件大小后原工具运行成功。未改变项目依赖。Rustfmt 换行使新增回滚断言的静态字面串不匹配，改用允许空白的正则并保留实际断言内容，随后原门禁通过。原使用清单随格式化/新增来源刷新，不放宽契约。

## 待验、风险与回退

仅 Windows 动态验证：本项原 Windows Automated 负责完整frontend/类型/production build、17视口、fmt/Clippy/workspace tests、release/MSI/NSIS/启动，启动后停止轮询并等待用户反馈。没有执行 Linux/macOS Cargo 或客户端动态验证；本项必须取得精确 SHA 的自身 CI 才能 DONE，并据此开放08。

真实 PG/历史四项/账本/有效 XLSX/Windows Full/私有 P4/P7固定概率与 Golden Master 统一最终封包新库验收，ignored 编译不能计为实跑 PASS。公开 unavailable stub 不代表私有引擎回归。继承 delete_match 将 model.runs.match_id 置null与0041不可变trigger冲突的历史风险未在本项关闭，仍需最终夹具验证。原 SQL 事务隔离与快照冲突政策不改，不承诺新增并发保证。

保存/隐藏局部事务随原调用生命周期释放；无缓存、后台任务、监听器、定时器或额外 State。Mermaid Chart 已更新实际数据库比赛/手工请求、正式/影子、事务失败、历史读取/隐藏链路；交接时 Create State 保存节点/约束。

失败时受控 revert 本节点完整提交，恢复基线 `eadb49d51722f50349509a0402c5a925402a53dc` 的唯一 owner，同步上述23文件、测试/门禁/清单与文档状态；不手工复制旧文件、不保留双实现、不修改历史数据库。当前实施记录不冒充已完成验收记录或阶段完成记录。


## R8-07 精确 Windows 验收与收尾（2026-10-08）

精确实施 `58b390a6400646ac6131dfef0503a49ce1128eca` / [Windows run `37796909083`](https://github.com/uniquenesssta/123/actions/runs/37796909083) / job `113378629485` 全 SUCCESS，完成于 2026-10-08 23:27:02（北京时间）。完整日志确认 Application **92** / Persistence **148**、七项新增边界及原审计测试、前端契约/类型/生产构建、**17** 视口、fmt/Clippy/workspace tests、Windows release/MSI/NSIS 和启动 **7 条记录 / 3 个完成操作**通过。报告 `logs/windows-acceptance-20261008-145959.json`；artifact `11559974847`，14,025,581 字节，SHA-256 `b31ac49010ca525cd6c836fa9746be451ed5dc72bc8e918522e6ecad7e22f2ac`。

已核对精确 SHA、全部 job/steps、日志与 artifact；实施时预期/待验文字保留为历史记录。07 正式 **DONE**，08 前置通过；用户已授权“收尾07开始08”，沿用唯一 R8 分支。收尾仅同步五份文档，源码/测试/清单保持上述已验树，链接/状态/diff 核对，不重复全量构建，文档提交使用 `[skip ci]`。真实 PG/历史四项/账本/有效 XLSX/Windows Full/私有 Golden Master 与继承 model.runs/0041 删除风险继续最终封包新库待验，ignored 不计实跑 PASS。
