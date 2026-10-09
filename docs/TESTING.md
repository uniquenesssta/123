# Testing

## 2026-09-30 验证执行约定

沿用本文件现有 Windows 验证命令、`windows-acceptance.ps1`、`run_database_baseline.mjs` 和已有 Rust 测试目标。不新增持续回归框架、runner、CI workflow 或数据库专项入口；已有验证器路径/调用遗漏可在现有入口内修正。

最终封包由用户按既有流程建立新数据库，不要求每个节点反复建库。节点可以复用专用测试库；暂不能运行的数据库/XLSX/Full 项必须在节点和阶段记录中标记“最终封包新库待验”，不能写成通过。历史失败经代码/夹具修复后仍需新库实跑才算关闭。最终封包前使用现有数据库基线、直接 cargo 命令运行相关已有 contract、真实 XLSX 与 Windows Full，清零失败和待验项。既有真实事务、身份、历史引用保护和模型资产要求保持。


## R7-06 数据库待验（2026-09-30）

R7-05 精确提交 `02e56bad` 的 Windows run `36712015030` 已通过，Application 53 项实际执行。R7-06 精确提交 `4496399` 的 run `36734194083` 已通过，Persistence 99 项/Application 53 项实际通过；18 项数据库测试仍 ignored。R7-07 精确提交 `3ff21cb` 的 Windows run `36757339619` 已通过，Persistence 102 项/Application 53 项实际通过；R7-08 不继承该 PASS，Windows Automated 负责本轮编译/单测/Clippy/打包/启动。

数据库仍沿用原 18 项 `postgres_integration` 和 `run_database_baseline.mjs`，没有新增专项或库。最终封包在 Windows 新库实跑时，核对四项历史 chain/pair/events/P4 测试；pair 已扩充预检后末行外键失败回滚、合法重试的业务/ended_previous_count/审计一致、成功批次重复拒绝。P4 已补亚微秒首建/重试实际时间、真实 1 微秒差异、published/effective 未来证据拒绝、原四链及冻结不可变性。全部当前“已修改、最终封包新库待验”，ignored 编译不能作为历史失败关闭证据。有效 XLSX 和 Windows Full 按既有最后封包流程执行。

## R7-07 数据库待验（2026-10-01）

沿用原 `postgres_integration.rs::match_scope_inference_and_lineup_pair_transaction_are_atomic`、18 项 broad 基线及现有 Windows 入口，不新增 target/runner/workflow/数据库。原 pair 断言现在包括身份/类型/窗口校验、单侧失败审计回滚、客队明细外键等待期间取消后父锁释放和零遗留；两次 pair/一次 single/一次 workbook 同时观察父锁等待后放行，检查每侧唯一活动版本、完整 11 人及前驱链、成功审计/账本一致。SQLx 提交前取消与提交往返中的不确定结果分开，后者不能一概承诺已回滚。真实并发/取消场景当前“最终封包新库待验”，ignored 编译不计实跑通过。

`pair_transaction/validation.rs` 的 3 个 inline 单测随原 workspace tests 在 Windows 执行，不依赖 PostgreSQL。受影响原 Lineups Service/Match Workflow/Match Lineup Chain/Player Role 等静态检查指向当前 owner；R7-07 的 module-boundaries 直接模块数量为 39，R7-08 删除旧根 chain 后为 38；既有 lineups 命名空间承接职责。

## R7-08 当前待验（2026-10-01）

沿用原 workspace tests、18 项 broad 与 Windows 入口。原 5 个窗口测试迁入 `chain/window.rs`，追加窗口精确起点/起点前一纳秒、各正式时点开球前一秒截止与规范化断言；原阵容枚举检查迁入 `history/mapping.rs`。现有 chain 测试增加截止/起点相等与前后一微秒、同时点 confirmed 优先/较新 expected 优先、等时间 UUID 排序、隐藏历史详情/链/球队列表隔离、201 行的两种列表 0/max limit 与链上限。原 API clamp 1..=200 保持，内部 chain 请求 500 仍实际最多 200。

现有 pair 测试增加历史按 ID 追溯、隐藏版本重复删除拒绝、物理删除读回失败、删除/创建同时等待父锁后放行，核对无死锁、唯一活动版本、隐藏前驱不恢复及一次审计；不改变后续原账本、取消和混合并发测试。历史删除现先锁比赛再锁阵容，READ COMMITTED 锁后重读与引用核验/恢复/审计同事务。真实 PG 仍最终封包新库待验；本项 Windows 编译/非数据库测试/打包等待精确提交 CI，ignored 不计通过。不新增 target/runner/workflow/数据库设施。

R7-08 首轮 Windows run `36764662888` 在阵型验证器旧路径处失败；Rust/截图/构建/打包/启动未执行。修订三个现有 verifier 的阵型/历史 owner 路径和 Rustfmt 空白匹配；不删除检查。四个临时缺失契约探针仍被拒绝并恢复。前端入口 83 个源码/契约检查通过，其余 5 个浏览器交互/截图检查待 Windows CI（当前无 Windows 浏览器）；未运行 Linux/macOS 客户端验收。Windows 动态验收等新精确提交 CI，不将本地 Node 源码契约检查计为 Windows 或数据库通过。

## Target platform

The current development verification and acceptance target is Windows only, as confirmed on 2026-09-30. Run the commands below in Windows PowerShell or the existing Windows batch entry points. Linux and macOS build, test, package, runtime and interaction checks are outside the current acceptance scope and must not be added as required CI matrix entries.

The existing CI runs on `windows-2025`; there are no Linux/macOS jobs to remove. PostgreSQL integration and real XLSX checks remain required where specified, with the client tests run on Windows. A database server's host OS does not define the desktop acceptance target. Source inspection or static checks from another environment are audit evidence only, not Windows acceptance.


## Frontend

```bash
npm ci
npm run verify:frontend
```

This runs the public model-boundary audit, project-specific static checks, TypeScript validation, and the Vite production build.

## Rust

```bash
npm run verify:rust
```

Application Ports 与 PostgreSQL 模块清单检查沿用 `npm run verify:architecture`；Ports 和 R6-03 Player Directory/Detail 同时位于现有 frontend 入口。Ports verifier 递归核对声明及子文件禁止依赖，`sourceScan` 表示当前 Port 文件/摘要/数量与组合根导入，R3 的持久化 209/232 统计仅保留为历史基线。需要同步已审查的 Port 源码变化时，使用原脚本：

```powershell
node scripts/verify-application-ports.mjs --refresh-source-scan
node scripts/verify-application-ports.mjs
```

刷新仅在清单声明/依赖门禁通过后写入扫描字段，不登记未知 trait 或放宽 JSON 兼容边界；审查 Git diff 后提交。PostgreSQL owner 清单核对 `lib.rs`/`adapters/mod.rs` 的直接模块声明，R5 子检查继续经 competition verifier 间接执行。

This runs formatting, Clippy with warnings denied, and workspace tests with `Cargo.lock` enforced.

## Public boundary

```bash
npm run verify:public-model-boundary
```

The check fails when private model directories, parameters, fixtures, model-specific contracts, or direct dependencies on the removed engine crates appear in the repository.

## Database integration

Database tests require a dedicated PostgreSQL test database. Never point destructive or reset tests at a production database.

R7-01 reuses the existing `node scripts/run_database_baseline.mjs` and Windows Full entry points. They now run `match_catalog_repository_contract` before `postgres_integration`; Automated CI compiles the target and runs its non-database safety test but leaves the PostgreSQL contract ignored. The contract checks both the URL database name and `current_database()` before migration/writes, removes only the current run's fixtures even after assertion failure, and retains its reusable immutable schema artifact and audit evidence. `--dry-run` on the existing database runner verifies target selection without connecting or executing Rust.

R7-02 extends the existing `entity_matching_references_repository_contract` target. Automated Windows CI compiles its PostgreSQL assertions and runs the database-name safety test; it does not execute the ignored database tests. On the final dedicated Windows test database, run the existing target directly: `cargo test --locked -p football-persistence-postgres --test entity_matching_references_repository_contract -- --ignored`. It covers identity conflicts, concurrent binding/metadata writes and direct/import interleaving with full batch rollback. Fixture cleanup retains audit evidence. No new runner or workflow is added.

