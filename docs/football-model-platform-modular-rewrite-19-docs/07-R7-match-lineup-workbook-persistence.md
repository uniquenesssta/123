# R7：比赛、阵容、工作簿持久化与累计审计整改——独立执行任务书

> 文档编号：`R07`；前置阶段：R6；后续阶段：R8。与总纲共同适用。
> 2026-09-30 根据用户决定调整：在原 R7-02 之前插入 5 个整改节点；沿用现有验证方式，不建设新的持续回归体系。

## 1. 目标、范围与当前状态

保留 R1–R6 已完成的迁移成果，先修复当前 R7-01 调用链和累计审计确认的缺口，再继续双方阵容、历史、预设与工作簿持久化重写。不得把前六阶段全部推倒重做。

- 唯一阶段分支：`rewrite/r7-match-lineup-workbook-persistence`。
- 规划依据：源码 `985f01816060cfd05672bdc03b6771dec7b4e842`，文档基线 `51f746729b2f94925e6382f2ae2108cf80855af6`，以及阶段 README 中 A1–A8、B1–B7 审计记录。
- R7-01～R7-11 已完成精确提交的 Windows Automated 并为 `DONE`；11 run `36824027121` 的 Persistence 127 项/Application 53 项通过。用户已启动 R7-12，共享球队资料包持久化、调用/门禁及回归已实现，当前 `VERIFYING`；R7-13 及以后 `BLOCKED`。真实 PG/XLSX/Full 保留最终封包新库待验。
- 当前状态以 `docs/modular-rewrite/R07-match-lineup-workbook-persistence/README.md` 的唯一状态表为准；历史审计中的旧编号按下表映射。
- 允许整改触及直接相关的 Application composition/Ports、catalog identity/deletion、现有 P4 时间比较和测试；不提前实施 R8/R10 的整阶段重写。
- 不包含前端导入 UI 重写、工作簿解析器整体验证框架建设、模型内部算法/参数修改或历史 migration 改写。

## 2. 新编号与执行顺序

| 当前任务 | 内容 | 原编号 |
|---|---|---|
| R7-01 | Match Catalog 与当前阻塞修复 | R7-01 |
| R7-02 | 外部 ID 身份保护 | 新增，B1 |
| R7-03 | 普通删除与历史引用保护 | 新增，B2 |
| R7-04 | 架构清单、验证器与执行记录对齐 | 新增，B3/B7；B4 仅现有入口遗漏 |
| R7-05 | 关键 Application 用例验证 | 新增，B5 |
| R7-06 | 历史数据库失败与账本问题收口 | 新增，B6/A8 |
| R7-07 | Lineup Pair Transaction | 原 R7-02 |
| R7-08 | Lineup Chain / History | 原 R7-03 |
| R7-09 | Team Lineup Presets | 原 R7-04 |
| R7-10 | Spreadsheet Batch Ledger | 原 R7-05 |
| R7-11 | Player Workbook | 原 R7-06 |
| R7-12 | Team Package | 原 R7-07 |
| R7-13 | Monthly Workbook | 原 R7-08 |
| R7-14 | Match Lineup Workbook | 原 R7-09 |
| R7-15 | Row / Subrecord Identity | 原 R7-10 |

按 R7-01 → R7-02～06 → R7-07～15 执行。R7-01 关闭后只开放 R7-02；R7-02～06 的代码整改和节点最小门禁完成前，不开始原双方阵容重写。第 3 节允许登记至最终新库验收的动态验证项，单独保留待验状态，不冒充通过。

## 3. 验证方式：沿用现有流程

用户明确不新增“持续回归建设”任务、runner、测试框架、数据库调度器、CI workflow 或持续回归专用入口。不要求每个节点反复创建数据库，也不建立第二套验收体系。

