# UI 接入审计处置与剩余任务基线（2026-10-31）

## 适用结论与授权边界

用户已授权将本轮审计落实到后续任务书。本文件是17页接入差异和原40个UI业务节点的唯一处置表；与[共享执行约定](remaining-execution-contract.md)、R13～R18执行卡共同生效，取代10/30计划中“保持当前外观即可”及“页面设计尚未开始”的过时判断。R12保持结束，R13-01～06及原10随01的业务交付保持DONE。**R13-07当前PAUSED；本次只改规划，不恢复开发或宣称差异已修。**

代码核对基线：`6e1361210ad4b9fd9f6dd969fad112397e84f6ff`，分支 `rewrite/r13-core-ui-features`。原任务书追溯提交 `4420fd48`；设计记录 `ui-design-system@40109ceec8372391a7680e7e86c1265fda06ab3d`；实时画布为10/31读取的 [Figma文件](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu)，Screens页3:5。旧Git设计快照不是实时像素；实施时再次核对最新**已批准**记录，WIP不能覆盖批准版。

审计取得17个主Frame元数据/截图及17个生产View+Shell+CSS的Windows Chromium隔离渲染；17路由均存在并渲染成功。本次输入为已有fixture，未挂载完整控制器/Workspace恢复/控件增强，未执行真实数据库/API/Tauri写入。复盘、运行记录、日志等数据状态不完全一致；不把空态与有数据态的差别当缺功能。本审计不授予任何整页全状态1:1或真实链路PASS。

- **已接入并验证（限定范围）**：[10/28批准接入](R12-frontend-foundation-ui-system/approved-design-integration-20261028.md)及[R13已交付记录](R13-core-ui-features/README.md)中的受测能力；沿原范围复用，不重做。
- **已接入但待验**：下表7页已有生产实现，但尚缺同状态设计/细节/完整挂载证据。
- **确实未接入（局部）**：下表10页存在已定位布局或内容差异；不是整页未开发，不能重造业务。
- 旧“17视口截图”来自tests/ui夹具；批准接入脚本8场景对应5路由。二者都不等于17生产页逐状态对Figma验收。原HTML前后相等只能证明迁移保留旧呈现。

## 17页差异、责任与关闭依据

状态均为本轮审计结果，**全部差异/待验项当前OPEN**。后续在责任批次实施记录填写实际Frame、受测提交、状态、结果及证据，再回写本表关闭引用；不整组打勾。下表源文件路径相对仓库根。

