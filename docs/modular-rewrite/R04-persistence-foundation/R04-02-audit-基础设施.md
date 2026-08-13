# R4-02 Audit 基础设施

## 状态

`DONE`

## 实施结果

- 新建 `crates/persistence-postgres/src/audit/`，按 `audit_event.rs`、`audit_payload.rs`、`audit_hash.rs`、`write_audit_event.rs` 拆分事件/实体 ID、payload、SHA-256 与唯一 SQL writer；`mod.rs` 只负责显式出口。
- 删除 `lib.rs` 中旧 `write_audit_event` 与 `sha256_json` 实现；根模块只保留 crate 内兼容 re-export，既有调用签名与 SHA-256 JSON 算法保持。
- 迁移 Match Exchange、Player、Spreadsheet Exchange、Team Catalog、Team Force Delete 中全部直接 `INSERT INTO audit.events`；生产 Rust 的审计 INSERT 只允许存在于 `audit/write_audit_event.rs`。
- 删除 Player Catalog 的 pool 级 `audit` 和事务级 `audit_in_tx` 重复 writer。`create_team` 改为 team row 与 `team_created` audit row 同一事务后统一 commit，消除审计失败时业务行已单独落库的非原子状态。
- 未修改任何 0001–0046 migration SQL、数据库 schema、公共 Application/Tauri 接口、DTO、配置、模型资产或生产依赖；R4-03/R4-04 未提前实施。

## 验证

- scope audit run `31627365139` 与 detail runs `31627711449` / `31627837301` 用 PowerShell 扫描 Audit owner、直接 SQL 与 player helper 调用面，均未修改生产源码。
- strict hard gate run `31629430245` 在提交前执行 R4-02 Audit 专项、R4-01 Persistence Foundation、数据库冻结/历史契约、官方 Domain inventory、完整 architecture/frontend、rustfmt、Persistence check/tests 以及 workspace Clippy/tests；全部通过后才提交本节点源码树。
- clean Public Platform CI run `31630618699` / Windows automated delivery gate job `94228122818`：`SUCCESS`；架构检查、Windows Automated acceptance 与 evidence upload 全部成功。
- evidence artifact `9155761101`，13,909,390 bytes，SHA-256 `82dbb88b6dd974ad43c2d86bfb291aca8831faa9d956212bdc04de7110d87dee`。
- 18 个要求专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试未执行；未执行 destructive database reset。

## 兼容性与剩余风险

- audit event_type、entity_type、entity_id、payload 值与 SQL 列保持；SQLx/Serde 错误继续通过既有 `PersistenceError` 转换传播，不捕获忽略。
- `sha256_json` 仍为 `serde_json::to_vec -> SHA-256 -> hex`，不改变现有 fingerprint/hash 输出。
- 真实可写 PostgreSQL 的 18 个集成测试与 destructive reset 仍留待具备专用测试数据库时执行；该限制不改变本节点已通过的静态、crate、workspace 与 Windows Automated 门禁结论。

## 正式收口

- PR #23 已由 Draft 转 Ready，并按固定 HEAD `6598065e6edf671e8806fc77901d083bad910542` 合并到 `rewrite/r4-persistence-foundation`；merge commit `bc4154044f194e5b1505b4ebb308ba51d6208663`。
- R4-02 正式关闭为 `DONE`；R4-03 开放为 `READY`，R4-04 继续 `BLOCKED`。
- 本收口只同步任务状态与验证证据，不新增 R4-03/R4-04 生产实现。
