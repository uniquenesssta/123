# R7-12 Team Package：节点完成记录

状态：`DONE`（共享球队资料包职责、裁决计数修订及 Windows Automated 完成；真实 PG/XLSX/Windows Full 仍最终封包新库待验）。

## 实际范围

70 个共享球队导入函数和 22 个原测试从 monthly_workbooks.rs 迁入唯一 `adapters/workbooks/team_package/`，职责为 preview/conflict/commit/write/names/identity/formation/validation/values。Team 批次/行 SQL、同源复用、只读 preview 与严格 codec 复用原 batch_ledger；MonthlyWorkbookPort 四入口显式分派。95 个未改原函数体和七个公开签名保持。原文名、中文主名、简称、身份隔离/合并/歧义、子记录、覆盖范围保持。

修复 Team 人工 skip/候选解决后的 pending skipped/error 计数未同步：裁决行及计数共同父锁事务提交。预检仅暂存；球队 commit 唯一事务内规范化、合并/关联、写事实/行，再同一 result 写原计数/审计，成功重试读回、末行失败整链回滚。完整资料包保留原双链边界：球队先提交，球员失败时保留已完成球队并可原批次重试。

新增 Persistence inline 5 项、Application policy 2 项，原 PG 月度夹具补重复预检身份/无事实、末行失败回滚/同批次恢复、一次审计、名称共存、非法候选拒绝和裁决计数。既有门禁路径与唯一 owner/事务/恢复检查更新；83 项源码、架构、Rustfmt 通过，六项破坏探针拒绝并恢复。无新测试目标、runner、workflow、数据库设施、依赖或迁移。

## 精确 Windows 证据

- 提交 `fe2f7ebcbf50e4bb40016d649a95b2c295aeab55`；树 `9c3fc39ad1615b8ca3a142a2b588ca87c3841b25`。
- [Public Platform CI run 36831302483](https://github.com/uniquenesssta/123/actions/runs/36831302483)，Windows job `110268300510` 全步骤 SUCCESS；2026-10-01 16:05:00（北京时间）更新为 completed/success，本轮核实。
- 架构、前端契约/类型/17 个截图视口/生产构建、Windows fmt/Clippy/workspace tests、release/MSI/NSIS 打包与启动通过。Persistence inline **132 项**、Application **55 项**实际通过；运行日志 **7 条记录、3 个完成操作**通过。
- artifact `11148207904`，14,019,073 字节；SHA-256 `19a5d423e18bb2e53a529416207194e8e829e26b4673e21425fd51011d34df2b`。
- 18 项 broad PostgreSQL 仍 ignored，仅编译通过；其他数据库 contracts ignored 同样不算实际 PASS。历史四项失败、账本及真实 XLSX/Full 仍最终封包新库待验。

## 下一项与回退

用户授权收尾并启动 R7-13 Monthly Workbook；13 必须取得自己的精确 Windows CI。进入/回退基线 `310c46bbedb731e20ffaca2c67df0eefc3ce6e3a`；受控回退同步职责、调用、门禁/清单与 PG 指纹，不恢复双实现或单独放宽门禁。公共 API/DTO、算法/参数/保护资产、生产依赖/锁文件及 0001–0046 迁移保持冻结。
