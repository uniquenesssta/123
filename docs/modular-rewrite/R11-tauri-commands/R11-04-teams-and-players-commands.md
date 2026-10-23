# R11-04 Teams 与 Players Commands

状态：`VERIFYING`（自身完整Windows尚待验收）。

R11-01～03 DONE；04 Teams与Players Commands已实施VERIFYING，05～10 BLOCKED，R11 IN_PROGRESS。用户2026-10-23“收尾03开始04”，继续唯一rewrite/r11-tauri-commands；基线d1e9b7d01df6c4225876c90a15ce246dcabae80a为03五文档[skip ci]收尾，生产源码同自身完整Windows成功849f1a4。

### 1. 目标

将球队、球员、教练和实体引用的35条命令收敛为明确的传输入口；保留原Application/Ports/真实存储路径，尤其是UUID错误顺序和强制删除的非Send线程隔离。

### 2. 现状与来源

原commands/catalog.rs共54条命令，其中35条由原Teams/Players能力承担，另19条为后续R11-05的阵型、比赛、阵容和预设能力。直接保留旧文件会造成模块入口冲突或混合owner；本节点将这19条完整函数原样放到四个具名能力文件，保留05后续拆分/工作簿及自身验收，不提前标记05完成。

### 3. 目标文件与目录

commands/catalog/mod.rs是唯一根出口；teams/10、players/14、coaches/5、entities/6分别一命令一文件，各子mod.rs只登记与显式导出。formations.rs/4、matches.rs/2、lineups.rs/7、lineup_presets.rs/6为剩余原命令的四个具名owner。共44个新增Rust文件，无空request/response/mapping/error或单纯为测试增加的转发层。

### 4. 文件职责边界

35命令文件承担各自原State/DTO转换、调用与错误投影；两个强删文件分别保留原Tauri运行时线程切换，属于各自传输执行边界，不复制事务。五个mod.rs只登记/显式导出函数与双宏。Application校验、注册、引用预检、归档、生命周期和PG事务/审计均维持原唯一所有者。

### 5. 输入

原State<AppState>、各原Domain Draft/Query/Request和原String/Option/String数组/limit；不新增参数、字段、默认值、请求ID、取消表或状态。

### 6. 输出

54条原Result返回类型与字段不变，包含列表、明细、批量结果和强删预检/执行结果；无本地DTO复制，错误仍原String。

### 7. 允许依赖

原Tauri State/async_runtime、原bootstrap::AppState、原Domain类型与commands::parse_uuid；原AppState.service句柄和公开Application方法。不引入新的第三方API。

### 8. 禁止依赖

命令不执行SQL/直连PG/模型计算，不创建新ApplicationService、连接池、worker、锁或状态；不新增生产依赖/迁移/配置、CI workflow/runner/测试target/数据库专项入口。

### 9. 状态所有权

AppState及Service/Ports/Config/运行日志所有者字节不变，state-ownership保持19。所有命令继续借用同一State，强删在线程切换前克隆原服务Arc取得拥有所有权的句柄，不将State借用移入阻塞闭包。

### 10. 副作用边界

普通命令仍调用原Application门面与Teams/Players/Lineups服务，经既有Ports和PG adapter执行。强删预检先解析原team_id，再取得服务Arc和Tauri handle；强删执行原样移动原request。两者都沿原Tauri spawn_blocking(move || runtime.block_on(...))，保持原Application组合适配器内部的Tokio阻塞桥接。两层既有边界均不移除、不另加一层。

### 11. 异常路径

单项UUID继续调用原parse_uuid，标签和空格保持，失败先于服务/线程启动。bulk_archive_entities、bulk_delete_players、bulk_delete_teams仍iter→map原UUID→collect::<Result<Vec<_>,_>>()?后再调用服务；保留顺序、重复项、空数组传递与首个错误，不能filter_map跳过坏ID或提交部分列表。所有服务错误仍error.to_string()；强删保留两个不同“任务执行失败”join错误上下文和原await/?传播。不trim请求或伪造成功。

