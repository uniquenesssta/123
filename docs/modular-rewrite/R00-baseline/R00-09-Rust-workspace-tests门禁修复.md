# R0-09 Rust workspace tests 门禁修复

## 任务目标

关闭 R0-08 后仍阻断 `npm run verify:rust` 的 workspace tests 失败，不改变生产行为、公共接口、数据结构、配置、依赖、迁移或模型保护边界。

## 起始状态

R0-09 先使用 `cargo test --locked --workspace --no-fail-fast` 完整枚举失败集合。诊断 workflow run `30967005489`、job `92183036236`、artifact `8915175849`，artifact SHA-256 为 `1003613a04ff826626719b22edc49d9b528c157d35762042625d537941130057`。诊断命令退出码为 `101`，确认只有以下两个失败：

1. `openai_research::tests::built_in_gateway_is_strict_and_has_no_secret`：测试以大小写敏感方式检查提示词中的禁止概率计算语句，而版本化提示词以句首大写 `Do not calculate probabilities` 表达同一契约。
2. `team_package::tests::physical_worksheet_row_number_survives_blank_rows`：测试构造的稀疏工作簿没有写入第 0 行前置列，Calamine 读取范围从实际首个非空列开始裁剪，导致固定字段键列错位并报“缺少固定字段 action”。真实模板会先写入分组标题行，不存在该构造缺口。

## 实施内容

实施提交：`50daa258af8ac8e09f8e4f5f428249fe670f2dd2`。

- `crates/application/src/openai_research.rs`：仅在测试断言前将提示词转换为 ASCII 小写，使断言验证语义契约而不是句首大小写。
- `crates/spreadsheet-io/src/team_package.rs`：仅在空白行物理行号测试中写入与真实模板一致的分组标题行，保持前置列范围并继续验证空白行后的物理行号。
- 最终代码变化为 2 个文件、2 行新增；没有修改生产逻辑。
- 临时诊断 workflow 与临时 patch 已在实施提交中删除，没有遗留临时验证资产。
- Draft PR #6 已关闭且未合并。

## 验证结果

最终 workflow run `30967448070`、job `92184375540` 全部步骤通过；证据 artifact `8915431192`，SHA-256 为 `a745bded71179bb6542d3a06b5c65f61cdf48845b8f88193dc7ef0ac5c8fcadc`。

- OpenAI 提示词专项测试：1/1 通过。
- 表格空白行物理行号专项测试：1/1 通过。
- `cargo test --locked --workspace --no-fail-fast`：185 个测试通过、0 个失败、18 个忽略。
- 18 个忽略项为既有 PostgreSQL 集成测试，继续按用户要求留到最终统一验证，不描述为已执行。
- `npm run verify:rust`：退出码 0；Cargo.lock、rustfmt、workspace all-targets Clippy `-D warnings` 与 workspace tests 全部通过。
- `git diff --check` 和两文件白名单检查通过。

## 兼容性与影响范围

- 公共 API、命令契约、DTO、持久化结构、迁移、配置默认值、日志格式和用户可观察行为均未改变。
- 未新增或升级依赖，未放宽 lint/test 门禁，未跳过或删除失败测试，未添加 `allow`。
- 模型保护文件与私有模型资产未触碰。

## 节点结论

R0-09 状态：**DONE**。

Rust 完整验证门禁已关闭。R00 仍不能完成，剩余硬缺口为 Linux Chromium 前端验收、最终 PostgreSQL 实跑、Windows Full 与用户本机 Windows 10/11 实机验收。下一唯一 READY 任务为 `R0-10 Linux Chromium 前端验收门禁修复`；不得进入 R1。