# R3-09 Exchange / AI Workspace / Release Services

## 状态

`DONE`

## Atomic Task 1 — Match Lineup / AI Match Package Exchange

状态：`DONE`

### 当前实施范围

- 删除旧 `crates/application/src/exchange.rs`，不保留转发壳；8 个既有 ApplicationService/Tauri 入口保持原名、参数和返回 DTO。
- 建立唯一 `ExchangeService`，8 个公共职责分别进入 `use_cases/exchange/<use-case>/` 独立目录；路径/扩展名校验集中于 `file_validation` 共享职责。
- 复用并最小对齐 R3-01 `MatchLineupExchangePort`，具体 PostgreSQL 调用仅由 `composition/adapters/exchange.rs` 适配。
- 文件导出/读取继续先完成原有路径、扩展名、文件存在性和 workbook/package 解析，再惰性获取数据库 session，以保持既有错误优先级。
- `spreadsheet.rs`、`api_workspace.rs`、`release_acceptance.rs` 留给后续 Atomic Tasks，本 AT 不修改。

### 兼容边界

- Tauri 命令/DTO、前端产品源码、PostgreSQL SQL/migration/Schema、Domain 类型与数据格式保持不变。
- 模型保护区、配置、日志等级和生产依赖保持不变。

### 验证状态

- Windows hard gate run `31470329279`：rustfmt、官方 Domain inventory、Exchange 专项、Application Ports、Application Composition、Match Lineup chain、完整 architecture、保护资产、Application check/tests、完整 `verify:frontend` 与完整 `verify:rust` 均 `SUCCESS`。
- 18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试继续按既有安全设计保持 `ignored`，未记为已执行；AT1 未执行破坏性数据库验证。
- clean publication commit `a1bc2cf283a5ffbd81f01950466c5c8c79a9cc1b` 的 PR #17 Public Platform CI run `31471485706` 已整体 `SUCCESS`；canonical Public Platform CI run `31473739191` / Windows Automated job `93722595150` 亦整体 `SUCCESS`，evidence artifact `9095149094` 大小 `14073911` 字节，SHA-256 `12f9e3fcca2a90c510a03295299d5c5cda9ecfa0dce844d63b3f335c04216087`。
- AT1 已正式关闭为 `DONE`；Atomic Task 2 — Spreadsheet Exchange 后续亦已通过 clean Public Platform CI 并关闭为 `DONE`。R3-09 继续为 `IN_PROGRESS`，Atomic Task 3 — AI Workspace 已开放为 `READY`，R3-10 继续 `BLOCKED`。

## Atomic Task 2 — Spreadsheet Exchange

状态：`DONE`

### 当前实施范围

- 删除旧 `crates/application/src/spreadsheet.rs`，不保留转发壳；球队完整资料包、球员工作包、球队月度工作包共 16 个既有 ApplicationService/Tauri 入口保持原名、参数和返回 DTO。
- 16 个 Spreadsheet 公共职责分别迁入 `use_cases/exchange/<use-case>/` 独立目录；球队完整资料包预检进一步将行分类/球队引用解析、覆盖率计算拆为独立职责，提交可写性策略保留原有重试测试。
- `ExchangeService`、ApplicationService facade 与 composition adapters 按 Match Lineup / Spreadsheet 两条职责拆成子模块；AT1 + AT2 共 24 个 Exchange 公共用例继续由唯一 `ExchangeService` 编排。
- 最小补齐 R3-01 `SpreadsheetExchangePort` / `MonthlyWorkbookPort` 的真实 reference/export/preview/read/resolve/commit 能力；PostgreSQL 具体调用仅由 `composition/adapters/exchange/spreadsheet.rs` 的 `ActiveDatabase` 适配。
- Spreadsheet 路径/扩展名/文件存在性校验集中在 `file_validation/spreadsheet.rs`；球队完整资料包、球员工作包和球队月度工作包的原有错误识别顺序与错误文案保持不变。

### 兼容边界

- 公共 ApplicationService/Tauri 方法、参数、返回 DTO、Excel/JSON 格式、导入识别顺序、错误文本与错误优先级保持不变；`export_team_package_preview_json` 继续不要求数据库连接。
- PostgreSQL SQL/migration/Schema、Domain 公共契约、前端产品源码、模型保护区、配置、日志等级与生产依赖保持不变。
- 6 个历史 Spreadsheet 专项验证器只迁移 authoritative owner 路径，原业务断言未删除、跳过或放宽；官方 Domain inventory 按最终源码重新生成。

### 验证状态

