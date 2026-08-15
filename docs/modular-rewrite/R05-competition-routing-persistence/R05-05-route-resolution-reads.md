# R05-05 — Route Resolution Reads

## 状态

`VERIFYING`

R5-05 的 Route Resolution / Competition Context PostgreSQL persistence owner 已完成切换，并已通过旧/new owner 同一 PostgreSQL 16 contract、R5 ownership、official Domain inventory 与节点 hard gate。当前等待 clean PR canonical CI、固定 HEAD merge、merged stage CI 与最终 closeout canonical CI；这些完成前 R5-06 保持 `BLOCKED`。

## 基线与分支

- R5-04 最终绿色基线：`5a159e6ebcaeee0d25d833cd48042ffc9915a713`。
- 该 HEAD 的 canonical Public Platform CI run `31866449663` / Windows job `94968219120` 为 `SUCCESS`，artifact `9242455827`，SHA-256 `d34740b34981adb496fbbe6ce96186dfa4f85df0c9b1d7d49101563892a71f11`。
- R5-05 实施分支：`agent/r5-05-route-resolution`，从上述 HEAD 精确建立。

## 实际问题与边界

R5-05 开工前，Route Resolution persistence 分散在两个旧 owner：

- `crates/persistence-postgres/src/routing.rs`：`resolve_route` 与动态 Route Row -> Domain mapper；同文件还承载 R5-06 model registration。
- `crates/persistence-postgres/src/competitions.rs`：`resolve_competition_context` 与 `ensure_scope_id`；R5-02 后该文件已不再承担其他生产职责。

Application `RuleRoutingPort` 同时暴露 context read 与 route read，`preview_route` 先解析 `ResolvedCompetitionContext`，再以解析后的 scope 调用 `resolve_route`。因此 R5-05 必须一起迁移 context read 与 route read，避免形成跨 legacy/new owner 的半拆分。R5-06 model registration 不在本节点范围内，继续保留在 `routing.rs`。

## 实际实现

新的唯一 Route Resolution persistence owner：

```text
crates/persistence-postgres/src/adapters/competition/route_resolution/
├─ mod.rs
├─ context/
│  ├─ mod.rs
│  ├─ record_row.rs
│  ├─ record_mapper.rs
│  ├─ resolve_context.rs
│  ├─ season_context.rs
│  ├─ stage_context.rs
│  └─ validate_scope.rs
└─ route/
   ├─ mod.rs
   ├─ record_row.rs
   ├─ record_mapper.rs
   ├─ resolve_route.rs
   ├─ explicit_rule_package.rs
   └─ binding_candidate.rs
```

- context coordinator 只负责编排，不含 SQL；Stage/Season context 各自拥有单一 SELECT，typed `sqlx::FromRow` Row 与 Domain Mapper 分离。
- scope mismatch 校验独立为 `validate_scope.rs`，保持原 `InvalidState` 文案语义。
- route coordinator 只负责编排；explicit package 与 automatic binding candidate 各自拥有单一 SELECT。
- automatic candidate 保持 Stage > Season > Competition > CompetitionKind Default 的 specificity 排序，以及 priority / created_at / id 排序、有效期过滤与 model family/exact model 过滤。
- `RouteDecision.reason`、`RouteSource`、package/model/parameter/profile/output 字段映射保持原语义。
- CompetitionKind 继续复用共享 `parse_competition_kind`，没有在 SQL 或新 mapper 中复制领域解析规则。
- `crates/persistence-postgres/src/competitions.rs` 已删除；`lib.rs` 同步删除 legacy module 注册，不保留转发壳。
- `routing.rs` 已删除 R5-05 `resolve_route` / dynamic mapper，只保留 R5-06 `register_model` / `register_model_in_tx` 及其 model/version/parameter persistence 链。
- 未创建 `model_run_identity/`，R5-06 未提前实施。

## 契约冻结与验证

新增永久 contract：`crates/persistence-postgres/tests/route_resolution_repository_contract.rs`。

覆盖：fallback/competition/season/stage context、层级不一致与停用赛事错误；explicit package；Stage -> Season -> Competition -> Kind Default 回退；binding 有效期；family/exact model 过滤；`RouteNotFound`；以及完整 `RouteDecision.reason` 与返回字段。

实际验证记录：

