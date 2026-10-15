# R11-01 Bootstrap、Workspace 与 Logs Commands

状态：`VERIFYING`。

R11-01 Bootstrap、Workspace 与 Logs Commands已实施 `VERIFYING`。用户2026-10-15“开始R11-01”，uniquenesssta/123 v0.23.0，从R10文档收尾3d43e8ec216f55daa04d5b8afeec68e372ce467c建立唯一 `rewrite/r11-tauri-commands`。9条平台命令已切换唯一owner，公共171命令与注册顺序保持；自身完整Windows待验，02～10 BLOCKED，R11 IN_PROGRESS。

### 1. 目标

将初始化、工作区状态、运行日志和问题日志命令收拢到真实平台边界，保留原状态、恢复连接、副作用顺序、错误与公共IPC。

### 2. 现状与来源

原bootstrap与数据库三条变更命令共处database.rs；workspace.rs含3条状态透传，logging.rs含命令/DTO/校验/2原unit，issues.rs含4命令和被Analytics复用的失败任务投影。原host/bootstrap已是正确组合根，不重写注册器或AppState。

### 3. 目标文件与目录

commands/platform下bootstrap/{command,connection,coordinator,response,tests,mod}.rs、runtime_operation/{command,request,validation,tests,mod}.rs；工作区3条和问题日志4条各具名命令文件，background_jobs.rs为唯一失败任务投影，mod仅登记/导出，tests只测试。bootstrap与runtime为已有多职责的目录升级，简单透传不建空request/mapper/error文件。

### 4. 文件职责边界

bootstrap command仅Tauri State→原桌面协调；connection保留已连接短路/保存配置读取与恢复；coordinator保留恢复→初始化/原两类日志→回包；response只原4字段DTO。runtime command保留输入校验→原日志投影，request只原Deserialize字段，validation只原phase/text政策。其他7条命令继续原Store/Service调用，文件/日志/配置副作用仍各自原Store实现，background_jobs只有原Failed过滤/问题投影。

### 5. 输入

原State<AppState>、workspace Value、RuntimeOperationDraft、IssueLogDraft、limit:Option<u32>和output_path:String。初始化配置、原服务/存储句柄仍来自同一个AppState，无新输入或配置。

### 6. 输出

原BootstrapResponse四字段data/connection_error/config_path/runtime_log_path、Value/()/IssueLogEntry/Vec<IssueLogEntry>及原String错误。DTO字段/serde/显式null及原路径显示保持；内部类型位置调整，不宣称私有Rust模块路径不变。

### 7. 允许依赖

原Tauri State、唯一AppState、ApplicationService、DesktopConfig、IssueLogStore/RuntimeLogStore/WorkspaceStateStore、Domain BackgroundJob/JobStatus及serde/json。没有新生产依赖，临时测试只原tempfile/tokio/unit目标。

### 8. 禁止依赖

无SQL、直接PostgresStore、模型计算、新全局State/连接/锁/任务worker、UI业务、万能dispatcher或新命令宏。Tauri自动生成宏只按锁定版本导出，不自造分派协议。

### 9. 状态所有权

AppState原7字段与host安装保持，19状态owner/43Ports/19Port源不变；原issue/workspace writers清单更新实际路径，未新增状态。Analytics及平台问题列表复用同一background_jobs投影，未复制日志缓冲、队列或数据库会话。bootstrap当前连接仍由原DatabaseService持有。

### 10. 副作用边界

bootstrap先原恢复（已连接立即返回；无配置不连接；配置load仍执行原旧格式迁移），恢复错误只记录原issue/runtime然后继续Application bootstrap；初始化错误记录原critical并返回原String，日志写入失败仍被原let _忽略，不把日志失败改成业务失败。工作区read/write/clear仍原Store，敏感键/1MiB/原子写入保持。

runtime先phase、operation、可选trace校验，随后一次record；failed→error，其他info，subsystem/frontend.operation和operation_phase事件名、duration/context缺失null保持。问题record先尽力运行日志再原问题Store；list仅数据库已连接才读取原500个后台任务，扫描失败尽力记录原后台错误，再按请求limit或500列表；clear/export仍原Store。无新增事务或回读。

### 11. 异常路径

phase trim/ASCII lower与原6值保持，未知错误原字符串；operation/trace按trim后Unicode标量计数160，不改成UTF8字节。配置JSON/版本错误透传，不自动修复；原旧格式由DesktopConfig唯一迁移。问题扫描只Failed且error_message为Some才投影，job_id:attempts继续去重键；重复同attempt不增加次数，重试新attempt可记录。原日志脱敏/文件错误与列表/导出错误传播保持。

