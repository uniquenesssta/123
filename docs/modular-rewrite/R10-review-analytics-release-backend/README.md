# R10 Review / Postmatch / Analytics / Release 后端：执行记录索引

## 当前阶段状态

`IN_PROGRESS`。用户2026-10-12“开始R10-01”授权；从R9文档收尾HEAD `48618fed2d7ff7c2a4db831a037f227dc789f278` 建立唯一阶段分支 `rewrite/r10-review-analytics-release-backend`。01自身精确Windows run38229119264/job114743919870全SUCCESS并DONE；02自身1ad6e68 / Windows run38245932270/job114799659672全SUCCESS并DONE；用户“收尾02开始03”授权03已实施；03自身99b15ad / Windows run38259438025/job114845878326全SUCCESS并DONE，04自身221d211 / Windows run38271387412/job114886215505全SUCCESS并DONE；05自身fe3529f / Windows run38277864089/job114907643831全SUCCESS并DONE；06自身1ed6dbe / Windows run38290328302/job114948593892全SUCCESS并DONE；07自身cdd2797 / Windows run38350692611/job115142166008全SUCCESS并DONE；08自身f834e31 / Windows run38357781835/job115164583225全SUCCESS并DONE；09自身fbfe83a / Windows run38398255939/job115298907093全SUCCESS并DONE；10自身3365b2b / Windows run38411713492/job115345476640全SUCCESS并DONE；11自身9d3727c / Windows run38423742534/job115387546853全SUCCESS并DONE；12自身6c4243c / Windows run38508581096/job115671629414全SUCCESS并DONE；用户2026-10-14“收尾12开始13”授权13 READY。

## 前置基线与依据

