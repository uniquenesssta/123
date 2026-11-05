# R16-01 剩余入口、类型与兼容实现审计

状态：DONE。2026-11-05用户授权收尾R15-12并开始本批；分支rewrite/r16-legacy-cleanup，基线7e99e236a446a6b53c465b33bf33a41f8bfac92f，工作区起始干净。覆盖原R16-01～08，不授权R16-09或R17。R15-12精确ed1e892d、CI42006118669/job127173177599及官方artifact13406729451已正式核验，见[R15出口](../R15-ai-profiles-logs-ui/R15-stage-exit.md)。

## 本批结论与实际修复

没有以旧路径、文件长度或直接引用数认定删除。保留现有业务、接口和数据流程；本批没有删除生产文件，没有改运行时代码、DTO、Schema、迁移、模型、依赖或视觉。发现并修复四类真实缺口：

1. 原featureForPath从目录名猜登记名，漏判ai-workspace与api-workspace、openai-profiles与openai的别名；21个实际内部文件未进入应有检查。改用契约中真实public_entry根和独立页面owner，未登记的Feature源/目标明确失败；公开入口只接受已登记文件，不增加例外。
2. 原resolveRelativeImport把存在的目录提前当文件，漏掉与CSS目录同名的lineups/prediction页面及目录index入口。现在只接受真实文件，继续按既有.ts/.tsx/.js等和index候选解析。原静态import、re-export、动态字面量路径均复验，正确公共出口/同域私有调用保留。
3. 架构token登记仍是193变量/13文字的旧来源，而原生产检查与CSS已采用237变量/25文字/6效果的批准快照。同步当前source_record与数量，原token检查读取同一登记，并保留其批准版本、237/25/6、computed-value和CSS一致性断言。历史快照与R12记录保持；没有改UI数值或重开R12。
4. 原数据库静态基线漏登记R14-07已验收的测试增强。当前PG测试blob24c55211与该批正式f38b52dd一致，较旧33e58c32仅增加真实隐藏入口、隐藏前后血缘一致及可见目录排除检查。登记当前指纹并保留旧指纹、原变更da42c749和正式验收提交，补必要语义要求；原46迁移、18测试名/数量、不可变检查与迁移聚合不变。现有verify_database_baseline接入原前端入口，完整门禁由96变为97步。

唯一新代码归属为scripts/architecture/lib/frontendFeatures.mjs（Feature身份与导入边界）；verifyFrontendFeatures.mjs是同入口的隔离样例，不是新runner或workflow。原imports.mjs继续拥有真实模块解析；原verifyModuleBoundaries组合规则，原token和数据库检查分别保留职责。

## 原01～08逐项处置

| 原ID / 对象 | 实际owner及消费者 | 本批处置与依据 |
| --- | --- | --- |
| 01 main | bootstrap/main→registerApplicationModules的字面量动态import→main.createBrowserApplicationModule；19Feature、Router/Session/Workspace/Modal等原公开能力 | 保留组合与跨功能接线。main仍有实际临时演练、快捷建球员、目录加载和返回原任务等兼容编排，没有把它冒称为“已无业务”。真实按钮/ports仍消费这些路径，不为减行数重迁 |
| 02 API Client | src/api/client.ts的179个具名成员，原Platform文件/传输边界与171 native命令 | 保留公共wire适配；API逐成员消费者见source-audit.json。14项无直接api属性语法引用并不等于死代码，其中ZIP两项经files:api注入有真实调用；其余保留原契约，由09继续复核，不擅删命令 |
| 03 Types | src/types.ts的274个TS公共声明；跨Feature消费者明确，未发现其他src文件重复声明同名wire类型 | 保留兼容定义。Rust Domain清单366是另一口径，不能与TS数量混为一致性证据。本批不迁移类型，字段/空值/时间语义按未变源码及现有契约保留，不冒称全部序列化实跑 |
| 04 Pages | 当前5个TS页入口均从真实组合图可达；lineups/prediction/workbooks仍含真实View，lineupPresets是独立页面owner；analytics重导出仍由main消费 | 全部保留。已退出的review/runs/openai/apiWorkspace/logs路径不重新创建；真实实现和公共出口保持唯一 |
| 05 Components/Controllers | 7个components文件及pageLoaders.ts均有实际生产消费者；fetchLineups提供原三项并行读取，main导航请求核对后采用 | 保留已有单一职责，不新造空目录/镜像控制器。共享格式、足球文案、名单、可搜索控件等由原owner继续提供 |
| 06 CSS | src/styles/index.css为唯一运行入口，实际依赖图191张CSS；token入口src/ui/tokens/index.css | 只修登记来源，保留R12-18顺序、作用域、共享/Feature边界和所有真实样式；不是按文件名批量删CSS |
| 07 Rust | 11 workspace crate、原Tauri组合根/171命令、唯一持久化owner，Rust运行树与R15-12相同 | 无模块退出需求。BA-03的model.runs保护计数/明确拒绝/不置空运行身份仍在；真实PG与回滚仍R17-06。显式cfg的Windows验收host不是生产第二入口 |
| 08 公开出口/兼容壳 | 85个仅导入/重导出模块，真实公共Feature/UI出口与analytics兼容页分别登记 | 保留有职责的公共边界和实际消费者，不把barrel本身当重复实现。此次未生成兼容壳；源/目标身份与路径解析修复让这些消费者真实进入审计 |

