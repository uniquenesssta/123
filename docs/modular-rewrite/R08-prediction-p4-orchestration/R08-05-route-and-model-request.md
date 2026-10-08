# R8-05 Route / Model Request：节点完成记录

状态：`DONE`（唯一职责和 Windows Automated 已完成；真实 PG、私有固定回归和 Full 仍待验）。收尾核实日期 2026-10-08（北京时间）。

## 基线、目标与结果

唯一分支 `rewrite/r8-prediction-p4-orchestration`；起点/回退基线 `59c567912194e232b29f36ecf94c3a26c969ed06`。首轮实施 `1f1613d4578b8b8f23f2a458cedc213d072e65b9`，最终修复并通过的提交 `2aa99a76cb97bb289fd486bb8e3ed5059620fb88`，树 `d44f788ff9bfc2a22b212251204c197b72aed9d4`。依据 [R8 任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md)。

实际来源为 shared/routing 的十二项 helper、执行器的显式上下文覆盖和实际 ModelRequest 组装，以及默认 dry run 的公开外壳请求组装。收敛到 route_model_request：mod 仅导出；selection 持有家族规范化与精确注册；route 持有快照类型和受审计路由身份；request 持有输入/上下文及两种请求组装。旧 routing 完整删除，shared 保留真实 p4_planning。preview、build_input、readiness、executor、default 和 lib 原测试直接使用唯一 owner，无空转发或双实现。

Port 路由读取、supports/predict、计时和正式保存/影子不保存仍由原用例负责，R8-06/07 没有提前实施。十二原 helper、显式覆盖、两个组装块及两项原测试去空白/格式逗号比较等价；重新内联后原 executor/default 完整函数等价，preview/build/readiness 除 import 外等价。

## 新增文件

- `crates/application/src/use_cases/prediction/route_model_request/mod.rs`。
- `crates/application/src/use_cases/prediction/route_model_request/request.rs`。
- `crates/application/src/use_cases/prediction/route_model_request/route.rs`。
- `crates/application/src/use_cases/prediction/route_model_request/selection.rs`。
- `crates/application/src/use_cases/prediction/route_model_request/tests.rs`。
- `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-04-deterministic-input-manifest.md`。

## 修改文件

- `README.md`。
- `architecture/application-port-inventory.json`。
- `architecture/domain-type-inventory.json`。
- `crates/application/src/lib.rs`。
- `crates/application/src/use_cases/prediction/build_input/mod.rs`。
- `crates/application/src/use_cases/prediction/dry_run_default_fixture/mod.rs`。
- `crates/application/src/use_cases/prediction/execute_prediction/mod.rs`。
- `crates/application/src/use_cases/prediction/mod.rs`。
- `crates/application/src/use_cases/prediction/preview_route/mod.rs`。
- `crates/application/src/use_cases/prediction/readiness/workflow.rs`。
- `crates/application/src/use_cases/prediction/shared/mod.rs`。
- `crates/application/src/use_cases/prediction/tests.rs`。
- `docs/TESTING.md`。
- `docs/football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md`。
- `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md`。
- `scripts/verify-prediction-service.mjs`。

## 移动或重命名文件

无完整文件移动/重命名；按职责提取。清单采用 `git diff --name-status --no-renames 59c5679..2aa99a7`，Git 相似度检测可能将 request 提取显示为 rename。

## 删除文件

- `crates/application/src/use_cases/prediction/shared/routing.rs`。

以上为实施及修复的累计 **23** 文件清单（6 新增、16 修改、1 删除），包含同轮 R8-04 收尾记录。首轮至修复仅六个修改文件：mod 出口、既有 verifier、Domain 使用清单、根 README、TESTING、阶段 README。本记录本轮新增，并同步根 README、TESTING、任务书和阶段索引；本轮只有文档变化，没有源码/测试/依赖/清单变更。

## 行为、兼容与生命周期

