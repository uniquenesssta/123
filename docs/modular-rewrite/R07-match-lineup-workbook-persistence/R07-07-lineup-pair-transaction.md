# R7-07 Lineup Pair Transaction：节点完成记录

状态：`DONE`（唯一创建职责、并发锁与 Windows Automated 完成；真实 PG 并发/取消/回滚及 XLSX/Full 仍“最终封包新库待验”）。

## 实际范围与责任路径

`adapters/lineups/pair_transaction/` 唯一持有单侧/双方创建入口、结构校验和事务写入；旧 player_catalog 创建实现删除。单侧和双方以 READ COMMITTED、比赛 FOR UPDATE 父锁保护版本链，工作簿复用相同锁与调用方批次事务。明细、单侧及双方审计共同提交；公开接口、身份/窗口/类型校验、角色来源、11 首发/阵型/actual 门禁及提交后读回语义保持。

现有 pair 集成测试补身份错误、单侧失败审计回滚、提交前外键等待期间取消、两次 pair/一次 single/一次 workbook 混合父锁等待及完整版本/审计/账本断言。3 个 inline 结构校验测试实际执行；没有新 target、runner、workflow 或数据库设施。已有验证器增加唯一职责/事务/锁检查，五项破坏探针均拒绝并恢复。取消发生在 commit 往返期间时服务器结果可能不确定，不能承诺所有取消均回滚。

## 精确 Windows 证据

- 提交：`3ff21cba4ab01bc02108e8545f0ed133e25de687`；树：`b0f44bf4c443eb3e97b97756990ed55f1eba4239`。
- [Public Platform CI run 36757339619](https://github.com/uniquenesssta/123/actions/runs/36757339619)，Windows job `110030801579` 全步骤 SUCCESS；北京时间 2026-10-01 02:35 run 更新为 completed/success，本轮收尾核实。
- Windows fmt/workspace all-targets Clippy（拒绝 warnings）/workspace tests、架构、前端静态契约/类型/17 个截图视口/生产构建、release/MSI/NSIS 打包及启动通过。Persistence inline **102 项**、Application **53 项**全部通过，包含本项 3 个结构测试。
- Runtime 日志验收 PASS：7 条记录、3 个完成操作。artifact `11117629192`，14,019,726 字节；SHA-256 `b02e5b65456cccf28c4b29dd2095cc24297d5d447e7a05fd89c17c32da915dff`。
- broad PostgreSQL 仍 0 passed、0 failed、18 ignored；其他 PG contracts 仍 ignored，仅编译通过。没有真实数据库 PASS，也未将 R7-06 四项历史失败标为实跑关闭。

## 延期、下一项与回退

真实事务/并发/取消、有效 XLSX、Windows Full 仍沿用最终封包新库流程；封包前实跑并清零待验。用户确认通过并启动 R7-08 Lineup Chain / History；新代码不得继承本项 Windows PASS。

进入/回退基线：`4496399bd520b324360826d613113779c416c41e`。受控 revert 本项时同步创建职责、共同锁、Application 分派、清单/指纹与原验证器，重跑受影响门禁，不独立放宽检查。公开 API/DTO、模型算法/参数/保护资产、依赖/锁文件与 0001–0046 迁移保持。
