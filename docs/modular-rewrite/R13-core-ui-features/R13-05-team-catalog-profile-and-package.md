# R13-05 球队目录、档案与资料包

状态：**DONE**，2026-10-31。本批仅覆盖原R13-05；03及并入的04已按精确b5b23c46/run41116421113正式收尾，收尾文档提交5c0bd02f。本批代码基线5c0bd02f，初始工作区干净，实施中没有用户未提交修改；06未启动，R12保持结束。

依据：阶段任务书R13-05、remaining-execution-contract、前期基线BA-05/BA-07。源码锚点核对后，实际迁移全部球队状态/动作与视图，不只迁移任务书代表函数。提交推送沿用用户长期授权；自身CI精确提交单次查询、不轮询。

## 实施与唯一owner

| 原入口/状态及调用 | 当前唯一owner和真实接线 |
| --- | --- |
| initialTeamQuery/persistTeamQuery、teamQuery、teamCursorHistory、selectedTeamIds | TeamCatalog：规范化原偏好键、成功后提交查询/游标、稳定ID选择；main只注入原localPreferences/Workspace能力 |
| fetchTeamCatalog/applyTeamCatalog/loadTeamCatalog、teamListPage/selectedTeam/lineupHistory/presets | TeamCatalog.load/open/readDetail；原listTeams/readTeam/listTeamMatchLineups/listTeamLineupPresets，身份/Session/代次检查，原子发布；原pageLoaders重复实现退出 |
| searchTeams/clearTeamFilters/nextTeamPage/previousTeamPage | TeamsFeature调用同一Catalog，筛选成功重置游标、失败页码不前移；目录选择和详情保持分工 |
| openTeam/openTeamProfile/selectLineupPresetTeam/reloadSelectedTeam | TeamsFeature.open/openProfile/selectPresetTeam，Catalog.refreshDetail/refreshSelected；Router导航、Workspace标签/布局仍由原owner唯一管理 |
| updateTeam/addTeamName/saveTeamProfile | TeamEditor，捕获teamId及实际DTO再await；原updateTeam/addTeamName/upsertTeamProfile，中文空值不抹除旧名称、别名历史保持；身份已保存但别名失败单独报告 |
| createTeam/createCoach/addTeamCoachPeriod/saveFormationUsage | TeamsFeature.create、TeamEditor.createCoach/coachPeriod/formation；原createTeam/createCoach/addTeamCoachPeriod/saveFormationUsageDistribution命令及参数保持，当前主教练来自后端投影，教练任期/阵型/历史没有删减 |
| 同名DOM字段的草稿 | TeamDrafts：按Session快照+teamId隔离，成功只确认已提交且未再编辑的字段；技术属性data-team-draft-id标识作用域，data-workspace-persist=false阻止Workspace再保存同一档案草稿；Workspace继续拥有布局/标签/其他模块控件 |
| teamPackagePreview、preview/resolve/commit/exportTeamPackagePreviewJson | TeamImports独占当前包、文件摘要/子批次/模式/代次与提交锁；TeamsFeature供原UI操作，原API/后端账本不变；R13-09复用此owner |
| teamSpreadsheetPreview、preview/resolve/commitTeamImport | 同一TeamImports的monthly分支；完整预检、冲突裁决、原批次提交和后端读回，旧SHA或模式响应拒绝回填 |
| exportTeamPackageTemplate/exportTeamTemplate/exportTeamData | TeamsFeature.exportFile，原文件选择/导出命令保留；JSON导出先捕获预览，取消或期间换包不写旧文件 |
| requestBulkDeleteTeams、forceDeleteTeamPreview/confirmForceDeleteTeam | TeamDeletion：原后端逐ID预检、无引用过滤、单次确认、Session/对象改变失效；TeamsFeature复用原Modal/确认/结果面板，取消不写入。最终引用保护仍由原后端事务负责 |
| bulkArchiveSummary/bulkDeleteSummary | components/entityDeletion.ts共享纯展示，球员与球队共同复用；团队引用/强删说明留teamDeletionView，不复制策略 |
| archive-selected-entities球队分支 | TeamsFeature.archive捕获选择与上下文，仍调用原bulkArchiveEntities；main仅协调其他模块标签和全局摘要刷新 |
| previewPlayerFromTeam/openPlayerFromTeam/openPlayerProfileFromTeam/返回来源球队 | TeamsFeature.readPlayerFromTeam捕获原teamId/name及最新球员请求；main公开桥接传稳定身份、原来源上下文与原Router；返回可读不在当前筛选列表中的原球队 |
| 原pages/teams.ts的全部视图 | teamsView、profileView、packageView、teamPresentation，经features/teams/index.ts公开出口组合；原players/lineups/presets/workbooks保留各自职责，未提前迁移06/07/09 |

