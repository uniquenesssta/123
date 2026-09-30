# R7-03 普通删除与历史引用保护：节点完成记录

状态：`DONE`（代码及 Windows Automated 完成；真实数据库/XLSX/Full 单独待验）。

## 实际范围与兼容性

Player/Team 普通删除显式使用 READ COMMITTED 事务，在取得实体 FOR UPDATE 锁后通过同一连接重新统计全部既有受保护引用，再进行外部 ID/实体删除及成功审计。最终裁决与 UI 预检共用原引用 owner 和归档提示；Player 原缺失的事务复检已补齐，Team 原比赛/复盘两项局部复检替换为完整复检。16 项 Team、13 项 Player 引用保持，未新增平行关系清单或改写历史 FK。

公开命令、DTO、Port、bulk 返回形态、missing-entity 错误、正常资料级联、force-delete 的完整名称确认/事务/墓碑保持。竞争窗口中 Team 原比赛/复盘专用错误统一采用既有共享归档提示。0001～0046 migrations、模型保护资产、依赖和锁文件保持。

现有 permanent-delete contract 保留原断言，并加入通过实际锁等待观察的 Player dynamic tag / Team lineup preset 并发提交、回滚、原实体/历史/外部 ID 保持、审计与重复删除检查。现有删除 verifier 覆盖隔离级别→排他锁→完整复检→变更顺序及共享裁决，未新增 runner、test target、workflow 或数据库设施。

## 文件变化

- 新增：本完成记录；R7-02 完成记录随本节点首次提交补齐，属于前项收尾。
- 修改：`deletion/delete_write.rs`、`deletion/preflight/{check,references,mod}.rs`、现有 `entity_permanent_delete_repository_contract.rs`、删除/球队球员管理两个现有 verifier、Domain inventory，以及根 README、R7 任务书/阶段索引、TESTING 文档。
- 移动或重命名：无。
- 删除文件：无；Team 写入内的两项重复局部检查由共享完整检查替代。

## Windows 验证证据

- 精确提交：`073d1557b9fb63edbdff59b851a2897f514ba8f2`；树：`2ab15eaeed34b20ddc8483c42e322151694f7cb0`。
- [Public Platform CI run 36695374298](https://github.com/uniquenesssta/123/actions/runs/36695374298)，Windows job `109821951512` 全部 SUCCESS，北京时间 2026-09-30 17:47 完成。
- 架构、前端静态契约/类型/截图/生产构建、Rust fmt/workspace all-targets Clippy（拒绝 warnings）/tests、Windows release 构建及 MSI/NSIS 两种打包通过。客户端启动与状态载入通过，运行日志验收 PASS：7 条记录、3 个完成操作。
- `safe_delete_rejects_non_test_database_before_connecting` 实际通过。`safe_permanent_delete_contract_is_preserved` 和 `safe_delete_rechecks_concurrent_history_after_parent_lock` 均 ignored，仅 Windows 编译通过，不计为真实数据库测试通过。
- artifact `11089310554`，14,015,005 字节；SHA-256 `f277160d6b6827fd50c2c1ebdc8203c3ab6651a35dd840ff4e3c027062990483`。

首轮 `e657a79` 的 run `36693857568` 在球队/球员管理旧断言失败：脚本仍在写入文件查比赛/复盘 SQL，未适配共享引用 owner。修订原脚本后实际检查事务调用、共享计数/裁决/拒绝及 Team 比赛/复盘 SQL，保持保护强度；四项临时破坏均被拒绝。首轮未执行的后续步骤已由上述成功提交补齐，失败历史保留于阶段索引。

本次收尾只补文档与状态，业务源码、测试、验证器、清单、配置和工作流均与已通过提交一致，沿用该 Windows 证据。

## 延期、下一项与回退

真实 PG 删除/并发断言、已有相关 deletion/archive/force-delete contracts、真实 XLSX 和 Windows Full 均为“最终封包新库待验”，使用已有 contract 与验收方式，不创建专项入口。节点 DONE 不代替最终封包验收。

下一项 R7-04 架构清单、验证器与执行记录对齐进入 READY、尚未实施；R7-05 及以后 BLOCKED。B3～B7/A8、历史四项数据库失败和账本问题仍由对应后续节点处理。

进入基线：`1e4da04b6a6c892dcca4e0499e963d5f39b9746c`。需要回退时受控 revert R7-03 实施/修订提交并同步状态、重跑受影响门禁；不将事务外预检作为普通删除长期最终依据。
