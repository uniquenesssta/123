# R7-08 Lineup Chain / History：节点完成记录

状态：`DONE`（唯一职责迁移、历史删除共同锁和 Windows Automated 完成；真实 PG/XLSX/Windows Full 仍最终封包新库待验）。

## 实际范围

阵容链、时点窗口和门禁分别迁入 `adapters/lineups/chain/`；历史列表、明细映射与删除/恢复分别迁入 `adapters/lineups/history/`。删除与创建共同按比赛父锁 → 阵容版本锁排序，并使用 READ COMMITTED，引用检查、恢复与审计共同提交。旧根 `lineup_chain.rs` 删除，`player_catalog.rs` 仅保留引用数据聚合；全部直接调用方及 Application 分派同步。14 个原规则、读取及映射函数体经规范化对比保持。

原内联测试补窗口边界和严格映射；原 PostgreSQL 用例补截止等时/一微秒边界、confirmed 优先级、UUID 稳定排序、隐藏历史明细、列表上限，以及历史删除与创建共同父锁并发。五项结构破坏探针均被现有门禁拒绝，没有新增测试目标或工作流。

## 精确 Windows 证据

- 提交 `3c8f7f0230207f7352b7f806b8a1e92c4ebc81e5`，树 `a3739565108a0ab4549c35818583c73062455265`。
- [Public Platform CI run 36801488090](https://github.com/uniquenesssta/123/actions/runs/36801488090)，Windows job `110176585645` 全步骤 SUCCESS；2026-10-01 09:55:49（北京时间）更新为 completed/success。
- 架构、前端契约/类型/17 个截图视口/生产构建、Windows fmt/Clippy/workspace tests、release/MSI/NSIS 打包与启动验收通过。Persistence inline **107 项**、Application **53 项**全通过；启动日志 **7 条记录、3 个完成操作**通过。
- artifact `11135923979`，14,019,398 字节；SHA-256 `46efa8a67291ef70e325869158d9d2c20ebebb277a63092b545edb3a6677fb50`。
- 18 项 broad PostgreSQL 测试仍 ignored，仅完成编译；其他数据库 contract 的 ignored 亦不算执行通过。历史四项失败及账本没有实跑关闭。

首轮提交 `3362f456` 的 run `36764662888` 在旧阵型 verifier 路径失败，后续 Rust/截图/打包/启动当时未执行。修订现有阵型、历史比分和阵容链验证器的职责路径及 Rustfmt 空白匹配，保留检查并经四项缺失契约探针拒绝验证；上述成功提交覆盖该修订。

## 下一项与回退

用户已授权收尾并启动 R7-09 Team Lineup Presets，新代码需要自己的精确 Windows CI。真实数据库、有效 XLSX 与 Windows Full 沿用最终封包新库流程。进入/回退基线 `3ff21cba4ab01bc02108e8545f0ed133e25de687`；受控回退须同步职责、共同锁、调用方、清单/指纹及现有验证器，不单独放宽门禁。公开 API/DTO、算法/参数/保护资产、生产依赖/锁文件与 0001–0046 迁移保持冻结。
