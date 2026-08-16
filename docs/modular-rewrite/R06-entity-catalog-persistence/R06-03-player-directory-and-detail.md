# R06-03 — Player Directory 与 Detail

## 状态

`IN_PROGRESS`

R6-03 从已验证的 R6-02 最终 closeout HEAD `4ce9eb405e897beedf8b122e6794d7c9141b660e` 精确建立实施分支 `agent/r6-03-player-directory-detail`。本节点只重写 Player Directory / Detail 的生产持久化 owner；R6-04 Names/Positions、R6-05 Team Periods/Availability、R6-06 Abilities/Dynamic Tags、R6-09 deletion、R6-10 global search 不提前迁移。

## 进入基线

- R6-02：`DONE`。
- R6-02 merged-stage canonical：run `31925219003` / Windows job `95111615032`，`SUCCESS`。
- R6-02 最终 closeout tree canonical：HEAD `4ce9eb405e897beedf8b122e6794d7c9141b660e`，Public Platform CI run `31926475831` / Windows job `95114758145`，`SUCCESS`；artifact `9258046111`，SHA-256 `76ae850a02080163086f0ec504e3a7c3cfed87ba7063e378b2c168298772cf7f`。

## 开工扫描与实际问题

R6-03 开工时，目标职责仍集中在 `crates/persistence-postgres/src/player_catalog.rs`：

- `create_player`：输入校验、名称规范化、Player INSERT、primary name INSERT、审计与事务提交混在同一方法。
- `update_player`：输入校验、名称规范化、旧 normalized-name 锁定、Player UPDATE、primary-name 切换、审计与事务提交混在同一方法。
- `list_players`：分页游标、NameSearch、球队过滤、状态过滤、多个 LATERAL 聚合、动态 Row mapping 与 page cursor 组装集中在单一方法。
- `list_player_options`：球队上下文过滤、名称/位置/可用性等投影与列表 Row mapping仍由旧 owner 承担。
- `read_player`：基础 Player、Names、Positions、Team Periods、Availability、Ability Profile/Observations、Dynamic Tags、External IDs 的多段查询和最终 `PlayerDetail` 聚合集中在单一方法。

直接上游公共边界保持冻结：Application `PlayerCatalogPort` 的 `create_player`、`update_player`、`list_players`、`read_player` 方法签名不变；Tauri catalog commands/DTO 映射不变。`list_player_options` 仍保持现有 `PostgresStore` 调用语义。

## 目标职责结构

```text
crates/persistence-postgres/src/adapters/catalog/players/
├─ mod.rs
├─ directory/
│  ├─ mod.rs
│  ├─ create_player.rs
│  ├─ update_player.rs
│  ├─ input_policy.rs
│  ├─ normalization.rs
│  ├─ record_row.rs
│  ├─ record_mapper.rs
│  ├─ list_players.rs
│  ├─ list_row.rs
│  ├─ list_mapper.rs
│  ├─ list_player_options.rs
│  ├─ option_row.rs
│  └─ option_mapper.rs
└─ detail/
   ├─ mod.rs
   ├─ read_player.rs
   ├─ read_player_record.rs
   ├─ record_row.rs
   ├─ record_mapper.rs
   ├─ read_names.rs
   ├─ read_positions.rs
   ├─ read_team_periods.rs
   ├─ read_availability.rs
   ├─ read_ability_profile.rs
   ├─ read_ability_observations.rs
   ├─ read_dynamic_tags.rs
   └─ read_external_ids.rs
```

最终文件边界允许在真实 SQL/类型扫描后进一步细分或合并，但不得把第二独立职责重新堆回单文件；`mod.rs` 只负责显式模块出口。

## 必须保持的行为

- 公共 `PlayerCatalogPort`、Tauri command/DTO、Domain shape、Schema、0001–0046 migration、配置、错误语义、日志等级和用户可观察行为不变。
- `create_player` / `update_player` 保留姓名不能为空、身高 120–230 cm、球员不存在等既有错误语义；名称规范化、primary name 切换、metadata merge 与 audit transaction 语义不变。
- `list_players` 保留 limit clamp、成对 pagination cursor、中文/英文/别名/重音/多关键词 NameSearch、team/status 过滤、稳定排序与 next cursor 语义。
- Detail 继续聚合 Names、Positions、Team Periods、Availability、Abilities、Dynamic Tags、External IDs；只移动读取 owner，不改变后续 R6 节点对应写入 owner。
- 默认战术角色/位置映射和历史 P4/运行引用不变。
- 不新增生产依赖，不修改模型保护区。

## 当前验证计划

1. 在旧 owner 上冻结真实 PostgreSQL 16 Player Directory/Detail contract。
2. 按职责建立新 `players/{directory,detail}` 模块并切换唯一 owner。
3. owner-switch 后复跑同一 contract、Rust check/unit、R6-01/R6-02 retained ownership 与 R6-03 ownership。
4. 运行完整 architecture、模型保护、database baseline。
5. 运行阶段回归：frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests。
6. 清理所有临时 workflow/helper，完成 clean PR canonical、squash merge 与 merged-stage canonical 后才允许 `DONE`。

## 当前未执行项

- R6-03 PostgreSQL 16 baseline contract：尚未运行。
- owner switch：尚未执行。
- 阶段 hard gate：尚未运行。
- 用户现有 PostgreSQL 数据库真实数据写入/sample 验收：未执行。
- Windows Full 人工交互验收：未执行。

## 回退点

- R6-03 节点起点：`4ce9eb405e897beedf8b122e6794d7c9141b660e`。
- 所有实施通过 Git 提交回退，不复制旧实现、不保留长期双 owner 或转发壳。
