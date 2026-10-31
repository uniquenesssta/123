# R13-06 球员目录、履历与观察

状态：**DONE**，2026-10-31。仅覆盖原R13-06，代码基线3e930a5a；初始工作区干净。05按精确587b6752/run41172800714/下载摘要/双bundle/startup正式DONE；07未启动，R12保持结束。

依据：阶段任务书、remaining-execution-contract及BA-05/BA-07。沿既有rewrite/r13-core-ui-features分支，推送沿用户长期授权，不轮询CI。任务聚焦现存功能的唯一owner、真实接线与已证实缺陷，保留批准视觉。

## 实际盘点与原入口映射

| 原状态/入口/事件 | 当前owner、原命令与保留行为 |
| --- | --- |
| playerQuery/playerCursorHistory/playerListPage/playerPageLoading/selectedPlayer/selectedPlayerIds/playerReferences | PlayerCatalog，唯一查询/游标/选择/详情/引用目录；原listPlayers/readPlayer/playerCatalogReferenceData，真实响应原子发布；查询和游标仅成功后提交 |
| initialPlayerQuery/readPlayerNavigationContext/persistPlayerQuery/setPlayerNavigationContext | PlayerCatalog.normalizePlayerQuery/normalizePlayerSource/setSource；原两个preferenceKey不变，根只注入持久能力；source使用原PlayerNavigationContext DTO，不造另一套Router/NavigationContext |
| fetchPlayerCatalog/applyPlayerCatalog/loadPlayerCatalog | 移入PlayerCatalog.load/refreshDetail；原pageLoaders重复实现退出；main/Teams跨域读取通过公开PlayersFeature和明确setter，不保留可写根别名 |
| searchPlayers/clearPlayerFilters/nextPlayerPage/previousPlayerPage | PlayersFeature与PlayerCatalog；原双向游标、筛选值、cachedReferences保留。playerDirectoryDom沿原tbody/pagination/ribbon局部更新，不以整页busy遮罩代替翻页 |
| openPlayer/openPlayerProfile/reloadSelectedPlayer | PlayersFeature.openDirectory/openProfile与Catalog.open/refreshDetail；旧详情立即失效、响应player.id核对、最新请求/Session/页面保护，稳定ID标签仍归原Workspace |
| prepareDirectPlayerDirectoryEntry；openPlayerFromTeam/openPlayerProfileFromTeam/openPlayerFromLineup及返回 | PlayersFeature.directEntry/enterSource、Catalog.source与原根跨域桥接；只传原team/player ID，保留普通目录/显式完整档案区别、返回原球队或比赛阵容；不把来源变成锁定 |
| updatePlayer/addPlayerName/assignPlayerPosition | PlayerEditor身份/别名/位置写入，DTO字段/空值/语言/主位置/default_role_code/valid_from保持原契约；原updatePlayer/addPlayerName/assignPlayerPosition命令不变 |
| addPlayerTeamPeriod/addPlayerAvailability | PlayerEditor独立履历/可用性提交，原生效区间、队伍、号码、注册状态、confidence和日期转换不变；后端继续校验 |
| addPlayerAbility/addPlayerDynamicTag | PlayerEditor分别调用addPlayerAbilityObservation/addPlayerDynamicTag；general/manual.v1、原时间/样本/失效默认值保持；标签不写长期能力，提交仅确认对应字段 |
| addPlayerExternalId/createProvider/searchTeamOptions | PlayerEditor原addExternalEntityId/createDataProvider与Catalog.searchTeamOptions；保留同源实体ID、providerId、手工metadata和原listTeamOptions；选项迟到不覆盖新会话/新查询 |
| calculatePlayerContribution | PlayerEditor捕获稳定playerId和实际competition/position/as_of请求，原calculatePlayerMatchContribution；上下文变化不弹旧结果，不构造贡献数值 |
| createPlayer | PlayerEditor先提交捕获草稿，再通过原根跨页协调刷新摘要/目录并读取后端created.id；刷新失败明确“已创建、勿重复创建”，不造目录项或再次调用create |
| spreadsheetPreview与preview/resolve/commitPlayerImport，导出模板/数据 | PlayerImports单一批次/hash/mode/代次/锁；Players和Workbooks同一实例，R13-09复用。原previewPlayerCatalogImport/resolvePlayerCatalogImportConflict/commitPlayerCatalogImport/readPlayerCatalogImportPreview/export命令全部保留 |
| requestDeletePlayer/requestBulkDeletePlayers/bulkArchiveEntities玩家分支 | PlayerDeletion确认能力与真实输入/结果校验；PlayersFeature原Modal确认和共享bulkArchiveSummary/bulkDeleteSummary；后端仍负责引用/逐项事务；根只协调实际成功ID的标签/摘要与他域刷新 |
| 原players.ts全部View | playerDirectoryView/playerProfileView/playerWorkbookView/playerPresentation，公开index.ts组合；唯一旧players.ts路径单项退出，原CSS/图标/控件复用 |
| 编辑DOM快照 | PlayerDrafts按Session快照+playerId隔离，明确data-player-draft-id与data-workspace-persist=false；Workspace仍拥有布局/标签/其他控件；成功仅确认已提交且等待期间未再改的字段，不清所有表单 |

