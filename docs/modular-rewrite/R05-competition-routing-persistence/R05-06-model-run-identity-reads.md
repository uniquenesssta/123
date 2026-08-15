# R05-06 Model Run Identity Reads

状态：`DONE`

## 1. 节点目标与当前结论

R5-06 将 Model Run Identity 读取与模型注册持久化职责从 legacy `crates/persistence-postgres/src/routing.rs` 收敛到唯一模块目录 `crates/persistence-postgres/src/adapters/competition/model_run_identity/`。当前源码、专项 PostgreSQL 16 契约与 canonical Windows 全链验证均已通过，根 README 与阶段 README 已同步；节点仍保持 `VERIFYING`，等待文档 clean HEAD canonical CI、clean PR、stage merge 与 merged-stage canonical CI 完成后再关闭为 `DONE`。

## 2. 实际影响范围

- Model Run 单条详情读取中的模型 identity 查询。
- `PostgresStore::register_model` 与 crate 内共享 `register_model_in_tx`。
- Rule Package 注册事务对模型注册能力的调用路径。
- R5 历史 ownership verifier 对已完成 R5-06 owner 的断言路径。
- Domain inventory 的 Rust usage digest。

未修改 `list_recent_runs` 查询，避免将本节点扩展为历史列表重写或引入 N+1 查询；未修改模型算法、路由决策算法、前端、migration、Schema、配置、依赖或用户数据库。

## 3. 修改前职责

- `model_runs.rs::read_run` 内直接 JOIN `model.versions`、`model.definitions`、`model.parameter_sets` 与 `model.rule_packages` 读取运行 identity。
- legacy `routing.rs` 仍持有 `ModelRegistration`、`PostgresStore::register_model`、`register_model_in_tx`、definition/version/parameter-set SQL 与冲突校验。
- Rule Package registration 通过 legacy routing owner 复用模型注册事务。

## 4. 修改后职责边界

```text
adapters/competition/model_run_identity/
├── mod.rs                         # 显式出口
├── read_identity.rs               # 单次 Model Run identity SELECT
├── record.rs                      # identity record 与 typed Row -> record 映射
├── record_row.rs                  # sqlx::FromRow typed row
└── registration/
    ├── mod.rs                     # 显式出口
    ├── record.rs                  # ModelRegistration 公共契约
    ├── transaction.rs             # register_model / register_model_in_tx 事务编排
    ├── definition.rs              # model definition upsert
    ├── version.rs                 # model version insert + typed conflict read
    └── parameter_set.rs           # parameter set insert + typed conflict read
```

`model_runs.rs::read_run` 只委托 `read_model_run_identity` 获取 identity，主 run document 查询不再重复拥有模型 identity JOIN。Rule Package transaction 改为经 `adapters::register_model_in_tx` 访问唯一新 owner。

## 5. 公共接口、DTO、Schema 与行为

- `ModelRegistration { model_version_id, parameter_set_id }` 字段与序列化契约保持不变。
- `PostgresStore::register_model` 调用语义保持不变。
- `read_run` JSON 的 `model_key`、`model_version`、`parameter_version`、`rule_package_id`、`rule_package_key`、`rule_package_version`、`rule_package_name`、`route_binding_id` 保持不变。
- nullable Rule Package / Binding identity 语义保持不变。
- SQL/RowNotFound 继续映射为既有 `PersistenceError::Sqlx`。
- 模型版本冲突错误保持：`模型版本 {model_version} 已存在但引擎或 Schema 不一致；请创建新模型版本`。
- 参数版本冲突错误保持：`参数版本 {parameter_version} 已存在但内容不同；请创建新参数版本`。
- migration / Schema：无变化。
- 配置 / 环境变量：生产无变化；专项测试继续使用现有测试约定 `FOOTBALL_TEST_DATABASE_URL`。
- 公共 API / Tauri command / UI：无变化。
- 模型保护资产与模型算法：无变化。

