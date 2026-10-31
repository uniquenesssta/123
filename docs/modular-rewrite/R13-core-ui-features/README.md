# R13 基础数据与编排业务模块化：执行索引

阶段状态：**IN_PROGRESS**。分支 `rewrite/r13-core-ui-features`，起点 `f0770665`。01、02已收尾；03及并入的04已按修复提交b5b23c46精确CI/下载证据/双bundle/startup共同DONE。用户已授权05，当前IN_PROGRESS；06及其后未启动。R12保持结束。

依据：[阶段任务书](../../football-model-platform-modular-rewrite-19-docs/13-R13-core-ui-features.md)、[执行约定](../remaining-execution-contract.md)、[前期遗留项基线](../prior-stage-baseline-20261030.md)。

| 主交付 | 当前状态 | 记录与原ID覆盖 |
| --- | --- | --- |
| R13-01 首页/架构只读接入 | DONE：9cdd1f9b精确CI/证据摘要/MSI/NSIS/startup核验通过 | [实施记录](R13-01-read-only-entry-pilot.md)，原R13-01/R13-10合并验收 |
| R13-02 数据库连接/危险操作 | DONE | [实施记录](R13-02-database-connection-and-reset.md)：96步前端、15行为/21浏览器/原生startup及d167e948精确CI/产物已核验 |
| R13-03 赛事层级/规则/路由 | DONE | [实施与收尾](R13-03-competition-rules-and-routing.md)：b5b23c46精确run41116421113、artifact13009230984、双bundle/startup已核验 |
| R13-04 规则包/绑定/路由 | 并入03 / DONE | 独立义务映射及同一精确CI验收完成，详03记录 |
| R13-05 球队/档案/资料包 | IN_PROGRESS | 用户已授权；基线8ee45f15，承接BA-05/07，盘点和实施中 |
| R13-06 球员/履历/观察 | BLOCKED | 未启动 |
| R13-07 预设/双方阵容 | BLOCKED | 原07/08，BA-03删除保护风险须按本批条件处理 |
| R13-09 工作簿 | BLOCKED | 未启动；真实文件/账本按BA-04/07承接 |

R13-01已将两个实际View迁至各自Feature公开入口，首页投影独立，Session/Router/Workspace唯一owner保持；原HTML精确比较一致。96步前端、7项新生产模块行为、18项Windows浏览器生产运行及Windows原生startup通过。Rust源码/依赖不变，完整Rust测试复用已有同树证据，本批仅实际重建桌面开发版。

原生运行使用真实配置目录及隔离日志，startup不代替Full。真实PG/历史四项/账本与并发回滚/有效XLSX/API凭据/私有固定回归仍由[前期BA账本](../prior-stage-baseline-20261030.md)逐项承接到R17-06；本阶段不冒称这些通过。自身CI只核精确提交一次，不轮询。

2026-10-31收尾：run41085494919 / job124165444411全SUCCESS，artifact12994712478下载摘要一致；两类安装包与包内startup 7记录/3完成/0错误通过。精确来源、哈希与未验边界见01记录。R13整体仍IN_PROGRESS。

R13-02：连接/断开串行与去重、Session身份绑定重置、未知结果核查、临时凭据释放已接原命令和既有确认组件。配置或摘要未核实禁止重置，失败不清Workspace、不自动重试。39个状态owner，后端/Schema/模型/依赖不变；专用新库真实重置仍BA-14/R17-06待验。

02正式收尾：run41098940648、artifact13001455633及双bundle/startup已核验；真实新库重置等原待验仍按BA-14/R17-06承接。

R13-03：赛事目录/层级与规则/绑定/路由分别归属两个实际Feature，保留原rules页面、Session/Workspace/Modal唯一owner。修复父子选择、乱序文件读取、失败草稿及隐藏预览；目录说明与预览不授予执行许可。41个状态owner，后端/Schema/依赖不变。真实PG/Full按BA-14/R17-06承接；05尚未启动。

2026-10-31补修：03原精确CI失败在安装依赖前加载TypeScript；三组共31项行为现归已安装依赖的原前端组合根门禁，无依赖完整架构副本修复前失败/修复后通过。04按既有合并卡逐项核对，仍随03共同VERIFYING；不得把旧CI记为成功。

修复提交b5b23c46已推送；96步前端复验全部通过，31项行为未减少。新run41116421113精确head匹配、单次查询运行中，包与startup尚待验收；03/04不得标DONE。

2026-10-31共同收尾：03/04精确修复提交的run/job全部SUCCESS，证据摘要一致，MSI/NSIS与startup 7记录/3操作/0错误通过。05已授权开始；此前VERIFYING段落为历史快照，当前状态以表格和收尾记录为准。