R7-03 extends the existing `entity_permanent_delete_repository_contract` target. On the final dedicated Windows test database run `cargo test --locked -p football-persistence-postgres --test entity_permanent_delete_repository_contract -- --ignored`, plus the existing deletion/archive and force-delete contracts. It observes actual parent-lock waits with `pg_blocking_pids` before committing or rolling back dynamic-tag/preset writers; no timing guess decides transaction order. Automated CI compiles these ignored assertions and runs the database-name guard. Real PostgreSQL execution remains deferred until the final new database; fixture cleanup retains audit evidence.



## Expected model behavior

The public model stub must always return an explicit unavailable error. A successful prediction from `football-model-stub` is a test failure because the public repository must not contain an executable private engine.

## 默认战术角色验收

验证球员位置的默认战术角色能够自动继承到阵容、比赛输入、Excel 与复盘记录，并保留角色来源和历史时点审计。

## Windows 全链路验收

使用 `验收平台.bat` 运行环境预检、前端契约与构建、Rust 格式/Clippy/测试、专用 PostgreSQL 集成测试、Tauri release 构建、运行时冒烟和日志分析。未提供专用测试数据库或外部模型运行时时，相关阶段必须明确标记为 blocked。

## R7-08 收尾与 R7-09 待验（2026-10-01）

08 精确修订提交 `3c8f7f0` 的 Windows run `36801488090` 全通过：Persistence 107、Application 53、17 个截图视口、Rust fmt/Clippy/tests、安装包及启动日志（7 条/3 操作）。此前 08 首轮失败/浏览器待验由该成功 CI 更新；真实 PG/XLSX/Full 待验保持。

09 在既有预设文件内补 8 个 inline 测试，原 2 个保留，随既有 workspace tests 在 Windows 执行；新提交不得继承 08 PASS。原双方阵容 broad 用例追加预设合法重复预览、无正式阵容/审计/预设变更、跨队保存拒绝且旧版本不变、成员过期/归档拒绝、复制与删除级联断言；不增加测试目标或数据库设施。18 项 broad 仍 ignored、真实执行最终封包新库待验。module-boundaries 直接模块 37，迁移/算法/保护资产/生产依赖/锁文件冻结。

## R7-09 收尾与 R7-10 待验（2026-10-01）

09 精确提交 `3525c04` 的 Windows run `36808609584` / job `110198409740` 全步骤通过：Persistence 115、Application 53、17 个截图视口、Rust fmt/Clippy/tests、安装包与启动日志（7 条/3 操作）。真实 PG/XLSX/Full 待验保持。

10 在新账本实际职责文件内补 6 个 inline 测试，随既有 workspace tests 在 Windows 执行；不继承 09 PASS。原球员月度与双方阵容导入 broad 用例增加非法状态/冲突拒绝、跳过与候选解决的 pending 计数一致、球员重新预检/成功重试、跨导入类型读取拒绝及返回/账本/一次审计断言，并继续原末行失败整批回滚/重试测试。18 项 broad 仍 ignored，数据库实跑沿用最终封包新库流程。无新测试目标或数据库设施。直接模块 38，迁移/算法/保护资产/生产依赖/锁文件冻结。

### R7-10 完成与 R7-11 待验（2026-10-01）

精确 `cc0d34d2f4bce0a9a656a0624c5e59c602aa7d86` / Windows run `36815568401` / job `110219751121` 全 SUCCESS；Persistence 121 项/Application 53 项、前端类型/17 视口/构建、fmt/Clippy/workspace tests、release/MSI/NSIS 与启动日志 7 条/3 操作实际通过。artifact `11141479972` SHA-256 `ef4a1166fc4e302944381a02c714ee0a22d0b51ac3e752be8b0f3dbb8382c211`；10 DONE。

11 迁移 59 个生产函数与原 8 个内联测试，新增 6 项现有生产职责内联测试，期待 Windows Persistence 127 项，尚未实际执行。原 PG 月度用例增加重复只读预览行身份/载荷、无球员/效力期/自动球队事实和无提交审计断言。外部 ID 重名/并发/改绑拒绝、资料包多效力期、重复导入与整批回滚沿用既有测试。18 broad 仍 ignored，仅编译不算数据库 PASS；历史四项、账本、有效 XLSX/Windows Full 统一最终封包新库验收。沿用原 architecture/frontend/Windows Automated，无新 runner/工作流/数据库设施。11 保持 VERIFYING，必须取得自己的精确 Windows CI。

### R7-11 完成与 R7-12 待验（2026-10-01）

精确 `310c46bbedb731e20ffaca2c67df0eefc3ce6e3a` / Windows run `36824027121` / job `110245578512` 全 SUCCESS；Persistence 127 项/Application 53 项、前端类型/17 视口/构建、fmt/Clippy/workspace tests、release/MSI/NSIS 与启动日志 7 条/3 操作实际通过。artifact `11145641022` SHA-256 `2c58fc6f91f671b07b243b64908c5c5e52ee8c74d8d70c6f79eece58aa1ac51e`；11 DONE。

12 原 22 个 inline 测试迁至真实共享球队职责，新增 Persistence 5 项/Application 2 项，预期 Windows 132/55 项，尚未执行。原 PG 月度夹具补重复预检行身份/载荷/无事实、末行失败后球队/行/账本/审计回滚及原批次恢复、一次成功审计、原文/中文主名/简称保存、非法候选拒绝及 skip/选择后 pending 计数共同事务。完整包保留两条独立原子链及原球员失败恢复边界。83 项原源码门禁/架构和 Rustfmt 通过，六项破坏探针拒绝并恢复；不继承 11 的 Windows PASS。18 broad 仍 ignored，真实 PG/历史四项/账本/有效 XLSX/Full 最终封包新库待验。沿用原验收流程，无新 target/runner/workflow/数据库设施；12 VERIFYING，13～15 BLOCKED。

### R7-12 完成与 R7-13 待验（2026-10-01）

精确 `fe2f7ebcbf50e4bb40016d649a95b2c295aeab55` / Windows run `36831302483` / job `110268300510` 全 SUCCESS；Persistence 132 项/Application 55 项、前端类型/17 视口/build、fmt/Clippy/workspace tests、release/MSI/NSIS 与启动日志 7 条/3 操作实际通过。artifact `11148207904` / 14,019,073 字节 / SHA-256 `19a5d423e18bb2e53a529416207194e8e829e26b4673e21425fd51011d34df2b`；12 DONE。

13 纯月度读取迁移，五个原函数体/四签名/18 查询与映射保持，不增加镜像 inline 测试；原 132/55 项需本项 Windows CI 实际执行。原 PG 月度夹具扩空资料默认/null 时间、clear 值/元数据与历史聚合、重复读取不改事实/账本/审计、当前关系包含起止日/未来关系不能消除缺口、90 天前后观察、未来资料非负 stale_days、上月/下月/当前日关系与两队三段历史导出/排序。83 项现有源码/契约、架构与 Rustfmt 通过，六项破坏探针拒绝并恢复。18 broad 仍 ignored，PG/历史四项/账本/XLSX/Full 最终封包新库待验；无新 target/runner/workflow/数据库，模块清单 36，迁移/算法/资产/deps/locks 冻结。13 VERIFYING，14～15 BLOCKED。


### R7-13 完成与 R7-14 待验（2026-10-01）

精确 `78138dba30ead1dbc4f4e5d2c6595e5775edfa0c` / Windows run `36854202029` / job `110342691574` 全 SUCCESS；Persistence 132/Application 55、前端类型/17 视口/build、fmt/Clippy/workspace tests、release/MSI/NSIS 与启动日志 7 条/3 操作实际通过。artifact `11158674438` / 14,017,971 字节 / SHA-256 `3b6b481c07e7626c81938480d81db1298d03797bce3785d593b3c27c745aa295`；13 DONE。

