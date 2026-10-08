# R08 Prediction / P4 Orchestration：执行记录索引

## 当前阶段状态

`IN_PROGRESS`。唯一阶段分支 `rewrite/r8-prediction-p4-orchestration`，起点/回退基线 `90680bf945fbb0d1c191937c2d9e90c2c916fb00`。R8-01～08 已 DONE；08精确修复 `52f23ab` / Windows run `37817443918` 全SUCCESS，首轮Clippy失败已关闭。用户已授权“收尾08开始09”，09已实施、`VERIFYING`，等待自身Windows CI；10～12 `BLOCKED`。精确当前状态只由本索引维护。

## 前置基线与范围

R7 代码/节点和 Windows Automated 已完成，见 [R7 阶段完成记录](../R07-match-lineup-workbook-persistence/R07-stage-completion.md)。最终源码 `a928c8b` 的 run `36871154039` 通过 Persistence 135/Application 55、17 视口、Windows 构建/打包/启动；`90680bf` 仅文档收尾。R7 的真实 PG/XLSX/Windows Full 和模型历史删除风险仍最终封包新库待验，不继承为 PASS。

R8 依据 [任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md) 及总纲订正，沿用现有 Windows 门禁、原单测/contract/PG 和最终新库方式；无新 runner/workflow/数据库或依赖。模型算法/参数/保护资产、公共接口、43 Ports、171 命令、365 Domain/300 映射及 0001～0046 保持。

## 任务状态

| 任务 | 责任 | 状态 |
|---|---|---|
| R8-01 | Match Prediction Input Builder | DONE · [完成记录](R08-01-match-prediction-input-builder.md) |
| R8-02 | Historical Feature Reader | DONE · [完成记录](R08-02-historical-feature-reader.md) |
| R8-03 | Readiness Audit | DONE · [完成记录](R08-03-readiness-audit.md) |
| R8-04 | Deterministic Input Manifest | DONE · [完成记录](R08-04-deterministic-input-manifest.md) |
| R8-05 | Route / Model Request | DONE · [完成记录](R08-05-route-and-model-request.md) |
| R8-06 | Model Execution Adapter | DONE · [完成记录](R08-06-model-execution-adapter.md) |
| R8-07 | Run Persistence | DONE · [完成记录](R08-07-run-persistence.md) |
| R8-08 | P4 Evidence Ledger | DONE · [完成记录](R08-08-p4-evidence-ledger.md) · 精确Windows修复CI全SUCCESS |
| R8-09 | Fact Pipeline | VERIFYING · [实施记录](R08-09-fact-pipeline.md) · 等待自身Windows CI |
| R8-10 | Horizon Orchestration | BLOCKED |
| R8-11 | Workbench Reads | BLOCKED |
| R8-12 | Freeze Transaction | BLOCKED |

## R8-01 实际来源与实施边界

旧 `crates/application/src/prediction.rs` 已在 R3 删除。实际来源是 `use_cases/prediction/execute_prediction_from_match/mod.rs` 中的 readiness→模式许可→Port 受检重建→指纹复核→审计附加→PredictionCommand 组装。该责任收敛到同层 `build_input/`，原执行入口仅编排输入构建及模型执行。

Readiness 评分/检查、manifest 算法、路由与模型执行仍由现有 owner 执行，留给 R8-03～06；PostgreSQL 的原载荷/历史贡献查询和窗口选择通过原 PredictionInputPort 复用，不复制实现或提前改写模型。请求/监听器生命周期仍由原调用方负责，用例不增加无对应需求的共享 State 或 UI 模板。

## 动态待验与阶段出口

R8-01～08 已取得各自精确 Windows CI；09及后续仍须独立取得自身门禁，不继承前项 PASS。数据库相关 contracts、历史四项/账本、模型历史删除风险、有效 XLSX、Windows Full 的真实结果仍最终新库待验，详见 R7 阶段清单。模型保护或真实固定回归不可取得时明确说明，公共 unavailable stub 不冒充私有引擎 Golden Master。后续节点成功后逐项更新；未完成全部节点不创建阶段完成记录。

## R8-01 实施与门禁记录（已通过）

新增 `crates/application/src/use_cases/prediction/build_input/mod.rs` 与本阶段索引；修改原 `execute_prediction_from_match/mod.rs`、模块登记、原 Probe、`verify-prediction-service.mjs`、Domain 使用清单、Application Port 源码计数清单、本任务书、根 README 与 TESTING。没有移动/重命名、删除文件。原输入构建责任整体迁出执行入口，唯一 owner 内依次完成评估、对应模式许可、家族规范化、按 assessed_at 重建、指纹复核、附加审计和原命令组装；原执行入口保留两种 API/模式并调用现有执行器。原 API/DTO/schema、数据格式、配置、错误/日志/UI 和模型保护资产保持；Persistence 输入构建及 cutoff SQL 无改动。

