# R4-01 Store / Error / Pool / Migration / Health / Statistics

## 状态

`VERIFYING`

## 实施结果

- 从 R3 最终收口基线 `f2e4841aac873f6a4812801e6be2a6524cd680c1` 进入 R4；R4 阶段分支起点为 `4f8bc9a19fbbd9f52a36b6f94d6d819755482e8f`。
- 删除旧 `crates/persistence-postgres/src/connection.rs` 与顶层 `migration_compatibility.rs`，不保留转发壳。
- `PostgresStore` 唯一 owner 迁入 `store/postgres_store.rs`；`PgPoolOptions` 创建唯一 owner 迁入 `pool/create_pool.rs`；`DatabaseOptions` 与 URL 脱敏进入 `pool/database_options.rs`。
- `PersistenceError/PersistenceResult` 唯一 owner 迁入 `error/persistence_error.rs`，公共错误变体、错误文本与 `From` 转换保持。
- migration runner、已知历史迁移兼容、destructive reset、runtime schema compatibility 分别进入 `migrations/` 独立职责文件；历史 0001–0046 migration SQL 未修改。
- `DatabaseHealth`/`health()` 与 `DatabaseStats`/`stats()` 分别进入 `health/`、`statistics/`；统计中大型事实表估算与小表精确计数策略保持。
- `lib.rs` 只显式组合/导出 R4-01 基础设施；Audit 与通用 mapping/hash 仍留待 R4-02/R4-03，不跨节点提前迁移。
- `architecture/database-baseline.json` 只将 runtime authoritative source/required token 指向新模块；46 个 migration 文件、版本、git blob SHA 与 migration aggregate 均保持。module/state ownership 同步到新的 Store/Pool owner。

## 验证

- scope audit 首轮 run `31612508324` 因 Windows runner 缺少 `rg` 在诊断命令阶段失败，未修改生产源码；恢复 run `31612633857` 改用 PowerShell `Select-String` 后成功并完成调用面/契约扫描。
- 严格 R4-01 hard gate run `31615481637`：migration/dependency diff gate、Persistence Foundation 专项、历史 migration compatibility/reset、数据库静态 baseline、完整 architecture、rustfmt、`cargo check --locked -p football-persistence-postgres`、crate tests、Clippy `-D warnings` 全部在提交前通过。
- 18 个需要专用可写 `FOOTBALL_TEST_DATABASE_URL` 的 PostgreSQL 集成测试继续保持 `ignored`，仅完成编译；没有执行 destructive database reset。

- 首轮 strict gate run `31613473804` 在提交前已通过 R4-01 专项、migration compatibility 与 reset 契约，但数据库静态 baseline 暴露 R0 对 `postgres_integration.rs` 的 runtime-source 指纹已陈旧；R4 起点实际 blob 为 `44dcd594ac739ab341bd5009f613d5de6fd1aa1e`。恢复 gate 以 `git diff --exit-code` 证明该测试文件、build.rs、Cargo.toml、Cargo.lock 与 0001–0046 migrations 对 R4 起点均零 diff，再仅刷新 runtime-source 指纹。Recovery V1/V2 只因临时 workflow 的嵌入脚本截取错误在验证前停止；V3 随后通过全部数据库冻结契约，并在完整 architecture 正确发现 Domain caller inventory 需随 Persistence 路径重排重算，均未提交生产源码。V4 使用官方 `generate-domain-type-inventory.mjs` 重建并机器锁定 365 类型、365 公共兼容类型、299 PostgreSQL 映射及全部 Domain 声明/Serde/映射不变量。

## 兼容性

- 公共 `PostgresStore`、`DatabaseOptions`、`DatabaseHealth`、`DatabaseStats`、`PersistenceError` 名称与根导出路径保持。
- 未修改 Application/Tauri 公共调用签名、DatabaseOptions Serde 字段/default、错误语义、SQL migration、配置、模型资产或生产依赖。
- R4-02/R4-03/R4-04 未开始。最终 clean Public Platform CI 与正式合并/收口前，本节点保持 `VERIFYING`。