- Windows hard gate run `31512091398` / job `93848182746`：最终 rustfmt、官方 Domain inventory、完整 `verify:architecture`、workspace Clippy `-D warnings` 与 workspace tests 均 `SUCCESS`；Exchange 专项确认 AT1 + AT2 共 24 个公共用例由唯一 `ExchangeService` 编排，Spreadsheet/Monthly Ports 与 ActiveDatabase 适配完整，旧 owners 清零且错误优先级保持。
- 同一 Rust 工作区回归中 Application tests 35/35、Domain tests 10/10、Domain Serde 17/17、Tauri tests 27/27、Persistence unit tests 74/74、Spreadsheet IO tests 12/12 均通过，未见测试失败。
- 18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试继续按既有安全设计保持 `ignored`，未记为已执行；未执行破坏性数据库验证。
- 最终 clean Public Platform CI run `31513432237` / Windows Automated job `93852598090` 已在正式只读 workflow 与 clean HEAD `8719008ba63241d313fe02ce4328ee8d0a727e9d` 上整体 `SUCCESS`；architecture、完整 Windows automated acceptance 与 validation evidence upload 均成功。artifact `9110944721` 大小 `14040458` 字节，SHA-256 `f0628be97bbfb13a765da3e4799bd7e8abf549352e391cb79ef1111cb9522a9f`。AT2 正式关闭为 `DONE`；AT3 AI Workspace 后续已完成源码迁移并进入 `VERIFYING`，AT4 Release 保持 `NOT_STARTED`。


## Atomic Task 3 — AI Workspace

状态：`DONE`

### 当前实施范围

- 删除旧 `crates/application/src/api_workspace.rs`，不保留转发壳；12 个既有 ApplicationService AI Workspace 入口保持原名、参数和返回 DTO，并由唯一 `AiWorkspaceService` 编排。
- 12 个公共职责分别进入 `use_cases/ai_workspace/<use-case>/`；Session、Context、Operation、Presets、Attachments 分责，Apply Operation 再拆为 orchestration、dispatch、payload 与 metadata，避免重新形成职责混合大文件。
- 最小补齐 R3-01 `ApiWorkspaceSessionPort` / `ApiWorkspaceOperationPort` 的 usage/session/message/generated-file/operation lifecycle 能力；具体 PostgreSQL 调用仅由 `composition/adapters/ai_workspace.rs` 的 `ActiveDatabase` 适配。
- Ports 禁止裸 JSON 的既有门禁保持不变；Operation 完成结果使用显式 `SerializedApiWorkspaceOperationResult` 穿越 Port，由组合 adapter 唯一反序列化后调用既有 persistence API。
- `ApiWorkspacePresetSpec`、preset 查询和附件读取公共导出改由新 Use Case 模块提供，原公开名称继续由 `lib.rs` re-export。

### 兼容边界

- 12 个 ApplicationService 方法、preset key、错误文本、200 球员通用上下文上限、Operation claim/apply/failed/reject 生命周期、7 种既有数据库提案类型、metadata 注入字段和附件数量/大小/类型/截断/hash 语义保持不变。
- Tauri AI Workspace 产品命令与 UI 未重构；当前“不接受新附件的 AI 问答”行为保持不变，已禁用的 apply/reject Tauri 命令没有重新暴露。
- PostgreSQL SQL/migration/Schema、Domain 公共契约、前端产品源码、模型保护区、Release/AT4、配置、日志等级与生产依赖未改变。
- `verify-api-workspace.mjs` 与 `verify-team-player-management.mjs` 仅迁移到新 authoritative owner，原业务断言未删除或放宽；新增 `verify-ai-workspace-service.mjs` 并接入完整 architecture/frontend 门禁。

### 验证状态

- Windows hard gate run `31553249625` / job `93980269528` 已整体 `SUCCESS`：Rust 1.88.0 rustfmt、官方 Domain inventory、完整 `verify:architecture`、37-Port 契约、新 AI Workspace 专项、`cargo check --locked -p football-application`、Application tests 33/33、`cargo clippy --locked -p football-application --all-targets -- -D warnings` 与 formatter scope 均通过。
- 第一轮门禁发现 `ApiWorkspaceOperationPort` 裸 `serde_json::Value` 违反 R3-01 边界，已改用显式序列化结果类型；第二处阻塞为历史验证器仍读取已删除 owner，已只迁移 authoritative source。两项都未通过放宽、跳过或删除门禁处理。
- 临时 hard-gate workflow 已自删除；最终源码树不保留该诊断入口。
- 18 个需要专用 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试未在本 hard gate 执行，未记为通过；未执行破坏性数据库验证。
- 首次 clean PR CI run `31553692737` 的 architecture 已通过，但 Windows Automated 因历史 v3 artifact 清单仍指向已删除的 `crates/application/src/api_workspace.rs` 而失败；合同文件未修改，仅将验证器映射到新的 authoritative 模块集合。恢复 run `31553867879` / Windows Automated job `93982110497` 已整体 `SUCCESS`，architecture、完整 Windows automated acceptance 与 validation evidence upload 均通过；artifact `9125791161` 大小 `13985122` 字节，SHA-256 `f5662c9e11d1fbd9da12e39efce9b44645144fce33bf6fe674011c2b7c1af0a6`。
- PR #19 已正式合并到阶段分支，merge commit `3d925671ccd424e25965ba9409189f32f920bc4f`。AT3 正式关闭为 `DONE`；Atomic Task 4 — Release 开放为 `READY`，R3-10 继续 `BLOCKED`。