- 客户端编译、Rust fmt/Clippy/tests、前端构建、Tauri 打包、运行与人工验收仅针对 Windows。Linux/macOS 不作为门禁或待补项。
- 沿用 `npm run verify:architecture`、`npm run verify:frontend`、`npm run verify:rust`、现有静态脚本、`scripts/windows-acceptance.ps1`、`scripts/run_database_baseline.mjs` 和现有测试文件。
- 节点内部先执行受影响的检查；可交付节点和阶段出口使用现有 Windows CI/验收。相同代码树已通过的检查可以引用证据，不因文档追加重复全量编译打包；测试或验证器本身变化时重跑受影响门禁。
- 可以修正现有验证器路径、集合核对与现有命令遗漏的调用，不借此新建持续回归入口。需要增加行为断言时优先补入现有单测模块、`postgres_integration.rs` 或现有 repository contract 文件，不新增专项 test target。
- 数据库验证复用已有专用测试库和现有执行方式；最终封包按用户流程建立新数据库，再执行现有数据库基线、相关已有 contract、真实 XLSX 与 Windows Full。已有 contract 可直接用现有 `cargo test --test <已有目标> -- --ignored` 方式执行，不包装成新框架。
- 暂无专用数据库时，先完成代码、Windows 编译/非数据库测试和静态门禁，将动态验证逐项登记为“最终封包新库待验”。这是明确延期，不是测试通过；不为此创建新数据库设施，也不要求迁移或保留用户旧数据库的数据。
- 已有失败测试必须修复对应代码/夹具，禁止删除、跳过或放宽断言来修绿。未重跑的修复记录为“已修改、待新库实跑确认”，不能宣称历史失败已验证关闭。
- 不对用户业务数据库执行清空。最终新库验证不取消正常运行时的身份稳定、事务、历史引用保护；新库也会产生这些业务数据。
- R7 可按代码/静态/非数据库门禁记录节点完成及明确延期项，但阶段收口必须列出所有动态待验项。最终封包必须清零待验项和已知失败；不能用阶段 DONE 代替最终验收完成。

## 4. 适度约束与不变量

1. 按实际职责、状态所有权和依赖决定拆分。行数/大小触发审查，不自动升级目录；紧密耦合的私有校验、Row/Mapper、事务步骤可共置，不机械地一函数一文件。
2. 允许修改问题真实影响链中的调用方、adapter、清单、验证器和文档。不得夹带无关功能、重命名、格式化或依赖升级。
3. Application 核心用例依赖 Ports；现有 composition-only 的 PostgreSQL 导入及 facade/session 边界可以保留。DatabaseSession alias 不应被描述为运行时或类型级隔离；不临时拆出新 crate。
4. Repository 管理事务、锁、超时、错误传播与取消后的提交/回滚结果；UI 请求 ID、结果过期和监听器释放由实际调用方负责，不向无对应生命周期的 Repository 强塞 UI 模板。
5. 公开命令/DTO、正常业务行为、数据格式默认兼容。B1 的覆盖改绑、B2 的错误删除、B6 的精度导致重复写入等已确认不变量缺陷属于修复范围；不得用“保留旧行为”维持缺陷，也不得静默扩成正常行为变更。
6. 模型保护资产、模型算法/参数、0001–0046 migration、生产依赖保持冻结。最终使用新数据库不授权修改历史迁移指纹。
7. 保留身份不改绑、普通删除保护历史、双方阵容单事务、actual/赛前 cutoff 隔离、未解决冲突不得提交、整批回滚及真实行/子记录身份。
8. 门禁失败阻止宣布通过和推进下一节点；仍可继续本节点内已授权的诊断、修复与独立工作。用户未授权业务实施时仅维护任务书。
9. 节点记录保留目标/实际范围、变更与兼容性、验证/延期、剩余问题和回退提交。允许引用真实 diff 与已有证据；不强制重复 23 节模板或额外架构图。

## 5. 节点共同交付要求

每个节点均需确认进入基线，核对现存调用链，先确定可观察验收目标，再实现/调整现有测试，完成直接受影响调用方切换并清理被替代实现。修改完成后复核 diff；不要删除仍有调用者的入口或保留无退出计划的双实现。

节点状态只用 BLOCKED、READY、IN_PROGRESS、VERIFYING、DONE；动态待验作为独立记录字段。按真实结果更新阶段索引与根 README，未做的节点不提前创建伪完成记录。回退使用实施前记录的 Git 提交或受控 revert，不靠复制旧文件恢复。

# Atomic Tasks

## R7-01 Match Catalog 与当前阻塞修复

状态：`DONE`。A1–A7 修订与 Windows Automated 通过；PG/XLSX/Full 最终封包新库待验。

