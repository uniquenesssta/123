# R7-02 外部 ID 身份保护：节点完成记录

状态：`DONE`（代码及 Windows Automated 完成；真实数据库/XLSX/Full 单独待验）。

## 实际范围与兼容性

公开添加和工作簿提交共用 `references/external_ids/write.rs` 的原子写入 owner，唯一键冲突只允许同实体合并 metadata，跨实体返回明确冲突且原绑定不变。工作簿沿用原批次事务，失败回滚此前业务行、ID、行状态、计数和成功审计。没有新增注册表、自动改绑开关或双实现。

现有 references contract 已补重试、并发争用/metadata 合并、provider/type 隔离、预检后的直写竞争和整批回滚断言；既有 R6-08 verifier 已覆盖共享 owner 与身份条件，Domain inventory 由原生成器同步。公开 DTO/Port/Tauri、正常 metadata 合并、批次结果、依赖和锁文件、0001～0046 migrations、模型保护资产保持。已有错误数据未自动纠正。

## Windows 验证证据

- 精确提交：`1e4da04b6a6c892dcca4e0499e963d5f39b9746c`；树：`300d87cf6e5473c026cd2c0d9171d054ba50fef0`。
- [Public Platform CI run 36686343584](https://github.com/uniquenesssta/123/actions/runs/36686343584)，Windows job `109792974772` 全部 `SUCCESS`，北京时间 2026-09-30 16:21 完成。
- 架构、前端构建、Rust fmt/workspace Clippy/tests、Windows release 打包及启动烟测通过。数据库名称 guard 单测实际通过。
- `entity_matching_and_references_contract_is_preserved` 与 `external_id_identity_and_import_atomicity_are_preserved` 均 ignored，仅编译通过，不计为真实数据库测试通过。
- artifact `11084048803`，14,009,636 字节；SHA-256 `a83d003d50dc6f9ce6c2ae85bc7e773c6df27ed69cf7c53cf6fa898e86ec5167`。

## 延期、下一项与回退

真实 PG 身份、并发、导入事务断言以及真实 XLSX/Windows Full 均为“最终封包新库待验”，直接使用已有 references contract 目标与现有验收方式，不新增 runner/workflow。节点 DONE 不等于最终封包验收完成。

B2～B7/A8 和历史四项数据库失败按后续节点处理。用户已要求开始 R7-03 普通删除与历史引用保护，其后节点仍 BLOCKED。

进入基线 `c826dd32e3ecb84dfc732ef60c3fd3aaf4153fdf`；需要回退时受控 revert R7-02 代码提交并同步实际状态、重跑受影响门禁，不恢复无条件覆盖路径为长期兼容实现。
