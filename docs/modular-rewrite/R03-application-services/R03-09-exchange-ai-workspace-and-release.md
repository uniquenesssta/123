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
- AT1 已正式关闭为 `DONE`；Atomic Task 2 — Spreadsheet Exchange 已开放为 `READY`。R3-09 继续为 `IN_PROGRESS`，R3-10 继续 `BLOCKED`。
