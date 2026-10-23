# R11-03 Competitions 与 Rules Commands

状态：`DONE`（自身精确849f1a4 / Windows run39930230184完整SUCCESS；下文VERIFYING及预期计数为实施历史）。

R11-01、02 DONE；03 Competitions与Rules Commands已实施，状态VERIFYING；04～10 BLOCKED，R11 IN_PROGRESS。基线df3d3cc8461a54da12211b068441282a204331c4为02五文档[skip ci]收尾，生产源码同自身完整Windows成功a2341de。用户2026-10-23“收尾02开始03”，继续唯一rewrite/r11-tauri-commands。

### 1. 目标

将赛事层级写入、赛事删除、规则包登记和规则绑定分别收敛为明确的命令传输入口；保留所有真实Application、Port和持久化调用。

### 2. 现状与来源

原commands/competition.rs集中七条薄命令。赛事五操作和规则两操作具有不同Application服务与Ports所有者；本节点仅重组命令层，不重复已经独立的业务规则、事务、映射或状态。

### 3. 目标文件与目录

采用commands/competition/，根mod.rs显式导出五条赛事层级命令及rules子目录出口；rules/mod.rs登记规则包和绑定两条命令。每条简单命令一文件，无空request/response/mapping/error，也不为单次真实调用添加转发coordinator。

### 4. 文件职责边界

七个命令文件只接收原State/DTO、执行原传输校验、调用原Application方法并转换原错误；两个mod.rs只登记与显式导出原函数和Tauri双宏。原业务校验、标识生成、规则模型注册、删除约束和审计继续各归原用例/Port/PG adapter。

### 5. 输入

原State<AppState>；六条命令接收各自原Domain Draft，delete_competition接收原competition_id:String。字段、参数名、默认值和serde均不变，不复制DTO。

### 6. 输出

原CompetitionRecord、SeasonRecord、StageRecord、RoundRecord、RulePackageSummary、CompetitionBindingSummary及删除Result<(),String>，错误均原String投影。

### 7. 允许依赖

原Tauri State、bootstrap::AppState、原football_domain输入/输出类型；删除入口复用crate::commands::parse_uuid。全部Application调用仍借用AppState已有service句柄。

### 8. 禁止依赖

不增加SQL、PG直连、模型计算、服务实例、连接池、状态、取消表、后台任务或依赖；不引入新的runner/workflow/测试target/数据库入口。

### 9. 状态所有权

AppState/Service/Ports及state-ownership文件字节不变。命令没有新持久状态；UI请求标识、过期结果和监听器继续归原调用方，本次不将模板套入七个简单异步函数。

### 10. 副作用边界

七条命令原样调用Application公开门面，后者先获取原活动会话，再委托原CompetitionService或RulesService/Use Case/Port/PG adapter。命令不复制校验或执行事务，不改变副作用顺序。

### 11. 异常路径

删除赛事仍先parse_uuid(&competition_id, "赛事 ID")?，无效UUID在进入Application前失败，原错误标签中的空格也保持；合法UUID继续原删除调用。其余六Draft直接传入原服务，所有服务错误仍error.to_string()，不新增trim、默认值、吞错、重试或硬编码成功。

### 12. 并发/异步/生命周期

原async、State借用和七条await保持，无新增锁/worker/线程/事件或补偿。真实数据库的并发、事务、历史保护及取消边界仍由原Application/PG实现承担；本节点源码等价不是这些真实行为的动态验收。

### 13. 兼容要求

171命令的名称、注册顺序、公开签名、前端调用/DTO/Schema/配置/数据/错误/日志/模型/依赖/锁文件均保持。公共集合SHA256仍dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669。

architecture/command-contract只同步competition物理出口/七条command_definition_files和内部定义映射SHA3733217ec51303de1ea73725dffa5559e16e0babc8c6eb046118e1206eb66fdc；物理组仍13，其他12组/顶层政策/前端字段精确不变。module-boundaries仅competition owner改目录mod.rs，Domain清单只刷新真实扫描与使用位置。

### 14. 实施步骤