## 6. 实际新增文件

- `crates/persistence-postgres/src/adapters/competition/model_run_identity/mod.rs`
- `crates/persistence-postgres/src/adapters/competition/model_run_identity/read_identity.rs`
- `crates/persistence-postgres/src/adapters/competition/model_run_identity/record.rs`
- `crates/persistence-postgres/src/adapters/competition/model_run_identity/record_row.rs`
- `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/mod.rs`
- `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/record.rs`
- `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/transaction.rs`
- `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/definition.rs`
- `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/version.rs`
- `crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/parameter_set.rs`
- `crates/persistence-postgres/tests/model_run_identity_repository_contract.rs`
- `scripts/verify-model-run-identity.mjs`

## 7. 实际修改文件

- `architecture/domain-type-inventory.json`
- `crates/persistence-postgres/src/adapters/competition/mod.rs`
- `crates/persistence-postgres/src/adapters/mod.rs`
- `crates/persistence-postgres/src/adapters/rules/packages/register_rule_package/insert_rule_package.rs`
- `crates/persistence-postgres/src/adapters/rules/packages/register_rule_package/transaction.rs`
- `crates/persistence-postgres/src/lib.rs`
- `crates/persistence-postgres/src/model_runs.rs`
- `scripts/verify-competition-bindings.mjs`
- `scripts/verify-competition-repository.mjs`
- `scripts/verify-route-resolution.mjs`
- `scripts/verify-rule-package-repository.mjs`

历史 verifier 的修改仅将 owner/call-path 断言推进到已完成的 R5-06 边界；没有删除、跳过或放宽原架构约束。

## 8. 实际移动或重命名文件

无。

## 9. 实际删除文件

- `crates/persistence-postgres/src/routing.rs`

该文件在 R5-03～R5-05 后仅剩 R5-06 模型注册职责；新 owner 切换完成后删除，不保留转发壳。

## 10. 入口切换与旧实现清理

- `read_run` 已从内联 identity JOIN 切换为 `read_model_run_identity(self, run_id)`。
- `register_model` / `register_model_in_tx` 已切换到 `model_run_identity/registration/`。
- Rule Package registration 已改用新 crate 内 registration boundary。
- persistence 顶层仍显式 re-export `ModelRegistration`，保持既有公共导出契约。
- legacy `routing.rs` 已删除。
- 所有临时 inventory / PostgreSQL / docs workflow 已在验证或同步后删除，最终树不保留临时 gate。

## 11. 专项验证与真实结果

### PostgreSQL 16

最终隔离数据库验证 run `31881761988`：`SUCCESS`。

- job `95005200186`：`model-run-identity-contract` — `SUCCESS`
  - `node scripts/verify-model-run-identity.mjs`
  - `cargo test --locked -p football-persistence-postgres --test model_run_identity_repository_contract -- --ignored --nocapture`
  - 结果：1 passed / 0 failed。
- job `95005200191`：`rule-package-registration-regression` — `SUCCESS`
  - `cargo test --locked -p football-persistence-postgres --test rule_package_repository_contract -- --ignored --nocapture`
  - 结果：1 passed / 0 failed。

两项使用独立 GitHub Actions PostgreSQL 16 service database；未连接或修改用户数据库。

### canonical Windows 全链

clean source HEAD `dc6d2baf0f6092f67200214b5b45e37c528394ad` 的 Public Platform CI run `31881842368` / Windows automated delivery gate job `95005434692`：`SUCCESS`。

该 canonical gate 实际完成 architecture verifier、frontend verification、Rust lock/rustfmt/workspace Clippy `-D warnings`、workspace tests、Tauri build、release binary startup 与 runtime error scan。validation artifact：`9246499189`，SHA-256 `74bc2a687b8499f798c0a91a28522fa74df0237ee24d005197067b1c97a1e452`。

## 12. 中间失败与订正

