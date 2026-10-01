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
