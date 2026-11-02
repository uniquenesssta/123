# R14-16 能力与参数候选决策

状态：VERIFYING（实现及本地适用验证完成，自身CI/官方安装包待按推送提交单独验收）。用户授权“收尾13开始16”，R14-13精确65e11ddc/CI41402896621、官方artifact13124247175摘要/双包/startup已验收DONE。当前基线9bc9de25，沿用rewrite/r14-prediction-review-analytics-ui，工作区开始干净；只开展16，下一主交付R14-17未启动。

## 实际归属与修复

复用现有171命令、13分析骨架、后台基线/投影/事务/不可变记录。crates、src-tauri、Schema、模型、参数公式、依赖及保护资产未改；Rust验证可复用13同一源码的成功CI，不伪称重跑。

Analytics/candidates下CandidateFeature组合CandidateContext（范围/人工说明/精确门禁/详情）、AbilityDecisions（能力投影人工接受与拒绝）、ParameterLifecycle（参数候选接受/拒绝、影子、晋升、回滚及原请求查回），CandidateActions/Controls/View负责受保护确认和原FIP控件。main退出八个旧业务函数，仅保留事件/生命周期转发。Review已交付的独立复盘确认流程继续消费原后端，不重写08状态机。

能力与参数候选各自绑定原ID/内容，确认只消费一次，写前重读状态。晋升须精确赛事/Profile/时点、基线模型/参数、最新影子报告及门禁；回滚核对原晋升决定与当前目标绑定，返回决定后还读取正式绑定，不把影子通过或回执存在冒称版本已切换。晋升/回滚说明分别按数据库、候选与操作保存，回滚不能自动复用晋升说明。

不再用全局审核清零隐藏整个候选页面；历史/配置/精确门禁可查看，写动作独立检查。生成表单的赛事/时点不作为别的候选许可。未知写入保存窄恢复引用，禁止自动重发；只读生命周期/诊断入口保留可用。迟到详情不能盖住后来打开的另一份诊断。

公开generate_parameter_tuning_candidate实际始终返回模型提供器不可用，设计与实现均保留该真实阻断，不以固定参数或概率补成功。原生成API/请求契约及最大变化字段继续保留；本页四个批准选择器用于查看条件和门禁，未冒充公开端有生成能力。当前已有候选的约束/数值仅从记录展示，不在前端重算。

## 设计

使用Impeccable Operate/harden、Figma design-to-code。文件PN0Whgu6HLWIHx4Mv6aHfu；session9b83d929-3549-4337-890c-cd13d3a983ab。Screens大页metadata读取Transport closed后，改用程序只读列出顶层状态，再获取指定高保真上下文/截图：Public612:151382、Pending612:152175、ShadowPassed612:156378、BindingChanged612:157258、Promoted612:158148、RolledBack612:159013、CompactDark817:341889、AcceptCandidate614:151669。沿既有FIP字体/色彩/8px间距/12px面板内边距/40px选择器和32px动作；未另立视觉方向，未修改或删除Figma节点。通用对话框承载原候选/分区/版本事实和明确取消。

## 验证进度与边界

完整96步前端检查已通过（94项原Node入口、TypeScript与Vite构建），涵盖架构/63项状态所有权/原171命令/保护资产和原有各Feature回归。最终两处显示修正后另作差异复验：36项生产候选工作流检查、34项Windows完整挂载候选操作、TypeScript、最终生产构建和原生13项全部通过；未把修改前的96步称为最终树重新全跑。Impeccable最终检测结果为空。原96步和差异复验共同覆盖交付树。

第一次浏览器复验发现原门禁变化后按钮需要显式重新核对；测试改为真实点击门禁刷新。第二次发现未知操作把只读生命周期按钮也禁用，已修复；拒绝通过关闭检查消除错误。一次并行启动浏览器因15328端口冲突失败，后续串行验证，未改超时或端口规则。自动审批拒绝了最小样本门槛放宽及影子旧回执保护移除两份草案，未执行；最终保留样本与新回执保护，其他修复独立完成。

证据根O:/Project/FOODBALL/logs/r14-16-20261103。BA-12真实PG候选写入/绑定并发/回滚及BA-13私有模型固定回归，继续按已批准安排R17-06关闭；公开阻断、合成fixture及空目录实测均不抵销Full。若后端未提供可核实的原请求记录，保持未知，不清恢复引用制造成功。

回退基线9bc9de25只撤销代码，不能撤销已经发生的能力历史、参数绑定或不可变决定。TEMP/TMP单独配置O:/Project/FOODBALL/tmp，必要日志保留；每次成功推送后实际清空唯一tmp/old，不跟随链接目标。后续R14-17，未启动。

## 最终入口、验收与交接（2026-11-03）

