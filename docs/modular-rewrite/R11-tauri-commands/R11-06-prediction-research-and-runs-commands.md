# R11-06 Prediction、Research 与 Runs Commands

状态：`VERIFYING`（自身完整Windows尚待验收）。

R11-01～05 DONE；06 Prediction/Research/Runs Commands已实施VERIFYING，07～10 BLOCKED，R11 IN_PROGRESS。用户2026-10-23“收尾05开始06”，继续唯一rewrite/r11-tauri-commands；基线29de82c820448e34ba65722addb47073b0824d7c为05五文档[skip ci]收尾，生产源码同自身完整Windows成功b9040df。

### 1. 目标

为推演、就绪检查、路由、运行记录、P4规划/工作台及研究冲突提供唯一具名传输入口，命令只转换原参数、调用原Application能力并映射原错误。

### 2. 现状与来源

旧prediction.rs混合9条推演/运行记录命令；orchestration.rs混合8条P4规划/工作台/Research冲突命令。正式171契约没有其他公开research命令；不能凭任务标题新增接口。AI问答/Profiles属08，本节点不迁移api_workspace/openai。原任务、研究执行、队列与快照业务在R8/R9和Application中已有唯一owner，本节点不重做。

### 3. 目标文件与目录

commands/prediction根6条推演入口，runs/3、p4/7、research/1；共17命令文件、4只登记/显式导出的mod.rs，21新Rust。按实际用途递归目录，不新增空DTO/mapper/error或无业务需要的coordinator。

### 4. 文件职责边界

每条命令一个文件；根/子目录只显式导出原函数与Tauri双宏。推演/运行记录继续原Prediction Service和用例；P4规划/工作台仍原底层owner；research/resolve_p4_conflict仍原Research facade和冲突用例，目录归并不改变业务owner。

### 5. 输入

原State<AppState>、PredictionCommand/StoredMatchPredictionCommand/RoutePreviewCommand、PlanP4HorizonsCommand/ResolveP4ConflictCommand、limit、String/Option<String>身份及reason保持；不改serde/default、不新增请求ID/取消表。

### 6. 输出

17原Result签名保持，包括原Value、PredictionExecution、MatchPredictionReadiness、RouteDecision、ModelRunListItem及P4记录/事件/就绪/工作台。ModelRunListItem沿用原PG crate公开DTO导入，不复制类型或新增数据访问；dry run仍原输出的serde_json::to_value。

### 7. 允许依赖

原Tauri State、bootstrap::AppState、commands::parse_uuid、既有Application/Domain DTO、serde_json及原ModelRunListItem类型。沿01～05已完整Windows证明的锁定Tauri2.11.1/macros2.6.3/CLI2.11.4出口与原API；无新API或版本行为，未触发新的Context7问题。

### 8. 禁止依赖

命令不新增SQL、文件I/O、池、模型计算、状态、服务实例、worker/锁/事务/重试；不新增DTO/依赖/迁移/runner/workflow/持续测试框架或数据库专项入口。

### 9. 状态所有权

AppState、ApplicationService/Arc接收者、数据库会话/队列/模型注册表/日志owner全冻结；state-ownership仍19。命令只借用原State，不持有新共享状态。

### 10. 副作用边界

原Application、Ports、PG adapters/外部模型与队列承担全部执行副作用；隐藏运行记录仍hide_run_from_history(parsed, reason)，不做物理删除。dry run仍同步无连接门槛，不加数据库检查或持久化；研究冲突提交仍原身份/人工裁决/账本事务。

### 11. 异常路径

原运行ID“运行 ID”、任务“冻结任务 ID”、比赛“比赛 ID”的文字/空格和前置顺序保持；list_p4_freeze_tasks仍Option.as_deref().map(parse_uuid).transpose()?，None、合法Some、非法Some语义不变。read_run/hide保留原let parsed，task/workspace保留原内联解析；limit/reason/command原样传递，原error.to_string()和dry run两个独立错误出口保持。

### 12. 并发/异步/生命周期

17完整正文与公开async签名等价；dry run同步，其他16 async沿原await顺序，无新时钟/ID/锁/线程/重试。UI请求和过期结果、P4任务/worker/取消/事件/事务恢复仍原所有者；不把“传输无变化”称为真实PG/并发运行证明。

### 13. 兼容要求

171命令名称/注册顺序/参数/返回DTO、前端/Schema/配置/错误/日志、版本0.23.0、依赖/锁/模型和所有保护资产保持，公共集合SHA256 dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669。