- 初始 Public Platform CI run `31878845597` 失败；未作为通过节点继续推进。
- 新 Rust 文件导致 Domain inventory drift；仅通过项目官方 `generate-domain-type-inventory.mjs` / `verify-domain-type-inventory.mjs` 刷新，未绕过 inventory gate。
- 历史 R5 verifier 仍断言 legacy owner；只更新 owner/call-path 到新边界，保留原约束强度。
- rustfmt gate发现 `read_identity.rs` 与新增 contract 格式差异；按 canonical rustfmt 修复。
- 首次专项 PostgreSQL run `31881498897` 暴露 crate visibility E0603；仅修正 crate 内 visibility，没有扩大公共 API。
- 后续 PostgreSQL workflow 首轮将两个 ignored contract 复用同一数据库，第二个 contract 因重复执行 migration 出现 `relation "settings" already exists`；改为两个独立 PostgreSQL service database 后 run `31881761988` 双 job 全部通过。该问题属于测试隔离，不是生产数据库逻辑失败。

## 13. 未执行项与剩余风险

源码、专项契约、clean-source canonical CI、根 README 与阶段 README 同步已完成。当前未完成项仅为发布流程：

- 文档同步后 fixed clean HEAD canonical CI；
- clean PR CI；
- squash merge 到 `rewrite/r5-competition-routing-persistence`；
- merged-stage canonical CI；
- R5 阶段完成记录与出口门禁。

因此本节点当前只能保持 `VERIFYING`。

## 14. 计划偏差

任务名为 Model Run Identity Reads，但 legacy `routing.rs` 在 R5-05 后还唯一持有模型注册 identity 写入职责。若只迁移读取会保留重复/残余 legacy owner，无法满足 R5 阶段“旧 routing module 删除、唯一 Adapter owner”的出口门禁。因此本节点在同一 identity 边界内同步迁移 `ModelRegistration` 与注册事务；未扩展到模型算法、路由算法或其他 Model Run 写入职责。

## 15. 回退点与回退方法

R5-06 开始前已验证基线：`7d61d0f70d39966b44e1a5cc001b34c2788ea04d`。

如需回退，应通过 Git 回退到该已验证提交或后续正式合并提交，不通过复制 legacy 文件恢复双实现。

## 16. 当前验收结论

- 唯一新模块 owner：已完成。
- legacy `routing.rs` 清理：已完成。
- typed Row / 独立 mapper / transaction / SQL responsibility：已完成。
- 公共接口、错误、JSON、Schema、migration、模型行为兼容：保持不变。
- PostgreSQL 16 专项契约：通过。
- canonical Windows clean-source 全链：通过。
- 根 README / 阶段 README：已同步。
- 文档 clean HEAD / PR / merged-stage gate：待执行。

**当前状态：`DONE`。**


## 17. 正式收口证据

- fixed clean HEAD：`058884e82f3c584f87751dec3bb5f9b6531a151e`。
- 文档同步后 push canonical：run `31883239072` / job `95008635990` — `SUCCESS`；artifact `9246859935`，SHA-256 `133baec479d64edd8217bb8d7c1c7f82b0171d2e6362c5d49277caf6b51413f1`。
- PR #31 canonical：run `31883703954` / job `95009730520` — `SUCCESS`；artifact `9246957428`，SHA-256 `da854dc637f59e2832f4515b4154f46a4ae88a7fb7e141ce8ee18b36c132e001`。
- PR #31 使用 expected head `058884e82f3c584f87751dec3bb5f9b6531a151e` squash merge；stage merge commit：`acb0491003b365b3f775780d8c98ecfdf1e80104`。
- merged-stage canonical：run `31884882480` / Windows job `95012628426` — `SUCCESS`；artifact `9247274708`，SHA-256 `4c6b107922787a732537a83b58b005bb364ff10298146c87f0dff40f8bdf5a26`。
- R5-06 自此正式 `DONE`；后续仅进入 R5 阶段级完成记录与 R6 基线建立，不再修改本节点生产实现。
