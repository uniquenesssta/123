# R10-07 Provider Score

状态：`VERIFYING`。用户2026-10-13“收尾06开始07”授权；uniquenesssta/123 v0.23.0，唯一rewrite/r10-review-analytics-release-backend，基线85ecf15e0b9f75e5f02ccf73576f6430f5c2f7c9。01～06自身精确Windows DONE；08～13 BLOCKED，R10 IN_PROGRESS。

### 1. 目标

把原供应商证据聚合、评分与快照读写收拢到唯一实际模块，保持同一正式分区、人工决定来源、历史快照和监控顺序。

### 2. 现状与来源

原PG root postmatch.rs实际持有两供应商方法、mapper、ProviderAggregate/total与mean。Application只有综合refresh_postmatch_monitoring/postmatch_overview及PostmatchMonitoringPort，没有独立Provider Score入口；任务书App目标目录为参考，不为满足文件路径新增Port/DTO/空转发或重复公式。原活动分区/漂移/总览属于跨供应商与08的原共享编排，本项不提前重写08。

### 3. 目标文件与目录

crates/persistence-postgres/src/adapters/postmatch/provider_score/{mod,aggregation,repository,read,mapping,tests}.rs；原postmatch.rs继续活动分区/漂移/总览/共享纯支持。Rust snake_case职责目录符合现有crate，不新crate或通用框架。

### 4. 文件职责边界

mod仅登记。aggregation持有原临时ProviderAggregate/total/mean、原BTreeMap逐行解码聚合、scores与snapshot_identity；repository持有原五分区查询和逐供应商快照写入；read持有原最新快照查询；mapping逐字段解码原Domain记录；tests只cfg(test)边界行为。不是把整原root文件搬走，剩余10真实函数保持，原五供应商职责迁出。

### 5. 输入

原PostmatchMonitoringRequest的competition_id/horizon/baseline_size/current_size；活动绑定产生原PostmatchPartition五分区身份与partition_key；原人工决定verdict/可选accuracy/reliability/全部timeliness/按SQL顺序的decision_fingerprint；原总览limit。无新配置、请求ID、字段或自动判定。

### 6. 输出

原ProviderScoreSnapshotRecord全部字段、analytics.provider_score_snapshots原快照；原PostmatchOverview供原App返回。空查询无分组/无供应商写入；无accuracy时weighted_score=0，timeliness/reliability均值保持原输入，不凭不可验证样本伪造准确率。

### 7. 允许依赖

现有SQLx Row/pool、Uuid、Domain ProviderScoreSnapshotRecord和POSTMATCH_MONITORING_VERSION、sha256_json、PersistenceResult及root原round6/non_negative_u64/PostmatchPartition。纯均值/权重/身份函数无数据库或外部模型调用；聚合行函数只解码并建立局部集合。

### 8. 禁止依赖

不引入Application→SQL/具体PG新引用、不新Provider Port/入口/通用Repository、无共享可变State/全局缓存/新后台任务；不调用提供器预测或改P4/P7/参数/Profile/Schema/Golden，不自动决定证据或晋升参数。供应商链不添加事务/审计/覆盖更新。

### 9. 状态所有权

原快照存储及snapshot_key唯一约束保留；ProviderAggregate为每次刷新内部局部BTreeMap/Vec，不是长期State。PG直接owner37/state19、App420/43Ports/19Port文件保持。原PostmatchPartition仍root唯一owner，结构/字段仅pub(crate)支持原root与内部适配器跨模块调用；外部公共API不扩展。

### 10. 副作用边界

原监控先时点/窗口前检→活动绑定查询→供应商→漂移→overview。供应商先pool读取，按provider_id与decision.id原SQL升序、BTreeMap provider顺序聚合；每组先均值/加权/指纹/键，再原20字段INSERT ON CONFLICT(snapshot_key) DO NOTHING并execute(&self.pool)。没有原不存在的统一begin/commit/rollback或新增审计，某后组失败前组已完成写入不能声称一并回滚。

### 11. 异常路径

