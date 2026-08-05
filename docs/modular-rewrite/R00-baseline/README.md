# R00 基线冻结与可重复验收：执行记录索引

## 阶段范围

冻结模块化重写开始前的远端提交、分支、保护资产、公共命令、数据库、前后端与 Windows 验收基线。R00 不实施业务重写，不移动业务源码，不修改模型保护区。

## 当前基线

- 基线分支：`main`
- 起始基线提交：`db79995873460688c15abb3497bf1c61b73ffb18`
- R0-02 起始提交：`beb00b6f723d75703e9235967c4f49f985d41c4e`
- R0-03 起始提交：`83a7d0bddd83894b4f551725c40b50705470e731`
- R0-04 起始提交：`8cddf452e01770a042b671ed8c11bc29afd2f1b1`
- R0-05 起始提交：`92976da2372287a66f91d31da6d4f090734dee6c`
- R0-05 CI 测试提交：`94f86842db227c5153d4334ad5176159a840e429`
- R0-06 起始提交：`06732375bf3c3f40d94b5fdf2ff7609e07698f5a`
- R0-06 Windows 证据提交：`2001154dff611e6326f2182d8e4a0b8aa35ca98b`
- R0-06.1 起始提交：`b165bf5c4a658b63edb698e40b49efebf567b334`
- R0-06.1 Windows 验证提交：`278f1093b90c5c13aed7108535329bd1dc528441`
- R0-06.2 起始提交：`7a6b542abbe366daa829616658f8234351c1daad`
- R0-06.2 Windows 验证提交：`4d9eb14d83e661b09e6403b0b8677e9b229dfb58`
- R0-07 起始提交：`190939f71768d0c1bb349bdf67f288c6d0daebec`
- R0-07 rustfmt 实施提交：`9e7be511ae2d97a0782fee1a2bea5e25d910d10d`
- R0-08 起始提交：`b163597e497736dbbf73e50e4abf097b43c899fe`
- R0-08 Clippy 实施提交：`919d62a2eaf95ade5ba1efa18924a9d578ef3f63`
- R0-09 workspace tests 实施提交：`50daa258af8ac8e09f8e4f5f428249fe670f2dd2`
- 实施分支：`new-A`
- 已完成节点：`R0-01`、`R0-02`、`R0-03`、`R0-04（数据库实跑延期）`、`R0-05`、`R0-06`、`R0-06.1`、`R0-06.2`、`R0-07`、`R0-08`、`R0-09`、`R00-stage-completion`

## 保护资产

- `architecture/protected-assets.json` 冻结 18 个公开模型边界及校验文件。
- 聚合 SHA-256：`d2263a5ff09c8cf633a42b7bb35fffe3d42fb18648db4d12691817f51015c85c`。
- 真实 P4/P7 源码、参数、Profile、专用 Schema、fixture、Golden Master 和私有研究资产继续禁止进入公开仓库。

## 命令契约

- `architecture/command-contract.json` 冻结 171 个公共命令、注册顺序和 15 个 Rust 命令模块。
- 命令集合 SHA-256：`dfc477a8b0bec95735fdad006349fcea0f367b4ebc85aede1b63a128a55de669`。
- 命令定义映射 SHA-256：`eb908c4ae31ec8d49e65a1b336c1105cb550e755fcdbae40f4943f99433cc9f5`。

## 数据库基线

- `architecture/database-baseline.json` 冻结 `0001`–`0046` 共 46 个连续迁移。
- 迁移聚合 SHA-256：`d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93`。
- 数据库公共接口、迁移集合和不可变约束继续以 `main` 基线保持不变。
- 用户于 2026-08-04 明确要求将 PostgreSQL 迁移幂等和集成测试推迟到最终统一验证；未执行项不得描述为通过。

## 前端与 Rust 基线

- R0-05 workflow run `30910130867`：`npm ci` 通过，存在 1 个 moderate npm audit 警告。
- Linux `npm run verify:frontend` 曾失败于 Chromium 未开放调试端口；用户于 2026-08-05 明确取消 Linux 目标平台，该结果仅保留为历史记录，不再属于交付或阶段门禁。
- Rust 工具链、Tauri Linux 依赖、公开模型边界、Cargo.lock 与 locked metadata 通过。
- R0-05 的 `cargo fmt --all -- --check` 失败已由 R0-07 关闭。
- R0-07 冻结的 16 个 Clippy 错误已由 R0-08 关闭。
- R0-08 冻结的两个 workspace tests 失败已由 R0-09 关闭。
- R0-09 中精确 `npm run verify:rust` 已以退出码 0 完成。