- **目标/范围**：完成 `adapters/matches/catalog/` 的唯一职责切换，连同 Application `composition/adapters/lineups.rs`、直接 persistence 调用方、相关现有验证器及 Domain inventory。
- **实施**：补齐失效的 `read_match_exchange` 调用，调用明确的现存公开读取方法并避免 trait 递归；用官方生成器更新并审查 Domain inventory；将 Lineups/Match Workflow/History Scoreline 验证器指向当前 owner，保留原断言；将已有 R7 verifier 接入现有合适入口。
- **数据边界**：先拒绝主客相同和非法 scope；任何会自动创建/更新 season 的 scope 解析必须与成功比赛写入保持一致原子边界，失败不得遗留写入。保持 external key、排序/limit、删除保护和审计语义。
- **验证**：Windows Application/Persistence 编译、相关静态检查和现有 `match_catalog_repository_contract.rs`；补非法 scope、保护删除、排序/limit及失败无遗留等实际缺口。测试库识别与清理在现有测试内处理，不新建测试入口；数据库执行按第 3 节记录。
- **完成标准**：没有失效方法调用、清单漂移与旧 owner 误报；相关代码/非数据库门禁通过，动态结果或延期项明确。清理本节点临时迁移脚本前确认不再需要。
- **记录**：`R07-01-match-catalog.md`。

## R7-02 外部 ID 身份保护

状态：`DONE`，精确提交 `1e4da04` 的 Windows CI 全通过；PG/XLSX/Full 最终封包新库待验。来源 B1。

- **目标**：同 provider/type/external ID 只能指向同一个稳定实体；同实体重试允许，跨实体冲突拒绝，原绑定不变。
- **范围**：`adapters/catalog/references/external_ids/`，Players Application/Port 的直接调用链，既有 spreadsheet ID 绑定逻辑与 `entity_matching_references_repository_contract.rs`。
- **实施**：核对直写与导入行为；用原子数据库冲突处理确保并发跨实体写入不能覆盖原目标，不采用存在检查后无保护写入；维持同实体 metadata 合并和错误边界。不得增加平行 ID 注册表或自动改绑开关。
- **验证**：首次绑定、同实体重试、不同实体拒绝、并发竞争、失败后原绑定/metadata 保持；导入入口与公开直接入口一致。优先扩充现有 contract，Windows 执行或按第 3 节登记实跑待验。
- **完成标准**：代码不存在无条件跨实体覆盖路径，身份冲突有明确可观察错误；测试结果与未运行项如实记录。
- **记录**：`R07-02-external-id-integrity.md`。

## R7-03 普通删除与历史引用保护

状态：`DONE`，精确提交 `073d155` 的 Windows CI run `36695374298` 全通过，完成记录见阶段索引。真实 PG 删除/并发测试仍 ignored，PG/XLSX/Full 最终封包新库待验。来源 B2。

- **目标**：普通删除只允许删除没有受保护引用的实体；并发新增历史不能被过期预检放行并级联清除。
- **范围**：`adapters/catalog/deletion/` preflight/safe_delete/delete_write/bulk_delete，以及已有 entity deletion/permanent delete/force delete contracts。
- **实施**：在删除事务取得实体保护锁后重新检查受保护关系，保证检查与执行之间不能插入受保护引用；覆盖 Player，并核对 Team 同根问题，不能只检查部分关系。UI 预检仍供展示，不能替代事务内裁决。
- **兼容**：保留 bulk 返回形态；已有引用仍走归档/阻止普通删除。强制删除继续使用完整名称确认、单事务、审计墓碑和既有显式权限，不改成普通删除默认行为。
- **验证**：无引用可删、已有引用拒绝、预检后并发新增 dynamic tag/其他代表性历史时拒绝、事务失败回滚、审计与实际结果一致；补入现有 contract，避免靠随机 sleep 的不确定性测试。
- **完成标准**：事务外预检不再成为删除的最终依据；代码和测试覆盖并发交错，Windows/PG 实跑状态明确。
- **记录**：`R07-03-safe-entity-deletion.md`。

## R7-04 架构清单、验证器与执行记录对齐

状态：`DONE`，精确提交 `9bdb84923da714843e03383db1872aecd30250a2` 的 Windows run `36706905645` / job `109859181344` 全部 SUCCESS；证据见阶段索引与 `R07-04-architecture-inventory.md`。来源 B3/B7；B4 仅修正现有验证入口的遗漏。

