# Figma · AI Chat

**完成并复核：2026-09-30。**

视觉尺寸、比例、间距、字号和颜色以 Figma 为唯一依据。应用源码仅用于确认功能、状态与数据语义。本组没有修改应用源码、既有 Controls 主组件或全局变量。

## 1. 当前检查点

本组完成历史侧栏、消息、只读附件和 Composer，新增 **4 个组件集 / 21 个变体 / 2 个生产单组件**。Patterns 六组计划均已完成，当前合计 **17 sets / 125 variants / 13 production singles**。Controls 保持 **61 sets / 506 variants**；Foundations 保持 **193 variables / 13 text styles / 6 effects**。

下一阶段是产品 Screens。17 条页面链路、7 个导航模块的产品页面尚未开始；本组 Review 与 AI Chat Lab 均为 Patterns 验收材料，不计入产品 Screens。

明确的恢复依据：[AI Chat 状态与节点账本](FIGMA_AI_CHAT_STATE.json)。恢复时读取当前计划和最新状态文件，再核对实际 Figma 节点；不能用旧截图回退阶段。

## 2. 来源与范围

- Repository：`uniquenesssta/123`。
- Design branch：`ui-design-system`。
- Semantic source：`main/src/pages/apiWorkspace.ts`。
- Source blob：`79f4ee4e505ac9da12e6579c1a7729cdeb981bc1`。
- Figma file：`PN0Whgu6HLWIHx4Mv6aHfu`；Patterns page：`3:4`。
- 文档根：`434:17896`，位置 `(200, 29915)`，尺寸 `1600 × 8886`。
- Review 根：`440:18135`。
- 原型入口：`444:18719`。

代码确认了 `messageCard`、`pendingMessageCard`、`conversation`、`sessionList` 与发送区的边界：普通文本问答、匹配会话的 Pending、可继续浏览历史、可用 requestId 对应的取消入口、显式勾选只读上下文，以及既有会话身份锁定。

**附件不是新增上传能力。** 当前页面不提供新文件上传、生成文件或执行数据库提案。`legacyAudit` 保留的是历史结构化记录与文件关联，因此本组附件只有 Context 与 Legacy File 两类只读展示，不增加上传、下载或执行按钮。问答服务请求本身需要 API；“无联网能力”指不为 AI 增加浏览或资料抓取工具，不表示离线完成模型请求。

本地没有同名 AI Chat 组件。可用 Simple Design System 搜索返回 AI Chat Box、AI Sidebar 等通用资产；没有导入。这里复用现有 FIP Button、Textarea、Search Field、Checkbox、Icon Slot、Spinner 与 Empty State，保持本项目的 token 与纯文本功能边界。未将通用 UI kit 的富文本、文件处理或工具执行能力带入项目。

## 3. 组件清单

| 组件 | 节点 | 结构 |
|---|---|---|
| AI Chat/History Item | 434:18130 | Selected False/True × State Default/Hover/Pressed/Focus/Disabled，10 variants |
| AI Chat/Attachment | 434:18157 | Kind Context / Legacy File，2 variants |
| AI Chat/Message | 434:18225 | Kind User / Assistant / Pending，3 variants |
| AI Chat/Composer | 436:18436 | State Empty / Ready / Sending / Error / Cancelled / Unavailable，6 variants |
| AI Chat/History Sidebar | 438:18009 | 生产单组件，Sessions SLOT |
| AI Chat/Workspace | 439:18050 | 生产单组件，History / Messages / Composer SLOTs |

### 3.1 History Item

API：`Title`、`Context`、`Metadata`、`Show archive`、`Selected`、`State`。

248px specimen；宽度可由父级目录内容区填充。`Open session` 是独立的内容 Frame，旁边 `Archive session` 是真实 Ghost Button。选中会话与发送中请求不共用状态；归档不是永久删除。原型不执行真实归档。

### 3.2 Attachment

API：`Kind`、`Label`、`Metadata`、`Description`、`Show description`。