先核对02精确Windows成功并完成五文档收尾；将原专项验证器改为读取七个目标并补齐完整传输/宏/归属断言，确认缺失新入口时失败。再迁入原完整命令签名和正文，切换唯一目录owner、删除旧文件并同步正式清单，执行静态/格式/等价与破坏探针，交原Windows CI。

### 15. 切换入口

commands.rs原mod competition/pub use competition::*及bootstrap/command_registry.rs逐字不变。目录名沿已有competition模块标识，避免无必要改名；任务书competitions/为目录示例，规则职责单列rules/。根与rules两层显式导出原函数、__cmd__及__tauri_command_name_宏，沿01/02自身Windows已证明的锁定Tauri出口策略，无新第三方API/版本行为。

### 16. 删除清单

删除src-tauri/src/commands/competition.rs。原七条实现各仅一份，没有旧转发入口、重复规则、重复类型或兼容状态。整文件逐字移动/重命名：无；七条完整函数按真实职责迁移。

### 17. 最小验证

已通过82/82原只读源码入口、完整npm run verify:architecture、9个新增Rust文件独立Rustfmt1.88 --check、38项完整等价核对、17项破坏探针拒绝且全部bytes恢复；保护资产18/171命令/46迁移及18PG静态契约与git diff --check通过。

本次新增Rust单测0、迁移原单测0；七条本已是薄适配，不引入仅为测试再转发的执行层。原Application用例单测和原命令契约继续由既有Windows workspace target/验收覆盖。预期Desktop46、Application230、Persistence259、Gateway112unit+41contract保持，须03自身精确Windows确认；源码守卫不是新增动态单测。

### 18. 阶段回归

沿原Public Platform CI，完整frontend/contracts/TS/Vite/17视口、Rust fmt/locked Clippy workspace all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动与日志。82源码排除原Windows进程入口及五个浏览器入口，交原Windows执行；没有执行非Windows Cargo/check/Clippy/build/test/前端构建/浏览器/loopback。

### 19. 失败停止条件

03自身完整Windows成功前保持VERIFYING，04～10 BLOCKED；保护资产或公共契约不兼容立即修复，不能借02成功进入下一节点。原Windows job首次实际in_progress后停止所有CI查询，提示约20～30分钟，等待用户返回。

### 20. 回退点

df3d3cc8461a54da12211b068441282a204331c4；仅文档收尾，生产源码同完整成功a2341de。按节点提交受控revert命令/守卫/清单/记录，不硬reset，不复制旧实现造成双owner或改变数据库历史；回退源码自身仍需原Windows验证。

### 21. 根 README 摘要记录

记录实际七命令/规则子目录、原调用与错误不变、旧入口删除、静态结果及自身Windows待验，链接本记录；不把静态检查或02动态成功替代03动态验收。

### 22. docs 阶段节点详细记录

本记录、阶段索引、任务书和TESTING同步；01/02记录在本节点字节冻结。38等价核对包括七条完整正文/签名/原AppState、全部冻结业务/模型/UI/DTO/依赖/CI/既有节点、公共契约及其他12组、state-ownership和原专项验证器全部断言。

17破坏探针覆盖旧owner恢复、根命令宏/规则名称宏缺失、mod承载逻辑、子模块漏登记、公开参数改名、删除UUID门槛/精确错误标签、原错误投影、六条Draft真实调用、定义映射和模块边界归属；全部拒绝，所有文件bytes恢复并再次正向通过。临时比较/探针/报告仅为本轮源码审查，不加入repo持续回归框架。

### 23. 完成标准

唯一职责、入口切换、旧owner清理、记录和静态验证完成；03当前VERIFYING，必须自身完整Windows成功才能DONE，R11未完成。真实PG/历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。

## 七条命令的实际归属

路径均相对src-tauri/src/commands/competition/。

| 命令 | 文件 | 原Application调用 |
|---|---|---|
| create_competition | create_competition.rs | create_competition(draft) |
| delete_competition | delete_competition.rs | 原UUID解析后delete_competition(competition_id) |
| create_season | create_season.rs | create_season(draft) |
| create_stage | create_stage.rs | create_stage(draft) |
| create_round | create_round.rs | create_round(draft) |
| register_rule_package | rules/register_rule_package.rs | register_rule_package(draft) |
| create_competition_binding | rules/create_competition_binding.rs | create_competition_binding(draft) |

