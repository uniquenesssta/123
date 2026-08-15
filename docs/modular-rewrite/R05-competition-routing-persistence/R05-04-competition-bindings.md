# R05-04 — Competition Bindings

## 状态

`VERIFYING`

Competition Binding PostgreSQL persistence 已完成唯一 owner 切换；旧/新 owner 同一 PostgreSQL 16 契约、R5 ownership、Domain inventory、完整 architecture、模型保护、rustfmt、Persistence/Application 编译与单测均已有真实通过证据。当前仍需完成 transient workflow 清理、最终净 diff 审计、clean PR canonical CI、固定 HEAD 合并与 merged stage CI，因此不得提前标记 `DONE`，R5-05 保持 `BLOCKED`。

## 基线与分支

- R5-03 最终 closeout 基线：`f2b555ae0dda092b1039ff57bcea3f6290b33e61`。
- 该 HEAD 的 canonical Public Platform CI run `31827109598` / Windows job `94853717551` 为 `SUCCESS`，artifact `9229953727`，SHA-256 `fecb5969dfddce59a19af1f4770d0cecdce983585dac1b697f21cd6c403f887d`。
- R5-04 实施分支：`agent/r5-04-competition-bindings`，从上述 HEAD 精确建立。
- 开工扫描时不存在其他 R5-04 分支或并行 Competition Binding persistence owner。

## 实际问题与边界

R5-04 开工前，`crates/persistence-postgres/src/routing.rs` 同时承担：

- `ensure_type_default_binding`
- `create_competition_binding`
- `list_competition_bindings`
- 私有 `read_binding`
- Rule Package route metadata read
- Binding 动态 `PgRow` -> Domain mapper
- Binding list SQL builder
- R5-05 尚未迁移的 `resolve_route` / route decision mapper
- R5-06 尚未迁移的 model/version/parameter registration

本节点只迁移 Competition Binding persistence。`crates/persistence-postgres/src/competitions.rs` 中 R5-05 `resolve_competition_context` / `ensure_scope_id` 继续保留；`routing.rs` 中 R5-05 `resolve_route` 与 R5-06 model registration 同样保留。

源码核对确认 Domain 实际绑定范围只有四类：Competition、Season、Stage 与 CompetitionKind type-default；当前 Domain 契约没有 Round Binding，因此本节点没有新增不存在的 Round 绑定语义。

## 实际实现

新的唯一 Binding persistence owner：

```text
crates/persistence-postgres/src/adapters/competition/bindings/
├─ mod.rs
├─ record_row.rs
├─ record_mapper.rs
├─ package_route_metadata.rs
├─ list_bindings.rs
├─ read_binding.rs
├─ create_binding/
│  ├─ mod.rs
│  ├─ validation.rs
│  ├─ insert_binding.rs
│  └─ transaction.rs
└─ ensure_type_default/
   ├─ mod.rs
   ├─ find_existing_binding.rs
   ├─ insert_binding.rs
   └─ transaction.rs
```

- `record_row.rs` 使用 typed `sqlx::FromRow`；Row 与 Domain Mapper 分离，不再使用动态 `PgRow`。
- `list_bindings.rs` 与 `read_binding.rs` 各自只拥有一个明确 SELECT 目的。
- `package_route_metadata.rs` 只读取 active Rule Package 的 model version、parameter set 与 competition kind，并继续复用共享 `parse_competition_kind`。
- `create_binding/validation.rs` 只承担空 scope 与有效期窗口校验，不包含 SQL。
- `create_binding/insert_binding.rs` 只拥有 Binding INSERT；`transaction.rs` 只编排 scope resolution、package metadata、insert、audit、commit 与 typed detail read。
- `ensure_type_default/find_existing_binding.rs` 只拥有 type-default 幂等查询；`insert_binding.rs` 只拥有 INSERT；`transaction.rs` 只做事务编排与 audit。
- `crates/persistence-postgres/src/adapters/competition/mod.rs` 注册 `mod bindings;`，R5-01 directory/detail 与 R5-02 hierarchy owner 保持不变。
- 原 `routing.rs` 已删除全部 R5-04 create/list/default/read/package metadata/dynamic mapper/query builder，不保留 Binding 转发壳。
- 新增 `scripts/verify-competition-bindings.mjs` 并串入现有 R5 competition architecture gate。
- 随阶段推进同步更新 R5-03 verifier 与 R4-03 mapping verifier：不是删除旧断言，而是把“Binding 必须留在 routing.rs”的旧边界改为“Binding 必须迁出、R5-05/R5-06 必须继续留在旧 owner”，并把共享 `CompetitionKind` parser 的调用路径锁到实际新 owner。

