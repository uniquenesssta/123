# R0-07 Rust 格式门禁修复：实施与验证记录

## 1. 基本信息

- 所属阶段：R00 基线冻结与可重复验收
- 任务状态：DONE
- 起始提交：`190939f71768d0c1bb349bdf67f288c6d0daebec`
- rustfmt 实施提交：`9e7be511ae2d97a0782fee1a2bea5e25d910d10d`
- 实施分支：`new-A`
- 完成日期：2026-08-05
- 临时验证 Draft PR：`#4`
- rustfmt 诊断 workflow run：`30961081516`
- rustfmt 应用 workflow run：`30961167749`
- 精确 Rust 验证 workflow run：`30961535208`

## 2. 目标

关闭 R0-05、R0-06.1 和 R0-06.2 保留的 Rust 格式门禁失败：

```text
cargo fmt --all -- --check
```

本节点只执行 Rust 官方 `rustfmt` 规范化，不修复 Clippy，不改业务逻辑、公共接口、依赖、数据库迁移、模型边界或用户可观察行为。

## 3. 现状与可复现行为

R0-05 Linux 基线中：

- Cargo.lock 与 locked metadata 通过；
- `cargo fmt --all -- --check` 失败；
- CI 因 fail-fast 未执行 Clippy 和 workspace tests。

R0-06.1 的 Windows Automated 已能越过前端阶段，但同样在既有格式门禁处退出。因此 R0-07 的唯一直接问题是工作区 Rust 源码未完全符合 Rust 1.88.0 所带 rustfmt 的输出。

## 4. 任务边界

### 允许变化

- 对 rustfmt 诊断确认的 `.rs` 文件应用 `cargo fmt --all`。
- 建立一次性诊断、精确文件白名单和验证证据。
- 更新根 `README.md`、R00 索引和本节点实施记录。

### 禁止变化

- 不修改函数语义、控制流、SQL、数据结构、序列化格式或错误语义。
- 不修改公共 Tauri command、DTO、配置默认值或环境变量。
- 不修改 `Cargo.toml`、`Cargo.lock`、`package.json`、`package-lock.json` 或工具链版本。
- 不修改数据库迁移。
- 不修改 `crates/model-api`、`crates/model-stub` 或模型保护清单。
- 不通过 `#[allow(...)]`、降低 `-D warnings`、跳过测试或放宽门禁掩盖 Clippy 问题。
- 不在本节点顺手修复 Clippy，避免跨 Atomic Task 混改。

## 5. 工作区状态与保护说明

当前执行环境未建立用户设备上的本地 Git 工作树，无法查看用户本机未提交或未跟踪文件。实施仅写入远端 `new-A`，未覆盖用户设备上的本地内容。

R0-07 起始提交与最终格式化差异经过精确路径白名单控制。rustfmt 诊断没有返回以下范围：

- `crates/model-api/**`
- `crates/model-stub/**`
- `architecture/protected-assets.json`
- 数据库迁移 SQL
- 依赖和锁文件

## 6. 诊断方式

临时 GitHub Actions 在 Ubuntu、Rust `1.88.0` 和 rustfmt 组件下执行：

```text
cargo fmt --all
git diff --name-only -- '*.rs'
git diff -- '*.rs'
cargo fmt --all -- --check
```

诊断结果：

- 需要格式化的 Rust 文件：42 个；
- 非 Rust 文件变化：0；
- 保护模型文件变化：0；
- rustfmt 执行后的格式复查：通过。

诊断 artifact：

- ID：`8912948607`
- 名称：`r00-07-rust-format-diagnostics`
- SHA-256：`833a1aaae8e2155b4e87c880a7a060f01ae7a7546d8b723a9c1dfda92fa1f00c`

## 7. 实际格式化文件

### application（11）

1. `crates/application/src/analytics.rs`
2. `crates/application/src/lib.rs`
3. `crates/application/src/match_review_package.rs`
4. `crates/application/src/model_shell/mod.rs`
5. `crates/application/src/p4_orchestration.rs`
6. `crates/application/src/player_catalog.rs`
7. `crates/application/src/postmatch.rs`
8. `crates/application/src/prediction.rs`
9. `crates/application/src/release_acceptance.rs`
10. `crates/application/src/rule_packages.rs`
11. `crates/application/src/spreadsheet.rs`

### domain（5）

