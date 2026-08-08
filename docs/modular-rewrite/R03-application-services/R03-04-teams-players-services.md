# R03-04 Teams / Players Services

## 状态

`VERIFYING`

Teams / Players Services 源码重写与实施侧 Windows 严格验证已完成；节点仍等待用户 Windows 本机最小验证、完整 frontend / Rust 回归与非破坏性运行时烟测，因此不得标记为 `DONE`，R3-05 继续保持 `BLOCKED`。

## 基线与范围

- R3-04 起点：`212723ce9e0245a37a70f23feda1f156f9ab959a`（R3-03 `DONE` 后开放 R3-04）。
- 实施保护分支：`rewrite/r3-04-teams-players`。
- 当前实施侧已验证源码提交：`3844aaf704b51b5c84da1a44cbd293b4711b8e2c`；清理临时 workflow 后保护分支 HEAD 为 `d1928dfd5774b9f447a9a869ae043e24ab8406f7`。
- 范围：球队目录/详情/编辑/名称/profile、球员目录/详情/编辑/名称/位置/球队履历、球员可用性/能力观察/动态标签/比赛贡献、教练与教练履历、实体引用与匹配、删除预检/批量归档、球队强制删除、数据提供方与外部实体 ID。
- 排除范围：阵型、比赛、阵容、阵容预设及比赛阵容链，这些职责明确保留给 R3-05；不修改具体 PostgreSQL SQL、迁移、Tauri DTO、前端状态、模型实现或生产依赖。

## 实际实施

### 1. Teams Service / Use Cases

新增 `crates/application/src/services/teams/` 与 `crates/application/src/use_cases/teams/`，覆盖 10 个球队职责：创建球队、球队选项、球队列表、球队详情、球队编辑、球队名称、球队 profile、批量删除、强制删除预检、强制删除执行。

`ApplicationService` 的既有球队公开方法名、参数、返回类型和错误传播保持不变；兼容 facade 只负责活动数据库会话取得与 `TeamService` 委托。

### 2. Players Service / Use Cases

新增 `crates/application/src/services/players/` 与 `crates/application/src/use_cases/players/`，覆盖 25 个球员/教练/实体引用职责。球员注册状态历史校验与能力观察 `calculation_version` 非空校验已迁入对应 Use Case，并增加单元测试锁定既有错误语义。

R3-04 共建立 43 个 Teams / Players Service / Use Case Rust 文件，将旧 `player_catalog.rs` 中 35 个公开 Application 职责迁入新的 Services / Use Cases。原文件继续保留，但只持有 R3-05 的 19 个阵型、比赛、阵容与预设职责，避免提前跨节点迁移。

### 3. Ports / 具体持久化适配

沿用 R3-01 已冻结的 6 个最小 Port：

- `TeamCatalogPort`
- `TeamLifecyclePort`
- `PlayerCatalogPort`
- `PlayerSignalPort`
- `CoachCatalogPort`
- `EntityReferencePort`

为避免把新的具体适配实现重新堆叠到 `composition/port_registry.rs`，新增：

```text
crates/application/src/composition/adapters/mod.rs
crates/application/src/composition/adapters/teams.rs
crates/application/src/composition/adapters/players.rs
```

`port_registry.rs` 继续是 Application 内唯一直接导入 `football_persistence_postgres` 的组合根所有者，并提供活动数据库与持久化错误映射；Teams / Players 适配器按职责实现上述 Ports。Service / Use Case 不直接依赖 PostgreSQL、SQLx、PgPool、PostgresStore 或 SQL Row。

### 4. 球队强制删除的 non-Send 边界

首次 Windows 编译验证暴露真实问题：持久化层 `preview_force_delete_team` / `force_delete_team` 使用生命周期敏感的 SQLx transaction，直接放入默认 `async_trait` Send future 会产生 `Executor is not general enough` / `Send is not general enough` 编译错误。

