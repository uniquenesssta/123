# R7-15 Row 与 Subrecord Identity：节点完成记录

状态：`DONE`（共享身份职责和 Windows Automated 完成；真实 PG/XLSX/Windows Full 最终封包新库待验）。

## 基线与目标

- 唯一分支：`rewrite/r7-match-lineup-workbook-persistence`；进入/受控回退基线 `bb0012085d92cb63742ff5b5571a8003984db709`。
- 实施提交 `680b5890dfcd5dff1eb6006ae400ab23153defa9`；修订后的最终代码 `a928c8b5ddcf37b569fbdab8c413be06d6270aac`，树 `70ca7b7f2f161dd8a490bf0aff14fc9a5d57cd2e`；完成日期 2026-10-01。
- 对应 [R7 任务书](../../football-model-platform-modular-rewrite-19-docs/07-R7-match-lineup-workbook-persistence.md)。明确 UUID 定位、物理来源行和子记录唯一身份；保留同一物理行的多个合法实体/子记录，删除被迁移职责的双实现。

## 实际变更与兼容性

8 个原重复球队行合并函数和 5 个原 inline 测试迁入共享 `workbooks/identity/teams.rs`；Team commit 与来源关联调用唯一 owner。原函数体保持，包括国家/球队类型隔离、名称/来源规范化、显式总览优先、缺失字段填充、零/false/原显示名保留；Team values 五项策略只调整 workbooks 内可见性，不复制实现。提交仍借用原事务及 batch_ledger，不建立第二套账本。

`identity/row.rs` 提供暂存行 UUID/batch/sheet/physical/entity 的受检投影，替换原 `row_number as i32`。Player/Team/Match 三类预检在同源查找、取消或创建之前拒绝不满足原 >=2 CHECK 与 PostgreSQL integer 范围的行号，避免异常输入先取消旧 pending 批次再插入失败。原合法行号、UUID、载荷/候选/匹配字段与 SQL 保持。

子键仍由冻结 0043 生成列和五字段 UNIQUE 唯一判定：能力/标签代码 BTRIM，效力 team_id→team_key→lower(BTRIM(team_name))，其余空键。Rust 不重算子键、不按物理行去重、不使用 ON CONFLICT 吞掉重复。原 spreadsheet-io 空白行/多球队/多子记录解析保持原 owner。公开 API/DTO/Ports、正常错误优先级、事实/审计/回滚与冲突 UUID 定位保持。

## 文件与调用边界

| 范围 | 实际变更与唯一职责 |
|---|---|
| 新增 `identity/{mod,row,teams}.rs` | 注册共享身份、物理行投影/前检、原重复球队合并及测试 |
| `team_package/{identity,commit,mod,values}.rs` | 删除已迁移函数和测试；提交/来源调用新 owner；共享策略内部可见性 |
| `batch_ledger/rows.rs`、三类 `preview.rs`、`workbooks/mod.rs` | 原 INSERT 前受检投影，原批次副作用前预检，注册 identity |
| 原 `postgres_integration.rs`、八个原 verifier | 扩原子记录回归并适配实际 owner，保留旧断言及测试目标 |
| 原架构清单、根 README、任务书/索引、DATABASE/TESTING | 更新使用面/PG 指纹、兼容边界及验证实情 |

仅迁移原函数/测试，无整个旧文件删除；Team identity 的候选、批次引用等职责继续保留原 owner。下层 shared identity 复用 Team values 和 batch_ledger，无逆向调用业务编排；Ports 和 Application 用例不改。完整变更可由上述实施/修订提交 diff 复核，无新增依赖/迁移/workflow/runner/数据库。

## 测试与失败修订

新增现有生产 row.rs 中 3 项 inline：header/整数溢出拒绝、同物理行两子记录保持独立 UUID、前检不修改或去重实体/子记录；原 5 项迁移不增加数量。本项 Windows Persistence 135/Application 55 已实际通过。

沿用原 PG `team_package_player_team_period_subrecords_are_distinct`，扩生成键、同物理行多实体/能力/标签/双球队、跨行/跨表十条生产 skip 预检记录、UUID/载荷读回、重复维度整批回滚、三类非法行号前检、同源 pending 保留及账本/事实/审计无失败遗留。18 broad PG 数量不变，尚未实跑。

首轮 run `36866936459` 因旧 Team identity 遗留未使用的 PersistenceError 导入在 Clippy `-D warnings` 失败。修订 `a928c8b5ddcf37b569fbdab8c413be06d6270aac` 删除该无调用导入并同步官方使用摘要，无 allow、无业务/断言变化；本次精确 CI 已验证该失败关闭。83 项现有源码检查、架构和 Rustfmt 检查通过；六项临时破坏探针全部拒绝并恢复。它们不替代真实数据库测试。

## 精确 Windows 证据

精确 `a928c8b5ddcf37b569fbdab8c413be06d6270aac` / [Windows run 36871154039](https://github.com/uniquenesssta/123/actions/runs/36871154039) / job `110398949507` 全 SUCCESS；Persistence **135 项**、Application **55 项**、17 个截图视口、前端类型/生产构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 与启动日志 **7 条记录 / 3 个完成操作**实际通过。

2026-10-01 22:15:38（北京时间）更新为 completed/success，本轮核实 SHA、全部步骤、单测日志与产物。artifact `11167454668`，14,018,298 字节，SHA-256 `8843e8c822b590dc4fd4a8f56e5ab4df477fe36ae8c11592795de852fa1fbc20`。Domain 365/300、声明摘要保持；使用摘要 `60ddd966be60fb94f88b72de3f4fcd97b1deb8b73ab988d549c86e861a98a6a8`，PG source blob `675329e00ee90bedbf094040a8147c2165764b9e`。

## 延期、阶段收口与回退

18 broad PG 及其他数据库 contracts 在 CI 仍 ignored，历史四项、账本、并发/回滚、有效 XLSX 与 Windows Full 继续最终封包新库待验。节点 DONE 不等于这些门禁通过，详见 [阶段完成记录](R07-stage-completion.md)。本轮只补文档，验收引用上述精确最终代码；不重复编译打包、不修改源码或提前实施 R8。

受控回退到进入基线或针对实施/修订提交 revert，同步 owner、调用、前检、门禁/清单和 PG 指纹，不恢复双实现或放宽检查。冻结 171 命令/43 Ports/365 Domain、保护资产、生产依赖/锁文件和 0001–0046 迁移。
