# R8-12 Freeze Transaction 实施记录

状态：`VERIFYING`。2026-10-09，用户授权“收尾11开始12”；唯一分支 `rewrite/r8-prediction-p4-orchestration`。11精确47bda231575a9179cd629367d5a3273cd2ab654a / Windows37891346613全SUCCESS，118/169、17视口和Windows交付通过；五文档收尾基线 `1b7017fabcc131ef80515b5e5c70507de45da9e0`。本项须自身精确Windows，R8仍IN_PROGRESS，不启动R9。

## 实际来源、职责与唯一入口

原Application execute_p4_freeze/mod.rs混合目录与编排；snapshot_projection混合输入身份、字段和概率投影。mod现在仅登记/re-export原execute，workflow原编排保持；input附加原p4_orchestration来源并验证四项锁定身份，features投影29事实+数据库字段+就绪度，probabilities仅读取外部矩阵/原值/标记与哈希。原Service/Facade/Port/composition链保持，不创建research/p4/freeze空转发。

原Postgres p4_records中的快照责任迁入adapters/p4/freeze_transaction：write唯一拥有冻结事务与审计；input纯前检/排序副本/原五指纹；details只借用write事务写三类明细；validation借用同一事务检查真实引用与证据截止；read读取Bundle和复用记录，SQL与Row共置。原Schema/Prompt/Profile/Research职责留原owner，原horizon解析仅扩大crate内部可见供共享，不复制parser或修改public导出。原验证状态、幂等、audit和sha256_json继续唯一复用。

## 事务、状态与异常语义

纯前检→begin→原snapshot:key事务锁→同键指纹/正式精确队列复用→数据库引用与证据检查→快照头→31字段及证据链接→外部概率→原prematch_snapshot_frozen审计→commit，全部在原同一事务。两个复用出口只提交读取，不重复写明细/审计。不同幂等键并发仍遵守原正式队列唯一约束，不新增锁或错误恢复政策。

模型运行、快照、任务状态分别沿原Port事务。快照commit之后才迁FREEZING→FROZEN；其间失败返回原错误，重试先查已提交快照再合法登记状态，不重跑模型。终态只读返回；提前领取拒绝、超时MISSED、未就绪BLOCKED、RESEARCH_SUCCEEDED恢复、READY_TO_FREEZE进入FREEZING及全部中文提示保持。两次Utc分别在前检及模型/Schema结束捕获，保留截止前与宽限后严格比较；不会用更晚事实补冻。

31字段键/序号、多route原序/原值/来源、证据排序去重及状态聚合保持。概率拓扑归provider，至少一链，不强迫四链或计算概率；formal默认full且允许显式覆盖，clean-sheet仅formal附加，矩阵哈希来自原JSON字节。Draft前检保留1e-9概率和/有限值、小写64位哈希/u16/31序号/trim唯一、原240字节幂等键和正式时点。全局证据集合哈希去重，原feature.evidence_ids列表保持，重复链接继续由原SQL约束拒绝并整事务回滚。

原输入纳秒时间参与指纹，引用比较与返回时间按SQLx/PostgreSQL实际精度；首次与重试读回一致。指纹原排除frozen_at/交付key/trace/research/quality/根metadata，不擅自增加身份字段；14类原纳入身份/载荷变化仍改变指纹。数据库引用/trace检查只在原首建路径，复用仍在其前，无新增校验优先级。读仍使用原局部事务/隔离，失败/取消沿原SQLx drop回滚，不承诺新增跨Port原子或读隔离。

## 完整变更清单

### 新增（11）

- `crates/application/src/use_cases/prediction/execute_p4_freeze/features.rs`
- `crates/application/src/use_cases/prediction/execute_p4_freeze/input.rs`
- `crates/application/src/use_cases/prediction/execute_p4_freeze/probabilities.rs`
- `crates/application/src/use_cases/prediction/execute_p4_freeze/workflow.rs`
- `crates/persistence-postgres/src/adapters/p4/freeze_transaction/details.rs`
- `crates/persistence-postgres/src/adapters/p4/freeze_transaction/input.rs`
- `crates/persistence-postgres/src/adapters/p4/freeze_transaction/mod.rs`
- `crates/persistence-postgres/src/adapters/p4/freeze_transaction/read.rs`
- `crates/persistence-postgres/src/adapters/p4/freeze_transaction/validation.rs`
- `crates/persistence-postgres/src/adapters/p4/freeze_transaction/write.rs`
- `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-12-freeze-transaction.md`

### 修改（14）

- `README.md`
- `architecture/application-port-inventory.json`
- `architecture/database-baseline.json`
- `architecture/domain-type-inventory.json`
- `crates/application/src/services/p4_orchestration/tests.rs`
- `crates/application/src/use_cases/prediction/execute_p4_freeze/mod.rs`
- `crates/application/src/use_cases/prediction/tests.rs`
- `crates/persistence-postgres/src/adapters/p4/mod.rs`
- `crates/persistence-postgres/src/p4_records.rs`
- `crates/persistence-postgres/tests/postgres_integration.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md`
- `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md`
- `scripts/verify-prediction-service.mjs`

### 整文件删除（1）

- `crates/application/src/use_cases/prediction/execute_p4_freeze/snapshot_projection.rs`

### 移动/重命名

无整文件移动/重命名。函数按职责迁移，A/M/D采用实际 `git diff --no-renames --name-status` 与新增文件清单。

## 契约与保护范围

