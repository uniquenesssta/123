# R06-04 — Player Names 与 Positions

## 状态

`VERIFYING`

## 进入基线

- R6-03：`DONE`。
- R6-04 实施分支：`agent/r6-04-player-names-positions`。
- 节点起点：`90f1bb08bf5a7a1580448f791dd241118ea32284`。

## 已实施职责边界

- `adapters/catalog/players/names/`：Player Name 输入策略、typed Row、Domain mapper 与 `add_player_name` SQL/事务唯一 owner。
- `adapters/catalog/players/positions/`：Player Position 输入策略、typed Row、Domain mapper 与 `assign_player_position` SQL/事务唯一 owner。
- `detail/names/read.rs` 与 `detail/positions/read.rs` 继续只承担 Player Detail 读取 SQL，并复用上层 Names/Positions Row 与 mapper；原 Detail 重复 mapper/row 已删除。
- `player_catalog.rs` 已退出 Player Names/Positions 写职责与对应 Row mapping，不保留转发壳或双 owner。

## 保持不变的契约

- `PlayerCatalogPort::add_player_name` / `assign_player_position` 方法签名与 Application adapter 调用语义不变。
- Domain `PlayerNameDraft/Record`、`PlayerPositionDraft/Record` shape 不变。
- 名称 trim/normalize、primary name 同事务同步 `football.players`、primary position 切换、日期/熟练度/默认角色长度错误语义不变。
- Schema、migration、配置、日志等级、默认战术角色/位置映射、历史 P4/运行引用和模型保护资产未主动修改。
- 未新增生产依赖。

## 本轮实际验证

临时实施 gate run `31969378628` 已通过：

- `node scripts/verify-player-names-positions.mjs`：SUCCESS。
- `node scripts/generate-domain-type-inventory.mjs` + `node scripts/verify-domain-type-inventory.mjs`：SUCCESS。
- `npm run verify:architecture`：SUCCESS。
- `cargo fmt --all -- --check`：SUCCESS。
- `cargo test --locked -p football-persistence-postgres --lib`：SUCCESS。
- `cargo test --locked -p football-persistence-postgres --test player_names_positions_repository_contract --no-run`：SUCCESS。
- `cargo check --locked -p football-application`：SUCCESS。

## 尚未执行

- R6-04 PostgreSQL 16 ignored contract 真实执行。
- `npm run verify:frontend`。
- workspace Clippy `-D warnings`。
- `cargo test --locked --workspace`。
- clean PR canonical 与 merged-stage canonical gate。
- 用户现有 PostgreSQL 数据真实写入/sample 验收与 Windows Full 人工交互验收。

## 回退点

- R6-04 节点起点：`90f1bb08bf5a7a1580448f791dd241118ea32284`。
- 通过 Git 提交回退，不复制旧实现，不保留长期双 owner。
