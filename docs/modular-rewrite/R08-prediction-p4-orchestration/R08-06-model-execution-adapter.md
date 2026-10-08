# R8-06 Model Execution Adapter：实施记录

## 状态与基线

`VERIFYING`。2026-10-08 用户授权“05收尾开始06”，唯一分支 `rewrite/r8-prediction-p4-orchestration`。05 精确代码 `2aa99a7` / Windows run `36971419476` 已通过，收尾文档 run `37737677787` 亦 SUCCESS。06 起点/回退基线为 `4420fd48ab5e88d55ce15e5b0cdc4b6b05f1c43d`；它只补 UI 接入文档，模型源码仍为已验证 05 实现。06 需取得自身精确 Windows CI，不继承前项 PASS；07～12 仍 BLOCKED。

## 修改原因与最终职责

原 execute_prediction 在路由/审计/运行保存编排中还实现模型注册查找、supports、predict、错误转换与计时；默认 dry run 重复查找/predict/error。将这一模型边界收拢到唯一 `model_registry/prediction_model_adapter.rs`，两个调用方直接复用。ModelRegistry 原状态、公开方法和排序不变；适配器仅借用请求和克隆原 provider Arc，不新增注册表、缓存或转发服务。

路由执行仍按原顺序读取 scope、解析输入、精确选择检查、解析路由、上下文/窗口/受审计路由校验，再进入 adapter 的查找/supports；随后原 request owner 组装、原 manifest owner 计算/复核审计，再经 adapter 调用模型并计时。supports 使用实际 context；不支持提示仍使用 provider display_name 与原 scope 类型。缺失模型保留原 ID，五类模型错误完整转换为既有 ApplicationError，不重试或回退。

模型输出（identity、summary、payload、explanation）原样返回。计时继续只围绕原模型调用，截断为毫秒并以 i64::MAX 饱和；不计入查找/支持/请求/审计/保存。正式模式仍经原 Port 保存成功运行，影子仍 Uuid::nil；保存拆分留 R8-07。默认 dry run 仍先精确查找 P4_MODEL_ID，再构建默认请求并直接 predict，不增加 supports、validate 或计时。readiness 的只读支持评估及后续 Analytics 模型调用不提前迁移。

## 全部文件差异

本节点实际新增 3、修改 11；无完整文件移动/重命名或删除。迁出的旧调用责任已从两个入口删除，没有旧实现或空兼容转发。

| 类型 | 文件 |
|---|---|
| 新增 | `crates/application/src/model_registry/prediction_model_adapter.rs` |
| 新增 | `crates/application/src/model_registry/prediction_model_adapter/tests.rs` |
| 新增 | `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-06-model-execution-adapter.md` |
| 修改 | `crates/application/src/model_registry/mod.rs` |
| 修改 | `crates/application/src/use_cases/prediction/execute_prediction/mod.rs` |
| 修改 | `crates/application/src/use_cases/prediction/dry_run_default_fixture/mod.rs` |
| 修改 | `scripts/verify-prediction-service.mjs` |
| 修改 | `scripts/verify-application-composition.mjs` |
| 修改 | `architecture/application-port-inventory.json` |
| 修改 | `architecture/domain-type-inventory.json` |
| 修改 | `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md` |
| 修改 | `docs/football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md` |
| 修改 | `docs/TESTING.md` |
| 修改 | `README.md` |

适配器的查找、支持与调用/计时属于同一模型边界责任，保持一个生产文件，测试独立为原 crate 子模块。按总纲拆分订正，将旧任务书 execute-model 目录要求修订为实际 owner，不创建无实质职责的转发目录。同步过时来源、真实输入/依赖/同步生命周期与 Windows-only 验证范围，不增设任务节点。

## 兼容与状态

公共 Application API、ModelRegistry/PredictionModel、DTO、Schema、数据格式、配置、错误提示、日志、UI 行为保持。43 Ports、171 Tauri 命令、365 Domain/300 映射、18 模型保护资产、P4.4 SHADOW_ONLY、P7 算法/概率/矩阵/参数、依赖/锁文件与迁移0001～0046未修改。私有引擎/样本继续缺席；测试假提供器只验证编排，不能充当 Golden Master。

原 Registry 仍唯一持有 provider，适配器只持有单次调用局部状态，无任务/监听器/定时器/回调、共享 State 或新增并发策略。同步模型调用与请求取消继续由原调用方管理。Port 清单只更新 Application 文件388→390，trait/声明 SHA 不变；Domain 清单只更新使用方、usage digest 与扫描1044→1046，sourceDigest/365/300不变。

## 验证与失败处理

新增 7 项测试进入原 Application target，预期 Application 92（85+7）/Persistence 141；尚未将预期数量记为动态 PASS。覆盖精确注册/Arc、缺失错误、实际 context/原 scope 提示、完整请求和全部输出保真、五类错误/单次调用、毫秒截断和饱和、默认 dry run 无额外支持/校验、路由失败优先级/不写历史。原正式/影子和审计测试继续沿用。

实际本地验证（Node 24.19.0，Rustfmt 1.88.0）：

- 原 verify-frontend 88 项清单中 83 项源码检查 PASS；5 项浏览器检查留本次 Windows CI。报告 `r806-source-checks.json` 在本轮工作目录。
- `npm run verify:architecture` PASS；报告 `r806-architecture.log` 在本轮工作目录。
- Rustfmt `--edition 2021 --check` 针对模块登记、adapter（递归含 tests）和两个调用方 PASS；`git diff --check` PASS。
- `node scripts/verify_protected_assets.mjs`：18 指纹保持、私有资产缺席；`node scripts/verify_command_contract.mjs`：171命令；`node scripts/verify_database_baseline.mjs`：46连续迁移/18原PG静态契约，全部PASS，后者不等于真实PG运行。
- 六项临时破坏探针（跳过supports、错误提示类型漂移、丢模型错误详情、移除耗时饱和、额外validate、调用方绕过adapter）全部由增强的原门禁拒绝，恢复后PASS。
- 将新调用重新内联原模型块后，整个原执行函数去空白比较一致，确认路由/审计/保存/响应顺序不变；再核对 adapter 的精确查找、错误、借用请求、无输出重写与耗时规则。

首轮源码检查发现格式化后 usage scan 过期、原组合根精确文件集合未登记新 adapter，已刷新原清单并更新精确集合，83项及完整架构复跑通过；未删除公共注册/状态 owner 断言。现有本地 formatter 动态库不可读，恢复同版本工具后目标格式检查通过。没有本地 Linux/macOS Cargo或客户端动态验证，也没有新增workflow、runner、test target、数据库或依赖。

本提交推送后沿用原 Windows CI 验证完整前端、17视口、fmt/Clippy/workspace tests、release/MSI/NSIS与启动；确认启动即结束，不持续轮询。CI成功才能将06改为DONE并开始07。真实PG/历史四项/账本、有效XLSX、Windows Full、私有Golden Master及继承delete_match→model.runs.match_id NULL与0041不可变trigger冲突，继续最终封包新库待验，ignored不计PASS。

## 入口、回退与继续

两个执行入口已切换到唯一 adapter，旧模型边界块已移除；保存仍是原 owner，没有提前实现07。使用Git受控revert本节点提交回退至`4420fd4`，同步调用、门禁、清单和文档，不复制旧代码建立双实现或修改历史数据。Mermaid Chart已更新实际链路，结束时Create State保存当前VERIFYING状态与约束。下一步是用户反馈06 CI后核验精确SHA和日志，成功则收尾06并按指令开始07。