| 编号/页面 | Figma主Frame | 生产入口 | 当前判定 | 剩余事项 | 接入验收owner |
|---|---|---|---|---|---|
| S01 首页 | [453:2](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=453-2) | `src/features/dashboard/index.ts` | 已接入但待验 | 主视觉渐变、2+3快捷卡片和四色统计已接入；当前首行卡片高度、文字尺度/链接色与实时画布不同，需同数据逐项核准。 | R13-UI01 |
| S02 比赛与阵容 | [466:4436](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=466-4436) | `src/pages/lineups.ts` | 确实未接入（局部） | Matches默认页仍为多层标题/嵌套分区；设计的紧凑比赛详情表单未整体落地。双方阵容此前已专项接入，不能把本页其他状态一并认作通过。 | R13-07 |
| S03 赛事推演 | [487:22278](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=487-22278) | `src/pages/prediction.ts` | 确实未接入（局部） | 正式推演仍为大块嵌套设置/说明卡；设计把Provider警告、设置、准入、输入审计及底部动作明确分区。当前默认截图首屏主要被设置占据。 | R14-01（正式/影子）与 R14-06（P4） |
| S04 赛后复盘 | [505:33973](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=505-33973) | `src/pages/review.ts` | 已接入但待验 | 九步流程与工作区已实现；本次代码为空比赛态，Figma为Loaded，不能用内容数量或锁定步骤差异判断缺功能。须补相同Loaded/确认/结算状态。 | R14-08（复盘）与 R14-12（结算） |
| S05 推演记录 | [516:48546](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=516-48546) | `src/pages/runs.ts` | 已接入但待验 | 生产列表/详情/隐藏入口存在；代码截图空列表，Figma六条记录。数据列、审计色块、详情和隐藏后的血缘需匹配同状态验收。 | R14-07 |
| S06 球队 | [527:54630](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=527-54630) | `src/features/teams/index.ts` | 已接入但待验 | 固定筛选栏、两列卡片、底部分页已落地；当前卡片使用完整度条和右侧档案动作，Figma使用文字完整度及下方完整档案按钮。应记录差异，不以HTML前后等价声称设计一致。 | R13-UI01 |
| S07 球员 | [550:69407](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=550-69407) | `src/features/players/index.ts` | 已接入但待验 | 筛选+名单+分页已接入；当前筛选分组卡片、表格头部及档案按钮与设计不同。18行与6行、姓名/能力值差别属于夹具数据，不算缺陷。 | R13-UI01 |
| S08 阵容预设 | [570:86212](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=570-86212) | `src/pages/lineupPresets.ts` | 确实未接入（局部） | 未选择态为整块居中空态；Figma为顶部具名面板+紧凑空态，左栏宽度与工具栏套用规则/球队中心入口也不同。 | R13-07 |
| S09 Excel工作包 | [579:102496](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=579-102496) | `src/pages/workbooks.ts` | 确实未接入（局部） | Figma左侧竖排分类及底部确认区，当前左栏横排分类、第三列内确认；左栏比例和三步面板组成不同。 | R13-09 |
| S10 赛事与规则 | [589:112453](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=589-112453) | `src/features/rules/index.ts` | 已接入但待验 | 固定体系/地区栏和具体赛事两列网格已落地；标题动作/搜索工具栏/底部路由说明有差异。同数据滚动、结构/路由/规则包状态需逐项核验。 | R13-UI01 |
| S11 应用内验收 | [597:123771](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=597-123771) | `src/pages/release.ts` | 确实未接入（局部） | 生产有六分区和验收表单；Figma顶部四张报告指标未按同样布局出现。当前仍使用“固定P4 fixture”文案，设计已强调公开外壳/外部运行时，不应误导能力。 | R14-17 |
| S12 分析与历史 | [608:135680](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=608-135680) | `src/pages/analytics.ts` | 确实未接入（局部） | 当前为纵向折叠步骤与步骤内指标；Figma为顶部四指标、横向四步骤及独立内容区。结算数差异是输入数据，布局组成差异可确认。 | R14-13（总览/任务/建议）与 R14-16（候选） |
| S13 AI问答 | [628:152166](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=628-152166) | `src/pages/apiWorkspace.ts` | 确实未接入（局部） | 设置/会话/输入已存在；当前设置说明区较高，隔离渲染首屏未露出composer，Figma底部输入常驻。需在完整挂载中验证滚动后再修，不能判定发送功能缺失。 | R15-01 |
| S14 兼容API配置 | [647:166715](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=647-166715) | `src/pages/openai.ts` | 确实未接入（局部） | Figma API示例为次级弹窗入口，生产为表单上方11行内联textarea；主表单/底部动作位置及左栏宽度不同。解析能力已经存在且有过期响应防护。 | R15-07 |
| S15 数据库 | [673:178191](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=673-178191) | `src/features/database/index.ts` | 确实未接入（局部） | Figma运行概览为连接身份、四指标和业务边界提示；生产为三统计卡、折叠连接/统计和同屏危险操作。02完成真实连接/危险操作owner不等于此布局已匹配。 | R13-UI01 |
| S16 问题日志 | [683:186758](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=683-186758) | `src/pages/logs.ts` | 已接入但待验 | 聚合、导出、清空入口已在生产；本次空日志与Figma有日志态不一致，不能据此认定日志卡未实现。需相同严重/警告/重复/详情状态验证。 | R15-12 |
| S17 系统架构 | [693:194738](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=693-194738) | `src/features/architecture/architectureView.ts` | 确实未接入（局部） | 生产有四层职责和历史/路由说明；缺Figma明确的“架构声明不等于运行能力”说明区，历史/模型运行边界仍过于笼统。分区点击行为本次未实测。 | R13-UI01 |