删除/强删/归档后的跨页刷新每个await后复核当前上下文；强删后的球员数据先读后核对再发布，避免旧刷新覆盖新页面。

四个新状态owner登记到architecture/state-ownership.json，总数45；teams真实边界和公开出口登记到module-boundaries.json，19个Feature、305个受扫前端文件。类型/传输DTO继续以原src/types.ts为唯一契约；TeamApi仅类型端口，实际注入原api，不新增客户端/命令/Schema/模型/依赖/runner/workflow。

保留共享coaches/formations的现有组合根投影，因为阵容/球员/预设仍消费它们；通过具名回调发布，未复制一个Teams独占的全局目录。main只剩事件分派、共享跨页协调和公共owner接线，移出原球队可写状态与工作流。

## 修复及恢复语义

- 旧A详情、历史或预设晚于B返回，不发布A，也不把A标成B。后端对象/子记录team_id不匹配拒绝；列表加载中途Session或导航切换，不继续发布。分页失败保留先前成功游标。
- 档案字段以前按同名DOM控件跨对象恢复；现在Session/球队隔离。失败保留输入，成功仅确认实际提交字段，等待期间的新编辑不被清掉；Session换库不带入旧草稿。
- 包A/文件选择A/球队A的旧回调不能覆盖B；裁决比对batch_id/source_sha256/import_mode；提交只发送捕获的子批次，A返回不清B、同一代次不能双提交。
- 拒绝或部分处理保留原包、冲突选择和后端读回，手动恢复仍用原批次；无自动重试。提交返回但读回失败明确提示先核对原批次，不把它说成“未写入”。月度成功沿原refreshAll刷新Session，失败保留候选；不构造前端账本。
- 普通删除只把后端预检允许的无引用ID传给原删除命令；最终后端仍可拒绝。强删输入匹配且对象仍有效才发原请求，能力消耗后失败需重新预检；没有扩大删除权限。
- 当前中文名只投影有效日期区间的后端名称；历史完整保留，不把过期或未来名称当当前别名。能力/角色/履历缺失继续显示待补全或“—”，不补造。

## 场景证据与分层结果

生产测试scripts/features/verifyTeamsWorkflow.mjs通过现有verify-browser-bootstrap入口运行实际TeamCatalog/TeamEditor/TeamDrafts/TeamImports/TeamDeletion/TeamsFeature，仅外部API/Session/DOM边界可控。新增42项，不在测试中复制生产状态机。另三个生产组合根回调通过AST提取直接执行，每个覆盖换页/换球队/换Session及成功；包提交两个子批次结果同时与原候选核对，缺失或错批次均保留预览并要求核查。保持前置架构入口不依赖TypeScript；TS运行测试只在已安装依赖的原frontend门禁执行。

| 场景 | 前置、触发与实际结果 | 层级/证据 |
| --- | --- | --- |
| V1 读取/搜索 | team A延迟、先完成B再释放A；最终仍B。B失败不残留A；多阶段列表原子性、Session/导航失效、错team_id、next/previous失败回退和新搜索第一页均断言通过 | 生产行为及真实入口Windows浏览器，浏览器结果result.json |
| V2 包身份/恢复 | 文件a.xlsx预检延迟，再选b.xlsx并先完成B；A不覆盖B；提交捕获b-team/b-player，后端拒绝后B预览仍在。换球队后旧预检不可提交；裁决行/实体/批次、部分链恢复、月度SHA和读回失败另由实际owner断言覆盖 | 42项生产行为、浏览器26项 |
| V3 删除 | 已引用球队预检后无bulkDeleteTeams请求；可删确认取消0写入；正式提交时后端引用复检拒绝并显示原错误；强删取消0写入。改变对象/选择、迟到预检、重复确认和失败能力消耗由生产owner断言覆盖 | 生产行为、浏览器26项；不是实际PG删除 |
| V4 来源/草稿 | A档案编辑失败保留A；B字段不带A、各自再打开恢复自身草稿；球队A进入真实球员页后返回按钮携带原teamId，返回A档案与A草稿。目录过滤排除A时仍可按显式ID返回 | 生产行为、真实页面/Router/Workspace桥接；readPlayer边界受控 |

