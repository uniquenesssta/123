# R17 验证覆盖与真实环境验收

用户已授权收尾R17-01并开始R17-06，沿用rewrite/r17-verification-system。01精确2101ce14/CI42252996208及官方摘要、双包、startup已正式接受。

| 主交付 | 状态 | 实际范围 |
| --- | --- | --- |
| R17-01 验证覆盖与执行入口收敛 | DONE | 原01/02/03/04/05/07/08/09；[记录](R17-01-verification-coverage-and-entrypoints.md)、[覆盖矩阵](R17-01-coverage-matrix.md)；98步前端/35架构/7负向及自身CI/包/startup通过 |
| R17-06 最终新库与Windows Full | BLOCKED，部分实证完成 | [真实环境记录](R17-06-real-environment-and-full.md)：20 PG target/40原ignored实跑通过；实际XLSX经应用提交、重建连接读回通过；修复空摘要/缺失惯用脚和示例路径，原模型/Schema/UI不变；e651b423/CI42384401592及官方双包/startup已接受；月度真实更新/重连及首档案修复已补验，Windows验收目录/客户端定位修复；补修ebef8903 CI短路径误判，22路径/10失败传播通过；新增通用宿主装配，23私有回归及真实P4/P7服务层落库/重连通过，777普通Rust/98前端/35架构/6原生启动通过，新提交CI单列 |

临时PostgreSQL18.4在项目tmp独立55437端口，日常库未改；用户明确暂缓真实API/系统凭据。已找回用户本机完整模型与固定资产，真实P4/P7已通过当前ApplicationService；公开默认桌面仍明确Unavailable，私有桌面装配尚待验。正式同对象链及Windows Full未执行，R17阶段未完成，R18仍BLOCKED。下一步仍为补齐R17-06真实环境义务。
