# R7-01 Match Catalog：节点完成记录

状态：`DONE`（代码及 Windows Automated 门禁完成；数据库/XLSX/Full 单独待验）。

## 实际范围与兼容性

完成 Match Catalog 向 `adapters/matches/catalog/` 的唯一 owner 切换及累计审计 A1～A7 修复：Application 显式读取调用、Domain 使用清单、历史验证器路径、现有聚合入口、契约测试安全与断言、scope/自动赛季/比赛共享事务。详细修订见阶段 README 的 2026-09-30 修复表。

正常公开命令/DTO、external key、scope、排序/limit、删除与审计契约保持；生产依赖、0001～0046 migrations 与模型保护资产未修改。旧临时迁移脚本已删除。

## Windows 验证证据

- 精确代码提交：`c826dd32e3ecb84dfc732ef60c3fd3aaf4153fdf`；树：`99039a6babc2941da091d92fe36c05eaae5de053`。
- [Public Platform CI run 36678914535](https://github.com/uniquenesssta/123/actions/runs/36678914535)，Windows job `109769864719`：`SUCCESS`，北京时间 2026-09-30 14:53 完成。
- 架构检查、前端契约/构建、Rust fmt/workspace Clippy/tests、Windows release 打包与启动日志烟测通过；Cargo.lock 同步检查通过。
- 非数据库 guard 单测实际通过；需 PG 的 `match_catalog_contract_is_preserved` 被标记 ignored，不计为通过。
- 证据 artifact：`11081433081`，`windows-automated-delivery-evidence-c826dd32e3ecb84dfc732ef60c3fd3aaf4153fdf`。

## 延期与剩余问题

`match_catalog_repository_contract` 真实 PG、既有 broad PG、真实 XLSX 与 Windows Full 均为“最终封包新库待验”。历史四项数据库失败和 A8 交给 R7-06；B1～B7 按当前 R7-02～06 顺序处理。现有 `delete_match` 清空 `model.runs.match_id` 与 migration 0041 输入身份触发器的冲突仍须确认，禁止放开历史触发器掩盖问题。

R7-01 的节点 DONE 不代表最终数据库/封包验收通过。仅开放 R7-02。

## 回退

整改进入远端基线：`421206717c5d8d36ce1f5c61795f8dac2bbc8c9f`；需要回退时受控 revert `c826dd3` 并重跑受影响检查，不重建旧脚本或双实现。整体迁移前基线见阶段审计记录。