- **目标**：机器清单反映当前真实声明与 owner，未登记新增项不能静默通过；进度和审计编号可以追溯。
- **范围**：`architecture/application-port-inventory.json`、`module-boundaries.json`、现有 Application/架构验证脚本及现有 package/frontend 聚合入口，相关阶段 README。
- **实施**：递归发现全部 public Port trait，补齐审计时遗漏的 5 项，比较实际集合与清单集合；递归检查子文件依赖，更新 sourceScan 口径，不能只把常量 38 改成 43。更新已删除的 8 个 adapter 登记与现存路径，不建立第二份清单。
- **现有入口**：保留 R5 经 competition verifier 的间接导入；补接遗漏的 R6-03 player-directory-detail 到现有入口，避免重复执行。不得新增持续回归框架、runner、workflow 或数据库专项入口。
- **记录**：R3-06/07 历史详细记录在 R03 README，可标记该章节为权威记录并建立明确链接，不强制复制成两份新文档；历史 DONE 不因格式差异整体撤销。纠正 R6 删除记录的复检表述，保留审计发现和后续修复证据。
- **验证**：现有 architecture/Ports 检查；验证临时未登记 trait、子文件禁止依赖、失效 owner 能被拒绝且测试后恢复干净工作区。集合以实施时源码为准，不能永久写死审计数量。
- **完成标准**：新增/缺失声明与过期路径均有可检测差异；历史记录和当前状态不混淆。
- **记录**：`R07-04-architecture-inventory.md`。

## R7-05 关键 Application 用例验证

状态：`DONE`，精确提交 `02e56bad` 的 Windows run `36712015030` / job `109875821076` 全通过，53 个 Application 单测包含本项全部 20 个新增行为测试。完成记录见阶段目录，真实 PG/XLSX/Full 仍最终封包新库待验。来源 B5。

- **目标**：验证真实的跨 Port 编排、失败与重试边界，不以 33 个已有测试的通过数代替关键流程覆盖。
- **范围**：现有 Prediction/Research/P4 orchestration/Exchange 单测模块和对应 use cases；必要的最小测试注入边界。沿用现有 Rust 单测与 fake ports，不引入测试框架或私有模型资产。
- **实施**：按风险选取正式与影子运行的写入差异；模型/Port 失败后的后续写入阻止；freeze/research 状态迁移、重复执行和取消/失败后的副作用。fake-port 调用记录应验证顺序、参数与不应发生的动作，而不是只返回固定成功。
- **限制**：不为每个转发函数写镜像测试；不为测试重新拆 Application crate，也不将模型 stub 的“未分发”伪造为真实模型成功。测试暴露的问题仅在同一已确认影响链内修复。
- **验证**：Windows `cargo test --locked -p football-application` 与相关编译/Clippy、既有架构/模型保护检查；这些测试不依赖 PostgreSQL 或网络。
- **完成标准**：所选关键用例的正常、失败与重复执行行为都有明确断言，生产副作用边界保持，测试在 Windows 有实际结果。
- **记录**：`R07-05-application-use-case-tests.md`。

## R7-06 历史数据库失败与账本问题收口

状态：`DONE`，精确提交 `4496399` 的 Windows run `36734194083` / job `109951439572` 全通过，完成记录见阶段索引；四项历史失败与账本仍“已修改、最终封包新库待验”，不记实跑关闭。来源 B6/A8。位于原 R7-02 之前，不再把已知问题只写“后续处理”。

- **范围**：现有 `postgres_integration.rs`、相关现存 P4 persistence 与 `match_exchange.rs`，已有 match-lineup/import tests。只修已确认缺陷和夹具，不提前迁移 R8/R10 整体架构。
- **四项历史失败**：
  1. `match_lineup_chain_versions_model_selection_and_freeze_gate_are_consistent`：按现行合法 11 人阵容构造夹具，保留类型隔离与 freeze 断言。
  2. `match_scope_inference_and_lineup_pair_transaction_are_atomic`：统一 kickoff/T-6h 的有效时间窗口，保留双方原子性和无遗留写入断言。
  3. `structured_match_events_are_queryable_and_revision_aware`：按当前 MatchResultRecord 补足夹具必填字段，保留 revision/history 断言。
  4. `p4_stage_c_writes_are_idempotent_and_frozen_history_is_immutable`：先确认具体时间比较 owner，按实际 PostgreSQL 时间精度统一落库/比较边界；不删除不可变/幂等检查，不用任意宽时间容差掩盖差异，不触碰模型算法。