修复保持既有 `TeamLifecyclePort` 和公共 Application/Tauri API 不变，在 Teams 组合适配器中仅对这两个 non-Send 持久化调用显式使用 `tokio::task::spawn_blocking` 与当前 Tokio `Handle::block_on` 桥接；`bulk_delete_teams` 仍走普通 async 路径。没有修改 SQL、事务内容、完整名称确认、事务本地权限或强制删除清理范围，也没有增加 lint 抑制或放宽门禁。

### 5. Tauri 与兼容边界

`src-tauri/src/commands/catalog.rs` 的球队、球员、教练与实体引用公共命令名称、参数和返回 DTO 保持不变。既有球队强制删除 Tauri non-Send 隔离语义继续保留；R3-04 没有把 destructive 操作用于用户原数据库验证。

## 专项门禁

新增 `scripts/verify-teams-players-service.mjs` 并接入：

- `npm run verify:teams-players-service`
- `npm run verify:architecture`
- `npm run verify:frontend`

门禁验证 43 个 R3-04 Service / Use Case 文件、35 个公开 Application 职责迁移、6 个 Ports 的职责适配、Service/Use Case 无具体 PostgreSQL 泄漏、Tauri 兼容调用链、两项历史输入校验，并明确要求 19 个 R3-05 职责继续留在过渡所有者中。

实体关系、球队强制清除、球队与球员管理三个既有静态验证器同步改读新的职责 owner；其业务断言未被删除或放宽。

## 实施侧验证

Windows 2025 严格修复/验证 run `31258038424` / job `93104371481` 全部通过，并生成修复提交 `3844aaf704b51b5c84da1a44cbd293b4711b8e2c`；临时 workflow 已从最终保护分支删除。

已通过：

- Domain 类型清单重新生成与验证：365 个类型、365 个公共兼容类型、299 个 PostgreSQL 映射类型；当前扫描 377 个 Rust 文件。
- `npm run verify:teams-players-service`：43 个 Service / Use Case Rust 文件，35 个公开 Application 职责已切换 Ports，R3-05 职责保持原位。
- `node scripts/verify-entity-relationships.mjs`。
- `node scripts/verify-force-team-delete.mjs`。
- `node scripts/verify-team-player-management.mjs`。
- `npm run verify:architecture`。
- `node scripts/verify-protected-assets-deterministic.mjs`：18 个保护文件一致，聚合 SHA-256 为 `eb4292a0f5616b3b9e5bad1a5cfe135000013e584a998444283efbdf9eef89d4`。
- `cargo check --locked -p football-application`。
- `cargo test --locked -p football-application`：33/33 通过，含新增的注册状态与能力观察版本校验测试。
- `cargo clippy --locked --workspace --all-targets -- -D warnings`。
- `git diff --check`。

首次 Windows 验证真实暴露 non-Send SQLx transaction 适配失败；该失败没有被跳过。修复后所有上述检查均以独立步骤重新执行并通过。

## 尚未完成

仍需用户 Windows 本机作为本节点验收依据：

- R3-04 最小验证复跑；
- 完整 `npm run verify:frontend`；
- 完整 `npm run verify:rust`；
- `npm run tauri:dev` 非破坏性球队、球员、教练、实体引用读取/编辑链烟测。

用户原数据库不得执行强制删除、批量删除或 reset。18 个需要 `FOOTBALL_TEST_DATABASE_URL` 的真实 PostgreSQL 集成测试若未配置专用测试库，继续按既有安全设计保持 `ignored`，不记为已执行。

## 回退与下一步

R3-04 可回退到起点 `212723ce9e0245a37a70f23feda1f156f9ab959a`。不得恢复将球队/球员职责堆叠回 `player_catalog.rs` 的结构，也不得把 R3-05 阵型/比赛/阵容职责提前迁入本节点。

状态保持 `VERIFYING`。只有用户 Windows 本机完整回归和非破坏性运行时烟测通过后，才能将 R3-04 标记为 `DONE` 并开放 R3-05 Lineups Service。