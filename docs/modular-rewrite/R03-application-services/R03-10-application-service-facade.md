# R3-10 ApplicationService 兼容门面

## 状态

`VERIFYING`

## 目标与边界

R3-10 以 R3-09 已关闭的 Application Services 累计代码树为基线，将 `ApplicationService` 收敛为稳定兼容门面。公共 ApplicationService 方法名、参数、返回 DTO、Tauri 调用面、错误语义、数据库行为与模型调用契约保持不变；本节点不修改 PostgreSQL SQL/migration/Schema、Tauri 产品命令、前端产品源码、模型实现/私有资产、配置或生产依赖。

阶段工作分支 `agent/r3-10-application-service-facade` 从阶段分支提交 `961c7bbc58a7f29edb06c8b30c565b5eb91540d7` 建立；该提交的源码树与 R3-09 收口 canonical CI 已验证树保持一致。

## Atomic Task 1 — P4 Orchestration Owner Extraction

状态：`DONE`

- 删除旧根级 `crates/application/src/p4_orchestration.rs`（136 行），不保留转发壳。
- 建立 `services/p4_orchestration/{service,worker,facade,tests}.rs` 与 `use_cases/p4_orchestration/{process_next,failure}.rs`，P4 job claim/dispatch/complete/fail、最大尝试次数终态迁移和 worker loop 分责。
- 新增最小 `P4OrchestrationQueuePort` 与显式 `SerializedP4OrchestrationResult`，具体 PostgreSQL job 调用仍只由 `composition/adapters/prediction.rs` 的 `ActiveDatabase` 适配；Application Ports 从 37 增至 38 个最小 trait，职责域仍为 15 个。
- P4 worker `AtomicBool` 状态从 `ApplicationService` 移入 `P4OrchestrationService.running`，`architecture/state-ownership.json` 同步为唯一 owner/writer；根 `ApplicationService` 不再持有该运行状态。
- 删除已经失去调用方的 Application/Database `active_store`、私有 Prediction freeze helper、私有 Research worker helper 和 `PersistenceStore` 内部 re-export；没有增加 dead-code/lint 抑制。
- 历史 P4 freeze、Prediction、Research、Application Composition 与 state ownership 验证器只迁移 authoritative owner；原契约断言未删除、跳过或放宽。

### AT1 验证

最终严格 hard gate run `31591061641` / job `94095867258` 为 `SUCCESS`：rustfmt、官方 Domain inventory、P4 freeze/Prediction/Research 专项、38-Port、Application Composition、完整 `verify:architecture`、`cargo check --locked -p football-application`、Application tests 33/33、`cargo clippy --locked -p football-application --all-targets -- -D warnings` 全部通过。最终 AT1 提交：`2bc98df768ed36bfe9e9712b4eae9cc54f83b1c6`。所有 AT1 临时 workflows 已在同一成功提交删除。

## Atomic Task 2 — ApplicationService Facade Convergence

状态：`DONE`

- 新建 `use_cases/application_facade/bootstrap.rs` 与 `database_lifecycle/{connect,initialize,reset}.rs`；Database facade 的 connect/reset 与 bootstrap facade 只保留公共兼容签名和单一 Use Case 委托。
- 保持连接生命周期既有顺序：prepare → Rules/Research built-in initialization → health → activate → Analytics worker → P4 worker；初始化失败关闭 prepared connection。reset 失败且连接丢失时继续保留 best-effort reconnect，成功 reset 后重新 connect。
- Research facade 不再直接构造 `P4ManualConflictAccess` 或重复传递多个 `&session`；多 Port access 组装下沉到 `ResearchService` 的单-session compatibility 方法。
- Prediction `list_recent_runs` facade 不再持有 iterator/DTO mapping；`ModelRunListItem` 的既有 compatibility 身份保持，由 `services/prediction/compatibility.rs` 完成转换。
- 新增 `scripts/verify-application-service-facade.mjs` 并接入 `verify:architecture`。门禁扫描当前 22 个 `impl ApplicationService` 文件，拒绝根对象业务异步方法/运行状态、具体 PostgreSQL/SQLx/HTTP 实现、Database 生命周期编排、Research 多 Port 组装、Prediction DTO mapping 与 P4 orchestration 回流。
- R3-02/R3-03/R3-06/R3-07 历史验证器仅将 bootstrap/lifecycle/Research/Prediction authoritative owner 指向当前模块；没有弱化原初始化、Competition/Rules、Prediction 或 Research 断言。

### AT2 验证

最终严格 hard gate run `31592634518` / job `94100834403` 为 `SUCCESS`：新的 ApplicationService facade 专项、Database、Competition/Rules、Research、Prediction、Application Composition、完整 `verify:architecture`、Application check/tests 与 Clippy `-D warnings` 全部通过。最终 AT2 提交：`dd0aa8e32c8459acbbb68805b6fa099d92d17f91`。所有 AT2 临时 workflows 已在同一成功提交删除。

实施过程中若 recovery run 在源码变换、历史 owner 契约、rustfmt/Clippy 等硬门禁失败，提交步骤均被跳过；失败树未被描述或提交为通过结果。

文档后的完整 `verify:frontend` 首轮发现历史 `verify-database-reset.mjs` 仍从 Database facade 读取自动重连与 P4 worker 入口；验证器现分别改读 `database_lifecycle/reset.rs` 与 `services/p4_orchestration/facade.rs`，原强确认、自动重连和 worker 幂等恢复断言保持。恢复 run 已越过该契约并通过 17 个截图回归，但临时 runner 未执行仓库 Node setup，随后因无法解析 `typescript` 停止；该环境失败未记为通过，后续先执行仓库既有 `npm run setup` 再重跑完整 frontend。

## 模块规模与职责收敛

相对 R3-10 基线，当前变更为 42 个文件、`+740/-389`。旧 136 行 `p4_orchestration.rs` 被删除；P4 orchestration 分布到 8 个小型职责文件。Database bootstrap 从本节点 diff 的 `+2/-69`、Database facade `+5/-65`、Prediction facade `+3/-18`、Research facade `+4/-31`；新增 application lifecycle/bootstrap use case 文件合计 155 行，Prediction compatibility 18 行，最终 facade verifier 78 行。没有把迁出的流程重新堆入单一新文件。

## 兼容性与未执行验证

- 公共 ApplicationService 方法、参数、返回类型和 Tauri 调用保持不变。
- 数据格式、持久化结构、SQL/migration、配置、日志等级、安全/权限、模型保护边界和生产依赖未改变。
- 18 个需要专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试未在 AT1/AT2 hard gate 执行，未记为通过；未执行破坏性数据库验证。
- PR #21 当前保持 Draft / Open / 未合并。R3-10 技术实现和专项硬门禁已完成，但 clean Public Platform CI 与正式合并/收口前状态保持 `VERIFYING`。
- `R03-stage-completion.md` 仅在 R3-10 正式关闭后创建；本节点当前不提前关闭 R3 阶段。
