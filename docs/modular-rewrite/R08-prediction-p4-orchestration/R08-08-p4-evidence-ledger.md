# R08-08 P4 Evidence Ledger 实施记录

## 状态与基线

2026-10-08，用户“收尾07开始08”。07精确 `58b390a6400646ac6131dfef0503a49ce1128eca` / Windows run `37796909083` / job `113378629485` 已全SUCCESS，Application92/Persistence148、17视口、Windows构建/打包/启动通过；收尾文档 `1db0e195cdbc26514d7ad015b4110a53a446ba26` 使用 `[skip ci]`，生产源码保持。08从此开始，沿用唯一分支 `rewrite/r8-prediction-p4-orchestration`。

08 **VERIFYING**：已实施，首轮 `91e9585` / Windows run `37804265878` 因测试专用顶层导入被Clippy拦截；已修复并通过本地受影响静态门禁，修复自身精确Windows CI待验；09～12 BLOCKED。没有将07的动态PASS继承为08。状态见 [索引](README.md)，范围依据 [任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md)。

## 来源与最终职责

实际来源为Postgres `p4_records.rs`，原1944行文件同时包含版本登记、研究任务、证据声明/冲突和冻结快照。08只迁出证据账本及必要共享依赖，不用R3已删除的Application prediction/p4文件作来源。

| 最终文件 | 职责 |
|---|---|
| adapters/p4/mod.rs | 登记两个职责模块 |
| p4/idempotency.rs | 原P4共用事务锁、240字节键验证、精确指纹判定 |
| evidence_ledger/mod.rs | 登记私有模块/原target测试，显式导出共享验证状态投影 |
| evidence_ledger/claims.rs | 原append_evidence_claim原子追加与审计 |
| evidence_ledger/conflicts.rs | 原create_evidence_conflict原子建组、成员/opened事件/审计 |
| evidence_ledger/input.rs | 声明前检/指纹与冲突集合纯前检/指纹 |
| evidence_ledger/references.rs | 借用同一事务核对研究关联的Schema/Prompt/冲突引用 |
| evidence_ledger/row.rs | 原声明Row映射及六种验证状态投影 |
| evidence_ledger/tests.rs | 原一项及新增八项边界测试，复用原Persistence target |

上游ResearchService/原ledger用例与FactPipeline仍经ResearchEvidenceLedgerPort/composition调用原PostgresStore方法。没有新增Application只转发层，没有改Port或Service接口。原共享幂等3函数在版本/研究/快照及账本中均有真实调用，迁入独立同职责owner，不重复复制。共享六种验证状态由ledger/row唯一持有，原snapshot feature映射直接复用。

原p4_records仍承担实际版本/研究/快照职责，保留原34个函数而非空壳；账本两写方法、五辅助、共用幂等3函数和原证据测试已迁出，无旧账本出口/双实现。R8-09～12 Fact/Horizon/Workbench/Freeze未提前实施。

## 事务、身份与兼容

声明保持原顺序：纯必填/来源/时间前检 → value hash/完整claim fingerprint → 一次begin → evidence:key事务锁 → 已有key精确指纹并返回原记录，或核对research match/schema及版本/冲突引用 → INSERT声明 → 完成审计 → commit。冲突保持：键前检/BTreeSet去重至少2成员/原指纹 → 一次begin → conflict:key事务锁 → 已有精确指纹返回，或检查所有成员存在且比赛/实体类型/实体ID/字段一致 → 冲突头 → 成员 → 单opened事件 → 审计 → commit。重试和新建各保留原独立commit出口，重试不重写载荷、不重复审计。references只借用transaction，不新增pool/commit。

冲突纯前检从原方法提取到input，重新内联后原函数完全等价。10个原生产函数、原一项测试、2公开签名、31 SQL raw literals与旧剩余34函数去空白/格式逗号比较保持，报告 `r808-equivalence.json`。模块内部可见性仅满足现有兄弟调用，不增加公共API。PostgresStore/Domain DTO/Serde/Schema/数据格式/配置、43 Ports/171命令、错误中文提示/种类/优先级、日志、UI、模型算法/参数/资产保持。