src/main.ts中coachList/formationCatalog是原共享参考数据接线，Lineups通过公开ports读写；实体详情、目录、草稿仍归对应Feature。没有复制出第二个可写状态。跨功能workflowContinuation仅保存返回页面/比赛/窗口上下文，不持有第二份业务账本。

## 调用图、动态路径与保留清单

一次性审计工具仅放任务临时目录，报告保存在O:/Project/FOODBALL/logs/r16-01-20261105；不成为生产配置或新增回归框架。source-audit.json/source-edges.json覆盖455前端文件、196个Feature目录文件、19个登记Feature、17路由、191CSS及API/类型/公开出口消费者。

17个Page union、RouteRegistry实际导航项和main.render分支一致。唯一字面量动态import为registerApplicationModules→../main；未发现非字面量动态模块导入。此结论不把DOM事件/文件资源当TS import：HTML启动入口、真实data-action/Feature actions、CSS图、SVG声明/资产与Tauri注册另外核对。

从bootstrap计算的静态可达图含类型边；24个不在该图的文件已分类保留：10浏览器夹具、1个svg.d.ts、2个公开feedback/overlays出口及11个已交付共享控件文件。它们有tsconfig、既有控件夹具或公共组件能力约束，不是24个待删文件。路径修复前错误图明确标为invalid并保留，禁止拿旧“不可达”数做清理依据。

原legacyLineup的单边阵容、主页面临时演练、ZIP导入/导出、部分仅native公开的命令与公共DTO均保留。新库、多命令事务和私有Provider结果不由静态消费者图证明；不把当前未直接调用的API改成空函数或删功能制造绿色。

## 17页责任与既有证据交接

本表只核对前批分配的设计/行为义务及当前owner，不重新宣称每页所有状态1:1。实际同状态、完整挂载和Windows层级以各批记录为准；本批没有修改Figma或界面。10/31处置表中的OPEN是历史审计状态，本表给出后续正式记录，禁止按旧行重建页面或靠删除动作关闭。

| 页面 | 当前归属 / 本域关闭记录 | 仍需真实外部验收 |
| --- | --- | --- |
| S01首页 | dashboard；[UI01](../R13-core-ui-features/R13-UI01-approved-screen-deltas.md)、[只读01](../R13-core-ui-features/R13-01-read-only-entry-pilot.md) | 全局Full按BA-14 |
| S02比赛阵容 | lineups；[R13-07](../R13-core-ui-features/R13-07-presets-and-paired-lineups.md) | BA-03/04/06/07 |
| S03推演 | prediction及其research；[R14-01](../R14-prediction-review-analytics-ui/R14-01-context-readiness-and-execution.md)、[06](../R14-prediction-review-analytics-ui/R14-06-research-evidence-and-freeze.md) | BA-02/08/13/14 |
| S04复盘 | review/postmatch；[R14-08](../R14-prediction-review-analytics-ui/R14-08-review-facts-and-package.md)、[12](../R14-prediction-review-analytics-ui/R14-12-postmatch-settlement-and-monitoring.md) | BA-07/12 |
| S05运行记录 | runs；[R14-07](../R14-prediction-review-analytics-ui/R14-07-run-history-and-lineage.md) | 当前PG隐藏/血缘专项及BA-14 |
| S06球队 | teams；[R13-05](../R13-core-ui-features/R13-05-team-catalog-profile-and-package.md)、UI01 | BA-05/07 |
| S07球员 | players；[R13-06](../R13-core-ui-features/R13-06-player-catalog-history-and-observations.md)、UI01 | BA-05/07 |
| S08预设 | lineup-presets/lineups；R13-07 | BA-03/06 |
| S09工作包 | workbooks接Teams/Players/Match唯一导入owner；[R13-09](../R13-core-ui-features/R13-09-workbook-workflows.md) | BA-04/07数据库读回 |
| S10赛事规则 | competitions/rules；[R13-03](../R13-core-ui-features/R13-03-competition-rules-and-routing.md)、UI01 | BA-14 |
| S11应用验收 | release；[R14-17](../R14-prediction-review-analytics-ui/R14-17-acceptance-and-reports.md) | BA-12/13/14；现库B契约阻断未核销 |
| S12分析 | analytics/candidates；[R14-13](../R14-prediction-review-analytics-ui/R14-13-analysis-jobs-and-packages.md)、[16](../R14-prediction-review-analytics-ui/R14-16-candidate-decisions-and-lifecycle.md) | BA-12 |
| S13问答 | ai-workspace；[R15-01](../R15-ai-profiles-logs-ui/R15-01-session-draft-request-lifecycle.md) | BA-10/11 |
| S14API配置 | openai-profiles；[R15-07](../R15-ai-profiles-logs-ui/R15-07-profiles-parser-credentials-tests.md) | BA-11 |
| S15数据库 | database；[R13-02](../R13-core-ui-features/R13-02-database-connection-and-reset.md)、UI01 | 新库重置/Full，BA-14 |
| S16日志 | logs及Platform；[R15-12](../R15-ai-profiles-logs-ui/R15-12-logs-diagnostics.md) | BA-11真实秘密边界 |
| S17架构 | architecture；只读01、UI01 | 声明不等于外部能力；BA-13/14 |

