# R13-07 预设与双方阵容闭环

状态：**DONE：精确代码CI、双bundle及包内startup已核验收尾**。覆盖原R13-07/R13-08；没有单独未实施的08。分支 `rewrite/r13-core-ui-features`，实施基线 `f59d12f6a8270e65a8d2bd166d6c1f206c09b962`，开始时工作区干净。用户明确“开始R13-07”，恢复本批；未启动UI01/09，不重开R12或已DONE的01～06。

## 交付与保留映射

| 旧入口/状态/调用 | 当前唯一owner与消费者 | 保留边界 |
|---|---|---|
| main中双方比赛选择/reset/capture、两侧名单、加/删/设置球员 | features/lineups/pairedBuilder.ts、pairedActions.ts；index公开actions | side独立RequestSequence；队/比赛/页面/生命周期校验；相同球队跨比赛也隔离 |
| 预设成员、角色/阵型映射、自动分配、结构校验 | presetPositions.ts、presetEditor.ts | 11首发、阵型槽位、队长、角色继承/本场覆盖、概率及默认方案均保留 |
| 预设目录/管理/编辑/复制/归档/永久删除/从当前阵容保存 | presetActions.ts；Teams只通过既有公开catalog facade供数据 | 不另建Teams store；确认绑定球队/页面；原save/list/duplicate/archive/delete命令不变 |
| 预览与应用副本 | presetApplication.ts | side/preset/version/比赛/整个草稿/页面绑定；确认时重新预检；只替换目标侧，不改原预设 |
| 双边提交与链路检查 | pairedActions.ts | 首次await前冻结双方payload及窗口；一次paired API，未拆成两个单边写入；保存与模型准入分开 |
| 比赛层级/赛季/球队过滤、新建修改、比赛删除、阵容历史删除 | matchEditor.ts | 原字段和真实API全部保留；保存/删除及回读不覆盖切换后的比赛 |
| 复盘等合法单侧入口 | legacyLineup.ts | 11首发及历史时间校验保留；单侧名单、链路、写入回读按当前对象隔离 |
| matchSpreadsheetPreview、XLSX/AI包预检/裁决/提交/账本读取 | MatchImports（matchImports.ts） | 唯一批次/hash/mode/上下文代次、提交锁、结果未知保护；比赛页和Workbooks消费同一实例，R13-09复用 |
| 文件选择、比赛模板/数据/AI分析包导出 | matchExchange.ts通过LineupFiles/LineupApi端口 | 文件选择仍调用既有Platform原生对话框；取消不预检、不写入；业务编排不堆回通用层 |
| 状态与跨域接线 | lineupState.ts、lineupContext.ts、lineupPorts.ts、index.ts | 无第二套Session/Router/Workspace；比赛、预设通过同一Feature上下文；类型/命令/序列化/Schema不改 |

从旧main迁移76个函数。新模块按职责组织，root保留事件分派、现有pageLoader与其他域的组合接线；不称main所有业务已经退出。导入预检不再保留旧可写变量。模块入口登记，状态清单保留原49项并新增4项，共53项。DTO清单由原生成器刷新；没有放宽扫描或依赖限制。

跨页返回、补录赛事/球队、跳转球员档案、导出、历史查看/删除、比赛/模型链路及工作区状态恢复继续沿原真实入口；P4/P7/概率算法/模型资产与46项迁移均未改动。

## 已批准设计接入

来源：[Figma Screens](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu)。实时读回设计上下文与截图，按原DOM/CSS/token技术栈实现，不复制原型假数据、假完成态或验收入口。

| 状态 | 已核对Frame/子节点 | 本批处理和证据 |
|---|---|---|
| S02 Matches | 466:4436 | 248px目录、紧凑标题/详情、01/02分组、双列球队/状态、独立目录和详情滚动；移除重复大标题、步骤圆标和多层嵌套；保留层级、阶段、轮次、教练/阵型等所有真实功能 |
| S02双方编排 | 467:5786（既有批准接入） | 保留双边行列、44px行、完整位置/角色及原滚动恢复；本批验证实际预览/套用不会写原预设或客侧 |
| S08未选定 | 570:86212；目录…570:87108、结果…570:87414 | 首次进入预设页不误继承球队页选择，不自动选首队；248px目录及顶部72px空态，分页常驻 |
| S08选定 | 570:87458；结果…570:88499 | 60px标题、三块蓝色指标、预设事实/范围说明、下方动作；列表独立滚动 |
| S08编辑 | 571:91867；元资料…571:92654、成员A01…571:92958 | 资料与成员两区；元资料桌面3列/紧凑2列；6列表头/行同列对齐；40px控件、13px输入，成员48px行；一个编辑滚动区，底部保存区常驻 |
| 受阻/可应用、保存失败、复制读回 | 原预览/确认/Workspace控件与上述批准编辑来源 | 完整main挂载后由界面触发真实前端工作流，传输边界控制can_apply/失败/读回；不是后端真实结果 |