原来源仅检查None/空字符串，不新增trim或来源规范化；检索时间保留纳秒精确先后关系。原claim指纹保留身份/出处/时间/版本/metadata，idempotency key不入内容指纹；原conflict指纹保留排序去重成员与trace，key/metadata按原政策排除，重试仍保留首次原记录。数据库时间存储仍原SQLx微秒精度，载荷身份不因同微秒而放宽。没有UPDATE/DELETE账本业务入口、自动重试、缓存、后台任务或UI State。

## 全部变更文件

新增（10）：

- `crates/persistence-postgres/src/adapters/p4/mod.rs`
- `crates/persistence-postgres/src/adapters/p4/idempotency.rs`
- `crates/persistence-postgres/src/adapters/p4/evidence_ledger/mod.rs`
- `crates/persistence-postgres/src/adapters/p4/evidence_ledger/claims.rs`
- `crates/persistence-postgres/src/adapters/p4/evidence_ledger/conflicts.rs`
- `crates/persistence-postgres/src/adapters/p4/evidence_ledger/input.rs`
- `crates/persistence-postgres/src/adapters/p4/evidence_ledger/references.rs`
- `crates/persistence-postgres/src/adapters/p4/evidence_ledger/row.rs`
- `crates/persistence-postgres/src/adapters/p4/evidence_ledger/tests.rs`
- `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-08-p4-evidence-ledger.md`

修改（11）：

- `architecture/database-baseline.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `crates/persistence-postgres/src/adapters/mod.rs`
- `crates/persistence-postgres/src/p4_records.rs`
- `crates/persistence-postgres/tests/postgres_integration.rs`
- `scripts/verify-prediction-service.mjs`
- `README.md`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md`
- `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md`

移动/重命名：无；按职责提取，无整文件搬迁。删除文件：无；原p4_records仍有非账本职责。清单以08基线的 `git diff --no-renames --name-status` 核对，不混入07已提交的五文档收尾。

## 测试与本地证据

原evidence_source_is_required_for_supported_facts保留并迁入tests；新增八项进入原Persistence target：

- source_states_and_complete_provenance_keep_original_policy
- claim_mandatory_fields_keep_error_priority_and_raw_values
- claim_retrieval_window_keeps_nanosecond_boundary
- claim_fingerprint_preserves_semantic_identity_and_retry_key_policy
- conflict_preparation_deduplicates_sorts_and_keeps_identity_policy
- conflict_preflight_keeps_byte_key_limit_and_distinct_member_minimum
- verification_row_parser_keeps_six_states_and_unknown_errors
- idempotent_retry_keeps_exact_fingerprint_and_original_error

覆盖六种状态与来源完整性、原空白来源政策、五类必填与错误先后、retrieval相等及前后1纳秒、全部身份/出处/版本/时间/metadata敏感性与value hash、raw字段不修改、成员排序去重/trace/原metadata排除、240字节中英文键边界、至少两不同成员、状态未知错误和精确指纹重试提示。预期Persistence **156**（148+8）/Application **92**；尚未Rust实跑，不把静态检查或预期数量当通过。

现有ignored PG Stage C测试保留原schema/research/claim/conflict/snapshot/cutoff与不可变夹具，扩公开账本链：完整声明非时间字段读回和四时间微秒精度；metadata/同微秒纳秒漂移仍拒绝；冲突排序/重复/metadata原策略；run/match/schema/prompt/conflict引用和source_document FK失败无claim/audit；首次同key并发仅一个UUID/一次审计；缺失/身份不符/重复最小集合冲突无头/成员/事件/审计；并发冲突2成员/1opened/1审计；四类账本mutation拒绝。不新增临时数据库trigger或专项入口；其事务原子顺序另由原增强源码门禁核对。18 broad数量不变，PG尚未执行，ignored编译不算实跑PASS。

| 实际本地命令/检查 | 结果 |
|---|---|
| 原verify-frontend列表逐项执行源码检查 | 83/83 PASS；5浏览器项留Windows |
| npm run verify:architecture | 原完整门禁PASS |
| Rustfmt1.88.0，12 changed Rust targets --edition2021 --config skip_children=true --check | PASS |
| node scripts/verify_protected_assets.mjs | 18项PASS；摘要d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57 |
| node scripts/verify_command_contract.mjs | 171命令PASS |
| node scripts/verify_database_baseline.mjs | 46连续迁移/18原PG静态契约PASS；摘要d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93 |
| 原函数/SQL/签名及测试比较、git diff --check | PASS |