## Windows 基线

- R0-06 证据：workflow run `30912862564`、job `92003616257`、artifact `8894874465`。
- Windows release 构建与 RuntimeOnly startup 通过；startup report 为 PASS，7 条记录、3 个完成操作、0 个无效行、0 个运行时错误。
- R0-06 精确 Automated 暴露目录联接下依赖安装假通过及 `.cmd` 子进程调用问题。
- Windows Full 未执行：数据库按用户要求最后验证，人工 GUI 链不适合 hosted runner。

## R0-06.1 Windows Node 调用链

- 新增 `scripts/process/execution-context.mjs`，统一直接执行身份的规范物理路径判断。
- 新增 `scripts/process/node-package-cli.mjs`，通过当前 Node 执行本地包 JavaScript CLI，不再直接启动 `.cmd` 包装器。
- 新增 `scripts/verify-node-process-compatibility.mjs`，覆盖目录联接、参数传递、非零退出码和缺失入口拒绝。
- `scripts/ensure-node-dependencies.mjs` 已修复联接路径下未执行却返回成功的问题。
- `scripts/verify-frontend.mjs` 已通过 Node 直接执行 TypeScript 与 Vite CLI。
- Windows workflow run `30919764753` 中，完整 `npm run verify:frontend` 通过；精确 Automated 前端阶段通过并继续进入 Rust 阶段。
- Automated 总体仍在当时既有 `cargo fmt --check` 处退出 1，不描述为总体通过。
- 证据 artifact `8896665715`，SHA-256：`95e567f917082670390860d2671699a59a7c65018cbbb875f5f37ea05288a16d`。
- Draft PR #2 已关闭且未合并；临时 workflow 已删除。

## R0-06.2 Windows 路径契约

- 新增 `scripts/windows/acceptance-paths.psm1`，独立负责验收日志、Cargo target 与 release EXE 路径解析。
- 新增 `scripts/verify-windows-path-contract.mjs`，并纳入完整前端验证。
- `scripts/windows-acceptance.ps1` 已声明并支持根入口既有 `LogDirectory` 参数。
- 验收证据目录与应用 runtime 日志目录已分离；runtime JSONL 继续固定在项目根目录 `logs`。
- Cargo target 优先读取 `.cargo/target-location.json`，缺失时按 `.cargo/config.toml` 回退到源码上级 `.cargo-target`。
- Windows workflow run `30922384735` 中，完整 frontend、release 构建和 RuntimeOnly runner 均实际通过；startup report 为 PASS。
- release EXE 从 `D:\a\123\123\.cargo-target\release\football-match-model-desktop.exe` 成功启动。
- 证据 artifact `8898312587`，SHA-256：`d6ed06066aab354686f86938ec7c55f2c1f740e11a37e42a6a1b5edbbd53df63`。
- 临时 workflow 的最终 job 状态为 failure，原因是所有产品验证通过后的一条中文完整日志行辅助精确匹配未命中；证据文件已复核 target、启动路径和 PASS 报告一致，未将 workflow 总体描述为通过。
- Draft PR #3 已关闭且未合并；临时 workflow 已删除。

## R0-07 Rust 格式门禁

- Rust 1.88.0 rustfmt 诊断确认 42 个 `.rs` 文件需要规范化。
- 文件范围只涉及 application、domain、persistence-postgres、spreadsheet-io 和 Tauri 适配层；未触碰 `model-api`、`model-stub`、迁移、依赖或锁文件。
- 写回过程使用固定 42 文件白名单，拒绝任何非 Rust 文件变化。
- rustfmt 实施提交：`9e7be511ae2d97a0782fee1a2bea5e25d910d10d`。
- `git diff --check` 与 `cargo fmt --all -- --check` 均通过。
- 精确 workflow run `30961535208` 执行完整 `npm run verify:rust`：Cargo.lock 与格式门禁通过，Clippy 在 16 个错误处以退出码 101 失败，workspace tests 因 fail-fast 未执行。
- Clippy 错误分布：`football-spreadsheet-io` 3 个，`football-persistence-postgres` 13 个。
- 诊断 artifact `8912948607`，SHA-256：`833a1aaae8e2155b4e87c880a7a060f01ae7a7546d8b723a9c1dfda92fa1f00c`。
- 应用 artifact `8912978503`，SHA-256：`3d812ee6e64e3988cd66e6c0f4f73c54c91888d61a3ec4f5b4245007ac61a321`。
- 精确验证 artifact `8913160029`，SHA-256：`47712408cb9fbd37088f42cab92e71565b0c982d5c0492a78fb6c4ef2e53ad49`。
- 未添加 Clippy allow、门禁放宽、测试跳过或生产依赖。