- **账本问题**：修复结束旧效力期时返回/审计中的 `ended_previous` 与批次账本 `ended_previous_count` 不一致；保持业务写、账本和审计同事务。后续 R7-10/R7-14 迁移必须继承这个修复。
- **验证**：沿用现有 18 项 broad 基线与现存 workbook 测试；对四项及 ended_previous 场景补必要断言，不新增专项 target、runner 或 workflow。Windows 新库实跑按第 3 节，在封包前确认结果。
- **完成标准**：四项及账本问题都有实际修订、测试和责任路径，不以“已登记”冒充修复；代码/非数据库门禁通过后可按第 3 节保留明确的新库待验项，历史失败只有实跑通过才标记关闭。
- **记录**：`R07-06-database-contract-closeout.md`。

## R7-07 Lineup Pair Transaction

状态：`DONE`，精确提交 `3ff21cb` 的 Windows run `36757339619` / job `110030801579` 全通过，102 项 Persistence/53 项 Application 实际通过；真实并发/取消/事务最终封包新库待验。完成记录见阶段索引。原任务 R7-02。

- **目标/契约**：主客两侧由同一个事务拥有，验证双方 match/team、时间和阵容类型；业务行、球员明细与审计一起提交。单侧失败或并发写入不得留下半条阵容或重复有效状态。
- **来源与目标**：player_catalog.rs 中 create_lineup/create_lineup_pair 及 Application LineupPort；迁入 `crates/persistence-postgres/src/adapters/lineups/pair_transaction/`。按实际职责调整私有文件布局，不强制一函数一文件。
- **输入/输出**：沿用对应 Application Port、Domain Draft/Query/Record 与现有持久化结果；允许 Domain、SQLx、现有基础 mapping/audit。禁止依赖前端状态或私有模型实现，不保存第二份事实状态。
- **实施与切换**：识别全部直接调用方和现有测试，完成唯一 owner 切换，同步现有验证器和清单，删除被替代实现；保留上游正常 API/错误契约。数据库事务是写入与审计的共同提交边界。
- **验证**：复用修复后的 pair/scope 现有集成测试，验证单侧非法、完整成功、回滚与并发；保留 R7-06 已修复的合法时间夹具。 Windows 最小检查、现有阶段回归和动态延期规则按第 3 节执行。
- **完成标准**：目标职责完成且无双实现；受影响调用链/非数据库门禁通过，真实数据库/XLSX 结果或最终新库待验项完整登记；下一节点按索引开放。
- **记录**：`R07-07-lineup-pair-transaction.md`。

## R7-08 Lineup Chain 与 History

状态：`DONE`，精确提交 `3c8f7f0` 的 Windows run `36801488090` / job `110176585645` 全通过，详见阶段完成记录；真实数据库边界/并发仍最终封包新库待验。原任务 R7-03。

- **目标/契约**：收敛按时点的阵容选择、版本链与历史读取；actual 不进入赛前模型输入，保留截止时间、优先级、历史可追溯及稳定列表语义。
- **来源与目标**：lineup_chain.rs 与现存 chain/history 调用方；迁入 `crates/persistence-postgres/src/adapters/lineups/history/、adapters/lineups/chain/`。按实际职责调整私有文件布局，不强制一函数一文件。
- **输入/输出**：沿用对应 Application Port、Domain Draft/Query/Record 与现有持久化结果；允许 Domain、SQLx、现有基础 mapping/audit。禁止依赖前端状态或私有模型实现，不保存第二份事实状态。
- **实施与切换**：识别全部直接调用方和现有测试，完成唯一 owner 切换，同步现有验证器和清单，删除被替代实现；保留上游正常 API/错误契约。数据库事务是写入与审计的共同提交边界。
- **验证**：复用 chain/freeze 历史测试，验证类型、边界时点、历史版本、列表 limit 与 preferred lineup；保持 R7-06 的 11 人规则夹具。 Windows 最小检查、现有阶段回归和动态延期规则按第 3 节执行。
- **完成标准**：目标职责完成且无双实现；受影响调用链/非数据库门禁通过，真实数据库/XLSX 结果或最终新库待验项完整登记；下一节点按索引开放。
- **记录**：`R07-08-lineup-chain-and-history.md`。

## R7-09 Team Lineup Presets

状态：`DONE`，精确提交 `3525c04` 的 Windows run `36808609584` / job `110198409740` 全通过，详见阶段完成记录；真实 PG 最终封包新库待验。原任务 R7-04。

