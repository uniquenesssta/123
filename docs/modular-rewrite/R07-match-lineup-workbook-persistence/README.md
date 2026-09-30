# R07 Match Lineup Workbook Persistence：执行记录索引

## 阶段状态

`IN_PROGRESS`

R7 重写 Matches、Lineups、Presets 与 Workbook 持久化，重点保证双方阵容原子事务、截止时间、历史链路、批次账本、真实 XLSX 行/子记录 identity 与整批回滚；不实施工作簿解析算法本体或前端导入 UI。

## 前置基线

- R6 已正式 `DONE`；阶段记录：[`../R06-entity-catalog-persistence/R06-stage-completion.md`](../R06-entity-catalog-persistence/R06-stage-completion.md)。
- R7 唯一阶段分支：`rewrite/r7-match-lineup-workbook-persistence`。
- R7 分支精确起点：`a25da2bdd2b93d8146e46af60947a687e998f987`。
- R6 最终出口 run `32276040092`：Windows job `96143592950`、PostgreSQL 16 job `96143592734` 均 `SUCCESS`。
- 公共 Lineup/Match/Workbook Port、Domain DTO、0001–0046 migrations、模型保护资产和 UI 默认保持兼容；本阶段不新增生产依赖。

## 当前源码基线

R7 开始时相关 PostgreSQL 职责仍主要分布于：

- `crates/persistence-postgres/src/player_catalog.rs`：Match Catalog 与 Lineup 写读混合；R7-01 首先迁出 Match Catalog，Lineup 留给后续 R7 节点。
- `crates/persistence-postgres/src/lineup_chain.rs`：阵容时间窗口、validation、chain/history。
- `crates/persistence-postgres/src/team_lineup_presets.rs`：球队阵容预设。
- `crates/persistence-postgres/src/spreadsheet_exchange.rs`：Player/Team spreadsheet preview、batch ledger 与 commit 链路。
- `crates/persistence-postgres/src/monthly_workbooks.rs`：monthly team/player workbook persistence。
- `crates/persistence-postgres/src/match_exchange.rs`：match-lineup workbook/export/import persistence。

目标 owner 统一收敛到：

```text
crates/persistence-postgres/src/adapters/matches/
crates/persistence-postgres/src/adapters/lineups/
crates/persistence-postgres/src/adapters/workbooks/
```

## 任务状态

| 任务 | 范围 | 状态 |
|---|---|---|
| R7-01 | Match Catalog | VERIFYING |
| R7-02 | Lineup Pair Transaction | BLOCKED |
| R7-03 | Lineup Chain / History | BLOCKED |
| R7-04 | Team Lineup Presets | BLOCKED |
| R7-05 | Spreadsheet Batch Ledger | BLOCKED |
| R7-06 | Player Workbook | BLOCKED |
| R7-07 | Team Package | BLOCKED |
| R7-08 | Monthly Workbook | BLOCKED |
| R7-09 | Match Lineup Workbook | BLOCKED |
| R7-10 | Row / Subrecord Identity | BLOCKED |

## R7-01 READY 边界

- 只处理 Match Catalog 的创建、删除、列表、单场读取及其直接 validation/mapping/query owner。
- 目标目录：`crates/persistence-postgres/src/adapters/matches/catalog/`。
- 不提前迁移 `create_lineup`、`create_lineup_pair`、lineup chain/history、preset 或 workbook 事务。
- 新实现通过最小验证后切换唯一入口，并删除被 R7-01 替代的旧 Match Catalog 职责；不得保留转发壳或双实现。
- 必须保持比赛 external key、competition/season/stage/round scope、主客队校验、列表排序/limit、删除保护与审计语义不变。

## 阶段硬约束

- 双方阵容不得逐侧提交；R7-02 必须由单一 pair transaction 拥有双方写入原子性。
- actual 阵容不得进入赛前快照；截止时间与历史时点语义保持。
- Workbook preview 与 commit 分离；冲突未解决禁止提交；整批失败必须回滚。
- 真实 XLSX 与 PostgreSQL 链路必须验证，不得用 mock 替代要求的真实链路。
- 每个 Atomic Task 先通过最小验证，再运行阶段回归；失败即停，不进入下一节点。

## 记录规则

- R7-01 完成时创建 [`R07-01-match-catalog.md`](R07-01-match-catalog.md)，并把 R7-02 切到 `READY`。
- 后续节点按任务书依次创建对应记录；未完成的记录不提前创建为伪完成文件。
- 全部 R7-01～R7-10 完成且最终出口门禁通过后才创建 `R07-stage-completion.md`。

## R7-01 当前验证状态

- Match Catalog owner 已切换到 `adapters/matches/catalog/`；节点保持 `VERIFYING`，等待最小门禁、真实 PostgreSQL contract 与阶段回归。


## 2026-09-30 分支审计与 Windows 验证范围

### 基线、范围与边界