## R0-08 Rust Clippy 门禁

- 起始错误来自 R0-07 冻结的 16 个 Clippy `-D warnings` 错误；修复过程中继续处理全 workspace all-targets 深层暴露的同类 lint。
- 最终只修改 11 个 Rust 文件，并通过固定路径白名单保护范围。
- 新增 `EventScoreContext`、`EvaluationSamplePartition`、`MatchEventFixture` 和 `RuntimeLogEntry` 四个私有参数对象；未改变公共 API、DTO、持久化或日志 JSON 格式。
- 其余修复包括 Copy 值无效 clone、无效借用、无效 `drop(state)`、内联格式参数、私有死代码清理和测试模块 `json!` 导入。
- 实施提交：`919d62a2eaf95ade5ba1efa18924a9d578ef3f63`。
- workflow run `30965687503`：11 文件白名单、rustfmt、diff check、持久化库测试 73/73、Tauri runtime log 7/7 和全 workspace Clippy 均通过。
- 表格库完整测试为 11/12；`team_package::tests::physical_worksheet_row_number_survives_blank_rows` 因模板缺少固定字段 `action` 失败。该失败已保留，未被描述为通过。
- 精确 workflow run `30966064295`：Cargo.lock、rustfmt 和 Clippy 连续通过，workspace tests 在 `football-application` 的 `openai_research::tests::built_in_gateway_is_strict_and_has_no_secret` 处失败并退出 `101`。
- 精确测试在 `football-application` 处 fail-fast，后续 crate 测试未完整执行，因此后续失败集合尚未全部枚举。
- 最终应用 artifact `8914718704`，SHA-256：`b8e75726c6ad53bdb4932ceb0bb3d35ff4554f306179178e6a566187723c6c60`。
- 精确验证 artifact `8914844238`，SHA-256：`05ee24344468b9613bf18c139ff7d3aabecb92e005f93afb1f9037ed7f21cede`。
- 未添加 `#[allow]`、降低 `-D warnings`、删除失败测试、修改依赖或触碰模型保护区。

## R0-09 Rust workspace tests 门禁

- 诊断 workflow run `30967005489` 使用 `--no-fail-fast` 完整枚举失败，确认只有 OpenAI 提示词断言和表格空白行物理行号两个失败。
- OpenAI 测试改为大小写无关地验证既有禁止概率计算契约；表格测试补齐真实模板已有的分组标题行，避免稀疏工作簿读取范围裁剪导致键列错位。
- 最终只修改两个测试位置，共新增 2 行；未改变生产逻辑、公共接口、数据结构、配置、依赖、迁移或模型保护边界。
- 实施提交：`50daa258af8ac8e09f8e4f5f428249fe670f2dd2`。
- 最终 workflow run `30967448070`、job `92184375540` 全部通过。
- 两个专项测试均 1/1 通过；`cargo test --locked --workspace --no-fail-fast` 为 185 通过、0 失败、18 忽略。
- 精确 `npm run verify:rust` 退出码 0，Cargo.lock、rustfmt、Clippy 与 workspace tests 全部通过。
- 最终 artifact `8915431192`，SHA-256：`a745bded71179bb6542d3a06b5c65f61cdf48845b8f88193dc7ef0ac5c8fcadc`。
- 18 个忽略项为按用户要求延期的 PostgreSQL 集成测试；未描述为已执行。
- Draft PR #6 已关闭且未合并；临时 workflow 和 patch 已删除。

## 任务状态表

