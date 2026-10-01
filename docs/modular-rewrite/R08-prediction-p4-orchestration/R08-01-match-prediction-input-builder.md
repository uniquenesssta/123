# R8-01 Match Prediction Input Builder：节点完成记录

状态：`DONE`（唯一输入构建责任和 Windows Automated 完成；真实 PG、私有固定回归及 Windows Full 仍待验）。完成日期 2026-10-02（北京时间）。

## 基线、目标与实际结果

唯一分支 `rewrite/r8-prediction-p4-orchestration`；起点/受控回退基线 `90680bf945fbb0d1c191937c2d9e90c2c916fb00`。实施代码 `cba72fdfaaf640a17c1e73c326536189dda74d17`，树 `c167e3f547cc63423357d9fe9f950dfbd96705d3`；对应 [R8 任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md)。

原 `execute_prediction_from_match` 同时承担受审计输入构建和推演执行。完整度评估→正式/影子许可→家族规范化→复用 assessed_at 准备→指纹复核→附加审计→PredictionCommand 组装整体迁入唯一 `build_input/mod.rs`；原两个公开入口仍保留对应模式，顺序调用构建与原执行器。原权限/准备/指纹/审计及命令字段去除空白比较完全等价。没有静默修复、不新增重试或历史写入。

## 新增文件

- `crates/application/src/use_cases/prediction/build_input/mod.rs`：唯一受审计输入构建与六项 inline 行为测试。
- `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md`：阶段索引和当前节点门禁记录。

## 修改文件

- `crates/application/src/use_cases/prediction/execute_prediction_from_match/mod.rs`：移除构建责任，保留正式/影子执行编排。
- `crates/application/src/use_cases/prediction/mod.rs`：登记新模块。
- `crates/application/src/use_cases/prediction/tests.rs`：原 Probe 增加准备输入和请求记录，复用原故障注入；未选中的 Port 仍拒绝。
- `scripts/verify-prediction-service.mjs`：扩充唯一 owner、顺序、时钟/字段、副作用与六项测试门禁。
- `architecture/application-port-inventory.json`：Application 文件数 376→377，43 traits 与 Port SHA 不变。
- `architecture/domain-type-inventory.json`：使用方及扫描数 1029→1030，365 类型/300 映射及声明摘要不变。
- 根 `README.md`、`docs/TESTING.md`、R8 任务书：更新实际来源、实施、验证和待验边界。

以上为实施提交的 11 文件完整清单；本文件为本轮新增收尾记录，索引/README/任务书/TESTING 同步收尾状态。01 的收尾文档不改上述已验证源码；同轮 R8-02 另行实施并独立验证。

## 移动或重命名文件

无。

## 删除文件

无。旧执行入口中的输入构建函数体已迁出，未保留重复实现；旧 `prediction.rs` 已在 R3 删除，不重建。

## 兼容、依赖与决策

核心用例依赖既有 Ports、Domain/DTO、registry、readiness 及 shared audit/routing；所有输入 I/O 经过原 PredictionInputPort，SQL/历史贡献与 Persistence 原载荷构建不变。无共享状态、监听器或额外时钟，原调用方管理生命周期。Readiness/manifest/路由/模型执行/保存分别留 R8-03～07；历史读取留 R8-02。目录使用 Rust `build_input` 命名，按总纲订正保留紧密耦合的小用例实现。

171 命令、43 Ports、API/DTO/schema、数据格式、数据库迁移、配置、依赖/锁文件、错误提示、日志/UI、算法/参数/模型保护资产均保持。正式失败和影子不保存继续由原执行器测试覆盖；测试 Probe 不冒充私有提供器。

## 验证与精确 Windows 证据

- 83 项既有前端源码检查、完整 `npm run verify:architecture`、Rustfmt、`git diff --check` PASS；六项破坏探针（权限、重取时钟、指纹、审计、显式路由、影子变正式）均拒绝并恢复。
- `verify_protected_assets.mjs`：18 个指纹/私有资产缺席 PASS；`verify_command_contract.mjs`：171 命令 PASS；`verify_database_baseline.mjs`：46 连续迁移及 18 原 PG 契约静态基线 PASS。
- 精确 [Windows run 36881256338](https://github.com/uniquenesssta/123/actions/runs/36881256338)，HEAD 为上述 `cba72fd`，job `110433291567` 全 SUCCESS。2026-10-01 23:48:06（北京时间）completed/success，本轮核实 SHA、全部步骤、日志及 artifact。
- Application **61 项**（原 55+6）、Persistence **135 项**、17 个截图视口、前端类型/生产构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 和启动 **7 条记录 / 3 个完成操作**实际通过；六项新输入构建测试全部 ok。
- artifact `11173767485`，14,020,690 字节，SHA-256 `33770c2250cc68e434a98b00139c525eabe47d7ed9222fa18f161eeee03f666f`。

## 延期、风险与回退

18 broad PG 与数据库 contracts 在 CI 仍 ignored；历史四项/账本/真实 cutoff/并发/回滚、有效 XLSX、Windows Full 继续用户最终封包新库待验。公开仓库未分发私有 P4/P7 固定概率 Golden Master，不能用 stub 当其通过。继承的比赛删除与模型历史不可变 trigger 风险仍需最终新库真实 run fixture 验证，未关闭。

只使用 Windows 动态验收，没有 Linux/macOS Cargo 或客户端验证、新测试目标/runner/workflow/数据库。无未说明失败；源码范围已完成，可按用户指令开始 R8-02，但不表示动态待验项完成。受控 revert 实施提交回到 `90680bf`，同步入口/测试/门禁/清单，不手工复制旧文件，不影响历史数据或迁移。