Context 与 Legacy File 使用固定的只读类型说明，名称和描述仍可编辑。上下文存在不意味着已经附加；由 Composer 内默认未勾选的 Checkbox 决定下一条消息是否携带摘要。历史文件仅保留审计信息，不重新处理文件。

### 3.3 Message

API：`Kind`、`Metadata`、`Body`、`Attachments` SLOT、`Show attachments`。

User 与 Assistant 使用不同的语义表面；Pending 是尚在处理的用户问题，不制造流式 Assistant 答案。Assistant 的 `Copy answer` 保持真实 Ghost Button。Body 是普通文本，特殊字符和 Markdown 标记按原文展示；真正的安全转义由实现层负责。

Pending 必须同时匹配所查看会话与相应请求。切到另一会话不能显示前一会话的待处理问题。失败或取消后的草稿保留是宿主状态契约，不通过复制消息伪造成功结果。

### 3.4 Composer

API：`State`、`Show context`、`Show cancel`。Draft 由暴露的真实 Textarea 属性 `Value / placeholder` 编辑；请求状态说明由 State 固定控制，不再使用一个共享 TEXT 默认值覆盖各状态。

| State | Send | 草稿 | 说明 |
|---|---|---|---|
| Empty | Disabled | 可编辑 | 输入普通文本后才可发送 |
| Ready | Default | 可编辑 | 普通发送 |
| Sending | Disabled | 可编辑 | 匹配 active requestId 时可显示取消 |
| Error | Default | 保留 | 用户确认后手动重新发送，无自动重试 |
| Cancelled | Default | 保留 | 旧请求不得覆盖后续草稿 |
| Unavailable | Disabled | 可编辑 | 等待可用 API 配置 |

`Show cancel` 默认 false，只有 Sending 且存在匹配请求 ID 时才开启。`Show context` 默认 false；展示可用上下文后 Checkbox 仍默认为 Unchecked。发送中不禁用整个应用、历史列表或草稿编辑器。

### 3.5 History Sidebar / Workspace

Sidebar 使用既有 `layout/directory/width=248`，默认 `248 × 620`；包含新建、Live Search、Sessions SLOT 和归档说明。Sessions 位于真实 FRAME `History viewport`（`438:18092`）中，可独立纵向滚动。

Workspace 默认 `1120 × 780`。API：`Title`、`Session identity`、`Show history`，以及 History / Messages / Composer 三个 SLOT。History visibility 在外层控制；切换内部会话不会重新展开隐藏侧栏。

Messages SLOT（`439:18130`）位于真实 FRAME `Messages viewport`（`439:18129`）。Composer SLOT（`439:18149`）在滚动区之外，保持输入位置稳定。Figma 不允许暴露 SLOT 中的实例；这些内容通过 SLOT 内的真实实例直接编辑，而非 detach。

760px Review 明确设置 `Show history=false`，不是宣称 Figma 会自动执行媒体查询。代码实现时由宿主布局策略决定何时收起和重新打开历史。

## 4. 验收中修正的问题

1. Message Header 初始固定 20px，但身份与复制按钮需要 32px，造成元数据裁切。三个 master Header 均改为内容自适应高度；深层实例复检通过。
2. 合并 variants 后共享 TEXT 默认值让所有 Composer 状态都显示 Empty 提示。状态说明改由 State 拥有；移除该共享 TEXT 属性，避免请求失败、取消等信息被错误覆盖。
3. Message 示例明确设置 User 与 Assistant 各自 Body；Attachment 增加固定只读 Kind 说明，避免共享默认文案模糊类型。
4. 原型内 Copy answer 的字宽舍入产生 1px 越界。仅本组 Message master 的真实按钮设为稳定 `80 × 32`，Header 重新计算，未修改全局 Button。
5. 非 Empty Composer 的输入内容局部绑定 `color/text/primary`，避免已输入草稿沿用 placeholder 外观。没有改变 Textarea 主组件。

## 5. Review