| 原入口/消费者 | 唯一实际owner与真实数据流 | 保留与增量验证 |
| --- | --- | --- |
| main的能力候选决定与证据；质量页/候选页 | AbilityDecisions.prepare/submit/recover → listAbilityCandidates/decideAbilityCandidate；CandidateActions.ability/abilityDiagnostic | 原球员/维度/旧值/新值/版本确认，accepted_observation_id读回；Review仍复用原受保护决定 |
| 参数待审接受/拒绝与目录/诊断 | ParameterLifecycle.prepare/submit + CandidateActions.parameter/diagnostic；listParameterTuningCandidates/decideParameterTuningCandidate | 接受仅进入影子队列；不把建议接受改成正式绑定 |
| checkParameterLifecycleReadiness | CandidateContext.gate/inspect → parameterLifecycleReadiness原request | 具体赛事/Profile/时点/最小样本/当前绑定和分区均取后端；字段变化立即作废旧门禁 |
| runParameterShadowValidation | ParameterLifecycle.submit(shadow) → runParameterShadowValidation + listParameterShadowValidations | 原候选新报告，passed/failed/blocked分别投影；通过仍待人工晋升，阻断不用成功色 |
| promoteParameterCandidate/rollbackParameterCandidate | ParameterLifecycle.submit/recover → 原promote/rollback命令 + listParameterPromotionDecisions + 当前readiness | 原候选版本/最新报告/人工说明/新决定/当前绑定分别查回，未知不重发，回滚不复用晋升说明 |
| showParameterLifecycleHistory；恢复按钮 | CandidateActions.history + CandidateFeature.recover；原候选ledger和DB独立恢复引用 | 迟到详情/错误不污染新页新库；未知写入仍可查看只读历史，写动作继续锁定 |
| 生命周期生成表单/选择器/草稿 | CandidateView/Controls、CandidateContext + analyticsStorage.capture | 四个原FIP Select单次挂载；公开generation始终不可用，保留真实API契约和最大变化草稿，不造私有结果 |

以上对应退出main八个旧业务函数，新增9个职责文件；main只组合和转发。无文件删除、依赖/Schema/命令/wire变更。原质量审核、建议包和Review能力接受路径继续保留；不把13正确业务重复重写。

| 场景 | 前置/触发与实际结果 | 证据/层级 |
| --- | --- | --- |
| D1 / V1 原候选/分区/基线变化 | 确认后改候选状态、基线ID、定义摘要或精确分区；生产工作流拒绝且0写入；页面正式绑定变化显示阻断 | candidate-delta.log：36项行为；browser-delivery/results.json：34项完整挂载 |
| V2 双击/冲突/未知晋升 | 延迟API后双击仅1写入；另一绑定接管0写入；丢回执从原决定及真实绑定查回，无重复晋升 | 同上，合成外部边界，不代替真实并发事务 |
| V3 回滚失败 | 原晋升后使用独立新说明，注入绑定冲突；保留当前候选绑定和未知引用，绝不显示原版本已恢复；丢回执可只读查回 | 同上；真实PG回滚与私有固定回归仍待验 |
| D2 同候选状态与设计 | Public612:151382/Pending612:152175/Passed612:156378/Changed612:157258/Promoted612:158148/RolledBack612:159013；接受确认614:151669、CompactDark817:341889 | Figma上下文与已查看截图；browser-delivery明暗/1440×900、960×720、1280×720、1920×1080，100%缩放。QA身份明确；单内容滚动、8px行间距、26/700标题、说明与控件可达 |
| Windows实际数据 | 无mock的Tauri传输；12个实际模型，0能力候选/0参数候选/0结算；具体赛事T-1h真实返回已结算0/可验证0和100场门槛阻断；切赛事废弃旧门禁；前后目录一致，0业务写入 | native/report.json：13项PASS；native-startup-final.json：28条日志、10完成操作、0无效/运行错误；真实数据读取通过不等于候选写入通过 |

截图保留browser-delivery/*.png、native/*.png；失败迭代及端口冲突原日志保留，不用后续PASS抹除。最终核对还修正了“只读可查看却标未解锁”和空样本下暗示质量审核已通过的提示；最终native-delivery.log和截图来自最后构建。真实提供器未运行，generation按钮保持设计批准的禁用态。

Rust源码与13精确65e11ddc完全一致（git diff --exit-code核验）；复用其成功CI的753通过/40 ignored、fmt/Clippy。原生可执行SHA256 edc7a9c4fa0d9e091c98f692ba38a65dbe39aae6769347d76af03ff9a460b0e2与13记录相同，本次加载最新dist实测；没有冒称本批重编Rust或完成安装包验收。

最终源文件及构建摘要保存在同证据根validation.json/source-manifest.json；CI按最终提交SHA只查一次，不轮询，不拿13成功代替16。官方新包未验前本批保持VERIFYING；推送与tmp/old实际清理数字写入delivery.json/old-cleanup.json。下一主交付 **R14-17 应用内验收与报告**，尚未启动。BA-12/BA-13最迟R17-06关闭，不能以本次空样本或模拟记录抵销。