没有据本次审计关闭新的布局/业务缺口，也没有把已发现的守卫失败延期；本批实际发现均在上述修复处理。R17-01仍应做最终证据覆盖，R17-06逐项关闭既定真实环境项。

## BA-01～16承接核对

| 账本 | 本次核对与接收位置 |
| --- | --- |
| 01 | 当前静态/契约必需门禁；数据库旧登记漂移已修，不能用旧PASS抵销当前失败 |
| 02 | 原4项PG回归和18 broad契约保留，仍需R17-06实际执行 |
| 03 | 当前删除保护包含model/research/freeze/settlement，明确拒绝，不置空model运行身份；R13-07修复保留，真实PG/回滚OPEN至R17-06 |
| 04～07 | 导入身份、A8、实体删除并发、双边锁/历史、有效XLSX到数据库读回，分别沿R13-05/06/07/09及R14-08；真实部分R17-06 |
| 08 | 正式/影子/研究/历史仍按R14-01/06/07归属；多事务/并发/故障读回不冒称已原子，R17-06 |
| 09 | R15-01已修前端串线证据保留；本批不重做会话Feature |
| 10/11 | R15已正式关闭适用出口；真实API、凭据、取消和部分写入读回仍R17-06，不据本机样例关闭 |
| 12 | 复盘/结算/分析/候选/不可变报告实际边界保留，现库B契约问题仍R17-06；R18不能跳过 |
| 13/14 | 私有固定回归、全部必要PG target、正式P4及最终包安装/Full仍按R17/R18安排；startup不能替代 |
| 15 | Rust过渡边与前端transitional_imports仍为0；目录别名漏扫已补，21文件不再绕过规则，无新增例外。09继续复核最终实际图 |
| 16 | R12已结束；不重做R12-19或拿历史基础证据冒充最终候选，R17-01/R18保留最终复核 |

## 验收、失败与恢复

V1：真实目录/index、同名CSS目录、字面量动态import、ZIP ports和svg.d.ts消费者均追踪并保留，未按表面未引用删除。V2：api.executePrediction仍由运行演练动作消费，client与对应类型继续保留，未宣称已退出。V3：只有公共出口重导出，没有新生成替代业务壳；所有保留项有职责和当前消费者。

baseline-scanner-gap记录21文件漏判；baseline-import-resolution记录4个真实目录/文件解析错误；修复后的原模块门禁含28项别名/公私有/动态/登记/目录负向样例。baseline-token-metadata保存来源漂移。database-baseline-initial保留真实失败；reconciliation保存历史blob差异和验收来源，negative报告4项原守卫隔离验证（即使夹具指纹同步，删除真实隐藏/血缘断言仍失败；迁移改动仍失败）。没有修改PG测试源码、测试名/数量或历史迁移。

适用验证以validation.json与各命令日志为准：原完整架构、两条171命令入口、样式/状态/受保护导入、366 Domain公共类型/根出口、保护资产和数据库静态基线分别记录。最终完整前端包含原有界面/工作流回归及新接入的旧数据库守卫，计97步；先前96步是接回守卫前的中间记录，不充当最终结果。

Rust及运行时代码/依赖/资产与R15-12受测树无差异，复用其精确ed1e892d的本地Rust768/40 ignored、Windows14项和官方CI/双包/startup分层证据；不写成本批新执行Rust或原生业务。当前完整Windows浏览器检查实际重新执行。40项外部测试未因此变为通过，非Full。2026-11-06已单次核对精确3baf4ed2/CI42041966141/job127294792894为success；官方artifact13425857136共52,493,790字节、SHA256 57a0667b8be34c14c5f9057565ac057214f6f6b1418ecd41ab6c579d20c7954b一致，MSI/NSIS签头/摘要及包内7条startup日志通过，0非法行/运行错误，正式DONE。证据O:/Project/FOODBALL/logs/r16-01-ci-42041966141/acceptance.json；不是本机安装、签名或Full验收。

回退点7e99e236，仅撤回本批守卫/登记/记录；没有业务数据写入或迁移，不能用代码回退改变真实用户资料。TEMP/TMP沿项目tmp，已单独提醒。必要证据留logs；无用临时产物按清单归入唯一tmp/old，成功推送后逐文件实际删除、保留old根并独立复核0；稳定依赖/缓存保留。用户2026-11-06已授权下一项R16-09，见[09记录](R16-09-dependencies-and-responsibilities.md)。