### 12. 并发/异步/生命周期

54完整函数正文/签名保持；无新重试、任务取消或UI生命周期。强删既有非Send Future必须由原拥有所有权的闭包和运行时驱动，不能改成普通spawn或捕获State；真实锁、事务、完整名称确认和历史清除仍原PG实现。并发/回滚与最终新库仍待真实验证。

### 13. 兼容要求

171公共命令名称/注册顺序/签名、DTO/serde、前端、contracts/schemas、配置、日志、错误、数据、模型、版本0.23.0及依赖/锁/workflow保持。command_set_sha256仍dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669。

architecture/command-contract仅catalog根/54定义路径与内部定义映射SHA3c083de8e5b0bdc565b065db286d44005ac3506bac9b77f4bd2d888887940f3f更新；物理组仍13，其他12组/顶层政策/前端字段精确不变。module-boundaries仅catalog owner改mod.rs，Domain清单刷新真实使用位置。

### 14. 实施步骤

核对03自身完整Windows并五文档收尾；先补原Teams/Players守卫的新入口与签名/UUID/线程/owner断言，确认缺少新文件时失败。迁入35条独立命令与19条必要的原样能力归位，显式双层导出并删除旧catalog.rs。修正受影响的原源码读取与正式定义扫描，完成静态/格式/等价/负向/保护检查后交原Windows。

### 15. 切换入口

commands.rs原mod catalog/pub use catalog::*与bootstrap/command_registry.rs逐字不变，文件系统同名目录接管模块。根和四个子目录显式导出原函数、__cmd__及__tauri_command_name_双宏。使用01～03已自身Windows证明的锁定Tauri出口及原运行时调用；无新增API或版本变化，不重复查询无关文档。

### 16. 删除清单

删除src-tauri/src/commands/catalog.rs。54原定义各只有一个新owner，无旧转发壳、重复状态/类型/逻辑。35条按独立命令承担，19条只做后续能力的必要原样归位；整文件逐字移动/重命名：无。

### 17. 最小验证

82/82原只读JS入口、完整npm run verify:architecture、44新增Rust独立Rustfmt1.88 --check、189完整等价核对、20负向拒绝并恢复全部bytes、18模型保护资产、171命令13组、46迁移18PG静态契约、git diff --check均通过。189包括54完整正文/54签名/54原AppState身份、冻结资产/契约/状态及11个原守卫反向等价；不是189项动态测试。

本次新增/迁移Rust单测均0；原薄命令与线程桥接完整保留，不为测试再加转发层。预期原Desktop46/App230/PG259/Gateway112unit+41contract保持，必须04自身Windows确认；原单测和集成target继续沿既有入口，真实PG未实跑。

### 18. 阶段回归

原Public Platform CI完整frontend/contracts/TS/Vite/17视口、Rust fmt/locked Clippy workspace all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动及日志。82源码排除原Windows进程入口及五个浏览器动态入口；未执行非Windows Cargo/check/Clippy/build/test/前端build/浏览器/loopback。

### 19. 失败停止条件

04自身完整Windows成功前VERIFYING、05～10 BLOCKED；不能继承03成功。保护或公共契约不兼容即修复，失败继续同一R11分支。Windows job首次实际in_progress即停止全部CI查询，提示约20～30分钟并等待用户返回，无阶段完成记录。

### 20. 回退点

d1e9b7d01df6c4225876c90a15ce246dcabae80a，生产源码同03完整成功849f1a4。受控revert本节点命令/清单/守卫/记录，不硬reset或复制旧模块形成双owner，不改历史数据库。回退源码仍需自身Windows结果。

### 21. 根 README 摘要记录

记录35条命令职责、19条必要原样归位、原UUID/批量/非Send与错误保持、删除旧入口、静态结果和自身Windows待验；完整变更与风险见本节点，不把静态证据算动态PASS。

### 22. docs 阶段节点详细记录