### 12. 并发/异步/生命周期

命令仍原sync/async和State借用；没有新取消表、timer、listener、跨请求State或DB事务。初始化恢复/后台扫描的并发保证仍原Service/Store，不承诺跨请求原子。原UI过期结果/监听器由前端持有，本节点不把UI生命周期搬入Repository或命令。

### 13. 兼容要求

171名称/参数/返回和注册顺序、前端client/types、所有contracts/schemas、日志/错误/配置/存储格式/版本0.23.0保持。公共命令集合SHA dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669保持。

依总纲22.4同步architecture/command-contract内部来源：原15物理组变13组，database仅余3命令，platform归9命令；definition_file为出口，command_definition_files逐条定位真实文件，原验证器读取实际定义和签名位置；仅内部定义映射SHA变60bb424a490da2908aacbaee125f19cb7c81f76793f631c56f2d6b765ed78a83。其余顶层字段/兼容政策/基线/前端签名位置保持，不将内部路径变化误说成contract文件bytes冻结或公开协议变更。

### 14. 实施步骤

从3d43e8e新建本唯一阶段分支；提取bootstrap职责/9命令及原DTO/校验/共享投影，切根出口与Analytics；移除3旧owner，数据库configure/disconnect/reset完整正文保持；补11原unit，更新实际契约/状态writers/使用摘要与原验证器，完成静态/格式/完整等价/破坏恢复后推送原Windows。

### 15. 切换入口

commands.rs仅以platform出口接入原commands::name，注册表逐字保持。Tauri2.11.1/Cargo.lock tauri-macros2.6.3，Context7返回dev示例后另查官方tauri-macros-v2.6.3真实wrapper源码，确认生成__cmd__name和__tauri_command_name_name；每层具名显式导出函数及两宏，无自定义dispatcher。旧DTO不复制，State仍组合根唯一来源。

### 16. 删除清单

commands/{workspace,logging,issues}.rs删除；database.rs的bootstrap/Response/恢复helper迁出；旧helper来源import已切换。3D，21新增Rust文件由原真实职责接管；无整文件逐字移动/重命名，无旧转发壳或空DTO。

### 17. 最小验证

82/82原只读源码守卫、完整npm run verify:architecture、24现存变更Rust独立Rustfmt1.88 --check、33等价核对与16负向拒绝/bytes恢复通过。18保护资产、171命令两契约、46迁移/18PG静态保持。原Tauri target新增11 unit（5连接/回包，4运行日志输入，2失败任务投影），迁移2原unit；预期Desktop38=27+11、Application230、Persistence259、Gateway112unit+41contract，须自身Windows实际确认。

### 18. 阶段回归

复用原Public Platform CI：完整frontend/contracts/TypeScript/Vite/17视口、fmt/locked Clippy all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动/运行日志。82源码排除原Windows进程与五浏览器入口；未运行非Windows Cargo/check/Clippy/build/test/前端构建/浏览器/loopback，不新增runner/workflow/target/迁移/数据库专项设施。

### 19. 失败停止条件

本项自身精确完整Windows成功前VERIFYING；R10成功不替代01新代码成功，02～10保持BLOCKED。Windows job实际in_progress即停止所有CI查询并提示约20～30分钟；失败仅修本唯一R11分支，不创建第二线或提前DONE。

### 20. 回退点

3d43e8ec216f55daa04d5b8afeec68e372ce467c（R10纯文档收尾，源码同已验4ca43ad）。确认用户改动后受控revert本节点，入口/owner/宏/元数据/使用清单/守卫/tests/docs成组恢复；不硬reset、复制旧模块或改历史数据。源码回退仍需原Windows自身验收。

### 21. 根 README 摘要记录

根README只记录平台命令唯一职责、原恢复/日志/公共接口保持、11新增unit/静态通过及自身Windows待验，链接本节点/阶段索引。

### 22. docs 阶段节点详细记录

本记录与阶段索引、任务书和TESTING同步，R11 IN_PROGRESS/01VERIFYING/02～10BLOCKED，无R11阶段完成记录。实际完整A/M/D列下方。33等价覆盖8原完整命令正文、bootstrap协调逆切/保存连接完整正文、3后续DB命令、两原纯校验/两原unit、共享投影、2完整wire DTO、所有冻结源码/171公共契约/前端签名/11未改组与状态owner。