新增 6 项 inline 行为测试，复用原 Probe，覆盖 P4/P7 与正式/影子、权限拒绝前置、stale 输入/质量、仅运行身份变化、Port 失败/无自动重试、缺失指纹/manifest、非法家族及非对象输入。Application 61（原 55+6）、Persistence 135，本项精确 Windows CI 已实际通过。原模型执行器测试继续核对影子不写历史与正式失败不保存；不使用虚构模型概率冒充真实 Golden Master。

沿用现有前端源码清单、架构/保护资产/命令/迁移验证器及 Rustfmt。六项破坏探针：绕过权限、重取时钟、漏指纹、漏审计、丢显式路由、影子改正式，均须被现有增强门禁拒绝后恢复。六项实际均被拒绝并恢复；83 项现有前端源码检查、完整 `npm run verify:architecture`、Rustfmt 和 `git diff --check` 已 PASS，5 项浏览器检查已在 01 精确 Windows CI 通过。`verify_protected_assets.mjs` 的 18 项指纹及私有资产缺席、`verify_command_contract.mjs` 的 171 命令、`verify_database_baseline.mjs` 的 46 连续迁移/18 原 PG 契约静态检查均 PASS；这不等同于真实 PG 执行。去除空白比较确认原权限/输入准备/指纹/审计及命令字段原样迁移。Port 清单只将 Application 文件数 376→377，43 traits 与 Port 源码 SHA 不变；Domain 清单只更新使用方与扫描文件数（1029→1030），365/300 和 sourceDigest 不变。未运行 Linux/macOS Cargo 或客户端验收。本节点 Windows CI 触发后停止轮询，等待用户反馈；现已创建正式完成记录并关闭 01。

回退点为 `90680bf945fbb0d1c191937c2d9e90c2c916fb00`；使用 Git 反向本节点提交回退，不手工恢复旧文件。02 已按用户指令开始，当前独立门禁状态见下方。Mermaid Chart 已按实际输入构建/权限/指纹/正式影子链路更新；结束时 Create State 保存节点与剩余约束。

## R8-01 收尾 / R8-02 开始

2026-10-02 核实 `cba72fd` / run `36881256338` / job `110433291567` 全 SUCCESS，Application 61/Persistence 135、17 视口、Windows release/MSI/NSIS/启动通过。artifact `11173767485`，14,020,690 字节，SHA-256 `33770c2250cc68e434a98b00139c525eabe47d7ed9222fa18f161eeee03f666f`；详见 [01 完成记录](R08-01-match-prediction-input-builder.md)。01 DONE，02 进入实施；02 起点为已验证 `cba72fdfaaf640a17c1e73c326536189dda74d17`，不另建分支。PG、私有固定回归、Full 和继承风险保持待验。

## R8-02 实施与门禁记录（实施时记录；现已通过，见下方收尾）

实际唯一读取入口仍是 `PostgresStore::calculate_team_pre_match_features`，迁入 `adapters/prediction/historical_features/mod.rs`：只读球队历史，空历史直接中性，原范围选择→最多 12 场→相同 cutoff 的进球基准→原纯投影。`read.rs` 将两段原 SQL、绑定、typed rows 和主客场/赛事/赛季/基准映射收敛为单一读取 owner。`team_features.rs` 保留中性载荷、样本类型、原连续曲线、时间衰减、置信度、证据/quality 投影及三项原测试；不含 SQL/store/async，不是空兼容转发。

新文件为 `adapters/prediction/mod.rs`、`historical_features/{mod,read}.rs`；修改原 adapters 登记、team_features、原 postgres_integration、既有 Prediction verifier、Domain/module/PG 清单、README/TESTING/任务书/本索引。没有移动/重命名或删除整个文件；迁出的旧 SQL/范围 owner 已移除。Application、PredictionInputPort、match_prediction、R6 球员贡献 owner 无改动，不创建只有转发的 Application historical wrapper。私有样本结构不进入公共 Domain/DTO，43 Ports/171 命令/365 Domain/300 映射/依赖/模型/迁移保持。

六项 inline 新测试覆盖四场范围阈值、同赛季优先/同赛事/跨赛事、过滤后 12 上限与顺序、typed 主客场与空 scope 映射、基准 null/非有限/零策略、原平台固定评分/证据和中性载荷；原三项曲线测试保留。Persistence 源码预期 141（135+6），Application 61 保持，须 Windows CI 实跑确认。原 PG cutoff 用例扩 finalized/created 相等、后一微秒各自拒绝、有效样本存在时 baseline 两个未来守卫、前一微秒纳入；18 个已有 broad targets 不变，真实结果最终新库待验。

已实际 PASS：83 项现有前端源码检查、完整 `npm run verify:architecture`、Rustfmt、`git diff --check`、18 保护指纹/私有资产缺席、171 命令、46 连续迁移/18 原 PG 契约静态基线。六项探针（入库 cutoff、基准相等、同时间排序、样本上限、置信度系数、投影 I/O）均拒绝并恢复。逐段比较确认原 SQL literal、scope/neutral/曲线/有限值 helper、原三测试及加权投影不变；只从 Result 包装解耦纯投影。5 项浏览器检查、Rust 编译/Clippy/测试/Windows 交付待自身 CI。没有 Linux/macOS 动态验证或新基础设施。

