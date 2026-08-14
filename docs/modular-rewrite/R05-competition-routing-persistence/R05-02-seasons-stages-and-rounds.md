# R05-02 — Seasons、Stages 与 Rounds

## 状态

`VERIFYING`

R5-02 的生产 owner 切换和节点 hard gate 已完成；当前等待临时验证文件清理、最终净 diff、clean PR canonical CI、固定 HEAD 合并与 merged stage CI。在这些收口门禁全部成功前不得标记 `DONE`。

## 基线与分支

- R5-01 最终 closeout HEAD：`c8f3ba0f35ccec1f2795328fe887c622a5b8a0f6`。
- 该 HEAD 的 canonical Public Platform CI：run `31779160119`，`SUCCESS`。
- R5 stage 分支：`rewrite/r5-competition-routing-persistence`，R5-02 开工时精确位于上述 HEAD。
- R5-02 实施分支：`agent/r5-02-seasons-stages-rounds`，从上述 HEAD 精确建立。
- 开工扫描时不存在其他 R5-02 分支或并行 owner。

## 实际问题与边界

R5-02 开工前，`crates/persistence-postgres/src/competitions.rs` 在 R5-01 已移出 Competition CRUD 后，仍同时承担：

- Season：`create_season` / `list_seasons` / 私有 `read_season` / 动态 PgRow mapper。
- Stage：`create_stage` / `list_stages` / 私有 `read_stage` / 动态 PgRow mapper。
- Round：`create_round` / `list_rounds` / 私有 `read_round` / 动态 PgRow mapper。
- R5-05 尚未迁移的 `resolve_competition_context` 与 `ensure_scope_id`。

本节点只迁移 Season / Stage / Round 持久化；`resolve_competition_context` 与 scope 校验明确保留给 R5-05。Rule Package、Binding、Route Resolution、Model Run Identity 均未提前实施。

## 实际实现

新增唯一 hierarchy owner：

```text
crates/persistence-postgres/src/adapters/competition/hierarchy/
├─ mod.rs
├─ seasons/
│  ├─ mod.rs
│  ├─ create_season.rs
│  ├─ read_season.rs
│  ├─ list_seasons.rs
│  ├─ record_row.rs
│  └─ record_mapper.rs
├─ stages/
│  ├─ mod.rs
│  ├─ create_stage.rs
│  ├─ read_stage.rs
│  ├─ list_stages.rs
│  ├─ record_row.rs
│  └─ record_mapper.rs
└─ rounds/
   ├─ mod.rs
   ├─ create_round.rs
   ├─ read_round.rs
   ├─ list_rounds.rs
   ├─ record_row.rs
   └─ record_mapper.rs
```

- `competition/mod.rs` 只增加 `mod hierarchy;` 注册；R5-01 `directory/`、`detail/` owner 保持不变。
- Season / Stage / Round 分别使用独立 `sqlx::FromRow` typed Row，并在独立 mapper 中完成 Row -> Domain 映射。
- 每个 create/read/list 文件只承担一个 SQL 目的；本节点写入均为单表 INSERT，不人为增加 transaction 容器。
- Stage 的 `CompetitionKind` 字符串解析仍通过既有 `parse_competition_kind`，未将领域解析规则复制进 SQL。
- 原 `competitions.rs` 删除全部 R5-02 CRUD、私有 read 和动态 PgRow mapper，仅保留 R5-05 context/scope 解析及其既有单元测试；不保留 R5-02 转发壳。
- 新增 `scripts/verify-competition-hierarchy.mjs`，锁定 hierarchy 唯一 owner、typed Row/Mapper、单 SQL 目的、旧 owner 清理、R5-05 保留和“不提前实施 R5-03~R5-06”。
- 既有 `scripts/verify-competition-repository.mjs` 继续完整验证 R5-01，并通过 import 链纳入 R5-02 hierarchy gate；没有弱化或删除 R5-01 门禁。
- `architecture/domain-type-inventory.json` 仅通过项目既有 `generate-domain-type-inventory.mjs` 生成器刷新，未手工修改指纹。
- 第三轮 hard gate 暴露 child `mod.rs` 使用 `pub(super) use` 将仅限直接父模块可见的 helper/Row 向上级重导出，触发 Rust E0364/E0365。修复仅将这 9 个内部别名改为 module-private `use`，没有扩大可见性或改变 SQL/映射/公共接口。