- 首次 old-owner run `31868912136` / job `94974325107`：仅因新增 contract canonical rustfmt 差异 fail-fast，PostgreSQL contract 未执行，生产源码未修改。
- format run `31868960850` / job `94974446617`：`SUCCESS`，仅格式化新增 contract。
- old-owner PostgreSQL 16 contract run `31869022228` / job `94974599427`：`SUCCESS`。
- owner-switch run `31869137395` / job `94974903779`：`SUCCESS`；Persistence/Application compile 与同一 new-owner PostgreSQL contract 均通过，生产切换提交 `3490d93032245e0f420f90ca099b87e420817dd3`。
- ownership gate 前三轮分别发现 R5-05 verifier 过严字面量、R5-03 stale route-owner 断言、R5-02 legacy path 假设；均 fail-fast 且未提交错误 verifier。第四轮 run `31869503602` / job `94975820076` 为 `SUCCESS`，R5-01～R5-05 ownership 全链通过，R5-06 仍被锁定。
- official Domain inventory run `31869537207` / job `94975903916` 为 `SUCCESS`；仅使用 `generate-domain-type-inventory.mjs` + drift verifier。
- 第一轮 stage hard gate run `31869573253` / job `94975993679`：R5 ownership 通过后，R4-03 mapping verifier 因读取已正式删除的 `competitions.rs` 触发 ENOENT，后续步骤按 fail-fast 跳过；未记为通过。
- R4-03 mapping gate 已推进到实际 owner：保留全部基础 scalar/JSON/optional/UUID/time mapping 约束，同时验证 R5-01/R5-03/R5-04/R5-05 继续复用共享 CompetitionKind parser，并验证 R5-05 context mapper 为 typed Row -> Domain、无 SQL/PgRow。
- 第二轮 stage hard gate run `31869727870` / job `94976380579` 为 `SUCCESS`：R5-01～R5-05 ownership、完整 `verify:architecture`、public/protected model boundary、rustfmt、Persistence/Application check、Persistence tests、Application tests 与同一 PostgreSQL 16 Route Resolution contract 全部通过。

## PR #30 canonical 验证与修复

- PR #30 首轮 fixed clean HEAD `bc93518f8436ee5175b4fea4af3bd32441c04e1d` 的 Public Platform CI run `31870124821` / Windows job `94977359038` 为 `FAILURE`。architecture 已通过，失败发生在 Windows automated acceptance 的 workspace Clippy `-D warnings`：typed `RouteRow.competition_kind` 自 owner switch 后从未被 Domain mapper 或 source 判定消费，因此触发 `dead_code`。
- 修复没有使用 `#[allow]`、没有放宽门禁，也没有改变 route algorithm/result。仅删除 `RouteRow.competition_kind` 未使用字段，以及 explicit package / binding candidate SELECT 中对应的冗余返回列；automatic candidate 的 `b.competition_kind = $4` WHERE 过滤、Stage > Season > Competition > CompetitionKind Default specificity、priority/created_at/id 排序、有效期与模型过滤均保持不变。
- 该字段删除改变 PostgreSQL mapping usage digest 后，full architecture 正确发现 inventory drift；只使用官方 `node scripts/generate-domain-type-inventory.mjs` 刷新。run `31870479459` / job `94978235169` 为 `SUCCESS`。
- Ubuntu 专项修复 gate run `31870518531` 在 R5 ownership、full architecture 与 canonical rustfmt 通过后，workspace Clippy 因 runner 缺少系统 `glib-2.0` / `glib-2.0.pc` 停止；PostgreSQL contract 按 fail-fast 未执行。该环境阻塞未记为源码或测试通过，也未通过安装新生产依赖绕过。
- 同一修复源码随后由 canonical Public Platform CI run `31870519567` / Windows job `94978336960` 完成验证并为 `SUCCESS`；artifact `9243579284`，大小 `13929568` 字节，SHA-256 `a728b4806b18d034a34a79142dd926734e5461ece23f04ddf577039c7a1304e4`。该 run 证明 Windows architecture 与完整 automated acceptance（含 workspace Clippy/tests/Tauri release/runtime）通过。
- 上述成功 run 对修复验证有效，但当时分支仍含临时修复 workflow；临时 workflow 已在随后提交中全部删除。R5-05 仍保持 `VERIFYING`，最终 clean HEAD 必须再次通过 canonical Public Platform CI 才允许 fixed-head merge。

## 兼容性与未变范围

未修改 Domain 公共类型、Application `RuleRoutingPort`、Application/Tauri 调用面、Schema、0001–0046 migration、配置、错误/日志等级、前端行为、route algorithm/result、model identity、Cargo manifests/Cargo.lock、生产依赖与模型保护资产。

## 当前未执行项与限制

- 既有 `postgres_integration.rs` 18 个 broad PostgreSQL ignored tests 未在 R5-05 节点 hard gate 执行。
- 未执行 destructive database reset，未触碰用户数据库；R5-05 contract 只使用 GitHub Actions 临时 PostgreSQL 16。
- canonical Windows frontend/workspace Clippy/workspace tests/Tauri release/runtime 仍需由 clean PR 的 `Public Platform CI` 对固定 clean HEAD 执行。

## 下一状态门禁

R5-05 当前为 `VERIFYING`；只有 clean PR canonical CI、固定 HEAD merge、merged stage CI 与最终 closeout canonical CI 全部成功后，才可改为 `DONE` 并将 R5-06 改为 `READY`。
