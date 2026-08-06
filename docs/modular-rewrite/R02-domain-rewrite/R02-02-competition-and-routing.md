# R02-02 Competition 与 Routing 实施记录

- 任务状态：`VERIFYING`
- 前置已验证提交：`43029b3ca5c6e61ea6ff81d74a1fb489d398ad48`
- 目标平台：Windows

## 1. 实际问题

R2-01 已确认 22 个 Competition 与 Routing 类型仍直接定义在 `crates/domain/src/lib.rs`。根文件同时承担比赛结构、规则包、绑定解析、路由决策以及其他领域类型，导致职责边界不清晰；任何后续领域修改都需要进入同一根文件。

## 2. 完成内容

- 新建 `crates/domain/src/competition/`，拆分 CompetitionKind、赛事目录、Profile、Season、Stage、Round 与 Rule Package 契约。
- 新建 `crates/domain/src/routing/`，拆分 ModelIdentity、RuleRouting、CompetitionBinding、ResolvedCompetitionContext 与 Route 决策契约。
- 从 `crates/domain/src/lib.rs` 删除 22 个原始定义，根文件仅声明业务模块并保留兼容 re-export。
- 新增公共模块路径契约测试，确认 22 个新模块路径与既有根级类型是同一 Rust 类型。
- 新增 `architecture/domain-migration-progress.json`，并扩展通用清单门禁：已完成任务的类型必须实际位于登记目标目录。
- 更新清单目标策略，使后续生成器识别已迁移的 Competition 与 Routing 子目录。
- 重新生成 `architecture/domain-type-inventory.json`。

## 3. 类型范围

Competition 共 14 个：

- `CompetitionKind`
- `CompetitionDraft`
- `CompetitionRecord`
- `CompetitionProfile`
- `SeasonDraft`
- `SeasonRecord`
- `SeasonTeamMembershipOption`
- `StageDraft`
- `StageRecord`
- `RoundDraft`
- `RoundRecord`
- `RuleSourceReference`
- `RulePackageDraft`
- `RulePackageSummary`

Routing 共 8 个：

- `ModelIdentity`
- `RuleRouting`
- `CompetitionBindingDraft`
- `CompetitionBindingSummary`
- `ResolvedCompetitionContext`
- `RouteRequest`
- `RouteSource`
- `RouteDecision`

## 4. 兼容性

以下内容保持不变：

- 根级 `football_domain::TypeName` 调用路径和类型身份；
- Serde 字段名、枚举 snake_case 线值、默认值与 optional 语义；
- PostgreSQL 映射、历史 JSON、公共命令、错误语义和日志等级；
- Application、Tauri DTO、前端行为、P4/P7 Schema 与模型保护边界；
- Cargo 与 npm 生产依赖。

`football_domain::competition::*` 与 `football_domain::routing::*` 为新增的业务语义路径。根级 glob 出口只为兼容保留，按计划在 R2-08 统一收敛，不在本节点提前改变调用面。

## 5. 实际验证

Windows 实施 workflow run `31087811267` 已实际通过：

- `cargo test --locked -p football-domain --test serde_contracts`：通过。
- `node scripts/generate-domain-type-inventory.mjs`：通过。
- `node scripts/verify-domain-type-inventory.mjs`：通过。
- `npm run verify:architecture`：通过。
- `node scripts/verify-protected-assets-deterministic.mjs`：通过。
- `npm run verify:frontend`：通过。
- `npm run verify:rust`：通过。
- 临时实施 workflow 与脚本在提交前删除：通过。

最终实施提交上的正式 `Public Platform CI` Windows Automated 尚待确认；完成前保持 `VERIFYING`，R2-03 保持 `BLOCKED`。

## 6. 未执行与延期项

- 真实 PostgreSQL 实跑保留到最终统一验收。
- Windows Full 交互验收和用户本机 Windows 10/11 实机验收保留到最终统一验收。
- 既有 moderate npm vulnerability 和 Vite 大 chunk 警告不属于本节点修改范围。

## 7. 回退

回退到前置提交 `43029b3ca5c6e61ea6ff81d74a1fb489d398ad48`；不保留双实现、复制文件或长期转发层。
