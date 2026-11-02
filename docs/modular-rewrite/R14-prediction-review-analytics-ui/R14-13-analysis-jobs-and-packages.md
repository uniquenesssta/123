# R14-13 分析查询、长任务与建议包

状态：VERIFYING（实现及本地适用验证完成，13自身CI/官方安装包待推送后按精确SHA验收）。用户授权“收尾12开始13”，覆盖原13～15；后续主交付R14-16未启动。开始基线fd9b32dd924b6b7b6255d8f2816c624bb5611f0f，12精确CI/官方包已验收，收尾文档本地提交24e39945。沿用rewrite/r14-prediction-review-analytics-ui，R12保持DONE。

## 实际范围和唯一归属

| 原入口 | 当前唯一owner与真实调用 | 保留/修复 |
| --- | --- | --- |
| main总览、loadAnalysisCenter、pageLoaders的分析批读 | AnalyticsFeature / AnalyticsQuery，实际analytics_overview、list_background_jobs和三个候选目录 | 复用原API；原子批读、保留旧值并报错，跨库/离页迟到结果不发布 |
| enqueueAnalysisTask、cancel/retry分支 | AnalyticsJobs，四种原任务，稳定幂等键、原job ID/输入/版本 | 确认前后查重；running且取消已请求仍是running；失败/取消且未达尝试上限才重试；回执丢失保留原请求 |
| export/template/preview/import全局路径与包ID | AnalyticsExchange + AnalyticsStorage，原生文件对话框及原ZIP API | 按数据库绑定已确认导出，预检A与文件B不混用；确认后再预检，后端写前比较完整内容；未知导入只查原response ID |
| decideAiSuggestion、decideDataQualityFinding | AnalyticsReviews / AnalyticsActions，原建议/扫描/问题ID | 一次确认、写前重读、未知结果查原对象；普通建议接受只登记审核，能力建议接受仅生成待审能力候选 |
| 旧纵向折叠页 | analyticsPage / ModelView / JobsView / QualityView / ExchangeView / Analytics CSS | 最新批准S12顶部四指标、横向四步骤、一个剩余高度滚动区；模型结果优先，完整任务目录仍可达 |
| 历史样本、人工证据与H监控 | 继续公开消费Postmatch Feature | 不复制12状态机，不以分析全局计数替代H/I精确分区 |
| 参数/能力候选叶子动作 | 保留原main功能及提取的analyticsLifecycleView | 本批供16复用骨架，不提前声称能力写入/参数晋升已重新验收 |

## 已证实的后端缺陷修复

原refresh插入A后再读全局latest，可能读到并发B；旧总览还缺少原快照/原请求身份。现在同一summary存储原request，提交后按原snapshot ID读回；已有analytics_overview增加可选snapshotId，缺失明确原ID时报错，不退回其他最新。AnalyticsOverview/完整任务结果仅附加可选source_snapshot；旧结果缺失时不输出该字段，保留展示而不推断原始范围。独立扫描仍按原全库范围读取，单独标明来源与时间。

原导入会重新读路径但不知道用户确认的是哪份预检。已有import_ai_analysis_response增加可选expectedPreview；真实ZIP重新解析后比较完整语义预检，变化在Port写入之前拒绝。已有export_ai_analysis_package增加可选expectedSnapshotId，在文件创建/导出记录之前拒绝其他快照。三处None分支保留原命令、Service入口、返回与错误映射，不改171个命令名、不新增迁移/依赖/队列/分析算法。严格静态契约同时锁定None旧调用和Some受控调用。

导出前还会重读同一分析与质量来源；失败/过期读数保留可见，但不授予质量写动作。未决导入阻止新包覆盖原恢复来源。RFC3339比较保留纳秒与时区同一瞬间；不以毫秒截断合并不同输入。

## 设计依据

文件PN0Whgu6HLWIHx4Mv6aHfu，当前读取session c9c6f43c-9ba5-4863-9ac4-becb3845f9fe。S12 History608:135680、CancelRequested608:148958、Quality612:144602、Exchange612:148687、Advanced608:157580及原确认框均已读取高保真上下文并查看截图。ModelResults整页读取超时后按metadata定位真实结果实例I608:151916;380:1212;412:15458;608:152896，再读取并查看高保真截图，未把超时当成功。

复用FIP字体/色彩、Button/Badge/StatCard/DataTable/Progress/InlineMessage/EmptyState、原确认框和全局壳；26/700页面标题、14/700区块标题、12正文/表格、13/500控件，8px组间距、12px面板内边距，默认96px指标和58px步骤。960/1280/1440/1920与明暗主题均核对真实字体、行中心、边界、滚动和键盘。没有另建视觉方向或改写Figma文件。

