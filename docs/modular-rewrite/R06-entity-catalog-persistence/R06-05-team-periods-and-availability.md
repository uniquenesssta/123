# R06-05 — Team Periods 与 Availability

## 状态

`VERIFYING`

## 进入基线

- R6-04：`DONE`。
- 本阶段继续使用唯一分支：`rewrite/r6-entity-catalog-persistence`。
- 节点起点：`702a84dd8d702d272e0c69ff3399fce20a7e8a4b`。

## 已实施职责边界

- `adapters/catalog/players/team_periods/`：效力期输入策略、typed Row、Domain mapper 与 `add_player_team_period` SQL 唯一 owner。
- `adapters/catalog/availability/`：可用性输入策略、typed Row、Domain mapper 与 `add_player_availability` SQL 唯一 owner。
- `players/detail/team_periods/read.rs` 与 `players/detail/availability/read.rs` 继续只承担 Player Detail 读取 SQL，并复用新的 Row/mapper owner。
- `player_catalog.rs` 已退出 Team Periods / Availability 写职责和对应 Row mapping，不保留转发壳或双 owner。

## 保持不变的契约

- `PlayerCatalogPort::add_player_team_period` 与 `PlayerSignalPort::add_availability` 方法签名及 Application adapter 调用语义不变。
- Domain `PlayerTeamPeriodDraft/Record`、`PlayerAvailabilityDraft/Record` shape 不变。
- 效力期日期与球衣号码校验、registration status trim、Availability confidence/日期校验、reason trim/空值折叠及错误文案保持原语义。
- Schema、0001–0046 migration、配置、日志等级、默认战术角色/位置映射、历史 P4/运行引用和模型保护资产未主动修改。
- 未新增生产依赖。

## 文件变更

```text
M	README.md
M	architecture/domain-type-inventory.json
A	crates/persistence-postgres/src/adapters/catalog/availability/add_player_availability.rs
A	crates/persistence-postgres/src/adapters/catalog/availability/input_policy.rs
R091	crates/persistence-postgres/src/adapters/catalog/players/detail/availability/mapper.rs	crates/persistence-postgres/src/adapters/catalog/availability/mapper.rs
A	crates/persistence-postgres/src/adapters/catalog/availability/mod.rs
R087	crates/persistence-postgres/src/adapters/catalog/players/detail/availability/row.rs	crates/persistence-postgres/src/adapters/catalog/availability/row.rs
M	crates/persistence-postgres/src/adapters/catalog/mod.rs
M	crates/persistence-postgres/src/adapters/catalog/players/detail/availability/mod.rs
M	crates/persistence-postgres/src/adapters/catalog/players/detail/availability/read.rs
M	crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/mod.rs
M	crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/read.rs
M	crates/persistence-postgres/src/adapters/catalog/players/mod.rs
A	crates/persistence-postgres/src/adapters/catalog/players/team_periods/add_player_team_period.rs
A	crates/persistence-postgres/src/adapters/catalog/players/team_periods/input_policy.rs
R078	crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/mapper.rs	crates/persistence-postgres/src/adapters/catalog/players/team_periods/mapper.rs
A	crates/persistence-postgres/src/adapters/catalog/players/team_periods/mod.rs
R084	crates/persistence-postgres/src/adapters/catalog/players/detail/team_periods/row.rs	crates/persistence-postgres/src/adapters/catalog/players/team_periods/row.rs
M	crates/persistence-postgres/src/player_catalog.rs
A	crates/persistence-postgres/tests/player_team_periods_availability_repository_contract.rs
A	docs/modular-rewrite/R06-entity-catalog-persistence/R06-05-team-periods-and-availability.md
M	docs/modular-rewrite/R06-entity-catalog-persistence/README.md
M	package.json
M	scripts/verify-player-directory-detail.mjs
A	scripts/verify-player-team-periods-availability.mjs
```

## 本轮实际验证

临时实施 gate run `32026919056` 只有在以下检查全部成功后才提交生产变更：

- `node scripts/verify-player-team-periods-availability.mjs`。
- `node scripts/verify-player-directory-detail.mjs`。
- `npm run verify:architecture`。
- `node scripts/verify_protected_assets.mjs`。
- `node scripts/verify_database_baseline.mjs`。
- `cargo fmt --all -- --check`。
- `cargo test --locked -p football-persistence-postgres --lib`。
- `cargo test --locked -p football-persistence-postgres --test player_team_periods_availability_repository_contract --no-run`。
- PostgreSQL 16 专用测试库真实执行 `player_team_periods_availability_repository_contract` ignored test。
- `cargo check --locked -p football-application`。

## 阶段 hard gate

- 尚未执行；节点保持 `VERIFYING`。

## 尚未执行

- 用户现有 PostgreSQL 数据真实写入/sample 验收与 Windows Full 人工交互验收；继续保留到最终统一验收，不描述为通过。

## 回退点

- R6-05 节点起点：`702a84dd8d702d272e0c69ff3399fce20a7e8a4b`。
- 通过 Git 提交回退，不复制旧实现，不保留长期双 owner。
