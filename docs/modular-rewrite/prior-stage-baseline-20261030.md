# R1～R12 遗留项基线核对与后续承接

核对日期：2026-10-30。受测提交：`c70ac2f5037ad21c65a55f500778f0fccafad375`。目录：`O:/Project/FOODBALL/123 r3`。

## 结论与适用边界

本次完成12阶段的索引/阶段完成记录核对、已登记遗留项归并、关键风险源码复核和9项当前静态检查。它是R13开始前的遗留项基线，不是前12阶段的全量动态重新验收，也不是缺陷修复完成报告。

当前9项静态检查未发现阻止R13-01只读接入试点的结构/契约失败；仍需用户启动授权。本结论不放行R13整个阶段，更不证明全部软件没有问题。已确认的源码风险在对应业务批次交付前处理；已经明确允许最终新库补验的项目继续逐项承接，最迟R17-06关闭，R18不允许带着必需待验项发布。

R12按用户最新明确纠正保持已结束。本核对不修改R1～R12任务书、阶段索引或历史完成记录，不创建R12新任务，不恢复已否定的UI改版。原67项仍由24批承担；下面BA编号仅为遗留账本编号，不新增开发批次。

## 核对方法与证据强度

- 逐阶段读取R1～R11索引及完成记录，R12读取索引及18记录；以完成结论及后续订正区分历史尝试、已修复失败与未关闭事项。R3-06/07详细记录本就在R3索引，不把缺独立文档误判为未实施。
- 对比赛删除/0041触发器、AI会话回填、四项历史PG测试及时间精度断言、两个验收runner实际targets、架构过渡导入做源码复核。
- 历史CI/产物结果来自仓库记录，本次没有重新联网核验其保存期限或下载产物。引用历史通过仅说明当时覆盖；不是当前候选重新通过。
- 本次没有运行构建、自动行为测试、Windows客户端、真实PG/API、有效XLSX或私有模型回归；未读取用户密钥、未连接/重置数据库。源码确认与动态复现分别记载。

## 十二阶段覆盖表

| 前期阶段及依据 | 既有结果的正确解释 | 本次核对与后续承接 |
| --- | --- | --- |
| [R1 完成记录](R01-architecture-composition/R01-stage-completion.md) | 组合根/架构与当时Windows Automated完成，PG/Full曾延期 | 当前结构检查通过；共享基础继续复用，BA-01/14/15 |
| [R2 完成记录](R02-domain-rewrite/R02-stage-completion.md) | Domain迁移及代表性用户数据库读取完成，不等于专用集成与私有模型通过 | 当前365类型清单通过；历史兼容/固定回归保留，BA-01/13/14 |
| [R3 完成记录](R03-application-services/R03-stage-completion.md)及[索引](R03-application-services/README.md) | Services/Ports与当时Automated完成；破坏性重置、真实数据库/Full存在延期 | 不能从Service门面迁移推导跨Port事务保证；BA-01/08/10/12/14 |
| [R4 完成记录](R04-persistence-foundation/R04-stage-completion.md) §13 | foundation专用PG smoke及重置曾通过；broad当时14/18，四项失败另交业务阶段 | R7已有修订，不能仍写“未修”，也不能写“已实跑关闭”；BA-02 |
| [R5 完成记录](R05-competition-routing-persistence/R05-stage-completion.md) §13 | 赛事/路由专项PG曾通过，用户现有库及Full未由Automated替代 | 复用当时专项证据并检查后续受测变化；最终兼容/Full仍承接BA-14 |
| [R6 完成记录](R06-entity-catalog-persistence/R06-stage-completion.md) §4 | PostgreSQL 16的12个retained contracts有历史通过记录 | 后续R7修改过删除/引用等路径，不能用R6旧结果消除新待验；BA-05 |
| [R7 完成记录](R07-match-lineup-workbook-persistence/R07-stage-completion.md) | 节点及Automated完成；四项修订、账本、并发、XLSX、Full和删除风险待验 | 分拆为BA-02～07，逐项指定实际业务owner |
| [R8 完成记录](R08-prediction-p4-orchestration/R08-stage-completion.md) §13 | 推演/P4外围完成；幂等并发、run/快照跨事务及多次SQL读取边界保留 | BA-08先明确恢复/状态语义；BA-02/13/14保留真实证明 |
| [R9 完成记录](R09-research-ai-backend/R09-stage-completion.md) §13 | 协议/取消/凭据有原测试；普通消息双autocommit、跨Port及取消边界仍保留 | BA-10/11；本次另确认前端异步回填风险BA-09 |
| [R10 完成记录](R10-review-analytics-release-backend/R10-stage-completion.md) §13 | 复盘/结算/分析/候选/报告代码与Automated完成 | BA-12实际账本/事务/文件效果未由unit替代；BA-13/14仍需关闭 |
| [R11 完成记录](R11-tauri-commands/R11-stage-completion.md) §13 | 171命令迁移完成，原后端事务/凭据/模型仍归原owner | 当前三方命令一致；传输接通不消除BA-02～14 |
| [R12 索引](R12-frontend-foundation-ui-system/README.md)及[18记录](R12-frontend-foundation-ui-system/R12-18-样式入口-and-边界.md) | 用户已确认阶段结束；保留文档还含18 VERIFYING/19 BLOCKED旧快照 | BA-16约束状态解释与证据来源；不重开R12，不据此宣称未记录的动态结果通过 |

