# R07 阶段完成记录

阶段状态：`DONE`（R7-01～15 代码/节点和 Windows Automated 出口完成）。

**最终封包尚未验收完成：真实 PostgreSQL、四项历史数据库失败的修订、账本/并发/回滚、有效 XLSX 和 Windows Full 仍为“最终封包新库待验”。现有 Match 删除与不可变 model.runs 输入身份的继承风险也尚未验证关闭。阶段 DONE 不表示这些门禁通过；封包前必须实际闭合。**

## 基线与完成范围

- 唯一阶段分支：`rewrite/r7-match-lineup-workbook-persistence`，起点 `a25da2bdd2b93d8146e46af60947a687e998f987`。
- 最终已验收代码 `a928c8b5ddcf37b569fbdab8c413be06d6270aac`；树 `70ca7b7f2f161dd8a490bf0aff14fc9a5d57cd2e`；完成日期 2026-10-01。
- 依据：[R7 任务书](../../football-model-platform-modular-rewrite-19-docs/07-R7-match-lineup-workbook-persistence.md) 第 3 节延期规则及阶段出口，[唯一阶段索引/累计审计](README.md)。
- 本轮收尾仅创建完成记录并更新状态/验证文档；源码、测试、清单及工作流与上述成功提交一致。引用该精确源码证据，文档提交不重复 Windows 编译打包。

R7 按用户决定保留 R1–R6 已完成成果，先执行新增 R7-02～06 累计审计整改，再完成原 R7-02～10 顺延的 R7-07～15。实际修复身份不改绑、普通删除锁后完整复检、架构/调用清单、Application 关键用例、PG 夹具/时间精度/账本，以及阵容与各工作簿职责收敛；不建设新的持续回归体系。

## 已完成节点

| 任务 | 实际范围 | 记录 | 状态 |
|---|---|---|---|
| R7-01 | Match Catalog | [完成记录](R07-01-match-catalog.md) | DONE |
| R7-02 | 外部 ID 身份保护 | [完成记录](R07-02-external-id-integrity.md) | DONE |
| R7-03 | 普通删除与历史引用保护 | [完成记录](R07-03-safe-entity-deletion.md) | DONE |
| R7-04 | 架构清单、验证器与执行记录 | [完成记录](R07-04-architecture-inventory.md) | DONE |
| R7-05 | 关键 Application 用例测试 | [完成记录](R07-05-application-use-case-tests.md) | DONE |
| R7-06 | 历史数据库夹具与账本修订 | [完成记录](R07-06-database-contract-closeout.md) | DONE |
| R7-07 | 双方阵容共同事务 | [完成记录](R07-07-lineup-pair-transaction.md) | DONE |
| R7-08 | 阵容链与历史 | [完成记录](R07-08-lineup-chain-and-history.md) | DONE |
| R7-09 | 球队阵容预设 | [完成记录](R07-09-team-lineup-presets.md) | DONE |
| R7-10 | 工作簿批次/行账本 | [完成记录](R07-10-spreadsheet-batch-ledger.md) | DONE |
| R7-11 | 球员工作簿 | [完成记录](R07-11-player-workbook.md) | DONE |
| R7-12 | 球队资料包 | [完成记录](R07-12-team-package.md) | DONE |
| R7-13 | 月度工作簿 | [完成记录](R07-13-monthly-workbook.md) | DONE |
| R7-14 | 比赛阵容工作簿 | [完成记录](R07-14-match-lineup-workbook.md) | DONE |
| R7-15 | 行与子记录身份 | [完成记录](R07-15-row-and-subrecord-identity.md) | DONE |

各记录保留自己的精确 Windows 证据、实施/修订范围和动态待验。最终 R7-15 出口同时通过 R1–R6 既有架构入口及 R7 当前全部非数据库门禁；不是将旧阶段推倒重做或为所有分支执行新一轮审计。

## 最终职责与入口切换

