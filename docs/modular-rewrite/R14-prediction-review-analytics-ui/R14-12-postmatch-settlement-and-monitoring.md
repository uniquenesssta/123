# R14-12 赛后结算与监控

状态：VERIFYING（本地实现与适用验证通过，等待本批自身CI和官方产物验收）。用户已授权“收尾08开始 R14-12”。开始基线 `c40296f514c2152dd76b6147e6638cd44a703234`，分支 `rewrite/r14-prediction-review-analytics-ui`；工作区开始干净。08已按修复代码40c2df90自身CI41350740759、官方artifact13102382561摘要、MSI/NSIS及包内startup正式DONE，详08记录。本批只覆盖原R14-12，下一主交付为R14-13分析查询、长任务与建议包，尚未启动。

## 实际范围与唯一归属

复用后端7个Postmatch命令、原不可变账本/评分/分区策略，以及R12共享控件、确认弹窗、工作区和Session生命周期。命令名/参数/DTO、Rust、Schema和模型资产不因本批前端归属而改变。

| 原入口/状态 | 实际消费与新唯一owner | 真实命令与验收 |
| --- | --- | --- |
| main.inspectPostmatchReadiness / settlePostmatchReview；Review中的结算缓存 | Postmatch `SettlementWorkflow`；Review通过公开接口载入和只读投影，Review不再另存结算 | postmatch_settlement_readiness / settle_postmatch_review / list_postmatch_settlements，V1/V2 |
| main.prepareEvidenceDecision / submitEvidenceDecision；分析页评分队列 | `EvidenceWorkflow`与`PostmatchActions`，按DB/证据/结算固定草稿、确认和原请求恢复 | list_evidence_scoring_items / decide_evidence_scoring_item，V2/V3 |
| main.refreshPostmatchMonitoring / postmatchOverview；分析加载器 | `MonitoringWorkflow`及`PostmatchFeature`，原始总览读取、当前请求和监控响应分开 | postmatch_overview / refresh_postmatch_monitoring，D2/V3 |
| 分析页postmatchPanel | `postmatchView.ts`及本域样式、共享Select/NumberInput真实挂载 | 只接本批历史/评分/H监控区域，S12其余整页布局和分析任务仍归R14-13 |
| 原main写入确认和全局事件 | `PostmatchActions`组合现有弹窗，main只连导航/Session/Review公开上下文与事件 | 同一确认只消费一次；无自动重试、自动接受能力候选或参数晋升 |

## 已确认缺陷与处理

旧真实监控函数在A响应未回时切换B，随后A仍覆盖B；从main实际函数编译并注入延迟边界已复现，`logs/r14-12-20261102/legacy-race.json`记录expected B/actual A、一次请求。不是测试里另写状态机。

结算确认绑定原复盘/版本、赛果、正式run、当前后端门禁和工作流；提交前重读并比较。结果未知只查原账本；账本已写而工作流未登记时明确显示部分完成，只有核实原结算ID并再次确认后才调用后端同一幂等键补登记，不盲重发。相同比赛的其他复盘不能冒充当前结算。

证据确认绑定原证据、结算、判定与说明；写前重读pending/decision身份，无法验证保留空准确率。慢请求不跨页面/DB/筛选发布，原草稿保持各自归属。返回丢失保留原请求引用，查回不可变判定，不再次评分。

H刷新必须确认赛事/时点/窗口，全局样本数不提供当前分区许可；改条件立即撤销旧显示。报告保留Profile/模型/参数/时点和实际样本，未返回报告不等于稳定。供应商评分与漂移分步保存；发生未知部分完成时保留原请求和明确失败，不能仅发现某个报告就伪称全部完成。BA-12真实多Port事务仍需实际账本关闭。

## 设计依据和限制

采用用户已批准FIP体系与Impeccable harden/polish方法；Figma设计到代码技能已加载。文件 `PN0Whgu6HLWIHx4Mv6aHfu`，Screens `3:5`；本次session `eaabcfa3-147c-46ce-8d5f-469ba33dd2a9`。

已读取并视觉核对：S04阻断 `505:34708`、允许 `505:34757`、已结算 `505:34806`、结算确认 `509:47777`；S12历史/H `608:135680`、分区变化 `608:144127`、监控确认 `614:150934`、证据选择 `614:152422`、正确判定 `614:151914`、证据详情 `614:152383`。没有以无效短子ID或旧已否决版替代实际Frame。S12整页其余模型/质量/校准不在本批关闭范围。

复用14/700章节标题、12/400正文、13/500字段、12/500表头、12px面板内边距、8px组间距、16px面板圆角与语义明暗色。Select40与NumberInput32放入同高控制行居中；同行中心和同列起点按1440及960/1280/1920核验。控件控制器每次重绘及销毁释放，禁止被旧模糊选择器二次增强。

Impeccable context首次默认缓存下载失败，按技能回退直接读取现有Figma/项目等价规范；没有PRODUCT.md或DESIGN.md，不建立新方向。随后发现项目已保留0.1.17引擎；可用检测与最终人工截图结果须分别记录，不将CLI context失败隐瞒或视为整项技能未采用。

## 逐项业务验收

