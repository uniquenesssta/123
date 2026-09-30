# R7-05 关键 Application 用例验证：节点完成记录

状态：`DONE`（所选行为测试与 Windows Automated 完成；PG/有效 XLSX/Windows Full 单独最终封包新库待验）。

## 实际范围

沿用现有 Prediction/P4 orchestration tests 与 Exchange export 原 use case 内单测，新增 20 个行为测试：Prediction 6 项、P4 orchestration 11 项、Exchange 3 项。调用真实 Application 编排，核对顺序、参数、正式/影子历史写入差异、失败后的后续写入阻止、取消/失败终态只读、已有快照恢复和重复执行、研究入队/登记失败的相同幂等键重试，以及损坏输入/导出失败的副作用边界。

共享 fake 仅在 `cfg(test)` 中以 crate 可见方式复用，未选中 Port 方法立即 panic；模型 fake 只验证编排，真实公开 provider 缺席仍断言错误且不保存成功历史。生产逻辑、公开 Port/DTO、依赖/锁文件、0001–0046 迁移、模型保护资产与工作流保持。没有新增测试文件、target、框架、runner 或数据库专项；原 Domain inventory 仅更新测试使用摘要，365 类型与声明摘要不变。

## 精确 Windows 证据

- 提交：`02e56badb508f448afa498bc9cb227065f346a6d`；树：`18d2002e9ce0366da13420ee4e702e0634594c24`。
- [Public Platform CI run 36712015030](https://github.com/uniquenesssta/123/actions/runs/36712015030)，Windows job `109875821076` 所有步骤 SUCCESS；北京时间 2026-09-30 20:29:40 完成。
- Application 实际执行 53 项：53 passed、0 failed、0 ignored，包含本节点全部 20 个新增测试。Windows Rust fmt/workspace all-targets Clippy（拒绝 warnings）/workspace tests、架构、前端静态契约/类型/17 个截图视口/生产构建、release/MSI/NSIS 打包与启动烟测通过。
- Runtime 日志验收 PASS：7 条记录、3 个完成操作。artifact `11095489222`，14,012,999 字节；SHA-256 `ee3f46ec56bad1bac386b8447187eda52774e070ca7bb049768a67d831a43b27`。
- PostgreSQL contracts 仍 ignored，仅编译通过，不计数据库实跑 PASS。

## 覆盖边界、下一项与回退

不宣称覆盖完整研究 gateway、完整真实模型冻结成功、运行中的并发取消、PostgreSQL exactly-once 或有效 XLSX 往返；fake 队列仅证明 Application 传递稳定幂等键。未覆盖流程沿用现有后续和最终验收入口。PG/XLSX/Windows Full 继续“最终封包新库待验”。

用户确认 CI 通过并授权启动 R7-06；R7-07 及以后继续 BLOCKED。R7-06 的新增代码与夹具不能继承本记录的 Windows PASS。

进入基线：`9bdb84923da714843e03383db1872aecd30250a2`。回退时受控 revert 本节点测试与使用清单并重跑受影响门禁，保留 R7-04 的精确完成证据。