## Atomic Task 4 — Release

状态：`DONE`

### 当前实施范围

- 删除旧 `crates/application/src/release_acceptance.rs`，不保留转发壳；`run_release_acceptance`、`list_release_acceptance_runs`、`read_release_acceptance_run` 3 个既有 ApplicationService/Tauri 入口保持原名、参数和返回 DTO。
- 建立唯一 `ReleaseService`；3 个公共职责分别进入 `use_cases/release/`，运行验收再按 request validation、chain/performance/security/cost/release checks、summary 与 report hash 拆分，避免重新形成职责混合大文件。
- 最小补齐 R3-01 既有 `ReleaseAcceptancePort` 的 runtime-facts / persist / list / read 真实能力；具体 PostgreSQL 调用仅由 `composition/adapters/release.rs` 的 `ActiveDatabase` 适配。
- `run_acceptance` 保持原有“规范请求 → 获取运行时事实 → 生成固定顺序检查 → 汇总 → SHA-256 报告哈希 → 持久化”的编排顺序；Service / Use Case 不直接依赖 PostgreSQL、SQLx 或具体 Store。

### 兼容边界

- performance/cost window 继续 clamp 到 1..=365；非法预算的 Validation 文案保持不变。
- A–I 接入契约、外部模型边界、最低 27 条迁移、真实闭环样本、公开模型/外部 runtime、数据库延迟、模型运行 P95、查询健康、不可变账本触发器、凭据边界、OpenAI 成本预算和 0.23.0 + J 发布契约的状态判定、阈值、remediation 与检查顺序保持不变。
- overall/category/performance/cost 汇总和 report SHA-256 输入字段保持不变；Release Acceptance contract/schema、Persistence SQL/migration、Tauri 命令、前端产品源码、模型保护资产、配置、日志等级和生产依赖均未改变。
- 历史 `verify-release-acceptance.mjs` 与 `verify-public-model-boundary.mjs` 只迁移 authoritative owner 路径；原发布/保护断言未删除、跳过或放宽。

### 验证状态

- 最终严格 Windows hard gate run `31568170298` / job `94024298468` 已整体 `SUCCESS`：Release Service 专项、历史 Release Acceptance 契约、完整 `verify:architecture`、37-Port 契约、官方 Domain inventory、rustfmt、Application check、Application tests 33/33、Application Clippy `-D warnings` 均通过。
- 第一轮门禁实际发现历史公开模型边界验证器仍读取已删除 owner，以及源码迁移导致 Domain inventory 漂移；前者只迁移权威读取位置，后者两次均使用官方 `generate-domain-type-inventory.mjs` 重新生成。严格分步门禁随后发现唯一 rustfmt 差异并按 rustfmt 修正；没有使用 lint 抑制、跳过测试或手工篡改 inventory。
- 临时 hard-gate / inventory-refresh workflows 已清理；最终源码树不保留诊断入口。
- 18 个需要专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试未在本 hard gate 执行，未记为通过；未执行破坏性数据库验证。
- 首轮最终 clean Public Platform CI run `31568684129` / job `94025845936` 中独立 architecture 已通过，Windows Automated 在完整 frontend 的 deterministic protected-assets 门禁停止：AT4 为迁移旧 Release owner 而合法修改了受保护的 `scripts/verify-public-model-boundary.mjs` 权威扫描路径，但 `architecture/protected-assets.json` 尚未同步该验证器的新指纹；这不是 Release 业务、编译或契约失败，且该轮未被记为通过。
- 按仓库既有 `chore(verify): refresh public-boundary fingerprint` 机制，只刷新该受保护验证器的 Git blob / fingerprint 与聚合 SHA；refresh workflow 在提交前实际执行 `verify-protected-assets-deterministic.mjs` 并通过。新 blob 为 `2e5adc250dba986b3b0441c7f26e762f8b9ef6f4`，fingerprint 为 `13f9ea8c98624e156208c836f837875f2a54104be15e6115f5e7f547d4f491e0`，聚合 SHA-256 为 `d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57`；保护文件集合、禁止私有资产规则和验证逻辑未放宽。
- 恢复后的最终 clean Public Platform CI run `31569072962` / Windows Automated job `94027017727` 已在无临时 workflow 的 HEAD `316f7b055817f85e086090bebc3613c577ceae9a` 上整体 `SUCCESS`；architecture、完整 Windows automated acceptance 与 validation evidence upload 均成功。artifact `9131083580` 大小 `13994757` 字节，SHA-256 `03b34d9ba85f0cd82b2a90f6b8cd2a5db3d8a59cb323b0439a7886c2cb6a0f87`。
- PR #20 已正式合并到阶段分支，merge commit `fd1b8eb2b726338ec588df7c1e8cd87ef202ff51`。AT4 正式关闭为 `DONE`；AT1–AT4 全部完成，R3-09 正式关闭为 `DONE`，R3-10 ApplicationService 兼容门面开放为 `READY`。
