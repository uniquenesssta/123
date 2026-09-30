# R7-06 历史数据库失败与账本问题收口：节点完成记录

状态：`DONE`（修订与 Windows Automated 完成；四项历史数据库失败及账本仍“已修改、最终封包新库待验”，尚未实跑关闭）。

## 实际范围与责任路径

- 原 `postgres_integration.rs` 的 chain 测试先断言 10 名首发拒绝无遗留，再保存合法 11 人但缺阵型的模型不合格版本，在历史时点排除 actual/未来记录；pair 夹具使 T-6h 窗口真实开启并保留双方原子性；events fixture 完整序列化实际 MatchResultRecord，保留修订历史/统计。
- P4 时间精度 owner 为 `p4_records.rs::validate_snapshot_references`/`validate_snapshot_evidence`：SQLx 0.8.6 Chrono DateTime 以整数微秒落库，数据库绑定 timestamptz 精确比较研究 cutoff、开球及 published/effective；首建快照 RETURNING 实际 cutoff/frozen_at，与读回和重试一致。不使用宽容差或手写时间截断，原 payload 指纹保留纳秒身份。
- 原 P4 测试补亚微秒时间、真实 1 微秒差异/未来证据拒绝、首建/重试时间一致、不同载荷拒绝及四条概率链，保留 31 字段、更新/删除不可变断言。概率数据仅是公开 persistence fixture，不实施模型算法。
- `match_exchange.rs::commit_match_lineup_import` 的原 UPDATE 补 `ended_previous_count` 与对应绑定，保持业务/行/计数/审计同事务。原 pair 测试复用真实 preview/commit，模拟末行外键失效整批回滚，恢复后 12 插入/1 旧版本结束与账本/审计一致，成功批次重复提交拒绝且无额外效果。
- 原 Match Lineup Chain/Prediction Service verifier 扩充计数与精度门禁；三项临时破坏探针均拒绝并恢复。数据库基线只同步 integration 合法变更的 runtime-source blob；46 条迁移及聚合、18 项 ignored 集合冻结。Domain inventory 仅更新实际使用面，365 声明不变。

没有新增 test target、runner、workflow、数据库设施或持续回归体系；公开 Port/DTO、生产依赖/锁文件、模型资产与算法保持。完整实施表及责任链见 [阶段 README](README.md#r7-06-当前验证状态)。

## 精确 Windows 证据

- 提交：`4496399bd520b324360826d613113779c416c41e`；树：`6f7153a9d3f4561d0cc4d430bf98c9dec1cc81f9`。
- [Public Platform CI run 36734194083](https://github.com/uniquenesssta/123/actions/runs/36734194083)，Windows job `109951439572` 全步骤 SUCCESS；北京时间 2026-09-30 23:34:21 完成，2026-10-01 收尾核实。
- Windows fmt/workspace all-targets Clippy（拒绝 warnings）/workspace tests、架构、前端静态契约/类型/17 个截图视口/生产构建、release/MSI/NSIS 打包及启动通过。Application 53 项、Persistence inline 99 项实际全通过，包含新增亚微秒指纹身份测试。
- Runtime 日志验收 PASS：7 条记录、3 个完成操作。artifact `11107777886`，14,010,281 字节；SHA-256 `d420308239934efd7d231af31991422d6bc41f6ec07995233134550650025b75`。
- broad PostgreSQL 仍 0 passed、0 failed、18 ignored；其他真实 PG contracts 仍 ignored，仅编译通过，不能记为数据库 PASS 或历史失败关闭。

## 延期、下一项与回退

四项历史失败和 ended_previous 事务场景仍需最终 Windows 新库实跑确认，沿用原 `run_database_baseline.mjs`/相关 contracts/真实 XLSX/Windows Full，封包前清零失败和待验。不要求每节点新建数据库。

用户确认通过并启动 R7-07 Lineup Pair Transaction；后续 R7-10/R7-14 迁移继承本次计数/事务修复。R7-07 新实现与测试不得继承本节点的 Windows PASS。

进入/回退基线：`02e56badb508f448afa498bc9cb227065f346a6d`。受控 revert 本节点时同步 integration runtime-source 指纹及 Domain 使用清单，重跑受影响既有门禁，不独立放宽验证器。
