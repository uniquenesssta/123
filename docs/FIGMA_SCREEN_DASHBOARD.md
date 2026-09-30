# Screens · S01 Dashboard / 数据总览

**本页视觉与本页状态验收：2026-09-30。**

Figma 是视觉尺寸、比例、间距、字号和颜色的唯一依据。应用源码只用于确认功能入口、交互和数据语义。设计记录分支为 `ui-design-system`；本轮没有修改应用源码、既有组件主节点或全局变量。

## 1. 验收范围与计数

产品 Screens 已正式开始。本轮只完成 `dashboard` 一条页面链路的视觉组合、状态画面与有限本页原型：**1 / 17 条视觉页面已验收，16 条未开始**。本页的 15 个状态画面不是 15 个不同页面；Review Hub 也不计作产品页面。

跨页面原型连接仍待后续页面完成：五个快捷入口指向 `lineups / prediction / review / analytics / players`。这些目标当前只记录路由，不连接到旧 Pattern 验收示例冒充产品页面。因此，完整跨路由联调完成数仍为 0。

最新恢复文件：[Screens 状态](FIGMA_SCREENS_STATE.json)。下一项为 **S02：lineups / 比赛与阵容**，本轮未开始。

## 2. Figma 位置与状态画面

- File key：`PN0Whgu6HLWIHx4Mv6aHfu`
- Screens page：`3:5`
- 默认产品画面：`453:2`
- 验收目录：`458:7026`，非产品页面

| 状态 | 节点 | 尺寸 |
|---|---|---|
| Default Light | 453:2 | 1440 × 900 |
| Default Dark | 455:687 | 1440 × 900 |
| Compact Expanded | 455:1037 | 960 × 720 |
| Compact Collapsed | 455:1387 | 960 × 720 |
| System Details | 455:2226 | 1440 × 900 |
| Unconfigured | 456:1637 | 1440 × 900 |
| Connecting | 456:2359 | 1440 × 900 |
| Connection Error | 456:3079 | 1440 × 900 |
| First Load | 456:3747 | 1440 × 900 |
| Refreshing | 456:4300 | 1440 × 900 |
| Data Error | 456:4804 | 1440 × 900 |
| Empty Database | 456:5205 | 1440 × 900 |
| Connection Success | 456:5593 | 1440 × 900 |
| Desktop Collapsed Light | 458:5662 | 1440 × 900 |
| Desktop Collapsed Dark | 458:6344 | 1440 × 900 |

所有数值均为演示数据，不是实际数据库查询结果。1440 / 960 是本轮审查画幅，不代表已经实现自动断点；两个画幅都维持既有 FIP 字号与 Shell 尺寸。

## 3. 组合与来源

页面没有创建新的 COMPONENT 或 COMPONENT_SET，也没有 detach 既有实例。

| 作用 | 复用组件 |
|---|---|
| 一级/二级导航与顶栏 | PC App Shell，380:1366；Expanded 380:1086 / Collapsed 380:1226 |
| 页面标题 | Page Heading，404:14589 |
| 五项快捷入口 | Action Card，411:15483 |
| 数据概览 | Metric Card，410:15370 |
| 页面容器、系统详情、连接表单表面 | Panel，412:15426 |
| 系统信息展开 | Disclosure，323:5557 |
| 数据库连接输入 | Input/Password，147:616；Input/Number，144:437 |
| 状态反馈 | Inline Alert，303:4259；Empty State，343:6571 |
| 加载反馈 | Skeleton，342:6572；Spinner，126:294；Shell 内既有 Progress |

### Shell 内容接入

Shell 的 `Workspace` 是 INSTANCE_SWAP，不是假定存在的 SLOT。本页将其替换为既有 Panel，隐藏 Panel header、actions 和 divider，并通过局部实例覆盖去掉额外边框、圆角和背景。

在这个 Panel 的 Body SLOT 中放置本页创建的 `Dashboard viewport` Frame，然后放置 `Dashboard content` 自动布局列。滚动属性在可编辑的自有 Frame 上设置，不尝试覆盖不可修改的继承属性。外层 Body / viewport 保持 FILL；内部业务 Panel Body 保持 HUG。

语义子节点名称用于恢复：`Dashboard workspace` → `Body` → `Dashboard viewport` → `Dashboard content`。Figma 会规范化 SLOT 内虚拟节点 ID；更新内容时应依据稳定页面 ID 与这些语义层级重新定位，不能按过期虚拟 ID 猜测删除。

### 几何

保持已有 Shell：primary rail 60、secondary sidebar 154、topbar 44。页面内边距 x14 / y10，纵向区块间距16，卡片间距12。

- 1440 展开：内容宽1198；快捷卡3列、指标卡4列。
- 960 展开：内容宽718；快捷卡2列、指标卡2列。
- 960 折叠：内容宽872；快捷卡2列、指标卡2列。
- 1440 折叠：内容宽1352；快捷卡3列、指标卡4列。

上述列数和显式重排是 Figma 设计结果，不来自旧 CSS。实现层需要据此建立真正的响应式布局。

## 4. 业务内容和状态边界

首页标题为“今天要处理什么？”。五个快捷入口依次为录入比赛与阵容、开始赛事推演、完成赛后复盘、查看分析结论、查找或维护球员。系统信息默认收起；展开后展示模型注册信息与赛事、球员、模型、复盘四类数据范围。

默认示例显示数据库健康、球员1,248、比赛86、推演312。数据库“已配置”仅描述配置状态，不等同于“运行正常”；健康指标必须由健康读取结果提供。

