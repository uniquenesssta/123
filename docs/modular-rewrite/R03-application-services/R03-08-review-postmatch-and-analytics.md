# R3-08 Review / Postmatch / Analytics Services

## 状态

`IN_PROGRESS`

## Atomic Task 1 — Review Core

### 已实施

- 将 `generate_match_review`、`list_reviewable_matches`、`list_match_reviews`、`read_match_review`、`list_ability_candidates`、`decide_ability_candidate` 六个既有公共 Application 职责迁入 `services/review/` 与 `use_cases/review/`。
- `ActiveDatabase` 通过 `composition/adapters/review.rs` 实现 `MatchReviewPort`；Service / Use Case 不直接依赖 PostgreSQL、SQLx、PgPool 或 `PersistenceStore`。
- 删除旧 `crates/application/src/review.rs`，不保留空转发层。
- `ApplicationService`、`ApplicationComposition` 增加唯一 `ReviewService` owner；Tauri Review 命令、公共方法名/参数/返回 DTO 保持不变。
- Match Review Package、Postmatch、Analytics 未在 AT1 修改。

### 验证事实

- Windows AT1 hard gate run `31328023642`：Review 专项、Application Ports、完整 architecture、模型保护资产、rustfmt、Application check/tests、完整 frontend、完整 Rust、精确作用域与 clean tree 均通过。
- clean Review Core tree 已形成正式 Atomic commit `6059d7a79f34bb31a5b81b56a1c15af0a30db11a`。
- 首次 Public Platform CI run `31328591975` 在 `verify:architecture` 停止，明确错误为 Domain 类型与契约清单漂移；Windows Automated 未执行，因此 AT1 仍为 `VERIFYING`。
- 根因：Review owner 拆分改变了 Application Rust 文件/调用面，但 `architecture/domain-type-inventory.json` 仍保存拆分前的 `rustUsageDigest`、扫描文件数量及外部调用者路径。修复只重新生成该确定性清单，不改变 Domain 类型定义、Serde、数据库映射或公共契约。

### 未改变

- PostgreSQL SQL、migration、Schema、数据格式。
- Tauri DTO/命令签名与前端状态。
- 错误语义、日志等级和用户可观察业务行为。
- `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 与模型保护资产。
- 生产依赖。

### 后续门禁

- 刷新 Domain inventory 后重新通过 Review 专项、Application Ports、完整 architecture、保护资产、完整 frontend、完整 Rust。
- 形成 clean recovery commit 后重新运行 Public Platform CI；只有正式 CI 与 Windows Automated 成功并完成记录回写，AT1 才可标记 `DONE`，随后才允许进入 AT2。