清单仅登记 prediction direct module，PostgreSQL 直接模块计数 35→36；Domain 使用扫描 1030→1033，声明摘要/365/300 保持，usageDigest `ff639cbfc14fb7f113de1a7caa47b85a488e107246e683ff47a5eb02ba50096f`；PG runtime_sources 仅刷新原测试 blob，46 迁移摘要不变。原两次只读 SQL 没有跨查询事务快照保证，本节点保持已有行为，不承诺新增隔离或提供器实跑。原 P4/P7 time-window/contract/fact/orchestration/workbench/PG 使用现有入口，私有 Golden Master 仍不分发。

02 回退点为 `cba72fdfaaf640a17c1e73c326536189dda74d17`，受控 revert 后同步唯一 owner、清单/门禁；不恢复双实现，不改历史数据。CI 启动后停止轮询，等待结果；通过才创建 02 完成记录并开始 03。本轮 Mermaid Chart 已更新真实历史读取/范围/基准/投影链路，结束时 Create State 保存继续位置。


## R8-02 收尾 / R8-03 开始

2026-10-02 核实 `cc2b0fe` / run `36891488571` / job `110467936302` 全 SUCCESS，Application 61/Persistence 141、六项新历史特征测试、17 视口、Rust/前端构建和 Windows release/MSI/NSIS/启动通过。artifact `11178738200`，14,023,805 字节，SHA-256 `60662e5193de93cbef0da47ea1e5299dfab8954379de5d7d4a50d3c31ad33ed6`。详见 [02 完成记录](R08-02-historical-feature-reader.md)。02 DONE；03 起点/回退基线为该精确提交，沿用同一分支。

## R8-03 实施与门禁记录（实施时记录；已通过，见下方收尾）

实际来源为 `inspect_match_prediction_readiness/mod.rs` 和 `shared/readiness_checks.rs`，两处完整旧文件已删除。新增 `readiness/{mod.rs,workflow.rs,lineups.rs,input_quality.rs,report.rs,tests.rs}`：出口只导出；workflow 唯一持有一次 assessed_at 和读取/路由/准备顺序；lineups 与 input_quality 分别持有原纯检查；report 持有检查载荷和原分级/评分/原因归类；tests 在原 Application 单测 target 内复用现有 Probe。manifest/hash 继续使用 shared/audit（留 04），路由规范化/验证继续使用 shared/routing（留 05）；不改算法或 provider 策略。

修改 Service、build_input 和模块登记，直接使用同一 readiness owner；原 Probe 增加所选比赛/链读取、请求记录、scope 与错误种类注入；原 Prediction verifier 扩 owner/时钟/顺序/异常/纯检查/评分门槛，原 player-role verifier 只更新真实检查 owner 路径。Domain 和 Port 清单只更新使用方/扫描数（1033→1037、377→381），365 类型/300 映射、声明摘要、43 traits 与 Port SHA 不变；根 README、TESTING、任务书、本索引同步记录。无完整文件移动/重命名，按职责提取而非同名路径迁移；两旧文件删除，无兼容空壳。本轮另外新增 02 完成记录。

新增 8 项行为测试：同一 assessed_at、顺序/路由/manifest、原 5 场和 65%/40% 阈值、评分上限和许可、精确 selected lineup/门将/首发身份与阻断优先、InvalidState/NotFound 转报告、其他 Port 错误原样停止且不重试、非 ready 不准备、路由 snapshot/scope/supports、非法家族/未注册模型读前拒绝、原因顺序与去重。预期 Application 69（61+8）、Persistence 141 保持，须 03 精确 Windows CI 实跑；测试假模型只验证编排，不等同提供器结果。

原八项 helper、完整 async workflow（除汇总提取）、原报告汇总去除空白/格式逗号逐段比较等价；没有改变错误/字符串、权重、check/manifest 字段、身份来源、读取条件或错误优先级。六项探针（时钟重取、selected fallback、历史阈值、影子质量阈值、评分上限、吞 Port 错误）均被原增强门禁拒绝并恢复。83 项现有源码检查、完整 `npm run verify:architecture`、18 保护资产/171 命令/46 迁移及 18 PG 静态验证、Rustfmt 和 `git diff --check` 均已 PASS；准确结果见 TESTING。本项不新增 runner/workflow/target/数据库、依赖、迁移或公开类型，没有 Linux/macOS 动态验收。