内部物理组14→13：原prediction9+orchestration8按原组相对顺序合并17，原全局注册171顺序字节冻结；其他12组精确不变。同步实际逐命令定义和module-boundaries唯一目录，定义映射SHA fe7b8d78d2daf40329c3d007c278dde5a8086e63a5b68387d0dcf919e3f53ca2；不声称内部清单字节冻结。

### 14. 实施步骤

先fresh核05精确完整Windows、日志/步骤/artifact并五文档[skip ci]收尾；在既有Prediction门禁补17传输/纯出口/双宏/owner断言，先确认缺新模块失败。迁移原完整函数并接出口、删除旧owner，更新实际清单/定义读者；核对正文/签名/冻结资产，沿原静态及Windows流程。

### 15. 切换入口

commands.rs只删除mod orchestration/pub use orchestration::*两行，原prediction模块名直接解析新目录；不增加旧转发。原bootstrap/command_registry.rs及其171注册字节冻结。4mod只显式导出每函数与__cmd__/__tauri_command_name_双宏，唯一新目录接通原三方契约。

### 16. 删除清单

删除commands/prediction.rs与commands/orchestration.rs。原17完整函数按职责迁移，整文件逐字移动/重命名无；无重复函数、旧转发、第二服务/状态/DTO或被替代业务实现残留。

### 17. 最小验证

82/82原只读源码、完整npm run verify:architecture、22现存变更Rust独立Rustfmt1.88 --check（21新增+既有commands.rs）、69等价、隔离副本19破坏拒绝/3守卫恢复正向、主目录167源码/清单SHA256不变、18保护资产/171命令13组/46迁移18PG静态与git diff --check通过。

69包含17完整正文/17完整公开签名/17原AppState，业务/UI/Ports/模型/依赖/状态/其他命令owner/01～05冻结，公共契约及其他12组、根登记、两个原守卫逆变换。新增/迁移Rust单测0，沿原targets，预期Desktop46/App230/PG259/Gateway112unit+41contract须06自身Windows确认；这些静态核对不计动态测试。

### 18. 阶段回归

沿原Windows Public Platform CI执行完整frontend/contracts/TypeScript/Vite/17视口、fmt/locked Clippy workspace all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动与日志。不执行非Windows Cargo/check/Clippy/build/test/前端构建/浏览器/loopback。82源码排除原Windows进程入口与五浏览器入口。

### 19. 失败停止条件

06自身完整Windows成功前保持VERIFYING，07～10 BLOCKED；保护/公共契约/最小验证失败继续修复同一分支。实际Windows job首次in_progress后停止全部CI查询，提示约20～30分钟，等用户返回；05成功不是06通过证据。

### 20. 回退点

29de82c820448e34ba65722addb47073b0824d7c，生产源码同05自身Windows成功b9040df。按节点原子提交受控revert命令、清单、守卫和文档，不硬reset/手工复制旧owner/修改历史数据；回退源码亦需自身Windows。

### 21. 根 README 摘要记录

同步17条职责/旧owner退出、171兼容/13组、实际静态结果及06自身待验，链接本记录；01～05 DONE、06 VERIFYING、07～10 BLOCKED、R11 IN_PROGRESS，不创建阶段完成记录。

### 22. docs 阶段节点详细记录

本节点记录、阶段索引、任务书、根README与TESTING同步；01～05记录在实施中字节冻结。原Prediction门禁只新增传输块、旧业务断言逆变换精确保持；历史比分UI门禁只切换正式逐命令定义读者，所有原断言保持。正式command verifier字节冻结。

19隔离探针覆盖两个旧owner恢复、根登记/根宏/子宏缺失、出口夹带逻辑、Option非法ID吞错、UUID标签/前检丢失、reason/limit/签名、shadow转formal、研究参数替换、原错误、dry run加会话门槛/改async、边界owner和注册缺失。只在独立临时副本修改，3守卫恢复正向，主167文件指纹不变；不重复05曾在主目录做探针的恢复问题。首次静态汇总器路径误写使三个原architecture脚本MODULE_NOT_FOUND，纠正scratch路径并只重跑三项，最终82/82；生产源码无相关失败。

### 23. 完成标准

17唯一传输入口、旧实现清理、记录与静态验证完成；06 VERIFYING，07～10未开始，R11未完成。真实PG/历史保护四项/不可变账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计真实PASS。

## 实际能力归属