## 行为兼容

以下旧行为由 contract 和 architecture gate 显式锁定：

- Season `name/status` trim、日期、metadata 和列表排序 `c.name, s.starts_on DESC NULLS LAST, s.name`。
- Stage `code/name` trim、`stage_kind.as_str()`、sequence、rules 和列表排序 `c.name, s.name, st.sequence_no, st.name`。
- Round `code/name` trim、sequence、时间字段和列表排序 `st.name, r.sequence_no, r.starts_at NULLS LAST`。
- 三层级的 parent ID/name 与 active competition 过滤保持不变。

以下均未修改：

- Domain Season/Stage/Round 类型和 Serde。
- Application `CompetitionHierarchyPort` 方法、参数、返回类型和调用语义。
- Tauri command / DTO、前端赛事页面和用户可观察行为。
- competition/season/stage/round ID 语义与绑定关系。
- 数据库 Schema、0001–0046 migration、持久化数据格式。
- 配置、环境变量、错误类型/语义、日志等级。
- 路由算法、route result、model identity。
- Cargo manifests、Cargo.lock 和生产依赖。
- `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 与模型保护资产。

## PostgreSQL 契约冻结

新增 `crates/persistence-postgres/tests/competition_hierarchy_repository_contract.rs`，并在切换生产 owner 前先对旧实现运行。

- 首次 baseline run `31793270046` / job `94744700670`：`FAILURE`。失败发生在契约测试编译阶段，因为新测试错误使用不存在的 `CompetitionKind::Knockout`；当时尚未修改 R5-02 生产源码。
- 将测试修正为领域已冻结的 `CompetitionKind::KnockoutTwoLeg` 后，旧 owner baseline run `31793408123`：`SUCCESS`。
- contract 覆盖两条 Season、两条 Stage、两条 Round，冻结 trim、父子关系、类型映射、metadata/rules、时间字段和列表顺序。
- 新 owner 上使用完全同一份 contract 在第四轮 hard gate run `31794402789` / job `94748190282` 再次执行并 `SUCCESS`。

## Domain inventory

- 初次生成 workflow run `31793869960`：`SUCCESS`；Domain 类型仍为 365、公共兼容类型 365、PostgreSQL 映射类型 299，Rust 扫描文件数由 R5-01 基线 676 增至 696。
- canonical rustfmt 改变 Rust 使用图文本后，第二轮 hard gate 正确检测到 inventory digest 漂移；随后使用同一官方生成器再次刷新，workflow run `31794157058`：`SUCCESS`。
- visibility 修复再次改变 Rust usage digest 后，第三次 generator run `31794365962` / job `94748073290` 为 `SUCCESS`，生成 commit `476a1fa20dcdf3cef3e91904de0bf6537db8fdd8`。
- 未绕过 `verify-domain-type-inventory.mjs`，未手工伪造摘要或指纹。

## 验证记录

### 第一轮 hard gate

run `31793915203` / job `94746675596`：总体 `FAILURE`。

已通过：

- R5-01 + R5-02 ownership gate：`SUCCESS`。
- 完整 `npm run verify:architecture`：`SUCCESS`。
- public model boundary：`SUCCESS`。

停止点：

- `cargo fmt --all -- --check` 发现 R5-02 新 Rust 文件仅存在 canonical formatting 差异，`FAILURE`。
- 后续 crate check/tests 和 PostgreSQL contract 因 fail-fast 被正确跳过，未描述为通过。

随后 transient rustfmt workflow run `31793982624` / job `94746881429` 执行 `cargo fmt --all`，`SUCCESS`，仅提交格式变化。

### 第二轮 hard gate

run `31794033828` / job `94747040112`：总体 `FAILURE`。

- R5-01 + R5-02 ownership gate：`SUCCESS`。
- 完整 architecture 在 `verify-domain-type-inventory.mjs` 检测到 rustfmt 后 usage digest 漂移而停止。
- 后续保护资产、format、Rust crate/tests、PostgreSQL contract 因 fail-fast 未执行。
- 已使用项目官方 generator 重新生成 inventory，不放宽门禁。

### 第三轮 hard gate

run `31794195852` / job `94747539204`：总体 `FAILURE`。

已通过：

- R5-01 + R5-02 ownership gate：`SUCCESS`。
- 完整 `npm run verify:architecture`：`SUCCESS`。
- public model boundary：`SUCCESS`。
- `cargo fmt --all -- --check`：`SUCCESS`。

停止点：

- `cargo check --locked -p football-persistence-postgres -p football-application` 发现 hierarchy 三个 child `mod.rs` 的 9 个内部 helper/Row 使用了过宽的 `pub(super) use`，触发 E0364/E0365。
- Persistence/Application tests 与 PostgreSQL contract 因 fail-fast 被正确跳过。
- 只将内部 re-export 改为 module-private `use`，未改变生产行为；之后用官方 generator 刷新 domain inventory。

### 第四轮 hard gate

run `31794402789` / job `94748190282`：`SUCCESS`。

全部通过：

- R5-01 + R5-02 ownership gate。
- 完整 `npm run verify:architecture`。
- public model boundary。
- `cargo fmt --all -- --check`。
- `cargo check --locked -p football-persistence-postgres -p football-application`。
- `cargo test --locked -p football-persistence-postgres`。
- `cargo test --locked -p football-application`。
- `cargo test --locked -p football-persistence-postgres --test competition_hierarchy_repository_contract -- --ignored --test-threads=1`，新 owner PostgreSQL 16 contract `PASS`。

因此节点级最小验证、R5/R4 架构边界、模型保护、Persistence/Application 编译与单测、旧/新 owner PostgreSQL 契约已具有真实通过证据。

## 当前未执行项与限制

- clean PR Public Platform CI 尚未执行。
- workspace Clippy `-D warnings`、workspace tests、Windows frontend/Tauri Automated 需要由 clean PR canonical CI 对最终净 HEAD 执行。
- 既有 `postgres_integration.rs` 18 个 ignored broad PostgreSQL tests 未在本节点执行；R5-02 不进行 destructive database reset。
- 未对用户数据库执行写入；契约仅使用临时 PostgreSQL 16 测试数据库。

## 当前变更分类

### 新增

- `crates/persistence-postgres/src/adapters/competition/hierarchy/mod.rs`
- `crates/persistence-postgres/src/adapters/competition/hierarchy/seasons/*`
- `crates/persistence-postgres/src/adapters/competition/hierarchy/stages/*`
- `crates/persistence-postgres/src/adapters/competition/hierarchy/rounds/*`
- `crates/persistence-postgres/tests/competition_hierarchy_repository_contract.rs`
- `scripts/verify-competition-hierarchy.mjs`
- 本记录文件。

### 修改

- `crates/persistence-postgres/src/adapters/competition/mod.rs`
- `crates/persistence-postgres/src/competitions.rs`
- `scripts/verify-competition-repository.mjs`
- `architecture/domain-type-inventory.json`
- 阶段 `README.md`。
- 根 `README.md`（收口前同步实际事实）。

### 移动/重命名

无。

### 删除

最终净 diff 不保留临时验证 workflow；全部 transient `.github/workflows/r5-02-*` 在验证取证结束后删除。

## 入口切换与回退

- 唯一生产入口已切换为 `adapters/competition/hierarchy/{seasons,stages,rounds}`。
- 旧 `competitions.rs` 不再拥有任何 R5-02 CRUD/read/mapper。
- 若后续 clean PR/合并门禁无法通过，回退点为 R5-01 最终已验证 HEAD `c8f3ba0f35ccec1f2795328fe887c622a5b8a0f6`；不得手工复制旧实现形成双 owner。

## 下一状态门禁

只有在以下全部成立后才可改为 `DONE`：

1. transient workflow 全部清理，最终净 diff 无验证 helper。
2. root README 与本记录/阶段索引与最终事实一致。
3. clean PR canonical Public Platform CI 在固定 HEAD 上通过。
4. 固定 HEAD 合并到 `rewrite/r5-competition-routing-persistence`。
5. merged stage CI 通过。
6. 最终 closeout HEAD 再次通过 canonical Public Platform CI，之后 R5-03 才可开放为 `READY`。