1. `crates/domain/src/match_event.rs`
2. `crates/domain/src/match_review_package.rs`
3. `crates/domain/src/match_review_workflow.rs`
4. `crates/domain/src/release_acceptance.rs`
5. `crates/domain/src/team_package.rs`

### persistence-postgres（19）

1. `crates/persistence-postgres/src/analytics.rs`
2. `crates/persistence-postgres/src/connection.rs`
3. `crates/persistence-postgres/src/lineup_chain.rs`
4. `crates/persistence-postgres/src/match_exchange.rs`
5. `crates/persistence-postgres/src/match_prediction.rs`
6. `crates/persistence-postgres/src/match_review_package.rs`
7. `crates/persistence-postgres/src/model_runs.rs`
8. `crates/persistence-postgres/src/monthly_workbooks.rs`
9. `crates/persistence-postgres/src/name_search.rs`
10. `crates/persistence-postgres/src/parameter_lifecycle.rs`
11. `crates/persistence-postgres/src/player_catalog.rs`
12. `crates/persistence-postgres/src/postmatch.rs`
13. `crates/persistence-postgres/src/release_acceptance.rs`
14. `crates/persistence-postgres/src/review.rs`
15. `crates/persistence-postgres/src/role_resolution.rs`
16. `crates/persistence-postgres/src/spreadsheet_exchange.rs`
17. `crates/persistence-postgres/src/team_force_delete.rs`
18. `crates/persistence-postgres/src/team_lineup_presets.rs`
19. `crates/persistence-postgres/tests/postgres_integration.rs`

### spreadsheet-io（4）

1. `crates/spreadsheet-io/src/lib.rs`
2. `crates/spreadsheet-io/src/match_review_workbook.rs`
3. `crates/spreadsheet-io/src/monthly_workbook.rs`
4. `crates/spreadsheet-io/src/team_package.rs`

### Tauri 适配层（3）

1. `src-tauri/src/commands.rs`
2. `src-tauri/src/commands/catalog.rs`
3. `src-tauri/src/runtime_log.rs`

合计：42 个 `.rs` 文件。

## 8. 应用门禁

格式化写回 workflow 使用固定的 42 文件白名单，并在提交前执行：

```text
diff -u approved-rust-files.txt changed-rust-files.txt
git diff --check
cargo fmt --all -- --check
```

同时拒绝任何非 `.rs` 文件被 rustfmt 改动。实际结果：

- 精确文件集合与诊断白名单一致；
- `git diff --check` 通过；
- `cargo fmt --all -- --check` 通过；
- 格式化提交为 `9e7be511ae2d97a0782fee1a2bea5e25d910d10d`。

应用 artifact：

- ID：`8912978503`
- 名称：`r00-07-rust-format-apply`
- SHA-256：`3d812ee6e64e3988cd66e6c0f4f73c54c91888d61a3ec4f5b4245007ac61a321`

## 9. 最小验证

在现有正式 CI Rust job 中，格式化提交后执行：

```text
cargo fmt --all -- --check
```

结果：通过。CI 随后继续进入 Clippy，证明原始格式门禁不再阻断后续阶段。

## 10. 精确 `verify:rust` 阶段回归

在 Ubuntu、Node `22`、Rust `1.88.0`、rustfmt、Clippy 和完整 Tauri Linux 系统依赖环境中精确执行：

```text
npm ci
npm run verify:rust
```

`verify:rust` 实际命令链为：

```text
verify-cargo-lock
verify-cargo-lock-sync
prepare-cargo-target
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
```

结果：

- npm locked 依赖安装：通过；
- Cargo.lock 完整性：通过；
- Cargo.lock 语义同步：通过；
- Cargo target 准备：通过；
- `cargo fmt --all -- --check`：通过；
- Clippy：失败；
- `cargo test --locked --workspace`：因 Clippy fail-fast 未执行；
- `npm run verify:rust` 退出码：`101`。

精确验证 artifact：

- ID：`8913160029`
- 名称：`r00-07-exact-rust-verification`
- SHA-256：`47712408cb9fbd37088f42cab92e71565b0c982d5c0492a78fb6c4ef2e53ad49`

## 11. Clippy 暴露的下一层阻塞

本节点没有修改以下问题，仅将其作为 R0-08 的真实输入记录。

### `football-spreadsheet-io`：3 个错误

- `clippy::too_many_arguments`：1 个
  - `crates/spreadsheet-io/src/match_review_workbook.rs:1573`
- `clippy::explicit_auto_deref`：2 个
  - `crates/spreadsheet-io/src/team_package.rs:306`
  - `crates/spreadsheet-io/src/team_package.rs:1674`

