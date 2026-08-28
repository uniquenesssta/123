# Figma · PC App Shell

日期：2026-08-28。此记录只覆盖Patterns首组，不代表业务Screens或应用实现完成。

- [Figma组件区](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=370-2)
- [交互验收入口](https://www.figma.com/proto/PN0Whgu6HLWIHx4Mv6aHfu?node-id=389-24211&starting-point-node-id=389%3A24211)
- [Figma验收入口画布](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=389-24211)

## 1. 设计依据与边界

**设计依据（2026-08-28 用户确认）：所有视觉尺寸、比例、间距、字号和颜色均以已确认的 Figma 为准。应用源码只用于核对功能入口、交互和数据语义，不再作为视觉尺寸参考；旧记录中的源码尺寸仅是历史取证。**

继续使用已批准的191个变量、13个文字样式和6个效果样式，本组不新增或修改共享token值。源码只核对7模块、17入口、工作区历史和重置范围等功能语义。

未修改应用代码，未发布推测性的Code Connect。旧源码中不同的按钮尺寸或CSS断点不用于本组。界面文本、导航比例和状态视觉均由Figma已有组件决定。

## 2. 尺寸合同

| 对象 | Figma值 | 依据 |
|---|---:|---|
| 主导航宽度 | 60 | layout/nav/primary-width，VariableID:95:3 |
| 二级导航宽度 | 154 | layout/nav/secondary-width，VariableID:95:4 |
| Topbar高度 | 44 | layout/topbar/height，VariableID:95:2 |
| Icon Action | 32 ×32 | 既有Button/Icon32及FIP Control Scale |
| 主导航项 | 52 ×48 | 主栏60减两侧spacing/xs；高度Control Scale48 |
| 二级导航项 | 138 ×40 | 二级栏154减两侧spacing/sm；高度Control Scale40 |
| 组件圆角 | 7 | radius/md |
| 常用间距 | 4 /8 /12 /16 | spacing/xs、sm、md、lg |
| 图标 | 16 /20 /24 | 既有Icon Slot及FIP Icon Scale |
| 键盘焦点 | 1px内描边 | color/border/focus与control/stroke |
| 当前PC几何下限 | 960 ×600 | 本组Figma设计合同；不由旧CSS断点推导 |

导航文字复用FIP/Caption或FIP/Label，Topbar定位文字复用FIP/Data/Value。长导航项和定位文字单行省略，不缩小字体。工作区填充剩余空间，正文按已有文字样式换行。

## 3. 组件清单

文档根370:2，Patterns页面3:4。新增 **9个组件集 /60个变体**：

| 组件集 | Figma节点 | 变体数 |
|---|---|---:|
| Shell/Icon Action | 373:79 | 5 |
| Shell/Primary Navigation Item | 374:183 | 10 |
| Shell/Secondary Navigation Item | 374:321 | 10 |
| Shell/Service Status | 374:340 | 3 |
| Shell/Workspace History | 375:217 | 4 |
| Shell/Primary Rail | 376:855 | 7 |
| Shell/Secondary Navigation | 377:1511 | 17 |
| Shell/Topbar | 378:1109 | 2 |
| PC App Shell | 380:1366 | 2 |

生产单组件：Shell/Brand（375:221）、Shell/Workspace Slot（379:1087）。

QA专用单组件：Shell QA/Workspace Fixture（385:11899），只验证插槽替换、预置历史与状态边界，不纳入生产组件或业务Screens。

### 属性与组合

- Icon Action：State为Default /Hover /Pressed /Focus /Disabled；内部是真实Button/Icon32实例，图标经Button →Icon Slot替换。
- 两类Navigation Item：Selected False /True ×五种State；Label可编辑、Icon为暴露的Icon Slot。已选中同时有背景、图标和边缘指示，不只改变文字颜色。
- Service Status：Configured /Unconfigured /Loading。复用Badge与Spinner16；文案分别为“数据库已配置”“等待数据库”“正在载入”。
- Workspace History：Empty /Back /Forward /Both，两个真实Icon Action实例，禁用边界不占用新的变体维度。
- Primary Rail：Active为7个模块键。品牌顶部、主题底部，栏宽固定、高度填充。
- Secondary Navigation：Page为17个页面键，同时决定模块标题、菜单集合和唯一选中项。Version为运行时文本；缺失时不编造版本号。
- Topbar：Navigation Expanded /Collapsed；Location文本及History、Configuration、Reset嵌套组件独立暴露。
- PC App Shell：Navigation Expanded /Collapsed；Workspace为INSTANCE_SWAP，Loading为BOOLEAN。路由消费者同步Primary.Active、Secondary.Page和Topbar.Location。

根组件将二级栏放入独立的Secondary region。显隐由该容器承担，避免嵌套Page变体切换恢复可见性。已实测根Navigation切换后模块、页面、内容文字和主题保持。

### 复用依赖

Button/Icon（123:209）、Button/Primary（51:414）、Button/Ghost（52:255）、Badge（66:410）、Spinner（126:294）、Progress Bar（340:6522）、Blocking Message（304:4348）、Dialog/Confirmation（280:2814）、Toast（302:4210）、Icon Slot（37:44）。

Dialog Light效果仍为0 /10 /24 /4.5%，showShadowBehindNode=false；Dark仍为0 /12 /28 /26%。仅在深色确认实例选择既有Dark样式，没有改变共享效果定义。

## 4. 导航映射

一级模块点击进入该模块默认页；不是恢复该模块上次访问的叶子页。

| 模块 | 默认页 | 页面入口 |
|---|---|---|
| 首页 / home | dashboard | dashboard（数据总览） |
| 比赛 / matches | lineups | lineups（比赛中心）；prediction（赛事推演）；review（赛后复盘）；runs（推演记录） |
| 资源 / resources | teams | teams（球队中心）；players（球员中心）；lineup_presets（阵容预设）；workbooks（Excel 工作包） |
| 模型 / model | rules | rules（赛事设置）；release（发布验收） |
| 分析 / analysis | analytics | analytics（分析与历史） |
| AI / ai | api_workspace | api_workspace（AI 问答）；openai（兼容 API） |
| 管理 / management | database | database（数据库）；logs（问题日志）；architecture（系统信息） |

业务Screens仍未开始；导航验收实例中央明确标识为“导航验收”，不是已完成业务页面。

## 5. 行为合同与原型

### 全局导航、主题和折叠

- 只折叠二级栏，主栏与主题操作持续可用；展开入口位于Topbar。
- 切换主题或显隐保留当前页、工作区内容及详情历史。
- 模块或页面导航清理当前详情历史；点击当前选中页在原型中为空操作。
- 数据加载时仍能选择其他模块；异步旧响应不得覆盖新页面。

### 工作区历史

Back /Forward属于右侧工作区详情栈，不是浏览器历史。第一条详情返回基础页；前进恢复对应预置数据。QA使用基础页 →详情A →详情B的预置栈，逐一展示空边界、可后退、可前进和双向状态。

QA文字是预置样例，不宣称验证了真实输入框或滚动持久化。对应应用实现必须另测表单值、滚动与敏感字段排除。

### 状态、加载与错误

database_configured只支持“数据库已配置”的事实，不推断已连接或服务健康。状态标签使用已有primary文字色，避免浅色画布上的低对比组合；Badge母版和共享颜色值未改动。

导航加载使用既有Compact Indeterminate Progress。已有内容保留；QA遮罩只覆盖Workspace，操作按钮禁用。加载失败进入Blocking Message，可查看问题日志或重试；重试样例自动返回预置基础页。本组未用Skeleton替代保留内容。

### 重置

重置范围仅为当前页面的筛选、选中和临时工作区，不改变主题、导航显隐或已持久化的业务数据。

有未保存本地更改时采用确认对话框；取消不改状态，确认进入清空后的工作区样例。普通导航样例展示确认 →Toast反馈 →关闭。此保护确认是设计合同，现有直接重置逻辑尚未按此实现。

### 原型规模

| 类型 | 数量 |
|---|---:|
| 17入口 ×2主题 ×2显隐 | 68 |
| 7个历史/异步/重置样例 ×2主题 ×2显隐 | 28 |
| 重置确认与反馈覆盖层 | 8 |
| 原型起点 | 1 |
| 从起点可达的原型Frame | 105 |
| 独立尺寸/长文案样例，不计入原型Frame | 4 |

唯一原型起点为389:24211。正常导航、QA状态和覆盖层均有确定目标；被禁用的Back、Forward、Reset和样例按钮没有可触发反应。预置加载/重试分别使用2.2s /1.2s演示时序，不代表实际请求耗时。

## 6. 图标新增依赖

| 图标 | 节点 | 来源 |
|---|---|---|
| Arrow Left | 372:235 | [Lucide arrow-left](https://github.com/lucide-icons/lucide/blob/main/icons/arrow-left.svg) |
| Arrow Right | 372:266 | [Lucide arrow-right](https://github.com/lucide-icons/lucide/blob/main/icons/arrow-right.svg) |
| Contrast | 372:297 | [Lucide contrast](https://github.com/lucide-icons/lucide/blob/main/icons/contrast.svg) |

文档区372:226。三枚母版均绑定FIP Icon Scale笔画与FIP Icon Tone颜色，经Icon Slot接入。图形来源只用于图标几何，不是应用页面尺寸来源。

独立Icon母版34 →37；Icon Slot首选替换15 →18，保留原默认值及条目。AppIcon源码语义覆盖仍为25 /25，没有把新增Figma依赖冒充已实现代码能力。

## 7. 验收结果

| 检查 | 结果 |
|---|---|
| 组件集与变体 | 9 /60，与范围一致 |
| 204个已审计直接嵌套实例母版引用 | 全部可解析 |
| 自有未绑定视觉paint、缺失文字样式、默认名、重复集名、残留placeholder、broken alias | 均为0 |
| 尺寸：960 /1280 /1440 /1920 ×Light/Dark ×展开/折叠 | 16组通过 |
| 根显隐变体切换保留嵌套模块、页面、文字 | 通过 |
| 导航目标映射检查 | 1292项，0问题 |
| 禁用操作检查 | 196项，0可触发反应 |
| 原型可达性 | 105 /105 |
| 8个实际实例中的158个有效文字层 | 最低对比度4.533:1；无低于4.5项 |
| 焦点描边与画布对比度 | Light4.23:1 /Dark7.28:1 |
| Controls保持 | 61组 /506变体；1440 ×34245，无本组修改 |
| 共享变量 /文字样式 /效果样式 | 191 /13 /6，无增改 |
| Screens | 0个子节点 |

已查看组件、图标、双主题展开/折叠、加载、错误、确认、窄宽与宽屏长文案截图；图层几何与原型反应均读回检查。

透明度例外：已有control/disabled-opacity存储为0.42。原生opacity变量绑定按百分比解释成0.0042，因此新导航禁用态直接应用该既有token的解析值0.42，并在组件说明中记录；未改变共享token或其他组件。

上述结果是Figma结构、截图及原型目标检查，**不是应用端自动化或人工浏览器端到端测试**。验收统计不包含未来页面的业务行为。

## 8. 功能依据与后续实现

功能核对固定于main提交4c007be04f9fab4adf6399f7b216b50a88d650e4：

| 文件 | 仅用于核对 |
|---|---|
| src/app/navigation.ts | 模块与17条入口映射 |
| src/app/shell.ts | 主题、折叠、状态及操作语义 |
| src/app/viewState.ts | 工作区状态边界 |
| src/app/modal.ts | 右侧详情历史与表单/滚动快照 |
| src/main.ts | 导航协调、加载及重置行为 |

应用代码保持不变。实现阶段需补充：

1. 按Figma变量和实例尺寸实现，不复制旧CSS比例。
2. 正确的导航链接、aria-current、图标按钮可访问名称、Tab/Shift+Tab/Enter及焦点返回；确认对话框管理焦点。
3. Workspace的aria-busy /inert、过期请求丢弃及失败重试；尤其单独检查各页面分支的异步保护。
4. 详情历史的真实表单/滚动恢复、敏感字段不持久化、导航清栈与主题保持。
5. 未保存更改的重置确认、取消保护、仅清除当前工作区和成功反馈。
6. 低于当前960px PC合同的布局需要另行设计，不能据此宣称移动端已完成。

## 9. 下一组

继续Patterns：**Page Heading /Toolbar /Filter Bar /Selection Command Bar**。先完成模式，再进入Screens。各组继续独立归档。
