# R3-08 Review / Postmatch / Analytics Services

## 状态

`IN_PROGRESS`

## Atomic Task 1 — Review Core

状态：`DONE`

### 已实施

- 将 `generate_match_review`、`list_reviewable_matches`、`list_match_reviews`、`read_match_review`、`list_ability_candidates`、`decide_ability_candidate` 六个既有公共 Application 职责迁入 `services/review/` 与 `use_cases/review/`。
- `ActiveDatabase` 通过 `composition/adapters/review.rs` 实现 `MatchReviewPort`；Service / Use Case 不直接依赖 PostgreSQL、SQLx、PgPool 或 `PersistenceStore`。
- 删除旧 `crates/application/src/review.rs`，不保留空转发层。
- `ApplicationService`、`ApplicationComposition` 增加唯一 `ReviewService` owner；Tauri Review 命令、公共方法名/参数/返回 DTO 保持不变。
- Match Review Package、Postmatch、Analytics 未在 AT1 修改。

### 验证事实

- Windows AT1 hard gate run `31328023642`：Review 专项、Application Ports、完整 architecture、模型保护资产、rustfmt、Application check/tests、完整 frontend、完整 Rust、精确作用域与 clean tree 均通过。
- clean Review Core tree 已形成正式 Atomic commit `6059d7a79f34bb31a5b81b56a1c15af0a30db11a`。
- 首次 Public Platform CI run `31328591975` 在 `verify:architecture` 停止，明确错误为 Domain 类型与契约清单漂移；Windows Automated 未执行，因此 AT1 保持 `VERIFYING`。
- 根因：Review owner 拆分改变了 Application Rust 文件/调用面，但 `architecture/domain-type-inventory.json` 仍保存拆分前的 `rustUsageDigest`、扫描文件数量及外部调用者路径。已按既有生成器刷新清单；恢复 run `31348056577` 的 inventory、Review 专项、Application Ports、完整 architecture、保护资产、完整 frontend/Rust、scope 与 clean tree 均通过。
- 刷新后的正式提交 `586ad2196be7218e366cf1715fbbf69d87d91f01` 触发第二次 Public Platform CI run `31349803381`。Domain inventory、Domain 根出口、Application Ports、Database、Competition/Rules、Teams/Players、Lineups、Prediction、Research 门禁均已通过；随后 `scripts/verify-review-service.mjs` 在 Node 22 clean checkout 中因普通字符串被错误写成跨行文本而触发 `SyntaxError: Invalid or unexpected token`，Windows Automated 再次因前置架构门禁失败被跳过。
- 第二次失败定位为 Review 专项验证器自身语法缺陷，不是 Review Core 业务实现失败。修复仅将失败输出改为显式 `\n` 拼接，并在恢复门禁增加 `node --check scripts/verify-review-service.mjs`；不删除、不放宽任何验证断言。

- 第三次正式 Public Platform CI run `31350677129` / Windows Automated job `93340716563` 已整体 `SUCCESS`；architecture、Windows automated acceptance、validation evidence upload 与 post-job 均成功。artifact `9049246515`，大小 `14276530` 字节，SHA-256 `62f0bcfdb8f83ce0a715de58ee14f8a0a87b1f3192c938e841d65400b8ecb7ef`。正式修复提交 `bac0ca3c1192b8da919b409b06aa6e454ceca87e`。

### 未改变

- PostgreSQL SQL、migration、Schema、数据格式。
- Tauri DTO/命令签名与前端状态。
- 错误语义、日志等级和用户可观察业务行为。
- `crates/model-api/`、`crates/model-p4/`、`crates/model-p7/` 与模型保护资产。
- 生产依赖。

### AT1 关闭

- Review Core 6 个公开职责已完成迁移，两个正式 CI 暴露的验证缺口均已修复且未放宽门禁。
- AT1 正式关闭为 `DONE`。下一 Atomic Task：Match Review Package，状态 `READY`；Postmatch 与 Analytics 继续保持未修改。