原SQL/Row/序列化/InvalidState错误与负数计数标签逐字保持。未知verdict仍计not_verifiable；可选accuracy/reliability按原Some样本独立分母，timeliness包含每行，原错误读取先后不改。读取或解码/哈希/写入失败直接停止后续漂移和总览；漂移/overview失败不会撤销之前pool写入。

### 12. 并发/异步/生命周期

原snapshot_key含供应商/partition_key/有序指纹SHA/POSTMATCH_MONITORING_VERSION，ON CONFLICT DO NOTHING保持幂等身份；不重排/去重指纹、追加锁/重试/取消/超时/并发任务。原DISTINCT ON(provider_id,scope_key)以generated_at DESC,id DESC取最新，limit仍clamp(1,500)。真实并发重试、不可变保护、部分写入后失败及读回仍最终新库待验。

### 13. 兼容要求

原App/7公共命令/facade/Service/两Ports/composition、公开PostgresStore方法签名与Domain记录、全部SQL/绑定/错误/JSON/日志/UI/配置/依赖/迁移/工作流/版本0.23.0保持。仅两个原私有供应商方法跨实际模块改pub(crate)；分区内部可见性同步。原均值未先round，weighted=round6(accuracy*.60+timeliness*.15+reliability*.25)，插入时三均值分别round6。

### 14. 实施步骤

提取原供应商query/write到repository、read和mapper；原BTreeMap逐行循环提取为aggregate_provider_rows，原评分与身份段提取为scores/snapshot_identity，逆内联精确核对。原root删除五职责及供应商struct/import、登记新module；原共享partition仅crate可见。原两个源码门禁更新实际路径并补原政策，Domain使用摘要更新，新增9原PG unit；完成原静态/负向/文档后推送原Windows。

### 15. 切换入口

原root self.refresh_provider_scores_for_partition与self.list_provider_scores同名方法由新repository/read实际实现；没有保留root空壳或重复函数。repository调用实际aggregation，read复用同一mapper；原App无需切新接口，原Ports仍进入原监控/总览。

### 16. 删除清单

从root移除refresh_provider_scores_for_partition/list_provider_scores/provider_score_from_row、ProviderAggregate/total/mean与相关BTreeMap/ProviderScore import；root文件保留10真实监控/漂移/共享函数。没有整文件删除或重命名，实际0D；不删08活动分区/漂移实现或06/05已验模块。

### 17. 最小验证

实际84/84既有JS源码门禁、完整npm run verify:architecture、8实际Rust文件独立Rustfmt1.88 --check、14等价核对、10负向拒绝且文件字节恢复、diff/23字段/实际A/M/D/链接核对通过。新增9PG unit须自身Windows实跑，预期216=207+9，App165/Gateway112+41不变；本地不运行非Windows Cargo/check/Clippy/build/test/loopback/browser。

### 18. 阶段回归

沿用原Public Platform CI Windows Automated/原workspace unit和contract targets执行前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace tests、Windows release/MSI/NSIS/启动/运行日志。原PGintegration与18静态契约保持，没有新target/runner/workflow/专项库；ignored/公开unavailable stub不计实库或私有模型PASS。

### 19. 失败停止条件

07自身精确Windows完整SUCCESS前VERIFYING、08～13 BLOCKED；失败只修同一R10分支，不借06成功、不跳测试或压警告。原Windows job启动后停止CI查询，提示约20～30分钟；不提前记录R10阶段完成或启动08。

### 20. 回退点

85ecf15e0b9f75e5f02ccf73576f6430f5c2f7c9（06纯文档收尾，源码同1ed6dbe/Windows run38290328302全SUCCESS）。受控revert07源码/门禁/使用摘要/文档，不复制旧函数保留双owner。

### 21. 根README摘要记录

根README记录06DONE、07实际供应商owner/保留政策与新增9unit/验证待自身Windows、08～13BLOCKED，链接本节点/阶段索引；完整公式/风险/证据在此处。

### 22. docs阶段详细记录

