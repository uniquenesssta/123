# R11-02 Database Commands

状态：`VERIFYING`。R11-02 Database Commands已实施 `VERIFYING`，基线f56a1c004cee8018f657b6cdd9fa5389a81dec20（01五文档[skip ci]收尾，源码同自身完整Windows成功e56d21f）。用户2026-10-23“已通过收尾01开始02”，继续唯一rewrite/r11-tauri-commands；01 DONE、02 VERIFYING、03～10 BLOCKED，R11 IN_PROGRESS。

### 1. 目标

让数据库配置、断开和重置三条命令有唯一传输入口和明确的桌面生命周期职责，保持原真实服务、配置/凭据存储、错误和日志顺序。

### 2. 现状与来源

R11-01已移走bootstrap，旧commands/database.rs仍持有三个独立流程；configure/reset混合Tauri输入和较长桌面协调，disconnect只有保存空配置与服务断开。原Application数据库生命周期、PG原重置/强确认和Config/RuntimeLogStore已各有唯一实现，不重复这些规则。

### 3. 目标文件与目录

commands/database/mod.rs；configure/{command,coordinator,mod}.rs，reset/{command,coordinator,mod}.rs；disconnect_database.rs保持紧密耦合的短传输/执行；tests.rs为原Tauri单测目标中的同一数据库边界测试。没有空request/response/mapper、通用dispatcher或新Port。

### 4. 文件职责边界

两条复杂命令command仅State和原DTO→具名coordinator；各coordinator保留原完整桌面流程。断开文件承担唯一短断开操作，命令适配与两句执行同址；mod只登记/显式导出。Config唯一负责加密/旧格式迁移/原子文件写入，原Service负责真实连接/预检/重置/重连，RuntimeLogStore仍负责脱敏与写入。

### 5. 输入

原State<AppState>、DatabaseOptions及confirmation:String；原config_path、service Arc与runtime_log句柄，不新增参数、取消表、配置或状态。

### 6. 输出

configure/reset原Result<DatabaseHealth,String>，disconnect原Result<(),String>。原DatabaseOptions/Health字段、serde、参数名、命令名和错误字符串保持；不新增本地DTO复制。

### 7. 允许依赖

原Tauri State、AppState、DesktopConfig、ApplicationService公开API、原DatabaseOptions/Health与RuntimeLogStore；测试复用原tempfile/tokio和RuntimeLogStore既有test_store。

### 8. 禁止依赖

命令不执行SQL/PG直连/模型计算，不创建第二Service/连接池/worker/通用状态袋；不改变数据/迁移/模型/依赖，不添加workflow/runner/target或数据库专项入口。

### 9. 状态所有权

AppState七字段与service:Arc<ApplicationService>逐字不变；三个执行入口借用完整&AppState，connect/reset仍调用原self:&Arc<Self>方法，避免01曾发生的Arc接收者丢失。19状态owner/43Ports/19Port源保持。RuntimeLogStore原测试模块/test_store仅增加pub(crate)可见性且仍#[cfg(test)]，复用其构造创建每测试独立临时日志；生产实现/发现路径/状态不变，不改环境变量/CWD或写真实运行日志。

### 10. 副作用边界

configure保持started尽力日志→原connect(options.clone)→失败原String和failed日志；成功先原succeeded日志再保存加密配置，保存失败调用原disconnect后返回文件错误。该既有顺序保留，不虚构额外日志或改变保存失败补偿。

disconnect保持先DesktopConfig::default().save（含清凭据）成功后再service.disconnect；保存失败原?返回。reset保持加载保存配置→无连接原错误→Application preflight名称/目标预检→started日志→原reset(options,confirmation)→成功yield_now→原P4 worker幂等恢复→succeeded日志。原Application内部独立验证连接/再预检/停止/重置/重连都保持，无新增事务或读回。

### 11. 异常路径