本记录与阶段索引/任务书/TESTING同步，01～03节点在本次实现中字节冻结。全部实际路径见清单。原10个读取catalog.rs的验证器改读准确新owner，原断言完整保留；强删守卫直接读两个完整命令，替代按旧混合文件后续函数位置切片。历史team-player-management-contract的旧artifact项字节不改，由原验证器检查旧owner不存在、全部真实定义owner存在，不能直接跳过检查。

正式verify_command_contract.mjs按command_definition_files先把命令归到实际文件再扫描一次；既支持35个单命令文件，也检查四个既有能力文件中的19条命令，避免把同一文件重复算多次。仍严格核对文件内命令集合、全局重复/缺失、三方集合/注册顺序、内部映射摘要与13组；未关闭任何原检查。当前12个其他组同时验证原单文件模式和已有单命令映射模式。

20负向覆盖旧owner、双层宏/子登记/mod逻辑、公开参数、单项UUID/批量坏ID/顺序/错误标签、强删线程/State捕获/请求/join上下文、真实调用、边界/定义owner，以及共享文件漏定义、重复定义、映射错且重算hash。全部被拒绝，所有bytes恢复且三个相关守卫正向复核通过；临时脚本/报告仅scratch审查，不入repo框架。

### 23. 完成标准

本节点35命令唯一职责、54原函数等价、原入口切换/清理、记录与静态完成；04仍VERIFYING，05～10未开始，R11未完成。真实PG/历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。

## 实际能力归属

路径均相对src-tauri/src/commands/catalog/。

| 能力 | 命令数 | 实际文件 | 本节点范围 |
|---|---:|---|---|
| 球队 | 10 | teams/各命令.rs及mod.rs | 独立传输入口，含强删预检/执行 |
| 球员 | 14 | players/各命令.rs及mod.rs | 独立传输入口 |
| 教练 | 5 | coaches/各命令.rs及mod.rs | 独立传输入口 |
| 实体引用/匹配/归档/提供者 | 6 | entities/各命令.rs及mod.rs | 独立传输入口 |
| 阵型 | 4 | formations.rs | 原完整函数必要归位，05继续 |
| 比赛 | 2 | matches.rs | 原完整函数必要归位，05继续 |
| 阵容及历史 | 7 | lineups.rs | 原完整函数必要归位，05继续 |
| 阵容预设 | 6 | lineup_presets.rs | 原完整函数必要归位，05继续 |

## 实际验证与未执行范围

执行原82只读JS、npm run verify:architecture、node scripts/verify_protected_assets.mjs、node scripts/verify_command_contract.mjs、node scripts/verify_database_baseline.mjs及44文件独立Rustfmt1.88 --edition 2021 --config skip_children=true --check；scratch/r1104_compare.py、scratch/r1104_probes.py与git diff --check。临时报告r1104-static.json、r1104-equivalence.json、r1104-negative.json在repo外scratch可复建。

Domain365/129声明源/301映射、scan1306→1349，sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，usageDigest1b1a78eda0e17bf3f1f9eb63484e4972a25ae77d8b3670d5eb4b0857be64db95。App422/43Ports/19Port文件、PG直接38、状态19保持；保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57与DB静态aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

没有真实PG实体创建/改名/关系写入/引用预检/归档/批删/强删和线程失败运行验收；只能在04原Windows及最终新库链路验证相应效果，不能以原PG ignored、源码字符串或03成功代替。私有模型、有效XLSX和其他最终新库项目保持待验。

Mermaid Chart已更新中文目录/UUID→Application/Ports与原Tauri、Tokio两层强删阻塞桥接图。Create State使用足球稳定UUID96f03270-c236-46de-99fa-db85d2fbf4ce续接；Git制品不另存，无子代理。

## 全部实际文件清单

基于d1e9b7d01df6c4225876c90a15ce246dcabae80a，no-renames口径45A/18M/1D，共64路径。44新增Rust文件（35独立命令、5出口、4原样能力文件）；整文件逐字移动/重命名：无，54完整函数按上述职责迁移。

### 新增（45）

