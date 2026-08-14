# R05-03 — Rule Packages

## 状态

`DONE`

R5-03 的生产 owner 切换、契约冻结、节点 hard gate、clean PR canonical CI、固定 HEAD 合并与 merged stage CI 均已完成；R5-03 正式关闭为 `DONE`，R5-04 开放为 `READY`。本次 closeout 文档提交仍需通过 canonical Public Platform CI 后，才可作为 R5-04 的起始基线。

## 基线与分支

- R5-02 最终验证基线：`bcd17c78dde89c999d7666fa41e7267f609c6a33`。
- 该 HEAD 的 canonical Public Platform CI run `31798978218` / Windows job `94762276340` 为 `SUCCESS`。
- R5-03 分支：`agent/r5-03-rule-packages`，从上述 HEAD 精确建立。
- 开工时不存在其他 `agent/r5-03-rule-packages` 分支或并行 Rule Package owner。

## 实际问题与边界

R5-03 开工前，`crates/persistence-postgres/src/routing.rs` 仍同时承担 Rule Package 与后续 Binding / Route Resolution / Model Run Identity 相关职责。本节点只迁移：

- `register_rule_package`
- `list_rule_packages`
- Rule Package source document upsert
- Rule Package Row -> Domain 映射

R5-04 Competition Bindings、R5-05 Route Resolution Reads、R5-06 Model Run Identity Reads 均未提前迁移。共享的 model/version/parameter registration 仍由原 routing owner 持有，仅将 `register_model_in_tx` 调整为 crate 内可见供 R5-03 事务编排复用。

## 实际实现

唯一 Rule Package persistence owner：

```text
crates/persistence-postgres/src/adapters/rules/
├─ mod.rs
└─ packages/
   ├─ mod.rs
   ├─ list_rule_packages.rs
   ├─ record_row.rs
   ├─ record_mapper.rs
   ├─ source_documents/
   │  ├─ mod.rs
   │  └─ upsert_source_document.rs
   └─ register_rule_package/
      ├─ mod.rs
      ├─ transaction.rs
      ├─ insert_rule_package.rs
      ├─ find_existing_package.rs
      └─ attach_competition_profile.rs
```

- `transaction.rs` 只负责事务编排，不嵌入 SQL。
- package INSERT、existing-package conflict SELECT、competition profile guarded UPDATE、source-document upsert 分别由独立文件拥有单一 SQL 目的。
- Rule Package list 使用 typed `sqlx::FromRow`，Row 与 Domain Mapper 分离，不再使用动态 `PgRow` mapper。
- `record_mapper.rs` 继续通过既有 `parse_competition_kind` 恢复 `CompetitionKind`，没有把领域解析规则复制到 SQL。
- 原 `routing.rs` 删除全部 R5-03 register/list/source-document/RulePackage mapper 职责，不保留转发壳；Binding、Route Resolution 与共享 model registration 职责继续留在旧 owner。
- `crates/persistence-postgres/src/adapters/mod.rs` 注册新的 `rules` adapter。
- 新增 `scripts/verify-rule-package-repository.mjs`，并由既有 `verify-competition-repository.mjs` 串入 `verify:architecture`；R5-01/R5-02 门禁继续完整执行。

## 行为兼容

以下旧行为由同一 PostgreSQL contract 与 architecture gate 显式冻结：

- register 仍在单一事务内依次处理 source document、model/version/parameter、competition profile、rule package 与 audit。
- `package_key + version` 相同且内容相同保持幂等，返回既有 package ID；不重复写入 `rule_package_registered` audit。
- 同 key/version 内容不同继续返回 `InvalidState`，错误语义保持“已存在但内容不同；请创建新版本”。
- 已绑定不同 Competition Profile Version 继续返回 `InvalidState`。
- source document 仍按 `content_sha256` 去重；冲突时 `source_uri` 使用原有 `COALESCE(existing, excluded)` 语义，metadata 使用 JSON merge。
- `source_type` 保持 `competition_rule_standard`。
- list 排序保持 `created_at DESC, package_key, version`。
- list 仍通过既有 CompetitionKind parser 映射；register 返回值仍保持旧实现的 `created_at = Utc::now()` 语义。

以下均未修改：

- Domain `RulePackageDraft` / `RulePackageSummary` / `RuleSourceReference` 类型与 Serde。
- Application `RulePackagePort` 方法、参数、返回类型与调用语义。
- Tauri command / DTO、前端赛事与规则页面、用户可观察行为。
- Schema、0001–0046 migration、历史数据格式和数据库兼容路径。
- 配置、环境变量、错误类型/日志等级。
- Competition Binding、route resolution 算法、route result、model identity。
- Cargo manifests、Cargo.lock 与生产依赖。
- 模型公开边界及受保护模型资产。

## PostgreSQL 契约冻结

新增 `crates/persistence-postgres/tests/rule_package_repository_contract.rs`，在生产 owner 切换前先对旧实现运行：

