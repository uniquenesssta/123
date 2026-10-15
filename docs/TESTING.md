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


## R9-02 Credentials与Redaction 实施验证（2026-10-09，VERIFYING）

原Gateway unit target新增12项密钥错误/字节与Unicode、来源/桌面隔离、目标优先级与IO前拒绝、编码/终止/失败、递归模板与公开解析器接入边界。原4凭据/3示例测试保持，源码预期35/contract16，Application126/Persistence175；实际Rust运行仅本项WindowsCI，不读写真实凭据、不修改全局测试环境、不调用真实API。

83/83静态、完整architecture、Rustfmt1.88源check、保护18/命令171/46迁移与18PG静态、diff实际PASS。8探针全部拒绝恢复；8原native函数及provider政策/错误literal保持，四项安全修复与全部A/M/D、等价/报告/回退见 [02记录](modular-rewrite/R09-research-ai-backend/R09-02-credentials-and-redaction.md)。首轮清单漂移已用官方生成器及完整复验解决；无非Windows编译/单测/浏览器动态。报告r902-static-checks.json/r902-architecture.log/r902-equivalence.json/r902-negative-probes.json。

待精确Windows前端/17视口/Rust fmt-Clippy-workspace/12新unit/release-MSI-NSIS/启动；CI开始后停止轮询，02VERIFYING/03～11BLOCKED，不继承01成功。真实PostgreSQL、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险继续最终封包新库待验；ignored与公共unavailable stub不计PASS。


### R9-02 首轮 Windows 失败与修复（2026-10-09）