同一路由多个owner按状态分工：S03正式/影子归14-01、P4研究/冻结归14-06；S04包/事实/复盘归14-08、正式结算归14-12；S12总览/任务/建议归14-13、候选/影子/晋升/回滚归14-16。共用页面骨架由先行批次维护，后批只补自己的状态，不另建同名整页。

## 原40个节点的核销和保留

本表核销的是重复工程，不取消功能或未完成验收。原ID的业务owner保持；已完成页面的剩余设计义务由R13-UI01承接，原DONE不回退。其余27个R16～R18原ID仍见[67项映射](task-plan-review-20261030.md#67项完整去向)。

| 原ID/能力 | 页面 | 业务owner | 本次处置 |
|---|---|---|---|
| R13-01 Dashboard | dashboard | R13-01 | 已交付只读模块；保留首页视觉细节差异，不重建页面。 |
| R13-02 Database | database | R13-02 | 02业务交付保持DONE；最新概览/危险区布局未完整匹配，真实新库重置待验。 |
| R13-03 Competitions | rules | R13-03 | 赛事目录/层级owner及网格已经交付，取消重复建设。 |
| R13-04 Rules | rules | R13-03 | 与03共同DONE；原04独立义务保留证据，不再单起实现。 |
| R13-05 Teams | teams | R13-05 | 05业务交付保持DONE；资料包复用现owner，同状态视觉/真实写入分开验。 |
| R13-06 Players | players | R13-06 | 06业务交付保持DONE；不再重写列表/档案/履历owner。 |
| R13-07 Lineup Presets | lineup_presets | R13-07 | 已有编辑/归档/套用能力；补确认时对象身份、复制隔离和局部布局。 |
| R13-08 Lineups | lineups | R13-07 | 已有比赛/双边/历史/链路能力；核对await前后快照和受保护比赛删除。 |
| R13-09 Workbooks | workbooks | R13-09 | 三类工作簿入口已有；复用Teams/Players流程，补Match共享链路/恢复/真实文件证据。 |
| R13-10 Architecture Page | architecture | R13-01 | 原10随01已完成只读模块归属；原表“合并验收项非DONE”已订正为业务DONE；内容差异另列。 |
| R14-01 Prediction Context | prediction | R14-01 | 已有比赛/模型/窗口选择；验证跨输入切换的状态失效。 |
| R14-02 Readiness 与 Fingerprint | prediction | R14-01 | 已有检查与审计显示；验证许可绑定当前身份/窗口/指纹，不能以Provider在线代替。 |
| R14-03 Formal Run | prediction | R14-01 | 已有真实正式执行调用；验证重复点击、部分提交及结果未知恢复，不重建按钮/API。 |
| R14-04 Shadow Run | prediction | R14-01 | 已有影子执行；核验与正式写入/状态隔离。 |
| R14-05 Prediction Result | prediction | R14-01 | 已有结果呈现；同状态概率/未知/失败/历史血缘待验，不造模型输出。 |
| R14-06 P4 Research UI | prediction | R14-06 | 已有P4工作台/任务/冻结调用；核验研究证据与冻结身份，私有运行时仍需授权外部环境。 |
| R14-07 Runs | runs | R14-07 | 已有列表/详情/隐藏；核验隐藏不删除历史与未记录值。 |
| R14-08 Review Selection 与 Workflow | review | R14-08 | 已有九步流程；核验当前比赛与后端allowed_actions，补同Loaded状态比对。 |
| R14-09 Review Facts | review | R14-08 | 已有事实录入/保存；验证真实身份、版本、失败草稿。 |
| R14-10 Review Package | review | R14-08 | 已有导出/预检/确认；校验package/SHA/比赛与一次提交归属。 |
| R14-11 Review Results | review | R14-08 | 已有结果生成/展示；验证失败或部分完成可追溯，不以查看结果授予结算许可。 |
| R14-12 Postmatch | review | R14-12 | 已有结算与H监控入口；核验正式身份/幂等/精确分区，不能把复盘完成等同结算。 |
| R14-13 Analytics Overview | analytics | R14-13 | 已有总览与查询；当前纵向折叠结构与设计不同，业务和布局分开记录。 |
| R14-14 Analytics Jobs | analytics | R14-13 | 已有任务列表/取消/重试；核验终态与切页返回，非重新创建任务系统。 |
| R14-15 Analysis Packages 与 Suggestions | analytics | R14-13 | 已有导出/回包/人工建议；核验待审身份、失败恢复与提交结果。 |
| R14-16 Ability 与 Parameter Candidates | analytics | R14-16 | 已有人工决定/影子/晋升/回滚入口；核验分区/版本/许可，公开Provider不可伪装可用。 |
| R14-17 Release | release | R14-17 | 已有验收请求/报告/历史；修正未匹配指标与误导性固定fixture文案，保留真实报告验收。 |
| R15-01 AI Session Directory | api_workspace | R15-01 | 已有会话列表/选择/归档；复现乱序加载并保证每次await后校验当前会话。 |
| R15-02 Conversation | api_workspace | R15-01 | 已有真实消息投影；发送后刷新列表期间切B不能回填A。 |
| R15-03 Composer | api_workspace | R15-01 | 输入与草稿已存在；完整挂载验证常驻输入区及会话草稿隔离。 |
| R15-04 Pending Request | api_workspace | R15-01 | 已有pending消息/请求ID；核验会话关联与终态唯一。 |
| R15-05 Request Cancellation | api_workspace | R15-01 | 已有取消调用；验证只取消目标请求，已提交写入不能伪称撤销。 |
| R15-06 Context Picker | api_workspace | R15-01 | 已有比赛/球队/球员只读上下文；核验勾选/切换身份与发送快照。 |
| R15-07 OpenAI Profile Directory | openai | R15-07 | 目录/新建/选择已存在，S14设计已完成有限验收，不再以无设计阻塞。 |
| R15-08 OpenAI Profile Editor | openai | R15-07 | 已有编辑/保存；同配置草稿、切换和测试结果身份待验。 |
| R15-09 API Example Parser | openai | R15-07 | 解析能力与过期响应校验已存在，取消从零开发/重复竞态修复；仍需设计弹窗呈现与相关状态验收。 |
| R15-10 Credential Actions | openai | R15-07 | 已有密钥保存/删除/激活API；核验元数据与系统凭据部分失败，真实凭据待验。 |
| R15-11 Profile Test | openai | R15-07 | 已有连接测试；核验测试回填到发起配置，真实API结果待验。 |
| R15-12 Logs | logs | R15-12 | 已有聚合/导出/清空；补同状态设计比对、损坏/读取失败及脱敏验证。 |
| R15-13 Diagnostics Bridge | logs | R15-12 | 已有诊断关联链；核验请求ID/来源/脱敏，不新建第二套日志系统。 |

立即核销：再次搭17路由、Shell/字体/基础控件/Workspace；固定十层目录和空转发层；重新创建S14～S17设计；重复开发已有解析器、API和导入引擎。当前main解析响应已有请求序号、DOM对象和输入文本三重校验，R15-07只维护并验证，不再按旧描述重复修复。

## 实际必要的业务工作

| 证据/风险 | 必要任务与关闭条件 | 责任 |
|---|---|---|
| main:createPairedLineups在await链路后才从可变builder取双方draft | 校验、确认、提交绑定同一比赛/窗口/输入快照；延迟期间切A→B不串线；重复提交与未知结果单列 | R13-07 |
| main:loadPairedLineupSide主要核对side序号/team；比赛上下文仍需核验 | 相同球队跨比赛时也隔离；两侧独立且切比赛废弃旧响应 | R13-07 |
| matches/catalog/delete.rs计数未含model.runs而后置空match_id，BA-03记录0041冲突 | 明确保护拒绝，不放宽历史保护；无引用/model run/P4或结算三类行为，真实PG回滚按BA关闭 | R13-07 |
| main:stillViewingOrigin在await会话列表前计算 | A发送→列表刷新中切B不回填A；乱序会话选择不覆盖；B草稿保留 | R15-01 |
| 05/06已有团队/球员导入owner，Match与共享入口未全部归属 | 共享入口调用既有owner；预检票据/文件/目标身份绑定、重复和未知提交恢复 | R13-07/09 |
| BA-08/12记录跨事务/部分完成边界；前端已有推演/复盘/分析调用 | 故障可查回；指纹/许可对应当前对象；候选人工门禁与正式写入分离 | R14各责任批 |
| 凭据、真实请求、PG/XLSX/Full、私有固定回归仍未完成最终实证 | 按既有BA账本逐项执行；真实环境缺失不以stub/截图/startup替代 | R15、R17-06、R18 |

以上源码风险不冒称本次动态复现。执行时先确认当前是否仍存在，已有正确防护直接复用；行为或契约证据支持的缺陷按职责完整修复，不以“保持旧行为”保留错误。

## 顺序、数量与停止条件

- 原24批中的5个R13业务批次已DONE；本轮增加**1个补差批次R13-UI01**，专门承接既有义务，不新增产品功能。规划共25个交付批次、原67个ID不变；待完成20批不等于20次重写或固定20次提交。
- R13后续默认顺序：**UI01 → 07（覆盖原07/08）→ 09**。UI01完成同状态/完整挂载核验与6页必要补差；07继续保留暂停，恢复须有用户指令。新规划不自动授权这些任务实施。
- R14八批、R15三批保持真实业务依赖；局部设计补差随对应批完成，不全部推给R16。R16只有查到具体残留才改代码；无残留可以用证据核销。
- R17-01核验17页和原40项的证据覆盖；R17-06补既定真实外部闭环。R18只发布来源准确且必需账本关闭的候选。
- 本域可见差异、必要行为测试失败、身份串线、必要功能失联、设计状态未匹配，不属于可自动延期到最终新库的项目。阻塞本域交付；非本域事项登记owner，不临时无限扩展。
- 需要改变已批准设计或发现设计缺少必要业务能力时，先按Figma流程补设计并核准；不能删除真实功能来凑截图。本轮只写任务，无设计/源码修改。

## 交付证据与此次规划验证

本机审计产物保留于Codex可视化工作目录的 `ui-integration-audit-20261031`：`audit-report.md`、`comparison.html`、`audit-matrix.json`、17对截图、`production-report.json`及SHA256清单。不把30MB截图/内嵌字体页面或浏览器临时profile提交到仓库；本文件保留可脱离本机路径使用的17页/40项结论、Frame及源码定位。

本次文档交付只核对链接、引用、17/40/67覆盖、25批数量、状态/owner/依赖一致性及Git diff；不产生新的软件构建、自动行为、原生运行或Full结果。R12文档/历史交付记录、生产代码、测试和CI配置均保持原样。

文档校验结果：12份Markdown；25个唯一交付卡；原67项业务映射与修改前完全一致；40个UI原ID、17页全部有归属；19张剩余执行卡补充具体交付边界，原79个V场景保留并增加UI01的5个场景；271处本地链接及17个生产入口有效，Git差异空白检查通过。没有修改R12记录、源码、测试或CI配置。