- **目标/契约**：预设保存与应用预览分开，预览不得写正式阵容；位置/默认角色、球队成员与主客侧校验有明确 owner。正式应用仍经显式阵容提交。
- **来源与目标**：team_lineup_presets.rs；迁入 `crates/persistence-postgres/src/adapters/lineups/presets/`。按实际职责调整私有文件布局，不强制一函数一文件。
- **输入/输出**：沿用对应 Application Port、Domain Draft/Query/Record 与现有持久化结果；允许 Domain、SQLx、现有基础 mapping/audit。禁止依赖前端状态或私有模型实现，不保存第二份事实状态。
- **实施与切换**：识别全部直接调用方和现有测试，完成唯一 owner 切换，同步现有验证器和清单，删除被替代实现；保留上游正常 API/错误契约。数据库事务是写入与审计的共同提交边界。
- **验证**：沿用现有 preset/role tests 与静态 verifier，检查合法预览、blockers、只读性和显式提交。 Windows 最小检查、现有阶段回归和动态延期规则按第 3 节执行。
- **完成标准**：目标职责完成且无双实现；受影响调用链/非数据库门禁通过，真实数据库/XLSX 结果或最终新库待验项完整登记；下一节点按索引开放。
- **记录**：`R07-09-team-lineup-presets.md`。

## R7-10 Spreadsheet Batch Ledger

状态：`DONE`，精确提交 `cc0d34d` 的 Windows run `36815568401` / job `110219751121` 全通过，见阶段完成记录；真实 PG/XLSX/Full 仍最终封包新库待验。原任务 R7-05。

- **目标/契约**：批次状态、行结果、计数、审计与业务写入保持一致事务；非法状态与未解决冲突不得提交，失败整批回滚。继承 R7-06 ended_previous_count 修复。
- **来源与目标**：spreadsheet_exchange.rs、match_exchange.rs 的 batch/row ledger；迁入 `crates/persistence-postgres/src/adapters/workbooks/batch_ledger/`。按实际职责调整私有文件布局，不强制一函数一文件。
- **输入/输出**：沿用对应 Application Port、Domain Draft/Query/Record 与现有持久化结果；允许 Domain、SQLx、现有基础 mapping/audit。禁止依赖前端状态或私有模型实现，不保存第二份事实状态。
- **实施与切换**：识别全部直接调用方和现有测试，完成唯一 owner 切换，同步现有验证器和清单，删除被替代实现；保留上游正常 API/错误契约。数据库事务是写入与审计的共同提交边界。
- **验证**：沿用现有导入测试与 batch 查询，验证成功、冲突、失败回滚、重试状态、返回/审计/账本计数一致。 Windows 最小检查、现有阶段回归和动态延期规则按第 3 节执行。
- **完成标准**：目标职责完成且无双实现；受影响调用链/非数据库门禁通过，真实数据库/XLSX 结果或最终新库待验项完整登记；下一节点按索引开放。
- **记录**：`R07-10-spreadsheet-batch-ledger.md`。

## R7-11 Player Workbook

状态：`DONE`，精确提交 `310c46b` 的 Windows run `36824027121` / job `110245578512` 全通过，见阶段完成记录；真实 PG/XLSX/Full 最终封包新库待验。原任务 R7-06。

- **目标/契约**：收敛球员及名称、位置、效力期、可用性、能力、标签等子记录 preview/commit；同源 ID 不改绑，保留真实工作簿字段与行身份。
- **来源与目标**：spreadsheet_exchange.rs 的 Player workbook persistence；迁入 `crates/persistence-postgres/src/adapters/workbooks/player_catalog/`。按实际职责调整私有文件布局，不强制一函数一文件。
- **输入/输出**：沿用对应 Application Port、Domain Draft/Query/Record 与现有持久化结果；允许 Domain、SQLx、现有基础 mapping/audit。禁止依赖前端状态或私有模型实现，不保存第二份事实状态。
- **实施与切换**：识别全部直接调用方和现有测试，完成唯一 owner 切换，同步现有验证器和清单，删除被替代实现；保留上游正常 API/错误契约。数据库事务是写入与审计的共同提交边界。
- **验证**：复用现有 workbook/import tests 与真实 XLSX，验证重名/外部 ID 冲突、多子记录、重复导入及整批回滚。 Windows 最小检查、现有阶段回归和动态延期规则按第 3 节执行。
- **完成标准**：目标职责完成且无双实现；受影响调用链/非数据库门禁通过，真实数据库/XLSX 结果或最终新库待验项完整登记；下一节点按索引开放。
- **记录**：`R07-11-player-workbook.md`。

## R7-12 Team Package

