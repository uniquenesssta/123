# R11-05 Lineups、Presets 与 Workbooks Commands

状态：`VERIFYING`（自身完整Windows尚待验收）。

R11-01～04 DONE；05 Lineups/Presets/Workbooks Commands已实施VERIFYING，06～10 BLOCKED，R11 IN_PROGRESS。用户2026-10-23“收尾04开始05”，继续唯一rewrite/r11-tauri-commands；基线9e39ba0c22e5e1c224252b891965718e2c6ebad5为04五文档[skip ci]收尾，生产源码同自身完整Windows成功7ccb11e。

### 1. 目标

让阵容、阵型、比赛、预设及工作簿/资料包命令有明确、独立的传输职责；接通原Application/Ports/文件I/O和PG链路，不重复业务校验、事务或状态。

### 2. 现状与来源

04完成35条Teams/Players命令，另19条阵容相关函数暂居catalog四个具名能力文件；exchange.rs集中24条工作簿/资料包命令。本节点完整迁移这43条原命令，结束四个过渡能力owner及旧exchange单文件，不改变命令行为。

### 3. 目标文件与目录

commands/lineups/负责19命令：根目录7条阵容/历史入口，formations/4、matches/2、presets/6；commands/exchange/负责24命令：team_package/4、player_catalog/6、team_monthly/6、match_lineup/6、ai_match_package/2。43条命令各一文件，10个mod.rs只登记/显式导出，共53新Rust文件；无空DTO/mapper/error或单纯为测试再转发的coordinator。

### 4. 文件职责边界

命令仅原State/DTO/UUID转换、原Application调用和错误投影。复杂文件识别、XLSX/JSON读写、冲突/提交、规则与事务继续归既有Exchange/Lineups用例、Ports和PG adapter。catalog根出口只保留04已完成的35命令，35命令及四子目录字节冻结。

### 5. 输入

原State<AppState>、Draft/Query/Request、String/Option<String>/bool/u32及原Domain导入模式、冲突决议、资料包预览。原参数名、optional、默认值和serde保持，不新增请求ID/取消表/状态。

### 6. 输出

43条原Result返回签名、DTO字段与String错误不变；包括阵型/阵容/预设/历史结果、导出摘要、预检/冲突/提交和AI比赛资料包摘要。无类型复制或硬编码结果。

### 7. 允许依赖

原Tauri State、bootstrap::AppState、commands::parse_uuid和原football_domain类型；原State.service公开Application方法。不新增第三方API或库。

### 8. 禁止依赖

命令不直接文件I/O、SQL/PG连接、模型计算、序列化/文件识别、事务或UI状态管理；不新增服务实例/连接池/worker/锁/配置/依赖/迁移/runner/workflow/测试target/数据库专项入口。

### 9. 状态所有权

AppState及原Service/Ports/Config/运行日志所有者字节不变，state-ownership仍19。命令只借用同一State，无新共享状态或副作用owner。UI过期结果/请求标识与监听器仍归原调用方。

### 10. 副作用边界

19命令仍原Application→LineupService/Use Case/4 Ports→既有PG能力；24命令仍原Application→ExchangeService/Use Case→既有Spreadsheet/Monthly/MatchLineup Ports及原文件I/O。传输不新增会话门槛或改变文件校验与数据库错误优先级；预检JSON导出继续原无数据库会话用例。

### 11. 异常路径

原单项UUID、中文标签/空格与前置顺序保持；list_lineups原Option.map(parse_uuid).transpose()?，None、合法Some与非法Some行为不变，不能吞无效ID。remove_lineup_history/read_match_lineup_chain/list_team_match_lineups保留原内联UUID与reason/snapshot_type/limit；预设include_archived/name不变。工作簿原输出/输入路径、mode、resolution、request/preview原样传递，批次ID在原服务调用前解析；所有服务错误仍error.to_string()，不trim路径、替换模式或加连接检查。

### 12. 并发/异步/生命周期

43完整函数正文和公开签名保持，原async/State/await不变；不新增线程、重试、锁、worker、事务或补偿。双阵容父锁/原子写入、预设/history保护、导入批次/行锁、冲突和整批提交仍原唯一底层owner；源码等价不替代这些真实PG运行保证。

### 13. 兼容要求

171命令名称/注册顺序/参数/返回、前端、contracts/schemas、配置/数据/错误/日志、模型、0.23.0、依赖/锁/workflow均保持。公共集合SHA256仍dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669。

