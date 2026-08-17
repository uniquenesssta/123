# R06-06 — Abilities 与 Dynamic Tags

## 状态

`VERIFYING`

## 进入基线

- R6-05：`DONE`。
- 唯一阶段分支：`rewrite/r6-entity-catalog-persistence`。
- 节点起点：`f48841a8a50f51cd71e09969d858b95e7d9ba9fa`。

## 已实施职责边界

- `adapters/catalog/abilities/dimensions/`：能力维度 query、typed Row 与 Domain mapper。
- `adapters/catalog/abilities/observations/`：能力观察输入策略、写 SQL、typed Row 与 Domain mapper；Player Detail 只保留读取 SQL并复用该 Row/mapper owner。
- `adapters/catalog/dynamic_tags/definitions/`：动态标签定义读取。
- `adapters/catalog/dynamic_tags/tags/`：动态标签输入策略、写入、单条读取、时点列表、typed Row 与 mapper。
- `adapters/catalog/dynamic_tags/contribution/`：比赛贡献计算编排与纯评分组件分责。
- 旧 `dynamic_tags.rs` 单文件 owner 已删除；`player_catalog.rs` 已退出 R6-06 Abilities 职责。

## 保持不变的契约

- `PlayerSignalPort` 的 Ability、Dynamic Tag 与 Match Contribution 方法签名及 Application adapter 调用语义不变。
- Domain DTO、Schema、0001–0046 migration、配置、错误文案、默认战术角色/位置映射、历史 P4/运行引用和模型保护资产未主动修改。
- 未新增生产依赖。

## 验证

- implementation gate run `32043516067` 已实际通过 R6-06 ownership、R6-03～R6-05 retained ownership、完整 architecture、模型保护、数据库基线、rustfmt、Persistence unit tests、Application check、R6-06 PostgreSQL contract 编译及 PostgreSQL 16 专用测试库真实执行。
- 阶段 hard-gate 恢复过程中，GitHub codeload `429/503` 曾在第三方 action 下载阶段 fail-fast；改为使用 runner 自带 `rustup` 后，Windows 回归进一步暴露历史 `verify-player-role-inheritance.mjs` 仍读取已删除的 `dynamic_tags.rs`。修复提交 `23a201d28389955747641c12717592b377687251` 仅将原角色继承断言跟随到 `adapters/catalog/dynamic_tags/contribution/{calculate,scoring}.rs` 权威 owner，没有删除、跳过或放宽契约。
- 最终阶段 hard gate run `32044944449` 整体 `SUCCESS`：Windows job `95430654653` 已通过 `npm run verify:frontend`、`cargo fmt --all -- --check`、workspace Clippy `-D warnings` 与 `cargo test --locked --workspace`；PostgreSQL/architecture job `95430654659` 已通过 R6 retained ownership、完整 architecture、模型保护/数据库基线，以及 R6-03、R6-04、R6-05、R6-06 PostgreSQL 16 contracts。
- 临时 hard-gate workflow 已从 clean tree `e11a86632d6287f5f4a54848717556cafab6883c` 清理；当前提交用于触发无临时 workflow 的 clean canonical Public Platform CI，节点继续保持 `VERIFYING`。

## 尚未执行

- 用户现有 PostgreSQL 数据真实写入/sample 验收与 Windows Full 人工交互验收，继续留到最终统一验收。

## 回退点

- `f48841a8a50f51cd71e09969d858b95e7d9ba9fa`。