证据位置（全部为本地测试产物，未提交数据/日志）：

- 受控Windows浏览器：O:/Project/FOODBALL/tmp/r13-05-browser-usDkfu/result.json、teams.png，26项PASS。真实生产入口、Router/Session/Workspace/Feature及原DOM动作；只替换Tauri传输/原生dialog边界，不代表真实PG写入。
- 原HTML：r13-05-verify-views.mjs对16种已配置/未配置×选择/未选择×四页签，与5c0bd02f之前原teams.ts逐字相同，仅排除新草稿技术属性；批准视觉未改版。
- Windows开发版：cargo build --locked -p football-match-model-desktop退出0。O:/Project/FOODBALL/tmp/r13-05-native-trJwRS/startup-report.json为pass，5条日志、bootstrap/read_workspace_state完成、0错误；未重新执行Full。
- 原生真实读取：O:/Project/FOODBALL/tmp/r13-05-native-read-FKf5yl/read-report.json及native-team.png。无模拟transport/fixture，真实WebView2操作资源→球队目录；4项检查、14条日志、6个完成操作（bootstrap/read_workspace_state/list_formations/list_coaches/list_teams/save_workspace_state）、0错误、0业务写命令。沿当前Windows已保存配置，目录无可打开球队，所以未声称本次原生readTeam/编辑/删除已验。仅隔离日志目录；未声称APPDATA能隔离Windows配置。
- 架构：npm run verify:architecture退出0，r13-05-architecture.log；命令契约171项不变；保护资产18项，聚合SHA256 d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57，私有P4/P7仍缺席。
- 完整前端node scripts/verify-frontend.mjs退出0，r13-05-frontend-final.log实际96个[verify]步骤，含TypeScript与Vite build。该次含37项球队行为；随后最终回调/子批次身份补修后，组合根42项生产行为、5个受影响包/恢复/删除守卫、TypeScript与Vite build再次退出0（r13-05-behavior-final.log、r13-05-build-final.log）。没有把此前37项说成最终42项。

本批不改crates/、Cargo.toml/Cargo.lock、后端Schema/契约/私有模型；本地未重跑完整Rust测试/Clippy，沿同后端树b5b23c46精确Windows CI的Rust证据，另已实际重建当前开发版。上述复用不是本批Rust全测新结果。

## 消费者、历史契约与退出

原src/pages/teams.ts逐项退出并由四个真实视图替代，未删除其他文件。全部实际消费者迁到公开Feature或具名新owner：设计浏览器fixture、导航布局、pattern真实render/static、原页面/删除/资料包/表单/来源/工作区守卫。原31项赛事测试和全部原断言意图保留，旧函数文本断言改为根接线+新实际owner断言，新增行为门禁补足竞态。

四个历史JSON契约及0020迁移所钉摘要保持逐字不变。team-player-management旧artifact清单的teams.ts路径以与原有迁移一致的验证分支核对“旧文件退出+新实现存在+根使用公开入口”，没有改迁移/历史哈希来制造通过。

第一次全量复验暴露旧阵容/球员入口函数名断言，修复后复验；随后暴露pattern脚本动态拼接旧路径，已把四类实际消费者断言迁往真实新视图，未删除或跳过检查。最终96步及后续受影响复验全部通过。开发进程TEMP/TMP使用O:/Project/FOODBALL/tmp，已单独提醒；所有测试清理路径均保留产物，没有批量删除。

## BA承接、回退与后续

| 待验 | 本批完成与剩余 |
| --- | --- |
| BA-05 同源身份/引用/删除 | 本批稳定ID、后端引用拒绝及确认失效/取消保护完成；真实匹配/普通删除/强删repository contracts、并发父锁/提交/回滚及历史读回仍R17-06授权专用最终新库验收 |
| BA-07 有效XLSX/账本 | 完整包和月度preview/resolve/commit/recovery原命令接通；有效Team/Player/Monthly/Match XLSX实际提交、行/子记录身份、暂存裁决及账本读回仍R13-06/09和R17-06承接；生成fixture或预览不能抵销 |
| Full/私有资产/安装包 | 本地startup及真实目录读取不是Full；私有P4/P7固定回归、有效完整导入和PG持久化按既定R17-06；本批自身CI/MSI/NSIS及包内startup必须精确提交单独验收 |