新增4个状态owner，总数49；19个Feature、318个受扫前端源码。PlayerApi是从真实client签名生成的类型端口，DTO继续以src/types.ts为唯一契约。未新增生产依赖、命令、Schema、模型参数、runner或workflow；未改任何历史JSON契约/钉住的迁移摘要。

跨域保留：比赛页快速建球员及组装阵容仍归R13-07现存根流程，消费同一个Players引用目录；没有提前重做07。任务书“比赛历史”按实际源码核对：原PlayerDetail没有独立比赛历史字段、client没有独立player-match-history命令。本批完整保留其名称/球队履历/位置/伤停/能力/动态标签/外部ID历史及从实际比赛阵容进入和返回的稳定来源链；不凭任务名称制造新的历史API或假比赛列表。现存比赛/复盘历史所有权仍归相应工作区与后续批次。

## 已证实缺陷与实际修复

1. A读取迟于B返回会直接覆盖selectedPlayer：现清旧详情、校验返回ID，序列/Session/页面改变后丢弃旧完成及旧错误。
2. 查询/来源/表单散在main，跨对象同ID控件被恢复：现在Catalog与PlayerDrafts各自唯一，A/B和换库隔离；来源筛选可编辑可清除，普通用户自己设的球队过滤保持。
3. 真实浏览器复现“同页通用球员入口提前return，来源未清”：activatePage先执行受限来源清理；发生清理时同页也走原Router刷新；无来源的同页点击仍不重载。直接执行实际activatePage的永久回归保留。
4. 月度导入无代次/可变preview确认，error_count>0仍成功提示：捕获原batch/hash/mode，文件/对象/Session/模式变化作废旧确认；失败保留预览/裁决，部分失败显示错误计数和后端读回，不全局刷新掉候选，也不自动重试。
5. 批量操作后清理可能抹除失败对象：只移除后端deleted_ids/archived_ids/already_archived_ids，失败对象与选择保留，展示真实reason和成功/请求总数；确认单次、换对象/选择失效、取消0写入。
6. 写入返回但刷新失败有重复提交风险：编辑、创建和批次读回区分已写入与尚未核实，提示核查原对象/批次；身份保存后中文别名失败单独报告。

## V场景与分层验证

| 场景 | 实际调度与预期/结果 | 证据 |
| --- | --- | --- |
| V1 来源/游标 | 球队A→球员完整档案→点击同页通用球员入口，source清除、team_id=null；无来源普通过滤不清。next/previous游标与当前查询一致，失败不前移，翻页复用原表格节点 | 生产owner及实际activatePage测试、真实入口浏览器 |
| V2 独立草稿/观察 | A身份保存拒绝保留A；A伤停成功仍保留能力草稿88，标签只发add_player_dynamic_tag；B无A字段，切回A仍有A草稿；失败只影响对应写入，不ack全部 | PlayerEditor/PlayerDrafts行为、浏览器传输调用记录 |
| V3 部分成功 | 后端bulk_delete_players返回A成功/B引用拒绝，原确认取消0调用，正式确认只移除A、B仍选中并显示原reason。导入error_count=1只显示部分处理，保留readback、不发全局bootstrap刷新 | 真实PlayersFeature/PlayerDeletion/PlayerImports与共享结果View；不是PG事务验收 |
| V4 乱序身份 | 延迟A、完成B后再释放A，仍为B；过期错误不替换B；错playerId拒绝，Session/页面变更不发布。原写入A迟到不回绑B表单 | 生产类及Windows浏览器实际DOM |