API/DTO/serde/Schema/数据格式/配置/错误类型和提示/日志/UI、路由/cutoff/哈希、P4.4 SHADOW_ONLY与P7固定回归保持。43Ports/171命令/365Domain/300映射、18保护资产、0001～0046迁移、依赖/锁文件均保持，无新public类型/Port、workflow/runner/target/数据库或持续回归体系。未执行Linux/macOS Cargo、构建或客户端动态。

## 原测试、验证与延期

新增原Application target八项：完整冻结顺序与全部锁定身份/溯源/31字段/质量分数/外部单链，七个末段Port边界×六种kind/message停止，四路由身份/Schema/空矩阵/非法概率/空比分漂移阻断，快照已提交而FROZEN登记失败后的只读恢复；纯投影另覆盖原路由顺序/原值/证据排序去重与CONFLICT/STALE、外部拓扑/正式覆盖/clean-sheet/矩阵原字节哈希、矩阵缺失/非法/0和65536比分边界。四项原快照Persistence测试原样迁入input，新增六项前检/指纹测试：交付元数据原排除、14类不可变身份/原载荷变化、只排序副本而保留原键/证据列表、完整31序号/trim唯一、有限概率/1e-9和/小写64哈希/optional/u16、正式时点/截止相等及240字节键。源码预期Application **126**（118+8）、Persistence **175**（169+6），须本项Windows实跑。

原PG Stage C在同一TestDatabase/版本/研究/证据/快照夹具上扩重复证据链接SQL失败与最后概率JSON零字符SQL失败，核对快照头/字段/证据/概率/审计完全无残留；同键并发仅一个created、一份31字段/证据/四概率/审计；原顺序翻转与frozen_at变化重试、异键同正式队列复用/原幂等键保留、不同载荷拒绝不写账本，完整原JSON/字段/排序概率/正式与shadow flags读回。原published/effective截止+1微秒、研究截止差异与不可变更新/删除断言保留。18 broad仍ignored，无新trigger/Schema/target/runner/数据库；真实PG实跑最终新库待验。

83原源码/完整architecture、Prediction/Domain/源码卫生、18保护资产/171命令/46迁移/18PG静态契约、Rustfmt1.88 --check和diff均PASS。七探针：脱离明细、pool写、证据截止变包含边界、删除完成时钟、Schema/路由身份绕过、正式标记改变，全部拒绝并恢复。等价核对：两编排函数/七投影函数、26原Postgres函数逐token保持；拆出明细重内联后原快照事务保持；260条SQL/消息/哈希/审计literal与原八测试/fixture函数保持。

报告 `/workspace/scratch/eb298ad5cdcb/r812-static-checks.json`、`r812-architecture.log`、`r812-negative-probes.json`、`r812-equivalence.json`。Domain扫描1090→1099、usageDigest `cf79a5f2a44e5cae6a3ad16545150b6c32db7faae8c3693217956a993d7ea53c`，365/300/sourceDigest不变；Application399→402、43Ports和Postgres根35保持。PG runtime_sources仅原测试blob更新为 `33e58c3275014c1adf99f94a2f938c2dbff1fa75`。无依赖/API升级、迁移或数据库基础设施变化；不执行Linux/macOS动态。原Windows Automated确认启动后停止轮询；12 VERIFYING，R8 IN_PROGRESS、R9不开始。原真实PG/历史四项/账本/有效XLSX/Windows Full/私有固定回归及继承model.runs/0041删除风险继续最终新库待验。

## 发现、订正与计划偏差

最初门禁仍读取旧snapshot_projection/p4_records快照位置，已将原时间/账本验证状态检查绑定实际新owner；追加事务/流程边界检查。格式化使原共享幂等import折行，改为准确匹配原三个名字；冻结顺序检查区分恢复段FREEZING校验和正常进入段，避免误认同名条件。无业务或错误语义变化。

数据库清单初次重写导致无关JSON排版展开，已恢复原排版，仅更新原PG测试blob。单次验证误用了连字符的数据库脚本名，改用原verify_database_baseline.mjs后PASS；没有创建新脚本。新增测试的错误枚举已对照原PortErrorKind/PersistenceError实定义修正，避免以泛化名代替原契约。Context7无实际第三方API/版本变更或已知编译问题，现有API沿用；动态编译以本项Windows为准。

## Windows交付与继续位置

推送本项到原R8分支，复用原Public Platform CI/Windows Automated；确认新head/Windows job启动后停止轮询，不继承11 SUCCESS或预期126/175。12保持VERIFYING，全R8未实际完成前不创建阶段完成记录/启动R9。Mermaid Chart已展示同事务首建/复用/回滚及独立FROZEN登记恢复；Create State只保存足球模型96f03270-c236-46de-99fa-db85d2fbf4ce（uniquenesssta/123，0.23.0），不替代Git。

## 回退与真实风险

回退基线1b7017fabcc131ef80515b5e5c70507de45da9e0；受控revert并同步唯一owner/原测试/门禁/清单/记录，不复制旧实现或变更历史数据库。原不同键同正式队列并发可能返回唯一约束SQL错误、模型运行已保存后快照失败仍可能保留原run，均为保留的原边界，不新增跨事务修复。真实PG/历史四项/账本/有效XLSX/Windows Full/私有P4/P7 Golden Master及继承model.runs/0041删除风险继续最终封包新库待验，公开stub/ignored编译/静态保护指纹不能替代实跑。
