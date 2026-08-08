# R03-05 Lineups Service

## 状态

`VERIFYING`

Lineups Service 源码重写与实施侧 Windows Automated 全链路验证已完成。用户已用 R3-04 基线完成最小本机验证并明确授权进入 R3-05；R3-04 不因此自动改为 `DONE`。R3-05 仍等待用户 Windows 本机最小复核与非破坏性运行时烟测，因此本节点当前保持 `VERIFYING`，R3-06 不提前开放。

## 基线与范围

- R3-05 起点：`906e47c6a782f04159ffac4084dbf117fae67179`。
- 实施保护分支：`rewrite/r3-05-lineups-service`。
- 实施侧 clean 源码验证提交：`7e3fddeafcd32cc45e293fa9a7aeb05c7c66d4ec`。
- 范围：阵型目录与使用分布、比赛创建/删除、球队阵容预设、阵容创建/双方原子提交、阵容列表/详情/历史移除、比赛阵容链、球队比赛阵容历史。
- 共迁移 19 个既有公开 Application 职责：`list_formations`、`save_formation_usage_distribution`、`list_formation_usage_distributions`、`resolve_formation_distribution`、`create_match`、`delete_match`、`save_team_lineup_preset`、`list_team_lineup_presets`、`preview_team_lineup_preset_application`、`duplicate_team_lineup_preset`、`archive_team_lineup_preset`、`delete_team_lineup_preset`、`create_lineup`、`create_lineup_pair`、`list_lineups`、`read_lineup`、`remove_lineup_history`、`read_match_lineup_chain`、`list_team_match_lineups`。
- 排除范围：Prediction、Research、Tauri DTO、前端状态、模型实现、数据库 Schema/迁移与 SQL 行为；不新增生产依赖。

## 实际实施

### 1. Lineup Service / Use Cases

新增：

```text
crates/application/src/services/lineups/mod.rs
crates/application/src/services/lineups/service.rs
crates/application/src/services/lineups/facade.rs
crates/application/src/use_cases/lineups/mod.rs
crates/application/src/use_cases/lineups/<19 个职责>/mod.rs
```

`LineupService` 持有阵型、比赛、阵容、阵容预设的 Application 编排职责；每个公开操作进入对应 Use Case。`ApplicationService` 继续保留既有公开方法名、参数、返回类型和错误传播，facade 只负责取得活动数据库会话并委托 `LineupService`。

旧 `crates/application/src/player_catalog.rs` 已删除。R3-04 的 Teams / Players 职责继续由其现有 Services 持有，R3-05 不保留旧文件、空转发层或重复实现。

### 2. Ports / 具体持久化适配

沿用 R3-01 已冻结的 4 个既有最小 Port，不新增 Repository：

- `FormationPort`
- `MatchCatalogPort`
- `LineupPort`
- `LineupPresetPort`

新增 `crates/application/src/composition/adapters/lineups.rs`，在组合根 `ActiveDatabase` 上实现上述 Ports。Service / Use Case 不直接依赖 PostgreSQL、SQLx、PgPool、PostgresStore、PersistenceStore 或 SQL Row；具体持久化仍由 Application composition 边界适配。

`ApplicationComposition` 与 `ApplicationService` 增加唯一 `LineupService` 所有权；`services/mod.rs`、`use_cases/mod.rs` 与 `composition/adapters/mod.rs` 只登记对应职责模块。

### 3. MatchCatalogPort 读取边界修复

首次完整 Rust 编译真实暴露：`MatchCatalogPort::read_match` 的组合适配需要既有 `PostgresStore::read_match_exchange`，但该方法当时仅为 persistence crate 内 `pub(crate)`，导致 Application adapter 跨 crate 编译错误 `E0624`。

修复只将现有 `read_match_exchange` 的可见性提升为 workspace 可调用的 `pub async fn`；方法体、SQL、参数、返回 `MatchRecord`、错误语义和数据库行为均未改变。没有复制 SQL、修改 Port trait、放宽编译门禁或增加兼容旁路。`verify-lineups-service.mjs` 同步锁定该合法 persistence 边界。

### 4. 既有验证器 owner 路径同步

删除旧 `crates/application/src/player_catalog.rs` 后，完整 frontend 首轮回归依次暴露历史静态验证器仍直接读取旧 Application 文件。已同步迁移以下验证器到当前权威 owner，但未删除或弱化业务断言：