内部物理组13→14：catalog由54收敛35，新增lineups组承接原19且保持原相对顺序；exchange仍原24，只同步目录/逐命令定义。其他11组、顶层政策/基线/前端字段精确不变，不能声称整个内部清单字节冻结。定义映射SHA129344d380145a89a5ea8015876ffcd165e3b4847d98edc338d0d309ced0ea10；module-boundaries同步lineups新owner和exchange目录，Domain清单刷新实际使用位置。

### 14. 实施步骤

先核04精确完整Windows与五文档收尾；补原Lineups/Exchange验证器的命令位置、公开签名、原参数/UUID/错误、纯出口与唯一owner断言，确认缺新入口时失败。迁入43原完整函数，接入显式根出口、删除5旧owner、同步正式清单及受影响读者。执行等价/保护/格式/静态及隔离破坏检查，补齐记录后沿原Windows验收。

### 15. 切换入口

commands.rs仅增加mod lineups/pub use lineups::*两行；原171条bootstrap/command_registry.rs字节不变。catalog不再导出19条已迁命令，lineups根/3子目录与exchange根/5子目录显式导出函数和__cmd__/__tauri_command_name_双宏。沿01～04已完整Windows证明的锁定Tauri出口和原API，无需新增兼容层。

### 16. 删除清单

删除catalog/formations.rs、matches.rs、lineups.rs、lineup_presets.rs及commands/exchange.rs。43原命令各唯一owner，不保留旧转发/重复状态/DTO/逻辑。整文件逐字移动/重命名：无；按真实命令职责迁移完整函数。

### 17. 最小验证

最终82/82原只读源码、完整npm run verify:architecture、55现存变更Rust独立Rustfmt1.88 --check（53新增+2既有出口）、161完整等价、隔离副本20破坏拒绝/4相关守卫恢复正向、主工作区162源码/清单指纹不变、18保护资产/171命令14组/46迁移18PG静态、git diff --check通过。

161包括43完整正文/43签名/43原AppState、业务/模型/依赖/状态/既有35命令与01～04冻结、公共契约及11组、两个根出口、10原守卫反向比对；不是动态测试。本节点新增/迁移Rust单测0，沿原workspace/契约targets，预期Desktop46/App230/PG259/Gateway112unit+41contract保持，须05自身Windows确认。

### 18. 阶段回归

沿原Public Platform CI完整frontend/contracts/TypeScript/Vite/17视口、fmt/locked Clippy workspace all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动和日志。82源码排除原Windows进程入口和五浏览器入口；未执行非Windows Cargo/check/Clippy/build/test/前端构建/浏览器/loopback。

### 19. 失败停止条件

05自身完整Windows成功前VERIFYING、06～10 BLOCKED，不继承04成功。保护/公共契约或最小验证失败即继续修复同一R11分支，不宣布阶段完成。Windows job首次实际in_progress后停止全部CI查询，提示约20～30分钟并等用户返回。

### 20. 回退点

9e39ba0c22e5e1c224252b891965718e2c6ebad5，生产源码同04完整成功7ccb11e。按节点提交受控revert命令/登记/清单/守卫/记录，不硬reset、手工复制旧owner或改历史数据；回退源码仍需自身Windows。

### 21. 根 README 摘要记录

记录实际19+24命令独立、catalog收敛与14物理组、171兼容、删除旧owner、静态结果及05自身Windows待验，链接本记录；详细文件清单和失败/延期不堆入根摘要。

### 22. docs 阶段节点详细记录

本记录、索引、任务书和TESTING同步；01～04记录在本次实施中字节冻结。原Lineups/Exchange守卫保留全部业务断言，增加实际薄命令/纯出口/owner/签名/完整传输形状；7个既有源读取入口改读实际定义map。Teams/Players守卫仅退休19条迁出归属，原35命令的签名、UUID/批量/强删等断言不变；正式命令验证器字节冻结。

20隔离探针覆盖旧owner恢复、根登记/双层宏/出口逻辑、Option参数/transpose、内联UUID/标签、include_archived/reason、路径/mode/resolution/批次、原错误、JSON导出错误加会话门槛、定义归属，以及重算内部hash后跨组重复仍拒绝。全部20拒绝，4相关守卫恢复正向，主工作区162源码/清单指纹不变。