- 审计分支：`rewrite/r7-match-lineup-workbook-persistence`；HEAD：`985f01816060cfd05672bdc03b6771dec7b4e842`。
- R7 起点：`a25da2bdd2b93d8146e46af60947a687e998f987`；累计变更 18 文件，`+1269/-517`。
- 检查覆盖 R7 累计差异、Match Catalog 全部新 owner、Application/Port 调用方、历史验证器、CI、数据库测试入口，以及尚未迁移的 pair transaction、chain/history、presets 和 workbook 的事务/账本/identity 关键链路。这是 R7 范围静态审计，不宣称对整个仓库所有业务完成运行验证。
- 审计 checkout 起始工作区干净。未修改生产源码、测试、脚本、依赖、Schema、迁移或模型资产。
- 用户只要求 Windows 端验证：CI 当前已经使用 `windows-2025`，没有 Linux/macOS job。后续最小门禁、阶段回归、真实 PostgreSQL/XLSX 客户端验证均在 Windows 执行；Linux/macOS 不再作为验收条件。审计环境的静态检查不替代 Windows 验收。

### 已确认问题与归属

| ID / 优先级 | 证据与影响 | 来源 / 处理节点 |
|---|---|---|
| A1 / 高 | `crates/application/src/composition/adapters/lineups.rs:65` 仍调用 `read_match_exchange`；该方法已从 Persistence 删除，`PersistenceStore` 实际是 `PostgresStore` 别名。源码确认 Application 读取链未完成切换，预期阻塞编译；本次未运行 Windows 编译，不伪造编译器日志。 | R7-01 引入；必须在 R7-01 关闭前修复。使用 Persistence 固有公共 `read_match` 边界并验证方法解析，避免 Port 方法自调用递归。 |
| A2 / 高 | `verify-domain-type-inventory.mjs` 实际失败。Domain declaration digest 与 365 个类型保持不变，但 Rust usage digest、扫描文件数（966→975）及 MatchDraft/MatchRecord/MatchStatus/TeamDraft 的调用方清单改变。最新 CI run `32324419622` 于此停止，Windows acceptance 被跳过。 | R7-01 引入的清单同步遗漏；用官方生成器生成、审查声明/调用方差异后提交，禁止跳过漂移门禁。 |
| A3 / 中 | `verify-lineups-service.mjs` 仍要求旧 `match_exchange.rs` 暴露被删除的方法；`verify-match-workflow-ui.mjs` 与 `verify-history-scoreline-ui.mjs` 仍从 `player_catalog.rs` 检查已移走的 scope/delete 内容。这三个检查均实际失败。业务保护代码仍在新 owner，不能将检查失败直接等同于保护逻辑已删除。 | R7-01 引入；迁移权威读取路径并保留原断言，不能删断言或添加双实现掩盖失败。 |
| A4 / 中 | 新 `verify-r7-match-catalog.mjs` 单独通过，但未接入 `verify:architecture`/frontend，且只检查 Persistence 内部调用方，未覆盖 A1 的 Application adapter。 | R7-01；接入门禁并覆盖真实跨 crate 调用链。 |
| A5 / 中 | 新 `match_catalog_repository_contract.rs` 默认 ignored；现有 `run_database_baseline.mjs` 只显式执行 `postgres_integration`，不会运行该新 test binary。现有 CI 没有显式执行该测试。测试缺少比赛层级冲突、列表 limit/排序、受保护删除和审计/引用释放断言。 | R7-01；增加经专用测试库校验的 Windows 执行入口及相关行为断言。未执行不记为通过。 |
| A6 / 中 | 新契约测试只读取连接字符串后立即连接并执行 migrate/写入，未校验数据库名含 test，也未清理所建球队；单独 `cargo test --ignored` 不会经过既有安全 runner。 | R7-01 新增测试缺口；在真实执行前落实专用测试库前检与隔离/清理，避免误连正式库及残留 fixture。 |
| A7 / 中 | `create_match` 在主客队/层级验证前调用 scope resolution；`scope.rs` 自动创建赛季使用 pool 独立提交，因此“指定赛事、缺少赛季且主客队相同”等失败路径可留下赛季。R6 起点已存在同一顺序。 | 继承问题；归 R7-01 Match Catalog，未来业务修复时增加失败无残留的 PostgreSQL 证据；本次不实施。 |
| A8 / 中 | `commit_match_lineup_import` 累加并返回/审计 `ended_previous`，但 `match_exchange.rs:423` 更新批次账本没有持久化 `ended_previous_count`。存在 superseded 旧阵容时响应与账本可能不一致；R6 起点已有该实现。 | 继承问题；归 R7-09 Match Lineup Workbook，并由 R7-05 账本契约覆盖。需真实数据库验证，不能仅靠源码 token。 |

