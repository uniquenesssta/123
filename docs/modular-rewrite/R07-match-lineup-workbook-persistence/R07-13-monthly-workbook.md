# R7-13 Monthly Workbook：节点完成记录

状态：`DONE`（月度只读聚合/缺口唯一职责与 Windows Automated 完成；真实 PG/XLSX/Windows Full 最终封包新库待验）。

## 实际范围

球队聚合、两类缺口与共享映射从旧 monthly_workbooks.rs 迁入唯一 workbooks/monthly_team/{read,gaps}、monthly_player/gaps 和 monthly_gaps；R7-11 的共享球员导出从 player_catalog/export 迁入 monthly_player/read。旧位置、注册全部删除，无转发壳；根 player_catalog.rs 引用目录保持。五个原函数体、四个公开签名及 18 查询/字段映射不变，默认 club/0.5/null 时间、历史与未来记录、排序、元数据、current_date 包含边界、90/120 天观察和非负 stale_days 保持。Port 显式分派，导入继续复用 R7-11/12。

原 PG 月度夹具扩展聚合、空资料/清空值/元数据、读取无事实/账本/审计写入、日期/观察边界及多球队三段履历。七个原门禁切换实际 owner，月度 verifier 增唯一职责、只读/映射/原 SQL 检查。83 项现有源码、架构与 Rustfmt 通过，六项破坏探针拒绝并恢复；没有新增镜像单测、测试目标、runner、workflow、数据库设施、依赖或迁移。

## 精确 Windows 证据

- 提交 `78138dba30ead1dbc4f4e5d2c6595e5775edfa0c`；树 `63f9d9a67217fda838ab82764281d166c2fdbad9`。
- [Public Platform CI run 36854202029](https://github.com/uniquenesssta/123/actions/runs/36854202029)，Windows job `110342691574` 全步骤 SUCCESS；2026-10-01 19:44:10（北京时间）更新为 completed/success，本轮核实精确 SHA 和日志。
- 架构、前端契约/类型/17 个截图视口/生产构建、Windows fmt/Clippy/workspace tests、release/MSI/NSIS 与启动通过；Persistence **132 项**、Application **55 项**实际通过；运行日志 **7 条记录、3 个完成操作**通过。
- artifact `11158674438`，14,017,971 字节；SHA-256 `3b6b481c07e7626c81938480d81db1298d03797bce3785d593b3c27c745aa295`。
- 18 broad PG 及其他数据库 contracts 仍 ignored，只编译不算实际 PASS；历史四项、账本、有效 XLSX/Full 最终封包新库待验。

## 下一项与回退

用户授权收尾并启动 R7-14 Match Lineup Workbook；14 必须取得自己的精确 Windows CI，15 保持 BLOCKED。13 进入/回退基线 `fe2f7ebcbf50e4bb40016d649a95b2c295aeab55`。受控回退同步 owner、调用、门禁/清单与 PG 指纹，不恢复双实现或单独放宽门禁；正常 API/DTO、算法/参数/资产、依赖/锁文件及 0001–0046 迁移冻结。