首次在主目录做探针后正向复核拦住嵌套宏未恢复，随后又发现JSON导出残留的探针连接门槛；当时未推送。改用独立临时源码副本运行破坏探针，主工作区43正文按成功Git基线恢复核对，重建纯出口并重新通过完整82源码/架构/55格式/等价。副本与临时报告不入repo，不建立新持续回归框架或CI入口，后续破坏检查避免修改交付工作区。

### 23. 完成标准

43唯一传输入口、旧owner清理、记录和最终静态验证完成；05仍VERIFYING，06～10未开始，R11未完成。真实PG/历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。

## 实际能力归属

| 能力 | 命令数 | 实际位置 |
|---|---:|---|
| 阵容/历史/读取链 | 7 | commands/lineups/各命令.rs |
| 阵型 | 4 | commands/lineups/formations/ |
| 比赛 | 2 | commands/lineups/matches/ |
| 阵容预设 | 6 | commands/lineups/presets/ |
| 球队完整资料包 | 4 | commands/exchange/team_package/ |
| 球员目录工作簿 | 6 | commands/exchange/player_catalog/ |
| 球队月度工作簿 | 6 | commands/exchange/team_monthly/ |
| 比赛阵容工作簿 | 6 | commands/exchange/match_lineup/ |
| AI比赛资料包 | 2 | commands/exchange/ai_match_package/ |

## 实际验证与未执行范围

原82只读入口、npm run verify:architecture、node scripts/verify_protected_assets.mjs、node scripts/verify_command_contract.mjs、node scripts/verify_database_baseline.mjs、55文件独立Rustfmt1.88 --edition 2021 --config skip_children=true --check、scratch/r1105_compare.py/隔离目录中的r1105_probes.py、git diff --check。最终报告r1105-static.json、r1105-equivalence.json、r1105-negative.json和主目录指纹记录位于repo外scratch，可从实际源码复建。

Domain365/129声明源/301映射，scan1349→1397；sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，usageDigest22bfb45754826ae27197d6f54ff41ea4f68eed6b4db3bda3f86d5967f2007c73。App422/43Ports/19Port文件、PG直接38、状态19保持；保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57和DB静态aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

没有真实PG双阵容/预设/history或有效XLSX导入/冲突/提交/回滚运行验收；原底层代码、integration targets与断言保留，真实效果最终新库待验。JSON导出无会话路径、文件错误优先与原格式识别均源码保持，不将静态或既有04成功计为05实跑。Mermaid Chart已按真实阵容/工作簿→Application/Ports/原文件I/O/PG链路更新中文图；Create State沿足球稳定UUID96f03270-c236-46de-99fa-db85d2fbf4ce续接。Git制品不另存，无子代理。

## 全部实际文件清单

基于9e39ba0c22e5e1c224252b891965718e2c6ebad5，no-renames口径54A/19M/5D，共78路径。53新增Rust（43命令、10出口），另2既有Rust出口修改；整文件逐字移动/重命名：无，43完整函数按职责迁移。

### 新增（54）