## 遗留账本

状态含义：**本次静态通过**只适用于检查范围；**已修未实验**指历史已有修订但缺真实环境关闭结果；**源码风险确认**不等于本次动态复现；**继承边界待核实**不能未经评估升级为新功能；**真实环境待验**必须有实际结果才能关闭。

| 编号 / 当前状态 | 具体问题、来源与证据 | 责任批次、处理截止点与关闭条件 |
| --- | --- | --- |
| BA-01 / 本次静态通过 | 组合根/模块依赖、状态owner、Domain、171命令、模型保护、迁移兼容及静态数据库基线；见本报告检查表 | R13-01启动前核对受测SHA与差异；后续每批按影响复验。任何实际基础门禁失败阻塞依赖工作，不用本次旧PASS覆盖 |
| BA-02 / 已修未实验 | R4四项历史失败；R7-06已修夹具/精度并保留原18 broad测试，当前仍需显式ignored真实库运行，详下表 | R13-07、R14-06/08分别核对相关修订未丢失，真实复验由R17-06逐项关闭；已批准延期不能写成通过 |
| BA-03 / 源码风险确认 | `crates/persistence-postgres/src/adapters/matches/catalog/delete.rs` 保护计数含research/freeze/settlement，却随后将 `model.runs.match_id` 置空；0041触发器禁止该变更。孤立的模型运行引用路径可能以数据库异常结束，当前未动态复现 | **R13-07交付前**核实并处理受保护拒绝语义与错误呈现，补无保护引用/有model run/已有P4或结算三类场景；不能放宽0041或移除历史保护。真实PG读回及事务回滚证据最迟R17-06关闭；若需改变删除产品策略/Schema先停止并报告决策需求 |
| BA-04 / 已修未实验 | R7-06的A8：阵容导入 `ended_previous_count` 已修，仍缺当前新库证明返回、批次账本、审计一致及末行失败回滚 | R13-07与R13-09复用同一账本工作流，不能迁回旧计数实现；R17-06真实验证12插入/1旧版本结束的既有场景、整批回滚、修复重试和成功批次重复拒绝 |
| BA-05 / 真实环境待验 | R6历史专项通过后，R7增加外部ID、实体引用复检、父锁和删除并发保护；历史专项不能代表后改代码已实跑 | R13-05/06交付前保留同源ID不改绑、引用保护及明确拒绝；R17-06补实际匹配/删除/强删contracts、并发提交/回滚及历史读回 |
| BA-06 / 真实环境待验 | R7双边/单边/工作簿共同锁、取消、历史cutoff/actual隔离、恢复归档、预设明细原子性仍待新库 | R13-07实现/自动场景随批完成，R17-06补真实锁竞争和回滚；不能承诺网络断开必定取消已经提交的数据 |
| BA-07 / 真实环境待验 | R7真实有效Player/Team/Monthly/Match XLSX、行/子记录身份、预检暂存、裁决、批次恢复与读回仍待验 | R13-05/06/09维护完整链；R14-08承接复盘资料包。R17-06用有效文件跑到数据库读回；内联JSON、生成预览或解析成功不代表提交成功 |
| BA-08 / 继承边界待核实 | R8记录：不同幂等key竞争同正式队列可遇唯一约束；run先提交后快照失败可能留run；多次SQL无新增统一快照保证 | R14-01/06/07接入前按真实事务分界确定状态/错误/恢复，故障时查回已有run，不盲重跑。能否接受部分完成须有明确业务语义；违反现有身份/历史要求则修复后交付，不能仅移文件。R17-06给真实并发/故障读回证据 |
| BA-09 / 源码风险确认 | `src/main.ts:2102` 的 `stillViewingOrigin` 在列表刷新await前计算，刷新后继续使用；`:6953`选择会话直接await后回填，缺此分支自身的请求序号核对。本次未动态复现 | **R15-01交付前**以真实工作流受控延迟证明复现并修复，每次await后重核身份，覆盖A发送→刷新中切B、A/B/C乱序选择、B草稿保留；沿既有R15-01-V1～V4验收，不能保留旧竞态作为兼容要求 |
| BA-10 / 继承边界待核实 | R9普通消息双autocommit、会话/尝试证据跨Port、准备中取消不能撤销已提交写入，多次查询不保证同一快照 | R15-01交付前明确请求/会话/草稿和终态归属，部分失败或取消不伪造全回滚；若现有产品必须原子则先补行为/事务修复。R17-06真实PG/API检查消息/尝试账本、重复发送及取消读回 |
| BA-11 / 真实环境待验 | R9/R11的真实API、Windows Credential Manager、取消UI端到端尚待；未知密钥不能靠通用正则保证全识别 | R15-07/12维护显式保存/清除和统一脱敏；R15-01维护取消身份。R17-06使用授权凭据验证，不在日志/报告公开密钥；测试样例通过不代表用户凭据实跑 |
| BA-12 / 继承边界与真实环境待验 | R10记录跨Port事务、供应商逐组autocommit、文件先写后元数据、重复影子报告后续更新；九步复盘/结算幂等、候选人工门禁、隔离和不可变报告未获新库实证 | R14-08/12/13/16/17逐域检查，部分失败必须可追溯并可恢复，不前端拼接全成功；R17-06验证实际账本、时间切分、生产绑定/回滚及不可变触发器，R18拒绝未闭合报告 |
| BA-13 / 真实环境待验 | 公开保护资产静态一致；检查器明确私有P4/P7资产缺席，不具备私有Golden Master通过证据 | R14-01/06/08/16保持私有ModelProvider与公开unavailable边界；R17-06取得授权资产并运行原固定回归，缺失则阻塞最终验收；不以公开stub/模拟概率替代 |
| BA-14 / 真实环境待验 | R1以来Full/用户Windows实机与当前候选验证不可由startup替代；两个既有数据库runner仅默认调match_catalog与postgres_integration，不覆盖所有20个现存target文件 | R17-01建立实际target/操作矩阵，R17-06执行专用新库、完整客户端链与额外必要contracts，R18-01核对最终MSI/NSIS来源及安装运行；不得把文件数量当执行数量或自动回车替代Full操作 |
| BA-15 / 登记过渡边待退出 | 当前模块检查通过但明确保留 `src/pages/prediction.ts → src/pages/p4Workbench.ts` 及 `→ src/pages/runs.ts` 两条过渡导入，机器契约已有退出任务 | 分别R14-06/07以具名公开接口消除跨页面内部依赖；R16-09确认实际图及过渡清单一致，不能只删例外或扩白名单获得PASS |
| BA-16 / 状态解释已明确，最终证据待核 | R12用户确认结束优先于保留索引中的旧快照；旧记录不提供当前最终候选的全量证明，也不据此要求重做R12-19 | R13-01沿用户最新状态开始前置核对；R17-01维护现有基础设施断言，R18-01验证精确最终候选。需要追溯旧证据时单次核精确提交，不轮询、不冒称未查的CI已通过 |

