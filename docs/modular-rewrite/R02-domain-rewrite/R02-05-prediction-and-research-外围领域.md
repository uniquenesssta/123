# R02-05 Prediction 与 Research 外围领域实施记录

- 任务状态：`DONE`
- 前置已验证提交：`3e5812c7c0d1d626e0f5ed3a6db0295af6d6979c`
- 目标平台：Windows
- 类型范围：Prediction 48 + Research 27，共 75 个公共兼容类型

## 1. 实际问题

R2-01 冻结清单确认 Prediction 48 个类型仍分布在 Domain 根类型、`p4_orchestration.rs`、`p4_persistence.rs`、`p4_workbench.rs`、`prediction_readiness.rs`；Research 27 个类型仍分布在 `fact_pipeline.rs`、`research_gateway.rs`。这些文件同时承载计划编排、冻结状态、持久化契约、快照、工作台、输入就绪度、实体解析、时间审计、来源策略、证据路由、冲突评估和研究网关等不同变化原因，需按职责目录拆分。

## 2. 本节点目标结构

### Prediction

- `prediction/context.rs`：比赛预测上下文
- `prediction/result.rs`：预测摘要与持久化运行结果
- `prediction/horizon.rs`：P4 时间窗及其时间计算
- `prediction/orchestration/`：规划、状态、冻结任务、冻结就绪度
- `prediction/persistence/`：版本、研究运行、证据、赛前快照持久化契约
- `prediction/workbench/`：P4 工作台聚合与人工冲突处理
- `prediction/readiness/`：预测输入就绪等级、检查与审计摘要

### Research

- `research/entity.rs`：实体解析
- `research/time_audit.rs`：时间审计
- `research/source_policy.rs`：来源等级与来源策略
- `research/routing.rs`：证据路由注册、草稿与结果
- `research/conflict.rs`：冲突评估
- `research/pipeline.rs`：事实管线上下文与汇总
- `research/gateway/`：OpenAI 尝试、Web 来源与使用量

## 3. Atomic Task

### R2-05A Prediction

迁移 48 个 Prediction 类型，删除对应根定义与旧 Prediction 源文件；保持根级兼容导出、Serde、数据库映射和调用语义不变。完成后先执行 Domain Serde/模块路径、类型清单与架构专项门禁，硬失败则停止。

### R2-05B Research

在 R2-05A 通过后迁移 27 个 Research 类型，删除旧 `fact_pipeline.rs`、`research_gateway.rs`；更新迁移进度和 75 个模块路径身份断言，再执行完整前端、Rust、架构与保护资产门禁。硬失败不得进入正式 Windows Automated。

## 4. 必须保持不变

- 根级 `football_domain::TypeName` 公共类型路径与类型身份；
- Serde 字段名、枚举 wire value、默认值、optional 语义、历史 JSON；
- PostgreSQL 映射、SQL Row、迁移、Application、Tauri DTO、公共命令与错误语义；
- P4/P7 模型实现、参数、Profile、Schema、fixture、Golden Master 与保护资产；
- Cargo/npm 生产依赖及用户可观察行为。

R2-08 之前继续保留既有根级兼容 re-export；本节点不提前处理全局 glob 出口债务。

## 5. 验证计划

- `cargo test --locked -p football-domain --test serde_contracts`
- `node scripts/generate-domain-type-inventory.mjs`
- `node scripts/verify-domain-type-inventory.mjs`
- `npm run verify:architecture`
- `node scripts/verify-protected-assets-deterministic.mjs`
- `npm run verify:frontend`
- `npm run verify:rust`
- 最终实施源码树上的 `Public Platform CI` / `scripts/windows-acceptance.ps1 -Mode Automated`

## 6. 延期项

真实 PostgreSQL、Windows Full 与用户本机 Windows 10/11 实机验收继续按既定计划保留到最终统一验收。

## 7. 实施与验证结果

- staged run `31158780693`：R2-05A Prediction 迁移、Serde 10/10、类型清单、架构与保护资产门禁通过并生成独立提交 `2cd685b8057a1bce2f75e4c7f5b56aed1bf3d142`；R2-05B Research 迁移与 Serde 11/11、类型清单、架构、保护资产、完整 frontend 通过，但完整 Rust 在 Clippy `-D warnings` 因 `prediction/orchestration/planning.rs` 两个未使用 import 停止，Research 未提交。
- 已直接删除两个无效 import，不增加 `allow` 或放宽 Clippy。
- recovery run `31159547810`：Prediction Serde 10/10 通过，随后类型清单因上述源文件变化后的指纹未刷新而按门禁停止；Research 未执行。
- inventory refresh run `31159710816`：仅重新生成并验证 `architecture/domain-type-inventory.json`，成功后自清理临时 workflow。
- recovery run `31159821513`：从已验证 Prediction 基线继续完成 Research 27 类型迁移、清单刷新、架构、保护资产、完整 frontend 与 Rust 回归；旧 `fact_pipeline.rs`、`research_gateway.rs` 已删除。
- 75 个类型均位于 `prediction/` 或 `research/` 职责目录，根级公共兼容路径与类型身份保持不变。
- 正式 Windows Automated run `31171082098`、job `92842834091` 已在包含 R2-05 最终源码树的 `new-B` 提交 `e328b4aa5a7737e6bb378abf8b891cd953b99f62` 上通过；artifact `8991618221` 大小 `14117154` 字节，SHA-256 为 `71320b8ef97e62be2fe2323327d21f4870476092ad024d7b8c2c26a4ade9dc59`。
- R2-05 状态现为 `DONE`，R2-06 已开放为 `READY`。