共 11 个保留验收夹具，内容均由真实组件实例组成：

| Review | 节点 |
|---|---|
| Light / 1120 | 440:18136 |
| Dark / 1120 | 440:18238 |
| Narrow / 760 / history hidden | 440:18340 |
| History Item 10-state matrix | 440:18442 |
| Attachment 2-kind matrix | 440:18555 |
| Message 3-kind matrix | 440:18580 |
| Composer 6-state matrix | 440:18619 |
| Long plain text + historical attachment | 440:18760 |
| Context explicit opt-in | 440:18831 |
| Empty history search | 440:18877 |
| Long conversation scroll | 447:19475 |

Long conversation 实例宽760、高620；消息 viewport 高354，实际内容高1120，纵向裁切与滚动是明确设计。Composer 位于滚动区外。此项记录为 **1 个有意的滚动溢出场景**，不能将其误算为错误，也不能声称滚动内容没有溢出。

## 6. 七个预置原型场景

| 场景 | 节点 | 验证重点 |
|---|---|---|
| Ready A | 444:18719 | A 草稿与发送入口 |
| Sending A | 444:19399 | A Pending、发送禁用、当前请求可取消 |
| Error A | 444:19915 | A 草稿保留，显式重试 |
| Cancelled A | 444:20364 | A 草稿保留，旧请求不可覆盖后续输入 |
| Browse B while A pending | 444:20813 | B 草稿独立，Pending A 不进入 B；发送仍受当前请求占用限制 |
| Browse B after A completed | 444:21265 | A 完成后仍停在 B，不自动跳走 |
| Completed A | 444:21629 | 回看 A 的预置答案 |

已连接并读回核对 **17 条原生连接**，7 个场景从入口均可到达。发送、取消、手动重试、模拟完成/失败、切换 A/B 与重置路径已接线。入口的重置自身没有导航意义，因此使用 Disabled，而不是创建非法自连接。

A 草稿在 Ready / Sending / Error / Cancelled 场景均为“请再列出两项需要人工确认的内容。”；两个 B 场景均保留“这是 B 的独立草稿。”。只有 Sending A 出现 `Pending / request-A-01`。

**这不是运行时端到端测试。** 新建、搜索、归档、清空、复制和历史折叠等未列出的动作仅展示设计与组件契约，不宣称全都已接线。没有真实 API、剪贴板或数据库写入；真实 requestId 取消、迟到响应、键盘焦点、屏幕阅读器与草稿持久化需要实现阶段测试。

## 7. 最终证据

- 新组件名称唯一；4 sets / 21 variants / 2 singles。
- 深层遍历包含 INSTANCE 与 SLOT，检查 1417 个可见节点、351 个可见嵌套实例；失效主组件引用0。
- **非预期越界0**；长会话夹具的预期纵向滚动溢出1。
- 未绑定可见 solid fill/stroke 0；本组自有文字缺失 Text Style 0；顶层根重叠0。
- 普通非 Disabled 文字测量579层，最低对比度 Light **4.5326557047:1**，Dark **6.8625187908:1**。
- 原型17条连接，失效目标0；7个场景可达。检查15个可见 Disabled 控件，其反应连接均为0。
- 344个当前可达的本组 primary node IDs 以无损区间写入状态文件。SLOT 中规范化后的虚拟实例按实际路径、主组件与原型连接另行定位，不编造连续 ID。
- 没有新增 variables、text styles 或 effects；没有改动应用代码、既有 Controls 主组件或发布 speculative Code Connect。

## 8. 下一阶段

**Patterns 六组完成。下一阶段：产品 Screens，尚未开始。**

继续执行17条页面链路与7个模块的 Screens 计划，先读取最新计划并选定当前屏幕范围；不要把本组 AI Chat Workspace 或预置 Lab 当作已完成的产品页面。恢复依据为本记录、`FIGMA_AI_CHAT_STATE.json`、主索引和实际 Figma 节点，不依赖只有标题的 handoff。
