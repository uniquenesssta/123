# R8-02 Historical Feature Reader：节点完成记录

状态：`DONE`（唯一历史读取责任及 Windows Automated 完成；真实 PG、私有固定回归和 Full 仍待验）。完成核实日期 2026-10-02（北京时间）。

## 基线、目标与结果

唯一分支 `rewrite/r8-prediction-p4-orchestration`；起点/回退基线 `cba72fdfaaf640a17c1e73c326536189dda74d17`。实施提交 `cc2b0fe7601efdbad44fa5badcb6507f226698a7`，树 `69d31cce568219aad5466d3c4045d10acf8dd24f`，对应 [R8 任务书](../../football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md)。

原 team_features 同时查询历史、映射数据库行并计算特征。`calculate_team_pre_match_features` 的唯一读取入口迁入 `adapters/prediction/historical_features/mod.rs`；`read.rs` 唯一持有两段 SQL、原绑定与 typed row 映射；原 `team_features.rs` 保留中性载荷、纯投影、连续曲线、衰减/置信度及 evidence/quality，承担实际职责。范围仍为同赛季→同赛事→跨赛事，36 候选、4 场阈值、过滤后最多 12 场；主客场、排序、cutoff、进球基准、评分/证据原样保留。空历史直接中性，不追加基准查询。

## 新增文件

- `crates/persistence-postgres/src/adapters/prediction/mod.rs`。
- `crates/persistence-postgres/src/adapters/prediction/historical_features/mod.rs`：只读协调、范围选择与原目标内两项测试。
- `crates/persistence-postgres/src/adapters/prediction/historical_features/read.rs`：原 SQL、typed rows/映射与两项测试。
- `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-01-match-prediction-input-builder.md`：同轮 01 收尾记录。

## 修改文件

- `crates/persistence-postgres/src/adapters/mod.rs`：登记读取目录。
- `crates/persistence-postgres/src/team_features.rs`：移除 I/O，保留原纯计算和三项曲线测试，增加两项平台投影/中性测试。
- `crates/persistence-postgres/tests/postgres_integration.rs`：在原 cutoff target 增加相等、前后一微秒、finalized/created 各自未来隔离及 baseline 守卫。
- `scripts/verify-prediction-service.mjs`：原验证器增加历史唯一 owner、时间/顺序/范围/投影和测试门禁。
- `architecture/database-baseline.json`：只刷新原 PG 测试 blob。
- `architecture/domain-type-inventory.json`：使用方/扫描计数 1030→1033，365/300 及声明摘要保持。
- `architecture/module-boundaries.json`：prediction 直接 PG 模块登记，35→36。
- 根 `README.md`、`docs/TESTING.md`、R8 任务书、阶段 `README.md`：实施、验证与实际边界。

以上为实施提交 15 文件完整清单。本完成记录本轮新增，相关文档同步收尾；R8-03 源码另行实施并独立接受 Windows CI。

## 移动或重命名文件

无。

## 删除文件

无整个文件删除。原 team_features 中迁出的 SQL/读取/范围选择实现已清理，无重复 owner 或空转发。

## 兼容与依赖

Application/PredictionInputPort、match_prediction、R6 球员贡献 owner 无改动，不增加 Application historical wrapper 或公共样本类型。公共 API/DTO/schema、43 Ports、171 命令、365 Domain/300 映射、配置、错误/日志/UI、依赖/锁文件、模型算法/参数/保护资产和 0001～0046 均保持。读取仍通过原 PostgresStore 方法接入原输入构建；纯投影只接收私有样本。原两次只读 SQL 没有跨查询事务快照保证，本节点不增加隔离承诺。

## 实际验证与 Windows 证据

- 83 项既有前端源码检查、完整 `npm run verify:architecture`、Rustfmt、`git diff --check` PASS。六项破坏探针（入库 cutoff、基准相等、排序、样本上限、置信度系数、投影 I/O）均拒绝并恢复。
- 原 SQL literal/bind、scope/neutral/有限值 helper、曲线/三测试及加权投影逐段比较保持；仅解耦纯函数的 Result 包装。
- 18 保护指纹/私有资产缺席、171 命令、46 连续迁移及 18 原 PG 契约静态基线 PASS，不等同真实数据库执行。
- 精确 [Windows run 36891488571](https://github.com/uniquenesssta/123/actions/runs/36891488571)，HEAD 为上述 `cc2b0fe`，job `110467936302` 全 SUCCESS，2026-10-02 00:50:29（北京时间）completed/success。本轮核实 SHA、全部步骤、日志和 artifact。
- Application **61**、Persistence **141**（原 135+6 新增，原三曲线测试保留），6 项新测试均 ok；**17** 视口、前端类型/生产构建、Rust fmt/Clippy/workspace tests、Windows release/MSI/NSIS 及启动 **7 条记录 / 3 个完成操作**实际通过。
- artifact `11178738200`，14,023,805 字节，SHA-256 `60662e5193de93cbef0da47ea1e5299dfab8954379de5d7d4a50d3c31ad33ed6`。

## 延期、风险和回退

18 broad PG 和数据库 contracts 仍 ignored；真实 cutoff/历史四项/账本/并发/回滚、有效 XLSX、Windows Full 沿用用户最终封包新库验收，不创建数据库或专项体系。公开平台投影测试不冒充私有 P4/P7 固定概率 Golden Master。继承的比赛删除与 model.runs 历史不可变 trigger 风险仍开放，须最终新库真实 run fixture 验证保护/拒绝。

只使用 Windows 动态验收，未运行 Linux/macOS Cargo 或客户端验证；没有新增 runner/workflow/target/依赖/迁移。受控 revert 本节点源码回到 `cba72fd`，同步 owner/门禁/清单，不复制旧实现，不改历史数据。02 可关闭，03 按用户指令开始，但上述动态待验项不计通过。