R9-02 首轮 `98d542f` / [Windows run `37944445186`](https://github.com/uniquenesssta/123/actions/runs/37944445186) 在新增 provider Debug 测试被 Clippy `uninlined_format_args` 拦截；前端/类型/构建和17视口已通过，Rust单测/打包/启动未执行。已改为内联参数，提供者生产实现、原测试断言与35项测试数量保持；既有源码卫生检查增加该回归检查，原写法/换行/尾逗号三探针均拒绝并恢复。83/83静态、完整架构、Rustfmt与保护资产通过；仍VERIFYING，须修复提交自身Windows确认，03～11BLOCKED。

修复本地只做源码/静态：完整83检查、verify:architecture、Rustfmt1.88源码check、18保护资产/171命令/46迁移18PG静态、等价及diff通过。未执行非Windows Cargo/编译/单测，Clippy与实际35/16/126/175、release/MSI/NSIS/启动待修复精确Windows。报告 `/workspace/scratch/eb298ad5cdcb/r902-fix-static-checks.json`、`r902-fix-architecture.log`、`r902-fix-review.json`、`r902-fix-equivalence.json`；完整失败证据及8M清单见02记录。


## R9-02 精确 Windows 收尾（2026-10-10，DONE）

精确修复 `05055d1fd90e9a19ee4bc791e769018ca700dd14` / [Windows run `37954536795`](https://github.com/uniquenesssta/123/actions/runs/37954536795) / job `113901577714` 全 SUCCESS，2026-10-10 00:15:32北京时间完成。Gateway **35/35**、contract **16/16**、Application **126/126**、Persistence **175/175**，新增12项及原凭据/示例/传输测试全部通过；前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动7条记录/3完成操作均通过。证据artifact `11628233815`，14,037,356字节，SHA-256 `9a738defde297f2ff10bc0218885efa72d9e7f2c9ad4ff999bcf237cafc0e45a`。

首轮 Clippy 失败已由该修复自身精确 Windows 关闭。当前02 DONE；用户“收尾02开始03”授权同一R9分支03 READY，04～11 BLOCKED，R9整体IN_PROGRESS。本次只同步既有五份文档，源码/验证器/依赖保持，使用 `[skip ci]` 复用上述精确源码结果；此前VERIFYING叙述为实施历史。真实PG、历史四项/账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored和公共stub不计PASS。


## R9-03 Retry/Circuit/Cancel（2026-10-10，VERIFYING）

R9-03 Retry/Circuit/Cancel已实施、`VERIFYING`，基线59af3f2，同一R9分支。原重试预算/退避/可取消等待、单一熔断state与本地token各归resilience具名职责，旧cancellation.rs及client重复实现删除；协议fallback/工具兼容/远端取消与Semaphore原scope保持。5A/10M/1D，无API/DTO/Schema/依赖/DB/UI/生产策略变化。原unit新增12、contract新增5，源码预期Gateway47/contract21、Application126/Persistence175，须自身Windows实跑；83/83静态/完整architecture/Rustfmt/18保护资产/171命令/46迁移18PG静态与6破坏探针均PASS，原生产体/重路由client/251literal与原16contract等价。

## 原目标新增测试与真实结果边界

原Gateway35 unit与16 contract保留。新增12 unit：retry4覆盖0/10/u32MAX预算、指数0/1/2/11/12/MAX与毫秒溢出、正常/预取消等待、pending取消/丢弃；cancellation4覆盖Send+Sync/克隆/独立、首次poll前取消、注册及未poll多等待者、丢弃与重复取消；circuit4覆盖全部19category、原4计数不看retryable/其余保持、阈值/原完整错误/到期复位、成功复位、8并发失败与u32饱和。超时界限使用原tokio time，未启用test-util或暂停时钟，到期测试仅测试内设置已过期Instant。

新增5 contract通过公开Gateway验证：Formal失败打开三入口共享熔断；两轮成功复位避免非连续失败误熔断；sink取消后停止重试/模型fallback、保留offset8且许可释放；Persistence sink失败先于circuit mutation；挂起IO与并发排队取消后无额外请求、future Drop实际执行且容量恢复。所有请求为原mock或明确pending future，无真实API/OS凭据副作用，无新target或loopback。

本地已PASS的是静态，不是Rust单测。源码预期Gateway47/contract21、Application126/Persistence175须自身Windows实跑。保护聚合 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`，PG静态聚合 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93` 保持。

报告：`/workspace/scratch/eb298ad5cdcb/r903-static-checks.json`、`r903-architecture.log`、`r903-equivalence.json`、`r903-negative-probes.json`。83/83既有静态、完整architecture、Rustfmt1.88源码format/check、原保护资产/数据库静态/命令契约、git diff --check均PASS。6探针分别破坏退避接入/指数上限/计数类别/复位/通知全部等待者/通知创建顺序，原增强兼容传输验证器全部拒绝，逐字恢复后通过。无Clippy抑制或测试删减。

生产三熔断方法体/两个纯helper/原令牌全部tokens保持；client重内联三等待并反向还原owner路由后全体其余tokens保持；原contract16测试及25具名函数集合（含原helper/trait实现）保持，251条生产literal保持。没有声称将不同协议的fallback或工具兼容策略统一；本项明确没有生产政策变化。

## 工具、取舍与继续位置

Context7按Cargo.lock tokio1.52.3核对Notify，但目录仅给latest；准确docs.rs1.52.3页面访问失败，已通过GitHub读取官方tokio-rs/tokio的tokio-1.52.3标签notify.rs，确认Notified在创建后即能收到notify_waiters且不必先poll。原顺序原样迁移，不增加enable/notify_one、新库或依赖升级。Mermaid Chart已展示真实闭合/熔断/到期/在途成功与取消生命周期，不引入半开探针；Create State仅保存核实的足球稳定model96f03270-c236-46de-99fa-db85d2fbf4ce，合成摘要不替代源码及CI。

沿用原Windows workflow，精确head与Windows job确认开始后停止轮询，预计20～30分钟。下一轮先核实03自身结果：成功收尾、失败修相关完整链路；04必须另有授权且门禁通过才能开始。真实PostgreSQL、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041删除风险仍最终封包新库待验；ignored与公开unavailable stub不计PASS。R9整体IN_PROGRESS，无阶段完成记录。


## R9-03 精确 Windows 收尾（2026-10-10，DONE）

精确实现 `d58aa38f88c2b59e3ee3b6f568b54d5430843b29` / [Windows run `37960534145`](https://github.com/uniquenesssta/123/actions/runs/37960534145) / job `113921936420` 全 SUCCESS，2026-10-10 01:05:49北京时间完成。Gateway **47/47**、contract **21/21**、Application **126/126**、Persistence **175/175**，新增12单测/5契约及原测试全部通过；完整前端/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动验收均通过。artifact `11632067812`，14,041,285字节，SHA-256 `71b4a13385cb2f77cb773e0181f2d1987d533148220b657317ad802507450d5b`。

03 DONE；用户“收尾03开始04”授权同一R9分支04 READY，05～11 BLOCKED，R9整体IN_PROGRESS。本次仅既有五文档 `[skip ci]` 收尾，同源码树复用上述精确Windows；此前VERIFYING是实施历史。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险继续最终新库待验，ignored及公共stub不计PASS。

## R9-04 Formal Request（2026-10-10，VERIFYING）

R9-04 Formal Request已实施、`VERIFYING`。正式输入/Schema校验、Responses载荷及请求前预算分别归唯一职责；原连接测试/Plain等复用唯一Token投影与定价政策。9A/9M/0D，无API/DTO/Schema/配置/依赖/DB/UI或生产策略变化；7原函数及其余Gateway编排、247literal/原47unit/21contract保持。新增13单测/4契约，源码预期Gateway60/contract25、Application126/Persistence175，须本项Windows实跑。83现有静态/完整architecture/Rustfmt/保护资产/171命令/46迁移18PG静态及7破坏探针通过。

## 新增行为测试与实际验证

原47 Gateway unit/21 contract保持，新13 unit进入原target：

- 输入4：精确字符/字节边界及来源不修改；空白/超长/非ASCII Schema名称与完整错误；31/32事实、空/重复/100/101字符及raw身份保持；三类Schema根错误与输入错误优先/Schema版本和指令非空。
- 载荷3：可信静态instructions与不可信嵌套context隔离、trace/match/cutoff/事实元数据；根Schema投影/嵌套$id保留与两次构造不修改来源；同步/后台、store、白名单/无filters、两Token别名、reasoning/search/tool/include原政策。
- 正式预算3：每模型初次+retry、相同fallback去重与不同fallback成本；NaN/正负无限/负值、daily→monthly→per-request优先及原缺席价格政策；ASCII/Unicode相同字符量/更多字节的金额阈值及严格超限。
- 共用政策3：精确价格优先、最长dash前缀/非dash拒绝、返回原配置借用；原完整预算错误/recovery/metadata；两Token别名替换、其他字段保留与非object不改。

新4 contract通过公开入口核对：正式错误优先/Chat隔离在key/IO前停止；预算在key/IO前拒绝且CircuitOpen仍优先于预算；resume共享请求校验先于response_id与后台守卫；Extraction→fallback两次attempt8/9保持上下文/Schema副本、嵌套$id、研究字段与原来源不修改。使用原mock及本项计数provider，无真实API/凭据调用、不修改全局环境，无新增test target。

源码预期Gateway **60** / contract **25**、Application **126** / Persistence **175**，自身Windows尚待验。实际本地PASS为83/83现有静态、完整verify:architecture、Rustfmt1.88源码format/check、原兼容传输/Domain/命令/数据库静态/18保护资产与git diff --check；无非Windows编译/单测/客户端动态。保护聚合 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`、PG静态聚合 `d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`保持。

7破坏探针分别移除正式校验接入、31事实改32、不可信标志反转、根$id投影丢失、fallback去重反转、Token别名清理丢失、最长价格前缀改最短，原增强兼容传输验证器全部拒绝；所有源码逐字恢复、原门禁及Domain清单通过。无新回归框架/runner/workflow/数据库入口。

等价核对迁出7函数：校验完整tokens，构造/预算/估算/定价体在仅还原接收参数后tokens保持，两共享helper完整tokens保持；剩余client反向恢复入口路由后全部tokens保持，247条原生产literal、原47unit所有source/21contract及35原具名函数集合保持。公共lib导出保持，仅新增私有模块登记，生产政策变化=[]。

报告：`/workspace/scratch/eb298ad5cdcb/r904-static-checks.json`、`r904-architecture.log`、`r904-equivalence.json`、`r904-negative-probes.json`、`r904-review.json`。本地没有运行Clippy，Clippy及Rust运行以本项Windows为准。

## 工具、取舍与继续位置

Context7按Cargo.lock serde_json1.0.150核对Value clone/Map remove，但仅返回latest；已额外读取官方serde-rs/json v1.0.150的src/value/mod.rs和src/map.rs确认Clone/持有Map/Vec及remove实现，原clone/根remove原样迁移，无API升级或preserve_order feature变化。Mermaid Chart已展示正式执行/恢复、纯校验/构造/预算、共享政策及原IO链路。Create State仅保存本轮已核实足球稳定model96f03270-c236-46de-99fa-db85d2fbf4ce，摘要不替代Git及CI。

实际偏差仅参考目录按职责具体化：定价/预算错误与Token别名是真实跨调用方共用政策，不能放在正式目录让Plain/连接依赖正式私有实现，也不复制。原model_for_operation仍唯一Gateway路由；预算参数primary_model只是借用既有确定性选择，不增加IO/状态或改变拒绝优先级。响应解析/后台状态/实际成本和Plain预算继续原owner，后续05/07按任务处理，不把本项变成整个Gateway重写。

沿用Public Platform CI，确认自身精确head/Windows job开始后立即停止轮询，预计20～30分钟。下一轮先核实04自身结果，成功才收尾，失败只修相关完整链路；05～11仍BLOCKED，R9整体IN_PROGRESS。真实PG/历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master与继承model.runs/0041历史删除风险仍最终封包新库待验；ignored与公共unavailable stub不计PASS。


## R9-04 首轮失败与修复（2026-10-10，VERIFYING）

R9-04 首轮 `16b5d4e` / [Windows run `38021521371`](https://github.com/uniquenesssta/123/actions/runs/38021521371) 的新增协议拒绝契约仅切换Chat Completions，却保留Responses显式端点，Gateway初始化报InvalidConfiguration，未到达该测试断言。前端/类型/构建/17视口及fmt/Clippy通过，Gateway60、Application126、Persistence175通过；contract24/25，release/MSI/NSIS/启动未执行。修复只补该夹具的匹配Chat端点，生产实现及全部断言/测试数量保持；既有兼容传输门禁增加夹具顺序与原断言守卫，缺失/错协议/构造后配置三探针均拒绝恢复。83/83静态、完整架构、Rustfmt/保护资产/命令/数据库静态和等价复核通过；仍VERIFYING，05～11BLOCKED，必须取得修复自身Windows完整SUCCESS。

Windows job `114123347337` 完成于2026-10-10 11:54:24北京，失败于gateway_contract.rs:1283构造器unwrap，错误原文“兼容 API请求端点与所选协议不一致”；不是正式执行入口拒绝结果。修复补显式 `/v1/chat/completions` 端点，原03/04全部生产Rust文件逐字保持，原21契约及新增4项断言保持，源码预期60/25/126/175不变。本次8M/0A/0D，无新target/runner/workflow/DB/依赖/协议；Domain使用摘要更新为 `0b91bb6cd84c76569b6b6e4a67c70f3e690994372350566b67619fadbde9b90c`，365/300/1121扫描与声明摘要保持。报告r904fix-static-checks.json、r904fix-architecture.log、r904fix-equivalence.json、r904fix-source-preservation.json、r904fix-negative-probes.json。原Windows CI精确启动后停止轮询；真实PG/历史四项/账本并发回滚/XLSX/Full/私有固定回归及继承删除风险仍最终新库待验，ignored不计PASS。


## R9-04 精确 Windows 收尾（2026-10-11，DONE）

精确修复提交 `cb3d2cadfb8caa0c7dc3f4eb7c34faedd49f79bc` / tree `8da6d35f383939d61ef4278b6d575f8c8eca1e34`，同一分支rewrite/r9-research-ai-backend，[Public Platform CI run `38055651688`](https://github.com/uniquenesssta/123/actions/runs/38055651688) 与 [Windows job `114223464446`](https://github.com/uniquenesssta/123/actions/runs/38055651688/job/114223464446) 均 **SUCCESS**；完成于2026-10-10 21:53:28北京。前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及运行日志覆盖/错误扫描通过；实际Gateway60/contract25、Application126/Persistence175均通过，首轮失败契约已执行成功。

交付artifact `11671897216` / `windows-automated-delivery-evidence-cb3d2cadfb8caa0c7dc3f4eb7c34faedd49f79bc`，14,040,701 bytes，SHA-256 `b8928c736b0d462199c251a39ff1a9bb821cc23242e36874dcb86d6558275dd3`。验收报告 `D:\a\123\123\logs\windows-acceptance-20261010-132633.json`，过程记录同名.txt；Windows制品为原0.23.0 x64 zh-CN MSI和NSIS setup。原18 broad PG仍ignored；真实数据库、历史四项/不可变账本/并发/回滚、有效XLSX、Windows Full、私有P4/P7 Golden Master及继承model.runs/0041历史删除风险仍最终封包新库待验，ignored与公共stub不计PASS。

首轮失败由该修复自身精确Windows关闭。04 **DONE**；用户“收尾04开始05”授权同一R9分支05 **READY**，06～11BLOCKED，R9整体IN_PROGRESS。本次仅既有五文档 `[skip ci]` 收尾，源码/门禁/依赖保持，同源码树复用上述Windows成功。此前VERIFYING叙述为实施历史，不创建R9阶段完成记录。


## R9-05 Formal Response、Schema 与 Citation（2026-10-11，VERIFYING）

R9-05 Formal Response/Schema/Citation已实施、`VERIFYING`，基线53e49f8，同一R9分支。正式严格解析/解码/公开输出校验/引用关联各归唯一职责，共用引用wire/用量/错误映射仍唯一且兼容调用方不依赖Formal；URL/domain/time原政策留06。12A/12M/0D，无公开API/DTO/Schema/配置/依赖/DB/UI/生产政策变化，17原函数/657生产literal/原60unit和25contract保持。新增22unit/4contract，源码预期Gateway82/contract29、Application126/Persistence175须自身Windows；83/83静态/完整architecture/Rustfmt/保护资产/171命令/46迁移18PG静态及6破坏探针通过。

新增unit/契约细节、所有24文件、等价/探针/工具证据与报告见 [05记录](modular-rewrite/R09-research-ai-backend/R09-05-formal-response-schema-and-citation.md#行为覆盖与真实结果)。Domain扫描1132、usageDigest7dca548fa0c39319eb8f393a24a1823e8ffa705046dc89d86b1596dc6e4d13a9，365/300/声明摘要保持；生产URL/时间/协议/取消/预算/凭据/模型/迁移/锁/工作流保持。不执行Linux/macOS动态，不将82/29预期或04成功冒充05PASS。精确Windows开始后停止轮询；真实PG/历史四项/账本并发回滚/XLSX/Full/私有固定回归及继承删除风险仍最终新库待验，ignored/公共stub不计PASS。

## R9-05 精确 Windows 收尾（2026-10-11，DONE）

精确提交 `aa70d2b4464e86254eecfcace11621d1b03aad51` / tree `e3581a2ee6264999ab5c3afb37be695e70f6e9bb`，同一分支rewrite/r9-research-ai-backend，[Public Platform CI run `38072987214`](https://github.com/uniquenesssta/123/actions/runs/38072987214) 与 [Windows job `114274071102`](https://github.com/uniquenesssta/123/actions/runs/38072987214/job/114274071102) 均 **SUCCESS**，2026-10-11 02:04:42北京完成。前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace tests、Windows release/MSI/NSIS/客户端启动及运行日志覆盖/错误扫描通过；实际 **Gateway82 / contract29 / Application126 / Persistence175** 均通过。

交付artifact `11677353373` / `windows-automated-delivery-evidence-aa70d2b4464e86254eecfcace11621d1b03aad51`，14,042,360 bytes，SHA-256 `ae05f14fdbdbe18f49bc34de912ab2b0e63ef928f74ff5409db585f6d0cbbb47`。报告 `D:\a\123\123\logs\windows-acceptance-20261010-174601.json`，过程记录同名.txt。原18 broad PG仍ignored；真实PG/历史四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险保持最终封包新库待验，ignored与公开stub不计PASS。

05 **DONE**；用户2026-10-11“收尾05开始06”授权06 **READY**，07～11BLOCKED，R9整体IN_PROGRESS。本次仅既有五文档 `[skip ci]` 收尾，源码/门禁/依赖保持，复用上述同源码Windows成功；此前VERIFYING叙述属于实施历史，不创建R9阶段完成记录。

## R9-06 Source 与 Time Policy（2026-10-11，VERIFYING）

R9-06 Source/Time Policy已实施、`VERIFYING`，基线b6636fe，同一R9分支。URL/domain与禁用字段/内容归source_policy，三可选事实时间归time_policy；Schema保留身份/主体/值形状/覆盖，Citation保留关联，删除旧root validation。4A/11M/1D、16文件；原公开API/DTO/Schema/SourcePolicy配置/协议/依赖/DB/UI/错误/生产策略不变。原82unit/29contract保留，新增8unit/2contract；源码预期Gateway90/contract31、Application126/Persistence175须06自身Windows。83/83静态、完整architecture、Rustfmt、657生产literal/原规则与组合等价、保护资产/171命令/46迁移18PG静态、7破坏探针均通过。

新增8unit/2contract与等价/7探针、16文件、报告/工具/回退详见 [06记录](modular-rewrite/R09-research-ai-backend/R09-06-source-and-time-policy.md#行为覆盖与证据)。复用原Windows Automated，不执行非Windows动态，不将05 SUCCESS或90/31源码预期冒充06通过；真实PG/历史四项/账本并发回滚/XLSX/Full/私有固定回归及继承删除风险保持最终封包新库待验。

## R9-06 Windows Clippy 修复（2026-10-11，VERIFYING）

R9-06 初版f714acb的Windows run38108158888/job114377958564因SourcePolicy字段提示触发Clippy uninlined_format_args失败。现将普通变量格式参数改为内嵌field_key，错误文字/类别/恢复建议/政策保持；原源码卫生门禁新增该回归守卫（旧源码拒绝、修复后通过），Domain使用摘要同步。83/83现有静态、完整architecture、Rustfmt和差异/提示等价通过；Gateway90/contract31仍是待本轮Windows验证的预期，06 VERIFYING、07～11BLOCKED。

初轮前端/contracts/TypeScript/17视口/生产构建通过，Rust Clippy在source_policy.rs:77阻断；工作区测试、release/MSI/NSIS/客户端启动未执行，不计PASS。原90unit/31contract测试源码保持，原同步契约已有home_ODDS完整提示断言；本轮生产只改一个Display格式表达式。当前usageDigest `d52e1a00455c4b29c48382a6c591f0d65aafde4a4da093bb7c6bb769fcaf3cbd`，1134/365/300/声明摘要保持。报告r906f-static-checks.json/r906f-architecture.log/r906f-equivalence.json/r906f-review.json；不执行非Windows动态，最终新库延期边界保持。

## R9-06 Windows 域名测试修复（2026-10-11，VERIFYING）

R9-06 第二轮 `3981b9d` / [Windows run38109741446](https://github.com/uniquenesssta/123/actions/runs/38109741446) / job114382644048 已通过fmt/Clippy，Gateway单测89/90，新增域名测试错误地把大写WWW配置当作去前缀后的apex。已按原“trim→去*.→去小写www.→小写”顺序修正夹具，补大写/混合前缀拒绝与保留前缀的子域成功断言；生产代码保持。原兼容传输门禁锁定该顺序及夹具，三反向探针均拒绝恢复。

第二轮于2026-10-11 12:03:19北京在source_policy.rs:135失败。该轮前端/TypeScript/Vite/17视口及fmt/Clippy -D warnings通过，Application126/126、Persistence175/175单测通过；Gateway89/90未全过，Gateway contract31、后续workspace/release/MSI/NSIS/启动未完成，不能计全链路PASS。ignored数据库测试不计真实数据库通过。

本轮保留90unit/31contract数量与其他Gateway测试字节，usageDigest仅刷新为 `c6b0e2980fbdb43f4f4a7feecb0df38fa8ccf98c503eda723c93f314bc593510`。83/83既有静态、完整architecture、Rustfmt、生产等价及保护资产复核通过，报告r906f2-static-checks.json/r906f2-architecture.log/r906f2-equivalence.json/r906f2-review.json。06自身Windows待验、07～11BLOCKED，真实新库/Full/XLSX/私有固定回归边界保持；不执行非Windows动态。

## R9-06 Windows 来源列表契约修复（2026-10-11，VERIFYING）

R9-06 第三轮43dba5f / [Windows run38135480448](https://github.com/uniquenesssta/123/actions/runs/38135480448) 的Gateway单测90/90通过，contract30/31；新增后台/恢复契约误把完整URL排序后首条引用来源当作搜索来源。已改为比较包含#citation、#search的完整有序列表，保留其他断言与全部生产源码；原兼容门禁补排序/完整列表守卫，四反向探针拒绝恢复。

该轮2026-10-11 19:35:14北京在gateway_contract.rs:1917失败：实际首项#citation，误期望#search。前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings、Gateway90/90及Application126/126、Persistence175/175单测通过；Gateway contract30/31、后续workspace/release/MSI/NSIS/启动未完成，不能计全链路PASS。之前两项Clippy和域名前缀单测修复已通过，ignored数据库测试不计真实数据库通过。

本轮仅替换该契约一处sources断言；两种入口×三种情形、原raw/事实/引用URL、UTC cutoff、错误、缓存用量、提供方请求ID、凭据/POST/GET次数和真实GET URL保持。90unit/31contract数量保持；所有Gateway src字节不变，其他contract字节不变。Domain清单仅usageDigest变为c3aece9b4b2beec64c5434d549d9596532b310b9e88541fda2220ba2c1845bc0。83/83既有JS静态、完整architecture、Rustfmt、等价/四探针与保护复核通过，证据r906f3-static-checks.json/r906f3-architecture.log/r906f3-equivalence.json/r906f3-review.json。06自身Windows待验、07～11BLOCKED，最终新库边界保持；不执行非Windows动态。

## R9-06 Windows 完成与07前置（2026-10-12）

精确修复提交d996a4d4bea34d53486b14a642fa1f655478bde7 / tree2584ce620b3d9f53595f8d4d49da0b3f5ae42df0，[Windows run38145691538](https://github.com/uniquenesssta/123/actions/runs/38145691538) / job114488432087均SUCCESS，2026-10-11 22:30:11北京完成。Gateway90/90、contract31/31、Application126/126、Persistence175/175；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7条记录、3个完成操作）。

证据artifact11705243220，14,039,496字节，SHA-256 4d11fab6a5732281e1cbe5b3112ebdf2314d20f9d2d99d13141cb25e9ca499d8；报告windows-acceptance-20261011-140603.json及同名.txt。三次已知Windows失败均由精确修复自身成功关闭，新增8unit/2contract全部执行通过。

06 DONE；用户2026-10-12“收尾06开始07”授权07 READY，同一rewrite/r9-research-ai-backend分支，08～11 BLOCKED、R9 IN_PROGRESS。本次只同步五既有文档，使用[skip ci]复用上述同源码Windows成功；此前VERIFYING为实施历史，无R9阶段完成记录。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored和公共stub不计PASS。

## R9-07 Plain Chat Responses（2026-10-12，VERIFYING）

R9-07 Plain Chat Responses已实施、`VERIFYING`，基线9ddb27a，同一R9分支。普通Responses最小载荷/原始文本解码归plain_chat/responses/{request,response}.rs，两个mod只登记/显式导出；共用校验/预算/许可/取消/重试/provider/refusal/回退/用量/attempt仍原owner，原Chat分支留08。5A/11M/0D、16文件：新增上述四Rust模块及07节点记录；修改根README、TESTING、R9任务书/索引、Domain清单、lib/client/response、原contract及两个既有兼容门禁，整文件删除/移动无。原API/DTO/Schema/配置/profile/DB/UI/协议/依赖/错误/生产政策保持。原90unit/31contract全文保持，新增8unit/4contract，预期Gateway98/35、Application126/Persistence175待07自身Windows。83/83静态、完整architecture、Rustfmt、完整共用client/response逆迁移等价、字符串多重集/保护/171命令/46迁移18PG静态与7破坏探针通过。报告r907-static-checks.json/r907-architecture.log/r907-equivalence.json/r907-negative-probes.json/r907-review.json；真实文件、23职责与兼容细节见07记录。Context7/官方serde_json1.0.150源码/Mermaid/足球状态同步；Windows精确开始后停止轮询，预计20～30分钟。06 DONE、07 VERIFYING、08～11BLOCKED，R9 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041风险仍最终封包新库待验，不把ignored/公共stub记PASS。

原Public Platform CI的Windows Automated执行frontend/TypeScript/Vite/17视口、fmt/Clippy -D warnings/workspace tests、Windows release/MSI/NSIS/启动/日志。当前实跑仅源码静态与格式，98/35不可记动态PASS；没有执行非WindowsCargo或浏览器运行时。07契约沿用原targets/mocks，最小载荷与完整attempt、7类容错元数据/回退、5类provider/refusal优先、5类入口拒绝共18情形；预算耗尽夹具先满足正数配置，后台夹具先store=true。

## R9-07 完成与08授权（2026-10-12）

精确07提交3dbfde3c7246a08e91ad8bde6dd324a774f5a4a9 / tree38d8ce7f358eb45e8ce73e1414013b46f7462ced，[Windows run38155453893](https://github.com/uniquenesssta/123/actions/runs/38155453893) / job114517387791全SUCCESS，2026-10-12 00:54:23北京完成。Gateway98/98 unit、35/35 contract、Application126/126、Persistence175/175；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过。

证据artifact11709500697，14,040,090字节，SHA-256 10c5237fa3aa7713212cfdcd7647b927bd61d60ed5f82878ce03b190c16f8c01；报告windows-acceptance-20261011-162740.json及同名.txt。07新增8unit/4contract全部执行通过。

07 DONE；用户2026-10-12“收尾07开始08”授权08 READY，同一rewrite/r9-research-ai-backend分支，09～11 BLOCKED、R9 IN_PROGRESS。本次五既有文档[skip ci]收尾，源码树与上述成功一致，先前VERIFYING叙述为实施历史；未创建R9阶段完成记录。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored和公共stub不计PASS。

## R9-08 Chat Completions（2026-10-12，VERIFYING）

R9-08 Plain Chat Chat Completions已实施、`VERIFYING`，同一R9分支，基线d065cff。原model/messages载荷与首choice文本提取迁入plain_chat/chat_completions/{request,response}.rs；mod仅登记/显式导出，旧内联与旧函数移除，共用执行/错误/回退/预算/取消/用量/attempt保持。4A/11M/0D、15文件：新增三Chat模块及08节点，修改client/plain_chat根mod/response、原contract、两个既有兼容门禁、Domain清单和README/TESTING/任务书/索引，删除/移动无。原98unit/35contract保留，新增8unit/4contract；83/83静态、完整architecture、Rustfmt、逆迁移等价与8探针通过，106unit/39contract待08自身Windows。API/DTO/Schema/profile/配置/依赖/DB/UI/模型/生产政策保持；07 DONE、08 VERIFYING、09～11 BLOCKED、R9 IN_PROGRESS。

新增覆盖最小载荷/完整attempt、9类首choice/legacy/message/Responses回退与元数据、5类provider/refusal优先、限流重试/模型fallback/取消及许可释放两分支。报告r908-static-checks.json/r908-architecture.log/r908-equivalence.json/r908-negative-probes.json/r908-review.json；原Windows执行frontend/TypeScript/Vite/17视口、fmt/Clippy/workspace、release/MSI/NSIS/启动/日志，预计20～30分钟，精确开始后停止轮询。未执行非Windows动态，真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041风险仍最终封包新库待验，ignored/公共stub不计PASS。

## R9-08 完成与09授权（2026-10-12）

精确08提交7fd7e1e0eeec5f8b1e3a6a516a32ba21e9f067f9 / treea029512756ee7524367ff1c3daa4ba0ca293688c，[Windows run38191340582](https://github.com/uniquenesssta/123/actions/runs/38191340582) / job114624551223全SUCCESS，2026-10-12 09:45:37北京完成。Gateway106/106 unit、39/39 contract、Application126/126、Persistence175/175；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7记录、3完成操作）。

证据artifact11722500937，14,041,703字节，SHA-256 791a0a3414641bdbc70b524c69be0ef16e4b3e57a93078b2b4809bcd2844ff29；报告windows-acceptance-20261012-011909.json及同名.txt。08新增8unit/4contract全部执行通过。

08 DONE；用户2026-10-12“收尾08开始09”授权09 READY，同一rewrite/r9-research-ai-backend分支，10～11 BLOCKED、R9 IN_PROGRESS。本次五既有文档[skip ci]收尾，源码与上述成功一致，先前VERIFYING为实施历史；未创建R9阶段完成记录。真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored与公共stub不计PASS。

## R9-09 Session Persistence（2026-10-12，VERIFYING）

R9-09 Session Persistence已实施、`VERIFYING`，同一R9分支，基线06ecdd2。8会话用例移动到Application sessions且Service直连出口；9个PG会话方法迁入adapters/ai_workspace/sessions，查询、消息、回答事务、文件、用量、Row映射与载荷校验职责明确，原提案状态流转/校验/映射唯一owner保持。公开接口/DTO/SQL绑定/事务审计顺序/配置/DB迁移/模型/Gateway/UI/依赖不变。新增4Application/4Persistence unit；83/83静态、完整architecture/Rustfmt、14原PG方法字节等价与8探针通过，Application130/Persistence179、Gateway106/39待自身Windows。08 DONE、09 VERIFYING、10～11 BLOCKED，R9 IN_PROGRESS。

原targets新增8unit，覆盖输入先于factory、元数据/完整载荷/Port错误、用量顺序与饱和计数、角色/Unicode字符/文件字节边界；14方法SQL/绑定/审计/事务与7helper正文等价，8反向探针拒绝。原Windows执行frontend/17视口、fmt/Clippy/workspace、release/MSI/NSIS/启动/日志，精确开始后停止查询，预计20～30分钟。未执行非Windows动态，真实PG会话事务/历史四项/账本并发回滚/有效XLSX/Full/私有固定回归及继承model.runs/0041风险仍最终封包新库待验，不把ignored/stub记PASS。报告r909-static-checks.json/r909-architecture.log/r909-equivalence.json/r909-negative-probes.json/r909-review.json。

## R9-09 完成与10授权（2026-10-12）

精确09提交4ed656d51fac604cf74665009cd340f4d7ddbc59 / treee1ea669d51e98fe11797fc90213df5bb3ccbf48b，[Windows run38196834750](https://github.com/uniquenesssta/123/actions/runs/38196834750) / job114641262453全SUCCESS，2026-10-12 11:10:30北京完成。Application130/130、Persistence179/179、Gateway106/106 unit和39/39 contract；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7记录、3完成操作）。

证据artifact11724985136，14,040,086字节，SHA-256 00fdea25a74bfc9fc99bf0031a03161e2f2eb5419f158065c4a5f43c335550b7；报告windows-acceptance-20261012-024340.json及同名.txt。09新增4Application/4Persistence unit均执行通过，未增加/运行真实PG专项。

09 DONE；用户2026-10-12“收尾09开始10”授权10 READY，同一rewrite/r9-research-ai-backend分支，11 BLOCKED、R9 IN_PROGRESS。本次五既有文档[skip ci]收尾，源码与上述成功一致，先前VERIFYING为实施历史；未创建R9阶段完成记录。真实PG会话事务/历史四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored与公共stub不计PASS。


## R9-10 Cancellation Registry（2026-10-12，VERIFYING）

R9-10 Cancellation Registry已实施、`VERIFYING`，基线8d80ee6，同一R9分支。Application唯一Registry及不可克隆Lease管理活动ID/token，Tauri已切换且旧map/锁操作移除；重复ID不覆盖原token并在profile/会话副作用前拒绝，准备失败/网关结束/任务abort/unwind自动清理。原命令/DTO/错误文案/协议/profile/SQL事务/日志载荷/UI/模型/版本/依赖保持；仅新增两个Rust类型导出与明确取消生命周期修正。8新增Application unit待自身Windows，预期138/179与Gateway106/39；83/83静态、完整architecture/Rustfmt、逆切等价和8破坏探针通过。09 DONE、10 VERIFYING、11 BLOCKED、R9 IN_PROGRESS。

新增测试均纳入原Application unit target，未增加runner/workflow/target/数据库专项；未执行非Windows Rust/浏览器动态。自身Windows全SUCCESS前不得DONE，真实PG/历史保护/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored和公开stub不计PASS。完整命令与证据见 [10记录](modular-rewrite/R09-research-ai-backend/R09-10-cancellation-registry.md)。


## R9-10 完成与11授权（2026-10-12）

精确10提交07da8d9a70726fe707fe18fb50e6cbf49d402d49 / tree0ae872b0e41609a989ca4a8fbdc2aaca85cef34e，[Windows run38213432437](https://github.com/uniquenesssta/123/actions/runs/38213432437) / job114692567611整体SUCCESS，2026-10-12 15:05:56北京完成。Application138/138、Persistence179/179、Gateway106/106 unit和39/39 contract；前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动及日志扫描通过（7记录、3完成操作）。10新增8个生命周期unit均实际通过，包括并发同ID、任务abort和panic清理。

证据artifact11731842587，14,039,211字节，SHA-256 b610fb2b906fd93db285eab9c0908dff0cd3e37df73e25f88f16fda9fc798366；报告windows-acceptance-20261012-063850.json及同名.txt。

10 DONE；用户2026-10-12“收尾10开始11”授权11 READY，同一rewrite/r9-research-ai-backend分支、R9 IN_PROGRESS，未进入R10。本次五既有文档[skip ci]收尾，源码同精确成功提交；先前VERIFYING为实施历史。自身11完整Windows未完成前不创建R9阶段完成记录。真实PG/历史保护四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开stub不计PASS。


## R9-11 Runtime Diagnostics（2026-10-12，VERIFYING）

R9-11 Runtime Diagnostics已实施、`VERIFYING`，基线6f471d0，同一R9分支。Gateway diagnostics唯一持有尝试DTO、异步sink/default、JSON指纹/响应ID/毫秒耗时，根出口与实际client已切换，旧owner正文移除；原八条审计→尝试收集→熔断/重试顺序保持。56公开导出名字、DTO/错误/profile/协议/SQL事务/日志/UI/模型/版本/依赖保持，无新共享状态或IO。新增6 Gateway unit/2双协议contract待自身Windows，预期112/41、Application138/Persistence179；83/83静态、完整architecture/Rustfmt、逐字等价与8破坏探针通过。10 DONE、11 VERIFYING、R9 IN_PROGRESS，R10不启动。

新增测试只进入既有Gateway unit/contract targets，未新增runner/workflow/target/数据库专项。未运行非Windows Cargo/浏览器动态，自身Windows全SUCCESS前不得DONE；真实PG/历史保护四项/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开stub不计PASS。完整命令/证据见 [11记录](modular-rewrite/R09-research-ai-backend/R09-11-runtime-diagnostics.md)。


## R9-11 首轮Windows契约失败修复（2026-10-12，VERIFYING）

R9-11 首轮Windows run38218231140 / job114707912576在精确5ff9164失败：Gateway112/112 unit通过，contract39/41，新增plain/structured两测试把协议切为Chat Completions但保留默认Responses显式端点，构造网关即报“兼容 API请求端点与所选协议不一致”。已补两夹具逐协议端点match；原运行诊断门禁锁定两协议覆盖、构造前同步与正确映射，4负向探针均拒绝且恢复。生产源码/公开接口/配置验证/协议/SQL/日志/模型不变，测试数量112/41不变；83/83静态、完整architecture/Rustfmt与夹具逆切等价通过。11仍VERIFYING，R9 IN_PROGRESS，须修复提交自身完整Windows，不启动R10。

首轮Application138/138、Persistence179/179，Gateway112单测含新增6项通过；fmt/Clippy已通过，但workspace在两个contract失败处停止，后续workspace/Windows release/安装包/启动未完成，不能记为整轮SUCCESS。未运行非Windows Rust/浏览器动态，真实PG/历史保护/账本并发回滚/有效XLSX/Windows Full/私有固定回归及继承删除风险继续最终新库待验，ignored/公开stub不计PASS。


## R9-11 精确 Windows 完成与阶段收尾（2026-10-12）

精确修复提交b50d8047a6dbf32775d70704afde61d99b897069 / tree7dd421459bbff3156d4c034fc01e9b74dd2985d5，[Windows run38221009454](https://github.com/uniquenesssta/123/actions/runs/38221009454) / job114716987812整体SUCCESS，2026-10-12 16:29:01北京完成。Gateway112/112 unit、41/41 contract，Application138/138、Persistence179/179；两项plain/structured双协议审计失败契约已实际通过，首轮39/41失败由本修复自身关闭。前端/contracts/TypeScript/Vite/17视口、Rust fmt/Clippy -D warnings/workspace、Windows release/MSI/NSIS/启动与日志扫描通过（7记录、3完成操作）。

证据artifact11735687461，14,038,470字节，SHA-256 3e28db7fddd900f59e4655c058df08e7d69c04feaab92e02e641f99900c05a68；报告windows-acceptance-20261012-080203.json及同名.txt。

用户2026-10-12“收尾11”：11 DONE；R9全部01～11及Windows Automated阶段出口DONE，阶段完成记录与实际总表已建立。仅R10-01 Review Package Workflow READY，02～13 BLOCKED；未开始R10实现、未建立第二分支。本次纯文档[skip ci]，源码、门禁与清单同精确成功提交。真实PG/历史保护四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开stub不计真实PASS；此前VERIFYING/失败叙述保留为实施历史。

阶段出口静态复核：18保护资产/46迁移及18PG静态/171命令/runtime诊断/AI Workspace owner原门禁通过；不重跑同源码Windows，不运行非Windows动态。完整阶段证据与延期见 [R9完成记录](modular-rewrite/R09-research-ai-backend/R09-stage-completion.md)。


## R10-01 Review Package Workflow（2026-10-12，VERIFYING）

R10-01 Review Package Workflow 已实施、`VERIFYING`；唯一阶段分支 `rewrite/r10-review-analytics-release-backend`，从 R9 收尾 `48618fed2d7ff7c2a4db831a037f227dc789f278` 建立。Application workflow 独立持有编排、身份门禁与人工确认元数据，ReviewService 五入口已切换；PG 十个工作流方法迁入对应适配器，SQL、审计及事务正文逐字保持，旧两 owner 删除。原 Domain 八状态/九步界面、171 命令、Ports/DTO/错误/模型保护区保持；不自动接受候选或晋升参数。新增 8 项原 Application unit 待自身 Windows，预期 Application146 / Persistence179 / Gateway112 unit、41 contract。84/84 既有源码门禁、完整架构、Rustfmt、等价复核与 8 破坏探针通过；CI 开始后停止查询。R10 IN_PROGRESS，02～13 BLOCKED，真实 PG/历史四项/账本并发回滚/有效 XLSX/Windows Full/私有 P4/P7 固定回归及继承 model.runs/0041 删除风险仍最终封包新库待验。

现有 83 项源码检查加原 verify-review-service 共 84 项通过；原 verify:architecture、verify_protected_assets、verify_command_contract、verify_database_baseline 通过。独立 Rustfmt1.88 检查 13 个实际变更 Rust 文件；没有运行非 Windows Cargo/check/Clippy/build/test、loopback 或浏览器运行时。8 新增单测、全部工作区和 17 视口/打包/启动由原 Public Platform CI Windows Automated 执行；等待本节点精确提交自身完整 SUCCESS，静态与预期计数不记为动态 PASS。详见 [01记录](modular-rewrite/R10-review-analytics-release-backend/R10-01-review-package-workflow.md)。


## R10-01 精确 Windows 完成与02授权（2026-10-12）

精确提交 `838e39ea35355662c192769a358c56a06131a6ed` / tree `7f42a75620aa8eb44441670a69a379b0c1a7830b`，[Public Platform CI run38229119264](https://github.com/uniquenesssta/123/actions/runs/38229119264) / Windows job114743919870全SUCCESS，run于2026-10-12 17:44:15北京完成。实际Application146/146（新增8 workflow unit均成功）、Persistence179/179、Gateway112/112 unit和41/41 contract；frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/客户端启动及运行日志扫描全部通过。报告windows-acceptance-20261012-092001.json。

artifact11739056741，14,041,808字节，SHA-256 `2831911b9159911fb03ebc9bf894e03f00a4a01c73001b9ffddc8f9a289a1fa7`。01自身完整成功后标记DONE；用户2026-10-12“收尾01开始02”授权，02 Structured Match Facts READY，同一rewrite/r10-review-analytics-release-backend分支，03～13 BLOCKED，R10 IN_PROGRESS。本次仅五既有文档收尾[skip ci]，源码同上述成功提交；后续02必须取得自身完整Windows证据。

真实PG/历史保护四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4/P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验；ignored/公开unavailable stub/静态指纹不计这些真实项目PASS。此前01 VERIFYING及预期计数保留为实施历史，不替代当前DONE状态。


## R10-02 Structured Match Facts（2026-10-12，VERIFYING）

R10-02 Structured Match Facts 已实施 `VERIFYING`，基线ab43aee，继续唯一分支rewrite/r10-review-analytics-release-backend。Application真实事实阶段迁入facts coordinator，ReviewService与组合工作流直接复用；PG facts目录独占校验、事件身份、事务写入、查询/摘要、行映射和事实提交，旧review.rs的14职责及事实写入正文移除。原比赛锁、修订归属、SQL/审计/同事务上下文与十个公开方法签名保持，不改变报告/候选/参数人工门禁。新增8 PG unit待自身Windows，预期Persistence187 / Application146 / Gateway112 unit、41 contract；84/84静态、完整架构、19 Rust文件Rustfmt、等价与8破坏探针通过。01自身Windows已DONE，02 VERIFYING、03～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7回归与继承model.runs/0041删除风险仍最终新库待验。

复用原Public Platform CI Windows Automated与原Rust unit/contract targets；没有新增runner/workflow/target、数据库专项入口或迁移。PG新增8 unit覆盖稳定身份/来源、赛果/覆盖率/重复表现、指标范围/信号、换人/门将关系、事件顺序/核验/修订/比分、加时与取消、摘要；待Windows实际执行，静态或预期187不记动态PASS。19 Rust变更文件只做独立Rustfmt1.88 check，本地未运行Linux/macOS Cargo/check/Clippy/build/test、loopback或浏览器。

PG14原职责和全部剩余函数规范化等价；事实writer/原prepare内联等价，锁/SQL/同事务回读/预测上下文与审计保持。8/8负向源码探针拒绝并恢复。18保护资产、171命令、46迁移及18 PG原静态契约保持；301扫描映射中的新增SubstitutionDraft只来自新typed unit fixture，真实生产映射及Schema保持。详见 [02记录](modular-rewrite/R10-review-analytics-release-backend/R10-02-structured-match-facts.md)。


## R10-02 精确 Windows 完成与03授权（2026-10-12）

精确提交 `1ad6e68042e7749caf6671b2eb4756077dbfa17b` / tree `822588ffe13bd392fbe2851af389eb111b2391ad`，[Public Platform CI run38245932270](https://github.com/uniquenesssta/123/actions/runs/38245932270) / Windows job114799659672全SUCCESS，run于2026-10-12 20:18:27北京完成。实际Application146/146、Persistence187/187（新增8事实unit均成功）、Gateway112/112 unit和41/41 contract；frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/客户端启动及运行日志扫描通过。报告windows-acceptance-20261012-115847.json。

artifact11746489380，14,041,992字节，SHA-256 `378486f022954e5ee7044258e31b09bcd83c8c484c55af6fff11ef71ef1af3d1`。02自身完整成功后DONE；用户2026-10-12“收尾02开始03”授权03 Review Engine Adapter READY，同一rewrite/r10-review-analytics-release-backend，04～13 BLOCKED，R10 IN_PROGRESS。本次只五既有文档收尾[skip ci]，源码同上述已验提交；03必须取得自身精确Windows完整证据。

真实PG/历史保护四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4/P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开stub/静态指纹不计真实PASS。此前02 VERIFYING与预期计数保留为实施历史。


## R10-03 Review Engine Adapter（2026-10-12，VERIFYING）

R10-03 Review Engine Adapter已实施 `VERIFYING`，基线93a008e，继续唯一rewrite/r10-review-analytics-release-backend。Application普通/资料包两条实际生成入口迁入engine_adapter，Service与组合工作流已切换；PG六个引擎职责迁入准备、上下文、原计算事务和报告持久化，事实提交复用同一完整准备。纯review-engine源码/公式、原SQL/错误/JSON载荷、公开签名、事务/审计/提交后回读及pending候选人工门禁保持。原8 Application测试正文保持，其中2生成测试迁入新owner、共用原fixture；新增5入口unit待Windows，预期Application151 / Persistence187 / Gateway112 unit、41 contract。84/84静态、完整架构、18 Rustfmt、等价及8负向探针通过；01～02 DONE，03 VERIFYING，04～13 BLOCKED、R10 IN_PROGRESS。真实PG/历史四项/账本并发回滚/有效XLSX/Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

新增5原Application unit覆盖结果ID不一致不调用Port、空版本转None/空备注保留Some(空字符串)/None保持、Unicode trim且原事实/数据覆盖率/来源不变、完整Port详情透明返回、Port失败无工作流写入；迁移的2资料包生成测试继续覆盖顺序/确认/ID与每步失败停止。所有原测试正文保持，纯计算原tests和PG原integration不变；预期151/187不记动态PASS。

本地仅原84 JS、完整verify:architecture、18实际Rust独立Rustfmt1.88 check、规范化等价/生产SQL错误JSON字面量精确集合/8负向/diff及文档核对；未运行非Windows Cargo/check/Clippy/build/test、loopback或浏览器。原Public Platform CI Windows Automated承担全部动态/17视口/打包/启动；04～13等待03自身完整SUCCESS。最终新库延期沿用02。详见[03记录](modular-rewrite/R10-review-analytics-release-backend/R10-03-review-engine-adapter.md)。


## R10-03 精确 Windows 验收完成（2026-10-12）

精确提交 `99b15ad261a178322231320ad9dc2f8af00ded8e` / tree `a52fd71ad9551a615720c49d9a69d3380d90361f`，[Public Platform CI run38259438025](https://github.com/uniquenesssta/123/actions/runs/38259438025) / Windows job114845878326 全 SUCCESS，run 于 2026-10-12 22:22:12 北京时间完成。实际 Application151/151、Persistence187/187、Gateway112/112 unit 和41/41 contract；新增5个普通入口测试及迁移的2个资料包生成测试均实际通过。前端契约、TypeScript、Vite、17视口、Rust fmt、locked Clippy all-targets -D warnings、完整workspace tests、Windows release/MSI/NSIS、客户端启动和运行日志扫描全部通过（7条记录、3个完成操作）。

证据 artifact11753201967，14,041,892字节，SHA-256 `984de7735649121de192de496d5d92921feb46583e145ad7b8985d37a483ff04`；报告 `windows-acceptance-20261012-135927.json` 及同名.txt。验收对应上述03自身代码，不借用02结果。本次仅五份既有文档补齐验收状态并提交 [skip ci]，生产源码、测试、门禁、清单及依赖均与精确成功提交一致，不重复运行同源码Windows。

03自身完整Windows通过，当前 DONE；01～03 DONE，04～13 BLOCKED，R10 IN_PROGRESS，继续唯一 `rewrite/r10-review-analytics-release-backend` 分支。用户重复“收尾02开始03”的范围已完成；本次没有新增04授权或进入04实施，没有创建R10阶段完成记录。此前03 VERIFYING、预期151和待Windows描述保留为实施历史，以本节和首部当前状态为准。

真实PG、历史保护四项、不可变账本、并发/回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承的model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub、静态指纹不计这些真实验收PASS。本轮未运行非Windows Cargo/check/Clippy/build/test、loopback或浏览器动态。


## R10-04 Ability Candidate Transaction（2026-10-12，VERIFYING）

R10-04 Ability Candidate Transaction已实施 `VERIFYING`，基线aa24f5a，继续唯一rewrite/r10-review-analytics-release-backend。PG候选列表/详情/行映射/人工决策事务迁入ability_candidate_transaction唯一模块；原状态、处理人归一化、0.01过期判断和四状态解码提取并复用，旧root四职责删除。Application/Ports/公开PostgresStore签名、全部SQL/错误/JSON载荷、候选锁与单事务审计/提交后回读保持；拒绝不写能力观察，不自动接受或晋升参数。新增8原PG unit待Windows，预期Persistence195 / Application151 / Gateway112 unit、41 contract；84/84静态、完整架构、8 Rustfmt、13等价核对与8破坏探针通过。01～03自身Windows DONE，04 VERIFYING、05～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

新增8unit仅原PG target，覆盖已处理/未知pending、身份默认与Unicode trim、投影包含边界/缺失/非有限、四状态和未知codec。13等价核对确认SQL/载荷/签名/原门禁逆内联与剩余root不变；8负向均拒绝且文件字节恢复。8实际Rust独立Rustfmt，未运行非Windows Cargo/check/Clippy/build/test、loopback/浏览器；自身Windows全部动态结果待验，原真实PG/历史四项等仍最终新库待验。详见[04记录](modular-rewrite/R10-review-analytics-release-backend/R10-04-ability-candidate-transaction.md)。


## R10-04 精确 Windows 完成与05授权（2026-10-13）

精确提交 `221d21134c46fa45e280230238153694be415aa1` / tree `5f8234f89eb86a6aeb7c19dd8bfc1376385d31b4`，[Public Platform CI run38271387412](https://github.com/uniquenesssta/123/actions/runs/38271387412) / Windows job114886215505 全SUCCESS，run于2026-10-13 00:03:32北京时间完成。实际Application151/151、Persistence195/195（新增8候选unit均成功）、Gateway112/112 unit和41/41 contract；前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、客户端启动及日志扫描通过（7条记录、3个完成操作）。报告windows-acceptance-20261012-153709.json及同名.txt。

证据artifact11758811423，14,041,670字节，SHA-256 `296e86e05c1a649bfab6cda6cb9a938f8b83d75c4f2b19c71d80a3ad404fec59`。04自身完整成功后DONE；用户2026-10-13“收尾04开始05”授权05 Postmatch Settlement READY，继续唯一rewrite/r10-review-analytics-release-backend，06～13 BLOCKED，R10 IN_PROGRESS。本次只五份既有文档收尾[skip ci]，源码同精确成功提交，后续05须自身完整Windows证据。此前04 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。无R10阶段完成记录。


## R10-05 Postmatch Settlement（2026-10-13，VERIFYING）

R10-05 Postmatch Settlement已实施 `VERIFYING`，基线96ef21d，继续唯一rewrite/r10-review-analytics-release-backend。Application结算/readiness/列表迁入settlement唯一模块，三个旧用例删除；PG15原生产职责、3上下文类型及1原时效unit迁入具名结算适配器，证据队列/正式样本继续借用同一原事务。全部SQL/错误/JSON、指纹/幂等键、原readiness、既有返回/插入冲突路径及提交后回读保持，原资料包mark_settled仍独立Port；不自动接受候选或晋升参数。新增8 Application/4 PG unit待Windows，预期Application159 / Persistence199 / Gateway112 unit、41 contract；84/84静态、完整架构、18 Rustfmt、12等价与8负向通过。01～04自身Windows DONE，05 VERIFYING、06～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

新增12只进入原App/PG unit targets；workflow6不合法状态、Settled重试/无包、每Port失败与返回review身份、只读透传，以及PG八门禁/四身份轴/源/时点/已有ID覆盖。原timeliness unit完整迁移且登记一次；原integration不改，真实事务/并发/回滚仍最终新库待验。本地18实际Rust仅独立Rustfmt，不跑非Windows Cargo/运行时；自身05完整Windows前不能DONE。完整证据见[05记录](modular-rewrite/R10-review-analytics-release-backend/R10-05-postmatch-settlement.md)。


## R10-05 精确 Windows 完成与06授权（2026-10-13）

精确提交 `fe3529f93ac1ec6777ccff53da73c0836d310a7f` / tree `950654f5357fb05a162fe4cd1449cb071aa9fd79`，[Public Platform CI run38277864089](https://github.com/uniquenesssta/123/actions/runs/38277864089) / Windows job114907643831 全SUCCESS，run于2026-10-13 00:58:51北京时间完成。实际Application159/159、Persistence199/199、Gateway112/112 unit和41/41 contract；新增8App/4PG unit与迁移的原timeliness unit通过，前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、启动及日志扫描通过（7条记录、3个完成操作）。报告windows-acceptance-20261012-163146.json及同名.txt。

证据artifact11761067078，14,042,191字节，SHA-256 `65186f8312bf643c3b6547cc36cc7a61669e23f735a82be172128584ecb382f2`。05自身完整成功后DONE；用户2026-10-13“收尾05开始06”授权06 Evidence Scoring READY，继续唯一rewrite/r10-review-analytics-release-backend，07～13 BLOCKED，R10 IN_PROGRESS。本次只五份既有文档收尾[skip ci]，源码同精确成功提交，06须自身完整Windows证据。此前05 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计真实PASS。无R10阶段完成记录。


## R10-06 Evidence Scoring（2026-10-13，VERIFYING）

R10-06 Evidence Scoring已实施 `VERIFYING`，基线18f7264，继续唯一rewrite/r10-review-analytics-release-backend。App两证据用例迁入evidence_scoring并切Service/删旧路径；PG查询/映射/人工决定/评分与原队列收拢唯一owner，结算仍借用原tx调用同一队列方法。原SQL/错误/JSON、说明trim后UTF-8字节门槛、评分权重/别名、不可验证NULL、不可变决定/审计/提交后回读保持；不自动判定/晋升参数。新增6App/8PG unit待Windows，预期Application165 / Persistence207 / Gateway112 unit、41 contract；84/84静态、完整架构、16 Rustfmt、16等价及8负向通过。01～05自身Windows DONE，06 VERIFYING、07～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

新增14只原App/PG unit targets，覆盖完整Port字段/可选参数/错误无重试与status/UTF8字节/手工三数值/核验/来源别名/不可验证；原verdict/timeliness各迁移登记一次且正文保持。原PGintegration不改，真正重复/并发/不可变触发器/回滚仍最终新库待验。本地16实际Rust仅独立Rustfmt，不跑非WindowsCargo/运行时；自身06完整Windows前不能DONE。完整记录见[06](modular-rewrite/R10-review-analytics-release-backend/R10-06-evidence-scoring.md)。


## R10-06 精确 Windows 完成与07授权（2026-10-13）

精确提交 `1ed6dbe0a0805a1255917971fc053e509d7e8452` / tree `06e53984ede9ca117973950dabecb4d855ca79d1`，[Public Platform CI run38290328302](https://github.com/uniquenesssta/123/actions/runs/38290328302) / Windows job114948593892 全SUCCESS，run于2026-10-13 02:41:58北京时间完成。实际Application165/165、Persistence207/207、Gateway112/112 unit及41/41 contract；新增6App/8PG unit与原verdict/timeliness unit通过，前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、启动及日志扫描通过（7条记录、3个完成操作）。报告windows-acceptance-20261012-182256.json及同名.txt。

证据artifact11767052093，14,046,257字节，SHA-256 `e80a1c864624bde656ed13c83e379aa2e14946fac57ac4cca5be446f546dcf52`。06自身完整成功后DONE；用户2026-10-13“收尾06开始07”授权07 Provider Score READY，继续唯一rewrite/r10-review-analytics-release-backend，08～13 BLOCKED，R10 IN_PROGRESS。本次只五份既有文档收尾[skip ci]，源码同精确成功提交，07须自身完整Windows证据。此前06 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计真实PASS。无R10阶段完成记录。


## R10-07 供应商评分验证（2026-10-13）

R10-07 Provider Score已实施 `VERIFYING`，基线85ecf15，继续唯一rewrite/r10-review-analytics-release-backend。实际PG供应商聚合、原评分/指纹与快照写入、最新读取和字段映射归provider_score唯一owner；App综合监控/两原Ports不变，不为参考App目录虚构供应商入口。原SQL五分区过滤/BTreeMap与决定顺序、60/15/25原公式、空准确率零分、未先舍入的均值、原指纹/版本/快照键及逐组pool幂等写入保持；监控仍供应商→漂移→总览，没有新事务/审计或自动参数晋升。新增9原PG unit待自身Windows，预期Persistence216 / Application165 / Gateway112 unit、41 contract；84/84静态、完整架构、8 Rustfmt、14等价及10负向通过。01～06自身Windows DONE，07 VERIFYING、08～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

9PG新unit进入原target；84源码/完整architecture/8Rustfmt/14等价/10破坏探针与保护资产/171命令/数据库静态通过，预期216须自身精确Windows；App/原integration18静态契约冻结。见[R10-07记录](modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md)。原Windows job启动后停止查询并提示约20～30分钟；不运行非Windows Cargo/check/build/Clippy/test/浏览器/loopback。真实PG/Full/私有固定回归与继承历史删除风险仍最终新库待验。


## R10-07 首轮 Windows 失败修复（2026-10-13）

首轮精确提交 `688892dd1a67401df028cfdde9cbec08ccedf39b` / tree `aa331e6126bd926b7378ba44ba715d544ac02aff`，[Public Platform CI run38296053769](https://github.com/uniquenesssta/123/actions/runs/38296053769) / job114967284703已完整结束且FAILURE。架构、前端/contracts/TypeScript/Vite与17视口已实际通过，Rust在Clippy阶段因provider_score/aggregation.rs:47的 `uninlined_format_args` 失败；workspace单测和Windows release/MSI/NSIS/启动未执行完成，不记PASS。失败记录windows-acceptance-20261012-191326.txt，artifact11768138324，671字节，sha256 `23bbd18f0a2e47b2791aebbcd96de0138c8015b96ad23c464604b842065d8f8e`。

原因：原key的混合字段表达式partition.partition_key提取为纯函数参数partition_key后，四个format参数均为普通具名标识符，旧位置参数写法触发Rust1.88严格Clippy。现改为 `format!("{provider_id}:{partition_key}:{decision_set_sha256}:{POSTMATCH_MONITORING_VERSION}")`，相同四个Display值/冒号/顺序/版本，哈希和快照键字节语义不变。未改评分、SQL/绑定、幂等、事务、公开API、原9unit或前后调用顺序，未用allow或关闭-D warnings。原Postmatch身份守卫切到实际捕获写法；原Rust源码卫生门禁新增此已确认回归的含空白/换行/可选尾逗号拒绝检查，无新框架。

修复实际通过84/84原JS、完整npm run verify:architecture、1个实际修改Rust文件独立Rustfmt1.88check、单表达式逆替换精确恢复688892d全文、其余链/9unit冻结核对、原07的14项等价（仅先将捕获格式展开为原位置参数再比较）、原10破坏探针与新3格式回归探针，全部字节恢复。三新探针为旧多行、单行、可选尾逗号，分别由既有Rust卫生门禁独立拒绝。原9unit数量/断言保持，预期PG216/App165/Gateway112unit+41contract仍待修复提交自身Windows实跑。

Domain365/129/301映射、scan1217/sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持；使用摘要更新为 `18ce78fc3bd7b90bb6ba841e96ed6bd60a28f3660d9b278370c2ea4461f0891b`。18保护资产、171命令、46迁移与18原PG静态契约/State/Ports保持；旧使用摘要与首次无Windows失败描述保留为实施历史，以本节为准。scratch r1007fix-static-checks.json/r1007fix-architecture.log/r1007fix-equivalence-probes.json/r1007fix-original-equivalence.json均可重建。

按root AGENTS查询Context7：确认rust-toolchain/原workflow固定1.88.0；仓库索引无该精确问法结果，[Clippy官方stable文档](https://rust-lang.github.io/rust-clippy/stable/index.html#uninlined_format_args)给出uninlined_format_args规则及format!("{var}")写法（lint自1.66加入），结合本项目1.88实际诊断采用；没有声称取得1.88专属页面或新增版本/依赖。Create State使用稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce保存本轮精确失败/修复/新CI交接。链路逻辑未变，原Mermaid Chart图继续适用，无新增流程图或子代理。

修复提交仍VERIFYING，01～06DONE、08～13BLOCKED、R10IN_PROGRESS。沿用唯一rewrite/r10-review-analytics-release-backend和原Windows CI；实际Windows job启动后停止查询并提示约20～30分钟，不以首轮前端成功或源码检查替代整轮Windows通过。真实PG/历史四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored/公开stub/static指纹不计真实PASS。


## R10-07 第二轮 Windows 精度夹具修复（2026-10-13）

精确修复提交 `8c9ab5d4fae894ed0735c9adc74432e6b8a984c4` / tree `d31cfe075eaedc76c3f99ba59c251b179a9d4661`，[Windows run38346437982](https://github.com/uniquenesssta/123/actions/runs/38346437982) / job115128855419完整结束且FAILURE。前端/contracts/TypeScript/Vite/17视口与Rust fmt/Clippy all-targets -D warnings已实际通过，首轮uninlined_format_args问题关闭；Application165/165通过，Persistence215通过/1失败。失败唯一为weighted_score_uses_unrounded_means_before_original_six_digit_rounding；其余8个新供应商unit实际通过。PG失败后后续workspace目标、release/MSI/NSIS/启动未完成，不算完整WindowsPASS。失败记录windows-acceptance-20261013-042741.txt；artifact11789116976，669字节，sha256 `154ecca44731c101df3e238a31017de1e73f3f367f373aeb49fb834b12690717`。

实际原因是新增测试夹具/预期的算术错误，不是生产公式偏差：旧输入(.12345649,.87654349,.23456749)按原60/15/25加权的准确十进制值为.2641972900，原最后round6结果.264197，测试误写.264198；且该旧输入提前round均值也得.264197，原本不能区分被测错误。此前记录“该夹具正确.264198/能区分提前舍入”属于错误实施断言，由本节明确订正，不用于当前验收。

现只在该原unit中改为两个已独立核算的区分夹具，unit数量仍9、close容差仍1e-12，额外assert_ne明确拒绝提前舍入结果；没有删测试、调宽容差或改生产评分。两夹具见下表。原Postmatch Service源码门禁读取实际Rust测试数据，用整数BigInt按8位输入/6位输出独立核对60/15/25和正数四舍五入，检查正确分数、提前舍入分数及二者不同；这是静态测试数据检查，不能替代Rust单测实跑。无新框架/target/CI入口。

| 准确率输入 | 时效输入 | 可靠度输入 | 原公式未舍入结果 | 最后舍入正确值 | 提前舍入均值错误值 |
| --- | --- | --- | --- | --- | --- |
| .12345649 | .87654349 | .23456949 | .2641977900 | .264198 | .264197 |
| .12345651 | .87654351 | .23456751 | .2641973100 | .264197 | .264198 |

独立Decimal静态核对同时复查其余数值/样本计数与三个SHA断言：混合样本.5375、缺时效/可靠度.45、不可验证时效均值.9与零weighted、四verdict样本总数17及有序/重复/重排指纹SHA一致。只有一个测试函数变化，其他8个测试/close与全部生产Rust链、公式、快照键、原SQL/绑定/事务/Ports/API/Domain/状态/模型保护保持8c9ab5d逐字不变。

实际84/84原JS、完整npm run verify:architecture、1修改Rust独立Rustfmt1.88check、源夹具Decimal/整数双重核算、生产链冻结/单测试函数变更核对、3新夹具负向（错误预期、恢复旧坏夹具、让正确值与提前舍入值相同）均通过，探针全部文件bytes恢复；未运行非Windows Cargo/check/Clippy/build/test/前端构建/browser/loopback。之前13探针结果和快照格式化守卫保留，原07十四等价对应的生产代码此次完全不变，不重复生产破坏探针。本轮scratch r1007fix2-fixture-audit.json/r1007fix2-static-checks.json/r1007fix2-architecture.log可重建。修复后的原Rust9unit预期仍PG216/App165/Gateway112unit+41contract，必须自身完整Windows确认。

Domain365/129/301映射、scan1217/sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，使用摘要因测试源更新为 `3b104aec0014ca938b9e67dff0d7b210a39dcfe152d21b268d72fecbb4c05a7c`。18保护/171命令/46迁移与18PG静态契约不变；旧使用摘要与前轮“9测试未改”保留为各轮历史，不代表本次精度函数未改。

07仍VERIFYING、01～06DONE、08～13BLOCKED、R10IN_PROGRESS。继续唯一rewrite/r10-review-analytics-release-backend，原Windows job启动后停止查询并提示约20～30分钟；首轮/次轮部分PASS不替代完整Windows。原Mermaid链适用，本次纯测试数据错误无需重查版本文档/Context7；Create State按稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce记录精确失败/修复/新CI，无子代理。真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归与继承model.runs/0041删除风险仍最终新库待验，ignored/公开stub/静态指纹不计真实PASS。


## R10-07 精确 Windows 完成与08授权（2026-10-13）

精确修复提交 `cdd2797dbd1fa3fe01ff6ca2bdcc7f9d718745fc` / tree `bef40499f1e2b332adccd2008405d6889aefe699`，[Public Platform CI run38350692611](https://github.com/uniquenesssta/123/actions/runs/38350692611) / Windows job115142166008全SUCCESS，run于2026-10-13 13:44:22北京时间完成。实际Application165/165、Persistence216/216、Gateway112/112 unit和41/41 contract；9个新增供应商unit含纠正后的双精度夹具全部通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动/日志扫描均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-051954.json及同名.txt。

证据artifact11791662118，14,046,120字节，SHA-256 `e31726acc367194dd9e0f593775bfa45fbac649e71be7a9bffc310e99cd7f166`。首轮688892d的Clippy失败及次轮8c9ab5d的精度夹具失败已由本精确完整成功关闭；失败和纠正过程保留为历史，不以此前部分PASS替代最终证据。07自身完整成功后DONE，用户2026-10-13“收尾07开始08”授权08 Drift Monitoring READY，继续唯一rewrite/r10-review-analytics-release-backend，09～13BLOCKED、R10IN_PROGRESS。本次仅五既有文档收尾[skip ci]，源码同已验cdd2797，08须自身完整Windows成功，无阶段完成记录。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored/公开unavailable stub/static指纹不计真实PASS。


## R10-08 漂移监控验证（2026-10-13）

R10-08 Drift Monitoring已实施 `VERIFYING`，基线33f7188，继续唯一rewrite/r10-review-analytics-release-backend。App原综合监控两入口归monitoring；PG监控前检/活动分区/总览与漂移样本/纯准备/幂等事务/读取映射各有唯一owner。原Monitoring Port与公开签名保持；供应商→漂移→总览顺序、全部SQL/原纯引擎、窗口/状态/指纹、单漂移事务、新run才写finding/审计与冲突不重写保持。新增6App/11PG unit待自身Windows，预期Application171 / Persistence227 / Gateway112unit+41contract；84/84静态、完整架构、22Rustfmt、19等价及10负向通过。01～07自身Windows DONE、08VERIFYING、09～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

6App/11PG进入原unit targets，19等价/10负向/22Rustfmt与84原JS/完整architecture通过；171/227仍须本项Windows实跑，原pure engine/07九unit/18PG静态契约与UI/gateway/依赖保持。完整[08记录](modular-rewrite/R10-review-analytics-release-backend/R10-08-drift-monitoring.md)。原Windowsjob启动后停止查询，约20～30分钟；真实PG/历史保护/Full/XLSX/私有固定回归仍最终新库待验。


## R10-08 精确 Windows 完成与09授权（2026-10-13）

精确提交 `f834e3152e65e791b91897ce4803cedd284f7311` / tree `bc48b5f701f2f2262bd1118390b7691c23ad17a0`，[Public Platform CI run38357781835](https://github.com/uniquenesssta/123/actions/runs/38357781835) / Windows job115164583225全SUCCESS，2026-10-13 15:06:50北京时间完成。实际Application171/171、Persistence227/227、Gateway112/112 unit及41/41 contract；本项新增6App/11PG unit全部通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志扫描均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-063917.json及同名.txt。

证据artifact11794826120，14,048,433字节，SHA-256 `d188c4d0fae8316bde55f14089eac7a7240490a9bf8af36e64df0674db0f4756`。08自身完整SUCCESS后DONE；用户2026-10-13“收尾08开始09”授权09 Analytics Calculation READY，继续唯一rewrite/r10-review-analytics-release-backend，10～13BLOCKED、R10IN_PROGRESS。本次五既有文档收尾[skip ci]，源码同已验f834e31，09须自身完整Windows，无阶段完成记录。此前08 VERIFYING和预期计数保留为历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored/公开unavailable stub/静态指纹不计这些真实PASS。


### R10-09 分析计算（2026-10-13，VERIFYING）

R10-09 Analytics Calculation已实施 `VERIFYING`，基线3fdb443，同一唯一R10分支。App总览及两实际worker刷新归calculation；PG样本/解码/原事务/子指标/持久化与总览投影/读取各有唯一owner，旧root七职责移除，剩余29函数保持。原纯引擎/SQL/绑定/公开接口/错误/窗口/分组覆盖政策、500批次/单事务审计及提交后回读保持；后台进度/取消/全量质量→查询顺序保持，10未提前开始。新增5App/8PG unit待自身Windows，预期176/235、Gateway112unit+41contract；84/84静态、完整架构、18Rustfmt、15等价与11负向通过。01～08DONE、09VERIFYING、10～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

沿用原Windows workflow/targets，job实际启动后停止一切CI查询并提示20～30分钟；未执行非Windows动态验收。15等价含原273字面量/公开签名/29保留函数与13新unit登记，11负向全拒绝恢复；具体命令/静态报告和待验风险见[09记录](modular-rewrite/R10-review-analytics-release-backend/R10-09-analytics-calculation.md)。无阶段完成记录。


## R10-09 Windows 窗口类型修复（2026-10-13）

R10-09首轮精确656a97c / [Windows run38371099454](https://github.com/uniquenesssta/123/actions/runs/38371099454)因窗口返回类型触发Clippy type_complexity失败。现改为具名内部AnalysisWindows，同步唯一事务调用方和8处原窗口断言，原值、窗口政策、SQL/事务/审计及纯引擎保持；原5App/8PG测试数量不变。84/84源码、完整架构、3Rustfmt、22等价及5新负向通过；09仍VERIFYING，须修复提交自身完整Windows，10～13BLOCKED。

首轮前端/contracts/TypeScript/Vite/17视口及Rustfmt实际通过，Clippy阻断后workspace单测、release/MSI/NSIS/启动未完成，不能记完整PASS。artifact11799147265/672bytes，sha256 f4f8cd93df7fe95ab7c79c325591cf394b3d0bdef21381c5e0580247977ce227；报告windows-acceptance-20261013-084347.txt。原窗口边界期望只改具名形状，值不变；预期App176/PG235/Gateway112unit+41contract保持，须自身Windows确认。原Windows job启动后停止查询并提示20～30分钟；无非Windows动态验收，真实PG/历史四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验。详细10M清单、回退及Context7版本核对见[09记录](modular-rewrite/R10-review-analytics-release-backend/R10-09-analytics-calculation.md)。


## R10-09 精确 Windows 完成与10授权（2026-10-13）

精确修复提交 `fbfe83a9e3b979b2c10e154a718c19639900a41d` / tree `91c39ea42d331e1cc294752479f430bf2655c2a9`，[Public Platform CI run38398255939](https://github.com/uniquenesssta/123/actions/runs/38398255939) / Windows job115298907093全SUCCESS，run于2026-10-13 21:02:32北京时间完成。实际Application176/176、Persistence235/235、Gateway112/112 unit及41/41 contract；原5App/8PG新增unit全部通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志扫描均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-123542.json及同名.txt。

证据artifact11815192224，14,048,590字节，SHA-256 `eccc82a62ef2b48036040ad1ddec43f6c4c0001af7a9db0c83526ce5b28d3f3a`。首轮656a97c的Clippy type_complexity失败由本修复自身完整成功关闭；具名窗口与原8期望值实际通过，之前失败/VERIFYING为实施历史。09自身完整SUCCESS后DONE；用户2026-10-13“收尾09开始10”授权10 Background Jobs READY，同一唯一rewrite/r10-review-analytics-release-backend分支，11～13BLOCKED、R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验fbfe83a，10须自身精确Windows，无阶段完成记录。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开unavailable stub/static指纹不计真实PASS。


### R10-10 后台任务（2026-10-13，VERIFYING）

R10-10 Background Jobs已实施 `VERIFYING`，基线4a6fa9b，继续唯一R10分支。App四队列控制入口编组，启动/领取结算/任务执行/请求解码各唯一；PG共享jobs按控制/读取/领取/生命周期/映射/政策重写，旧root及四旧App目录删除。原四类型/六取消checkpoint、状态/错误/重试/到期/优先级/幂等、SKIP LOCKED/状态事件同事务及提交后读解码保持，P4与启动恢复原调用不变。新增13App/6PG unit待自身Windows，预期189/241、Gateway112unit+41contract；84/84源码、完整架构、18Rustfmt、17等价及12负向通过。01～09DONE、10VERIFYING、11～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验。

沿用原Windows workflow/targets，job实际启动后停止查询并提示20～30分钟；无非Windows动态验收。原PG并发领取/重启预算与18静态契约保持ignored/既有入口安排，不计真实PGPASS。17等价含70原生产字面量/11签名与19unit登记，12破坏探针全拒绝恢复；具体命令/静态报告和32路径见[10记录](modular-rewrite/R10-review-analytics-release-backend/R10-10-background-jobs.md)。无阶段完成记录。


## R10-10 精确 Windows 完成与11授权（2026-10-13）

精确提交 `3365b2bb5b6396fe010aee311c819bdd6b1198d8` / tree `1b4c59ba5d786bba1da11c13c2bc5ddd50e069e8`，[Public Platform CI run38411713492](https://github.com/uniquenesssta/123/actions/runs/38411713492) / Windows job115345476640全SUCCESS，run于2026-10-13 22:43:53北京时间完成。实际Application189/189、Persistence241/241、Gateway112/112 unit及41/41 contract；本项新增13App/6PG unit全部通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志扫描均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-141620.json及同名.txt。

证据artifact11824026576，14,051,930字节，SHA-256 `ec237a2d6899402531ad616aab6c7fba11cbb6d1098c3d3e868eafe75e8a030f`。10自身完整SUCCESS后DONE；用户2026-10-13“收尾10开始11”授权11 Analysis Package READY，同一唯一rewrite/r10-review-analytics-release-backend分支，12～13BLOCKED、R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验3365b2b，11须自身精确完整Windows，无阶段完成记录。此前10 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开unavailable stub/static指纹不计真实PASS。


### R10-11 分析资料包（2026-10-13，VERIFYING）

R10-11 Analysis Package已实施 `VERIFYING`，基线5a19c71，继续唯一R10分支。App导出/回包/人工建议六入口按三职责编组；PG资料包数据/元数据/导入/决定/读取/映射/政策各唯一，旧六目录与root15职责移除，剩余14函数保持。原ZIP库/格式/Schema/摘要验证、SQL/错误/身份核验/待审导入/人工候选事务与提交后回读保持，不自动晋升参数。新增11App/8PG unit待自身Windows，预期200/249、Gateway112unit+41contract；84/84源码、完整架构、17Rustfmt、18等价及13负向通过。01～10DONE、11VERIFYING、12～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终新库待验。

新实际文件IO测试只随原Windows unit执行，PG规则测试不算真实事务验收；18等价含原库冻结/240字面量/14保留函数与19unit原target登记，13负向拒绝恢复。Windows job启动即停止CI查询并提示20～30分钟；具体命令/未执行项和31路径见[11记录](modular-rewrite/R10-review-analytics-release-backend/R10-11-analysis-package.md)。无阶段完成记录。


## R10-11 精确 Windows 完成与12授权（2026-10-14）

精确提交 `9d3727c4bdc0ebeb21e83a466c49f97720f1e063` / tree `5606f26bcbddbc74be5ea105fe25faa3cf8c4b6c`，[Public Platform CI run38423742534](https://github.com/uniquenesssta/123/actions/runs/38423742534) / Windows job115387546853完整SUCCESS，2026-10-14 00:03:23北京时间完成。实际Application200/200、Persistence249/249、Gateway112/112 unit及41/41 contract；本项新增11App/8PG unit逐项通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-153953.json及同名.txt。

证据artifact11828902351，14,051,522字节，SHA-256 `bf5b6c6f869263510a06e462570b02fef9265c3a4b6aeb664f1f164ac96d7eea`。11自身完整SUCCESS后DONE；用户2026-10-14“收尾11开始12”授权12 Parameter Lifecycle READY，继续唯一rewrite/r10-review-analytics-release-backend，13BLOCKED、R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验9d3727c；12须自身精确完整Windows，无阶段完成记录。此前11VERIFYING/预期计数为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验；ignored/公开stub/static指纹不计这些真实PASS。

### R10-12 参数生命周期（2026-10-14，VERIFYING）

R10-12 Parameter Lifecycle已实施 `VERIFYING`，基线6da5057，同一R10分支。App九入口按候选/就绪/影子验证/人工绑定四职责编组；PG同名模块升级为职责目录，22原函数迁入，root analytics保留九原函数。指标公式/排序/阈值/哈希、公开提供器拒绝、H精确分区、不可覆盖报告、人工状态门禁/锁/CAS/两绑定事务及提交后回读保持。新增15App/6PG unit待自身Windows，预期215/255、Gateway112unit+41contract；82源码、完整架构、19Rustfmt、19等价及20负向通过。01～11DONE、12VERIFYING、13BLOCKED、R10IN_PROGRESS。真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终新库待验。

21新增测试只随原Windows unit运行，82源码检查不含五浏览器入口及Windows进程测试。具体命令/19等价/20负向/真实待验与39路径见[12记录](modular-rewrite/R10-review-analytics-release-backend/R10-12-parameter-lifecycle.md)。job实际启动即停止CI查询，预计20～30分钟。


## R10-12 精确 Windows 完成与13授权（2026-10-14）

精确提交 `6c4243c1150ba0bbf2c0139429e52c3477df6d8a` / tree `8b3928901a0e495f211eb0b937322eb4b48541d2`，[Public Platform CI run38508581096](https://github.com/uniquenesssta/123/actions/runs/38508581096) / Windows job115671629414完整SUCCESS，2026-10-14 12:18:42北京时间完成。实际Application215/215、Persistence255/255、Gateway112/112 unit及41/41 contract；本项新增15App/6PG unit逐项通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动及日志均通过（7条记录、3个完成操作）。报告windows-acceptance-20261014-035521.json及同名.txt。

证据artifact11869757552，14,050,358字节，SHA-256 `ae7631e1aafcfdfe6e230961c04709d0b8c205dc3e1b28213c7df67c5b4e5c1a`。12自身完整SUCCESS后DONE；用户2026-10-14“收尾12开始13”授权13 Release Acceptance READY，继续唯一rewrite/r10-review-analytics-release-backend，R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验6c4243c；13须自身精确完整Windows，无阶段完成记录。此前12VERIFYING为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验；ignored/公开stub/static指纹不是这些真实PASS。

### R10-13 发布验收（2026-10-14，VERIFYING）

R10-13 Release Acceptance已实施 `VERIFYING`，基线04b2b4c，继续同一R10分支。App编排和检查构造退出mod，两个原报告查询归并；PG同名模块升级为事实/不可变事务/读取映射职责目录。原检查、汇总、哈希、lazy会话、阻断报告写入、SQL/错误及全部保护资产保持。新增15App/4PG unit待自身Windows，预期230/259、Gateway112unit+41contract；82源码、完整架构、14Rustfmt、18等价、22负向通过。01～12DONE、13VERIFYING、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归与继承model.runs/0041删除风险继续最终新库待验。

19新增测试只随原Windows unit执行，82源码不含Windows进程及五浏览器入口。详细命令/等价/负向/未实跑与27路径见[13记录](modular-rewrite/R10-review-analytics-release-backend/R10-13-release-acceptance.md)。Windows job实际启动即停止CI查询，预计20～30分钟；本项自身完成前无阶段完成记录。


## R10-13 精确 Windows 与 R10 阶段完成（2026-10-15）

精确提交 `4ca43ad1c4757ace4c95e126bda34f470d3854ea` / tree `a37d2b14c7748d92600d6d61bc846f87ac057fb9`，[Public Platform CI run38591308033](https://github.com/uniquenesssta/123/actions/runs/38591308033) / Windows job115946614576完整SUCCESS，2026-10-15 00:09:19北京时间完成。实际Application230/230、Persistence259/259、Gateway112/112 unit及41/41 contract；本项新增15App/4PG unit逐项成功。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、客户端启动和运行日志扫描均通过（7条记录、3个完成操作）。

报告 `logs/windows-acceptance-20261014-154132.json` 及同名.txt；未过期artifact11908923021，14,051,269字节，SHA-256 `046186113ac42de27713e3b8633f83253c4844cb2690378da9ab9cd838984afa`。已核对run精确head/分支、全部job步骤、完整日志和artifact；本次纯文档[skip ci]收尾复用该源码树，不重复运行同源码Windows。

R10期初Application138/Persistence179，期末230/259，原unit累计净增92App/80PG；Gateway112unit/41contract保持，最后Windows包含全部已有与新增targets。各节点精确证据、失败修复、完整累计A/M/D、兼容和回退见[R10阶段完成记录](modular-rewrite/R10-review-analytics-release-backend/R10-stage-completion.md)。本次仅2A/6M文档，复核保护/171命令/DB静态、Release owner和运行诊断门禁、清单/链接/diff及源码冻结，不重复全量构建或非Windows动态。R10 DONE；R11仅01 READY、02～10 BLOCKED、未启动。

真实PostgreSQL、历史保护四项、不可变账本/并发回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承的model.runs/0041删除风险继续最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实项目PASS。


## R11-01 平台命令（2026-10-15，VERIFYING）

9条平台命令迁移，原171命令与注册顺序保持，internal定义组15→13和逐命令真实路径同步。新增11个原Tauri unit：5保存配置恢复/回包、4运行日志phase/Unicode/serde边界、2失败任务筛选/attempt去重；2原日志unit迁移正文保持。预期Desktop38=27+11，App230/PG259/Gateway112unit+41contract保持，须本项精确完整Windows确认。

82/82原只读源码、完整architecture、24Rustfmt1.88、33完整等价和16破坏拒绝/bytes恢复通过；保护18/171命令/46迁移18PG静态保持。命令、公开wire DTO、3后续DB命令与冻结源码等价边界、内部清单变化、真实A/M/D与回退见[01记录](modular-rewrite/R11-tauri-commands/R11-01-bootstrap-workspace-and-logs-commands.md)。未执行非Windows Cargo/Clippy/build/test/前端构建/浏览器/loopback；五原浏览器入口和Windows进程入口仍随原Windows全CI执行。job实际启动即停止查询，约20～30分钟；本项VERIFYING、02～10BLOCKED、R11IN_PROGRESS。

真实PostgreSQL、历史保护四项、不可变账本/并发回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。