- `docs/modular-rewrite/R11-tauri-commands/R11-05-lineups-presets-and-workbooks-commands.md`
- `src-tauri/src/commands/exchange/ai_match_package/export_ai_match_package.rs`
- `src-tauri/src/commands/exchange/ai_match_package/mod.rs`
- `src-tauri/src/commands/exchange/ai_match_package/preview_ai_match_package.rs`
- `src-tauri/src/commands/exchange/match_lineup/commit_match_lineup_import.rs`
- `src-tauri/src/commands/exchange/match_lineup/export_match_lineup_data.rs`
- `src-tauri/src/commands/exchange/match_lineup/export_match_lineup_template.rs`
- `src-tauri/src/commands/exchange/match_lineup/mod.rs`
- `src-tauri/src/commands/exchange/match_lineup/preview_match_lineup_import.rs`
- `src-tauri/src/commands/exchange/match_lineup/read_match_lineup_import_preview.rs`
- `src-tauri/src/commands/exchange/match_lineup/resolve_match_lineup_import_conflict.rs`
- `src-tauri/src/commands/exchange/mod.rs`
- `src-tauri/src/commands/exchange/player_catalog/commit_player_catalog_import.rs`
- `src-tauri/src/commands/exchange/player_catalog/export_player_catalog_data.rs`
- `src-tauri/src/commands/exchange/player_catalog/export_player_catalog_template.rs`
- `src-tauri/src/commands/exchange/player_catalog/mod.rs`
- `src-tauri/src/commands/exchange/player_catalog/preview_player_catalog_import.rs`
- `src-tauri/src/commands/exchange/player_catalog/read_player_catalog_import_preview.rs`
- `src-tauri/src/commands/exchange/player_catalog/resolve_player_catalog_import_conflict.rs`
- `src-tauri/src/commands/exchange/team_monthly/commit_team_monthly_import.rs`
- `src-tauri/src/commands/exchange/team_monthly/export_team_monthly_data.rs`
- `src-tauri/src/commands/exchange/team_monthly/export_team_monthly_template.rs`
- `src-tauri/src/commands/exchange/team_monthly/mod.rs`
- `src-tauri/src/commands/exchange/team_monthly/preview_team_monthly_import.rs`
- `src-tauri/src/commands/exchange/team_monthly/read_team_monthly_import_preview.rs`
- `src-tauri/src/commands/exchange/team_monthly/resolve_team_monthly_import_conflict.rs`
- `src-tauri/src/commands/exchange/team_package/commit_team_package_import.rs`
- `src-tauri/src/commands/exchange/team_package/export_team_package_preview_json.rs`
- `src-tauri/src/commands/exchange/team_package/export_team_package_template.rs`
- `src-tauri/src/commands/exchange/team_package/mod.rs`
- `src-tauri/src/commands/exchange/team_package/preview_team_package_import.rs`
- `src-tauri/src/commands/lineups/create_lineup.rs`
- `src-tauri/src/commands/lineups/create_lineup_pair.rs`
- `src-tauri/src/commands/lineups/formations/list_formation_usage_distributions.rs`
- `src-tauri/src/commands/lineups/formations/list_formations.rs`
- `src-tauri/src/commands/lineups/formations/mod.rs`
- `src-tauri/src/commands/lineups/formations/resolve_formation_distribution.rs`
- `src-tauri/src/commands/lineups/formations/save_formation_usage_distribution.rs`
- `src-tauri/src/commands/lineups/list_lineups.rs`
- `src-tauri/src/commands/lineups/list_team_match_lineups.rs`
- `src-tauri/src/commands/lineups/matches/create_match.rs`
- `src-tauri/src/commands/lineups/matches/delete_match.rs`
- `src-tauri/src/commands/lineups/matches/mod.rs`
- `src-tauri/src/commands/lineups/mod.rs`
- `src-tauri/src/commands/lineups/presets/archive_team_lineup_preset.rs`
- `src-tauri/src/commands/lineups/presets/delete_team_lineup_preset.rs`
- `src-tauri/src/commands/lineups/presets/duplicate_team_lineup_preset.rs`
- `src-tauri/src/commands/lineups/presets/list_team_lineup_presets.rs`
- `src-tauri/src/commands/lineups/presets/mod.rs`
- `src-tauri/src/commands/lineups/presets/preview_team_lineup_preset_application.rs`
- `src-tauri/src/commands/lineups/presets/save_team_lineup_preset.rs`
- `src-tauri/src/commands/lineups/read_lineup.rs`
- `src-tauri/src/commands/lineups/read_match_lineup_chain.rs`
- `src-tauri/src/commands/lineups/remove_lineup_history.rs`

### 修改（19）

- `README.md`
- `architecture/command-contract.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/README.md`
- `scripts/verify-exchange-service.mjs`
- `scripts/verify-formation-usage.mjs`
- `scripts/verify-history-scoreline-ui.mjs`
- `scripts/verify-lineups-service.mjs`
- `scripts/verify-match-lineup-chain.mjs`
- `scripts/verify-match-workflow-ui.mjs`
- `scripts/verify-monthly-workbooks.mjs`
- `scripts/verify-stage-e2-lineup-presets.mjs`
- `scripts/verify-team-package.mjs`
- `scripts/verify-teams-players-service.mjs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/commands/catalog/mod.rs`

### 删除（5）

- `src-tauri/src/commands/catalog/formations.rs`
- `src-tauri/src/commands/catalog/lineup_presets.rs`
- `src-tauri/src/commands/catalog/lineups.rs`
- `src-tauri/src/commands/catalog/matches.rs`
- `src-tauri/src/commands/exchange.rs`