状态：`VERIFYING`，11 DONE，用户已启动；共享球队链唯一职责、计数修订和现有回归已实现，等待本项精确 Windows CI；真实 PG/XLSX/Full 最终封包新库待验。原任务 R7-07。

- **目标/契约**：保留球队资料包名称、本地化、成员关系、覆盖范围、冲突裁决与恢复语义；preview 不写业务事实，每条持久化链的 commit 只有一个事务 owner；完整资料包保留现有先球队后球员、球员失败保留已完成球队并可重试的双链边界。
- **来源与目标**：现有资料包复用 monthly_workbooks.rs 的共享球队 preview/commit persistence，原 MonthlyWorkbookPort/API 保持；迁入 `crates/persistence-postgres/src/adapters/workbooks/team_package/`。按实际职责调整私有文件布局，不强制一函数一文件。
- **输入/输出**：沿用对应 Application Port、Domain Draft/Query/Record 与现有持久化结果；允许 Domain、SQLx、现有基础 mapping/audit。禁止依赖前端状态或私有模型实现，不保存第二份事实状态。
- **实施与切换**：识别全部直接调用方和现有测试，完成唯一 owner 切换，同步现有验证器和清单，删除被替代实现；保留上游正常 API/错误契约。数据库事务是写入与审计的共同提交边界。
- **验证**：沿用 team-package、localized-name、real-import-recovery 等既有检查/测试，真实文件预览提交与失败恢复按第 3 节登记。 Windows 最小检查、现有阶段回归和动态延期规则按第 3 节执行。
- **完成标准**：目标职责完成且无双实现；受影响调用链/非数据库门禁通过，真实数据库/XLSX 结果或最终新库待验项完整登记；下一节点按索引开放。
- **记录**：`R07-12-team-package.md`。

## R7-13 Monthly Workbook

状态：`BLOCKED`，依赖 R7-12；原任务 R7-08。

- **目标/契约**：收敛剩余球队/月度球员数据读取与缺口；导入继续复用 11/12 的唯一共享职责。时间窗口、历史效力和数据缺口语义保持，不复制整套导入框架。
- **来源与目标**：monthly_workbooks.rs 剩余 team_monthly_workbook_data/team_monthly_data_gaps/player_monthly_data_gaps 及 player_catalog.rs 月度导出聚合；迁入 `crates/persistence-postgres/src/adapters/workbooks/monthly_team/、adapters/workbooks/monthly_player/`。按实际职责调整私有文件布局，不强制一函数一文件。
- **输入/输出**：沿用对应 Application Port、Domain Draft/Query/Record 与现有持久化结果；允许 Domain、SQLx、现有基础 mapping/audit。禁止依赖前端状态或私有模型实现，不保存第二份事实状态。
- **实施与切换**：识别全部直接调用方和现有测试，完成唯一 owner 切换，同步现有验证器和清单，删除被替代实现；保留上游正常 API/错误契约。数据库事务是写入与审计的共同提交边界。
- **验证**：沿用 monthly-workbooks 既有测试/验证器和真实 XLSX，核对月界、缺失数据、多球队履历及行映射。 Windows 最小检查、现有阶段回归和动态延期规则按第 3 节执行。
- **完成标准**：目标职责完成且无双实现；受影响调用链/非数据库门禁通过，真实数据库/XLSX 结果或最终新库待验项完整登记；下一节点按索引开放。
- **记录**：`R07-13-monthly-workbook.md`。

## R7-14 Match Lineup Workbook

状态：`BLOCKED`，依赖 R7-13；原任务 R7-09。

- **目标/契约**：收敛单场比赛阵容导入导出，保持双方事务、时间截止、actual 隔离、冲突阻断及整批回滚；返回/账本/审计结束履历计数一致。
- **来源与目标**：match_exchange.rs；迁入 `crates/persistence-postgres/src/adapters/workbooks/match_lineup/`。按实际职责调整私有文件布局，不强制一函数一文件。
- **输入/输出**：沿用对应 Application Port、Domain Draft/Query/Record 与现有持久化结果；允许 Domain、SQLx、现有基础 mapping/audit。禁止依赖前端状态或私有模型实现，不保存第二份事实状态。
- **实施与切换**：识别全部直接调用方和现有测试，完成唯一 owner 切换，同步现有验证器和清单，删除被替代实现；保留上游正常 API/错误契约。数据库事务是写入与审计的共同提交边界。
- **验证**：复用现有 match-lineup/workbook 测试和真实 XLSX，验证合法双侧、非法单侧、未解决冲突、回滚、计数一致与重复操作。 Windows 最小检查、现有阶段回归和动态延期规则按第 3 节执行。
- **完成标准**：目标职责完成且无双实现；受影响调用链/非数据库门禁通过，真实数据库/XLSX 结果或最终新库待验项完整登记；下一节点按索引开放。
- **记录**：`R07-14-match-lineup-workbook.md`。

