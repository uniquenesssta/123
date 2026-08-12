# R3-09 Exchange / AI Workspace / Release Services

## 状态

`IN_PROGRESS`

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
- 最终 clean Public Platform CI run `31513432237` / Windows Automated job `93852598090` 已在正式只读 workflow 与 clean HEAD `8719008ba63241d313fe02ce4328ee8d0a727e9d` 上整体 `SUCCESS`；architecture、完整 Windows automated acceptance 与 validation evidence upload 均成功。artifact `9110944721` 大小 `14040458` 字节，SHA-256 `f0628be97bbfb13a765da3e4799bd7e8abf549352e391cb79ef1111cb9522a9f`。AT2 正式关闭为 `DONE`；AT3 AI Workspace 开放为 `READY`，AT4 Release 保持 `NOT_STARTED`。