| 任务 ID | 任务名称 | 状态 | 实施记录 | 最小验证 | 阶段回归 |
|---|---|---|---|---|---|
| R0-01 | 工作区状态与基线提交确认 | DONE | [记录](./R00-01-工作区状态与基线提交确认.md) | 远端基线与分支起点核对 | 已纳入出口判断 |
| R0-02 | 模型保护资产指纹 | DONE | [记录](./R00-02-模型保护资产指纹.md) | 指纹、篡改和禁止资产负向验证 | 已纳入出口判断 |
| R0-03 | 命令契约冻结 | DONE | [记录](./R00-03-命令契约冻结.md) | 171 命令三方扫描及负向验证 | 已纳入出口判断 |
| R0-04 | 数据库基线 | DONE | [记录](./R00-04-数据库基线.md) | 46 个迁移静态门禁；真实 PostgreSQL 延期 | 出口缺口保留 |
| R0-05 | 前端与 Rust 基线 | DONE | [记录](./R00-05-前端与Rust基线.md) | Linux 前端和 Rust 现有失败已冻结 | 出口缺口保留 |
| R0-06 | Windows 基线 | DONE | [记录](./R00-06-Windows基线.md) | release 与 RuntimeOnly 通过；精确 Automated 失败已冻结 | 出口缺口保留 |
| R0-06.1 | Windows Node 调用链修复 | DONE | [记录](./R00-06.1-Windows-Node调用链修复.md) | Windows frontend 通过；Automated 到达 Rust 阶段 | Node 调用缺口关闭 |
| R0-06.2 | Windows 路径契约修复 | DONE | [记录](./R00-06.2-Windows-路径契约修复.md) | Windows frontend、release、RuntimeOnly 通过 | Windows 路径缺口关闭 |
| R0-07 | Rust 格式门禁修复 | DONE | [记录](./R00-07-Rust格式门禁修复.md) | 42 文件 rustfmt；格式检查通过 | Clippy 16 错误已冻结 |
| R0-08 | Rust Clippy 门禁修复 | DONE | [记录](./R00-08-Rust-Clippy门禁修复.md) | 全 workspace all-targets Clippy 通过 | workspace tests 失败已冻结 |
| R0-09 | Rust workspace tests 门禁修复 | DONE | [记录](./R00-09-Rust-workspace-tests门禁修复.md) | 两专项测试、185 个 workspace 测试与精确 verify:rust 通过 | Rust 完整门禁关闭 |
| R0-10 | Linux Chromium 前端验收门禁修复 | NOT_APPLICABLE | 不创建 | 用户取消 Linux 目标平台 | 不纳入阶段出口 |

## 本阶段累计变化

- 新增模型保护、命令契约和数据库基线清单及只读校验器。
- 新增 R0-01 至 R0-09 节点记录。
- 新增职责单一的 Windows Node 执行模块、Windows 路径模块及相应专项验证器。
- R0-07 对 42 个 Rust 文件执行纯 rustfmt 规范化。
- R0-08 对 11 个 Rust 文件实施私有参数收束、Copy/借用修复、私有无效代码清理和测试编译补全。
- R0-09 只修正两个测试契约，共新增 2 行，完整 Rust 门禁现已通过。
- R00 按 Windows-only 范围完成收口；Linux 门禁取消，PostgreSQL 实跑、Windows Full 和用户本机实机验收统一转入最终验收。
- R0-08 与 R0-09 未修改依赖、锁文件、公共接口、迁移 SQL、模型实现、配置默认值或用户可观察数据格式。

## 最终验收保留项

- PostgreSQL 迁移幂等、不可变触发器和 18 个集成测试尚未执行，按用户要求统一保留到最终验收。
- Windows Full 和用户本机 Windows 10/11 实机验收尚未执行，统一保留到最终验收。
- 1 个 moderate npm vulnerability 和 Vite 大 chunk 警告未处理，不阻塞 R1，但最终交付必须重新评估。
- 用户设备上的未提交或未跟踪文件不可见；远端操作未覆盖这些本地内容。

## 非目标平台记录

- Linux Chromium 截图进程曾未开放调试端口。
- 用户于 2026-08-05 明确取消 Linux 目标平台，因此不再修复、不纳入交付、不作为 R00 或后续阶段门禁。

## 阶段门禁状态

**DONE。** Windows Node/frontend、路径契约、Rust 格式、Clippy 和 workspace tests 门禁均已关闭。Linux 已从目标范围移除；PostgreSQL 实跑、Windows Full 和用户本机实机验收已明确延期到最终验收，不阻塞模块化重写。

阶段完成记录：[`R00-stage-completion.md`](./R00-stage-completion.md)。

## 下一 READY 任务

`R1-01 模块边界契约` 是唯一 READY 任务。R1 阶段索引见 `../R01-architecture-composition/README.md`。
