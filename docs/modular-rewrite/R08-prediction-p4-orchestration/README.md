# R08 Prediction / P4 Orchestration：执行记录索引

## 当前阶段状态

`IN_PROGRESS`。唯一阶段分支 `rewrite/r8-prediction-p4-orchestration`，起点/回退基线 `90680bf945fbb0d1c191937c2d9e90c2c916fb00`。用户已启动 R8-01，当前 `VERIFYING`；02～12 `BLOCKED`。精确当前状态只由本索引维护。

## 前置基线与范围

R7 代码/节点和 Windows Automated 已完成，见 [R7 阶段完成记录](../R07-match-lineup-workbook-persistence/R07-stage-completion.md)。最终源码 `a928c8b` 的 run `36871154039` 通过 Persistence 135/Application 55、17 视口、Windows 构建/打包/启动；`90680bf` 仅文档收尾。R7 的真实 PG/XLSX/Windows Full 和模型历史删除风险仍最终封包新库待验，不继承为 PASS。

R8 依据 [任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md) 及总纲订正，沿用现有 Windows 门禁、原单测/contract/PG 和最终新库方式；无新 runner/workflow/数据库或依赖。模型算法/参数/保护资产、公共接口、43 Ports、171 命令、365 Domain/300 映射及 0001～0046 保持。

## 任务状态

| 任务 | 责任 | 状态 |
|---|---|---|
| R8-01 | Match Prediction Input Builder | VERIFYING |
| R8-02 | Historical Feature Reader | BLOCKED |
| R8-03 | Readiness Audit | BLOCKED |
| R8-04 | Deterministic Input Manifest | BLOCKED |
| R8-05 | Route / Model Request | BLOCKED |
| R8-06 | Model Execution Adapter | BLOCKED |
| R8-07 | Run Persistence | BLOCKED |
| R8-08 | P4 Evidence Ledger | BLOCKED |
| R8-09 | Fact Pipeline | BLOCKED |
| R8-10 | Horizon Orchestration | BLOCKED |
| R8-11 | Workbench Reads | BLOCKED |
| R8-12 | Freeze Transaction | BLOCKED |

## R8-01 实际来源与实施边界

旧 `crates/application/src/prediction.rs` 已在 R3 删除。实际来源是 `use_cases/prediction/execute_prediction_from_match/mod.rs` 中的 readiness→模式许可→Port 受检重建→指纹复核→审计附加→PredictionCommand 组装。该责任收敛到同层 `build_input/`，原执行入口仅编排输入构建及模型执行。

Readiness 评分/检查、manifest 算法、路由与模型执行仍由现有 owner 执行，留给 R8-03～06；PostgreSQL 的原载荷/历史贡献查询和窗口选择通过原 PredictionInputPort 复用，不复制实现或提前改写模型。请求/监听器生命周期仍由原调用方负责，用例不增加无对应需求的共享 State 或 UI 模板。

## 动态待验与阶段出口

R8-01 必须取得自己的精确 Windows CI；不继承 R7 编译 PASS。数据库相关 contracts、历史四项/账本、模型历史删除风险、有效 XLSX、Windows Full 的真实结果仍最终新库待验，详见 R7 阶段清单。模型保护或真实固定回归不可取得时明确说明，公共 unavailable stub 不冒充私有引擎 Golden Master。后续节点成功后逐项更新；未完成全部节点不创建阶段完成记录。

## R8-01 实施与门禁记录（等待 CI）

新增 `crates/application/src/use_cases/prediction/build_input/mod.rs` 与本阶段索引；修改原 `execute_prediction_from_match/mod.rs`、模块登记、原 Probe、`verify-prediction-service.mjs`、Domain 使用清单、Application Port 源码计数清单、本任务书、根 README 与 TESTING。没有移动/重命名、删除文件。原输入构建责任整体迁出执行入口，唯一 owner 内依次完成评估、对应模式许可、家族规范化、按 assessed_at 重建、指纹复核、附加审计和原命令组装；原执行入口保留两种 API/模式并调用现有执行器。原 API/DTO/schema、数据格式、配置、错误/日志/UI 和模型保护资产保持；Persistence 输入构建及 cutoff SQL 无改动。

新增 6 项 inline 行为测试，复用原 Probe，覆盖 P4/P7 与正式/影子、权限拒绝前置、stale 输入/质量、仅运行身份变化、Port 失败/无自动重试、缺失指纹/manifest、非法家族及非对象输入。源码预期 Application 61（原 55+6）、Persistence 135，实际执行待本项 Windows CI。原模型执行器测试继续核对影子不写历史与正式失败不保存；不使用虚构模型概率冒充真实 Golden Master。

沿用现有前端源码清单、架构/保护资产/命令/迁移验证器及 Rustfmt。六项破坏探针：绕过权限、重取时钟、漏指纹、漏审计、丢显式路由、影子改正式，均须被现有增强门禁拒绝后恢复。六项实际均被拒绝并恢复；83 项现有前端源码检查、完整 `npm run verify:architecture`、Rustfmt 和 `git diff --check` 已 PASS，5 项浏览器检查待 Windows。`verify_protected_assets.mjs` 的 18 项指纹及私有资产缺席、`verify_command_contract.mjs` 的 171 命令、`verify_database_baseline.mjs` 的 46 连续迁移/18 原 PG 契约静态检查均 PASS；这不等同于真实 PG 执行。去除空白比较确认原权限/输入准备/指纹/审计及命令字段原样迁移。Port 清单只将 Application 文件数 376→377，43 traits 与 Port 源码 SHA 不变；Domain 清单只更新使用方与扫描文件数（1029→1030），365/300 和 sourceDigest 不变。未运行 Linux/macOS Cargo 或客户端验收。本节点 Windows CI 触发后停止轮询，等待用户反馈；通过才创建正式节点完成记录并关闭 01。

回退点为 `90680bf945fbb0d1c191937c2d9e90c2c916fb00`；使用 Git 反向本节点提交回退，不手工恢复旧文件。02 待 01 精确门禁通过后开始，下一责任为原 historical feature reader。Mermaid Chart 已按实际输入构建/权限/指纹/正式影子链路更新；结束时 Create State 保存节点与剩余约束。