- old owner baseline run `31810196287` / job `94798701210`：`SUCCESS`，1/1 PASS。
- contract 覆盖 register/list、重复注册幂等、同 key/version 异内容拒绝、source document 去重/upsert、manifest/profile/routing/feature/output 原始 JSON、Competition Profile/source ID、排序与 audit 事实。

切换后使用完全同一份 contract：

- new owner run `31811073556` / job `94801559817`：`SUCCESS`，同一 contract 1/1 PASS。
- 第二轮节点 hard gate run `31811535324` / job `94803075524` 再次执行同一 PostgreSQL 16 contract并 `SUCCESS`。

## 实施与失败记录

- 初次 owner-switch run `31810593242` / job `94800000405`：`FAILURE`。Rust E0364/E0603 暴露 `upsert_rule_source_document` 子模块可见性不足；当时生产 owner 切换提交没有产生。
- 可见性仅收窄修正为 `pub(in crate::adapters::rules::packages)`，没有扩大为公共 API。
- repair run `31810761979` / job `94800547447`：可见性替换步骤成功，但 Action 尝试修改 `.github/workflows/*` 时被 GitHub App workflow 权限拒绝；这不是产品代码失败，也没有生产提交。
- owner-switch V2 run `31810913317` / job `94801046458`：`SUCCESS`。在提交前完成 rustfmt、Persistence/Application `cargo check --locked` 与 R5-03 专项 verifier，随后生成生产 owner 切换提交。
- official Domain inventory refresh run `31811229904` / job `94802081839`：`SUCCESS`；只使用 `scripts/generate-domain-type-inventory.mjs` 生成，并由 drift verifier 复核。
- 第一轮 hard gate run `31811298187` / job `94802307448`：总体 `FAILURE`。R5 ownership/full architecture、模型保护均通过；停止点为新增 contract 文件的 canonical rustfmt 差异，compile/tests/contract 按 fail-fast 正确跳过。
- format + inventory run `31811465209` / job `94802847771`：`SUCCESS`，仅执行 canonical rustfmt、官方 inventory generator/verification 并提交实际变化。
- 第二轮 hard gate run `31811535324` / job `94803075524`：`SUCCESS`。

## 第二轮节点 hard gate

全部通过：

- R5-01 / R5-02 / R5-03 ownership 与完整 `npm run verify:architecture`。
- public model boundary 与 protected assets。
- `cargo fmt --all -- --check`。
- `cargo check --locked -p football-persistence-postgres -p football-application`。
- `cargo test --locked -p football-persistence-postgres`。
- `cargo test --locked -p football-application`。
- 同一份 Rule Package PostgreSQL 16 contract。

最终 inventory 仍为 365 个 Domain 类型、365 个公共兼容类型、129 个 Domain 来源文件、299 个 PostgreSQL 映射类型；Rust 使用图扫描 709 个文件。

## PR / 合并 / 收口证据

- PR #28 clean Public Platform CI run `31812316772` / Windows automated delivery gate job `94805647551` 为 `SUCCESS`；artifact `9224448312`，13,919,752 bytes，SHA-256 `00cf61b683b7780a354ad5e35c59438d6d26ce1a6d0c17dd86edbaaa9e9ae127`。
- PR #28 固定 clean head `d087108d5cc722cc9ebecd788de6c07c1ae5dc3d` 已按 expected-head 防漂移检查 squash merge，stage merge commit 为 `3d609ace7cbe1db3a3caec18a6477fb41153cc88`。
- 合并后 Public Platform CI run `31824523536` / Windows automated delivery gate job `94845378301` 为 `SUCCESS`；artifact `9229038667`，13,918,817 bytes，SHA-256 `9c07c35abcd8b229b8044b0373f7ce3aecd6a03d58b1431c4f738e22131c7262`。
- clean PR 与 merged stage 的 Windows Automated 均覆盖 canonical architecture、frontend、workspace Rust/Clippy/tests、Tauri release/runtime 验收；未使用较弱结果替代 canonical 交付门禁。
- 全部 transient `.github/workflows/r5-03-*` 已在 clean PR 前清理；最终 PR diff 不含临时 workflow。

## 当前未执行项与限制

- 既有 `postgres_integration.rs` 18 个 ignored broad PostgreSQL tests 未在 R5-03 执行。
- 未执行 destructive database reset，未触碰用户数据库；R5-03 contract 仅使用临时 PostgreSQL 16 测试库。
- PR #28 clean canonical CI、固定 HEAD squash merge 与 merged stage canonical CI 均已完成并记录在上方。
- transient `.github/workflows/r5-03-*` 已全部清理；本 closeout 仅修改文档。

## 下一状态门禁

R5-03 的 clean PR CI、固定 HEAD merge 与 merged stage CI 已全部完成，节点状态为 `DONE`；R5-04 现可按任务书进入 `READY`。本次 closeout 仅修改文档，最终 stage closeout HEAD 仍需通过 canonical Public Platform CI 后，才可作为 R5-04 的起始基线。
