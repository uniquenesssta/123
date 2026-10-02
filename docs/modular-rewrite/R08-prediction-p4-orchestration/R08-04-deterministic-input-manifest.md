# R8-04 Deterministic Input Manifest：节点完成记录

状态：`DONE`（唯一清单职责与 Windows Automated 完成；真实 PG、私有固定回归和 Full 仍待验）。核实日期 2026-10-02（北京时间）。

## 基线、目标与结果

唯一分支 `rewrite/r8-prediction-p4-orchestration`；起点/受控回退基线 `47ba3dea6ca873248728b9846421c009d00f51bc`。实施提交 `59c567912194e232b29f36ecf94c3a26c969ed06`，树 `336d8aa3ebb1b775658968558d64a55e11f97870`，依据 [R8 任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md)。

原 shared/audit 的六项函数和两项测试迁入 input_manifest；canonical 唯一持有清单/哈希，audit 唯一持有受检重建、附加和摘要复核，mod 仅导出。build_input、readiness、执行器直接使用新 owner；旧 audit 文件及登记删除，无双实现或转发空壳。原函数/测试去空白及格式逗号比较等价，调用方除 import 外不变。

只在副本上排除原五个运行字段：根 feature_snapshot_id/input_audit、snapshot 的 snapshot_id/frozen_at、sources 对象的 accessed_at。事实、质量、路由、cutoff 纳秒、数组顺序及 null/缺失保持。原 `serde_json::to_vec`→SHA256→小写 hex 保持；执行器的完整 request.input 哈希继续包含运行身份及审计。缺失清单/哈希/非对象、版本/清单/哈希/篡改错误优先级、可选元数据回退与未 trim 身份保持。P4 输出矩阵哈希独立 owner 未迁移。

## 新增文件

- `crates/application/src/use_cases/prediction/input_manifest/audit.rs`。
- `crates/application/src/use_cases/prediction/input_manifest/canonical.rs`。
- `crates/application/src/use_cases/prediction/input_manifest/mod.rs`。
- `crates/application/src/use_cases/prediction/input_manifest/tests.rs`。
- `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-03-readiness-audit.md`。

## 修改文件

- `README.md`。
- `architecture/application-port-inventory.json`。
- `architecture/domain-type-inventory.json`。
- `crates/application/src/use_cases/prediction/build_input/mod.rs`。
- `crates/application/src/use_cases/prediction/execute_prediction/mod.rs`。
- `crates/application/src/use_cases/prediction/mod.rs`。
- `crates/application/src/use_cases/prediction/readiness/tests.rs`。
- `crates/application/src/use_cases/prediction/readiness/workflow.rs`。
- `crates/application/src/use_cases/prediction/shared/mod.rs`。
- `crates/application/src/use_cases/prediction/tests.rs`。
- `docs/TESTING.md`。
- `docs/football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md`。
- `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md`。
- `scripts/verify-prediction-service.mjs`。

## 移动或重命名文件

无完整文件移动/重命名；按函数责任提取（以上按 `git diff --name-status --no-renames` 记录，Git 相似度检测可能将 audit 提取显示为 rename）。

## 删除文件

- `crates/application/src/use_cases/prediction/shared/audit.rs`。

以上为实施提交 20 文件完整清单；本记录本轮新增，相关文档同步收尾，同轮 R8-05 独立实施及验收。

## 兼容、依赖与生命周期

公共 API/DTO/schema、171 命令、43 Ports、365 Domain/300 映射、数据/配置、错误/日志/UI、依赖/锁文件、模型算法/参数/18 保护资产及 0001～0046 均保持。核心只依赖 Domain/原 DTO、serde_json/SHA 和既有函数，无具体 Persistence、SQL 或新 Port。纯函数不持有共享 State、后台任务、缓存、监听器或时钟；局部副本随请求释放。公开 unavailable stub 和 P4.4/provider 策略保持。

## 实际验证与精确 Windows 证据

- 83 项既有源码检查、完整 `npm run verify:architecture`、Rustfmt、`git diff --check` PASS；18 保护指纹/私有资产缺席、171 命令、46 连续迁移和 18 原 PG 契约静态基线 PASS。
- 六项破坏探针（放宽排除、修改原输入、pretty JSON、跳过 stale 哈希、跳过摘要复核、执行绕过审计）全部拒绝并恢复。
- 精确 [Windows run 36966815323](https://github.com/uniquenesssta/123/actions/runs/36966815323)，HEAD 为上述 `59c5679`；job `110712250872` 全 SUCCESS，完成时间 2026-10-02 13:25:51（北京时间）。本轮核实 SHA、全部步骤、日志和 artifact。
- Application **77**（69+8），两项保留/八项新增清单测试全部 ok；Persistence **141**、**17** 视口、前端类型/构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 及启动 **7 条记录 / 3 个完成操作**实际通过。
- artifact `11210976146`，14,025,634 字节，SHA-256 `dc0c8ec9ea9d5c9b5c9e9d62b0cdb27e5a9420747de5b81f839e68b0b7625677`。
- 固定公开平台清单指纹 `178afe68af4d0cb8ba9341a7f5f47ec3b89c4c2b9ceaafd0e6615db3a9a0fb87` 是载荷契约，不冒充私有概率 Golden Master。

## 延期、风险和回退

真实 PG/cutoff/历史四项/账本/并发回滚、有效 XLSX、Windows Full、私有 P4/P7 固定回归继续最终封包新库待验；18 broad PG 和相关 contracts 仍 ignored，不计通过。继承的比赛删除与 model.runs/0041 不可变 trigger 风险保持开放，须新库真实 run fixture 验证保护/拒绝。

没有 Linux/macOS 动态验收、新 runner/workflow/target/数据库、依赖或迁移。受控 revert 到已验证 `47ba3de` 并同步唯一 owner/入口/门禁/清单，不复制旧实现或修改历史数据。04 可关闭；05 按用户指令开始，独立取得 Windows CI 后方可关闭。