按用户要求使用Impeccable。旧折叠源码形状断言与最新批准方向冲突，替换为29项真实渲染/激活/门禁检查，并保留其他原检查；此前一次更弱的脚本草案被自动审批拒绝，未执行，已用上述更强检查完成替代。

## 场景与当前证据层级

| 场景 | 前置和触发 | 预期与证据 |
| --- | --- | --- |
| D1/V1 | A/B不同任务、原快照与赛事，安排A慢于B，切页/切库ABA | 38项生产工作流检查包含原ID查询/迟到结果/范围/纳秒/旧快照；受控外部边界，非PG并发实证 |
| D2/V2 | running取消、回执丢失、原ID重试、达到上限、确认失效 | 原请求只发送一次，读回running+requested不伪造cancelled；同键明确恢复入队不新建任务；38项行为和48项挂载检查 |
| D2/V3 | ZIP A预检后换B、跨来源、阻断、重复response、断网及零建议回包 | 写前拒绝变化；原回包仅pending，重复按真实后端拒绝；恢复不重导。真实ZIP Rust测试核对无Port写/无文件/无审计 |
| 建议/质量 | 确认后并发决定或换扫描，普通/能力建议分别接受 | 原ID/回包/扫描绑定；未知保留；普通接受不伪造参数候选，能力接受必须有待审候选回执 |
| 原生与真实环境 | 最终dist + 新编译Tauri + 已配置数据库 | 15项原生验证通过，真实诊断job624c1031-624a-40da-8342-0c3a63391b84在当前PG中succeeded/100%；最终源码再作9项只读复验，不再入队。当前0正式结算、0有效样本、12注册模型，不使用QA替换真实业务结果 |

本地证据根O:/Project/FOODBALL/logs/r14-13-20261102。38项工作流、29项层级/原链路、48项Windows浏览器、完整96步前端均通过。最后发现的旧候选动作直接更新查询缓存已收口为AnalyticsFeature.refresh，公开查询仅提供Readonly视图；写动作留给16。该最终差异另行复验全部Feature启动/行为、21项路由、38项分析、参数生命周期、TypeScript、新生产构建及最终48项分析挂载与9项原生只读检查；不把较早构建冒称最后构建。

Rust cargo fmt --all -- --check、cargo clippy --locked --workspace --all-targets -- -D warnings、cargo test --locked --workspace和新Tauri构建通过；测试753通过、0失败、40明确ignored（真实PG等，不计为通过）。真实ZIP改动/导出断言及原快照参数传递是其中新增5项，不复算前端指标公式。

Windows实际诊断全流程15项及startup（74条、0无效、0运行错误）通过；最终构建只读复验9项及startup（23条、0无效、0运行错误）通过。最终Impeccable detect src/features/analytics --json返回[]，不是引擎缺失。原生第一次脚本误将空样本下禁用的文件入口当作可点击页面，已按真实禁用状态修正；失败脚本/原始日志保留，不删除失败证据。最终构建命令首次使用Vite受限包子路径失败，随后改用项目现有包入口解析器成功，不修改依赖。

关键证据：frontend-final-complete.log（96项）、frontend-ownership-delta.log、final-production-build.log、browser-committed/results.json、rust-final-tests.log、rust-clippy.log、native-delivery-verified/report.json、native-final-readonly/report.json、native-final-startup.json及impeccable-detect.json。新提交CI和官方双包单独待验，未沿用12的成功冒称13通过。

## 已知边界和回退

后端建议导入返回全库pending；本批按原response及内容验证，不把整个队列误认为当前包。零建议导入回执丢失时，现有建议目录无法证明导入账本是否已写，继续保留未知；不清标记或盲目重导。旧扫描问题从最新扫描目录消失时，同样不推断旧决定未写入。原入队目录不返回幂等键，明确恢复才提交同一原键，原队列upsert返回原任务；无定时轮询。

BA-12实际不可变账本/跨Port事务/并发/完整有样本分析与真实回包链仍按批准安排由R17-06关闭；本批受控外部测试及空样本原生检查不能抵销Full或私有P4成功。文件先写后审计失败可能留文件，UI保留“未核实”并提示核对原文件/记录，不虚构回滚。

回退参考fd9b32dd/24e39945，但代码回退不撤销真实任务/快照/已写建议。TEMP/TMP为O:/Project/FOODBALL/tmp；每次成功推送后实际逐文件清空唯一tmp/old，不跟随重解析目标，保留必要证据与可复用工具。后续R14-16，未启动。