| 状态 | 数据与交互规则 |
|---|---|
| Unconfigured | 连接地址为空；连接按钮 Disabled。业务统计显示“—”，不伪造为0。连接表单在本页出现。 |
| Connecting | 使用脱敏演示凭据；连接按钮与表单处于 Disabled，避免重复提交。 |
| Connection Error | 本页显示安全错误说明；地址只显示脱敏演示值；允许明确重试。 |
| First Load | 4个真实 Skeleton 实例；没有已取得的 Metric 数值。 |
| Refreshing | 保留上次成功读取的指标，不切回 Skeleton；明确标注是旧结果。 |
| Data Error | 保留旧业务统计；数据库健康改为“待复核”，提供重新读取入口。 |
| Empty Database | 已成功读取空数据库，业务统计才显示0；保留实际录入入口。 |
| Connection Success | 本页恢复正常概览并显示成功反馈；只是预置设计，不代表真的连接数据库。 |

连接表单保留源代码的语义：最大连接数默认10、范围1–100；连接超时默认10秒、范围1–120；连接地址按密码输入处理。不得将连接凭据写入日志正文。

**相对现有源实现的设计约束：**未知统计不再直接回落为0；未连接、连接中、连接错误、首次加载和刷新中的本页快捷卡采用 Disabled 状态。全局导航不因此被整页遮罩禁用。应用是否采用同样门禁，需要在实现阶段按本设计确认，而不是宣称源代码已经具备这些行为。

大表估算数量仍由宿主依据 `large_counts_are_estimates` 添加“约”前缀；Metric Value 是 TEXT，可以承载该文案，本轮未新增独立估算状态页。数据刷新、加载失败与错误脱敏均没有通过真实后端测试。

## 5. 本页原型与未接线部分

Review Hub 有15个状态入口。每个状态页通过 Esc 返回该目录。已读回核对 **47 条本轮 NAVIGATE 连接**：15个目录入口、15个返回目录连接，以及17个主题/侧栏/Disclosure/连接重试/读取重试和加载结果转换。

三个 AFTER_TIMEOUT 转换均为1.2秒预置模拟：Connecting → Connection Success、First Load → Default Light、Refreshing → Default Light。它们不验证表单、不发起请求、不执行数据库写入，也不证明实际连接必然成功。

深层反应中另有 **137 条继承自组件库的交互反应**，不计入本页导航连接数或产品页面可达数。最终从 Review Hub 可达的产品状态画面为15，而不是把组件变体目标混算进来。

本轮接线的本页功能包括：默认双主题切换、桌面侧栏折叠/恢复并保留主题、紧凑侧栏折叠/恢复、默认浅色系统详情展开/收起、连接失败重试与读取失败重试。其他状态中的全局操作没有承诺完整原型覆盖。

五个快捷入口、一级导航的其他模块目标、尚未创建的产品页面跳转仍待后续接线。连接输入和数字步进没有实现原型真实输入；Source 的运行时校验、键盘焦点、滚动持久化、异步竞态、历史与权限仍需应用测试。Reset workspace 在本页局部示例中为 Disabled，不把复位按钮冒充刷新。

## 6. 修正与最终验收

创建中发现并修复：网格实例误占整行、嵌套 Panel Body 固定高度导致溢出、Skeleton 说明行过宽、连接按钮高度收缩，以及 Shell 折叠后继承的外层 Panel 样式回退。修正只限本轮页面实例和自有布局。

普通文字对比度修正使用已有 `color/text/primary`，没有新增颜色或修改全局色值。首次修正81处局部文字，折叠版外层背景归一后再修正3处，共84处；没有把这些实例覆盖写回组件主节点。

最终深层验收包含 INSTANCE 与 SLOT 内部可见节点：

- 15个产品状态画面 + 1个非产品 Review Hub。
- 3,699个可见节点；1,422个可见嵌套实例均可解析。
- 非预期溢出0、失效主组件引用0、未绑定可见 solid paints 0、顶层重叠0。
- Connection Error 内容高838、可见滚动区域高836；这1处有意垂直滚动单独记录，不计作越界错误。
- 679个普通可见非Disabled文字层：Light最低4.5326557047:1；Dark最低6.8625187908:1。
- 47条自有页面导航连接，失效目标0；15个产品状态画面可达。
- 134个可见Disabled控件反应连接均为0。
- Screens 新主组件0；全局变量/文字样式/效果仍为193 / 13 / 6。
- Patterns仍为17 sets /125 variants /13 production singles；Controls仍为61 /506。

默认Light/Dark、紧凑展开/折叠、系统详情、连接错误的校正后截图已检查。以上是设计与有限原型验收，不是应用端到端通过声明。

## 7. 连续性与下一项

本轮先提交了进行中的 `FIGMA_SCREENS_STATE.json`，随后以验收结果更新。状态文件保存15个页面根、组件依赖、47条页面连接、49个实际主节点ID的无损区间、虚拟SLOT定位规则、未完成的跨页面集成及下一项。

恢复优先级：当前 README / Screens计划 → `FIGMA_SCREENS_STATE.json` → 当前页面独立记录 → 实际画布节点。AI Chat 与 Workflow 状态文件保留为历史检查点，不应再把 Screens 回退为未开始。`FIGMA_COMPONENT_RECORD.md` 仍是组件阶段记录，本轮没有新增组件，因此不重写其历史内容；其中 Screens未开始的描述由本次检查点明确取代。

**下一项：S02 `lineups` / 比赛中心 · 比赛与阵容。**

语义来源：`main/src/pages/dashboard.ts`，blob `9b975903e1e5d7384d34f158629817858470d765`；`main/src/components/databaseSetup.ts`，blob `4972b0fa669877b63afde483b6a8b119415856a6`。路由与Shell依据：`FIGMA_PC_APP_SHELL.md`。