14 迁移 48 个原函数，全部函数体、五个公开签名、原 SQL/投影不变，不新增镜像 inline；原 132/55 项需本项 Windows CI。沿用 pair/workbook 原 PG 夹具补重复导出/AI 字段/去重、空/未知比赛、读取不写事实/账本/审计、预检只暂存及行身份/载荷/计数、错误主客身份阻断后无副作用；既有成对事务、单侧合法替换、末行回滚/恢复、裁决计数/审计、共同锁并发和 cutoff/actual 模型隔离保留。83 项现有源码/契约、架构与 Rustfmt 通过，六项破坏探针拒绝并恢复。18 broad PG 数量不变且最终封包新库待验，ignored 编译不算实跑；XLSX/Full 同样待验。无新 target/runner/workflow/数据库设施；14 已通过下述精确 Windows CI 并 DONE，15 已启动、VERIFYING。

### R7-14 完成与 R7-15 待验（2026-10-01）

精确 `bb0012085d92cb63742ff5b5571a8003984db709` / Windows run `36860224760` / job `110362332343` 全 SUCCESS；Persistence 132/Application 55、前端类型/17 视口/build、fmt/Clippy/workspace tests、release/MSI/NSIS 与启动日志 7 条/3 操作实际通过。artifact `11161982535` / 14,017,440 字节 / SHA-256 `31f7042c8981c0f5d34fd77630e78ff206b188be36579f6b9656d2f25652caf1`；14 DONE。

15 原 8 个重复球队合并函数/5 inline 保持并迁入共享身份职责；新增原生产 row.rs inline 3 项，预期 Windows135/55，需本项CI实跑。原子记录 PG 用例扩生成键、同物理行多实体/维度/标签/双球队与跨行/表暂存、UUID/载荷读回、重复维度整批回滚、非法行号三类前检/同源 pending 保留、账本/事实/审计不留下失败副作用。原解析器空白行/多球队回归不变；18 broad数量不变且最终封包新库待验。83项现有源码/契约、架构/Rustfmt通过，六项破坏探针拒绝并恢复。完整R7当前职责/不变量已累计复核，15VERIFYING；不提前创建阶段完成记录或开始R8，PG/历史四项/账本/有效XLSX/Full的真实结果仍必须在最终新库验收取得。无新target/runner/workflow/数据库设施。


### R7-15 与 R7 阶段收口（2026-10-01）