不新增后台任务、缓存、监听器、State 或重试；局部审计状态随单次 async 请求结束释放，取消由原调用方管理。权限仍由原等级方法推导，P4.4/公开 unavailable stub 和 P7 保护边界不变。PG/历史四项/账本/XLSX/Full/私有 Golden Master 和继承删除 trigger 风险保持最终新库待验。回退用受控 revert 恢复已验证 `cc2b0fe`，同步唯一入口/检查/门禁/清单；不复制重复实现或修改历史数据。Mermaid Chart 已更新实际链路，结束时 Create State 保存状态；03 CI 启动后停止轮询，成功才创建正式完成记录。


## R8-03 收尾 / R8-04 开始

2026-10-02 核实 `47ba3de` / Windows run `36902069546` / job `110503393919` 全 SUCCESS；Application 69/Persistence 141、八项新审计测试、17 视口、Rust/前端、Windows release/MSI/NSIS/启动通过。artifact `11184750052`，14,023,532 字节，SHA-256 `1fe676d3e63a36507a0914e3f2339fc12f1aa25d68311dece17b1b0fc88a00f2`。详见 [03 完成记录](R08-03-readiness-audit.md)。03 DONE，04 起点/受控回退基线为精确 `47ba3dea6ca873248728b9846421c009d00f51bc`，沿用同一 R8 分支。


## R8-04 实施与门禁记录（实施时记录；现已通过，见下方收尾）

原 shared/audit.rs 六项函数迁入 `input_manifest/`：mod 仅登记/导出；canonical 唯一持有清单副本、原五运行字段排除与 JSON 字节/SHA256；audit 持有原受检重建、附加载荷和摘要复核。旧完整 audit 文件及登记删除。build_input、readiness/workflow、execute_prediction 及其原测试直接使用新 owner；P4 snapshot_projection 的输出矩阵哈希保持独立，routing 留 05。六项原函数及两项原测试去空白/格式逗号比较等价，无字段、序列化或错误语义变化。

新增文件为 `input_manifest/{mod,canonical,audit,tests}.rs` 和本轮 03 完成记录。修改原 `build_input/mod.rs`、`execute_prediction/mod.rs`、prediction/mod.rs、`readiness/{workflow,tests}.rs`、shared/mod.rs、prediction/tests.rs（迁出两原测试，原 command_fixture 仅扩大测试内可见性供复用）；原 Prediction verifier、Domain/Port 清单、README/TESTING/任务书/本索引同步。删除 `shared/audit.rs`，无完整文件移动/重命名、重复 owner 或空壳。Application 扫描 381→384、Domain 扫描 1037→1040；43 traits/Port SHA、365 类型/300 映射及 Domain 声明摘要保持。公共 API/DTO/schema、171 命令、数据/配置、错误/日志/UI、依赖/锁文件、模型算法/参数/资产及 0001～0046 均不变。

两项原测试保留且迁移；八项新测试进入同一 Application target，覆盖固定公开平台清单/指纹、精确运行身份排除、事实敏感和数组/null 语义、受检重建与审计载荷错误顺序、摘要元数据和篡改先于模型执行/历史写入。预期 Application 77（69+8）、Persistence 141，须 04 精确 CI 实跑。固定清单指纹 `178afe68af4d0cb8ba9341a7f5f47ec3b89c4c2b9ceaafd0e6615db3a9a0fb87`，不冒充私有 P4/P7 Golden Master。manifest 在副本上仅去掉原根/snapshot/sources 运行字段；原 request.input 的完整指纹仍包含运行身份和审计，无放宽 cutoff/事实/路由、不修剪身份、不排序数组。

六项破坏探针（排除层级、输入副本、JSON 字节、stale 哈希、摘要复核、执行审计前置）均拒绝并恢复。83 项既有源码检查、完整 `npm run verify:architecture`、Rustfmt、18 保护指纹/私有缺席、171 命令、46 连续迁移/18 原 PG 静态基线及 `git diff --check` 均已 PASS；准确结果见 TESTING。Rustfmt 的旧本地运行库截断已从现有同版本归档恢复，格式化与 --check 已通过，不改变项目依赖。5 项浏览器与 Rust/Windows 动态验收留自身 CI；无 Linux/macOS 动态验证或新基础设施。

纯函数无 I/O、随机/时钟/共享状态；附加函数在原错误检查成功后仅替换 input_audit，取消/异步生命周期由原调用方管理。原 provider/P4.4 策略保持。PG/历史四项/账本/XLSX/Full/私有固定回归和继承 model.runs/0041 删除 trigger 风险仍最终新库待验。回退用受控 revert 恢复已验证 `47ba3de`，同步 owner/调用/测试/门禁/清单，不复制旧实现或改历史数据。Mermaid Chart 已更新实际清单/复核/执行链路，结束时 Create State 保存状态；04 CI 启动即停止轮询，成功才创建正式完成记录。


## R8-04 收尾 / R8-05 开始