| 责任目录/接口 | 唯一职责与保留边界 |
|---|---|
| `adapters/matches/catalog/` | 比赛 create/delete/list/read/scope/mapping；原 Match API 与正常错误保持 |
| `adapters/catalog/entity_matching/`、`deletion/` | 外部 ID 幂等/不改绑，Player/Team 普通删除同事务父锁后共享引用复检 |
| `adapters/lineups/pair_transaction/` | single/pair 创建、共同比赛父锁、原角色与写入、成对原子性 |
| `adapters/lineups/chain/`、`history/` | 时间窗口/校验、活动版本链、历史读取/删除/恢复与模型 cutoff/actual 隔离 |
| `adapters/lineups/presets/` | 预设读/写/预检/永久删除与明细回滚 |
| `adapters/workbooks/batch_ledger/` | Player/Team/Match 原批次/行锁、暂存、计数、状态与映射；借用原事务 |
| `adapters/workbooks/player_catalog/`、`team_package/` | 球员、球队资料包预检/裁决/提交与原依赖写入顺序/双链恢复 |
| `adapters/workbooks/monthly_team/`、`monthly_player/`、`monthly_gaps.rs` | 原月度只读聚合与缺口，原 SQL/投影保持 |
| `adapters/workbooks/match_lineup/` | 原导出/AI 上下文/预检/裁决/整批提交；工作簿合法单侧替换保留，pair 要求双方 |
| `adapters/workbooks/identity/` | 原重复球队合并、受检行定位；子键由冻结数据库生成列/UNIQUE 唯一判定 |
| Application services/use_cases/composition 与既有 Ports | 保留用例边界及显式 Persistence 入口分派；不使核心用例依赖新具体数据库实现 |

旧根 `lineup_chain.rs`、`team_lineup_presets.rs`、`spreadsheet_exchange.rs`、`monthly_workbooks.rs`、`match_exchange.rs` 及相关注册已移除，不留空转发壳或双写。根 `player_catalog.rs` **仍保留**原 `player_catalog_reference_data` 真实引用数据聚合和原回归；它不再持有已迁出的 Match/Lineup 写读/历史实现。解析器继续归 spreadsheet-io，数据库 identity 不向解析器反向渗透。

阶段起点到最终代码的真实 diff 为 **150 个文件，19,901 行增加、12,491 行删除**（含阶段各节点文档/测试/门禁/清单，不含本轮纯文档收尾）。生产依赖及锁文件、46 条迁移和 CI workflow 无变化；package.json 只补既有 architecture 验证入口，不新增 runner/framework。各节点记录给出责任内变更与回退基线，不按文件数判断架构完成。

## 最终 Windows 出口证据