## 验证与真实待验

执行原82只读源码入口、npm run verify:architecture、node scripts/verify_protected_assets.mjs、node scripts/verify_command_contract.mjs、node scripts/verify_database_baseline.mjs；独立Rustfmt1.88 --edition 2021 --config skip_children=true --check检查9个新增Rust文件；scratch/r1103_compare.py、scratch/r1103_probes.py和git diff --check。临时报告r1103-static.json、r1103-equivalence.json、r1103-negative.json在repo外scratch可复建。

Domain365/129声明源/301映射，scan1298→1306；声明摘要217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，使用摘要65c552d254848eddd51c255c5c16e4be7910b5089b137eee88dabc86769a5dd7。App422/43Ports/19Port文件、state19、PG直接38保持；保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57，DB静态aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

没有真实创建赛事层级/删除历史实体/规则登记与绑定写库验收；原PG集成targets存在但ignored不得计实跑。最终新库按已有入口核对这些真实链路及历史保护，不新增专项入口。Mermaid Chart已按实际UI→七命令/原UUID→Application/Ports/PG关系更新中文图；复用锁定版本已有出口证据，无新库API不触发重复文档查询。Create State沿足球稳定UUID96f03270-c236-46de-99fa-db85d2fbf4ce交接。

## 全部实际文件清单

以df3d3cc8461a54da12211b068441282a204331c4为基线，no-renames口径10A/8M/1D，共19路径。整文件逐字移动/重命名：无；七条完整函数职责迁移。

### 新增（10）

- `docs/modular-rewrite/R11-tauri-commands/R11-03-competitions-and-rules-commands.md`
- `src-tauri/src/commands/competition/create_competition.rs`
- `src-tauri/src/commands/competition/create_round.rs`
- `src-tauri/src/commands/competition/create_season.rs`
- `src-tauri/src/commands/competition/create_stage.rs`
- `src-tauri/src/commands/competition/delete_competition.rs`
- `src-tauri/src/commands/competition/mod.rs`
- `src-tauri/src/commands/competition/rules/create_competition_binding.rs`
- `src-tauri/src/commands/competition/rules/mod.rs`
- `src-tauri/src/commands/competition/rules/register_rule_package.rs`

### 修改（8）

- `README.md`
- `architecture/command-contract.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/README.md`
- `scripts/verify-competition-rules-service.mjs`

### 删除（1）

- `src-tauri/src/commands/competition.rs`

## R11-03 Windows 完成与04授权（2026-10-23）

精确提交 `849f1a481e23bb48222aa5587f6d687332a334b0` / tree `6f75655aea59a1be89d46e5e5ffbf31d7a349857`，[Public Platform CI run39930230184](https://github.com/uniquenesssta/123/actions/runs/39930230184) / Windows job120336387937完整SUCCESS，run于2026-10-23 17:43:37北京时间完成。实际Desktop46/46、Application230/230、Persistence259/259、Gateway112/112 unit和41/41 contract；完整frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动与日志均通过（7条记录、3个完成操作）。本节点新增单测0、迁移单测0，未将静态等价核对计作动态测试。

报告logs/windows-acceptance-20261023-091359.json及同名.txt；artifact12492991566，14,052,485字节，SHA-256 `32b5d76068caa3fc1628c9d549b6833836a08ee229a6e5aa443b9b1e405b2411`，查询时未过期。已核对精确head/branch、全部job步骤、完整日志与artifact。用户2026-10-23“收尾03开始04”，03 DONE，04 Teams与Players Commands READY，05～10 BLOCKED，R11 IN_PROGRESS，继续唯一rewrite/r11-tauri-commands。本次仅5份既有文档[skip ci]收尾，源码沿用上述自身完整Windows成功，不重复同源码构建；历史VERIFYING/预期计数保留，当前状态以阶段索引为准，04须自身完整Windows。

真实PG/历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。