2026-10-02 核实 `59c5679` / Windows run `36966815323` / job `110712250872` 全 SUCCESS，Application 77/Persistence 141、两项保留及八项新清单测试、17 视口、Rust/前端、release/MSI/NSIS/启动 7 条/3 操作通过。artifact `11210976146`，14,025,634 字节，SHA-256 `dc0c8ec9ea9d5c9b5c9e9d62b0cdb27e5a9420747de5b81f839e68b0b7625677`。详见 [04 完成记录](R08-04-deterministic-input-manifest.md)。05 起点/回退基线为精确 `59c567912194e232b29f36ecf94c3a26c969ed06`，沿用唯一 R8 分支。

## R8-05 实施与门禁记录（实施时记录；已通过，见下方收尾）

实际来源为 shared/routing.rs 的十二项 helper、execute_prediction 的显式上下文覆盖/ModelRequest 组装以及 dry_run_default_fixture 的默认请求组装。新增唯一 `route_model_request/{mod,selection,route,request,tests}.rs`：mod 仅导出，selection 规范化和精确注册验证，route 快照类型与受审计身份，request 输入/上下文/实际和默认请求组装；全部纯计算。旧 routing 完整删除，shared 保留真实 p4_planning；没有兼容转发或第二 owner。预览、输入构建、审计、执行器、默认 dry run 和 lib 原测试接入唯一 owner。原 Port 路由 I/O、supports/predict、计时和正式/影子保存仍在原用例，留 06/07。

新增文件为上述五项 Rust 文件和本轮 04 完成记录；修改 lib.rs、build_input/mod.rs、dry_run_default_fixture/mod.rs、execute_prediction/mod.rs、prediction/mod.rs、preview_route/mod.rs、readiness/workflow.rs、shared/mod.rs、prediction/tests.rs（两原测试迁出，原 Probe 只扩大测试内 route_fixture/route_requests 可见性）、现有 Prediction verifier、Domain/Port 清单、README/TESTING/任务书/本索引。删除 shared/routing.rs；无完整文件移动/重命名。

十二项原 helper、原显式覆盖、两个组装块去空白/格式逗号比较等价；将提取调用重新内联后，原执行器和 dry run 完整函数等价；另三个调用方除 import 外等价。家族空白默认 P4、精确注册、RFC3339→UTC、手工 match key、原字符串保留、快照精确成员和 route identity 全十一字段保持。已有 match_id 在上下文 trim、输入载荷不 trim 的原行为明确保留。显式规则包覆盖执行上下文；readiness 原 catalog context 不改。默认请求仍 Custom/Null/无规则包身份和原公开外壳载荷，不回退提供器或私有算法。

原两项模型选择测试迁移且断言不变，新增八项 inline 行为测试复用原 Probe，覆盖选择/错误、UTC/身份/名称/解析优先级、显式覆盖/参数/identity、快照和全身份变化、默认载荷、预览只读/Port 错误优先级、自动/显式类型及先于后续校验。预期 Application 85（77+8）、Persistence 141 须 05 精确 Windows CI 实跑；不新增 runner/workflow/target/DB。

公共 API/DTO/schema、171 命令、43 Ports、365 Domain/300 映射、依赖/锁文件、模型算法/参数/18 资产、数据/配置/错误/日志/UI 及 0001～0046 保持。局部状态随原单次请求释放，没有新增 State、缓存、监听器、后台任务、重试或时钟。原取消由调用方管理。PG/历史四项/账本/XLSX/Full/私有固定回归与继承删除 trigger 风险保持最终新库待验。受控 revert 到 `59c5679` 并同步入口/清单/门禁，不复制重复实现。精确 CI 触发后停止轮询，05 成功后才创建正式完成记录及开放 06。


05 实际本地门禁：83/83 原源码检查 PASS（88 清单中的五浏览器项留 Windows）；完整 `npm run verify:architecture`、Rustfmt --check、`git diff --check`、18 保护资产、171 命令、46 迁移/18 PG 静态基线全部 PASS。六破坏探针拒绝并恢复。报告 `r805-source-checks.json` / `r805-architecture.log` 保存在本轮工作目录（可由上述原命令重现）；未执行本地 Cargo 或客户端动态验收。Application 扫描 384→388，Domain 1040→1044；43 traits/Port SHA、365 类型/300 映射及声明摘要保持。Mermaid Chart 已更新实际职责链路，结束时 Create State 保存进度；Windows 动态结果仍待本提交 CI。


## R8-05 首轮 Windows CI 修正（2026-10-02；实施时记录，修复已通过）