16破坏探针覆盖旧owner复活、命令/宏/签名缺失、恢复短路/调用、phase/trace校验、错误等级、默认limit/断连扫描、Failed过滤/attempt键、测试登记、目录出口业务和重复定义；探针首次发现phase缺失时indexOf(-1)假阳性，已补显式存在性门禁，最终全部拒绝且恢复原bytes。scratch比较/探针/报告不入仓库，不是新持续回归框架。

### 23. 完成标准

职责/入口/清理/记录与静态验证完成，01自身完整Windows成功后方可DONE；当前VERIFYING。真实PostgreSQL、历史保护四项、不可变账本/并发回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。

## 验证命令与边界

scratch/r1101_static.py；npm run verify:architecture；verify_command_contract.mjs及verify-command-contract.mjs/verify-tauri-bootstrap.mjs；verify_protected_assets.mjs/verify_database_baseline.mjs；独立Rustfmt1.88 --edition2021 --config skip_children=true --check；scratch/r1101_compare.py/r1101_probes.py；git diff --check。r1101-static-checks.json/equivalence.json/negative-probes.json/architecture.log为可复建临时报告。

App422/43Ports/19Port源、PG直接38/state19保持；Domain365/129声明/301映射，扫描1272→1290，声明摘要217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，usageDigestb74de8409500a1ea84ce2983ad23914f7c4afa3eac995bc6a4021f27654aa3b0。全部crates、模型/公式/参数、UI、host启动与各Store、Schema、46迁移、依赖/锁、原workflow冻结。18保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57；DB静态aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93。

Context7仅核实当前宏出口问题，最终以锁定2.6.3官方源码为依据：https://github.com/tauri-apps/tauri/blob/tauri-macros-v2.6.3/crates/tauri-macros/src/command/wrapper.rs 。Mermaid Chart中文真实链已更新，区分初始化/原Store/失败任务投影，未画待实现业务。Create State使用稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce交接。附件只scratch读取，Git制品不另存，无子代理。

## 全部实际文件清单

以3d43e8e基线git diff --no-renames及新增文件生成；实际22A/16M/3D，共41路径。职责迁移不是整文件逐字移动，移动/重命名：无。

### 新增（22）

- `docs/modular-rewrite/R11-tauri-commands/R11-01-bootstrap-workspace-and-logs-commands.md`
- `src-tauri/src/commands/platform/background_jobs.rs`
- `src-tauri/src/commands/platform/bootstrap/command.rs`
- `src-tauri/src/commands/platform/bootstrap/connection.rs`
- `src-tauri/src/commands/platform/bootstrap/coordinator.rs`
- `src-tauri/src/commands/platform/bootstrap/mod.rs`
- `src-tauri/src/commands/platform/bootstrap/response.rs`
- `src-tauri/src/commands/platform/bootstrap/tests.rs`
- `src-tauri/src/commands/platform/clear_issue_logs.rs`
- `src-tauri/src/commands/platform/clear_workspace_state.rs`
- `src-tauri/src/commands/platform/export_issue_logs.rs`
- `src-tauri/src/commands/platform/list_issue_logs.rs`
- `src-tauri/src/commands/platform/mod.rs`
- `src-tauri/src/commands/platform/read_workspace_state.rs`
- `src-tauri/src/commands/platform/record_issue.rs`
- `src-tauri/src/commands/platform/runtime_operation/command.rs`
- `src-tauri/src/commands/platform/runtime_operation/mod.rs`
- `src-tauri/src/commands/platform/runtime_operation/request.rs`
- `src-tauri/src/commands/platform/runtime_operation/tests.rs`
- `src-tauri/src/commands/platform/runtime_operation/validation.rs`
- `src-tauri/src/commands/platform/save_workspace_state.rs`
- `src-tauri/src/commands/platform/tests.rs`

### 修改（16）

- `README.md`
- `architecture/command-contract.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `architecture/state-ownership.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/README.md`
- `scripts/verify-api-runtime-diagnostics.mjs`
- `scripts/verify-searchable-select-diagnostics.mjs`
- `scripts/verify-tauri-bootstrap.mjs`
- `scripts/verify-workspace-ui.mjs`
- `scripts/verify_command_contract.mjs`
- `src-tauri/src/commands.rs`
- `src-tauri/src/commands/analytics.rs`
- `src-tauri/src/commands/database.rs`

### 删除（3）

- `src-tauri/src/commands/issues.rs`
- `src-tauri/src/commands/logging.rs`
- `src-tauri/src/commands/workspace.rs`