### BA-02四项必须分别关闭

以下符号均在当前 `crates/persistence-postgres/tests/postgres_integration.rs`，并保留显式ignored。R7修订依据见[R7-06当前验证状态](R07-match-lineup-workbook-persistence/README.md#r7-06-当前验证状态)。

| 实际测试函数 | 原问题与修订核对点 | 业务承接 |
| --- | --- | --- |
| `match_lineup_chain_versions_model_selection_and_freeze_gate_are_consistent` | 十人confirmed非法夹具；合法11人、历史/cutoff及冻结准入 | R13-07、R14-06 |
| `match_scope_inference_and_lineup_pair_transaction_are_atomic` | kickoff/T-6h时窗夹具；同时承载A8和双边原子/并发断言 | R13-07、R13-09 |
| `structured_match_events_are_queryable_and_revision_aware` | result_snapshot缺必填赛果字段；不能弱化真实赛果契约 | R14-08 |
| `p4_stage_c_writes_are_idempotent_and_frozen_history_is_immutable` | PG微秒与内存亚微秒差异；当前测试已有精度读回、不同指纹拒绝及一微秒边界断言 | R14-06 |

上表本次只验证函数/断言存在，不报告实跑结果。R17-06须给每项实际结果；任一失败先修复并重验，禁止用整组“库已连接”关闭。

## R13启动及后续放行规则

1. **启动前必须完成基线核对，而非提前宣称所有遗留已修复。** 本报告已覆盖12阶段；R13-01开始时核对当前SHA与此基线的代码/配置/资产差异及用户修改，变化影响的检查重跑。用户授权、适用前置和当前基础门禁缺一不可。
2. BA-03、BA-09已发现的源码风险必须在对应业务批次交付前处理；若在更早任务中实际触发，立即阻塞受影响路径，不等到编号到达才修。相关后端修复须按真实影响声明范围，不擅改模型或Schema。
3. BA-08/10/12是明确继承的事务边界，先核实现有产品要求与失败恢复；不得直接承诺跨Port全原子，也不得把真实业务缺陷解释成“老问题”而继续交付。
4. BA-02/04～07/11/13/14中的真实新库或外部验证，只有原已批准延期范围可交R17-06，记录实现/自动检查与真实未验两种状态。新出现的失败、跨对象写入或历史损坏不属于可自动延期项目。
5. R16只收敛遗漏；不能把已知阻塞问题一律推到R16。R17-06逐项关闭必需真实证据；R18-01复核剩余项为零或具备不影响必需功能的明确处置依据，不能靠改状态/删功能/关闭检查发布。

每项后续更新记录：BA编号、来源、当前基线、复现/验证层级、实际修复提交、验证命令和退出码、证据位置、剩余环境条件、承接批次、是否阻塞。关闭必须满足本项条件；本报告中“源码风险确认”不能直接改成“已修复”。

## 本次实际运行的静态检查

执行环境为Windows PowerShell，进程TEMP/TMP加载现有 `O:/Project/FOODBALL/project-temp.ps1` 指向项目 `tmp`；每条执行后核对退出码。以下9条均退出0，没有执行其中提及的PG测试或客户端操作。

| 实际命令 | 本次结果与界限 |
| --- | --- |
| `node scripts/architecture/verifyModuleBoundaries.mjs` | PASS：扫描270前端文件、11 Rust crate、18个Feature契约；两条登记过渡边仍在，见BA-15，非Feature全部已实现 |
| `node scripts/architecture/verifyStateOwnership.mjs` | PASS：38状态ID与owner源位置；不证明全部运行时竞态正确 |
| `node scripts/architecture/verifyProtectedImports.mjs` | PASS：270前端、1494 Rust文件、12 Cargo清单 |
| `node scripts/verify-domain-type-inventory.mjs` | PASS：365公共兼容类型、301 PG映射类型 |
| `node scripts/verify-command-contract.mjs` | PASS：前端/注册/源码171命令一致 |
| `node scripts/verify_command_contract.mjs` | PASS：171命令、12命令模块；没有用名称相似删除其中一个守卫 |
| `node scripts/verify-database-migration-compatibility.mjs` | PASS：历史兼容源码门禁、11个旧版本对账规则；未连接旧库实测 |
| `node scripts/verify-protected-assets-deterministic.mjs` | PASS：18保护资产；聚合 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`；私有P4/P7缺席 |
| `node scripts/verify_database_baseline.mjs` | PASS：46迁移、18 broad测试契约及不可变约束静态一致；聚合 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`；非真实PG通过 |

## 本轮修改与恢复

新增本报告，并将账本交叉引用写入R13～R18承接批次、共享执行约定、规划审计和根README。生产代码、测试、Schema、模型资产及R1～R12文档保持不变；本次没有删除文件。文档恢复可针对本次提交审阅后revert，不触碰数据库或回退用户其他修改。

文档交付检查通过：12阶段来源、16项账本与24批承接完整；原67项覆盖、79个既有场景和PowerShell代码块保持；238处本地链接、源码锚点及4个历史测试符号有效；仅10份Markdown变更，Git空白检查通过，前12阶段与生产文件无差异。