工作目录报告 `/workspace/scratch/0c69084e7a7f/r808-source-checks.json`、`r808-architecture.log`、`r808-negative-probes.json`、`r808-equivalence.json`，原仓库命令可重现。六项临时破坏探针（漏事务锁、漏metadata指纹、放宽字节键上限、漏冲突实体ID、跳过Schema版本、提前commit）全部被原Prediction gate拒绝，逐一恢复并健康复跑PASS，不提交临时探针。首次architecture仅P4新增直接模块计数未同步，精确35→36修正后完整通过，owner集合未放宽。

清单仅登记新P4直接module与原使用方；Domain扫描1051→1060，365类型/300映射和sourceDigest `217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd`保持，usageDigest `307b38c0ae795e65133ae11522339b21a47bd8ab5cc1129f02f679ca0ba97608`。Application390/43Ports及契约不变；PG runtime_sources仅刷新原postgres_integration blob `eb0f4dd987b3f4374a57c1f5295c8be0ffaf9966`。保护区、dependencies/locks、0001～0046、数据库runtime生产设施均不改。

## 首轮 Windows CI 失败与修复（2026-10-09）

首轮源码 `91e958581f8b675b58e56524c33834511af4985e` / [run `37804265878`](https://github.com/uniquenesssta/123/actions/runs/37804265878) / job `113404293519` 为 FAILURE。完整架构、前端契约/类型/生产构建、17视口和Cargo锁同步已通过；Rust验收在Clippy `-D warnings` 因 `p4_records.rs:8` 的未使用 `EvidenceVerificationState` 导入退出101，Rust tests及Windows release/MSI/NSIS/启动未完成，不记通过。

该类型拆分后仅由原快照测试夹具使用，生产状态投影已在 `evidence_ledger/row.rs`。将其从生产顶层use移入原 `#[cfg(test)] mod tests`，保留测试使用和严格警告门槛；同次拆分的账本/幂等模块导入已核对。唯一Domain清单仅更新 `rustUsageDigest` 为 `38d0ed328bcb3015ded4bcc7a85ab2b6206db82152d5eec2b3c8e1953218baaa`，365类型/300映射、声明摘要与使用方集合保持。

修复仅修改 `p4_records.rs`、`architecture/domain-type-inventory.json`、根README、本实施记录及阶段索引；无新增/移动/删除文件。生产函数、SQL、测试体、公开契约、依赖、迁移和模型资产不变，无警告抑制或新增专项测试。

修复本地 `npm run verify:architecture`、Rustfmt 1.88.0 单文件格式检查、`node scripts/verify-rust-source-hygiene.mjs`、`git diff --check` 已PASS；导入之外的生产/测试函数逐字比较保持。第一次本地架构检查因导入调整后使用摘要未同步而失败，按原生成器同步并审查唯一摘要差异后完整复跑通过。未执行非Windows Cargo/客户端动态验证；本轮修复的Clippy/workspace tests/Windows构建打包启动仍由原Windows CI验证，启动后停止轮询，08不提前DONE，09～12继续BLOCKED。修复可单独revert本次提交；完整节点回退仍按下方原08基线。

## 待验、交接与回退

仅Windows动态验证；08自身原Windows Automated负责完整frontend/类型/production build、17视口、fmt/Clippy/workspace tests、Windows release/MSI/NSIS/启动。启动后停止轮询、等待用户结果，取得精确SHA验收后才DONE并开放09。未执行Linux/macOS Cargo或客户端动态，无新增runner/workflow/target/数据库/框架。

真实PG/历史四项/账本/有效XLSX/Windows Full/私有P4/P7固定概率及Golden Master仍最终封包新库待验。公开unavailable stub不冒充私有模型回归；继承delete_match置model.runs.match_idNULL与0041inputimmutable trigger的风险未在本项关闭。原未收紧的来源字符串/冲突metadata指纹策略明确保留，不承诺新增数据规范化或隔离保证。

Mermaid Chart已更新真实声明/冲突事务、共用锁、身份引用/审计和共享状态投影；结束时Create State保存稳定项目下的节点/证据/约束。失败时受控revert08完整提交恢复 `1db0e195cdbc26514d7ad015b4110a53a446ba26` 已验源码文档树，同步上述21文件的唯一owner/调用/门禁/清单/文档，不手工复制旧实现或改变历史数据库。本实施记录不冒充完成记录或R8阶段收尾。