## 行为兼容

同一 PostgreSQL contract 显式冻结以下既有行为：

- scope 为空继续返回 `InvalidState`，错误语义保持“绑定范围不能为空；至少指定赛事、赛季、阶段或赛事类型”。
- `valid_to < valid_from` 继续拒绝。
- 显式 CompetitionKind 与赛事层级解析结果不一致继续拒绝。
- Rule Package CompetitionKind 与 resolved scope kind 不一致继续拒绝。
- Competition / Season / Stage Binding 仍通过既有 `resolve_competition_context` 补全 parent IDs 与真实 competition kind。
- type-default Binding 仍以相同 package + kind + active 为幂等条件；重复调用返回既有 ID，不重复写 `type_default_binding_created` audit。
- 非空 `binding_name` 保持原值，不新增 trim；空/缺省名称继续生成 `赛事规则绑定-{uuid前8位}`。
- current binding list 继续过滤 `is_active = true`、`valid_from <= now()`、`valid_to >= now()`，排序保持 `priority DESC, created_at DESC, id DESC`。
- detail read 仍按 ID 读取，不额外套用当前有效期过滤。
- 创建审计类型保持 `competition_binding_created`，默认绑定审计保持 `type_default_binding_created`。
- R5-05 route resolution 结果未迁移；契约确认 Stage Binding 仍优先于更宽 scope 并返回原 Binding/Rule Package/priority。

以下均未修改：