本轮验证情况：

- 架构：完整npm run verify:architecture退出0，r13-06-architecture.log；49状态owner、19Feature/318源码通过。Tauri命令契约171项、18份保护资产摘要不变（d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57）；私有P4/P7仍缺席。
- 自动行为：新增30项，通过现有verify-browser-bootstrap门禁；实际执行Catalog/Editor/Drafts/Imports/Deletion/Feature和提取的生产activatePage，只替换外部传输/Session/DOM边界。原42项Teams、31项赛事规则、生命周期/Session测试保留并通过。r13-06-behavior-final.log。
- HTML：与3e930a5a之前原players.ts对16种已配置/未配置×有无详情×四页签相同，仅排除新增草稿技术属性及空白行尾规范化；原批准视觉保持。
- Windows浏览器：O:/Project/FOODBALL/tmp/r13-06-browser-1l0eTJ/result.json、teams.png（复用截图工具文件名），26项PASS。实际bootstrap/main/Router/Session/Workspace/Feature/原动作；只替换Tauri和dialog边界。
- 类型/构建：最终新增创建刷新恢复断言后，TypeScript --noEmit与Vite build退出0，r13-06-build-final.log；构建保留原大chunk提示，没有关闭检查。
- Windows原生：cargo build --locked -p football-match-model-desktop退出0，r13-06-native-build.log。真实WebView2点击资源→球员中心，实际list_players/player_catalog_reference_data完成，21条日志、0错误/0业务写入。报告O:/Project/FOODBALL/tmp/r13-06-native-read-J7NiWC/read-report.json与native-team.png；4项检查。当前目录为空，未声称原生read_player/写入已验；沿真实Windows配置，只隔离日志，不冒称APPDATA隔离。
- 完整96步前端：最终标准node scripts/verify-frontend.mjs退出0；r13-06-frontend-verified.log实际96个[verify]步骤，最终30项球员行为和原42项Teams等均执行通过，包含TypeScript与Vite build。初轮导航守卫仍缓存旧main文本，随后原归档守卫还错误依赖已迁走的球员局部变量；分别修为真实playerDirectoryDom与Team afterArchive清理回调断言，失败记录保留，未删除/跳过门禁；最终完整入口一次性通过。

本机Windows/Node24.16；本批Rust/Schema/Cargo锁、171命令、模型资产与05精确587b6752同树，完整Rust检查/测试引用该精确CI，未冒称本地重新跑Rust全测。另已实际重建Windows开发版。自身代码CI/双bundle/包内startup在推送后单独验收，不轮询。

## 实际消费者及历史契约

所有真实View消费者更新到公开Feature或具名View：设计fixture、productionFixture原局部加载调用、pattern真实render、导航布局和原功能守卫。源路径变化后的静态断言查新owner与根接线；实际路由fixture改用真实PlayersFeature，原跨await失效检查仍执行。05的Team mutation ports继续验证跨页刷新，引用发布改接同一PlayerCatalog，未引入第二参考目录。

原历史contract.artifacts里的players.ts采用既有迁移分支验证“旧文件退出+四View及实际owner存在+根公开出口”，保持契约JSON及旧迁移哈希原样。仅旧src/pages/players.ts一个源码路径退出；没有批量删除或替代后端。

## 待验、临时文件与交接

| 项目 | 承接和停止条件 |
| --- | --- |
| BA-05 | 本批稳定同源ID、引用拒绝、取消及部分结果完成；真实PG匹配/删除contracts、并发/回滚/历史读回仍R17-06授权专用新库验收，旧fixture不抵销 |
| BA-07 | 原Player XLSX完整preview/resolve/commit/readback接线及恢复保留；有效文件的行/子记录身份、暂存裁决、账本实际读回由R13-09/R17-06闭合 |
| Windows Full/私有模型 | startup/真实目录读取不是Full；实际导入、正式推演/结算及私有P4/P7按既定R17-06/R18门禁 |

