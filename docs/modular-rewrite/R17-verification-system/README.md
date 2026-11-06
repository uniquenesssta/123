# R17 验证覆盖与真实环境验收

用户本次授权“收尾09开始R17-01”。R16修复精确c1574990/CI42125458762及官方摘要/双包/startup已接受，关闭文档ad743482已推送；阶段分支rewrite/r17-verification-system，以ad743482干净基线开始。

| 主交付 | 状态 | 实际范围 |
| --- | --- | --- |
| R17-01 验证覆盖与执行入口收敛 | VERIFYING | 原01/02/03/04/05/07/08/09；[记录](R17-01-verification-coverage-and-entrypoints.md)、[17/40/16覆盖矩阵](R17-01-coverage-matrix.md)；98步前端/35架构/7项负向通过；自身CI/包待验 |
| R17-06 最终新库与Windows Full | BLOCKED / 未启动 | 原06/10；需01适用出口及用户授权、专用新库/有效文件/私有模型/API/系统凭据；20 PG target/40 ignored及16 BA真实义务不能据本批核销 |

本批未改UI、生产业务、Rust/Schema、依赖版本或PNG；Rust同树768/40 ignored证据复用，非本批重跑。R17阶段出口未完成，R18仍BLOCKED。下一项：收尾R17-01，然后R17-06。
