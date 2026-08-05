# R0-08 Rust Clippy 门禁修复：实施与验证记录

## 1. 基本信息

- 所属阶段：R00 基线冻结与可重复验收
- 任务状态：DONE
- 起始提交：`b163597e497736dbbf73e50e4abf097b43c899fe`
- 实施提交：`919d62a2eaf95ade5ba1efa18924a9d578ef3f63`
- 实施分支：`new-A`
- 完成日期：2026-08-05
- 临时验证 Draft PR：`#5`
- 初始诊断 workflow run：`30962189062`
- 第二层诊断 workflow run：`30963301958`
- 最终应用 workflow run：`30965687503`
- 精确 Rust 验证 workflow run：`30966064295`

## 2. 目标

关闭 R0-07 精确验证冻结的 Rust Clippy 硬门禁：

```text
cargo clippy --locked --workspace --all-targets -- -D warnings
```

本节点只修复 Clippy 明确报告的问题及为到达该门禁必须通过的测试编译缺口，不修复 workspace tests 的业务断言失败，不修改依赖、锁文件、数据库迁移、模型保护边界、公共命令、公共 DTO 或用户可观察数据格式。

## 3. 起始状态

R0-07 已关闭 `cargo fmt --all -- --check`，完整 `npm run verify:rust` 随后在 Clippy 处以退出码 `101` 结束。

初始 16 个错误分布：

- `football-spreadsheet-io`：3 个；
- `football-persistence-postgres`：13 个。

主要类型包括：

- `clone_on_copy`；
- `needless_borrow` / `explicit_auto_deref`；
- `too_many_arguments`；
- `uninlined_format_args`。

由于命令链 fail-fast，R0-07 时 workspace tests 尚未执行。

## 4. 影响分析

### 4.1 直接相关入口

- 表格复盘事件比分一致性校验；
- 球队资料包工作表写入；
- 赛后结算和评估样本持久化；
- 发布验收持久化；
- 参数生命周期分析；
- 预测就绪度私有检查构造；
- 发布验收私有检查构造；
- 表格预检错误文本格式化；
- PostgreSQL 集成测试 fixture 构造；
- Tauri 球队强制删除命令适配；
- runtime JSONL 日志私有写入路径。

### 4.2 必须保持不变

- 公共 Tauri command 名称、参数和返回值；
- 公共 Rust trait、DTO、序列化字段及错误语义；
- SQL、表结构和迁移集合；
- runtime 日志 JSON 字段、去重窗口和脱敏行为；
- 模型 ID、路由、Stub 和保护资产；
- 球队资料、赛后结算、预测就绪度和发布验收的用户可观察业务行为。

## 5. 实施内容

最终实施提交只修改以下 11 个 Rust 文件：

1. `crates/application/src/analytics.rs`
2. `crates/application/src/prediction.rs`
3. `crates/application/src/release_acceptance.rs`
4. `crates/application/src/spreadsheet.rs`
5. `crates/persistence-postgres/src/postmatch.rs`
6. `crates/persistence-postgres/src/release_acceptance.rs`
7. `crates/persistence-postgres/tests/postgres_integration.rs`
8. `crates/spreadsheet-io/src/match_review_workbook.rs`
9. `crates/spreadsheet-io/src/team_package.rs`
10. `src-tauri/src/commands/catalog.rs`
11. `src-tauri/src/runtime_log.rs`

### 5.1 私有参数对象

为避免通过 `#[allow(clippy::too_many_arguments)]` 掩盖问题，新增职责局部、未公开的参数对象：

- `EventScoreContext`：聚合事件比分一致性校验所需比赛上下文；
- `EvaluationSamplePartition<'a>`：聚合赛后评估样本分区身份；
- `MatchEventFixture<'a>`：聚合 PostgreSQL 集成测试事件 fixture；
- `RuntimeLogEntry<'a>`：聚合 runtime 日志内部写入字段。

这些结构均未进入公共 API，也未改变序列化或持久化格式。

### 5.2 Copy 与借用等价修复

- 移除 `DateTime`、`Option<DateTime>` 等 Copy 值上的无效 `clone()`；
- 移除 SQLx bind 的无效额外借用；
- 使用自动解引用替代显式 `*value`；
- 删除两个对 `tauri::State` Copy 值无效的 `drop(state)`；
- 简化可省略的显式生命周期。

### 5.3 私有 helper 参数收束

- 预测就绪度私有 helper 将 `code` 和 `label` 收束为元组；
- 发布验收私有 helper 将 `category`、`code` 和 `title` 收束为元组；
- 所有调用点同步更新，公共返回结构与检查代码保持不变。

### 5.4 其他 lint 修复

- 内联 `format!` 命名参数；
- 使用 `Option::is_none_or` 表达空值或空字符串条件；
- 删除未读取的私有 `training_end` 字段和未调用的私有 `lifecycle_window_json`；
- 在球队资料包测试模块局部导入 `serde_json::json`，修复 `--all-targets` 测试编译缺口。

## 6. 范围保护

