# Figma Patterns & Screen Plan

**设计依据（2026-08-28 用户确认）：所有视觉尺寸、比例、间距、字号和颜色均以已确认的 Figma 为准。应用源码只用于核对功能入口、交互和数据语义，不再作为视觉尺寸参考；旧记录中的源码尺寸仅是历史取证。**

## 当前状态

截至 **2026-09-30**，图标与既定底层组件均完成，Controls保持 **61个组件集 /506个变体**。**Patterns 六组计划全部完成**：PC App Shell、Page Heading / Toolbar / Filter Bar / Selection Command Bar、Metric Card / Action Card / Panel、Master–Detail–Inspector、Workflow Stepper / Timeline、AI Chat。

**产品 Screens 已开始：S01 Dashboard / 数据总览的视觉与本页状态验收完成，1 / 17 条路由；其余16条尚未开始。** 本页15个状态画面仍只计1条路由；跨页面导航集成尚未完成，不能将本页验收称为完整产品端到端通过。

当前恢复入口为 [Screens状态](FIGMA_SCREENS_STATE.json)，执行顺序见 [17路由 Screens计划](FIGMA_SCREENS_PLAN.md)，本页证据见 [S01 Dashboard记录](FIGMA_SCREEN_DASHBOARD.md)。**下一项：S02 `lineups` / 比赛中心 · 比赛与阵容，尚未开始。**

Patterns仍为 **17个组件集 /125个变体 /13个生产单组件**，Foundations仍为 **193 variables /13 text styles /6 effects**。S01只组合现有真实实例，没有新增主组件、修改应用代码或全局变量。

## Patterns 执行记录

- [x] PC App Shell：一级导航、二级导航、Topbar、折叠与主题保持。
- [x] Page Heading / Toolbar / Filter Bar / Selection Command Bar。
- [x] Metric Card / Action Card / Panel。
- [x] Master–Detail–Inspector 工作区（含 Entity Row 依赖）。
- [x] Workflow Stepper / Timeline。
- [x] AI Chat：历史侧栏、消息、只读附件、Composer。

依赖按组归档，不合并为一份大文档。证据见 [Shell](FIGMA_PC_APP_SHELL.md)、[Page Bars](FIGMA_PAGE_HEADING_TOOLBAR_FILTER_SELECTION.md)、[Card / Panel](FIGMA_METRIC_ACTION_PANEL.md)、[Master–Detail–Inspector](FIGMA_MASTER_DETAIL_INSPECTOR.md)、[Workflow / Timeline](FIGMA_WORKFLOW_STEPPER_TIMELINE.md)、[AI Chat](FIGMA_AI_CHAT.md)。

[AI Chat检查点](FIGMA_AI_CHAT_STATE.json) 和 [Workflow检查点](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json) 保留为历史记录；它们的 Screens未开始字段已由本轮 Screens检查点取代。

## 当前阶段：Screens

- [x] S01 `dashboard`：数据总览视觉与本页状态验收。
- [ ] S01跨页面目标回填：lineups / prediction / review / analytics / players。
- [ ] **S02 `lineups`：下一项，尚未开始。**
- [ ] S03–S17：见独立Screens计划，均尚未开始。
- [ ] 完整跨路由原型联调。

S01包含默认Light/Dark、桌面与紧凑侧栏展开/折叠、系统详情、数据库未配置/连接中/失败/成功、首次加载、保留旧数据刷新、读取失败及已验证空数据。15个产品状态画面加1个非产品Review Hub，47条自有页面导航连接；另有137条继承组件交互，不混算页面数。

最终深层检查：3,699个可见节点、1,422个可见嵌套实例；非预期越界、失效引用、未绑定可见颜色、顶层重叠均为0。679个普通非Disabled文字层最低对比度 Light 4.53:1 / Dark 6.86:1；134个可见Disabled控件均无反应连接。连接错误页2px纵向内容超出属于有意滚动，单独记录。

## 页面与实现边界

产品页面只引用已完成组件和模式；有必要的新依赖单独补缺。适用的状态、控件来源与目标必须可追踪；未完成目标不得通过连接旧QA插槽来冒充完成。

所有Review、Pattern示例、Workflow Lab、AI Chat Lab和本轮Review Hub都不是额外产品路由。状态页计数、页面路由计数、本页原型通过、跨路由集成和真实应用测试分别记录。

本阶段没有修改应用代码。原型使用预置状态；连接、重试和加载定时跳转不执行真实后端操作，不替代键盘焦点、网络竞态、输入校验、权限、滚动与草稿持久化测试。

保持既有业务边界：Workflow查看步骤与执行权限独立；Timeline核验、修订、取消和有效比分由真实数据驱动；AI Chat纯文本、只读上下文主动勾选、原会话身份不变，Pending与取消绑定sessionId/requestId；附件不扩展为上传、生成文件或数据库执行。

恢复时先读本计划与 `FIGMA_SCREENS_STATE.json`，再核对当前画布。组件总记录中的阶段统计保留为历史快照，不能覆盖更新的Screens进度。