- 家族 trim/ASCII 小写、空白默认 P4、P4/P7 家族选择与精确注册政策保持；错误顺序仍按各原用例，预览先解析 kickoff，执行器先读 scope，readiness 先检查选择。
- RFC3339 转 UTC、cutoff 纳秒、模拟比赛键的 12 个 ASCII 字符/TEAM 回退和原队名保持。已有 match_id 在 context trim，输入 JSON 保留原字符串；只有缺失、空白或非字符串身份才补齐。
- 快照类型为空拒绝，非空支持列表用精确字符串成员匹配，不额外 trim/放宽。受审计 route_identity 的十一字段保持，缺失可选身份继续允许，JSON null/不同身份继续拒绝。
- 自动路由赛事类型不匹配仍拒绝；显式规则包覆盖执行 context 并保留原 metadata。readiness 的原 catalog context 不改。请求使用实际路由 model/parameter/package identity 与参数，不回退默认；默认 dry run 仍原公开载荷、Custom/Null/无规则包身份、T-1h。
- 公共 API/DTO/schema、171 命令、43 Ports、365 Domain/300 映射、数据/配置、错误/日志/UI、依赖/锁文件、模型算法/参数/18 保护资产及 0001～0046 保持。目录只依赖原 Domain/model-api、DTO/error、registry、chrono/serde_json 和公开外壳函数，不含 SQL/具体 Persistence/私有模型。
- 纯同步函数只持有单次局部值；原 async 取消仍由调用方管理，无新共享 State、缓存、监听器、后台任务、重试或时钟。公开 unavailable stub 和 P4.4/provider 策略保持。

## 首轮失败与修复

[首轮 run 36970160631](https://github.com/uniquenesssta/123/actions/runs/36970160631) / job `110722266615` 在精确 `1f1613d` 失败：普通 lib 的 ensure_match_input_id re-export 仅单测消费，Clippy `-D warnings` 将 unused import 提升为错误。前端已通过，但当轮 Rust tests/release/安装包/启动未到达，不计通过。

`2aa99a7` 将该单一目录 re-export 限定 `#[cfg(test)]`，生产 helper 和组装函数不变；其余十项生产导出逐项确认有非测试调用方。既有 verifier 增加 cfg/生产出口限制，移除 cfg 的破坏探针实际拒绝并恢复，没有禁用警告。Context7 已结合实际 Rust 1.88.0 核对 cfg(test)/use 语义。Domain inventory 仅刷新 rustUsageDigest `1dca3d12d1ce050d37721a68c41f5f251a99edbcc153adc5aeff21e8ca97a5a9`，声明摘要/365/300 保持；Application 扫描 388、Domain 扫描 1044。

## 验证与精确 Windows 证据

- 实施/修复时 83 项既有源码检查、完整 `npm run verify:architecture`、Rustfmt --check、`git diff --check`、18 保护指纹、171 命令、46 连续迁移/18 PG 静态基线 PASS。六项原破坏探针及本次 cfg 探针均拒绝并恢复。
- 原两项模型选择测试迁入同一 Application target；新增八项行为测试复用原 Probe，覆盖选择、UTC/身份/字段优先级、显式 context/实际参数、快照与十一身份字段、默认公开请求、预览只读和 Port 错误顺序、自动/显式类型先于后续校验。
- 精确 [Windows run 36971419476](https://github.com/uniquenesssta/123/actions/runs/36971419476)，HEAD `2aa99a76cb97bb289fd486bb8e3ed5059620fb88`，job `110726001870` 所有步骤 SUCCESS。CI 完成时间 **2026-10-02 14:22:38（北京时间）**，本轮再次核实 SHA、全部步骤、日志和 artifact。
- Application **85**（77+8）、Persistence **141**，route_model_request 的十项测试全部 ok；**17** 视口、前端契约/类型/生产构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 及启动 **7 条记录 / 3 个完成操作**实际通过。
- artifact `11212471101`，**14,023,932 字节**，SHA-256 `1cb00b0a91d1bbf25270dc8fb535b0405e0280b60595a313f0bc74f4503a3538`；日志报告 `logs/windows-acceptance-20261002-060054.json`。artifact 有保留期限，精确摘要/节点证据在此保存。
- 本轮纯文档收尾复用已验证源码证据，仅核对文档链接、状态、差异和源码不变；不重复本地/非 Windows 动态验收。推送若触发既有 CI，确认启动即停止完成轮询。

## 延期、风险、回退与下一项

真实 PG/cutoff/历史四项/账本/并发回滚、有效 XLSX、Windows Full、私有 P4/P7 Golden Master 仍最终封包新库待验；18 broad PG 及相关 contracts 仍 ignored，不计通过。公开外壳/fake 不冒充私有模型概率。继承 delete_match 与 model.runs/0041 不可变 trigger 风险继续开放，须新库真实 run fixture 验证保护/拒绝。

没有新 runner/workflow/target/数据库、依赖或迁移，也没有 Linux/macOS 动态验收。受控 revert 本节点到已验证 `59c5679`，同步唯一 owner/入口/门禁/清单，不复制双实现或修改历史数据。05 可关闭为 DONE；06 前置门禁已通过、进入 READY，本轮未开始其代码实施。07～12 继续 BLOCKED，不创建未完成的 R8 阶段完成记录。