配置/凭据读取失败和预检失败在started/reset之前返回；reset失败原字符串，只有原“彻底清空数据库失败：”前缀写critical，其他失败不加重置失败事件。configure/reset各三次原运行日志写入仍let _尽力执行，不把日志错误变成业务失败。原确认内容与options原样传入，不新增trim/别名/隐式默认。

### 12. 并发/异步/生命周期

三个命令原async与State借用保持；不引入跨命令锁、timer、listener、取消表或重试。数据库互斥/事务和worker生命周期仍原Service/PG实现。UI过期结果/监听器不移入命令；真实并发/重置保证继续沿已有数据库验收入口确认。

### 13. 兼容要求

171公共命令/参数/返回DTO/注册顺序、所有contracts/schemas/前端/数据库格式/日志/错误/配置/0.23.0/依赖/锁保持。command_set_sha256仍dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669。

仅architecture/command-contract的database定义出口和三条command_definition_files同步实际路径，物理组13保持；内部定义映射SHA变9202f5f9efc00f12491c4f1a644457935581e87214eac70e766fd568785710a9。其他12组和顶层兼容政策/前端签名/基线字段精确不变，module-boundaries仅database owner改mod.rs。不能声称整个内部清单文件bytes未变。

### 14. 实施步骤

先核实01精确完整Windows、五文档f56a1c0收尾。将原三流程完整迁入唯一执行职责，保留原State公共签名和全部正文；切文件系统模块入口，删除旧database.rs；复用隔离test_store补8原unit， retarget原验证器/正式清单/使用摘要，完成保护/静态/格式/等价/负向与真实记录后推原Windows。

### 15. 切换入口

commands.rs原mod database/pub use database::*与171注册表逐字不变，Rust同名目录接管原module。每层显式导出原函数及Tauri生成__cmd__/__tauri_command_name_宏；锁定2.11.1/tauri-macros2.6.3出口沿01已实跑成功策略，无新第三方API或版本行为需要重复查询。

### 16. 删除清单

旧src-tauri/src/commands/database.rs删除；原三个完整工作流分别由configure/reset coordinator及disconnect execute唯一承担，无旧转发入口或重复业务。逻辑职责迁移三流程，整文件逐字移动/重命名无。

### 17. 最小验证

82/82原只读源码检查、完整npm run verify:architecture、10现存变更Rust独立Rustfmt1.88 --check、32完整等价核对、17负向拒绝/bytes恢复、保护18/171命令/46迁移18PG静态、gitdiffcheck通过。新8原Tauri unit尚待自身Windows，预期Desktop46=38+8、Application230、Persistence259、Gateway112unit+41contract；不得将源码检查计为新单测或编译通过。

### 18. 阶段回归

沿原Public Platform CI完整frontend/contracts/TypeScript/Vite/17视口、fmt/locked Clippy all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动/日志。82源码排除原Windows进程入口和五个浏览器入口，留给原Windows运行；未执行非Windows Cargo/check/Clippy/build/test/前端构建/浏览器/loopback，不新增验收设施。

### 19. 失败停止条件

02自身精确完整Windows SUCCESS前VERIFYING，不能继承01成功；03～10 BLOCKED，无阶段完成记录。job实际in_progress即停止所有CI查询，提示约20～30分钟；失败只修本R11分支。

### 20. 回退点

f56a1c004cee8018f657b6cdd9fa5389a81dec20（01文档收尾、源码同完整成功e56d21f）。受控revert本节点源码/登记/元数据/守卫/测试/记录，保留用户改动，不硬reset/复制旧模块造成双owner或改历史数据。回退源码仍需原Windows自身结果。

### 21. 根 README 摘要记录

根README记录三命令职责、原副作用/失败门禁保持、8新unit与静态通过/自身Windows待验，链接本节点/索引，具体清单与风险在本记录。

### 22. docs 阶段节点详细记录

本23字段记录与R11索引/任务书/TESTING同步。32等价覆盖三工作流完整正文/三公开签名/三真实委托/三State借用、RuntimeLogStore整文件仅测试可见性逆切、冻结业务/模型/DTO/依赖/CI、公共契约、其他12组和数据库前端签名/命令及原Arc接收者。

