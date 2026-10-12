# R10 Review / Postmatch / Analytics / Release 后端：执行记录索引

## 当前阶段状态

`IN_PROGRESS`。用户2026-10-12“开始R10-01”授权；从R9文档收尾HEAD `48618fed2d7ff7c2a4db831a037f227dc789f278` 建立唯一阶段分支 `rewrite/r10-review-analytics-release-backend`。01自身精确Windows run38229119264/job114743919870全SUCCESS并DONE；02自身1ad6e68 / Windows run38245932270/job114799659672全SUCCESS并DONE；用户“收尾02开始03”授权03已实施；03自身99b15ad / Windows run38259438025/job114845878326全SUCCESS并DONE，04自身221d211 / Windows run38271387412/job114886215505全SUCCESS并DONE；05自身fe3529f / Windows run38277864089/job114907643831全SUCCESS并DONE；用户“收尾05开始06”授权06已实施VERIFYING，07～13 BLOCKED。

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
| R10-06 | [Evidence Scoring](R10-06-evidence-scoring.md) | VERIFYING · 已实施，静态通过，自身Windows待验 |
| R10-07 | Provider Score | BLOCKED |
| R10-08 | Drift Monitoring | BLOCKED |
| R10-09 | Analytics Calculation | BLOCKED |
| R10-10 | Background Jobs | BLOCKED |
| R10-11 | Analysis Package | BLOCKED |
| R10-12 | Parameter Lifecycle | BLOCKED |
| R10-13 | Release Acceptance | BLOCKED |

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