- `verify-team-player-management.mjs`
- `verify-match-lineup-chain.mjs`
- `verify-history-scoreline-ui.mjs`
- `verify-match-workflow-ui.mjs`
- `verify-stage-e2-lineup-presets.mjs`

全仓旧路径扫描确认，其余命中均属于历史契约/文档或 R3-05 对旧 owner 必须不存在的反向断言，未修改。

## 专项门禁

新增 `scripts/verify-lineups-service.mjs` 并接入：

- `npm run verify:lineups-service`
- `npm run verify:architecture`
- `npm run verify:frontend`

专项门禁验证 23 个 Lineups Service / Use Case Rust 文件、19 个公开 Application 职责、4 个既有 Ports、旧 `player_catalog.rs` 退出、Tauri 公共调用链不变、Service / Use Case 无具体持久化泄漏、组合适配器覆盖既有内部读取能力，以及 `read_match_exchange` 的合法 workspace 持久化边界。

Domain 类型清单已按最终源码重新生成；最终清单继续登记 365 个类型、365 个公共兼容类型与 299 个 PostgreSQL 映射类型，架构扫描范围增至 400 个 Rust 文件。

## 实施侧验证

实施过程中所有硬失败均停止后续推进并修复后重跑，没有跳过、弱化或白名单化：

1. 初始架构 run 暴露新增 Rust 文件导致 Domain inventory 漂移；重新生成清单后通过。
2. 完整 frontend 依次暴露历史验证器旧 owner 路径；全仓扫描后将确认受影响的验证器改读当前权威 owner。
3. 完整 Rust 首轮在 `cargo fmt --check` 暴露 3 处格式差异；仅应用 rustfmt 等价格式并重新生成清单。
4. 后续 Rust 编译暴露 `read_match_exchange` crate 可见性错误；按既有 Port 语义提升该 persistence 读取方法可见性并增加专项断言。
5. 专项修复 workflow 已通过 `verify-lineups-service`、`cargo fmt --check` 与 `cargo check --locked -p football-application`；临时生成/扫描/修复 workflow 均已从 clean 源码树删除。

clean 源码提交 `7e3fddeafcd32cc45e293fa9a7aeb05c7c66d4ec` 的 Public Platform CI run `31260698438` / job `93110942400` 已通过：

- `npm run verify:architecture`；
- 完整 `scripts/windows-acceptance.ps1 -Mode Automated`；
- 完整 `npm run verify:frontend`，包含业务契约、17 个截图回归视口、TypeScript 与 Vite production build；
- 完整 `npm run verify:rust`，包含 Cargo.lock、rustfmt、workspace Clippy `-D warnings` 与 workspace tests；
- Tauri Windows release 构建；
- release 客户端启动与运行日志覆盖率/错误扫描。

CI evidence artifact：`9022970030`，大小 `14242839` 字节，SHA-256 `275e17a78db9d5205d49401a1a1d20ed91f08102594d2d04c339051165beb052`。

Automated 模式不配置真实专用 PostgreSQL 测试库，因此需要 `FOOTBALL_TEST_DATABASE_URL` 的 ignored PostgreSQL 集成测试没有被描述为已执行。Vite 继续保留既有大 chunk warning；npm 安装阶段显示既有依赖审计告警，本节点未修改依赖或门禁。

## 尚未完成

仍需用户 Windows 本机作为 R3-05 最终节点验收依据：

```powershell
git status --short
git pull --ff-only origin rewrite/r3-05-lineups-service
cargo fmt --all -- --check
npm run verify:lineups-service
npm run verify:architecture
cargo check --locked -p football-application
cargo test --locked -p football-application
```

建议随后运行完整：

```powershell
npm run verify:frontend
npm run verify:rust
npm run tauri:dev
```

运行时只做非破坏性阵型、比赛、阵容、阵容预设读取/创建链烟测；用户原数据库不得执行删除、reset 或其他破坏性操作。若要运行真实 PostgreSQL ignored 集成测试，只能使用符合项目安全契约的专用测试数据库。

## 回退与下一步

R3-05 可回退到起点 `906e47c6a782f04159ffac4084dbf117fae67179`。不得恢复把 Teams / Players / Lineups 再堆叠回单一 `player_catalog.rs` 的结构，也不得提前迁移 R3-06 Prediction 职责。

状态保持 `VERIFYING`。用户本机复核与非破坏性 runtime 烟测通过并明确关闭本节点后，才将 R3-05 标记为 `DONE` 并开放 R3-06 Prediction Service。