| 能力 | 命令数 | 实际位置 | 原业务所有者 |
|---|---:|---|---|
| 推演/就绪/路由/dry run | 6 | commands/prediction/各命令.rs | 原Prediction Service/用例 |
| 运行历史/读取/隐藏 | 3 | commands/prediction/runs/ | 原Prediction Service/Ports |
| P4规划/任务/事件/工作台 | 7 | commands/prediction/p4/ | 原规划与Prediction工作台用例 |
| 研究冲突裁决 | 1 | commands/prediction/research/ | 原Research Service/冲突用例 |

## 实际验证与未执行范围

原82read-only node入口、npm run verify:architecture、node scripts/verify_protected_assets.mjs、node scripts/verify_command_contract.mjs、node scripts/verify_database_baseline.mjs、22文件独立Rustfmt1.88 --edition 2021 --config skip_children=true --check、scratch/r1106_compare.py和隔离副本探针、git diff --check实际通过。最终报告r1106-static.json、r1106-architecture.log、r1106-equivalence.json、r1106-negative.json、r1106-primary-fingerprints.json位于repo外scratch，可从源码复建，不新增持续runner。

Domain365/129声明源/301映射、scan1397→1416；sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，usageDigest1164aba99321eb90f30185c02633d60a012506b917680f5077759f5ac75ed12f。App422/43Ports/19Port文件、PG直接38、状态19保持；保护aggregate d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57和DB静态aggregate d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。

Mermaid Chart按实际17命令→原Prediction/规划/Research→Ports/模型/PG链路更新中文图；Create State沿足球稳定UUID96f03270-c236-46de-99fa-db85d2fbf4ce续接。未新增依赖或新第三方API，沿既有完整Windows证明的Tauri出口；无子代理。未运行真实PG任务/研究冲突/隐藏账本或私有模型固定回归，原底层代码/targets保留，实际效果与继承删除风险仍最终新库待验。

## 全部实际文件清单

基于29de82c820448e34ba65722addb47073b0824d7c，no-renames口径22A/10M/2D，共34路径。21新Rust（17命令、4纯出口），另既有commands.rs修改；整文件逐字移动/重命名：无，17完整函数按职责迁移。

### 新增（22）

- `docs/modular-rewrite/R11-tauri-commands/R11-06-prediction-research-and-runs-commands.md`
- `src-tauri/src/commands/prediction/dry_run_default_fixture.rs`
- `src-tauri/src/commands/prediction/execute_prediction.rs`
- `src-tauri/src/commands/prediction/execute_prediction_from_match.rs`
- `src-tauri/src/commands/prediction/execute_shadow_prediction_from_match.rs`
- `src-tauri/src/commands/prediction/inspect_match_prediction_readiness.rs`
- `src-tauri/src/commands/prediction/mod.rs`
- `src-tauri/src/commands/prediction/p4/list_p4_freeze_task_events.rs`
- `src-tauri/src/commands/prediction/p4/list_p4_freeze_tasks.rs`
- `src-tauri/src/commands/prediction/p4/mod.rs`
- `src-tauri/src/commands/prediction/p4/p4_freeze_readiness.rs`
- `src-tauri/src/commands/prediction/p4/plan_p4_horizons.rs`
- `src-tauri/src/commands/prediction/p4/read_p4_freeze_task.rs`
- `src-tauri/src/commands/prediction/p4/read_p4_match_workspace.rs`
- `src-tauri/src/commands/prediction/p4/read_p4_task_workspace.rs`
- `src-tauri/src/commands/prediction/preview_route.rs`
- `src-tauri/src/commands/prediction/research/mod.rs`
- `src-tauri/src/commands/prediction/research/resolve_p4_conflict.rs`
- `src-tauri/src/commands/prediction/runs/hide_model_run_history.rs`
- `src-tauri/src/commands/prediction/runs/list_recent_runs.rs`
- `src-tauri/src/commands/prediction/runs/mod.rs`
- `src-tauri/src/commands/prediction/runs/read_run.rs`

### 修改（10）

- `README.md`
- `architecture/command-contract.json`
- `architecture/domain-type-inventory.json`
- `architecture/module-boundaries.json`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/11-R11-tauri-commands.md`
- `docs/modular-rewrite/R11-tauri-commands/README.md`
- `scripts/verify-history-scoreline-ui.mjs`
- `scripts/verify-prediction-service.mjs`
- `src-tauri/src/commands.rs`

### 删除（2）

- `src-tauri/src/commands/orchestration.rs`
- `src-tauri/src/commands/prediction.rs`
