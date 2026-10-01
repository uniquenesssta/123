# R7-14 Match Lineup Workbook：节点完成记录

状态：`DONE`（比赛阵容工作簿唯一职责与 Windows Automated 完成；真实 PG/XLSX/Windows Full 最终封包新库待验）。

## 实际范围

48 个原函数从 match_exchange.rs 迁入唯一 workbooks/match_lineup/{read,context,preview,conflict,commit,write,identity,validation,values}，mod 仅注册及保存原内部结果；旧根实现/注册删除。全部函数体、五个公开签名、22 条直接 SQL 及动态候选查询/字段映射保持；Port 六入口显式分派，原 batch_ledger/codec 继续复用 R7-10。

原预检只暂存、两阶段冲突父/行锁与候选复核、唯一整批提交事务、共同比赛锁、旧版本结束、捕获日角色继承、阵容门禁/返回/账本/审计计数、末行回滚与重复提交拒绝保持。成对创建与合法单侧工作簿替换边界明确；模型 cutoff/actual 隔离继续由 R7-08 owner 执行。导出/AI 包保留活动历史、引用时间、去重、角色和原 cutoff。

原 PG pair/workbook 夹具补读取无副作用、投影、空/未知比赛、预检只暂存、行身份及错误主客身份阻断；所有既有回滚/并发/计数断言保留。五个原门禁跟随实际职责，83 项源码/架构/Rustfmt 通过，六项破坏探针拒绝并恢复；无新测试目标、runner、workflow、数据库设施、依赖或迁移。

## 精确 Windows 证据

- 提交 `bb0012085d92cb63742ff5b5571a8003984db709`；树 `940e07450acae0e99e4f625cbc19ddabbd34d83c`。
- [Public Platform CI run 36860224760](https://github.com/uniquenesssta/123/actions/runs/36860224760)，Windows job `110362332343` 全步骤 SUCCESS；2026-10-01 20:41:27（北京时间）更新为 completed/success，本轮核实精确 SHA 和日志。
- 架构、前端契约/类型/17 个截图视口/生产构建、Windows fmt/Clippy/workspace tests、release/MSI/NSIS 与启动通过；Persistence **132 项**、Application **55 项**实际通过；运行日志 **7 条记录、3 个完成操作**通过。
- artifact `11161982535`，14,017,440 字节；SHA-256 `31f7042c8981c0f5d34fd77630e78ff206b188be36579f6b9656d2f25652caf1`。
- 18 broad PG 及其他数据库 contracts 仍 ignored，仅编译不算 PG 实跑；历史四项、账本、有效 XLSX/Full 最终封包新库待验。

## 下一项与回退

用户授权收尾并启动 R7-15 Row 与 Subrecord Identity；15 须取得自己的精确 Windows CI 后才能完成 R7 节点收口。14 进入/回退基线 `78138dba30ead1dbc4f4e5d2c6595e5775edfa0c`；受控回退同步 owner、调用、门禁/清单与 PG 指纹，不恢复双实现或单独放宽门禁。正常 API/DTO、算法/参数/资产、依赖/锁文件及 0001–0046 迁移保持冻结。