目录/卡片/文字层级沿批准token，普通输入400、标签/表格500、分区700；没有另起配色方向或全局改字号。新的编辑器样式由多页共用的presetEditor.css承担；S02样式限制在lineups路由，保留单一CSS图与旧加载顺序，新入口显式登记。没有新增生产依赖或远程资产。

Impeccable技能及layout/craft-floor已使用，手动检查实际截图、密度、滚动与可达性；自动engine-probe未通过：本机原引擎路径不存在，0.1.17官方下载失败。日志 `O:/Project/FOODBALL/logs/r13-07-impeccable-engine.log`。**不声称自动检测器通过。** 本记录只关闭所列差异及受测状态，不把一个主Frame或132项断言夸大为S02/S08所有原型状态逐像素验收。

## 风险复现与真实模块回归

原双边流程在await链路后读取可变builder。新生产行为用受控延迟复现A→B：修复前能发出一次不应有的paired写入，修复后0次。测试只替代外部API，执行生产模块，不重写状态机。

| 任务场景 | 前置/操作 | 预期及实际本地结果 |
|---|---|---|
| D1/V1 | 比赛A链路/同队名单延迟→换B或窗口、修改球员 | 原快照失效；0次错误提交、旧名单不回填；两侧序号独立。通过 |
| D2/V2 | A预设预览→换B/改草稿；正常应用后改本场 | 旧票据拒绝；正常应用只复制目标侧，原预设/客侧保持。通过 |
| V3 | 双边原子API抛错/迟返；重复点击；结果未知 | 保留双方草稿，不造单边成功；并发只发一次；未知结果先读原比赛链路和版本，未核实禁止盲重试。通过（仅前端边界） |
| V4 | 10首发/重复球员/缺位置/越界时间 | 写前拒绝；既有预设角色和阵型校验继续执行。通过；真实PG原子性另外待验 |
| 确认/生命周期 | 球员设置、历史删除、比赛保存中换对象/stop | 旧确认不发写入；迟到成功不清理新草稿或覆盖新链路。通过 |
| 导入 | 文件A迟返换B、旧裁决、取消、双击、错误摘要、超时、部分导入 | 正确批次/hash/mode、一次写入；取消0预检；未知先读原账本；错误行保留，不误报全部成功。8项通过 |

生产模块29项：21项阵容/预设/比赛，8项比赛导入；入口 `scripts/features/verifyLineupsWorkflow.mjs`、`verifyMatchImports.mjs`。均接入原verify-browser-bootstrap。132项完整挂载断言由 `scripts/features/lineups-browser/` 执行，使用真实main/Shell/Workspace/控件增强，**只有Tauri传输为明确fixture**。覆盖1440×900、960×720、1280×720、1920×1080及编辑/预设明暗，确认同列像素、无横向越界、末行与保存可达；包括受阻/正常预览、目标侧应用、失败保留、复制读回。

## BA-03受保护删除

`matches/catalog/delete.rs`在原事务/FOR UPDATE内将model.runs加入保护计数，保留P4研究、冻结和正式结算保护。移除与0041不可变历史冲突的 `UPDATE model.runs SET match_id=NULL`；错误明确表示保护拒绝。没有降低触发器或关闭历史检查。

新增真实PG契约 `match_deletion_preserves_model_run_identity`：实际插入比赛及model定义/版本/参数/run，删除应拒绝，读回match_id、比赛和零条删除成功审计。此测试已编译，但按既有专用授权新库边界保持ignored，**本批没有运行真实PG写测试**。不以Rust通过替代其结果，也不为清理fixture关闭不可变保护。

## 验证分层与证据