### `football-persistence-postgres`：13 个错误

- `clippy::clone_on_copy`：6 个
  - `crates/persistence-postgres/src/postmatch.rs:108`
  - `crates/persistence-postgres/src/postmatch.rs:121`
  - `crates/persistence-postgres/src/postmatch.rs:183`
  - `crates/persistence-postgres/src/postmatch.rs:1405`
  - `crates/persistence-postgres/src/postmatch.rs:1459`
  - `crates/persistence-postgres/src/postmatch.rs:1460`
- `clippy::needless_borrows_for_generic_args`：5 个
  - `crates/persistence-postgres/src/postmatch.rs:168`
  - `crates/persistence-postgres/src/postmatch.rs:573`
  - `crates/persistence-postgres/src/postmatch.rs:574`
  - `crates/persistence-postgres/src/release_acceptance.rs:143`
  - `crates/persistence-postgres/src/release_acceptance.rs:144`
- `clippy::too_many_arguments`：1 个
  - `crates/persistence-postgres/src/postmatch.rs:583`
- `clippy::uninlined_format_args`：1 个
  - `crates/persistence-postgres/src/postmatch.rs:752`

合计：16 个 Clippy 错误。由于命令使用 `-D warnings`，任何一个错误都会阻止 workspace tests。

## 12. 前端回归状态

格式化后的正式 CI 同时运行前端 job：

- `npm ci`：通过；
- Linux `npm run verify:frontend`：仍失败于已知 Chromium/截图环境链；
- 该问题在 R0-05 已冻结，不由 Rust 格式化引入，也不在 R0-07 范围内。

Windows 完整 frontend 的既有通过证据仍由 R0-06.1 和 R0-06.2 保留。

## 13. 保持不变的行为

- 没有新增、删除或重命名公共函数、类型、模块、Tauri command 或 DTO。
- 没有修改 SQL、数据库迁移、持久化结构或数据契约。
- 没有修改依赖、锁文件、构建配置、默认值或环境变量。
- 没有修改模型公开边界和保护资产。
- 没有添加 Clippy allow、测试跳过、白名单或兼容层。
- 没有修改用户可观察业务行为。

## 14. 未执行项

- workspace tests 未执行，具体原因是精确 `verify:rust` 在 Clippy 的 16 个错误处 fail-fast。
- PostgreSQL 真实迁移幂等、不可变触发器和 18 个忽略型集成测试继续按用户要求留到最终统一验证。
- Windows Full 和用户本机 Windows 10/11 人工验收未执行。
- Linux Chromium 启动问题未在本节点修复。

## 15. 删除清单

节点收尾删除以下临时文件：

- `.github/workflows/r00-rust-format-diagnostics.yml`
- `docs/modular-rewrite/R00-baseline/.r0-07-trigger`
- `docs/modular-rewrite/R00-baseline/.r0-07-task-state`
- `docs/modular-rewrite/R00-baseline/.r0-07-diagnostics-note`
- `docs/modular-rewrite/R00-baseline/.r0-07-pr-ready`

临时 Draft PR `#4` 关闭且不合并。

## 16. 风险与回退

### 剩余风险

- Clippy 的 16 个错误尚未修复，完整 `verify:rust` 仍不通过。
- workspace tests 尚未运行，测试层风险仍未排除。
- 大型 rustfmt 差异会增加审阅噪声，但变更由官方 rustfmt 生成，并以精确文件白名单、`git diff --check` 和格式复查约束。

### 回退点

回退到 R0-07 起始提交：

```text
190939f71768d0c1bb349bdf67f288c6d0daebec
```

或者仅回退 rustfmt 提交：

```text
9e7be511ae2d97a0782fee1a2bea5e25d910d10d
```

回退不涉及数据库、依赖、迁移或公共接口。

## 17. 完成结论

- 42 个 Rust 文件已使用 Rust 1.88.0 rustfmt 统一格式。
- `cargo fmt --all -- --check` 已真实通过。
- R0-07 原始格式门禁阻塞已关闭。
- 完整 `verify:rust` 已真实执行，并在 Clippy 的 16 个现有错误处退出 101。
- workspace tests 未被描述为通过。
- R0-07 状态为 DONE。
- R00 仍为 BLOCKED。
- 下一唯一 READY 任务为 `R0-08 Rust Clippy 门禁修复`，不得提前进入 R1。