## R7-15 Row 与 Subrecord Identity

状态：`BLOCKED`，依赖 R7-14；原任务 R7-10。

- **目标/契约**：明确物理行号与实体/子记录 identity 的不同语义；同一物理行的多个合法子记录不误去重，多球队效力子记录不被覆盖；不修改冻结迁移来省略处理。
- **来源与目标**：现有导入行和效力期/能力/标签等子记录 identity 规则；迁入 `crates/persistence-postgres/src/adapters/workbooks/identity/`。按实际职责调整私有文件布局，不强制一函数一文件。
- **输入/输出**：沿用对应 Application Port、Domain Draft/Query/Record 与现有持久化结果；允许 Domain、SQLx、现有基础 mapping/audit。禁止依赖前端状态或私有模型实现，不保存第二份事实状态。
- **实施与切换**：识别全部直接调用方和现有测试，完成唯一 owner 切换，同步现有验证器和清单，删除被替代实现；保留上游正常 API/错误契约。数据库事务是写入与审计的共同提交边界。
- **验证**：沿用 verify-import-row-identity.mjs 与现有导入测试，验证同物理行多子记录、稳定定位、冲突/重试与回滚；完成 R7 全阶段复核。 Windows 最小检查、现有阶段回归和动态延期规则按第 3 节执行。
- **完成标准**：目标职责完成且无双实现；受影响调用链/非数据库门禁通过，真实数据库/XLSX 结果或最终新库待验项完整登记；下一节点按索引开放。
- **记录**：`R07-15-row-and-subrecord-identity.md`。

# 阶段出口与最终封包验收

| 验证对象 | 既有方式 | 通过目标 |
|---|---|---|
| Windows 架构/编译/单测/前端/客户端 | 现有 npm scripts、CI、windows-acceptance | 无已知当前代码失败，公共接口与保护资产保持 |
| 身份与删除 | 现有 references/deletion contracts | 不改绑、不误删历史，事务失败不留下副作用 |
| 历史四项数据库失败 | 现有 postgres_integration / run_database_baseline | 相关修复实跑通过；未实跑不能写已关闭 |
| 阵容/历史 | 现有 pair/chain/cutoff tests | 双方原子、actual 隔离、合法窗口与历史链一致 |
| 工作簿 | 现有真实 XLSX preview/commit 验证 | 冲突阻断、整批回滚、行/子记录及账本一致 |

- R7-01～15 的代码和节点最小门禁完成后，更新唯一阶段索引；R7-02～06 没有完成不得绕过进入 R7-07。
- 阶段收口必须说明最终责任目录、入口切换/删除结果、实际差异、Windows 证据、动态待验项、剩余风险与回退提交。精简记录可以引用已有证据，不复制日志。
- 若数据库/XLSX/Full 按用户流程留到最终新库封包，阶段完成记录必须显著列出这些待验项；不得宣称整体验收通过。它们在最终封包是必须完成的门禁。
- 所有已知失败和延期项在最终 Windows 新库验收前闭合；新库的 0001–0046 迁移、已有相关数据库测试、真实 XLSX、Tauri 运行与日志验收均需真实结果。未通过不得封包交付。
- 不新增持续回归体系，不把临时排查脚本长期保留为第二套运行入口。

# 文档与回退

实施记录统一放入 `docs/modular-rewrite/R07-match-lineup-workbook-persistence/`，文件名使用各节点“记录”字段；全部节点完成后生成 `R07-stage-completion.md`，不提前创建。

根 README 记录结果摘要和阶段链接，阶段 README 维护当前唯一状态、原新编号映射与动态待验清单。历史 R1–R6 / 审计中的旧编号保留，标明是当时编号，不无痕改写历史事实。

各节点保留可回退的原子提交；回退到该节点实施前记录的提交并重跑受影响的现有检查，不恢复长期双实现。当前任务仅授权优化任务书，实际代码整改在用户开始对应节点后执行。