- `docs/modular-rewrite/R11-tauri-commands/R11-04-teams-and-players-commands.md`
- `src-tauri/src/commands/catalog/coaches/add_coach_name.rs`
- `src-tauri/src/commands/catalog/coaches/add_team_coach_period.rs`
- `src-tauri/src/commands/catalog/coaches/create_coach.rs`
- `src-tauri/src/commands/catalog/coaches/list_coaches.rs`
- `src-tauri/src/commands/catalog/coaches/mod.rs`
- `src-tauri/src/commands/catalog/coaches/read_coach.rs`
- `src-tauri/src/commands/catalog/entities/add_external_entity_id.rs`
- `src-tauri/src/commands/catalog/entities/bulk_archive_entities.rs`
- `src-tauri/src/commands/catalog/entities/check_entity_deletion.rs`
- `src-tauri/src/commands/catalog/entities/create_data_provider.rs`
- `src-tauri/src/commands/catalog/entities/list_entity_references.rs`
- `src-tauri/src/commands/catalog/entities/mod.rs`
- `src-tauri/src/commands/catalog/entities/resolve_entity_reference.rs`
- `src-tauri/src/commands/catalog/formations.rs`
- `src-tauri/src/commands/catalog/lineup_presets.rs`
- `src-tauri/src/commands/catalog/lineups.rs`
- `src-tauri/src/commands/catalog/matches.rs`
- `src-tauri/src/commands/catalog/mod.rs`
- `src-tauri/src/commands/catalog/players/add_player_ability_observation.rs`
- `src-tauri/src/commands/catalog/players/add_player_availability.rs`
- `src-tauri/src/commands/catalog/players/add_player_dynamic_tag.rs`
- `src-tauri/src/commands/catalog/players/add_player_name.rs`
- `src-tauri/src/commands/catalog/players/add_player_team_period.rs`
- `src-tauri/src/commands/catalog/players/assign_player_position.rs`
- `src-tauri/src/commands/catalog/players/bulk_delete_players.rs`
- `src-tauri/src/commands/catalog/players/calculate_player_match_contribution.rs`
- `src-tauri/src/commands/catalog/players/create_player.rs`
- `src-tauri/src/commands/catalog/players/delete_player.rs`
- `src-tauri/src/commands/catalog/players/list_players.rs`
- `src-tauri/src/commands/catalog/players/mod.rs`
- `src-tauri/src/commands/catalog/players/player_catalog_reference_data.rs`
- `src-tauri/src/commands/catalog/players/read_player.rs`
- `src-tauri/src/commands/catalog/players/update_player.rs`
- `src-tauri/src/commands/catalog/teams/add_team_name.rs`
- `src-tauri/src/commands/catalog/teams/bulk_delete_teams.rs`
- `src-tauri/src/commands/catalog/teams/create_team.rs`
- `src-tauri/src/commands/catalog/teams/force_delete_team.rs`
- `src-tauri/src/commands/catalog/teams/list_team_options.rs`
- `src-tauri/src/commands/catalog/teams/list_teams.rs`
- `src-tauri/src/commands/catalog/teams/mod.rs`
- `src-tauri/src/commands/catalog/teams/preview_force_delete_team.rs`
- `src-tauri/src/commands/catalog/teams/read_team.rs`
- `src-tauri/src/commands/catalog/teams/update_team.rs`
- `src-tauri/src/commands/catalog/teams/upsert_team_profile.rs`

### 修改（18）

- `README.md`
- `architecture/command-contract.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/README.md`
- `scripts/verify-entity-relationships.mjs`
- `scripts/verify-force-team-delete.mjs`
- `scripts/verify-formation-usage.mjs`
- `scripts/verify-history-scoreline-ui.mjs`
- `scripts/verify-lineups-service.mjs`
- `scripts/verify-match-lineup-chain.mjs`
- `scripts/verify-match-workflow-ui.mjs`
- `scripts/verify-stage-e2-lineup-presets.mjs`
- `scripts/verify-team-player-management.mjs`
- `scripts/verify-teams-players-service.mjs`
- `scripts/verify_command_contract.mjs`

### 删除（1）

- `src-tauri/src/commands/catalog.rs`