精确 `1f1613d4578b8b8f23f2a458cedc213d072e65b9` / [run 36970160631](https://github.com/uniquenesssta/123/actions/runs/36970160631) / job `110722266615` 失败：Windows Clippy 普通 lib 编译报告 route_model_request/mod.rs 的 `ensure_match_input_id` re-export 未使用，`-D warnings` 将其提升为错误。源码检查和 Windows 前端契约/类型/截图/生产构建已通过；Rust tests、release/安装包/启动尚未到达，不能计通过。

根因是提取后实际请求组装已在 request.rs 内部调用该 helper，目录级导出只有 lib/tests 与模块单测消费。修复将该单一 re-export 限定 `#[cfg(test)]`，生产 helper 及请求组装保持；其余生产导出均有真实非测试调用方。既有 Prediction verifier 增加 cfg 及生产出口限制，去掉 cfg 的破坏探针须被拒绝。无 `allow(unused_imports)`、公共 API/行为/测试数量、模型/资产/依赖/迁移/数据库变化。修复后沿用原静态门禁及 Windows CI，05 仍 VERIFYING；不得关闭或启动 06。Context7 按 Rust 1.88.0 对照 cfg(test)/use 语义，实际动态结果以新精确 SHA 为准。

本轮修复实际 PASS：83/83 原源码检查、完整 `npm run verify:architecture`、Rustfmt --check、18 保护资产、171 命令、46 迁移/18 PG 静态基线、`git diff --check`；移除 cfg 的单项破坏探针已拒绝并恢复，十项生产导出逐项确认非测试消费。报告 `r805-fix-source-checks.json` / `r805-fix-architecture.log` 位于本轮工作目录；未执行本地 Cargo 或客户端动态验收，Windows 结果待新精确 CI。此修复全部六个文件仅修改，无新增/移动/删除；Domain 使用清单的扫描指纹随出口源码刷新，365/300、声明摘要与其余冻结内容保持，生产函数和测试未变。

首次本地检查提示 Domain 使用清单扫描指纹过期，已运行原 `generate-domain-type-inventory.mjs` 并核对只有 usage scan 指纹变化，随后重跑原检查；无类型/接口变更。


## R8-05 收尾（2026-10-08）

核实精确修复 `2aa99a76cb97bb289fd486bb8e3ed5059620fb88` / Windows run `36971419476` / job `110726001870` 全 SUCCESS；CI 完成时间 2026-10-02 14:22:38（北京时间）。日志确认 Application 85/Persistence 141、两项保留及八项新路由请求测试、17 视口、Rust/前端、release/MSI/NSIS/启动 7 条/3 操作通过。artifact `11212471101`，14,023,932 字节，SHA-256 `1cb00b0a91d1bbf25270dc8fb535b0405e0280b60595a313f0bc74f4503a3538`。详见 [05 完成记录](R08-05-route-and-model-request.md)，含完整累计 23 文件、首轮失败/修复、兼容和延期风险。

本轮只新增完成记录并同步根 README、TESTING、任务书、本索引，源码/测试/清单不变，复用已验证 `2aa99a7` 源码。05 DONE；06 前置通过、READY 尚未实施；07～12 BLOCKED。PG/历史四项/账本/XLSX/Full/私有 Golden Master 和继承删除 trigger 风险继续最终封包新库待验。纯文档提交不重复全量验证，不将自动触发的尚未结束 CI 计为通过。


## R8-05 收尾确认 / R8-06 实施（2026-10-08；实施时记录，已通过见下方收尾）

05 代码验收 run `36971419476` 与收尾文档 run `37737677787` 均已 SUCCESS。06 从当前 `4420fd48ab5e88d55ce15e5b0cdc4b6b05f1c43d` 开始；该提交仅补充 UI 设计接入文档，模型代码仍为已验证 `2aa99a7`，沿用唯一 R8 分支。

06 将原执行器的注册查找/实际 context 支持检查/模型调用/错误转换/毫秒计时迁入唯一 `model_registry/prediction_model_adapter.rs`；默认 dry run 复用注册查找及不计时调用，按原政策不新增 supports/validate。查找返回同一 Arc provider，请求借用和输出原样传递，模型错误完整保留；错误提示继续使用原 scope 类型。路由/组装/输入审计及正式保存/影子 nil 仍由原 owner 负责，保存拆分留 07，不创建空 execute-model 转发目录。readiness 的只读 supports 和 R10 的 Analytics 调用不提前迁移。

原 Application target 新增 7 项模型边界测试；预期 Application 92（85+7）、Persistence 141，须本次精确 Windows CI 实跑。实际本地 PASS：83/83 原源码检查、完整 verify:architecture、Rustfmt 1.88.0 目标检查、18 保护资产、171 命令、46 迁移/18 PG 契约静态基线、git diff --check。六探针（跳过支持、提示类型漂移、丢错误详情、耗时溢出、额外 validate、绕过 adapter）均拒绝并恢复；原执行器重新内联后去空白比较完全一致。原组合根精确文件集合同步登记 adapter，未放宽公共注册/状态所有者断言。清单仅刷新使用方/源码扫描：Application 388→390，Domain 1044→1046；43/365/300 与声明摘要不变。

详细文件和验证见 [06 实施记录](R08-06-model-execution-adapter.md)。无 Linux/macOS Cargo/客户端动态验收、无新 runner/workflow/target/DB；本次 Windows CI 开始后停止轮询。06 VERIFYING，07～12 BLOCKED；CI 成功才收尾06并开放07。PG/历史四项/账本/XLSX/Full/私有 Golden Master 与继承删除 trigger 风险仍最终新库待验。Mermaid Chart 已更新真实链路，结束时 Create State 保存继续位置。

## R8-06 收尾（2026-10-08）

用户反馈“已通过”后核实精确实施 `09693318f8a7c8a557a15b89ef3a81e7ea391b11` / [Windows run `37752995641`](https://github.com/uniquenesssta/123/actions/runs/37752995641) / job `113230527909` 全 SUCCESS；完成于 2026-10-08 17:12:32（北京时间）。日志确认 Application **92** / Persistence **141**、七项新增 adapter 测试、17 视口、Rust/前端、Windows release/MSI/NSIS 及启动 **7 条记录 / 3 个完成操作**通过。artifact `11539138019`，14,023,860 字节，SHA-256 `c0e37e6bb7c706aa292215859844a93491266978dc88f4819abd93b3fb568dd3`；报告 `logs/windows-acceptance-20261008-085549.json`。详见 [06 完成记录](R08-06-model-execution-adapter.md)。

本轮仅同步五份现有文档，源码/测试/清单保持已验证状态，检查链接、任务状态和差异；纯文档提交使用 `[skip ci]`，不重复构建。06 DONE；07 前置通过、READY 尚未实施，等待用户启动指令；08～12 BLOCKED。真实 PG/历史四项/账本/XLSX/Windows Full/私有 Golden Master 及继承历史删除风险继续最终封包新库待验，ignored 不计 PASS。架构无变化，沿用实施时链路图；Create State 保存验收与继续位置。

## R8-07 实施与门禁记录（2026-10-08，VERIFYING）

用户“开始07”后从 `eadb49d51722f50349509a0402c5a925402a53dc` 开始，沿用唯一 R8 分支。旧 Postgres model_runs 完整删除，保存、纯输入前检、借用事务的明细、只读历史与隐藏事务迁入 `adapters/prediction/runs/` 的五个职责文件，mod 只登记/导出。原 Application 正式保存/影子 nil 和结果组装仍同一用例；Port/composition/共享 identity reader 不复制。原九项生产函数、SQL literals、四签名和原一测试比较保持。详见 [07 实施记录](R08-07-run-persistence.md) 的 7 新增/15 修改/1 删除清单，无整文件移动/重命名。

原单测 target 新增 Persistence 七项边界测试，原 Application 保存失败扩为六类 Port 错误；预期 **148/92**，需自身 Windows CI。原 PG identity target 增共同回滚、snapshot 复用、原载荷/明细/审计读回、历史隐藏/首次时间/原记录可读及不可变 trigger，仍 ignored。没有新 test target/runner/workflow/数据库设施，真实 PG 结果仍最终新库待验。

实际 PASS：83/83 现有源码检查、完整 architecture、目标 Rustfmt、18 保护资产、171 命令、46 迁移/18 PG 静态基线、git diff --check；六破坏探针均拒绝并恢复。Postgres 根模块 36→35，Domain 扫描 1046→1051、usageDigest 刷新；Application 390/43 Ports、365/300 与声明摘要不变。5 浏览器项和 Rust/Windows 编译/测试/交付留本项 CI，不继承06。无 Linux/macOS 动态验收。

回退用受控 revert 恢复本项已验源码基线 `eadb49d`，同步唯一 owner、出口、测试/门禁/清单和记录，不手工复制旧文件或改历史数据。Mermaid Chart 已更新数据库比赛与手工推演、正式事务/影子、历史读取与隐藏链路；结束时 Create State 保存状态。Windows CI 启动后停止轮询，07 保持 VERIFYING、08～12 BLOCKED；成功才收尾07。真实 PG/历史四项/账本/XLSX/Full/私有固定回归及继承删除 trigger 风险保持最终新库待验。


## R8-07 精确 Windows 验收与收尾（2026-10-08）

精确实施 `58b390a6400646ac6131dfef0503a49ce1128eca` / [Windows run `37796909083`](https://github.com/uniquenesssta/123/actions/runs/37796909083) / job `113378629485` 全 SUCCESS，完成于 2026-10-08 23:27:02（北京时间）。完整日志确认 Application **92** / Persistence **148**、七项新增边界及原审计测试、前端契约/类型/生产构建、**17** 视口、fmt/Clippy/workspace tests、Windows release/MSI/NSIS 和启动 **7 条记录 / 3 个完成操作**通过。报告 `logs/windows-acceptance-20261008-145959.json`；artifact `11559974847`，14,025,581 字节，SHA-256 `b31ac49010ca525cd6c836fa9746be451ed5dc72bc8e918522e6ecad7e22f2ac`。

已核对精确 SHA、全部 job/steps、日志与 artifact；实施时预期/待验文字保留为历史记录。07 正式 **DONE**，08 前置通过；用户已授权“收尾07开始08”，沿用唯一 R8 分支。收尾仅同步五份文档，源码/测试/清单保持上述已验树，链接/状态/diff 核对，不重复全量构建，文档提交使用 `[skip ci]`。真实 PG/历史四项/账本/有效 XLSX/Windows Full/私有 Golden Master 与继承 model.runs/0041 删除风险继续最终封包新库待验，ignored 不计实跑 PASS。


## R8-08 实施与门禁记录（2026-10-08，VERIFYING）

07 通过并文档收尾为 `1db0e195cdbc26514d7ad015b4110a53a446ba26`，08从该基线开始，沿用唯一R8分支。证据追加/冲突事务、纯输入前检、引用校验/Row投影从p4_records迁入evidence_ledger；原共用lock/key/fingerprint判定迁入p4/idempotency，旧非账本owner直接复用。原六种证据状态投影由ledger/row唯一持有且snapshot共享。没有新增Application空转发层；Artifact/ResearchRun/Snapshot/Fact/Workbench和冻结留对应后续owner。

10原生产函数与原一测试迁移等价，冲突纯前检重新内联后原事务函数等价；2公开签名、31SQL及旧剩余34函数保持。文件清单10A11M，无整文件移动/删除，详见 [08实施记录](R08-08-p4-evidence-ledger.md)。原Persistence target新增8测试，预期156/Application92，需自身精确Windows实跑；原PG Stage C扩读回/引用失败/并发唯一/审计和不可变，18 broad仍ignored，真实结果最终新库待验。

实际PASS：83/83原源码检查、完整architecture、12文件Rustfmt、18保护资产、171命令、46连续迁移/18PG静态基线和git diff --check；六探针（漏锁/漏metadata指纹/放宽字节键/漏实体校验/跳过Schema版本/提前commit）均拒绝并恢复。初次architecture发现新增p4直接module登记后计数未更新，已精确35→36并复跑通过，不放宽owner集合。Domain扫描1051→1060、365/300/sourceDigest保持，usageDigest `307b38c0ae795e65133ae11522339b21a47bd8ab5cc1129f02f679ca0ba97608`；Application390/43Ports保持，PG清单仅刷新原test blob。

08须本项Windows Automated，CI启动后停止轮询；未做Linux/macOS动态验收。Mermaid Chart更新实际声明/冲突事务链，结束时Create State保存。08VERIFYING，09～12BLOCKED；真实PG/历史四项/账本/XLSX/Full/私有固定回归及继承model.runs/0041删除风险保持最终新库待验。回退受控revert恢复已验源码文档基线1db0e19，同步唯一owner/出口/测试/清单，不手工恢复双实现或修改历史库。


## R8-09 实施与门禁记录（2026-10-08，VERIFYING）

08精确修复 `52f23ab` / Windows run `37817443918` 全SUCCESS，收尾文档已推送 `6718c996613edc6e4770457ed02fff16f8a95e26`。用户授权“收尾08开始09”，09从该基线开始，沿用唯一R8分支。

Application原Fact Pipeline已有纯职责复用；mod中的公共命令、编排和事实准备提取到command/process/prepare，artifact注册归source_policy，mod只显式出口，所有原通配导入改为具体依赖。原Postgres混合fact_pipeline_records完整删除，8公开方法/9内部helper按来源策略、上下文、候选、实体/时间/冲突/路由及指纹迁入既有P4 adapter。原ResearchService/Ports/上游调用路径保持，不新建空模板State或转发。完整13新增/22修改/1删除、无整文件移动，见 [09实施记录](R08-09-fact-pipeline.md)。

原Application/Persistence target各新增7边界测试，预期99/163，须09自身Windows实跑。原PG Stage C/E补实际context/candidates、各记录首次值/指纹/排序去重/重试/不可变及来源策略元数据/审计/前检零残留；18 broad仍ignored，真实结果最终新库待验。保留来源缺失发生在实体/时间写入后的原部分副作用，不虚构pipeline整体事务或自动重试。

实际本地PASS：83/83现有源码检查、完整architecture、25目标Rustfmt、18保护资产、171命令、46迁移/18PG静态基线与diff。17旧Postgres生产函数/8签名、18原SQL raw literals及208字符串、32原Application helper、注册/序列化命令和重新内联编排比较一致。六探针（目录实现/无序分组/cutoff身份/主客队过滤/路由ID去重/指纹前缀）均拒绝并恢复。清单精确刷新：Application390→393/43 Ports保持，Postgres根36→35，Domain扫描1060→1071/365/300/声明摘要保持；PG仅刷新原test blob。

09保持VERIFYING、10～12BLOCKED。Windows Automated启动后停止轮询，无Linux/macOS Cargo/客户端动态验收；真实PG/历史四项/账本/XLSX/Full/私有Golden Master及继承历史删除风险继续最终新库待验。Mermaid Chart已更新真实事实流水线，结束时Create State保存继续位置。回退受控revert至 `6718c99`，同步owner/出口/测试/清单/文档，不恢复双实现或修改历史库。