用户最新要求：06完成后将可删除临时文件移入O:/Project/FOODBALL/tmp/old，由用户手动删除。本轮不删除临时产物；保留依赖/缓存、最终验收日志/结果/截图。结束前按明确清单移动已被最终结果替代的失败中间物、一次性迁移稿与本轮Chrome临时profile；不移动用户其他项目文件、不覆盖同名目标、不批量删除。实际清单与位置完成后补记，避免无限累积或重复写入可复用文件。用户随后明确：后续允许项目临时目录按核对清单逐文件清理，本次仍由用户手动删除；执行约定已同步，不扩大到用户Temp或其他项目。

代码回退点3e930a5a，审阅本批提交后针对性revert；Git不能恢复数据库、配置或凭据。当前仅06已授权；07未启动，须本批自身出口及后续授权。

## 本批交付状态

本地实现及所有本地必需门禁已完成，当前VERIFYING；代码推送后只按自身精确SHA核对一次CI，双bundle/包内startup等待精确运行结果。未把05的绿色CI或本机startup当成本批打包通过。07未启动。

## 精确代码提交与CI交接

代码3683b7f30576704ba2c77a58f25606525390642a，已推送既有R13分支，源码树aa36c749f2016870b0bb9e8bec73d9385a076437。50路径：16新增、33修改、仅1个旧players.ts路径退出。最终完整96步退出0；30项球员行为、26项Windows浏览器、16种HTML等价、真实原生目录及startup报告通过。

自身[Public Platform CI run41183380347](https://github.com/uniquenesssta/123/actions/runs/41183380347)只查询一次：精确head/分支/push匹配，in_progress/conclusion=null。自身MSI/NSIS/包内startup尚未验收，06保持VERIFYING；不得以05证据替代。未轮询，07未启动。

## 本次old临时产物交接

按用户本次手动删除要求完成明确路径移动：O:/Project/FOODBALL/tmp/old/r13-06-20261031/manifest.json。首轮2269项移入待删区，其中2201个是captureHtmlScreenshot产生且已无活跃进程的临时Chromium profile，其最终PNG本就在独立证据目录。其余为本轮被最终结果替代的失败/中间日志、失败fixture、迁移前对照和一次性生成稿；目录边界逐项核对，无递归删除、无覆盖、无移动失败或活跃进程遗漏。记录脚本随后自身归档，最终项数以manifest为准。

保留O:/Project/FOODBALL/tmp/r13-06-frontend-verified.log、r13-06-behavior-final.log、r13-06-build-final.log、r13-06-architecture.log、最终浏览器和原生结果/截图/startup报告；最终复现工具移动到O:/Project/FOODBALL/tmp/r13-06-evidence-tools。node_modules、Rust目标缓存、05精确CI证据、真实Windows配置和其他用户文件保持。失败中间物的位置以manifest的source→destination对应表追溯。

用户已另行授权后续项目临时目录按核对清单逐文件清理；本次只移动，由用户手动删除old。不宣称移动已释放磁盘空间。上述后续授权已写入执行约定，没有改系统TEMP/TMP。

## 2026-10-31 正式收尾

用户授权收尾06开始07后，精确3683b7f30576704ba2c77a58f25606525390642a/run41183380347/job124485219358与全部返回步骤completed/success；2026-10-31T08:41:54Z（北京时间16:41:54）完成。30项球员行为、完整前端/Rust/release及startup在同一运行完成。

- artifact13036202935，52,306,225 bytes，未过期。下载ZIP SHA256 59284444b102cd70692d2967182c0178dabeeabfb923b5618a4e1d3e78c11b71，与API和上传日志一致。
- MSI 17,981,440 bytes，OLE头正确，SHA256 a80c33c0283e512133ddd8286e30cf290c074a914586303a2b930cee4f503c4c。
- NSIS 15,622,125 bytes，MZ头正确，SHA256 00d839f7d9b8448f27731c9bd1ac438975d40c9bb8772bceaa3ea12e101c0c83。
- 包内logs/windows-acceptance-20261031-080956.json为startup/pass，7条记录、bootstrap/read_workspace_state/save_workspace_state，0错误/0缺失。startup不等于Full。

本次单次结果验收、不轮询。解包证据O:/Project/FOODBALL/tmp/r13-06-ci-41183380347/extracted。06正式DONE，BA-05/07及Full/有效XLSX/私有模型仍按既定R17-06承接。07现已授权，覆盖原07/08；09未启动。此前VERIFYING段落保留为历史，以本收尾结论为准。