[Public Platform CI run 36871154039](https://github.com/uniquenesssta/123/actions/runs/36871154039) 对应上述最终代码，Windows job `110398949507` 全部步骤 SUCCESS，2026-10-01 22:15:38（北京时间）更新为 completed/success。

| 验证对象 | 实际结果 |
|---|---|
| 完整 architecture、命令/Ports/Domain、保护与数据库静态契约 | PASS；171 命令、43 Ports、365 Domain/300 PostgreSQL 映射保持 |
| Windows Rust fmt、workspace all-targets Clippy `-D warnings`、workspace tests | PASS；Persistence **135/135**、Application **55/55**，其余 workspace 已执行测试零失败 |
| 前端契约/类型/浏览器交互/生产构建 | PASS；**17** 个截图视口通过 |
| Tauri Windows release 与 MSI/NSIS | PASS，两类安装产物构建成功 |
| release 客户端启动和日志 | PASS；**7** 条记录、**3** 个完成操作 |
| broad PG 与其他数据库 contracts | 18 broad PG 及其他 contracts **ignored**；未记为真实 PG PASS |

artifact `11167454668`，14,018,298 字节，SHA-256 `8843e8c822b590dc4fd4a8f56e5ab4df477fe36ae8c11592795de852fa1fbc20`。该证据是 Windows Automated；CI 的日志汇总“全链路通过”不代表需要真实库和人工流程的 Windows Full 已执行。

R7-15 首轮 run `36866936459` 因旧 owner 未清理 unused PersistenceError 导入失败；最终修订删除无调用导入、同步使用指纹，未抑制警告或放宽断言，成功 CI 已实际关闭此失败。83 项既有源码/契约检查与六项身份破坏探针可作为辅助证据，不能替代 PG 实跑。

## 最终新库待验清单与剩余风险

| 待验对象 | 必须确认的实际结果 | 现有依据/执行方式 |
|---|---|---|
| 四项历史失败修订 | chain/freeze 十人非法夹具与合法 11 人历史/cutoff；pair/scope T-6h 开启；structured events 完整赛果；P4 亚微秒幂等/真实时间边界均实跑通过 | 原 postgres_integration/run_database_baseline，详见 [R7-06 表](README.md#r7-06-当前验证状态) |
| A8 与各工作簿账本 | 结束旧版本计数与返回/账本/审计一致；末行失败整批回滚、修复重试、成功批次重复提交拒绝 | 原 pair/workbook PG 夹具及 retained contracts |
| 外部 ID、Player/Team 普通删除 | 幂等不改绑；并发引用提交/回滚、父锁后共享裁决、事实/历史/审计保持 | 原 references/deletion/permanent-delete contracts |
| 阵容、历史与预设 | pair/single/workbook 共同锁竞争、提交前取消、恢复/归档、cutoff/actual 隔离、预设明细回滚 | 原 pair/chain/history/preset PG 测试；不承诺网络不确定提交必回滚 |
| 工作簿及行/子记录身份 | 各类预检只暂存、裁决/计数/整批提交/回滚；同物理行多子记录、UUID 读回、重复键拒绝、异常行号不取消旧批次 | 原 PG 工作簿/子记录用例，18 broad 数量保持 |
| Match 删除与不可变模型历史 | 现有 delete_match 尝试清空 model.runs.match_id，而 0041 输入身份触发器禁止变更；需要含模型运行引用夹具验证正确保护/拒绝语义 | [R7-01 继承风险](R07-01-match-catalog.md) 与阶段索引；**未修改该策略、未验证关闭**，禁止放开历史触发器掩盖风险 |
| 真实有效 XLSX | Player/Team/Monthly/Match 解析→预检→裁决→提交→读回的真实文件流程、错误提示和子记录完整性 | 原 Application/Spreadsheet/客户端流程；inline 或合成载荷不冒充 XLSX 实跑 |
| 最终新库 Windows Full | 0001–0046 新库迁移、相关数据库测试、客户端完整操作及实际日志无已知失败 | 原 `scripts/run_database_baseline.mjs`、已有 test targets、`scripts/windows-acceptance.ps1 -Mode Full` |

沿用已有专用测试库和已有命令；最终封包按用户流程建立新库，专用可清空库要求沿用既有安全检查，不清空用户业务库、不新增数据库设施/专项 runner/workflow。所有延期和风险必须取得真实结果，未通过不得最终封包交付。Linux/macOS 动态验证不列为补验。

## 兼容性、回退与下一阶段

公开命令/Ports/DTO、原正常规则、原事务与历史保护保持；修复已确认的身份、引用复检、时间精度/账本、异常行号缺陷不作为正常业务变更扩展。模型算法/参数/私有资产、0001–0046、生产依赖/锁文件继续冻结。

受控回退以各节点进入提交和实施/修订提交 revert 为依据，同时回退 owner、调用、现有 verifier、使用清单及 PG source 指纹；不能复制旧文件建立双实现、放宽原门禁或历史触发器。整体起点仅作审查/比较依据，回退前评估后续依赖。

R7 已满足任务书允许延期条件下的节点/Windows Automated 阶段收口，下一阶段按 [R8 任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md) 与总纲执行，继承本表中的动态待验责任。**R8 未启动，本轮不创建第二条实施分支或修改 R8 代码。**