- Domain `CompetitionBindingDraft` / `CompetitionBindingSummary` / Route DTO 与 Serde。
- Application `RuleRoutingPort` 方法、参数、返回类型与调用语义。
- Tauri command / DTO、前端赛事/规则页面及用户可观察行为。
- Schema、0001–0046 migration、历史数据格式和数据库兼容路径。
- 配置、环境变量、错误类型/日志等级。
- R5-05 route resolution 算法与 route result。
- R5-06 model identity / model registration 语义。
- Cargo manifests、Cargo.lock 与生产依赖。
- `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 及模型保护资产。

## PostgreSQL 契约冻结

新增 `crates/persistence-postgres/tests/competition_bindings_repository_contract.rs`，并在生产 owner 切换前先运行旧实现。

覆盖内容：

- empty scope / invalid date window / hierarchy kind mismatch / package kind mismatch。
- type-default 创建、幂等、原始数据库字段与 audit 唯一性。
- Competition / Season / Stage Binding 的 resolved scope、名称、模型/规则包信息与优先级。
- future-valid Binding 不进入当前有效列表以及现有排序。
- Binding 创建 audit。
- `resolve_route` 仍选择 Stage Binding，冻结 R5-05 输出不受 R5-04 owner 切换影响。

执行事实：

- 首次旧 owner run `31862259445` / job `94957570121`：`FAILURE`。停止点只是新增 contract 的 canonical rustfmt 差异，PostgreSQL contract 按 fail-fast 未执行，生产源码尚未修改。
- contract format run `31862305189` / job `94957686819`：`SUCCESS`，仅格式化新增测试。
- 旧 owner PostgreSQL 16 contract run `31862339200` / job `94957776614`：`SUCCESS`。
- 新 owner minimum gate run `31862713824` / job `94958748006`：`SUCCESS`，同一 PostgreSQL 16 contract 再次通过。
- 第二轮 hard gate run `31862969585` / job `94959376165`：`SUCCESS`，同一 contract 第三次通过。

## Owner 切换与门禁修复记录

- owner-switch workflow run `31862539782` / job `94958314365`：`SUCCESS`；生产切换提交 `2bd703d4b1624e8b73712d1f549e5d4b0f7a80f9`。
- 第一轮 new-owner minimum gate run `31862639885` / job `94958556589`：`FAILURE`。R5-04 自身 verifier 已通过，失败来自 R5-03 verifier 仍要求 Binding 留在 `routing.rs`；rustfmt/compile/tests/PG contract 按 fail-fast 跳过。
- 只推进 R5-03 边界断言：继续完整验证 Rule Package owner，同时要求 Binding 已迁出，并继续强制 R5-05 `resolve_route` 与 R5-06 model registration 留在旧 owner。第二轮 minimum gate `31862713824` / job `94958748006` 全部 `SUCCESS`。
- official Domain inventory refresh run `31862856576` / job `94959096109`：`SUCCESS`；仅使用项目既有 generator 与 drift verifier，生成提交 `4d88a60cd054b1c340974182be268b10edb050dc`。
- 第一轮 stage hard gate `31862890012` / job `94959177565`：`FAILURE`。完整 architecture 在 R4-03 mapping verifier 停止，原因是旧门禁硬编码要求 `routing.rs` 直接调用共享 CompetitionKind parser；模型保护、format、compile/tests/PG contract 均正确跳过。
- 只更新 R4-03 shared-parser 调用路径断言：继续锁定唯一 `competition_kind.rs` owner 与 crate 内共享出口，并明确验证 Binding mapper/package metadata、Rule Package mapper、competition 与 P4 owner 均复用该 parser，没有把领域解析规则复制到 SQL或新模块。
- 第二轮 stage hard gate `31862969585` / job `94959376165`：`SUCCESS`。

## 第二轮 stage hard gate

全部通过：

- 完整 `npm run verify:architecture`，包含 R1–R5 现有 ownership/boundary/inventory 门禁。
- public model boundary。
- protected model assets。
- `cargo fmt --all -- --check`。
- `cargo check --locked -p football-persistence-postgres -p football-application`。
- `cargo test --locked -p football-persistence-postgres`。
- `cargo test --locked -p football-application`。
- 同一 Competition Binding PostgreSQL 16 contract。

本轮 architecture 报告确认 Domain 类型仍为 365、公共兼容类型 365、PostgreSQL mapping 类型 299；受保护导入扫描 724 个 Rust 文件。

## 当前未执行项与限制

- clean PR canonical Public Platform CI、固定 HEAD merge、merged stage CI 与最终 closeout canonical CI 尚未执行，因此节点保持 `VERIFYING`。
- 既有 `crates/persistence-postgres/tests/postgres_integration.rs` 18 个 ignored broad PostgreSQL tests 未在 R5-04 执行。
- 未执行 destructive database reset；R5-04 contract 只使用 GitHub Actions 临时 PostgreSQL 16 测试数据库，未触碰用户数据库。
- Windows frontend、workspace Clippy/workspace tests、Tauri release/runtime canonical 交付验收将在 clean PR Public Platform CI 对固定 clean HEAD 执行；当前不得用 Ubuntu 节点门禁替代其结论。
- 当前仍存在本节点 transient `.github/workflows/r5-04-*`，clean PR 前必须全部删除。

## 当前变更分类

### 新增

- `crates/persistence-postgres/src/adapters/competition/bindings/**`
- `crates/persistence-postgres/tests/competition_bindings_repository_contract.rs`
- `scripts/verify-competition-bindings.mjs`
- 本记录文件。

### 修改

- `crates/persistence-postgres/src/adapters/competition/mod.rs`
- `crates/persistence-postgres/src/routing.rs`
- `scripts/verify-competition-repository.mjs`
- `scripts/verify-rule-package-repository.mjs`
- `scripts/verify-persistence-mapping.mjs`
- `architecture/domain-type-inventory.json`
- R5 阶段 README 与根 README（clean PR 前同步）。

### 删除

- `routing.rs` 中 R5-04 Binding persistence owner；不保留转发壳。
- transient workflow 仅用于验证取证，最终 clean diff 不得保留。

## 入口切换与回退

- 唯一 Binding persistence 入口已切换到 `adapters/competition/bindings/`。
- `routing.rs` 不再拥有 R5-04 CRUD/default/read/mapper/query helper。
- R5-05 `resolve_route` / competition context 与 R5-06 model registration 保持原 owner。
- 若 clean PR 或合并门禁无法通过，回退点为 R5-03 最终绿色基线 `f2b555ae0dda092b1039ff57bcea3f6290b33e61`；不得复制旧 Binding 实现形成双 owner。

## 下一状态门禁

R5-04 当前保持 `VERIFYING`。只有在 transient workflow 全部清理、README/阶段索引与最终净 diff 一致、clean PR canonical CI 对固定 HEAD 成功、固定 HEAD 合并到 `rewrite/r5-competition-routing-persistence`、merged stage CI 成功并完成最终 closeout canonical CI 后，才可标记 `DONE` 并开放 R5-05 `READY`。