实施过程使用固定 11 文件白名单，并在 runner 上执行：

```text
git apply --check
cargo fmt --all
git diff --check
git diff --name-only
cargo clippy --locked --workspace --all-targets -- -D warnings
```

未修改：

- `Cargo.toml`、`Cargo.lock`；
- `package.json`、`package-lock.json`；
- 数据库迁移；
- `crates/model-api/**`、`crates/model-stub/**`；
- 模型保护清单和命令契约；
- 前端源码和样式；
- 配置默认值和环境变量。

没有新增生产依赖、Clippy allow、测试跳过配置、白名单或兼容层。

## 7. 验证结果

### 7.1 最终应用验证

Workflow run：`30965687503`

结果：

- 原始 patch 应用：通过；
- patch SHA-256：`38ff57dae4c57546f884199a8349cbdb5b59533e192800d0f2a2b072e6974674`；
- 11 文件差异白名单：通过；
- `cargo fmt --all`：通过；
- `git diff --check`：通过；
- `football-persistence-postgres` 库测试：73/73 通过；
- `football-spreadsheet-io` 完整库测试：11/12 通过，1 个既有业务断言失败；
- 排除该已确认失败用例后的表格测试：11/11 通过；
- Tauri runtime log 专项测试：7/7 通过；
- `cargo clippy --locked --workspace --all-targets -- -D warnings`：通过；
- 实施提交：`919d62a2eaf95ade5ba1efa18924a9d578ef3f63`。

表格完整测试失败用例：

```text
team_package::tests::physical_worksheet_row_number_survives_blank_rows
```

失败事实：测试模板中的 `球员与评分` 工作表缺少固定字段 `action`。该失败未被删除、修改、跳过为全局默认或描述为通过；仅在替代验证中单独排除，以确认 R0-08 修改未破坏其余 11 个表格测试。

### 7.2 精确 `npm run verify:rust`

Workflow run：`30966064295`

测试提交：`9732bfc7e59ec32a6e5724ab78a026ff57a5b5c1`

精确命令：

```text
npm run verify:rust
```

结果：

- Cargo.lock 内容完整性：通过；
- Cargo.toml 与 Cargo.lock 语义同步：通过；
- Cargo target 准备：通过；
- `cargo fmt --all -- --check`：通过；
- `cargo clippy --locked --workspace --all-targets -- -D warnings`：通过；
- workspace tests：已实际开始执行；
- `football-analysis-package`：1/1 通过；
- `football-analytics-engine`：1/1 通过；
- `football-application`：25/26 通过，1 个失败；
- 命令退出码：`101`。

首个 fail-fast 阻塞：

```text
openai_research::tests::built_in_gateway_is_strict_and_has_no_secret
```

失败断言：

```text
built_in_research_prompt().content.contains("do not calculate probabilities")
```

由于 workspace tests 在 `football-application` 处退出，后续 crate 测试未在该精确命令中继续执行。因此不得将完整 `npm run verify:rust` 描述为通过，也不能断言后续只剩一个测试失败。

## 8. 证据

- 初始诊断 artifact：`8913342180`
  - SHA-256：`a088c91cb0f38aa9e72cc72563494a4504a34a739afd9d9d3b5fd19907f99d43`
- 第二层诊断 artifact：`8913738596`
  - SHA-256：`494b2e2463f93849b6d7e9299df22ab5d0b09ecc60f522febd4d2da71a149c32`
- 最终应用 artifact：`8914718704`
  - SHA-256：`b8e75726c6ad53bdb4932ceb0bb3d35ff4554f306179178e6a566187723c6c60`
- 精确 Rust 验证 artifact：`8914844238`
  - SHA-256：`05ee24344468b9613bf18c139ff7d3aabecb92e005f93afb1f9037ed7f21cede`

## 9. 未执行与限制

- PostgreSQL 被忽略的真实数据库集成测试继续按用户要求推迟到最终统一验证；
- Windows Full 和用户本机 Windows 10/11 GUI 验收未在本节点执行；
- Linux Chromium 前端截图基线未在本节点处理；
- workspace tests 未通过，且 fail-fast 后的全部后续测试集合尚未完整枚举；
- 用户设备上的本地未提交和未跟踪文件对当前远端执行环境不可见。

## 10. 结论

R0-08 的目标 Clippy 门禁已经真实关闭：

```text
cargo clippy --locked --workspace --all-targets -- -D warnings
```

R0-08 状态为 **DONE**。

R00 阶段仍为 **BLOCKED**，因为 workspace tests、Linux Chromium、PostgreSQL 实跑、Windows Full 和用户本机实机验收尚未关闭。

下一唯一 READY 任务：

```text
R0-09 Rust workspace tests 门禁修复
```

R0-09 至少需要处理并验证：

1. `openai_research::tests::built_in_gateway_is_strict_and_has_no_secret`；
2. `team_package::tests::physical_worksheet_row_number_survives_blank_rows`；
3. 修复前述 fail-fast 后重新执行完整 workspace tests，以枚举是否还有更深层失败。