| 层级 | 当前结果 |
|---|---|
| 静态/架构 | 模块/状态53/样式边界及DTO检查已通过；原守卫扫描迁移后的真实owner，未删断言、未降阈值 |
| 最终完整前端 | PASS（退出0，96步）：`O:/Project/FOODBALL/logs/r13-07-frontend-final.log`；包括29生产行为/132完整挂载、TypeScript和Vite；29项生产模块行为及132项完整挂载通过 |
| Rust | `npm run verify:rust`退出0；fmt/Clippy、748 passed、40 ignored；`O:/Project/FOODBALL/logs/r13-07-rust.log` |
| Windows原生构建 | `cargo build --locked --manifest-path src-tauri/Cargo.toml`退出0；`O:/Project/FOODBALL/logs/r13-07-native-build.log` |
| Windows实际运行 | 真实当前配置+原生IPC，比赛和预设目录读取；4项通过、32条日志、0 errors；无fixture、无业务写入；证据 `O:/Project/FOODBALL/tmp/r13-07-native-read-VIF2Tg`，startup-report.json为PASS |
| 自身CI/安装包 | 855d830b/run41211916572/job124568508883全成功；artifact13047682283摘要、MSI/NSIS及startup/pass已核验，见正式收尾 |
| 真实DB/XLSX/Full | 不通过模拟核销：BA-02/03/04/06/07按原账本R13-09/R17-06收口；Full、私有固定回归、正式推演/结算/API凭据仍须各自真实证据 |

测试进程TEMP/TMP为 `O:/Project/FOODBALL/tmp`。保留最终证据和依赖；测试profile的递归清理被外置retain-test-artifacts.cjs拦截而保留，未跳过验证。无用中间产物按明确清单交用户手动清理；不执行批量或递归删除，不触碰用户Temp。最终清单在同一项目临时目录保存，避免重复写入可复用依赖。

## 收尾、回退与下一项

代码回退基线f59d12f6：审阅本批提交后针对性revert；无Schema变更；Git不能恢复真实数据库写入/配置/凭据。本批无文件删除。推送沿用用户长期授权，不重复询问；CI不轮询。

本批CI、双bundle和startup未核实前不记DONE。正式收尾后下一项 **R13-UI01（已交付6页设计补差）**，再到 **R13-09（工作簿）**；R13-08已包含在本批，不单独重做。后续每批/阶段完成时，交接和回复都必须明确下一编号、主题及待满足的前置，不自动启动未授权任务。

最终完整挂载证据：`O:/Project/FOODBALL/tmp/r13-07-browser-MDho9G`（result.json、比赛/编辑器/预设多尺寸明暗截图）；最终原生证据 `O:/Project/FOODBALL/tmp/r13-07-native-read-VIF2Tg`。本地来源与日志摘要清单：`O:/Project/FOODBALL/tmp/r13-07-final-manifest.json`；手动清理清单：`O:/Project/FOODBALL/tmp/r13-07-cleanup-manifest.json`。

## 精确提交与CI交接

代码提交 `855d830b77ba6b6eb9c2e080009aa50cbea3efdb` 已推送，62路径（22新增、40修改、0删除）。最终完整96步退出0，29项生产行为及132项完整挂载通过；286处本地文档链接核验有效。精确[CI run41211916572](https://github.com/uniquenesssta/123/actions/runs/41211916572)仅查询一次：push事件，head_sha精确等于855d830b，status=in_progress、conclusion=null。本批仍VERIFYING，未验自身双bundle/包内startup，不轮询、不取消其他运行。

此CI交接为后续Markdown补记，不改变上述受测代码；ci.yml的push paths-ignore排除Markdown。本次手动清理清单含302个明确中间产物/profile/一次性辅助项，未执行批量删除；最终证据、依赖和原生复现助手保留。正式收尾后下一项R13-UI01，再R13-09，08已并入07。

## 正式收尾与UI01启动

用户授权“收尾07开始R13-UI01”后，一次核对精确855d830b/run41211916572/job124568508883：所有步骤completed/success，2026-10-31T15:22:23Z完成。本批正式DONE；此前VERIFYING段落保留为历史。

- artifact13047682283：52,323,930 bytes，SHA256 `32ad1aa78b01f0a090e0afe2798d9707dce03d287f5b8bee5b9536fed128e998`，与GitHub digest一致。CLI下载超时后通过GitHub集成取得同一产物并校验，没有重复查询CI。
- MSI：17,989,632 bytes，OLE头正确，SHA256 `89b5a77567730c3a4f0ea78d57257f96cfb14c6d767561fcc0274e7e678b506e`。
- NSIS：15,626,860 bytes，MZ头正确，SHA256 `5d056b53c784c230ed24a475a7c9bf9c447502c29248ea5da8144aa16ee3959d`。
- 包内windows-acceptance-20261031-145041.json：startup/pass，7条日志、bootstrap/read_workspace_state/save_workspace_state，0错误/0缺失；不等于Full。

证据：`O:/Project/FOODBALL/tmp/r13-07-ci-41211916572/extracted`。BA-03等真实外部项继续原R17-06账本。原08已包含；**接下来R13-UI01六页设计补差，已获本轮启动授权；其后R13-09，尚未启动。**