17负向覆盖旧owner、双层宏出口、参数形状/真实委托、连接保存/补偿/尽力日志、断开文件门槛、重置预检/critical前缀/yield、测试登记/fixture只测可见性/真实日志目录隔离、mod业务与元数据错误归属；全部拒绝并恢复原bytes。临时scratch比较/探针/报告可复建，不入repo/不新建持续回归框架。

### 23. 完成标准

唯一职责/入口/旧owner清理、记录和静态验证完成，当前02VERIFYING；自身完整Windows成功才能DONE。真实PG/历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不能计为这些真实PASS。

## 新增测试与未实跑边界

8个unit实际调用新协调函数和原Config/RuntimeLogStore/ApplicationService：配置无效URI保留配置/原错误与started-failed日志；日志写入失败仍返回连接错误；断开清元数据和加密凭据；断开保存失败保留凭据/原文件错误；重置无/空保存连接；坏JSON/不支持版本；缺失凭据；已加载配置的预检URI解析失败不写destructive日志且保留配置/凭据。

所有目录、凭据与日志均测试临时目录，复用原日志fixture，不设置进程环境或发现真实日志目录。无效URI在连接选项解析处失败，不连接真实服务器；不执行真实DROP/重连/配置成功后存储失败等需要专用库的路径。生产三完整正文/顺序守卫核对不替代这些真实集成，其余按最终新库流程待验；测试无伪造Health/业务数据或关闭检查。

## 命令、摘要与交接

实际命令：原82源码入口（排除原6动态项）；npm run verify:architecture；verify_protected_assets.mjs/verify_command_contract.mjs/verify_database_baseline.mjs；原verify-database-reset.mjs、verify-database-service.mjs、verify-tauri-bootstrap.mjs和source-hygiene；独立Rustfmt1.88 --edition2021 --config skip_children=true --check；scratch/r1102_compare.py/r1102_probes.py与gitdiffcheck。

Domain365/129声明/301映射，scan1290→1298；声明摘要217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，usageDigest86bab5f5cc9f4e4806ea09f94504b54bf3ace9c1dfa73eb2a28d28c94afb959f。App422/43Ports/19Port源、PG直接38/state19保持。保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57、DB静态aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

Mermaid Chart中文实际连接/断开/重置与存储/原Service链已更新；Create State使用足球稳定UUID96f03270-c236-46de-99fa-db85d2fbf4ce交接。本次旧scratch被维护清理后从准确Git分支恢复，无用户改动丢失；附件按scratch读，规则入口迁至skil/Code/0.0.1且AGENTS内容sha5cc6750404092a080ddcd18685a25dfb90d3f809不变。Git制品不另存，无子代理。

## 全部实际文件清单

基于f56a1c0，no-renames口径10A/12M/1D，共23路径；9新增Rust文件、10现存变更Rust检查，三流程职责迁移，整文件逐字移动/重命名：无。

### 新增（10）

- `docs/modular-rewrite/R11-tauri-commands/R11-02-database-commands.md`
- `src-tauri/src/commands/database/configure/command.rs`
- `src-tauri/src/commands/database/configure/coordinator.rs`
- `src-tauri/src/commands/database/configure/mod.rs`
- `src-tauri/src/commands/database/disconnect_database.rs`
- `src-tauri/src/commands/database/mod.rs`
- `src-tauri/src/commands/database/reset/command.rs`
- `src-tauri/src/commands/database/reset/coordinator.rs`
- `src-tauri/src/commands/database/reset/mod.rs`
- `src-tauri/src/commands/database/tests.rs`

### 修改（12）

- `README.md`
- `architecture/command-contract.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/README.md`
- `scripts/verify-database-reset.mjs`
- `scripts/verify-database-service.mjs`
- `scripts/verify-rust-source-hygiene.mjs`
- `scripts/verify-tauri-bootstrap.mjs`
- `src-tauri/src/runtime_log.rs`

### 删除（1）

- `src-tauri/src/commands/database.rs`