代码回退点5c0bd02f；审阅本批提交后针对性revert，不reset用户修改；仅旧teams.ts路径迁移退出。Git回退不恢复DB、工作区或凭据。下一批是R13-06，需本批自身出口及用户授权，不在本轮提前启动。

## 当前交付状态

本地实现与所有必需本地门禁完成，当前VERIFYING；自身精确提交CI、双bundle和包内startup尚待提交后核对，不能标DONE。BA-05/07和Full等真实待验继续显式保留。

最终补核：导入模式改变只使对应package/monthly预检及待返回请求失效，保留另一工作流；真实浏览器重新预检恢复路径通过。最终42项生产行为、26项浏览器及TypeScript/Vite复验通过；无依赖副本确认TypeScript MODULE_NOT_FOUND且完整verify:architecture退出0。副本及日志位于C:/Users/uniqu/.codex/visualizations/2026/10/24/01a16c6c-7941-7f71-9469-0ff856e39e3a/r13-05-ci-no-deps-iYdQFa/。

当前暂存56个路径：19新增、36修改、仅1个旧视图路径被具名实现替代退出；后端/历史契约/依赖/workflow无差异。

## 精确代码提交与单次CI查询

- 代码提交587b6752e00ca0aaa56601b27ba1a222081e4597，已推送rewrite/r13-core-ui-features。受测源码树784d0b33e36006b2b89c557ac2327aef8e09b491；无TypeScript副本最终架构门禁退出0，日志architecture-no-deps-final.log。
- 仅查询一次：[Public Platform CI run41172800714](https://github.com/uniquenesssta/123/actions/runs/41172800714)，head/分支/push事件精确匹配，状态in_progress，conclusion=null。创建时间2026-10-31T05:29:37Z（北京时间13:29:37）。没有再次查询、轮询或使用03/04绿色记录抵销。
- 当前未下载本批产物，MSI/NSIS及包内startup仍待此精确运行完成后验收；05保持VERIFYING，06未启动。
- 本机Node24.16.0；CI配置Node22/Windows2025。已对无TypeScript、无node_modules的安装前源码环境补验；不将本机环境等同于CI环境。
- 本次仅补三份Markdown记录；现有workflow对push已配置paths-ignore: **.md，因此此次文档提交不应触发另一份代码CI；未更改workflow。

## 2026-10-31 正式收尾

用户“收尾05开始06”后独立核验587b6752e00ca0aaa56601b27ba1a222081e4597：run41172800714/job124454158700与全部返回步骤completed/success，2026-10-31T06:16:26Z（北京时间14:16:26）完成。42项球队行为及Windows前端、Rust、release打包与startup链在同一运行完成。

- artifact13032811710，52,313,426 bytes，未过期；下载ZIP SHA256 2d89ef905be35a1d4df5d0021e9cec9d053ae81c34f568be7fbd9d7d1cb88882，与API digest和上传日志一致。
- MSI 17,981,440 bytes，OLE头D0-CF-11-E0-A1-B1-1A-E1，SHA256 99bdcfc442c783e25f032ab569057d9c9f125a6207643a00d67578c2fea5f83a。
- NSIS 15,620,116 bytes，MZ头正确，SHA256 b954f2215f5a6a771d2b8dea5997fb8707f51284b90cff57c76b092a64f4d5cc。
- 包内logs/windows-acceptance-20261031-053058.json为startup/pass，7记录、bootstrap/read_workspace_state/save_workspace_state、0错误/0缺失。日志泛称“全链路”不改变报告实际profile，未冒称Full。

解包证据O:/Project/FOODBALL/tmp/r13-05-ci-41172800714/extracted。本次单次结果核验，无轮询。05正式DONE；BA-05/07真实PG/有效XLSX/Full/私有模型仍按既定R17-06承接，不能以本次startup抵销。06现获授权，07未启动。此前VERIFYING段为历史快照，以本收尾结论为准。