R7-02～R7-10 目标 owner 尚未完成，原模块仍存在是当前阶段计划状态，不据此认定全部都是缺陷。已检查 pair transaction 使用同一事务写入双方，Workbook commit 具有冲突阻断与单批事务，actual 阵容通过 `model_eligible` 和 cutoff 路径隔离，Preset application 为 preview；这些是代码结构证据，不等于并发、回滚或真实 XLSX 验收通过。

### 任务书约束评估（建议，未实施）

结论：核心正确性门禁应保留，模板化结构/流程条款部分过强，且有信息不足的模板字段。本次只实施 Windows 验证范围调整；以下为后续可批准的定向优化，不自动覆盖现行任务书。

| 当前约束 | 判断与调整建议 |
|---|---|
| 总纲 §2.1/§20.2：出现第二职责立即目录化；500 行/24KB 默认硬失败 | 独立状态、依赖或业务职责需要模块边界，但行数应触发审查，不自动决定文件拆分。紧密耦合的事务步骤、私有 mapper/helper 可共置；禁止机械一函数一文件。仅审查当前节点影响范围，未到阶段的旧大文件按原计划迁移。 |
| R7 每节点 §12：所有异步请求都具备 ID、取消/过期丢弃并解除监听器 | 错套 UI 生命周期到 Repository。Repository 应关注事务、超时、锁、错误与取消后的回滚；没有监听器/定时器无需制造对应机制。请求身份和过期 UI 结果由调用方拥有；已提交结果不能因 UI 取消被当作未发生。 |
| 每个原子任务都跑完整 frontend、workspace Clippy/tests | 每个可交付节点仍要有相关最小验证与 Windows 回归；节点内只做相关验证，最终节点和阶段出口保留完整 Windows 回归。影响共享契约或公共入口时提前扩大检查，不反复跑相同成功门禁。 |
| R7 每节点 Input / allowed dependencies / forbidden dependencies 全是“无” | 不是可执行边界，容易误解成禁止 Domain/SQLx 等现有必需依赖。启动节点时应按真实 DTO/Port、SQLx/PostgreSQL 及禁止 UI/工作流所有权填写。 |
| 只允许修改目标模块 | 应允许同根问题所必需的 Application composition adapter、调用方、验证器、inventory 和文档；A1/A3 正是缺少这些衔接。授权不扩展到无关功能或重构。 |
| 最小验证失败立即停止 | 应停止进入下一节点和宣布 DONE，仍可在当前授权范围诊断、修复失败与做独立工作；不能理解为停止所有工作。 |
| 一律按完整固定模板重复记录 | 保留真实变更、兼容、验证、阻塞、回退和权威索引；无变化字段可简短写“无”，文件表与日志可引用唯一记录，避免复制过期事实。 |

必须保留：双方阵容单事务、actual/历史截止时间隔离、冲突未解决不得提交、整批回滚、物理行/子记录 identity、真实 PostgreSQL/XLSX 链路、模型与历史迁移保护、唯一状态 owner、公共兼容面，以及硬性验收失败不得进入下一阶段。

### 本次检查证据与限制

- 使用 `package.json` 实际登记的 32 个 architecture 静态检查逐项执行：30 通过、2 失败（Domain inventory、Lineups Service）。逐项检查用于审计被前置失败遮挡的问题；官方串行门禁仍为失败，未跳过门禁宣称全绿。
- R7 Match Catalog 专项通过；Match Workflow UI 与 History Scoreline UI 静态契约失败。Match Lineup Chain、Player Role Inheritance、Formation Usage、Monthly Workbooks、Import Row Identity、Team Package、Team Package Real Import Recovery、Persistence Adapters 和 171 条命令静态契约通过。
- `verify_database_baseline.mjs`：46 个迁移静态基线通过；`verify_protected_assets.mjs` / deterministic：18 个保护文件指纹通过。相对 R7 起点，模型 API、历史 migrations、Cargo.lock、package-lock.json 未变化。
- 审计调用中曾误用两个不存在的脚本名（`verify-spreadsheet-exchange.mjs`、`verify-database-baseline.mjs`）；另一次读取误用了不存在的 `ports/lineup.rs` 路径。均已按实际清单订正；这些调用错误不归为仓库缺陷，未记为通过。
- 本轮无 Windows 执行环境或专用 PostgreSQL 测试库，未执行 Windows Rust/frontend build、Clippy/tests、真实数据库/XLSX、Tauri 打包和交互验收。Linux/macOS 平台验证未执行且已排除为门禁；跨平台静态材料检查仅作审计证据。
- R7-01 继续 `VERIFYING`；R7-02～R7-10 继续 `BLOCKED`。没有创建伪完成的 `R07-01-match-catalog.md` 或 `R07-stage-completion.md`。

下一步如获修复授权，应先闭合 A1～A7 的 R7-01 链路并在 Windows 完成最小门禁、数据库契约和回归，再更新节点实施记录与状态；A8 留在匹配的后续 R7 节点。该建议不构成开始实施或进入 R8 的授权。
