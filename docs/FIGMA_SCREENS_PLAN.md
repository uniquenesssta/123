# Product Screens Plan

**最新检查点：2026-09-30，S01 Dashboard 视觉与本页状态通过。**

视觉以 Figma 为准；应用源码只用于功能、交互、数据语义。唯一设计记录分支为 `ui-design-system`。所有六组 Patterns 已完成，不回退重做。

## 1. 当前进度与计数定义

- 产品路由视觉/本页状态验收：**1 / 17**。
- 尚未开始的产品路由：**16**。
- 完整跨路由原型集成：**未完成**；当前完成0条完整跨路由验收链路。
- S01 有15个状态画面，仍只计1条路由。Review Hub、此前 Pattern QA 与 Lab 不计入产品路由。
- 当前状态文件：[FIGMA_SCREENS_STATE.json](FIGMA_SCREENS_STATE.json)。当前独立记录：[S01 Dashboard](FIGMA_SCREEN_DASHBOARD.md)。

本轮仅推进 S01，没有制作其他16个产品页面，也没有修改应用代码。后续每次只推进当前约定页面或直接相关依赖，完成后保存可恢复状态。

## 2. 执行顺序

以下按现有 Shell 的7个模块和17个路由确定顺序。标题为工作范围名称，具体屏幕文案在执行该页时再与对应语义源码核对；路由键是稳定身份。

| 阶段 | 模块 | 路由 | 页面范围 | 视觉与本页状态 | 跨页面接线 |
|---|---|---|---|---|---|
| S01 | home / 首页 | dashboard | 数据总览、快捷入口、系统详情、连接引导 | 已验收 | 待目标页面完成 |
| **S02** | matches / 比赛 | **lineups** | **比赛与阵容** | **下一项，未开始** | 未开始 |
| S03 | matches / 比赛 | prediction | 赛事推演 | 未开始 | 未开始 |
| S04 | matches / 比赛 | review | 赛后复盘 | 未开始 | 未开始 |
| S05 | matches / 比赛 | runs | 运行记录 | 未开始 | 未开始 |
| S06 | resources / 资源 | teams | 球队 | 未开始 | 未开始 |
| S07 | resources / 资源 | players | 球员 | 未开始 | 未开始 |
| S08 | resources / 资源 | lineup_presets | 阵容预设 | 未开始 | 未开始 |
| S09 | resources / 资源 | workbooks | Excel 工作包 | 未开始 | 未开始 |
| S10 | model / 模型 | rules | 规则与模型 | 未开始 | 未开始 |
| S11 | model / 模型 | release | 发布验收 | 未开始 | 未开始 |
| S12 | analysis / 分析 | analytics | 分析 | 未开始 | 未开始 |
| S13 | ai / AI | api_workspace | AI 问答 | 未开始 | 未开始 |
| S14 | ai / AI | openai | 兼容 API 配置 | 未开始 | 未开始 |
| S15 | management / 管理 | database | 数据库 | 未开始 | 未开始 |
| S16 | management / 管理 | logs | 问题日志 | 未开始 | 未开始 |
| S17 | management / 管理 | architecture | 架构信息 | 未开始 | 未开始 |

路由与模块依据：[PC App Shell](FIGMA_PC_APP_SHELL.md)。新页面存在真实产品目标后再回填 S01 快捷入口与 Shell 的导航，不指向旧的空插槽或 QA 场景。

## 3. 单页验收契约

每页首先读取对应源码语义、相关组件记录与最新画布，确认页面身份、功能入口、适用状态、编辑/只读边界和必要依赖。源 CSS 不提供视觉尺寸。

页面用现有 Controls / Patterns 的真实实例组合；新增依赖必须说明缺口，不把整个产品画面新建为组件来增加库计数。不 detach、不导入整页截图伪装为可编辑设计。

状态按实际适用性覆盖默认、首次加载、保留数据刷新、空、错误、禁用、成功，以及有真实危险写操作时的确认。并非每一页都要伪造所有状态；不把 loading、unknown、0、unconfigured 与 unhealthy 混为一谈。

验证适用的Light/Dark、紧凑布局、文字换行、滚动区域、实例引用、变量绑定、对比度与控件目标。意图内的滚动溢出与非预期溢出分开记录；页面NAVIGATE连接与继承的组件CHANGE_TO反应分别统计。

组件库、本页原型、跨路由联调、真实应用运行时测试为不同验收层。原型模拟必须明确标记；不得把预置定时器或状态跳转称为真实网络、权限、输入、持久化或端到端测试。

## 4. S01 已完成与后续回填

S01完成15个状态画面、47条页面/验收目录连接；134个可见Disabled控件均无反应连接。视觉检查通过，跨路由待回填目标为 `lineups / prediction / review / analytics / players`。首页快捷卡尚未执行真正页面跳转，待目标创建后集成。

S01 的新恢复文件已取代此前 AI Chat 检查点作为项目当前进度。既有 `FIGMA_COMPONENT_RECORD.md` 中 Screens未开始的文字是组件阶段历史快照，不是当前状态；本轮不重写未变更的组件API记录。

## 5. 下一次从 S02 开始

读取 `main/src/pages/lineups.ts` 及其直接引用的阵容编辑/状态相关源码，确认比赛选择、阵容创建/编辑/保存等实际功能；复用已完成 Shell、Page Bars、Panel、工作区和表单组件。只为该页面所需依赖补缺，不借此重做已验收整个组件库。

S02尚未开始，不因为S01快捷入口存在就计作S02已完成。完成S02后再把S01的 `lineups` 入口接到真实产品画面，并分别记录该页与导航集成结果。

## 6. 恢复顺序

README → 本计划 → `FIGMA_SCREENS_STATE.json` → 当前页独立记录 → 实际 Figma 节点。此前 AI Chat / Workflow 状态文件保留为历史记录。检查点中的 `currentScreenPhase` 与 `crossRouteIntegration` 应分别读取，不能把一个已验收画面等同于全产品集成完成。
