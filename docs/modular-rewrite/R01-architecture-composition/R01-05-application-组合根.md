# R01-05 Application 组合根实施记录

- 任务状态：`DONE`
- 前置基线：`f930b81a720ff1f87bb0e63cb39c9b14c6a2a23f`
- 实施提交：`f2df52e9fb3a2506c6991089e7d8bbdc5868e326`
- 正式验收提交：`08803725dcd9f403ffc25552c27d2a9c0d3acd2d`
- 正式验收源码树：`e5b93e149270168bd4626e34b5a514c8779c0d51`
- 目标平台：Windows

## 1. 完成内容

- 建立 `composition/`、`service/`、`model_registry/` 三个职责目录。
- `ApplicationComposition` 唯一构造模型注册表、持久化端口注册表和 P4 worker 初始状态。
- `ApplicationService` 保持原公共名称、`new/default` 构造语义及全部既有方法承载方式，仅作为兼容门面持有组合结果。
- `ModelRegistry` 保持 `new/register/get/descriptors/default` 公共接口和描述符排序语义。
- PostgreSQL 具体 crate 导入已收敛到 `composition/port_registry.rs`；既有业务模块改用内部 `PersistenceStore` 端口类型，不改变实际适配器、连接、事务或错误语义。
- 默认公开模型 Stub 的注册入口已从 `lib.rs` 切换到唯一组合根，注册集合和顺序不变。

## 2. 职责与依赖变化

- `lib.rs` 仅保留 crate 模块出口、公共 DTO、错误类型和原有契约测试，不再定义服务、模型注册表或活动数据库状态。
- `composition/application_composition.rs`：只负责对象图构造和默认适配器注册。
- `composition/port_registry.rs`：只负责具体持久化适配器导入、活动数据库状态和端口槽位构造。
- `service/application_service.rs`：只负责兼容门面与运行时状态持有。
- `model_registry/registry.rs`：只负责模型 ID 到 `PredictionModel` 的注册、查询和描述符输出。
- 未新增生产依赖，`Cargo.toml` 和 `Cargo.lock` 未修改。

## 3. 保持不变

- Tauri 命令名称、注册顺序、参数和返回类型不变。
- 公共 `ApplicationService`、`ModelRegistry`、DTO 和错误枚举名称不变。
- PostgreSQL 数据格式、迁移、连接行为、关闭行为和后台 worker 启动行为不变。
- 默认模型 ID、描述符、外部提供器 Stub 行为和模型保护资产不变。
- 前端 UI、启动顺序、窗口行为、配置和日志等级不变。

## 4. 实际新增文件

- `crates/application/src/composition/mod.rs`
- `crates/application/src/composition/application_composition.rs`
- `crates/application/src/composition/port_registry.rs`
- `crates/application/src/service/mod.rs`
- `crates/application/src/service/application_service.rs`
- `crates/application/src/model_registry/mod.rs`
- `crates/application/src/model_registry/registry.rs`
- `scripts/verify-application-composition.mjs`
- `docs/modular-rewrite/R01-architecture-composition/R01-05-application-组合根.md`

## 5. 实际修改文件

- `crates/application/src/lib.rs`
- `crates/application/src/analytics.rs`
- `crates/application/src/database.rs`
- `crates/application/src/fact_pipeline.rs`
- `crates/application/src/openai_research.rs`
- `crates/application/src/p4_orchestration.rs`
- `crates/application/src/p4_persistence.rs`
- `crates/application/src/p4_workbench.rs`
- `crates/application/src/prediction.rs`
- `architecture/module-boundaries.json`
- `architecture/state-ownership.json`
- `package.json`
- `scripts/verify-frontend.mjs`
- `README.md`
- `docs/modular-rewrite/R01-architecture-composition/README.md`

## 6. 移动、重命名和删除

- 移动或重命名：无。
- 最终交付树删除：无。
- 临时执行脚本和 workflow 在实施提交前自删除，不属于交付树。

## 7. 验证

实施提交生成前已实际执行：

- `node scripts/verify-application-composition.mjs`：通过。
- `npm run verify:architecture`：通过。
- `npm run verify:frontend`：通过。
- `npm run verify:rust`：通过。
- `cargo fmt --all -- --check`：通过。

正式 Windows Automated 已在正式验收提交上实际通过：

- Workflow run：`31073166446`。
- Windows job：`92525208547`。
- 架构边界、完整前端验证、Rust 格式、Clippy、工作区测试和 Tauri Windows release 构建：通过。
- release 客户端首次启动即建立运行日志；客户端启动与状态载入：通过。
- 运行日志验收：`PASS`，共 7 条记录、3 个完成操作，覆盖率与错误扫描通过。
- 证据 artifact：`8956912712`，名称 `windows-automated-delivery-evidence-08803725dcd9f403ffc25552c27d2a9c0d3acd2d`，大小 `14117884` 字节。
- artifact SHA-256：`495d3c5e29f2b474e97b98f89dd64b6175cc6e9496dd77681c0a567e00c60016`。

## 8. 未执行与剩余风险

- 真实 PostgreSQL 实跑尚未执行；18 个需要专用可写测试数据库的集成测试保持忽略，按既定策略保留到最终统一验收。
- Windows Full 交互验收和用户本机 Windows 10/11 实机验收尚未执行，保留到最终统一验收。
- 上述延期项已明确登记，不阻塞 R1-05 和 R1 阶段关闭，也不得在后续阶段描述为已通过。

## 9. 回退

- 回退到 R1-04 清洁基线提交 `f930b81a720ff1f87bb0e63cb39c9b14c6a2a23f`。
- 不保留双实现、复制文件或长期兼容层。
