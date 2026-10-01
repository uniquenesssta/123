# R7-09 Team Lineup Presets：节点完成记录

状态：`DONE`（唯一职责、只读预检与 Windows Automated 完成；真实 PG/XLSX/Windows Full 仍最终封包新库待验）。

## 实际范围

旧 `team_lineup_presets.rs` 删除，保存/复制/归档/删除与共同事务审计、结构/成员校验、读取 SQL/映射、只读入口/纯评估分别由 `adapters/lineups/presets/` 的 write/validation/read/preview 持有。9 个原函数体规范化对比保持；公共 DTO/API、错误、默认/版本、排序、位置和角色继承/来源、伤停提示保持。LineupPresetPort 六入口显式分派唯一实现。既有前端复检 can_apply 与所选主客侧 team_id，只替换这一侧草稿，正式保存仍经显式双方阵容提交。

原两个内联结构测试保留，增加 8 个内联边界/映射/预览行为测试。原 PostgreSQL 双方阵容用例补重复预览不改变预设/阵容/球员/审计、跨队保存拒绝、成员过期/归档拒绝、复制与删除级联断言。现有 Lineups/E2/Role 验证器、清单和既有测试指纹同步，5 项临时破坏探针被原门禁拒绝并恢复；未增加测试目标、runner、工作流或数据库设施。

## 精确 Windows 证据

- 提交 `3525c04cbb0f75d79cdbc39a4ef64f3698f5eb92`；树 `513cb4e838c937d04c97b7bee15130495fe201cf`。
- [Public Platform CI run 36808609584](https://github.com/uniquenesssta/123/actions/runs/36808609584)，Windows job `110198409740` 全步骤 SUCCESS；2026-10-01 11:29:41（北京时间）更新为 completed/success，本轮核实。
- 架构、前端契约/类型/17 个截图视口/生产构建、Windows fmt/Clippy/workspace tests、release/MSI/NSIS 打包与启动验收通过。Persistence inline **115 项**、Application **53 项**实际通过；启动日志 **7 条记录、3 个完成操作**通过。
- artifact `11139139299`，14,017,252 字节；SHA-256 `efc5d44918146f8142e4e04fa99b6b69a5a763c08b2fa747e6dc9ebec042d75e`。
- 18 项 broad PostgreSQL 仍 ignored，仅编译通过，其他数据库 contracts 的 ignored 不算实跑 PASS；历史四项失败及账本待验未实跑关闭。

## 下一项与回退

用户授权收尾并启动 R7-10 Spreadsheet Batch Ledger；10 必须取得自己的精确 Windows CI。真实数据库、有效 XLSX 和 Windows Full 保持最终封包新库流程。进入/回退基线 `3c8f7f0230207f7352b7f806b8a1e92c4ebc81e5`；受控回退同步职责、Port 分派、门禁/清单和测试指纹，不恢复双实现或独立放宽检查。API/DTO、算法/参数/保护资产、生产依赖/锁文件与 0001–0046 迁移保持冻结。
