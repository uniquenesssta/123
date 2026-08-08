# R02 Domain 完整重写阶段完成记录

- 阶段状态：`DONE`
- R2 开始前已验证基线：`274d689b9c4a3d3ce83c7006878a5508ca3f31d6`
- R2-08 最终实现提交：`62b1f622b9c14b33dbaac850812a49c063ccb090`
- 目标平台：Windows

## 1. 阶段完成范围

- R2-01：建立 365 个 Domain 公共兼容类型、Serde、数据库映射和调用链的机器清单。
- R2-02：完成 Competition / Routing 领域职责迁移。
- R2-03：完成 Team / Player / Coach / Formation / Shared 领域职责迁移。
- R2-04：完成 Lineup / Match 领域职责迁移。
- R2-05：完成 Prediction / Research 外围领域职责迁移，不触碰模型实现。
- R2-06：完成 Review / Postmatch 领域职责迁移，并保持原有 PostgreSQL 历史数据兼容。
- R2-07：完成 Analytics / Exchange / AI Workspace / Release 领域职责迁移。
- R2-08：将 Domain 根出口收敛为显式组合根；删除公共 glob export 和根文件领域实现。
- R2-01 至 R2-08 均已关闭为 `DONE`。

## 2. 最终 Domain 边界

- `crates/domain/src/lib.rs` 只保留 17 个业务模块声明、显式 re-export 和 crate 内默认值兼容出口。
- 365 个公共兼容类型继续保持 `football_domain::TypeName` 根路径。
- 34 个历史公共根常量/格式版本符号继续通过显式 re-export 保持兼容。
- 公共根 glob re-export：0。
- `default_true`、`default_team_page_limit`、`default_confidence` 的实现归属 `shared/defaults.rs`。
- 旧职责混合根文件已按各原子任务删除，不保留双实现或复制版本。

## 3. 验证结论

- Domain 类型清单：365 个类型、365 个公共兼容类型、299 个 PostgreSQL 映射类型。
- R2-07 Windows 本机：rustfmt、17/17 Serde、类型清单和架构门禁通过。
- R2-08 Windows 专项 workflow run `31236344727`：根出口验证、球队资料包契约和确定性保护资产验证通过，并生成最终实现提交 `62b1f622b9c14b33dbaac850812a49c063ccb090`。
- 模型公开保护边界保持不变；18 个冻结保护资产继续由原 verifier 指纹锁定，私有 P4/P7 资产未进入公开仓库。
- 用户本机阶段回归未见报错。上传 runtime 日志 `football-runtime-20260808T031048.796Z-pid28528-5d6458a8.jsonl` 共 58 条记录，全部为 `info`；`bootstrap` 返回 `connection_error=null`。
- runtime 实际完成球队列表/详情、阵容引用数据、分析概览、Postmatch 概览、OpenAI 配置与 API Workspace 预设/会话读取，未出现 `error`、`critical`、`panic`、migration 失败或 `new-match-competition` 缺失控件错误。

## 4. 兼容性结论

- 未改变已登记公共领域类型名、Serde 字段/枚举表示、optional/default 语义。
- 未改变 Application/Tauri 公共调用边界、SQL Row 或数据库历史 JSON 语义。
- 未修改生产依赖、模型实现、模型参数、Profile、私有 Schema/fixture/Golden Master。
- R2-06 的历史数据库兼容路径继续 fail-closed，不清库、不覆盖未知 migration 历史。

## 5. 延期到最终统一验收

以下项目仍按阶段既有策略保留到最终统一验收，不描述为已通过：

- 使用名称包含 `test` 的专用可清空 PostgreSQL 数据库执行完整 `run_database_baseline.mjs` 集成测试集合。
- 私有 ModelProvider 环境中的 P4/P7 固定模型回归；公开仓只验证模型 API 与保护边界。
- Windows Full 全交互验收。

用户本机已验证原有 PostgreSQL 数据库可正常启动和读取代表性业务路径，该证据补充但不替代上述专用集成测试。

## 6. 基线与下一阶段

- R2 最终实现基线：`62b1f622b9c14b33dbaac850812a49c063ccb090`。
- 不保留临时 workflow、临时生成脚本副本或并行 Domain 根实现。
- R2 已完成并关闭；下一阶段可进入 R3 Application Ports 与 Service 重写，从 R3-01 端口设计开始。
