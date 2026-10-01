# R7-10 Spreadsheet Batch Ledger：节点完成记录

状态：`DONE`（账本唯一职责、暂存计数与 Windows Automated 完成；真实 PG/XLSX/Windows Full 仍最终封包新库待验）。

## 实际范围

`adapters/workbooks/batch_ledger/` 唯一持有球员/比赛的批次与行 SQL、严格 codec、预览读取、行身份、依赖顺序及计数/审计。原两类工作簿单向调用账本并持有唯一业务事务；非法状态/未解决冲突拒绝，失败整批回滚，原成功重试差异保持。人工 skip/候选解决在父锁事务内同步 pending 的 skipped_count/error_count，首次 metadata.preview_counts 保留快照，成功账本和审计使用同一结果；ended_previous_count 保持。Monthly/Team 剩余迁移随 R7-12/13 执行。

增加 6 个生产文件内联状态/类型/codec/计数测试，原球员月度和比赛导入用例扩充状态、冲突、计数、跨类型读取及回滚/恢复断言。既有门禁增强，6 项临时破坏探针均拒绝并恢复；无新测试目标、runner、工作流、数据库或迁移。

## 精确 Windows 证据

- 提交 `cc0d34d2f4bce0a9a656a0624c5e59c602aa7d86`；树 `486d0afc25f37c5ffea428d1ae990e2caa740f39`。
- [Public Platform CI run 36815568401](https://github.com/uniquenesssta/123/actions/runs/36815568401)，Windows job `110219751121` 全步骤 SUCCESS；2026-10-01 12:59:51（北京时间）更新为 completed/success，本轮核实。
- 架构、前端契约/类型/17 个截图视口/生产构建、Windows fmt/Clippy/workspace tests、release/MSI/NSIS 打包与启动通过。Persistence inline **121 项**、Application **53 项**实际通过；运行日志 **7 条记录、3 个完成操作**通过。
- artifact `11141479972`，14,024,495 字节；SHA-256 `ef4a1166fc4e302944381a02c714ee0a22d0b51ac3e752be8b0f3dbb8382c211`。
- 18 项 broad PostgreSQL 仍 ignored，仅编译通过；其他数据库 contracts ignored 同样不算实际 PASS。历史四项失败、账本及真实 XLSX/Full 仍最终封包新库待验。

## 下一项与回退

用户授权收尾并启动 R7-11 Player Workbook；11 必须取得自己的精确 Windows CI。进入/回退基线 `3525c04cbb0f75d79cdbc39a4ef64f3698f5eb92`；受控回退同步账本职责、工作簿调用、暂存计数、门禁/清单和测试指纹，不恢复双实现或独立放宽门禁。公共 API/DTO、算法/参数/保护资产、生产依赖/锁文件及 0001–0046 迁移保持冻结。
