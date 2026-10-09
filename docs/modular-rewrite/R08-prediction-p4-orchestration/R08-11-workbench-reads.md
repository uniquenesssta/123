# R8-11 Workbench Reads 实施记录

状态：`VERIFYING`。2026-10-09，用户授权“收尾10 开始11”；唯一分支 `rewrite/r8-prediction-p4-orchestration`。10修复精确 `481bfcb967349afedbf7eac44959f2f8a020744b` / [Windows run `37886029199`](https://github.com/uniquenesssta/123/actions/runs/37886029199) 全SUCCESS，114/169、21新增测试、17视口及Windows交付通过；五文档收尾基线 `2770adca714fa471e3d801ecd3f541f0f6a05db4`。11须自身精确Windows CI，12 BLOCKED。

## 实际职责与设计

原Postgres p4_workbench混合比赛/任务读与人工裁决事务。读取迁入workbench目录：比赛查询/赛事与列表，任务读取编排，研究运行元数据，证据来源/时间投影，冲突成员与最新事件/评估/任务人工覆盖。SQL与紧密关联的Row投影共置；mod仅显式登记。原人工裁决writer及校验、事件、锁、Row helper五函数不动，根readiness/routed_facts、原快照读取继续复用，Freeze Transaction留12。

两个Application读取用例本身已满足独立Port职责，保持生产实现，仅补原target测试，不创建research/p4/workbench空转发目录。原Tauri命令→Facade→PredictionService→用例→PredictionWorkflowPort→composition adapter→同名public Store方法保持；方法定义现在唯一位于matches/tasks。

## 行为、事务与异常

比赛精确ID/LEFT JOIN可空赛事/“比赛不存在”提示/原任务100条上限保持。任务严格task→readiness→events→routes→research→evidence→conflicts→snapshot，任何错误原样提前停止，无部分成功/吞错/重试。无研究时元数据None、证据/冲突为空；无快照None；Row原全部字段、NULL和数组顺序保持。

证据按当前run筛选和field/created/id排序；冲突成员仅当前run，评估按当前run，人工裁决按当前task+conflict，事件状态仍是冲突全局最新值。三个最新读取保留时间DESC/idDESC；没有人工裁决的selected_evidence_ids从NULL变原空数组。原routes的task覆盖政策由原p4_routed_facts承担，不复制写入或裁决计算。

四SQL literal逐字保留；五writer逐token、比赛方法、两个Application生产入口保持，三个查询/Row和任务汇总在明确参数/Result返回提取后重内联等价。生产新代码没有INSERT/UPDATE/DELETE、事务、审计/任务/队列/模型操作。原读仍是独立pool请求，未新增跨查询一致快照；取消/UI过期结果归实际调用方，不套监听器生命周期模板。

## 完整变更清单

### 新增（7）

- `crates/persistence-postgres/src/adapters/p4/workbench/conflicts.rs`
- `crates/persistence-postgres/src/adapters/p4/workbench/evidence.rs`
- `crates/persistence-postgres/src/adapters/p4/workbench/matches.rs`
- `crates/persistence-postgres/src/adapters/p4/workbench/mod.rs`
- `crates/persistence-postgres/src/adapters/p4/workbench/research.rs`
- `crates/persistence-postgres/src/adapters/p4/workbench/tasks.rs`
- `docs/modular-rewrite/R08-prediction-p4-orchestration/R08-11-workbench-reads.md`

### 修改（14）

- `README.md`
- `architecture/database-baseline.json`
- `architecture/domain-type-inventory.json`
- `crates/application/src/services/p4_orchestration/tests.rs`
- `crates/application/src/use_cases/prediction/read_p4_match_workspace/mod.rs`
- `crates/application/src/use_cases/prediction/read_p4_task_workspace/mod.rs`
- `crates/application/src/use_cases/prediction/tests.rs`
- `crates/persistence-postgres/src/adapters/p4/mod.rs`
- `crates/persistence-postgres/src/p4_workbench.rs`
- `crates/persistence-postgres/tests/postgres_integration.rs`
- `docs/TESTING.md`
- `docs/football-model-platform-modular-rewrite-19-docs/08-R8-prediction-p4-orchestration.md`
- `docs/modular-rewrite/R08-prediction-p4-orchestration/README.md`
- `scripts/verify-prediction-service.mjs`

### 移动/重命名、整文件删除

无。旧根读取函数及其专用导入退出，根文件仍持人工writer。

## 契约及保护范围

API/DTO/serde/Schema、数据格式、配置、错误类型/用户提示、日志、UI及模型算法/参数/资产、P4.4 SHADOW_ONLY、P7固定回归、cutoff/指纹/路由均保持。43 Ports/171命令/365 Domain/300映射、Cargo/package依赖与锁文件、0001～0046迁移保持。无新的public类型或端口、runner、workflow、target、数据库或持续回归体系；本轮未执行Linux/macOS Cargo、构建或客户端动态验收。

## 原测试、静态验证与延期

新增四项Application工作台委托测试：精确身份与DTO/顺序、可空赛事，空与丰富的研究/来源/冲突/路由/事件和进展/终态视图，两个读取边界的全部六种PortErrorKind/message原样停止，未选Port调用立即失败且无任务/队列写入。预期Application **118**（10已验114+4）/Persistence **169**不变，本项Windows尚未实跑，不冒充PASS。

原PG Stage C复用同一TestDatabase/已注册版本、原31字段/概率快照，补比赛/任务缺失提示、NULL赛事和空研究集合、证据17字段/微秒时间/NULL来源、三字段排序、其他run证据/冲突排除、事件原序、人工路由与任务隔离、最新全局冲突事件、完整快照及研究次数/响应/模型/错误/起止时间；重复和未知ID读取保持12个任务/队列/研究/证据/快照/审计账本计数。人工夹具遵守0018的未来cutoff、RESEARCH_PARTIAL、共享trace、manual_required及唯一(task, conflict)，两个任务各有一个不可变决策；未禁用trigger或修改迁移。18 broad仍ignored，真实执行最终新库待验。

83项原前端源码检查、完整 `npm run verify:architecture`、Prediction/Domain/源码卫生、保护18指纹（聚合d74e0936…）、171命令、46迁移/18PG静态契约（聚合d9f2eb50…）、Rustfmt1.88 `--check` 与 `git diff --check`均PASS。七探针：任务上限、就绪度顺序、证据run过滤、人工task范围、最新事件ID排序、NULL证据数组、writer导入，均被原Prediction门禁拒绝且恢复。提取核对确认四SQL逐字、五writer/比赛函数/两Application生产入口与原逻辑等价，三个查询投影和任务汇总重内联保持。

报告 `/workspace/scratch/eb298ad5cdcb/r811-static-checks.json`、`r811-architecture.log`、`r811-negative-probes.json`、`r811-equivalence.json`。Domain使用扫描1084→1090，365/300/sourceDigest保持，usageDigest `f0d81ca6c8f8d56746b800dea3e4b9d44680643f584db3ceb248add79d759751`；Application399/43Ports与Postgres根35保持，PG runtime_sources只刷新原测试blob `d422b3ac0fd7acbc17794e1edc2f2c00f717f523`。无依赖/API升级、target/runner/workflow/数据库新增，无Linux/macOS动态验收。原Windows Automated执行完整frontend/类型/构建/17视口、Rust/Clippy/workspace tests及Windows交付；确认启动后停止轮询。11 VERIFYING，12 BLOCKED；真实PG/历史四项/账本/XLSX/Full/私有固定回归及继承model.runs/0041删除风险继续最终新库待验。

## 发现与订正

初次专项验证中的测试提示token写成workbench而实际断言是workspace，已统一原门禁提示；未影响业务。七探针的writer路径起初多一层父目录，前六已恢复，修正工具路径后七项全部拒绝并恢复。三个查询helper中的Vec返回直接用原PersistenceResult collect，由调用方传播错误，避免多余Ok/问号包裹。

原人工fixture起初复用已过cutoff的Stage C任务，并尝试同task同conflict追加第二决策。源码复核0018触发器后纠正：复用同数据库及版本但创建截止前的比赛/研究/证据/冲突/路由、trace一致的RESEARCH_PARTIAL任务和manual_required评估，每task每conflict一个不可变决策；另一任务的accept_unknown只改变其人工覆盖及全局冲突事件，不覆盖首任务决策。未运行真实PG，不将夹具订正或ignored编译冒充实跑；保留最终新库验证。

## Windows交付与继续位置

推送本项精确实现到同一R8分支，复用原Public Platform CI/Windows Automated。确认新head启动后停止轮询，不继承10 SUCCESS或预期118/169；11保持VERIFYING，12 BLOCKED，全部R8未完成前不创建阶段完成记录。Mermaid Chart已更新真实只读与人工投影链；结束时Create State保存足球模型 `96f03270-c236-46de-99fa-db85d2fbf4ce`（uniquenesssta/123，0.23.0），不写其他项目，不替代Git。

## 回退与真实风险

回退基线2770adca714fa471e3d801ecd3f541f0f6a05db4；受控revert本节点，同步出口/唯一owner/原测试/门禁/清单/记录，不手工复制旧实现或变更历史库。Postgres多读之间状态仍可能推进，这是保留的原语义。真实PG/历史四项/账本/有效XLSX/Windows Full/私有P4/P7 Golden Master及继承model.runs/0041历史删除风险仍最终封包新库待验；公开stub/编译/静态指纹不能替代这些结果。