本23字段与全A/M/D、真实入口/私有可见性取舍、原公式/SQL/事务边界、逆内联/单位测试/负向探针/风险/回退；阶段索引/R10任务书/TESTING同步。06另5M[skip ci]收尾85ecf15，07仅从该基线实施，保留实施历史。

### 23. 完成标准

供应商唯一owner/实际入口/旧职责清理与源码门禁通过；新增9unit及07自身精确Windows完整SUCCESS后才可DONE。真实PG、历史保护四项/不可变账本/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；静态指纹不计这些实际验收PASS。

## 等价与测试证据

14等价核对全通过：原五供应商职责迁出；root剩余十函数正文精确相同；read/mapper/total/mean原正文相同；BTreeMap聚合/各Row getter顺序相同；纯scores逆替self与原未舍入均值、空accuracy零分及权重精确相同；snapshot_identity与原有序指纹/格式/版本相同；repository逆内联三处与原方法精确一致；生产SQL/错误/JSON字面量多重集合相同；两struct字段相同、仅内部可见性；新供应商方法仅crate可见；外部公共PG签名相同；App/Ports/05/06/Review/DB/其余引擎/models/UI/contracts/deps/workflow/target/state冻结；Domain365声明/129文件/301映射保持；9新unit注册在原PG target。

新增9PG单测：空行不创建供应商组；空aggregate计数/均值/分数零；不可验证样本保留时效/可靠度但没有accuracy时weighted=0；混合样本独立分母；三个均值保留未舍入精度后weighted六位舍入（该fixture若先round均值将错为0.264197，原正确0.264198）；缺时效/可靠度原零默认；sample_size仅四verdict计数、不用评分Vec长度；有序指纹及重复指纹精确SHA/key fixture，重排/去重改身份；同输入重试同key、供应商或分区改变仅key身份隔离。预期PG216=207+9/App165/Gateway112unit+41contract，待自身Windows实际执行，不记本地动态PASS。无需为当前没有独立供应商App入口增加假的测试接口。

10负向探针均拒绝且全部文件字节恢复：改变权重、不可验证样本伪造weighted=1、哈希前sort、取消horizon过滤、决定排序加DESC、快照DO NOTHING改覆盖UPDATE、增加供应商事务、丢计数负数门禁、limit上限变更、逆供应商/漂移顺序。首次排序探针暴露前缀token无法拦DESC，补精确原ORDER BY行检查后10/10重跑拒绝，生产源码完整恢复；不是Windows失败记录。两既有Postmatch门禁继续保留旧全部检查，读取实际新owner并补原分区/公式/顺序/幂等/无事务/原unit检查，无新检查框架。

实际命令：原84JS、npm run verify:architecture；generate-domain-type-inventory；verify-postmatch-service/verify-postmatch-settlement；原保护资产/171命令/数据库基线；8实际Rust文件独立rustfmt1.88 --edition2021 --config skip_children=true --check；原逐函数/三处逆内联/字面量/公开签名/struct字段/冻结链、10负向、git diff --check/23字段/清单/链接。scratch r1007-static-checks.json、r1007-architecture.log、r1007-equivalence.json、r1007-negative-probes.json可重建，正式结论在此保留。

Domain365/129/sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd、301映射分类保持；scan1217，rustUsageDigestd5856f6a851ac8fb95e9dcb3ed87ba63e9b53b1da405f6c2d6f65d84f753f56c；App420/43Ports/19Port文件、PG直接owner37/state19保持。18保护聚合d74e0936b60c69f444a498405fed3e704b8db63b81f26b40036f772b4b6eac57、171命令、46迁移/18PG静态聚合d9f2eb50bacd747b7cbf08492189c2635b7c0ec2cf4c764def1d32a837f8ba93保持。Application源码未变无需重刷其Port扫描。

## 实际供应商与监控链路

