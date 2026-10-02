# R8-03 Readiness Audit：节点完成记录

状态：`DONE`（唯一就绪审计职责和 Windows Automated 完成；真实 PG、私有固定回归与 Full 仍待验）。核实日期 2026-10-02（北京时间）。

## 基线、目标与结果

唯一分支 `rewrite/r8-prediction-p4-orchestration`；起点/回退基线 `cc2b0fe7601efdbad44fa5badcb6507f226698a7`。实施提交 `47ba3dea6ca873248728b9846421c009d00f51bc`，树 `628cf2df7eab3cde4c3e9fc7b6b06bcea6acb0b3`，对应 [R8 任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md)。

原审计入口与 shared/readiness_checks 分散持有同一审计责任。迁入 readiness 目录后，workflow 唯一持有一次 assessed_at 和原只读 Port 顺序；lineups/input_quality 分别做纯检查；report 持有检查载荷与原等级/评分/原因归类；mod 只登记和导出。Service 和 build_input 直接接入，两个旧文件完整删除，无转发兼容层。manifest/hash 和 routing 仍复用原 owner，分别留 R8-04/05。

八项原 helper、原 async workflow（除汇总提取）和报告汇总逐段去空白/格式逗号比较等价：原权重、5 场历史、65%/40% 质量阈值、Blocked→ShadowOnly→ReadyWithWarnings→FormalReady 优先级、两种许可、原因排序/去重、check/manifest 字段及错误语义不变。窗口/准备 InvalidState 和缺失路由 NotFound 继续生成阻断报告；其他 Port 错误立即原样返回，无重试。仅 chain.ready_for_model 时准备输入，窗口和输入复用同一个评估时间。

## 新增文件

- `crates/application/src/use_cases/prediction/readiness/mod.rs`：模块出口。
- `crates/application/src/use_cases/prediction/readiness/workflow.rs`：只读审计编排及报告组装。
- `crates/application/src/use_cases/prediction/readiness/lineups.rs`：阵容/门将/首发上下文检查。
- `crates/application/src/use_cases/prediction/readiness/input_quality.rs`：历史覆盖和质量检查。
- `crates/application/src/use_cases/prediction/readiness/report.rs`：检查载荷、等级/评分/原因归类。
- `crates/application/src/use_cases/prediction/readiness/tests.rs`：原 Application target 内的八项行为测试。
- `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-02-historical-feature-reader.md`：同轮 02 收尾记录。

## 修改文件

- `crates/application/src/services/prediction/service.rs`：公开审计直接调用唯一 owner。
- `crates/application/src/use_cases/prediction/build_input/mod.rs`：使用同一审计入口。
- `crates/application/src/use_cases/prediction/mod.rs`、`shared/mod.rs`：登记新模块，移除旧登记。
- `crates/application/src/use_cases/prediction/tests.rs`：原 Probe 增加所选比赛/链读取、请求记录、scope 和错误种类注入，未选中 Port 继续拒绝。
- `scripts/verify-prediction-service.mjs`：唯一 owner、时钟/顺序/异常、纯检查、评分/许可及测试门禁。
- `scripts/verify-player-role-inheritance.mjs`：只更新真实阵容检查 owner 路径。
- `architecture/application-port-inventory.json`：Application 扫描 377→381，43 traits/Port SHA 保持。
- `architecture/domain-type-inventory.json`：使用方和扫描 1033→1037，365/300 及声明摘要保持。
- 根 `README.md`、`docs/TESTING.md`、R8 任务书、阶段 `README.md`：实施、验证与边界。

以上为实施提交 22 文件完整清单。本完成记录本轮新增，相关文档同步收尾；同轮 R8-04 另行实施并独立接受 Windows CI。

## 移动或重命名文件

无完整文件移动/重命名；原函数按责任提取。

## 删除文件

- `crates/application/src/use_cases/prediction/inspect_match_prediction_readiness/mod.rs`。
- `crates/application/src/use_cases/prediction/shared/readiness_checks.rs`。

旧审计/检查 owner 和登记均清理，未保留空壳或双实现。

## 兼容、依赖与生命周期

公共 API/DTO/schema、171 命令、43 Ports、365 Domain/300 映射、配置、日志/UI、依赖/锁文件、模型算法/参数/保护资产及 0001～0046 保持。核心只依赖原 Ports、Domain/DTO、registry/model-api 和原 audit/routing；没有具体 Persistence/SQL。局部状态随一次 async 请求结束释放，取消由既有调用方管理；无后台任务、缓存、监听器、共享 State、时钟重取、模型执行或历史保存。P4.4/provider 策略保持；fake registry 只验证编排。

## 实际验证与精确 Windows 证据

- 83 项既有源码检查、完整 `npm run verify:architecture`、Rustfmt、`git diff --check` PASS；18 保护指纹/私有资产缺席、171 命令、46 连续迁移及 18 原 PG 契约静态基线 PASS。
- 六项破坏探针（时钟重取、selected fallback、历史阈值、影子质量阈值、评分上限、吞 Port 错误）均拒绝并恢复。
- 精确 [Windows run 36902069546](https://github.com/uniquenesssta/123/actions/runs/36902069546)，HEAD 为上述 `47ba3de`，job `110503393919` 全 SUCCESS。完成时间 2026-10-02 02:16:52（北京时间）。本轮核实精确 SHA、全部步骤、日志和 artifact。
- Application **69**（61+8），新增八项审计测试全部 ok；Persistence **141**、**17** 视口、前端类型/生产构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 和启动 **7 条记录 / 3 个完成操作**实际通过。
- artifact `11184750052`，14,023,532 字节，SHA-256 `1fe676d3e63a36507a0914e3f2339fc12f1aa25d68311dece17b1b0fc88a00f2`。

## 延期、风险和回退

数据库 contracts 和 18 broad PG 仍 ignored；真实 cutoff/历史四项/账本/并发回滚、有效 XLSX、Windows Full、私有 P4/P7 Golden Master 继续最终封包新库待验。公开 unavailable stub 不冒充私有提供器。继承的比赛删除与 model.runs/0041 不可变 trigger 风险仍开放，须新库真实 run fixture 验证保护/拒绝。

没有 Linux/macOS 动态验收、新 runner/workflow/target/数据库/依赖/迁移。受控 revert 到已验证 `cc2b0fe`，同步 owner/入口/门禁/清单，不复制旧实现或修改历史数据。03 可关闭，04 按用户指令开始；延期项不计通过。
