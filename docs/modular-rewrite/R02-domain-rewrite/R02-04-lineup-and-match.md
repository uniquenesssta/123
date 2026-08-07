# R02-04 Lineup 与 Match 实施记录

- 任务状态：`VERIFYING`
- 前置已验证提交：`3256454e9d76b7b442a83f25964363256257cbcc`
- 实施触发提交：`42d24de5358d0e53aa8cae567eacf1179a236097`
- 目标平台：Windows

## 1. 实际问题

R2-01 清单确认 Lineup 16 个类型和 Match 3 个类型仍分散在 Domain 根文件及 `lineup_chain.rs`。比赛目录、比赛状态、阵容快照、球员条目、阵容预设和模型输入链路混在根级定义与单文件中，职责和变化原因不一致。

## 2. 完成内容

- 新建 `lineup/`，按 kind、player、snapshot、preset、chain 拆分 16 个类型及正式快照类型常量。
- 新建 `match_record/`，按 status、catalog 拆分 3 个类型。
- 从 Domain 根文件删除 17 个原始定义、默认值函数和专属实现；删除旧 `lineup_chain.rs`，不保留转发文件或双实现。
- 根文件仅声明业务模块并保留兼容 re-export。
- 新增 19 个公共模块路径身份断言，确认新路径与既有根级路径指向同一 Rust 类型。
- 更新目标模块策略、迁移进度和可机器复算的 Domain 类型清单。
- 根目录 `README.md` 继续作为唯一项目变更记录；R2-04 不迁移或替换 README 记录职责。
- 旧根 Domain 聚合检查已扩展覆盖新的 Lineup 与 Match 职责模块。

## 3. 类型范围

- Match：MatchStatus, MatchDraft, MatchRecord
- Lineup：LineupType, LineupPlayerDraft, LineupPlayerRecord, LineupDraft, LineupPairDraft, LineupPairRecord, LineupRecord, LineupHistoryRemovalResult, TeamLineupPresetMemberDraft, TeamLineupPresetDraft, TeamLineupPresetMemberRecord, TeamLineupPresetRecord, TeamLineupPresetApplicationPreview, MatchLineupTeamChain, MatchLineupChain, TeamMatchLineupHistoryItem

## 4. 兼容性

以下内容保持不变：

- 根级 `football_domain::TypeName` 路径和类型身份；
- `FORMAL_LINEUP_SNAPSHOT_TYPES` 根级导出和值；
- Serde 字段名、枚举 snake_case 线值、默认值、optional 语义与历史 JSON；
- PostgreSQL 映射、SQL Row、数据库迁移、公共命令、错误语义和日志等级；
- Application、Tauri DTO、前端行为、P4/P7 Schema 与模型保护边界；
- Cargo 与 npm 生产依赖。

新增 `football_domain::lineup::*` 与 `football_domain::match_record::*` 业务语义路径。根级 glob 出口债务仍由 R2-08 统一退出。

## 5. 实际验证

实施 workflow run `31151412918` 执行：

- `cargo test --locked -p football-domain --test serde_contracts`
- `node scripts/generate-domain-type-inventory.mjs`
- `node scripts/verify-domain-type-inventory.mjs`
- `npm run verify:architecture`
- `node scripts/verify-protected-assets-deterministic.mjs`
- `npm run verify:frontend`
- `npm run verify:rust`

正式 `Public Platform CI` Windows Automated 通过前保持 `VERIFYING`，R2-05 保持 `BLOCKED`。

## 6. 未执行与延期项

- 真实 PostgreSQL 实跑保留到最终统一验收。
- Windows Full 交互验收和用户本机 Windows 10/11 实机验收保留到最终统一验收。
- 既有 moderate npm vulnerability 和 Vite 大 chunk 警告不属于本节点修改范围。

## 7. 回退

回退到前置提交 `3256454e9d76b7b442a83f25964363256257cbcc`；不保留双实现、复制文件或长期转发层。
