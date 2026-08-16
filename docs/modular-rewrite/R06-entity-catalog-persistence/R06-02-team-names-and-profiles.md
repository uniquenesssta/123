# R06-02 — Team Names 与 Profiles

## 状态

`DONE`

Team Names 与 Team Profiles 的生产写入 owner 已从旧 `team_catalog.rs` 完整切换到 `adapters/catalog/teams/{names,profiles}/`；旧重复实现已删除，R6-09 删除职责继续留在原 owner。旧 owner 基线 PostgreSQL 契约、切换后专项契约、R6-01/R6-02 ownership、完整 architecture、模型保护、database baseline、frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests、clean PR canonical 与 merged-stage canonical 均已有实际成功证据。PR #33 已按固定 head squash merge，本节点正式 `DONE`。

## 基线与分支

- R6-01 最终 stage closeout HEAD：`d8622156b4566cbefaed52606c22af44112aeefe`。
- R6-02 实施分支：`agent/r6-02-team-names-profiles`，从上述 HEAD 精确建立。
- R6-01 代码 merge commit：`a54df5ca2695297ea5866a3efd74643239568825`；merged-stage canonical run `31892105125` / Windows job `95029805339` 为 `SUCCESS`。

## 实际问题与任务边界

R6-02 开工前，Team Names/Profile 写职责仍集中在 `crates/persistence-postgres/src/team_catalog.rs`：

- `add_team_name` 同时承担输入校验、名称规范化、SQL、动态 Row mapping、审计和事务提交。
- `upsert_team_profile` 同时承担 Profile 校验、文本清洗、SQL、动态 Row mapping、兼容投影保护、metadata 合并、审计和事务提交。
- R6-01 的 `directory/name_policy.rs` 还拥有 Team canonical-name 规范化，若 R6-02 再复制同一规则会产生双 owner。

本节点只迁移 Team Names 与 Team Profiles 写入及其真实共享 normalization policy；明确不提前迁移：

- R6-01：Team Directory/Detail 读取与聚合。
- R6-05：periods/availability。
- R6-07：coach/formation usage。
- R6-09：archive/delete/force-delete。
- R6-10：global name search。

Application `TeamCatalogPort`、Tauri command/DTO、Domain Draft/Record、Schema、0001–0046 migration、错误类别、配置与用户可观察行为保持兼容。

## 最终职责结构

```text
crates/persistence-postgres/src/adapters/catalog/teams/
├─ names/
│  ├─ mod.rs
│  ├─ add_team_name.rs
│  ├─ validation.rs
│  ├─ normalization.rs
│  ├─ row.rs
│  └─ mapper.rs
└─ profiles/
   ├─ mod.rs
   ├─ upsert_team_profile.rs
   ├─ input_policy.rs
   ├─ row.rs
   └─ mapper.rs
```

职责边界：

- `names/add_team_name.rs` 只拥有 Team Name 写事务、SQL 与审计边界。
- `names/validation.rs` 只拥有别名输入合法性与 language-code 清洗。
- `names/normalization.rs` 是 Team 名称规范化唯一 owner；R6-01 create/update 复用该 owner，旧 `directory/name_policy.rs` 删除。
- `names/row.rs` / `mapper.rs` 分别承担 typed SQL Row 与纯 Domain mapping。
- `profiles/upsert_team_profile.rs` 只拥有 Profile upsert 事务、SQL 与审计边界。
- `profiles/input_policy.rs` 只拥有 Profile 校验和可选文本清洗。
- `profiles/row.rs` / `mapper.rs` 分别承担 typed SQL Row 与纯 Domain mapping。
- `team_catalog.rs` 本节点后只保留尚属 R6-09 的球队/球员批量删除与 Team permanent-delete 路径，不保留 R6-02 转发壳或重复 helper。

## 行为与兼容性

保持不变：

- `PostgresStore::add_team_name` 与 `PostgresStore::upsert_team_profile` 的公开方法名、参数和返回类型。
- Application `TeamCatalogPort` 调用链与 Tauri/DTO 边界。
- Team stable ID 与既有 Schema/migration。
- Team name：外层 trim、lowercase + `split_whitespace` normalization、language code trim/空白转 `NULL`、有效期字段。
- Team name 精确错误语义：`球队别名不能为空`、`球队别名结束日期早于开始日期`。
- Team profile 的 `team_type` / `tactical_style` 枚举、成立年份 1850–2100、五项评分 0–100、可信度 0–1 校验及现有错误文案。
- Profile 可选文本 trim/空白转 `NULL`。
- Profile upsert 保留数据库中的 `head_coach` 投影，不使用 Draft 覆盖教练任期投影。
- Profile metadata 继续按 `football.team_profiles.metadata || EXCLUDED.metadata` 合并。
- `team_name_added` 与 `team_profile_updated` audit event 继续和业务写入位于同一显式事务。
- 模型保护资产、Cargo manifests、`Cargo.lock` 与生产依赖均未变化。

## 验证记录

### 旧 owner 基线契约

- PostgreSQL 16 baseline contract run `31897078340` / job `95041920033`：`SUCCESS`。
- 同一 `team_names_profiles_repository_contract` 在旧 `team_catalog.rs` owner 上冻结 name trim/normalization/language/validity、Profile trim/validation、head-coach 投影保护、metadata merge 与错误语义。

### owner switch 最小验证

- run `31897301166` / job `95042464123`：R6-01 ownership 与 R6-02 ownership 已通过；`cargo fmt --all -- --check` 对两个测试断言给出精确格式差异后 fail-fast，后续 check/unit/PG steps 被跳过，没有把该 run 描述为通过。
- 按 rustfmt 精确输出修正格式后，run `31897403363` / job `95042724731`：`SUCCESS`。
- 第二次实际通过：R6-01 retained ownership、R6-02 ownership、rustfmt、Persistence/Application `cargo check`、Persistence unit tests、真实 PostgreSQL 16 R6-02 contract。

