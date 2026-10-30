# R13 基础数据与编排业务模块化：执行索引

阶段状态：**IN_PROGRESS**。分支 `rewrite/r13-core-ui-features`，起点 `f0770665`。用户本轮授权R13-01，覆盖原R13-10；其他批次尚未启动。R12按用户确认保持结束。

依据：[阶段任务书](../../football-model-platform-modular-rewrite-19-docs/13-R13-core-ui-features.md)、[执行约定](../remaining-execution-contract.md)、[前期遗留项基线](../prior-stage-baseline-20261030.md)。

| 主交付 | 当前状态 | 记录与原ID覆盖 |
| --- | --- | --- |
| R13-01 首页/架构只读接入 | VERIFYING：本地门禁、Windows浏览器和原生startup通过；自身CI/安装包待验 | [实施记录](R13-01-read-only-entry-pilot.md)，原R13-01/R13-10合并验收 |
| R13-02 数据库连接/危险操作 | BLOCKED | 未启动，等待01出口与用户指令 |
| R13-03 赛事层级/规则/路由 | BLOCKED | 原03/04，未启动 |
| R13-05 球队/档案/资料包 | BLOCKED | 未启动 |
| R13-06 球员/履历/观察 | BLOCKED | 未启动 |
| R13-07 预设/双方阵容 | BLOCKED | 原07/08，BA-03删除保护风险须按本批条件处理 |
| R13-09 工作簿 | BLOCKED | 未启动；真实文件/账本按BA-04/07承接 |

R13-01已将两个实际View迁至各自Feature公开入口，首页投影独立，Session/Router/Workspace唯一owner保持；原HTML精确比较一致。96步前端、7项新生产模块行为、18项Windows浏览器生产运行及Windows原生startup通过。Rust源码/依赖不变，完整Rust测试复用已有同树证据，本批仅实际重建桌面开发版。

原生运行使用真实配置目录及隔离日志，startup不代替Full。真实PG/历史四项/账本与并发回滚/有效XLSX/API凭据/私有固定回归仍由[前期BA账本](../prior-stage-baseline-20261030.md)逐项承接到R17-06；本阶段不冒称这些通过。自身CI只核精确提交一次，不轮询。