精确 `a928c8b5ddcf37b569fbdab8c413be06d6270aac` / [Windows run 36871154039](https://github.com/uniquenesssta/123/actions/runs/36871154039) / job `110398949507` 全 SUCCESS；Persistence **135 项**、Application **55 项**、17 个截图视口、前端类型/生产构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 与启动日志 **7 条记录 / 3 个完成操作**实际通过。 artifact `11167454668` / 14,018,298 字节 / SHA-256 `8843e8c822b590dc4fd4a8f56e5ab4df477fe36ae8c11592795de852fa1fbc20`；首轮 unused import 的 Clippy 失败已由修订提交实跑关闭。

15 DONE，R7 节点及 Windows Automated 出口完成；[阶段完成记录](modular-rewrite/R07-match-lineup-workbook-persistence/R07-stage-completion.md) 列明历史四项/账本/各 contracts/有效 XLSX/Windows Full 最终新库待验。18 broad PG 及其他数据库 contracts 仍 ignored，不能写成实跑通过。本轮只补文档，引用上述已验收的最终源码，不重复 Windows 编译打包；R8 未启动。

## R8-01 当前验证（2026-10-01）

沿用原 Application workspace tests 与 `prediction/tests.rs` Probe，在输入构建生产模块增加 6 项 tokio inline 测试：正式/影子与 P4/P7 家族/原 route/scope/manifest 保留，权限拒绝零输入 I/O，输入/质量变化拒绝，运行身份变化不改指纹，Port 失败/缺失审计停止，以及非法家族/非对象错误语义。原 55 项保持，预期 Application 61；Persistence 原 135 保持。此数量仅为源码预期，需精确 Windows CI 实跑确认，当前 VERIFYING。

既有 `verify-prediction-service.mjs` 扩充唯一 owner、顺序、原时钟/字段和副作用边界检查；六项临时破坏探针必须拒绝并恢复。Node 源码检查及 Rustfmt 不计 Windows 编译/动态通过。原 P4/P7 contract/engine/time-window、persistence/fact/orchestration/workbench 测试沿用原目标；涉及 PG 的 ignored 项仍最终新库待验，公开 stub 不替代私有固定回归，不新增 runner/workflow/数据库。

## R8-01 Windows 收尾（2026-10-02）

精确 `cba72fdfaaf640a17c1e73c326536189dda74d17` / Windows run `36881256338` 全 SUCCESS；Application 61、Persistence 135（六项新增输入构建全部 ok）、17 视口、Rust fmt/Clippy/workspace tests、前端生产构建、release/MSI/NSIS 和启动 7 条/3 操作通过。01 DONE，02 已启动并须自身精确 CI。数据库 ignored/私有 Golden Master/Full 不计通过，沿用既有最终新库待验清单。

## R8-02 当前验证（2026-10-02）

现有 Persistence 生产模块 inline 增加六项测试（范围阈值/过滤后上限/typed 主客场及 scope/基准策略/原平台投影评分和证据/中性），原三项曲线测试保留；预期 Persistence 141、Application 61，尚待本项 Windows CI 确认。原 `postgres_integration::historical_snapshot_excludes_results_ingested_after_the_cutoff` 扩 cutoff 相等与前后一微秒、finalized/created 各自未来隔离和基准守卫；仍同一个 ignored target，18 broad 数量不变，沿用最终封包新库待验，不创建新 target/runner/workflow/数据库。

83 项源码检查、完整架构、Rustfmt、保护资产/171 命令/46 迁移静态基线 PASS；两段 SQL、原辅助函数/测试/范围及加权投影比对不变，六项破坏探针拒绝并恢复。5 项浏览器检查与 Rust/Windows 交付待精确 CI；公共固定平台评分只验证既有公开特征投影，不等于私有 P4/P7 概率 Golden Master。现有两次查询没有跨查询事务快照保证，保持原语义；实际 PG 结果必须新库实跑，ignored 不计 PASS。


## R8-02 Windows 收尾 / R8-03 验证

`cc2b0fe7601efdbad44fa5badcb6507f226698a7` / run `36891488571` / job `110467936302` 全 SUCCESS。Application 61/Persistence 141、六项新增历史测试、17 视口、前端类型/生产构建、Rust fmt/Clippy/workspace tests、release/MSI/NSIS 和启动 7 条/3 操作通过。02 收尾记录保存精确 artifact，前节“待 CI”为实施时历史记录。

03 在原 Application 单测 target 增加八项测试，复用原 Probe 和 fake registry；覆盖只读时钟/流程/manifest/路由、历史及质量临界值、阵容/身份阻断优先、异常转报告或原样返回、许可/评分/原因去重。预期 Application 69、Persistence 141，尚待本项 Windows CI，不将本地源码审查计为 Rust 实跑。原八 helper、async workflow 和报告汇总逐段等价；六项破坏探针均拒绝并恢复。83 项既有源码检查、完整 `npm run verify:architecture`、18 保护资产/171 命令/46 迁移及 18 PG 静态基线、Rustfmt 和 `git diff --check` 均已 PASS；5 项浏览器检查与编译/Clippy/单测/交付留自身 Windows CI。没有 Linux/macOS 动态验证、新 runner/workflow/target/数据库。

真实 PG、历史四项/账本/XLSX/Full、私有 P4/P7 Golden Master、模型历史删除 trigger 风险仍沿用最终封包新库待验；ignored 不能计通过。


## R8-03 Windows 收尾 / R8-04 实施时验证记录（2026-10-02；已通过，见下方）

精确 `47ba3dea6ca873248728b9846421c009d00f51bc` / run `36902069546` / job `110503393919` 全 SUCCESS；Application 69/Persistence 141、八项新审计测试、17 视口、前端类型/构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 及启动 7 条/3 操作通过。前节“待 CI”为实施时记录，03 正式完成记录保存 SHA/树/artifact。

04 沿用原 Application target：两项原清单/审计测试迁入唯一目录且断言保持，新增八项测试，预期 Application 77（69+8）/Persistence 141。覆盖公开固定清单 SHA-256 `178afe68af4d0cb8ba9341a7f5f47ec3b89c4c2b9ceaafd0e6615db3a9a0fb87`、原字段与 UTF-8/对象序列化、五个运行字段的精确排除层级、原输入不修改、完整输入指纹与清单指纹的区别、纳秒 cutoff/阵容贡献/质量/路由/比赛/窗口的指纹敏感性、数组顺序/null 与缺失、受检重建/附加/摘要的错误顺序和可选元数据、两种模式下篡改先于 predict/save 拒绝。固定平台载荷不是私有模型概率 Golden Master。

六项原函数及两项原测试逐段等价。六探针（放宽排除、修改原输入、改变序列化字节、跳过 stale 哈希、跳过摘要复核、执行绕过审计）均拒绝并恢复。83 项现有源码检查、完整 `npm run verify:architecture`、Rustfmt、18 保护资产/171 命令/46 迁移与 18 PG 静态基线及 `git diff --check` 全 PASS。原本地 Rustfmt 运行库截断导致 loader/SIGBUS，已从现有相同版本归档完整恢复并通过格式化及 --check，无依赖/锁文件变更；这不是 Rust 编译验收。

5 项浏览器检查、Rust 编译/Clippy/tests、Windows 交付仍待 04 自身精确 CI。没有 Linux/macOS 动态验证、新 runner/workflow/target/数据库；真实 PG/历史四项/账本/XLSX/Full/私有固定回归和继承模型历史删除 trigger 风险继续最终封包新库待验，ignored 不计 PASS。


## R8-04 Windows 收尾 / R8-05 实施时验证（2026-10-02；已通过，见下方）

精确 `59c567912194e232b29f36ecf94c3a26c969ed06` / run `36966815323` / job `110712250872` 全 SUCCESS；日志确认 Application 77/Persistence 141、input_manifest 十项测试、17 视口、前端类型/构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 和启动 7 条记录/3 操作通过。上节待 CI 是实施时记录，04 正式完成记录保存树和 artifact。

05 在原 Application target 迁移两项原选择测试，新增八项行为测试：选择空白/大小写/家族及精确注册、UTC/纳秒/模拟身份/名称及原输入身份、字段错误先于模型选择、显式类型覆盖/identity/参数、快照精确成员及十一字段路由变化、公开默认载荷、预览只读与 Port 错误顺序、自动/显式类型在后续检查前的行为。复用原 Probe，预期 Application 85（77+8）/Persistence 141。fake 只验证编排，不冒充私有模型固定概率。

逐段比较十二原 helper、显式覆盖、两个请求组装及两原测试等价；提取调用重新内联后原执行器/default 完整函数等价；preview/build/readiness 除 import 外等价。六项破坏探针（家族漏 trim、显式不覆盖、参数身份回退、快照放宽 trim、绕过受审计路由、执行跳过上下文校验）全部被原增强 verifier 拒绝并恢复。

本轮只执行原源码/架构/格式/资产/命令/迁移静态门禁；5 项浏览器检查、Rust 编译/Clippy/tests 和 Windows 交付等待 05 精确 CI。没有 Linux/macOS 动态验收、新 runner/workflow/target/数据库、依赖或迁移。PG/历史四项/账本/XLSX/Full/私有固定回归及模型历史删除 trigger 风险仍最终封包新库待验，ignored 不计通过。


05 本地实际 PASS：83/83 原源码检查（报告 `r805-source-checks.json`）、完整 `npm run verify:architecture`（`r805-architecture.log`）、Rustfmt --check、`git diff --check`、18 保护资产、171 命令及 46 迁移/18 PG 静态基线。Application 文件扫描 384→388、Domain 1040→1044，43 traits/Port SHA、365/300 和声明摘要保持；Rust 编译/测试与 Windows 浏览器/交付结果尚未实跑，不计通过。


## R8-05 首轮 Windows CI 修正（2026-10-02；修复已通过）

精确 `1f1613d4578b8b8f23f2a458cedc213d072e65b9` / [run 36970160631](https://github.com/uniquenesssta/123/actions/runs/36970160631) / job `110722266615` 失败：Windows Clippy 普通 lib 编译报告 route_model_request/mod.rs 的 `ensure_match_input_id` re-export 未使用，`-D warnings` 将其提升为错误。源码检查和 Windows 前端契约/类型/截图/生产构建已通过；Rust tests、release/安装包/启动尚未到达，不能计通过。

根因是提取后实际请求组装已在 request.rs 内部调用该 helper，目录级导出只有 lib/tests 与模块单测消费。修复将该单一 re-export 限定 `#[cfg(test)]`，生产 helper 及请求组装保持；其余生产导出均有真实非测试调用方。既有 Prediction verifier 增加 cfg 及生产出口限制，去掉 cfg 的破坏探针须被拒绝。无 `allow(unused_imports)`、公共 API/行为/测试数量、模型/资产/依赖/迁移/数据库变化。修复后沿用原静态门禁及 Windows CI，05 仍 VERIFYING；不得关闭或启动 06。Context7 按 Rust 1.88.0 对照 cfg(test)/use 语义，实际动态结果以新精确 SHA 为准。

本轮修复实际 PASS：83/83 原源码检查、完整 `npm run verify:architecture`、Rustfmt --check、18 保护资产、171 命令、46 迁移/18 PG 静态基线、`git diff --check`；移除 cfg 的单项破坏探针已拒绝并恢复，十项生产导出逐项确认非测试消费。报告 `r805-fix-source-checks.json` / `r805-fix-architecture.log` 位于本轮工作目录；未执行本地 Cargo 或客户端动态验收，Windows 结果待新精确 CI。此修复全部六个文件仅修改，无新增/移动/删除；Domain 使用清单的扫描指纹随出口源码刷新，365/300、声明摘要与其余冻结内容保持，生产函数和测试未变。

首次本地检查提示 Domain 使用清单扫描指纹过期，已运行原 `generate-domain-type-inventory.mjs` 并核对只有 usage scan 指纹变化，随后重跑原检查；无类型/接口变更。


## R8-05 Windows 验收与收尾（2026-10-08 核实）

精确 `2aa99a76cb97bb289fd486bb8e3ed5059620fb88` / [run 36971419476](https://github.com/uniquenesssta/123/actions/runs/36971419476) / job `110726001870` 全 SUCCESS，CI 完成时间 2026-10-02 14:22:38（北京时间）。实际通过 Application 85/Persistence 141、route_model_request 十项单测（保留2+新增8）、17 视口、前端契约/类型/生产构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 和启动 7 条记录/3 个完成操作。

artifact `11212471101`，14,023,932 字节，SHA-256 `1cb00b0a91d1bbf25270dc8fb535b0405e0280b60595a313f0bc74f4503a3538`。首轮 unused re-export 已由 cfg(test) 修复；前节待 CI 和失败记录仅属实施历史，不代表当前结果。详细差异/兼容/验证见 [05 完成记录](modular-rewrite/R08-prediction-p4-orchestration/R08-05-route-and-model-request.md)。

本轮仅文档收尾，源码/测试/清单不变，复用上述已验证源码证据；只核对相对链接、状态、Git 差异和源码不变，不重复非 Windows Cargo/客户端动态或全量构建。PG/历史四项/账本/XLSX/Windows Full/私有 Golden Master 与继承 model.runs/0041 删除 trigger 风险仍最终新库待验，ignored 不计 PASS。05 完成，06 READY 尚未实施。


## R8-06 实施验证（2026-10-08；实施时记录，已通过见下方收尾）

R8-05 的精确代码 run `36971419476` 和收尾文档 run `37737677787` 均 SUCCESS。06 不继承其动态 PASS。新模型适配器在原 Application 单测 target 内增加 7 项测试：精确注册/Arc 身份与缺失错误、实际 context/原 scope 提示、完整请求/输出保真、五类模型错误与单次调用、毫秒截断/饱和、默认 dry run 省略 supports/validate、路由失败先后顺序与不写历史。预期 Application 92（85+7）/Persistence 141；以本次精确 Windows 日志为准，不把静态解析计为 Rust 测试通过。

实际本地 PASS：现有 verify-frontend 清单的 83/83 源码项（5 浏览器项留 Windows）、`npm run verify:architecture`、Rustfmt 1.88.0 对 adapter/测试/登记/两调用方的 `--check`、`node scripts/verify_protected_assets.mjs`、`node scripts/verify_command_contract.mjs`、`node scripts/verify_database_baseline.mjs`、`git diff --check`。六项临时破坏探针均由增强的原 Prediction 门禁拒绝并恢复；调用方重新内联比较确认原顺序/审计/保存/输出不变。报告 `r806-source-checks.json`、`r806-architecture.log` 位于本轮工作目录；原命令可重现。

执行中首次静态门禁发现格式化后的 usage scan 过期，以及原组合根文件集合未登记 adapter；已用原生成器刷新清单并精确登记文件，复跑通过。现有 Rustfmt 运行库在本地不可读，恢复同版本工具后目标格式检查通过；未运行 Linux/macOS Cargo 或客户端动态检查。Windows CI 复用原 workflow 完成完整 frontend、17 视口、fmt/Clippy/workspace tests、release/MSI/NSIS/启动，启动后停止轮询。06 VERIFYING，07 未开始；详见 [实施记录](modular-rewrite/R08-prediction-p4-orchestration/R08-06-model-execution-adapter.md)。

没有新 workflow/runner/test target/数据库/依赖，公共接口、模型算法/参数/18 资产、43 Ports、171 命令、365 Domain/300 映射及迁移0001～0046保持。伪提供器只验证编排边界；私有固定比赛/Golden Master、真实 PG/历史四项/账本/XLSX/Windows Full 和继承历史删除风险仍最终新库待验，ignored 不计 PASS。

## R8-06 精确 Windows 验收与收尾（2026-10-08）

实施提交 `09693318f8a7c8a557a15b89ef3a81e7ea391b11` 的 [Windows run `37752995641`](https://github.com/uniquenesssta/123/actions/runs/37752995641) / job `113230527909` 全 SUCCESS，完成于 2026-10-08 17:12:32（北京时间）。核对精确 SHA、全部步骤、完整日志和 artifact；前节预期数及待 CI 状态仅属实施历史。

Application **92** / Persistence **141**、七项新增 adapter 测试、前端契约/类型/生产构建、**17** 视口、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 和客户端启动/状态载入全部通过。运行日志 **7 条记录 / 3 个完成操作**通过；报告 `logs/windows-acceptance-20261008-085549.json`。artifact `11539138019`，14,023,860 字节，SHA-256 `c0e37e6bb7c706aa292215859844a93491266978dc88f4819abd93b3fb568dd3`。详见 [06 完成记录](modular-rewrite/R08-prediction-p4-orchestration/R08-06-model-execution-adapter.md)。

本轮仅五份现有文档收尾，源码/测试/清单不变，复用已验证证据并检查链接、状态及差异，使用 `[skip ci]` 不重复全量构建。06 DONE；07 READY 尚未实施，08～12 BLOCKED。ignored PG/contract、真实 PG/历史四项/账本/有效 XLSX/Windows Full/私有 Golden Master 和继承历史删除风险继续最终封包新库待验，不计实跑通过。

## R8-07 实施验证（2026-10-08，VERIFYING）

07 从 `eadb49d51722f50349509a0402c5a925402a53dc` 开始，沿用唯一 R8 分支。旧 model_runs.rs 的九项生产函数、原一项审计测试、全部 SQL literals 和四个公开签名分别比较保持；唯一 runs owner 分为 write/input/details/read/visibility，mod 只登记/导出。原 Application 正式保存/影子 nil、Port、错误映射及结果组装保持。没有为了路径增加空转发层。

在原 Persistence target 保留原审计测试，新增七项边界测试：legacy 审计 shape/hash/null、受审计身份 trim/四等级/原可选分值语义、错误优先级/null manifest、快照身份/纳秒窗口/原 schema、缺元数据/身份变化拒绝、质量闭区间与类型、模型明细整数/浮点/缺字段错误。原 Application 保存失败测试扩为六类 Port 错误，验证原 kind/message、一次保存、不返回成功及无重试。预期 Persistence **148**（141+7）/Application **92**；尚未 Rust 实跑，须本项精确 Windows CI 确认。

原 `model_run_identity_repository_contract` target 增加公开 API 链路：第二比分的 smallint/DB 概率约束失败后，run/snapshot/modules/scorelines/completion audit 五类计数共同归零；两次成功产生不同 run 并复用一个 runtime snapshot；完整输入/输出/summary/explanation/hash/audit/route/duration 读回、模块细节/比分默认值/每次一条完成审计、历史 topscore/name/limit、重复隐藏保留首次时间和两次审计、隐藏后 read_run、缺失运行与输入 immutable trigger。保留原完整及 nullable 身份用例。仍 ignored，没有真实 PG 通过结论，18 broad targets 数量不变。

本地实际 PASS：现有 verify-frontend 清单 **83/83** 源码检查（5 浏览器项留 Windows）、完整 `npm run verify:architecture`、同版本 Rustfmt 1.88.0 目标 `--check`、`node scripts/verify_protected_assets.mjs`（18）、`node scripts/verify_command_contract.mjs`（171）、`node scripts/verify_database_baseline.mjs`（46 迁移/18 PG 静态契约）及 `git diff --check`。六项破坏探针（提前提交、快照复用来源放宽、绕过 audit hash、放宽 quality 上界、去掉 hidden 过滤、影子写正式历史）均被原增强 Prediction gate 拒绝且文件恢复；健康门禁复跑 PASS。格式化后的回滚断言换行曾使新增静态断言不匹配，已保留断言内容并用允许空白的正则检查，复跑通过。

报告 `/workspace/scratch/0c69084e7a7f/r807-source-checks.json` 和 `/workspace/scratch/0c69084e7a7f/r807-negative-probes.json`；命令由仓库原入口可重现。Rustfmt 本地运行库缺损从已有同版本归档完整恢复后检查通过，没有项目依赖变更。Application 扫描 390 不变；Postgres 直接模块 36→35（删除根 model_runs，runs 在既有 prediction 下）；Domain 扫描 1046→1051，365 类型/300 映射与声明摘要保持，usageDigest `57653c49ff4d861f79dbb439e83e066899455c6aeb98f94cc5f4474253c484d2`。

本项复用原 Windows Automated：完整前端/类型/生产构建、17 视口、fmt/Clippy/workspace tests、Windows release/MSI/NSIS/启动。CI 启动后停止轮询；未运行 Linux/macOS Cargo/客户端动态验收，无新 workflow/runner/target/数据库/依赖或迁移。07 VERIFYING、08～12 BLOCKED。PG/历史四项/账本/有效 XLSX/Windows Full/私有 Golden Master 和继承 model.runs/0041 删除风险继续最终封包新库待验；ignored 编译不计 PASS。详见 [07 实施记录](modular-rewrite/R08-prediction-p4-orchestration/R08-07-run-persistence.md)。


## R8-07 精确 Windows 验收与收尾（2026-10-08）

精确实施 `58b390a6400646ac6131dfef0503a49ce1128eca` / [Windows run `37796909083`](https://github.com/uniquenesssta/123/actions/runs/37796909083) / job `113378629485` 全 SUCCESS，完成于 2026-10-08 23:27:02（北京时间）。完整日志确认 Application **92** / Persistence **148**、七项新增边界及原审计测试、前端契约/类型/生产构建、**17** 视口、fmt/Clippy/workspace tests、Windows release/MSI/NSIS 和启动 **7 条记录 / 3 个完成操作**通过。报告 `logs/windows-acceptance-20261008-145959.json`；artifact `11559974847`，14,025,581 字节，SHA-256 `b31ac49010ca525cd6c836fa9746be451ed5dc72bc8e918522e6ecad7e22f2ac`。

已核对精确 SHA、全部 job/steps、日志与 artifact；实施时预期/待验文字保留为历史记录。07 正式 **DONE**，08 前置通过；用户已授权“收尾07开始08”，沿用唯一 R8 分支。收尾仅同步五份文档，源码/测试/清单保持上述已验树，链接/状态/diff 核对，不重复全量构建，文档提交使用 `[skip ci]`。真实 PG/历史四项/账本/有效 XLSX/Windows Full/私有 Golden Master 与继承 model.runs/0041 删除风险继续最终封包新库待验，ignored 不计实跑 PASS。


## R8-08 实施验证（2026-10-08，VERIFYING）

08从07已验源码文档基线 `1db0e195cdbc26514d7ad015b4110a53a446ba26` 开始。原账本追加/冲突、声明前检/指纹、版本引用、Row及共享验证状态、P4共用幂等3函数共10生产函数迁出，原证据测试迁入。冲突纯前检提取后重新内联比较等价，原其余34函数、31SQL和2公开签名保持，Application/Ports/composition无生产改动。

原Persistence target新增8项：六状态来源字段/空字符串原策略，必填/错误优先级与原文，retrieval纳秒边界，全部身份/出处/版本/时间/metadata指纹和retry-key策略，冲突BTreeSet去重/排序/指纹，240字节幂等键及不同成员下限，六种状态映射和未知错误，精确指纹重试错误。保留原一测试，预期Persistence **156**（148+8）/Application **92**；尚未Rust实跑，必须08精确Windows确认，不继承07。

原PG `p4_stage_c_writes_are_idempotent_and_frozen_history_is_immutable` 夹具扩完整声明字段和微秒落库读回、metadata/纳秒身份变化拒绝、原conflict排序/重复/metadata排除策略、run/match/schema/prompt/conflict引用和source FK失败后无声明/审计、首次并发同键返回同UUID/一次审计、成员缺失/身份不符/重复最小集合失败无头/成员/事件/审计、并发冲突2成员1opened1审计、四类账本mutation拒绝。原快照cutoff/31字段/4链/immutable断言保持。18 broad仍ignored，编译不计PG实跑PASS；真实PG/历史四项/账本/有效XLSX/WindowsFull/私有GoldenMaster与继承删除trigger风险最终封包新库待验。

实际本地PASS：原83/83frontend源码项（5浏览器项留Windows）、`npm run verify:architecture`、12 changed Rust targets的Rustfmt1.88.0 --check、`node scripts/verify_protected_assets.mjs`、`node scripts/verify_command_contract.mjs`、`node scripts/verify_database_baseline.mjs`、git diff --check。六项临时破坏探针均由原增强Prediction gate拒绝并恢复；健康门禁复跑PASS。初次architecture仅新增模块数量清单未同步，修正35→36后完整通过。无Linux/macOS Cargo/客户端动态、新runner/workflow/target/数据库/依赖/迁移。

报告 `/workspace/scratch/0c69084e7a7f/r808-source-checks.json`、`r808-architecture.log`、`r808-negative-probes.json`、`r808-equivalence.json`。Domain扫描1051→1060，365/300/声明摘要保持，usageDigest `307b38c0ae795e65133ae11522339b21a47bd8ab5cc1129f02f679ca0ba97608`；Application390/43Port保持。PG runtime_sources只更新原postgres_integration blob `eb0f4dd987b3f4374a57c1f5295c8be0ffaf9966`，迁移摘要不变。08VERIFYING，09～12BLOCKED；原Windows全链路CI启动后停止轮询，详见 [08实施记录](modular-rewrite/R08-prediction-p4-orchestration/R08-08-p4-evidence-ledger.md)。


## R8-08 精确 Windows 验收与收尾（2026-10-09）

修复 `52f23ab4e582f313f412058623f0d83a6ddf6f98` / [Windows run `37817443918`](https://github.com/uniquenesssta/123/actions/runs/37817443918) / job `113449572979` 全SUCCESS，前端/类型/生产构建、17视口、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS、启动状态载入和日志覆盖通过。首轮 `91e9585` 的未使用导入失败已关闭。报告 `logs/windows-acceptance-20261008-173456.json`，artifact `11568918565` / 14,023,349字节 / SHA-256 `d52b465dd73216453496a5a7b079ad6d331ebad3293c58af606e1187bc57acb8`。

08DONE；用户已授权09开始，09不能继承08动态PASS。08收尾仅同步既有五文档并使用 `[skip ci]`，无源码/清单变化。真实PG、历史四项/账本、有效XLSX、Windows Full、私有Golden Master和继承历史删除风险继续最终封包新库待验；ignored不计实跑PASS。


## R8-09 Fact Pipeline 实施待验（2026-10-08）

08精确修复 `52f23ab` / run `37817443918` 全SUCCESS并文档收尾为 `6718c99`；09已实施、VERIFYING，10～12BLOCKED。沿用原Windows Automated和原单测/PG入口，不新增target、runner、workflow、依赖或库。

原Application target保留9项事实纯helper测试，新增7项异步主链/错误/来源/主客队/截止/冲突边界；原Persistence target保留2项事实helper测试，新增7项持久化状态边界、精确指纹、候选等分和来源策略前检。预期Application99/Persistence163只作实施预期，09精确Windows实跑前不计PASS。Port错误测试包括六类错误与冲突创建/评估/事件/路由失败，检查即时停止与原kind/message。无pipeline整体原子回滚承诺。

原postgres_integration的Stage C扩context字段及微秒落库、主队候选、四类记录首次id/time/fingerprint/内容变化拒绝、路由证据ID排序去重、事件唯一及不可变；Stage E扩首次metadata保留/唯一audit/无效策略零残留。18 broad仍ignored；真实PG/历史四项/账本/XLSX/Windows Full/私有固定回归及继承删除风险最终封包新库待验，编译不冒充实跑。

本地静态PASS：83源码检查、完整architecture、25目标Rustfmt、18保护资产、171命令、46迁移/18PG静态基线、diff；六破坏探针拒绝并恢复。生产17函数/8签名/18原SQL及208字符串、32Application helper/注册/命令/重新内联主链等价核对通过。5浏览器项及Rust fmt/Clippy/workspace tests和Windows release/MSI/NSIS/启动由09自身CI执行，启动后停止轮询；无Linux/macOS动态验收。完整文件/行为/验证及回退见 [09实施记录](modular-rewrite/R08-prediction-p4-orchestration/R08-09-fact-pipeline.md)。


## R8-09 精确Windows完成（2026-10-09）

精确实施 `4025781f2f0419f374edfa7669a4c5bcfe71ee66` / [Windows run `37825126808`](https://github.com/uniquenesssta/123/actions/runs/37825126808) / job `113475889770` 全SUCCESS，完成于2026-10-09 02:56:31（北京时间）。日志确认Application **99** / Persistence **163**、14项新增事实边界测试、前端契约/类型/生产构建、**17**视口、fmt/Clippy/workspace tests、Windows release/MSI/NSIS与启动 **7条记录 / 3个完成操作**全部通过。报告 `logs/windows-acceptance-20261008-183420.json`；artifact `11571947799`，14,024,945字节，SHA-256 `5030ebbe6a3a7ffa551c404a7733297e555faf44c21776bdfee34c7c616cdd9e`。

09正式DONE；14新边界测试实际通过，99/163不再只是预期。五文档收尾复用同源码证据，不重复全量构建；用户授权10开始，须10自身WindowsCI。18 PG broad仍ignored，真实PG/历史四项/账本/XLSX/Full/私有Golden Master与继承删除风险最终新库待验。

## R8-10 Horizon Orchestration 实施待验（2026-10-09）

09精确Windows已实际通过99/163，文档收尾7c1ccd3；10独立VERIFYING，11/12 BLOCKED。复用原Application target/Probe新增8项规划测试：三个正式时点/固定路由Schema29事实及原队列政策、非PLANNED重试无写、前检拒绝/规范化、全部六版本与事实身份漂移、过期任务、八Port边界×六错误kind/message、首建后入队失败恢复、绑定失败复用job。queue原生产模块另补纳秒前/相等/后的截止恢复与非PLANNED无写；后台原target新增6项结算/完成错误/未耗尽与耗尽失败/终态或读取失败/尽力迁移和队列错误优先级测试。Application源码预期99+15=114，尚未在本项Windows实际执行。

Persistence原target新增6项纯职责测试：正式写入/兼容读取区别、空事实拒绝、排序去重指纹、固定身份变化、纳秒输入区别、12状态/5时点/6研究状态的精确解析。源码预期163+6=169，需本项实跑。原postgres_integration Stage C（18 broad数量不变）增加context/Schema/run读回、任务首次值/实际微秒/原始纳秒指纹、同键重试与并发、错指纹拒绝、兼容时点零残留、FK创建/状态更新失败回滚恢复、原队列绑定、同状态无写、expected/非法迁移冲突、三时点排序、事件唯一/不可变与共同成功审计。该target仍ignored，编译不计真实数据库PASS。

本地PASS：83/83原源码门禁（5浏览器项交Windows）、完整npm run verify:architecture、23目标Rustfmt 1.88、18保护资产/171命令/46迁移与18PG静态契约、git diff --check。六破坏探针为非正式时点、漏身份、错PLANNED恢复条件、错队列key、去FOR UPDATE、漏成功审计，全部拒绝恢复。静态批次在刻意移除FOR UPDATE期间读取到一次单项失败，恢复后串行重跑该项通过；首次Mapping gate指向原根context已修正真实owner，完整架构复跑通过。23原Postgres函数（重新内联preflight）、规划prepare/identity/draft/未来队列、dispatcher/terminal failure/settlement比较保持；PLANNED恢复明确属于缺陷修复，不伪称全行为等价。

### R8-10 首轮编译失败与导入修复

精确 `db5fd511518f50e8ab7a16ef8d693b0585bc552e` / [run `37884818742`](https://github.com/uniquenesssta/123/actions/runs/37884818742) / job `113672333836` 为FAILURE。日志唯一Rust编译错误为 `crates/persistence-postgres/src/p4_orchestration.rs:83` 的 E0433：`ResearchRunStatus` 未导入。其解析函数已移入horizon/context，但根中保留的readiness仍需该enum；子模块use不提供父模块名称绑定。该轮完整前端、TypeScript、Vite和17视口已通过，fmt通过后Clippy编译失败，workspace tests、Windows release/MSI/NSIS及启动未完成，114/169仍未在本项实际验证。

仅补回根生产导入、扩展原Prediction gate并刷新Domain usage摘要；8个相关Postgres生产文件的原Domain类型依赖复核无其他同类遗漏，五个保留函数体/SQL与失败head逐字保持。移除该导入的单项破坏探针被拒绝，恢复后通过；报告 `/workspace/scratch/eb298ad5cdcb/r810-import-fix-check.json`。本次复核完整architecture、Prediction专项、Domain清单、Rustfmt 1.88、源码卫生、18保护资产/171命令/46迁移与18PG静态契约和diff；不新增Rust测试、target或Clippy抑制，预期114/169不变。Context7按实际1.88.0/edition2021核对官方Rust Reference模块use作用域；依赖/API/可见性不变。新精确Windows CI启动后停止轮询，10仍VERIFYING，11/12 BLOCKED；PG等原动态待验保留。

报告：`/workspace/scratch/eb298ad5cdcb/r810-static-checks.json`、`r810-negative-probes.json`、`r810-equivalence.json`。Application393→399、Domain1071→1084，43 Ports/365/300/声明摘要不变；Postgres根35保持，数据库清单只刷新原PG test blob。复用原Windows Automated执行frontend/17视口/类型构建、fmt/Clippy/workspace tests、Windows release/MSI/NSIS/启动，启动后停止轮询。无Linux/macOS Cargo/客户端动态或新基础设施；真实PG/历史四项/账本/有效XLSX/Windows Full/私有Golden Master与继承删除风险继续最终封包新库待验。详见 [10记录](modular-rewrite/R08-prediction-p4-orchestration/R08-10-horizon-orchestration.md)。


## R8-10 精确Windows完成（2026-10-09）

精确修复提交 `481bfcb967349afedbf7eac44959f2f8a020744b` / [Windows run `37886029199`](https://github.com/uniquenesssta/123/actions/runs/37886029199) / job `113676124080` 全SUCCESS，完成于2026-10-09 13:22:30（北京时间）。完整日志确认Application **114/114**、Persistence **169/169**，10新增15项Application和6项Persistence测试均通过；前端契约/类型/生产构建、**17**视口、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS、启动 **7条记录 / 3个完成操作**全部通过。报告 `logs/windows-acceptance-20261009-045537.json`；artifact `11596878906`，14,033,721字节，SHA-256 `10c5c6e2cc39ffeb13ac964a5dd5200adff62d547cd63b06429dda7da02624c2`。

10正式DONE；21项新增测试及114/169已经实际通过。五文档收尾复用已验源码证据；用户授权11开始，须11自身Windows CI。18 broad PG仍ignored；原真实PG/历史四项/账本/XLSX/Full/私有固定回归和继承删除风险继续最终新库待验。

## R8-11 Workbench Reads 实施待验（2026-10-09）

新增四项Application工作台委托测试：精确身份与DTO/顺序、可空赛事，空与丰富的研究/来源/冲突/路由/事件和进展/终态视图，两个读取边界的全部六种PortErrorKind/message原样停止，未选Port调用立即失败且无任务/队列写入。预期Application **118**（10已验114+4）/Persistence **169**不变，本项Windows尚未实跑，不冒充PASS。

原PG Stage C复用同一TestDatabase/已注册版本、原31字段/概率快照，补比赛/任务缺失提示、NULL赛事和空研究集合、证据17字段/微秒时间/NULL来源、三字段排序、其他run证据/冲突排除、事件原序、人工路由与任务隔离、最新全局冲突事件、完整快照及研究次数/响应/模型/错误/起止时间；重复和未知ID读取保持12个任务/队列/研究/证据/快照/审计账本计数。人工夹具遵守0018的未来cutoff、RESEARCH_PARTIAL、共享trace、manual_required及唯一(task, conflict)，两个任务各有一个不可变决策；未禁用trigger或修改迁移。18 broad仍ignored，真实执行最终新库待验。

83项原前端源码检查、完整 `npm run verify:architecture`、Prediction/Domain/源码卫生、保护18指纹（聚合d74e0936…）、171命令、46迁移/18PG静态契约（聚合d9f2eb50…）、Rustfmt1.88 `--check` 与 `git diff --check`均PASS。七探针：任务上限、就绪度顺序、证据run过滤、人工task范围、最新事件ID排序、NULL证据数组、writer导入，均被原Prediction门禁拒绝且恢复。提取核对确认四SQL逐字、五writer/比赛函数/两Application生产入口与原逻辑等价，三个查询投影和任务汇总重内联保持。

报告 `/workspace/scratch/eb298ad5cdcb/r811-static-checks.json`、`r811-architecture.log`、`r811-negative-probes.json`、`r811-equivalence.json`。Domain使用扫描1084→1090，365/300/sourceDigest保持，usageDigest `f0d81ca6c8f8d56746b800dea3e4b9d44680643f584db3ceb248add79d759751`；Application399/43Ports与Postgres根35保持，PG runtime_sources只刷新原测试blob `d422b3ac0fd7acbc17794e1edc2f2c00f717f523`。无依赖/API升级、target/runner/workflow/数据库新增，无Linux/macOS动态验收。原Windows Automated执行完整frontend/类型/构建/17视口、Rust/Clippy/workspace tests及Windows交付；确认启动后停止轮询。11 VERIFYING，12 BLOCKED；真实PG/历史四项/账本/XLSX/Full/私有固定回归及继承model.runs/0041删除风险继续最终新库待验。


## R8-11 精确Windows完成（2026-10-09）

精确实施提交 `47bda231575a9179cd629367d5a3273cd2ab654a` / [Windows run `37891346613`](https://github.com/uniquenesssta/123/actions/runs/37891346613) / job `113692742591` 全SUCCESS，完成于2026-10-09 14:27:11（北京时间）。完整日志确认Application **118/118**、Persistence **169/169**，四项新增Application工作台测试通过；前端契约/类型/生产构建、**17**视口、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS、启动 **7条记录 / 3个完成操作**全部通过。报告 `logs/windows-acceptance-20261009-060211.json`；artifact `11599306895`，14,034,588字节，SHA-256 `86214d994c421fa9c50a1d9c4638aa7af116361ee3a7e8d14e696106029912ae`。

11正式DONE；四项新增测试及118/169已经实际通过。五文档收尾复用已验源码证据；用户授权12开始，须12自身Windows CI。18 broad PG仍ignored；原真实PG/历史四项/账本/XLSX/Full/私有固定回归和继承删除风险继续最终新库待验。R8尚未完成，不提前创建阶段完成记录或启动R9。

## R8-12 Freeze Transaction 实施验证（2026-10-09；以下为实施时记录，现已通过）

新增原Application target八项：完整冻结顺序与全部锁定身份/溯源/31字段/质量分数/外部单链，七个末段Port边界×六种kind/message停止，四路由身份/Schema/空矩阵/非法概率/空比分漂移阻断，快照已提交而FROZEN登记失败后的只读恢复；纯投影另覆盖原路由顺序/原值/证据排序去重与CONFLICT/STALE、外部拓扑/正式覆盖/clean-sheet/矩阵原字节哈希、矩阵缺失/非法/0和65536比分边界。四项原快照Persistence测试原样迁入input，新增六项前检/指纹测试：交付元数据原排除、14类不可变身份/原载荷变化、只排序副本而保留原键/证据列表、完整31序号/trim唯一、有限概率/1e-9和/小写64哈希/optional/u16、正式时点/截止相等及240字节键。源码预期Application **126**（118+8）、Persistence **175**（169+6），须本项Windows实跑。

原PG Stage C在同一TestDatabase/版本/研究/证据/快照夹具上扩重复证据链接SQL失败与最后概率JSON零字符SQL失败，核对快照头/字段/证据/概率/审计完全无残留；同键并发仅一个created、一份31字段/证据/四概率/审计；原顺序翻转与frozen_at变化重试、异键同正式队列复用/原幂等键保留、不同载荷拒绝不写账本，完整原JSON/字段/排序概率/正式与shadow flags读回。原published/effective截止+1微秒、研究截止差异与不可变更新/删除断言保留。18 broad仍ignored，无新trigger/Schema/target/runner/数据库；真实PG实跑最终新库待验。

83原源码/完整architecture、Prediction/Domain/源码卫生、18保护资产/171命令/46迁移/18PG静态契约、Rustfmt1.88 --check和diff均PASS。七探针：脱离明细、pool写、证据截止变包含边界、删除完成时钟、Schema/路由身份绕过、正式标记改变，全部拒绝并恢复。等价核对：两编排函数/七投影函数、26原Postgres函数逐token保持；拆出明细重内联后原快照事务保持；260条SQL/消息/哈希/审计literal与原八测试/fixture函数保持。

报告 `/workspace/scratch/eb298ad5cdcb/r812-static-checks.json`、`r812-architecture.log`、`r812-negative-probes.json`、`r812-equivalence.json`。Domain扫描1090→1099、usageDigest `cf79a5f2a44e5cae6a3ad16545150b6c32db7faae8c3693217956a993d7ea53c`，365/300/sourceDigest不变；Application399→402、43Ports和Postgres根35保持。PG runtime_sources仅原测试blob更新为 `33e58c3275014c1adf99f94a2f938c2dbff1fa75`。无依赖/API升级、迁移或数据库基础设施变化；不执行Linux/macOS动态。原Windows Automated确认启动后停止轮询；12 VERIFYING，R8 IN_PROGRESS、R9不开始。原真实PG/历史四项/账本/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险继续最终新库待验。


## R8-12 精确 Windows 验收及 R8 阶段出口（2026-10-09，DONE）

精确实施提交 `af3c98c31e28a332fe19ec47f4ed11c3a5a261ab`，源码树 `a911da88c084e9f6abfaf25a1a1909567a6eb166`；[Windows run `37899786755`](https://github.com/uniquenesssta/123/actions/runs/37899786755) / job `113719388521` 全 SUCCESS，完成于 2026-10-09 15:58:10（北京时间）。完整日志确认 Application **126/126**、Persistence **175/175**，本项新增八项 Application、六项 Persistence 及四项原快照测试均通过；前端契约/类型/Vite、**17** 视口、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 和启动 **7 条记录 / 3 个完成操作**通过。

报告 `logs/windows-acceptance-20261009-073548.json`；artifact `11603095137`，名称 `windows-automated-delivery-evidence-af3c98c31e28a332fe19ec47f4ed11c3a5a261ab`，14,034,892 字节，SHA-256 `d82a43858b5389bc66f998d05aea577e3709b1ec263142f28b5a4892b8d979ef`。已核对 run 的分支/head、全部 job/steps、完整日志及未过期的交付 artifact。

R8-01～12各节点均取得自身精确Windows，最终全workspace与原前端/交付入口在上述head通过。阶段出口额外复核现有保护资产（18项）、命令（171）、数据库静态基线（46迁移/18PG）、Prediction Service职责（48文件/18公开职责）；阶段与收尾范围核对确认 model-api/P4/P7/Domain/research-gateway、迁移/contracts/schemas、依赖/锁文件/workflow和前后端代码均无R8改动。完整文件清单、验收与回退见 [R8阶段完成记录](modular-rewrite/R08-prediction-p4-orchestration/R08-stage-completion.md)。

本轮六修改/两新增均文档；仅验证链接、状态、完整阶段文件清单及diff，不执行非Windows动态、不重复上述成功构建，提交 `[skip ci]`。R9-01只登记READY，未实施。真实 PostgreSQL、历史四项数据库验收、账本/并发/回滚、有效 XLSX、Windows Full、私有 P4/P7 Golden Master，以及继承的 model.runs/0041 历史删除风险继续最终封包新库待验；ignored、公开 unavailable stub 和静态保护指纹均不计真实执行 PASS。


## R9-01 Shared Transport 完成验证（2026-10-09，DONE）

唯一R9分支从c72e559起，原Gateway unit target新增8项transport边界测试，源码预期23（15+8）；原gateway_contract16、Application126/Persistence175保持。HTTP夹具仅有界127.0.0.1：检查三动作方法/URL/UA/Bearer/Content-Type/JSON或无body、HTTP429/请求ID透传、307禁止重定向及请求timeout覆盖响应体。其余覆盖非法URL网络错误、认证非法字符、任意JSON/HTTP状态、空body Null及非法JSON完整原错误。实际单测/loopback只在本项Windows执行，无真实API调用、新test target、依赖或fixture服务。

83/83原源码检查、完整verify:architecture、18保护资产/171命令/46迁移与18PG静态、Rustfmt1.88源码check和diff实际PASS；六破坏探针（redirect、timeout、HTTP方法、空body、错误类别、root导出）均拒绝并恢复。8原HTTP函数逐token等价，decode重内联execute等价、剩余client完整tokens与265生产strings保持。清单登记transport/contract原trait owner、Domain扫描1099→1103/使用摘要，365/300/声明摘要与43Ports保持。

报告 `/workspace/scratch/eb298ad5cdcb/r901-static-checks.json`、`r901-architecture.log`、`r901-negative-probes.json`、`r901-equivalence.json`；实际源/边界/全部文件及回退见 [01记录](modular-rewrite/R09-research-ai-backend/R09-01-shared-transport.md)。reqwest0.13.4官方精确文档核对原请求timeout从连接至body读完，不用Context7 latest替代锁定版本。原格式化缓存截断，已恢复官方SHA校验的同版formatter，仅做源码format/check，不执行Linux/macOS Cargo/单测或客户端动态。

精确实现 `f3da62d4b84235044c7483399f6f8aa145d9438d` / [Windows run `37933341557`](https://github.com/uniquenesssta/123/actions/runs/37933341557) / job `113829199317` 全 SUCCESS（2026-10-09 21:24北京时间完成）；Gateway **23/23**、contract **16/16**、Application **126/126**、Persistence **175/175**，本项8个新增测试全部通过；前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动全部通过。证据artifact `11617684270`，14,032,903字节，SHA-256 `a2178dd81182ec5839db1e0d53b57cc0a40c00802a3f553c159164124ae19c48`。 本次仅五文档 `[skip ci]` 收尾，不重复动态；用户授权02开始，03～11BLOCKED。真实PG、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共stub不计PASS。
