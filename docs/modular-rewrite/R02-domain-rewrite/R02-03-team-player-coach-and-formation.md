# R02-03 Team / Player / Coach / Formation 实施记录

- 任务状态：`VERIFYING`
- 前置已验证提交：`897a8bc0a43121908826eca41e0c1db3c2763889`
- 前置正式验收：workflow run `31088698579`、job `92574240109`
- 目标平台：Windows

## 1. 实际问题

R2-01 清单确认 72 个球队、球员、教练、阵型及其跨域引用类型仍直接定义在 `crates/domain/src/lib.rs`。根文件同时承担目录实体、列表查询、名称历史、归属周期、可用性、能力观察、阵型分布、删除预检和外部实体映射，职责和变化原因混杂。

## 2. 完成内容

- 新建 `team/`，按 catalog、listing、name、profile、detail、deletion、membership 拆分 17 个类型。
- 新建 `player/`，按 status、catalog、listing、name、position、membership、availability、ability、detail 拆分 21 个类型。
- 新建 `coach/`，按 catalog、listing、name、membership、detail 拆分 9 个类型。
- 新建 `formation/`，按 catalog、usage、resolution 拆分 8 个类型。
- 新建 `shared/`，按 ability、position、bulk archive、bulk delete、data provider、entity match、entity reference 拆分 17 个跨域契约。
- 从 Domain 根文件删除 72 个原始定义及其专属实现和默认值函数；根文件仅声明业务模块并保留兼容 re-export。
- 新增 72 个公共模块路径身份断言，确认新路径与既有根级路径指向同一 Rust 类型。
- 更新目标模块策略、迁移进度和可机器复算的 Domain 类型清单。

## 3. 类型范围

- Team：TeamDetail, TeamDraft, TeamForceDeletePreview, TeamForceDeleteRequest, TeamForceDeleteResult, TeamListItem, TeamListPage, TeamListQuery, TeamNameDraft, TeamNameRecord, TeamOption, TeamPlayerPeriodRecord, TeamProfileDraft, TeamProfileRecord, TeamRecentMatch, TeamRecord, TeamSquadPlayer
- Player：AvailabilityStatus, PlayerAbilityObservationDraft, PlayerAbilityObservationRecord, PlayerAbilityProfile, PlayerAvailabilityDraft, PlayerAvailabilityRecord, PlayerCatalogReferenceData, PlayerDetail, PlayerDraft, PlayerListItem, PlayerListPage, PlayerListQuery, PlayerNameDraft, PlayerNameRecord, PlayerPositionDraft, PlayerPositionRecord, PlayerRecord, PlayerStatus, PlayerTeamPeriodDraft, PlayerTeamPeriodRecord, PreferredFoot
- Coach：CoachDetail, CoachDraft, CoachListItem, CoachListQuery, CoachNameDraft, CoachNameRecord, CoachRecord, TeamCoachPeriodDraft, TeamCoachPeriodRecord
- Formation：FormationDistributionQuery, FormationRecord, FormationUsageDistributionDraft, FormationUsageDistributionRecord, FormationUsageEntryDraft, FormationUsageEntryRecord, FormationUsageListQuery, ResolvedFormationDistribution
- Shared：AbilityDimensionRecord, BulkArchiveFailedItem, BulkArchiveResult, BulkDeleteBlockedItem, BulkDeleteResult, DataProviderDraft, DataProviderRecord, EntityDeletionCheck, EntityMatchCandidate, EntityMatchRequest, EntityMatchResult, EntityReferenceCount, EntityReferenceQuery, EntityReferenceRecord, ExternalEntityIdDraft, ExternalEntityIdRecord, PositionReference

## 4. 兼容性

以下内容保持不变：

- 根级 `football_domain::TypeName` 路径和类型身份；
- Serde 字段名、枚举 snake_case 线值、默认值、optional 语义与历史 JSON；
- PostgreSQL 映射、SQL Row、数据库迁移、公共命令、错误语义和日志等级；
- Application、Tauri DTO、前端行为、P4/P7 Schema 与模型保护边界；
- Cargo 与 npm 生产依赖。

新增 `football_domain::team::*`、`player::*`、`coach::*`、`formation::*` 与 `shared::*` 业务语义路径。根级 glob 出口债务仍由 R2-08 统一退出。

## 5. 实际验证

Windows 实施 workflow run `31100515822` 已实际通过：

- `cargo test --locked -p football-domain --test serde_contracts`：通过。
- `node scripts/generate-domain-type-inventory.mjs`：通过。
- `node scripts/verify-domain-type-inventory.mjs`：通过。
- `npm run verify:architecture`：通过。
- `node scripts/verify-protected-assets-deterministic.mjs`：通过。
- `npm run verify:frontend`：通过。
- `npm run verify:rust`：通过。
- 临时实施 workflow、载荷与脚本在提交前删除：通过。

最终实施提交上的正式 `Public Platform CI` Windows Automated 尚待确认；完成前保持 `VERIFYING`，R2-04 保持 `BLOCKED`。

## 6. 未执行与延期项

- 真实 PostgreSQL 实跑保留到最终统一验收。
- Windows Full 交互验收和用户本机 Windows 10/11 实机验收保留到最终统一验收。
- 既有 moderate npm vulnerability 和 Vite 大 chunk 警告不属于本节点修改范围。

## 7. 回退

回退到前置提交 `897a8bc0a43121908826eca41e0c1db3c2763889`；不保留双实现、复制文件或长期转发层。