### 阶段 hard gate

- 首轮 run `31897523948` 在完整 architecture 中发现 official Domain inventory 因新增 Rust owner 文件发生真实漂移；没有放宽 drift gate。
- inventory refresh run `31897577673` / job `95043169681`：`SUCCESS`；仅 `architecture/domain-type-inventory.json` 被官方 generator 更新，临时 refresh workflow 随后清理。
- 第二轮 hard gate run `31897649045`：Ubuntu job `95043347119` 已通过 R6-01/R6-02 ownership、完整 architecture、模型保护、database baseline 与 PG16 contract；Windows job `95043347170` 在 `verify-entity-relationships.mjs` 仍读取旧 Team Profile owner 时 fail-fast。修复只把“Profile 写入不得覆盖教练任期投影”的检查源切到 `profiles/upsert_team_profile.rs`，删除/引用检查继续读取 R6-09 旧 owner，未删除或弱化原契约。
- 最终 hard gate run `31897727309` 全部 `SUCCESS`：Windows job `95043538718` 的 `npm run verify:frontend`、rustfmt、workspace Clippy `-D warnings`、workspace tests 全部通过；Ubuntu job `95043538723` 的 R6-01/R6-02 ownership、完整 architecture、模型保护、database baseline 与真实 PostgreSQL 16 contract 全部通过。
- 最终 hard-gate 验证源码 HEAD：`d9adf69892e271ef1b673aaea210c3873cbce570`。

### clean PR 与 merged-stage canonical

- 首次 transient closeout workflow run `31926406128` / job `95114586773` 因状态 selector 同时命中两处 `VERIFYING` 而 fail-fast；该 run 未提交任何源码或文档变更。selector 收窄到状态区后重新执行，未放宽任何验证门禁。
- clean PR fixed head：`f04923425f9d660f2ce1275da2b689ca681f3500`。Public Platform CI run `31898334326` / Windows job `95044999641`：`SUCCESS`；artifact `9250723192`，SHA-256 `15362b7164cc2ae0c7b177f4f4899b56678232595caa955d9602adea1eae2ee6`。
- PR #33 使用 expected head `f04923425f9d660f2ce1275da2b689ca681f3500` squash merge；merge commit：`334d5d86f08f5cd1adee5b23dc64d907aeb2eba2`。
- merged-stage Public Platform CI run `31925219003` / Windows job `95111615032`：`SUCCESS`；artifact `9257917686`，SHA-256 `ff29c7326ed23f3736fd433f2adde3aad587f9aa17506b2eedd96e913b7f2e9d`。

## 最终净变更清单

### 新增

- `crates/persistence-postgres/src/adapters/catalog/teams/names/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/names/add_team_name.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/names/validation.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/names/normalization.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/names/row.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/names/mapper.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/profiles/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/profiles/upsert_team_profile.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/profiles/input_policy.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/profiles/row.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/profiles/mapper.rs`
- `crates/persistence-postgres/tests/team_names_profiles_repository_contract.rs`
- `scripts/verify-team-names-profiles.mjs`
- `docs/modular-rewrite/R06-entity-catalog-persistence/R06-02-team-names-and-profiles.md`

### 修改

- `architecture/domain-type-inventory.json`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/create_team.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/mod.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/directory/update_team.rs`
- `crates/persistence-postgres/src/adapters/catalog/teams/mod.rs`
- `crates/persistence-postgres/src/team_catalog.rs`
- `package.json`
- `scripts/verify-entity-relationships.mjs`
- `scripts/verify-team-directory-detail.mjs`
- `README.md`
- `docs/modular-rewrite/R06-entity-catalog-persistence/README.md`

### 移动/重命名

无。

### 删除

- `crates/persistence-postgres/src/adapters/catalog/teams/directory/name_policy.rs`

临时 baseline/owner-switch/hard-gate/inventory/wiring/closeout workflow 与 helper 均在 clean PR 前清理，不进入最终净 diff。

## 公共契约与影响范围

- 公共接口：无变化。
- Tauri command / DTO：无变化。
- Domain 类型与 Serde：无变化；inventory 更新来自 PostgreSQL mapping 源文件归属变化，不是 Domain shape 变化。
- Schema / migration / 持久化格式：无变化。
- 配置 / 环境变量：无变化。
- 错误类型、错误文案、日志等级：无变化。
- UI / 用户可观察行为：无变化。
- 模型保护资产：无变化。
- 生产依赖：无变化。

## 未执行项与剩余限制

R6-02 节点要求的 clean PR、squash merge 与 merged-stage canonical 已全部完成。以下外部/人工验证仍未宣称执行：

- 用户现有 PostgreSQL 数据库写入/真实数据 sample 验收。
- Windows Full 人工交互验收。

上述两项不被云端 Automated 替代；它们不阻塞本次纯持久化职责重写节点收口，但继续作为明确限制保留。

## 回退点

- 节点起点：`d8622156b4566cbefaed52606c22af44112aeefe`。
- owner-switch 最小验证成功 HEAD：`a99113dd9290ad168baff4ddb84ee9f40e2764b0`。
- stage hard-gate verified HEAD：`d9adf69892e271ef1b673aaea210c3873cbce570`。
- clean PR fixed head：`f04923425f9d660f2ce1275da2b689ca681f3500`。
- R6-02 squash merge commit：`334d5d86f08f5cd1adee5b23dc64d907aeb2eba2`。
- 回退使用 Git 提交恢复，不复制旧实现或保留长期兼容壳。