[R10任务书](../../football-model-platform-modular-rewrite-19-docs/10-R10-review-analytics-release-backend.md)、[总纲执行订正](../../football-model-platform-modular-rewrite-19-docs/00-总体架构与前23节.md)、[R9完成记录](../R09-research-ai-backend/R09-stage-completion.md)。已验源码b50d8047a6dbf32775d70704afde61d99b897069 / tree7dd421459bbff3156d4c034fc01e9b74dd2985d5 / [Windows run38221009454](https://github.com/uniquenesssta/123/actions/runs/38221009454) / job114716987812 SUCCESS；R9收尾只改文档，不能替代R10自身验收。

## 任务状态

| 任务 | 责任 | 状态 |
|---|---|---|
| R10-01 | [Review Package Workflow](R10-01-review-package-workflow.md) | DONE · 自身精确Windows全SUCCESS |
| R10-02 | [Structured Match Facts](R10-02-structured-match-facts.md) | DONE · 自身精确Windows全SUCCESS |
| R10-03 | [Review Engine Adapter](R10-03-review-engine-adapter.md) | DONE · 自身精确Windows全SUCCESS |
| R10-04 | [Ability Candidate Transaction](R10-04-ability-candidate-transaction.md) | DONE · 自身精确Windows全SUCCESS |
| R10-05 | [Postmatch Settlement](R10-05-postmatch-settlement.md) | DONE · 自身精确Windows全SUCCESS |
| R10-06 | [Evidence Scoring](R10-06-evidence-scoring.md) | DONE · 自身精确Windows全SUCCESS |
| R10-07 | [Provider Score](R10-07-provider-score.md) | DONE · 自身精确Windows全SUCCESS |
| R10-08 | [Drift Monitoring](R10-08-drift-monitoring.md) | DONE · 自身精确Windows全SUCCESS |
| R10-09 | [Analytics Calculation](R10-09-analytics-calculation.md) | DONE · 自身精确修复Windows全SUCCESS |
| R10-10 | [Background Jobs](R10-10-background-jobs.md) | DONE · 自身精确Windows全SUCCESS |
| R10-11 | [Analysis Package](R10-11-analysis-package.md) | DONE · 自身精确Windows全SUCCESS |
| R10-12 | [Parameter Lifecycle](R10-12-parameter-lifecycle.md) | DONE · 自身精确Windows全SUCCESS |
| R10-13 | Release Acceptance | READY · 用户授权，前置门禁通过 |

## R10-01 实际边界与验证

R10-01 Review Package Workflow 已实施、`VERIFYING`；唯一阶段分支 `rewrite/r10-review-analytics-release-backend`，从 R9 收尾 `48618fed2d7ff7c2a4db831a037f227dc789f278` 建立。Application workflow 独立持有编排、身份门禁与人工确认元数据，ReviewService 五入口已切换；PG 十个工作流方法迁入对应适配器，SQL、审计及事务正文逐字保持，旧两 owner 删除。原 Domain 八状态/九步界面、171 命令、Ports/DTO/错误/模型保护区保持；不自动接受候选或晋升参数。新增 8 项原 Application unit 待自身 Windows，预期 Application146 / Persistence179 / Gateway112 unit、41 contract。84/84 既有源码门禁、完整架构、Rustfmt、等价复核与 8 破坏探针通过；CI 开始后停止查询。R10 IN_PROGRESS，02～13 BLOCKED，真实 PG/历史四项/账本并发回滚/有效 XLSX/Windows Full/私有 P4/P7 固定回归及继承 model.runs/0041 删除风险仍最终封包新库待验。

采用现有 package/workflow 子目录按职责递归拆分，而非为参考路径机械迁移整个资料包。export/preview/workbook仍承担其原职责；workflow独占五个生命周期编排、三项纯身份门禁和确认元数据，PG适配器独占工作流持久化。Domain状态权限继续唯一权威，事实提交原阵容→事实→状态三个Port操作保持各自原事务，未声称改成全包单事务。SQL条件保留幂等结算和单活动包约束。

无新runner/workflow/target/数据库专项入口、依赖或迁移；无R10阶段完成记录。ignored/公共unavailable stub/静态指纹不能记为真实新库项目PASS。


## R10-01 精确 Windows 完成与02授权（2026-10-12）

精确提交 `838e39ea35355662c192769a358c56a06131a6ed` / tree `7f42a75620aa8eb44441670a69a379b0c1a7830b`，[Public Platform CI run38229119264](https://github.com/uniquenesssta/123/actions/runs/38229119264) / Windows job114743919870全SUCCESS，run于2026-10-12 17:44:15北京完成。实际Application146/146（新增8 workflow unit均成功）、Persistence179/179、Gateway112/112 unit和41/41 contract；frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/客户端启动及运行日志扫描全部通过。报告windows-acceptance-20261012-092001.json。

artifact11739056741，14,041,808字节，SHA-256 `2831911b9159911fb03ebc9bf894e03f00a4a01c73001b9ffddc8f9a289a1fa7`。01自身完整成功后标记DONE；用户2026-10-12“收尾01开始02”授权，02 Structured Match Facts READY，同一rewrite/r10-review-analytics-release-backend分支，03～13 BLOCKED，R10 IN_PROGRESS。本次仅五既有文档收尾[skip ci]，源码同上述成功提交；后续02必须取得自身完整Windows证据。

真实PG/历史保护四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4/P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验；ignored/公开unavailable stub/静态指纹不计这些真实项目PASS。此前01 VERIFYING及预期计数保留为实施历史，不替代当前DONE状态。


## R10-02 实际边界与继续位置

R10-02 Structured Match Facts 已实施 `VERIFYING`，基线ab43aee，继续唯一分支rewrite/r10-review-analytics-release-backend。Application真实事实阶段迁入facts coordinator，ReviewService与组合工作流直接复用；PG facts目录独占校验、事件身份、事务写入、查询/摘要、行映射和事实提交，旧review.rs的14职责及事实写入正文移除。原比赛锁、修订归属、SQL/审计/同事务上下文与十个公开方法签名保持，不改变报告/候选/参数人工门禁。新增8 PG unit待自身Windows，预期Persistence187 / Application146 / Gateway112 unit、41 contract；84/84静态、完整架构、19 Rust文件Rustfmt、等价与8破坏探针通过。01自身Windows已DONE，02 VERIFYING、03～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4/P7回归与继承model.runs/0041删除风险仍最终新库待验。

事实事务借用边界与原完整准备上下文保持，原prepare_review_in_tx仍有真实球员/连续性/预测准备职责，留给03 Review Engine Adapter；无提前移除或新增计算。先核对02精确Windows完整结果，失败只修本唯一R10分支；成功且用户授权收尾后才推进03，无阶段完成记录。完整11A/19M/0D、职责迁移、原错误/SQL/风险与回退见 [02记录](R10-02-structured-match-facts.md)。


## R10-02 精确 Windows 完成与03授权（2026-10-12）

精确提交 `1ad6e68042e7749caf6671b2eb4756077dbfa17b` / tree `822588ffe13bd392fbe2851af389eb111b2391ad`，[Public Platform CI run38245932270](https://github.com/uniquenesssta/123/actions/runs/38245932270) / Windows job114799659672全SUCCESS，run于2026-10-12 20:18:27北京完成。实际Application146/146、Persistence187/187（新增8事实unit均成功）、Gateway112/112 unit和41/41 contract；frontend/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/客户端启动及运行日志扫描通过。报告windows-acceptance-20261012-115847.json。

artifact11746489380，14,041,992字节，SHA-256 `378486f022954e5ee7044258e31b09bcd83c8c484c55af6fff11ef71ef1af3d1`。02自身完整成功后DONE；用户2026-10-12“收尾02开始03”授权03 Review Engine Adapter READY，同一rewrite/r10-review-analytics-release-backend，04～13 BLOCKED，R10 IN_PROGRESS。本次只五既有文档收尾[skip ci]，源码同上述已验提交；03必须取得自身精确Windows完整证据。

真实PG/历史保护四项/不可变账本/并发/回滚/有效XLSX/Windows Full/私有P4/P7固定回归与继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开stub/静态指纹不计真实PASS。此前02 VERIFYING与预期计数保留为实施历史。


## R10-03 实际边界与继续位置

R10-03 Review Engine Adapter已实施 `VERIFYING`，基线93a008e，继续唯一rewrite/r10-review-analytics-release-backend。Application普通/资料包两条实际生成入口迁入engine_adapter，Service与组合工作流已切换；PG六个引擎职责迁入准备、上下文、原计算事务和报告持久化，事实提交复用同一完整准备。纯review-engine源码/公式、原SQL/错误/JSON载荷、公开签名、事务/审计/提交后回读及pending候选人工门禁保持。原8 Application测试正文保持，其中2生成测试迁入新owner、共用原fixture；新增5入口unit待Windows，预期Application151 / Persistence187 / Gateway112 unit、41 contract。84/84静态、完整架构、18 Rustfmt、等价及8负向探针通过；01～02 DONE，03 VERIFYING，04～13 BLOCKED、R10 IN_PROGRESS。真实PG/历史四项/账本并发回滚/有效XLSX/Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

普通入口原ID/版本/备注归一化与资料包原状态/确认流程保持各自契约，未把一条路径的前检新加到另一条。PG适配器直接调用原纯calculate_review；共享prepare仍先事实、再球员/历史/预测上下文，报告/候选pending/动态标签/审计同一原事务，提交后详情回读不改顺序。root review仍持有读取和候选人工决定，留给04等后续节点，无重复旧生成/上下文/报告owner。

完整11A/18M/1D及职责迁移/回退见[03记录](R10-03-review-engine-adapter.md)。先核对03自身精确Windows，失败修本唯一分支，成功并获用户收尾04授权后再推进；无R10阶段完成记录。


## R10-03 精确 Windows 验收完成（2026-10-12）

精确提交 `99b15ad261a178322231320ad9dc2f8af00ded8e` / tree `a52fd71ad9551a615720c49d9a69d3380d90361f`，[Public Platform CI run38259438025](https://github.com/uniquenesssta/123/actions/runs/38259438025) / Windows job114845878326 全 SUCCESS，run 于 2026-10-12 22:22:12 北京时间完成。实际 Application151/151、Persistence187/187、Gateway112/112 unit 和41/41 contract；新增5个普通入口测试及迁移的2个资料包生成测试均实际通过。前端契约、TypeScript、Vite、17视口、Rust fmt、locked Clippy all-targets -D warnings、完整workspace tests、Windows release/MSI/NSIS、客户端启动和运行日志扫描全部通过（7条记录、3个完成操作）。

证据 artifact11753201967，14,041,892字节，SHA-256 `984de7735649121de192de496d5d92921feb46583e145ad7b8985d37a483ff04`；报告 `windows-acceptance-20261012-135927.json` 及同名.txt。验收对应上述03自身代码，不借用02结果。本次仅五份既有文档补齐验收状态并提交 [skip ci]，生产源码、测试、门禁、清单及依赖均与精确成功提交一致，不重复运行同源码Windows。

03自身完整Windows通过，当前 DONE；01～03 DONE，04～13 BLOCKED，R10 IN_PROGRESS，继续唯一 `rewrite/r10-review-analytics-release-backend` 分支。用户重复“收尾02开始03”的范围已完成；本次没有新增04授权或进入04实施，没有创建R10阶段完成记录。此前03 VERIFYING、预期151和待Windows描述保留为实施历史，以本节和首部当前状态为准。

真实PG、历史保护四项、不可变账本、并发/回滚、有效XLSX、Windows Full、私有P4/P7固定回归及继承的model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub、静态指纹不计这些真实验收PASS。本轮未运行非Windows Cargo/check/Clippy/build/test、loopback或浏览器动态。


## R10-04 实际边界与继续位置

R10-04 Ability Candidate Transaction已实施 `VERIFYING`，基线aa24f5a，继续唯一rewrite/r10-review-analytics-release-backend。PG候选列表/详情/行映射/人工决策事务迁入ability_candidate_transaction唯一模块；原状态、处理人归一化、0.01过期判断和四状态解码提取并复用，旧root四职责删除。Application/Ports/公开PostgresStore签名、全部SQL/错误/JSON载荷、候选锁与单事务审计/提交后回读保持；拒绝不写能力观察，不自动接受或晋升参数。新增8原PG unit待Windows，预期Persistence195 / Application151 / Gateway112 unit、41 contract；84/84静态、完整架构、8 Rustfmt、13等价核对与8破坏探针通过。01～03自身Windows DONE，04 VERIFYING、05～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

列表与详情同一候选mapper；原资料包/复盘生成仍由03适配器写pending，人工决定才走04。接受观察/候选/决定/审计共用原事务；拒绝无观察，提交后读失败不能擅称回滚。完整7A/8M/0D及职责迁移/原边界/回退见[04记录](R10-04-ability-candidate-transaction.md)。先核对04自身精确Windows，失败修同分支，成功且获用户推进授权后再进入05；R10仍IN_PROGRESS。


## R10-04 精确 Windows 完成与05授权（2026-10-13）

精确提交 `221d21134c46fa45e280230238153694be415aa1` / tree `5f8234f89eb86a6aeb7c19dd8bfc1376385d31b4`，[Public Platform CI run38271387412](https://github.com/uniquenesssta/123/actions/runs/38271387412) / Windows job114886215505 全SUCCESS，run于2026-10-13 00:03:32北京时间完成。实际Application151/151、Persistence195/195（新增8候选unit均成功）、Gateway112/112 unit和41/41 contract；前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、客户端启动及日志扫描通过（7条记录、3个完成操作）。报告windows-acceptance-20261012-153709.json及同名.txt。

证据artifact11758811423，14,041,670字节，SHA-256 `296e86e05c1a649bfab6cda6cb9a938f8b83d75c4f2b19c71d80a3ad404fec59`。04自身完整成功后DONE；用户2026-10-13“收尾04开始05”授权05 Postmatch Settlement READY，继续唯一rewrite/r10-review-analytics-release-backend，06～13 BLOCKED，R10 IN_PROGRESS。本次只五份既有文档收尾[skip ci]，源码同精确成功提交，后续05须自身完整Windows证据。此前04 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计这些真实PASS。无R10阶段完成记录。


## R10-05 实际边界与继续位置

R10-05 Postmatch Settlement已实施 `VERIFYING`，基线96ef21d，继续唯一rewrite/r10-review-analytics-release-backend。Application结算/readiness/列表迁入settlement唯一模块，三个旧用例删除；PG15原生产职责、3上下文类型及1原时效unit迁入具名结算适配器，证据队列/正式样本继续借用同一原事务。全部SQL/错误/JSON、指纹/幂等键、原readiness、既有返回/插入冲突路径及提交后回读保持，原资料包mark_settled仍独立Port；不自动接受候选或晋升参数。新增8 Application/4 PG unit待Windows，预期Application159 / Persistence199 / Gateway112 unit、41 contract；84/84静态、完整架构、18 Rustfmt、12等价与8负向通过。01～04自身Windows DONE，05 VERIFYING、06～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

原已有结算快速回读/插入冲突指纹检查各自保持；结算数据库事务与资料包StatePort不合并。15A/14M/3D、真实队列/样本和测试迁移/回退详见[05记录](R10-05-postmatch-settlement.md)。先核对05自身完整Windows，失败修同分支，成功且用户授权再推进06；不提前创建阶段完成记录。


## R10-05 精确 Windows 完成与06授权（2026-10-13）

精确提交 `fe3529f93ac1ec6777ccff53da73c0836d310a7f` / tree `950654f5357fb05a162fe4cd1449cb071aa9fd79`，[Public Platform CI run38277864089](https://github.com/uniquenesssta/123/actions/runs/38277864089) / Windows job114907643831 全SUCCESS，run于2026-10-13 00:58:51北京时间完成。实际Application159/159、Persistence199/199、Gateway112/112 unit和41/41 contract；新增8App/4PG unit与迁移的原timeliness unit通过，前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、启动及日志扫描通过（7条记录、3个完成操作）。报告windows-acceptance-20261012-163146.json及同名.txt。

证据artifact11761067078，14,042,191字节，SHA-256 `65186f8312bf643c3b6547cc36cc7a61669e23f735a82be172128584ecb382f2`。05自身完整成功后DONE；用户2026-10-13“收尾05开始06”授权06 Evidence Scoring READY，继续唯一rewrite/r10-review-analytics-release-backend，07～13 BLOCKED，R10 IN_PROGRESS。本次只五份既有文档收尾[skip ci]，源码同精确成功提交，06须自身完整Windows证据。此前05 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计真实PASS。无R10阶段完成记录。


## R10-06 实际边界与继续位置

R10-06 Evidence Scoring已实施 `VERIFYING`，基线18f7264，继续唯一rewrite/r10-review-analytics-release-backend。App两证据用例迁入evidence_scoring并切Service/删旧路径；PG查询/映射/人工决定/评分与原队列收拢唯一owner，结算仍借用原tx调用同一队列方法。原SQL/错误/JSON、说明trim后UTF-8字节门槛、评分权重/别名、不可验证NULL、不可变决定/审计/提交后回读保持；不自动判定/晋升参数。新增6App/8PG unit待Windows，预期Application165 / Persistence207 / Gateway112 unit、41 contract；84/84静态、完整架构、16 Rustfmt、16等价及8负向通过。01～05自身Windows DONE，06 VERIFYING、07～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

证据队列归唯一评分owner，原结算同事务调用/支持权限保持；原note8字节/评分权重/NotVerifiable空分/不可变决定保护不改。12A/13M/3D及真实异常/旧test/回退见[06记录](R10-06-evidence-scoring.md)。先核对06自身完整Windows，失败修同分支，成功且用户授权后进入07；无阶段完成记录。


## R10-06 精确 Windows 完成与07授权（2026-10-13）

精确提交 `1ed6dbe0a0805a1255917971fc053e509d7e8452` / tree `06e53984ede9ca117973950dabecb4d855ca79d1`，[Public Platform CI run38290328302](https://github.com/uniquenesssta/123/actions/runs/38290328302) / Windows job114948593892 全SUCCESS，run于2026-10-13 02:41:58北京时间完成。实际Application165/165、Persistence207/207、Gateway112/112 unit及41/41 contract；新增6App/8PG unit与原verdict/timeliness unit通过，前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS、启动及日志扫描通过（7条记录、3个完成操作）。报告windows-acceptance-20261012-182256.json及同名.txt。

证据artifact11767052093，14,046,257字节，SHA-256 `e80a1c864624bde656ed13c83e379aa2e14946fac57ac4cca5be446f546dcf52`。06自身完整成功后DONE；用户2026-10-13“收尾06开始07”授权07 Provider Score READY，继续唯一rewrite/r10-review-analytics-release-backend，08～13 BLOCKED，R10 IN_PROGRESS。本次只五份既有文档收尾[skip ci]，源码同精确成功提交，07须自身完整Windows证据。此前06 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/Windows Full/私有P4/P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验；ignored、公开unavailable stub和静态指纹不计真实PASS。无R10阶段完成记录。


## R10-07 实际边界与继续位置

R10-07 Provider Score已实施 `VERIFYING`，基线85ecf15，继续唯一rewrite/r10-review-analytics-release-backend。实际PG供应商聚合、原评分/指纹与快照写入、最新读取和字段映射归provider_score唯一owner；App综合监控/两原Ports不变，不为参考App目录虚构供应商入口。原SQL五分区过滤/BTreeMap与决定顺序、60/15/25原公式、空准确率零分、未先舍入的均值、原指纹/版本/快照键及逐组pool幂等写入保持；监控仍供应商→漂移→总览，没有新事务/审计或自动参数晋升。新增9原PG unit待自身Windows，预期Persistence216 / Application165 / Gateway112 unit、41 contract；84/84静态、完整架构、8 Rustfmt、14等价及10负向通过。01～06自身Windows DONE，07 VERIFYING、08～13 BLOCKED、R10 IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/Windows Full/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

7A/9M/0D与23字段/原公式/调用/可见性/风险/回退见[07记录](R10-07-provider-score.md)。原共享partition及08漂移继续root，不新增不存在的App供应商API。先核对07自身精确Windows；失败修同分支，成功且用户授权后推进08，无阶段完成记录。


## R10-07 首轮 Windows 失败修复（2026-10-13）

首轮精确提交 `688892dd1a67401df028cfdde9cbec08ccedf39b` / tree `aa331e6126bd926b7378ba44ba715d544ac02aff`，[Public Platform CI run38296053769](https://github.com/uniquenesssta/123/actions/runs/38296053769) / job114967284703已完整结束且FAILURE。架构、前端/contracts/TypeScript/Vite与17视口已实际通过，Rust在Clippy阶段因provider_score/aggregation.rs:47的 `uninlined_format_args` 失败；workspace单测和Windows release/MSI/NSIS/启动未执行完成，不记PASS。失败记录windows-acceptance-20261012-191326.txt，artifact11768138324，671字节，sha256 `23bbd18f0a2e47b2791aebbcd96de0138c8015b96ad23c464604b842065d8f8e`。

原因：原key的混合字段表达式partition.partition_key提取为纯函数参数partition_key后，四个format参数均为普通具名标识符，旧位置参数写法触发Rust1.88严格Clippy。现改为 `format!("{provider_id}:{partition_key}:{decision_set_sha256}:{POSTMATCH_MONITORING_VERSION}")`，相同四个Display值/冒号/顺序/版本，哈希和快照键字节语义不变。未改评分、SQL/绑定、幂等、事务、公开API、原9unit或前后调用顺序，未用allow或关闭-D warnings。原Postmatch身份守卫切到实际捕获写法；原Rust源码卫生门禁新增此已确认回归的含空白/换行/可选尾逗号拒绝检查，无新框架。

修复实际通过84/84原JS、完整npm run verify:architecture、1个实际修改Rust文件独立Rustfmt1.88check、单表达式逆替换精确恢复688892d全文、其余链/9unit冻结核对、原07的14项等价（仅先将捕获格式展开为原位置参数再比较）、原10破坏探针与新3格式回归探针，全部字节恢复。三新探针为旧多行、单行、可选尾逗号，分别由既有Rust卫生门禁独立拒绝。原9unit数量/断言保持，预期PG216/App165/Gateway112unit+41contract仍待修复提交自身Windows实跑。

Domain365/129/301映射、scan1217/sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持；使用摘要更新为 `18ce78fc3bd7b90bb6ba841e96ed6bd60a28f3660d9b278370c2ea4461f0891b`。18保护资产、171命令、46迁移与18原PG静态契约/State/Ports保持；旧使用摘要与首次无Windows失败描述保留为实施历史，以本节为准。scratch r1007fix-static-checks.json/r1007fix-architecture.log/r1007fix-equivalence-probes.json/r1007fix-original-equivalence.json均可重建。

按root AGENTS查询Context7：确认rust-toolchain/原workflow固定1.88.0；仓库索引无该精确问法结果，[Clippy官方stable文档](https://rust-lang.github.io/rust-clippy/stable/index.html#uninlined_format_args)给出uninlined_format_args规则及format!("{var}")写法（lint自1.66加入），结合本项目1.88实际诊断采用；没有声称取得1.88专属页面或新增版本/依赖。Create State使用稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce保存本轮精确失败/修复/新CI交接。链路逻辑未变，原Mermaid Chart图继续适用，无新增流程图或子代理。

修复提交仍VERIFYING，01～06DONE、08～13BLOCKED、R10IN_PROGRESS。沿用唯一rewrite/r10-review-analytics-release-backend和原Windows CI；实际Windows job启动后停止查询并提示约20～30分钟，不以首轮前端成功或源码检查替代整轮Windows通过。真实PG/历史四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored/公开stub/static指纹不计真实PASS。


## R10-07 第二轮 Windows 精度夹具修复（2026-10-13）

精确修复提交 `8c9ab5d4fae894ed0735c9adc74432e6b8a984c4` / tree `d31cfe075eaedc76c3f99ba59c251b179a9d4661`，[Windows run38346437982](https://github.com/uniquenesssta/123/actions/runs/38346437982) / job115128855419完整结束且FAILURE。前端/contracts/TypeScript/Vite/17视口与Rust fmt/Clippy all-targets -D warnings已实际通过，首轮uninlined_format_args问题关闭；Application165/165通过，Persistence215通过/1失败。失败唯一为weighted_score_uses_unrounded_means_before_original_six_digit_rounding；其余8个新供应商unit实际通过。PG失败后后续workspace目标、release/MSI/NSIS/启动未完成，不算完整WindowsPASS。失败记录windows-acceptance-20261013-042741.txt；artifact11789116976，669字节，sha256 `154ecca44731c101df3e238a31017de1e73f3f367f373aeb49fb834b12690717`。

实际原因是新增测试夹具/预期的算术错误，不是生产公式偏差：旧输入(.12345649,.87654349,.23456749)按原60/15/25加权的准确十进制值为.2641972900，原最后round6结果.264197，测试误写.264198；且该旧输入提前round均值也得.264197，原本不能区分被测错误。此前记录“该夹具正确.264198/能区分提前舍入”属于错误实施断言，由本节明确订正，不用于当前验收。

现只在该原unit中改为两个已独立核算的区分夹具，unit数量仍9、close容差仍1e-12，额外assert_ne明确拒绝提前舍入结果；没有删测试、调宽容差或改生产评分。两夹具见下表。原Postmatch Service源码门禁读取实际Rust测试数据，用整数BigInt按8位输入/6位输出独立核对60/15/25和正数四舍五入，检查正确分数、提前舍入分数及二者不同；这是静态测试数据检查，不能替代Rust单测实跑。无新框架/target/CI入口。

| 准确率输入 | 时效输入 | 可靠度输入 | 原公式未舍入结果 | 最后舍入正确值 | 提前舍入均值错误值 |
| --- | --- | --- | --- | --- | --- |
| .12345649 | .87654349 | .23456949 | .2641977900 | .264198 | .264197 |
| .12345651 | .87654351 | .23456751 | .2641973100 | .264197 | .264198 |

独立Decimal静态核对同时复查其余数值/样本计数与三个SHA断言：混合样本.5375、缺时效/可靠度.45、不可验证时效均值.9与零weighted、四verdict样本总数17及有序/重复/重排指纹SHA一致。只有一个测试函数变化，其他8个测试/close与全部生产Rust链、公式、快照键、原SQL/绑定/事务/Ports/API/Domain/状态/模型保护保持8c9ab5d逐字不变。

实际84/84原JS、完整npm run verify:architecture、1修改Rust独立Rustfmt1.88check、源夹具Decimal/整数双重核算、生产链冻结/单测试函数变更核对、3新夹具负向（错误预期、恢复旧坏夹具、让正确值与提前舍入值相同）均通过，探针全部文件bytes恢复；未运行非Windows Cargo/check/Clippy/build/test/前端构建/browser/loopback。之前13探针结果和快照格式化守卫保留，原07十四等价对应的生产代码此次完全不变，不重复生产破坏探针。本轮scratch r1007fix2-fixture-audit.json/r1007fix2-static-checks.json/r1007fix2-architecture.log可重建。修复后的原Rust9unit预期仍PG216/App165/Gateway112unit+41contract，必须自身完整Windows确认。

Domain365/129/301映射、scan1217/sourceDigest217241ac243726a5ab6805a7f169e222b97e78c7c971749a5904714af673ebdd保持，使用摘要因测试源更新为 `3b104aec0014ca938b9e67dff0d7b210a39dcfe152d21b268d72fecbb4c05a7c`。18保护/171命令/46迁移与18PG静态契约不变；旧使用摘要与前轮“9测试未改”保留为各轮历史，不代表本次精度函数未改。

07仍VERIFYING、01～06DONE、08～13BLOCKED、R10IN_PROGRESS。继续唯一rewrite/r10-review-analytics-release-backend，原Windows job启动后停止查询并提示约20～30分钟；首轮/次轮部分PASS不替代完整Windows。原Mermaid链适用，本次纯测试数据错误无需重查版本文档/Context7；Create State按稳定足球UUID96f03270-c236-46de-99fa-db85d2fbf4ce记录精确失败/修复/新CI，无子代理。真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归与继承model.runs/0041删除风险仍最终新库待验，ignored/公开stub/静态指纹不计真实PASS。


## R10-07 精确 Windows 完成与08授权（2026-10-13）

精确修复提交 `cdd2797dbd1fa3fe01ff6ca2bdcc7f9d718745fc` / tree `bef40499f1e2b332adccd2008405d6889aefe699`，[Public Platform CI run38350692611](https://github.com/uniquenesssta/123/actions/runs/38350692611) / Windows job115142166008全SUCCESS，run于2026-10-13 13:44:22北京时间完成。实际Application165/165、Persistence216/216、Gateway112/112 unit和41/41 contract；9个新增供应商unit含纠正后的双精度夹具全部通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动/日志扫描均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-051954.json及同名.txt。

证据artifact11791662118，14,046,120字节，SHA-256 `e31726acc367194dd9e0f593775bfa45fbac649e71be7a9bffc310e99cd7f166`。首轮688892d的Clippy失败及次轮8c9ab5d的精度夹具失败已由本精确完整成功关闭；失败和纠正过程保留为历史，不以此前部分PASS替代最终证据。07自身完整成功后DONE，用户2026-10-13“收尾07开始08”授权08 Drift Monitoring READY，继续唯一rewrite/r10-review-analytics-release-backend，09～13BLOCKED、R10IN_PROGRESS。本次仅五既有文档收尾[skip ci]，源码同已验cdd2797，08须自身完整Windows成功，无阶段完成记录。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored/公开unavailable stub/static指纹不计真实PASS。


## R10-08 实际边界与继续位置

R10-08 Drift Monitoring已实施 `VERIFYING`，基线33f7188，继续唯一rewrite/r10-review-analytics-release-backend。App原综合监控两入口归monitoring；PG监控前检/活动分区/总览与漂移样本/纯准备/幂等事务/读取映射各有唯一owner。原Monitoring Port与公开签名保持；供应商→漂移→总览顺序、全部SQL/原纯引擎、窗口/状态/指纹、单漂移事务、新run才写finding/审计与冲突不重写保持。新增6App/11PG unit待自身Windows，预期Application171 / Persistence227 / Gateway112unit+41contract；84/84静态、完整架构、22Rustfmt、19等价及10负向通过。01～07自身Windows DONE、08VERIFYING、09～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

App采用原综合monitoring入口而非虚构Drift Port，PG实际漂移目录单独承担样本/准备/事务/read；原供应商共用分区。18A/13M/2D、23字段/职责/实际异常/回退见[08记录](R10-08-drift-monitoring.md)。先核对08自身精确Windows，失败修同分支，成功且获用户授权后进入09，无阶段完成记录。


## R10-08 精确 Windows 完成与09授权（2026-10-13）

精确提交 `f834e3152e65e791b91897ce4803cedd284f7311` / tree `bc48b5f701f2f2262bd1118390b7691c23ad17a0`，[Public Platform CI run38357781835](https://github.com/uniquenesssta/123/actions/runs/38357781835) / Windows job115164583225全SUCCESS，2026-10-13 15:06:50北京时间完成。实际Application171/171、Persistence227/227、Gateway112/112 unit及41/41 contract；本项新增6App/11PG unit全部通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志扫描均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-063917.json及同名.txt。

证据artifact11794826120，14,048,433字节，SHA-256 `d188c4d0fae8316bde55f14089eac7a7240490a9bf8af36e64df0674db0f4756`。08自身完整SUCCESS后DONE；用户2026-10-13“收尾08开始09”授权09 Analytics Calculation READY，继续唯一rewrite/r10-review-analytics-release-backend，10～13BLOCKED、R10IN_PROGRESS。本次五既有文档收尾[skip ci]，源码同已验f834e31，09须自身完整Windows，无阶段完成记录。此前08 VERIFYING和预期计数保留为历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验，ignored/公开unavailable stub/静态指纹不计这些真实PASS。


## R10-09 实际边界与继续位置

R10-09 Analytics Calculation已实施 `VERIFYING`，基线3fdb443，同一唯一R10分支。App总览及两实际worker刷新归calculation；PG样本/解码/原事务/子指标/持久化与总览投影/读取各有唯一owner，旧root七职责移除，剩余29函数保持。原纯引擎/SQL/绑定/公开接口/错误/窗口/分组覆盖政策、500批次/单事务审计及提交后回读保持；后台进度/取消/全量质量→查询顺序保持，10未提前开始。新增5App/8PG unit待自身Windows，预期176/235、Gateway112unit+41contract；84/84静态、完整架构、18Rustfmt、15等价与11负向通过。01～08DONE、09VERIFYING、10～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7回归及继承model.runs/0041删除风险仍最终新库待验。

App采用原AnalyticsPort，两实际刷新调用切换；PG计算目录与纯引擎保持独立，不把质量/AI/参数及队列生命周期提入本项。14A/13M/1D、逻辑迁移1项和23字段完整记录见[09记录](R10-09-analytics-calculation.md)。后续仅核对09自身精确Windows；失败修同分支、成功并获用户授权再进入10，job启动即停止查询，无阶段完成记录。


## R10-09 首轮 Windows 类型修复（2026-10-13）

R10-09首轮精确656a97c / [Windows run38371099454](https://github.com/uniquenesssta/123/actions/runs/38371099454)因窗口返回类型触发Clippy type_complexity失败。现改为具名内部AnalysisWindows，同步唯一事务调用方和8处原窗口断言，原值、窗口政策、SQL/事务/审计及纯引擎保持；原5App/8PG测试数量不变。84/84源码、完整架构、3Rustfmt、22等价及5新负向通过；09仍VERIFYING，须修复提交自身完整Windows，10～13BLOCKED。

原纯计算、事务/生命周期链不变；本轮10M/0A/0D、具名值投影与原断言逆切核对及未执行项见[09记录](R10-09-analytics-calculation.md)。01～08DONE，R10IN_PROGRESS，无阶段完成记录；Windows job启动即停止CI查询，真实PG/Full/私有回归仍最终新库待验。


## R10-09 精确 Windows 完成与10授权（2026-10-13）

精确修复提交 `fbfe83a9e3b979b2c10e154a718c19639900a41d` / tree `91c39ea42d331e1cc294752479f430bf2655c2a9`，[Public Platform CI run38398255939](https://github.com/uniquenesssta/123/actions/runs/38398255939) / Windows job115298907093全SUCCESS，run于2026-10-13 21:02:32北京时间完成。实际Application176/176、Persistence235/235、Gateway112/112 unit及41/41 contract；原5App/8PG新增unit全部通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志扫描均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-123542.json及同名.txt。

证据artifact11815192224，14,048,590字节，SHA-256 `eccc82a62ef2b48036040ad1ddec43f6c4c0001af7a9db0c83526ce5b28d3f3a`。首轮656a97c的Clippy type_complexity失败由本修复自身完整成功关闭；具名窗口与原8期望值实际通过，之前失败/VERIFYING为实施历史。09自身完整SUCCESS后DONE；用户2026-10-13“收尾09开始10”授权10 Background Jobs READY，同一唯一rewrite/r10-review-analytics-release-backend分支，11～13BLOCKED、R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验fbfe83a，10须自身精确Windows，无阶段完成记录。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开unavailable stub/static指纹不计真实PASS。


## R10-10 实际边界与继续位置

R10-10 Background Jobs已实施 `VERIFYING`，基线4a6fa9b，继续唯一R10分支。App四队列控制入口编组，启动/领取结算/任务执行/请求解码各唯一；PG共享jobs按控制/读取/领取/生命周期/映射/政策重写，旧root及四旧App目录删除。原四类型/六取消checkpoint、状态/错误/重试/到期/优先级/幂等、SKIP LOCKED/状态事件同事务及提交后读解码保持，P4与启动恢复原调用不变。新增13App/6PG unit待自身Windows，预期189/241、Gateway112unit+41contract；84/84源码、完整架构、18Rustfmt、17等价及12负向通过。01～09DONE、10VERIFYING、11～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终新库待验。

共享队列归PG adapters/jobs，P4及启动恢复沿用同一原公开方法，未变业务；App worker只有原异步启动，commands持有四真实控制入口。完整23字段/32路径/取舍/风险/回退见[10记录](R10-10-background-jobs.md)。先核对10自身精确Windows，失败修同分支，成功且获用户授权后再开始11；Windows job启动即停止CI查询，无阶段完成记录。


## R10-10 精确 Windows 完成与11授权（2026-10-13）

精确提交 `3365b2bb5b6396fe010aee311c819bdd6b1198d8` / tree `1b4c59ba5d786bba1da11c13c2bc5ddd50e069e8`，[Public Platform CI run38411713492](https://github.com/uniquenesssta/123/actions/runs/38411713492) / Windows job115345476640全SUCCESS，run于2026-10-13 22:43:53北京时间完成。实际Application189/189、Persistence241/241、Gateway112/112 unit及41/41 contract；本项新增13App/6PG unit全部通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志扫描均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-141620.json及同名.txt。

证据artifact11824026576，14,051,930字节，SHA-256 `ec237a2d6899402531ad616aab6c7fba11cbb6d1098c3d3e868eafe75e8a030f`。10自身完整SUCCESS后DONE；用户2026-10-13“收尾10开始11”授权11 Analysis Package READY，同一唯一rewrite/r10-review-analytics-release-backend分支，12～13BLOCKED、R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验3365b2b，11须自身精确完整Windows，无阶段完成记录。此前10 VERIFYING与预期计数保留为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险仍最终封包新库待验，ignored/公开unavailable stub/static指纹不计真实PASS。


## R10-11 实际边界与继续位置

R10-11 Analysis Package已实施 `VERIFYING`，基线5a19c71，继续唯一R10分支。App导出/回包/人工建议六入口按三职责编组；PG资料包数据/元数据/导入/决定/读取/映射/政策各唯一，旧六目录与root15职责移除，剩余14函数保持。原ZIP库/格式/Schema/摘要验证、SQL/错误/身份核验/待审导入/人工候选事务与提交后回读保持，不自动晋升参数。新增11App/8PG unit待自身Windows，预期200/249、Gateway112unit+41contract；84/84源码、完整架构、17Rustfmt、18等价及13负向通过。01～10DONE、11VERIFYING、12～13BLOCKED、R10IN_PROGRESS；真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终新库待验。

沿用原ai_package标识/资料包库，人工接受能力建议只产生pending候选；其他建议accepted不自动生成参数。完整23字段/31路径/取舍和回退见[11记录](R10-11-analysis-package.md)。先核对11自身精确Windows，失败修同分支，成功且获用户授权后进入12；job启动即停止查询，无阶段完成记录。


## R10-11 精确 Windows 完成与12授权（2026-10-14）

精确提交 `9d3727c4bdc0ebeb21e83a466c49f97720f1e063` / tree `5606f26bcbddbc74be5ea105fe25faa3cf8c4b6c`，[Public Platform CI run38423742534](https://github.com/uniquenesssta/123/actions/runs/38423742534) / Windows job115387546853完整SUCCESS，2026-10-14 00:03:23北京时间完成。实际Application200/200、Persistence249/249、Gateway112/112 unit及41/41 contract；本项新增11App/8PG unit逐项通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动与日志均通过（7条记录、3个完成操作）。报告windows-acceptance-20261013-153953.json及同名.txt。

证据artifact11828902351，14,051,522字节，SHA-256 `bf5b6c6f869263510a06e462570b02fef9265c3a4b6aeb664f1f164ac96d7eea`。11自身完整SUCCESS后DONE；用户2026-10-14“收尾11开始12”授权12 Parameter Lifecycle READY，继续唯一rewrite/r10-review-analytics-release-backend，13BLOCKED、R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验9d3727c；12须自身精确完整Windows，无阶段完成记录。此前11VERIFYING/预期计数为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验；ignored/公开stub/static指纹不计这些真实PASS。

## R10-12 实际边界与继续位置

R10-12 Parameter Lifecycle已实施 `VERIFYING`，基线6da5057，同一R10分支。App九入口按候选/就绪/影子验证/人工绑定四职责编组；PG同名模块升级为职责目录，22原函数迁入，root analytics保留九原函数。指标公式/排序/阈值/哈希、公开提供器拒绝、H精确分区、不可覆盖报告、人工状态门禁/锁/CAS/两绑定事务及提交后回读保持。新增15App/6PG unit待自身Windows，预期215/255、Gateway112unit+41contract；82源码、完整架构、19Rustfmt、19等价及20负向通过。01～11DONE、12VERIFYING、13BLOCKED、R10IN_PROGRESS。真实PG/历史四项/账本并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终新库待验。

沿用既有parameter_lifecycle标识与原PG根模块登记，无新Port/State或自动晋升。23字段/全部39路径及回退见[12记录](R10-12-parameter-lifecycle.md)。先核对12自身精确完整Windows，失败同分支修复，成功且用户授权后进入13；无阶段完成记录。


## R10-12 精确 Windows 完成与13授权（2026-10-14）

精确提交 `6c4243c1150ba0bbf2c0139429e52c3477df6d8a` / tree `8b3928901a0e495f211eb0b937322eb4b48541d2`，[Public Platform CI run38508581096](https://github.com/uniquenesssta/123/actions/runs/38508581096) / Windows job115671629414完整SUCCESS，2026-10-14 12:18:42北京时间完成。实际Application215/215、Persistence255/255、Gateway112/112 unit及41/41 contract；本项新增15App/6PG unit逐项通过。前端/contracts/TypeScript/Vite/17视口、Rust fmt/locked Clippy all-targets -D warnings/workspace、Windows release/MSI/NSIS/启动及日志均通过（7条记录、3个完成操作）。报告windows-acceptance-20261014-035521.json及同名.txt。

证据artifact11869757552，14,050,358字节，SHA-256 `ae7631e1aafcfdfe6e230961c04709d0b8c205dc3e1b28213c7df67c5b4e5c1a`。12自身完整SUCCESS后DONE；用户2026-10-14“收尾12开始13”授权13 Release Acceptance READY，继续唯一rewrite/r10-review-analytics-release-backend，R10IN_PROGRESS。本次仅五既有文档[skip ci]收尾，源码同已验6c4243c；13须自身精确完整Windows，无阶段完成记录。此前12VERIFYING为实施历史。

真实PG/历史保护四项/不可变账本/并发回滚/有效XLSX/WindowsFull/私有P4P7固定回归及继承model.runs/0041删除风险继续最终封包新库待验；ignored/公开stub/static指纹不是这些真实PASS。