| ID | 前置、操作与预期 | 实际结果及层级 |
| --- | --- | --- |
| D1 / V1 | 已生成复盘但门禁阻断；新确认后后端许可失效；正式run/赛果变化 | 真实生产Workflow与完整挂载均阻断，写入0次。就绪后命令回执与原复盘/run身份一致，再读账本和工作流；不是前端自行计算已结算。 |
| V2 | 同一确认重复触发；结算/评分实际外部响应丢失后重入与查回 | 生产Workflow和页面分别验证单次写入；读回原结算/原判定，不重发。工作流登记部分失败仅可明确确认后补全同一后端键，目录仍1条；不同复盘生成不同键。真实PG事务幂等留BA-12。 |
| V2 原身份 | 未知结算后，原复盘ID未变但赛果/版本已变 | 新增失败反证后修复；禁止用新内容恢复/补写旧请求，原未知引用保留，不清除制造完成。见recovery-target-before.log及workflow-final.log。 |
| V3 | 证据结算A/B切换、取消/失效确认、慢错误；赛事C1请求慢返时已改C2 | 原草稿按证据隔离，旧确认和响应不写新对象；C1不显示在C2下。DB切换/ABA不清除原库请求。 |
| D2 | 全局结算数为0但已选H赛事；后端按完整分区返回实际0样本或部分失败 | 总数不充当H许可；实际0显示样本不足，完整Profile/模型/参数/时点分列。部分成功仍保留原请求未知；只读报告不证明全部步骤成功。 |

## 分层验证与实际限制

- **静态与契约：** 19个Feature、61项状态归属通过。Postmatch公开接口接入Review/分析；Service/命令/历史/资料包守卫保持。新样式限定review与analytics两条声明路由；原禁止逃逸反例保留，新增未声明路由拒绝验证。导航测试编译新增真实loadAnalysisCenter及真实Postmatch owner，不复制加载器。
- **自动测试：** 最终Postmatch生产行为39项、原Review33项通过；受控延迟与丢失仅在外部传输边界。完整前端96步（含类型与构建）退出0，frontend-final.log。该完整轮次后针对最后的筛选说明、滚动和原请求身份修正，分别完成最终39/33、74挂载、样式范围、类型与构建复验，不把早先轮次冒充最后差异未经复验。
- **完整挂载：** browser-wheel-final/results.json为74项全通过，真实生产main/Shell/Feature/Select/NumberInput/弹窗。覆盖1440×900、960×720、1280×720、1920×1080及明暗主题。截图复查发现历史步骤被旧网格挤压而裁切，layout-before.json已保留；本域设置内容自然行高，保持原内容区为滚动owner。新增命中检测、隐藏祖先不裁切、真实滚轮滚动和顶栏固定验证，非仅脚本click或横向宽度检查。
- **Windows原生：** 最终dist配合未变Rust的现有debug客户端，真实Tauri、无QA注入，native-delivery-final/report.json的10项通过。界面与真实overview/settlement/evidence命令的0条记录一致，只读刷新、实际赛事确认与取消、九步复盘、事实账本前后未变化均通过。本次没有执行正式结算/评分/刷新写命令，不宣称真实事务、私有P4或Full成功。初轮原生脚本误用role选择器找dialog导致失败，已按实际dialog修正，失败日志保留在native-first.log。
- **startup：** startup-delivery-final.json为pass，40条有效日志、0无效、0运行错误、14种完成操作，启动/工作区载入与保存通过。不能代替Full。
- **检测和视觉：** 项目保留的Impeccable0.1.17源码检测为[]、退出0；最终截图人工复核。context默认路径首次下载失败已如实保留，未阻止使用已有引擎或改变FIP方向。
- **Rust/安装包：** 对08修复代码40c2df90的crates/src-tauri/Cargo.toml/Cargo.lock差异为空；本批不重复声称新Rust测试已执行。复用原生binary摘要和基础证据见validation-summary.json。新的自身CI和官方双包/startup只有推送后才能核验，当前保持VERIFYING。

首轮S12边界缺少真实DTO的comparisons数组引起路由失败、完整检查发现样式归属遗漏及导航测试未编译新真实加载函数，均修复后复验；没有缩短业务路径、延长期限或取消检查。首轮/中间日志继续保留。

## Figma补差与最终证据

补充可编辑筛选设计[Evidence / Settlement Filter](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=1179-376875)，复用文件内Panel与Select实例及原绑定样式、自动布局；不改既有页面方向。工具写请求多次超时，每次先读取实际画布再继续；最终design context与截图证明默认占位已退出显示、标题/正文/控件层级及自适应高度正确。证据figma-settlement-filter.png与figma-settlement-filter-context.txt。

代码通过真实选项“全部已载入结算”可从指定结算切回全部，不依赖不可选占位项；四种判定、原说明输入及最终确认保持。面板说明、分隔线、12px内边距和8px间距已对齐。S12其余布局仍归R14-13，本批没有关闭整页全部设计义务。

所有命令、最终构建、浏览器、真实原生与启动结果统一在 O:/Project/FOODBALL/logs/r14-12-20261102/validation-summary.json。公开代码提交后，精确SHA、一次CI快照和清理结果写入delivery.json；不轮询PR/CI，不引用其他提交的绿色运行。

## 外部待验、回退与临时产物

BA-12真实账本幂等/并发、评分事务和跨Port部分成功仍按原批准安排由R17-06关闭。受控边界能证明前端调用、确认和状态隔离，不能抵销真实PG/私有提供器证据；本批不以模拟正式结算冒充真实Full。

回退参照本批基线，仅回退代码不能撤销已发生的正式结算/评分/监控报告。未知写入必须核对原账本，不清除恢复引用制造回滚。TEMP/TMP为 `O:/Project/FOODBALL/tmp`，必要证据在 `O:/Project/FOODBALL/logs/r14-12-20261102`。按用户最新长期授权，每次成功推送后清空唯一 `tmp/old`；保留工具/依赖缓存和必要日志，逐文件处理，不跟随重解析链接。