```mermaid
flowchart TD
 A["原监控入口：时点与窗口前检"] --> B["读取活动模型分区"]
 B --> C["按五个分区条件读取人工证据决定"]
 C --> D["按供应商与决定 ID 原顺序聚合"]
 D --> E["可用准确率、可靠度和全部时效的原均值"]
 E --> F{"存在准确率样本"}
 F -->|否| G["加权分数为零"]
 F -->|是| H["原 60/15/25 权重与六位舍入"]
 G --> I["原有序指纹及供应商、分区、版本快照键"]
 H --> I
 I --> J["逐供应商使用连接池幂等插入快照"]
 J -->|全部成功| K["继续原漂移刷新"]
 K --> L["原总览读取最新快照并映射字段"]
 B -->|失败| X["返回原错误，停止后续"]
 C -->|读取或解码失败| X
 I -->|序列化失败| X
 J -->|写入失败| X
 K -->|失败| X
 L -->|读取或解码失败| X
```

逐供应商pool写入无统一事务，后组/漂移/总览失败不撤销已写快照；纯均值/加权无Result，图只标真实错误出口。快照幂等/并发/历史保护须实库新库验收，源码保持不等于动态PASS。

## 工具与来源

附件按用户要求读取scratch既有复制，SHA与root AGENTS保持；skil main0.0.1/AGENTS.md sha5cc6750404092a080ddcd18685a25dfb90d3f809核对；总纲/07全部字段与实际source调用已读取。GitHub核对06精确Windows/log/artifact后5文档收尾85ecf15[skip ci]；Mermaid Chart成功显示本供应商真实链；Create State按稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce保存交接。复用既有Rust/SQLx/哈希接口，无新API或版本疑问不重复Context7；无子代理。无新增依赖/迁移/runner/target/workflow/DB专项、第二分支或R10阶段完成记录。

## 实际完整 A/M/D 清单

相对已验06收尾基线 85ecf15e0b9f75e5f02ccf73576f6430f5c2f7c9：**7A / 9M / 0D，共16路径**；无整文件移动或重命名，旧root职责按函数迁出。

| 类型 | 文件 | 实际职责/原因 |
| --- | --- | --- |
| M | `README.md` | 根摘要/当前状态与07链接 |
| M | `architecture/domain-type-inventory.json` | 实际Domain使用位置和扫描摘要 |
| M | `crates/persistence-postgres/src/adapters/postmatch/mod.rs` | 登记唯一provider_score模块 |
| A | `crates/persistence-postgres/src/adapters/postmatch/provider_score/aggregation.rs` | 原局部聚合/计数/均值/加权/指纹身份唯一owner |
| A | `crates/persistence-postgres/src/adapters/postmatch/provider_score/mapping.rs` | 原完整DTO/计数负数解码 |
| A | `crates/persistence-postgres/src/adapters/postmatch/provider_score/mod.rs` | 仅登记评分职责与cfg(test)原unit |
| A | `crates/persistence-postgres/src/adapters/postmatch/provider_score/read.rs` | 原最新快照查询/限制/复用mapper |
| A | `crates/persistence-postgres/src/adapters/postmatch/provider_score/repository.rs` | 原分区查询与逐组pool幂等写入 |
| A | `crates/persistence-postgres/src/adapters/postmatch/provider_score/tests.rs` | 九项原PG target边界行为测试 |
| M | `crates/persistence-postgres/src/postmatch.rs` | 移除五供应商职责；原编排保留；partition仅内部可见 |
| M | `docs/TESTING.md` | 原验证结果与新增unit/实库延期 |
| M | `docs/football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md` | 任务书状态与真实来源订正 |
| A | `docs/modular-rewrite/R10-review-analytics-release-backend/R10-07-provider-score.md` | 本23字段/全部清单/验证/回退 |
| M | `docs/modular-rewrite/R10-review-analytics-release-backend/README.md` | 阶段状态/07入口与继续边界 |
| M | `scripts/verify-postmatch-service.mjs` | 既有门禁切实际owner/补原政策及唯一入口 |
| M | `scripts/verify-postmatch-settlement.mjs` | 既有验证读取实际新评分owner |

职责出口、旧owner清理、源路径与所有记录随同提交，清单未用预设文件数量替代实际diff。
