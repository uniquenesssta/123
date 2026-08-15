from pathlib import Path

NODE_PATH = Path("docs/modular-rewrite/R06-entity-catalog-persistence/R06-02-team-names-and-profiles.md")
NODE_PATH.write_text("""# R06-02 — Team Names 与 Profiles

## 状态

`VERIFYING`

Team Names 与 Team Profiles 的生产写入 owner 已从旧 `team_catalog.rs` 完整切换到 `adapters/catalog/teams/{names,profiles}/`；旧重复实现已删除，R6-09 删除职责继续留在原 owner。旧 owner 基线 PostgreSQL 契约、切换后专项契约、R6-01/R6-02 ownership、完整 architecture、模型保护、database baseline、frontend、rustfmt、workspace Clippy `-D warnings` 与 workspace tests 均已有实际成功证据。当前仅剩 clean PR canonical、squash merge 与 merged-stage canonical，因此不得提前标记为 `DONE`。

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

## 当前净变更清单（clean PR 前）

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

## 未执行项与剩余门禁

当前尚未宣称以下项目通过：

- clean PR canonical Public Platform CI。
- squash merge 与 merged-stage canonical Public Platform CI。
- 用户现有 PostgreSQL 数据库写入/真实数据 sample 验收。
- Windows Full 人工交互验收。

前两项完成前 R6-02 只能保持 `VERIFYING`。后两项不被云端 Automated 替代，继续作为明确外部/人工限制保留。

## 回退点

- 节点起点：`d8622156b4566cbefaed52606c22af44112aeefe`。
- owner-switch 最小验证成功 HEAD：`a99113dd9290ad168baff4ddb84ee9f40e2764b0`。
- stage hard-gate verified HEAD：`d9adf69892e271ef1b673aaea210c3873cbce570`。
- 回退使用 Git 提交恢复，不复制旧实现或保留长期兼容壳。
""", encoding="utf-8", newline="\n")

stage = Path("docs/modular-rewrite/R06-entity-catalog-persistence/README.md")
text = stage.read_text(encoding="utf-8")
text = text.replace("| R6-02 | Team Names 与 Profiles | READY |", "| R6-02 | Team Names 与 Profiles | VERIFYING |", 1)
marker = "## 当前边界与剩余门禁\n"
section = """## R6-02 当前事实

- 详细记录：[`R06-02-team-names-and-profiles.md`](R06-02-team-names-and-profiles.md)。
- Team Names/Profile 写职责已收敛到 `adapters/catalog/teams/{names,profiles}/`；validation、normalization/input policy、typed Row、Domain Mapper、SQL/transaction owner 分责。
- R6-01 create/update 复用 `names/normalization.rs` 唯一名称规范化 owner；旧 `directory/name_policy.rs` 已删除。R6-09 deletion 仍保留原 owner，没有提前跨节点迁移。
- 旧 owner PostgreSQL 16 baseline run `31897078340` / job `95041920033` 为 `SUCCESS`；owner switch 最终 run `31897403363` / job `95042724731` 为 `SUCCESS`。
- 第一次 owner-switch run `31897301166` 仅因 rustfmt 精确差异 fail-fast；按 rustfmt 输出修复后全链通过，没有降低门禁。
- hard gate 首轮发现 official Domain inventory 漂移；refresh run `31897577673` / job `95043169681` 仅更新官方 inventory 并 `SUCCESS`。
- 第二轮 hard gate 的 Windows job 发现 entity-relationship verifier 仍绑定旧 Profile owner；只推进投影保护检查源到 R6-02 新 owner，原契约未弱化。
- 最终 hard gate run `31897727309`：Windows job `95043538718` 与 PostgreSQL/architecture job `95043538723` 均 `SUCCESS`；frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests、完整 architecture、模型保护、database baseline 与 PG16 contract 均已实际通过。
- 当前等待 clean PR canonical、squash merge 与 merged-stage canonical；R6-02 保持 `VERIFYING`，R6-03 继续 `BLOCKED`。

"""
if marker not in text:
    raise SystemExit("R6 stage boundary marker missing")
text = text.replace(marker, section + marker, 1)
text = text.replace("- R6-02 Team Names 与 Profiles 为当前唯一 `READY` 节点；后续节点继续 `BLOCKED`。", "- R6-02 Team Names 与 Profiles 已进入 `VERIFYING`；当前仅剩 clean PR/merge/merged-stage canonical。\n- R6-03 继续 `BLOCKED`，直到 R6-02 完整收口为 `DONE`。", 1)
stage.write_text(text, encoding="utf-8", newline="\n")

root = Path("README.md")
text = root.read_text(encoding="utf-8")
marker = "### R6-01 Team Directory 与 Detail\n"
section = """### R6-02 Team Names 与 Profiles

- Team Names 与 Team Profiles 写入已从旧 `team_catalog.rs` 收敛到 `crates/persistence-postgres/src/adapters/catalog/teams/{names,profiles}/`；validation/policy、typed Row、Domain Mapper 与 SQL/transaction owner 已分责，旧重复 helper/实现已删除。
- Team 名称规范化收敛到 `names/normalization.rs` 唯一 owner，R6-01 create/update 复用该规则；R6-09 删除职责继续留在 `team_catalog.rs`，未提前跨节点迁移。
- 公共 `TeamCatalogPort`、Tauri command/DTO、Domain shape、Schema、0001–0046 migration、配置、错误文案、审计语义、用户可观察行为、模型保护资产与生产依赖保持不变；Profile head-coach 投影保护与 metadata merge 保持原语义。
- 旧 owner PostgreSQL 16 baseline run `31897078340` / job `95041920033` 为 `SUCCESS`；owner-switch run `31897403363` / job `95042724731` 为 `SUCCESS`。
- 最终阶段 hard gate run `31897727309`：Windows job `95043538718` 与 PostgreSQL/architecture job `95043538723` 均 `SUCCESS`，覆盖 frontend、rustfmt、workspace Clippy `-D warnings`、workspace tests、完整 architecture、模型保护、database baseline 与真实 PG16 contract。
- 当前节点状态为 `VERIFYING`；clean PR canonical、squash merge 与 merged-stage canonical 尚未完成。未对用户现有 PostgreSQL 数据库执行写入/真实数据 sample 验收，也未宣称 Windows Full 人工交互验收完成。详细记录见 `docs/modular-rewrite/R06-entity-catalog-persistence/R06-02-team-names-and-profiles.md`。


"""
if marker not in text:
    raise SystemExit("root README R6-01 marker missing")
text = text.replace(marker, section + marker, 1)
root.write_text(text, encoding="utf-8", newline="\n")

for temporary in [
    ".github/workflows/r6-02-baseline-contract.yml",
    ".github/workflows/r6-02-owner-switch.yml",
    ".github/workflows/r6-02-hard-gate.yml",
    ".github/workflows/r6-02-pre-pr-closeout.yml",
    "scripts/r6-02-pre-pr-closeout.py",
]:
    Path(temporary).unlink(missing_ok=True)
