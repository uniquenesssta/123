# Dialog · 普通 / 确认 / 名称校验危险确认

> 状态：Figma 底层组件、示例与原型连接验收完成。记录日期：2026-08-27。
> 本次只修改 Figma 与 `ui-design-system/docs`，没有修改应用代码、执行删除操作或连接真实业务数据。

- [Figma 组件区](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=276-2210)
- [交互测试入口](https://www.figma.com/proto/PN0Whgu6HLWIHx4Mv6aHfu?node-id=286-3364)
- 页面：`02 · Controls`（`3:3`）；根节点：`48:2`；本组 Section：`276:2210`。
- 新增 **6 个组件集 / 36 个变体 / 2 个单组件 / 5 个尺寸变量**。
- Controls 总计 **47 个组件集 / 398 个变体**。单组件不计入变体总数。
- 本组 Section 1312 × 7374；Controls 文档根 1440 × 23085。右侧另有 13 个组件原型测试 frame，不属于产品 Screens。

## 1. 发现、范围与复用决策

先检查代码最终执行函数、现有组件、变量与可用设计库，再确定拆分。现有库检索未找到可直接满足 FIP 紧凑桌面几何、公开属性和状态模型的资产；保持本地设计语言，不导入另一套 tokens。

本组复用 Button/Primary、Secondary、Danger、Icon、Loading，Input/Text Field、Spinner、Icon Slot、Info / Alert Circle / Close 图标、Light/Dark 语义变量及已确认 Dialog elevation。没有新增图标路径、输入框、按钮或 Alert 母版。

普通编辑、普通确认与不可撤销危险确认分开；可编辑内容通过 Header、Body、Facts、Actions 的嵌套实例公开。不把所有行为压入一个巨大组件集。每个 set 最多 8 个变体，低于 30 个拆分阈值。

## 2. 代码来源与真实语义

### 2.1 实现边界

`src/app/modal.ts` 中的 `ModalController` 当前把内容推入工作区详情页，渲染的是 `workspace-detail-page` 和 `role="region"`，不是已实现的模态覆盖层。`runPendingAction()` 先关闭详情，再等待 action。

因此本组 Overlay、Submitting、Error、焦点锁定与返回触发器均是明确的设计/实现合同，不能宣称为现有运行时代码。现有页面式详情流程可保留；未来接入本组组件时应明确选择真正 modal 或 workspace region，不能只换外观而保留错误语义。

### 2.2 动作映射

| 代码入口 | Figma 映射 | 必须保留的业务后果 |
|---|---|---|
| 保存当前阵容为预设 | Standard / Form | 允许编辑名称；提交中保留输入；失败不清空草稿 |
| 历史推演移出历史列表 | Confirmation / Neutral | 仅隐藏历史记录；模型运行、概率矩阵、快照、复盘和收敛血缘全部保留 |
| 阵容历史删除 | Confirmation / Danger 使用实例 | 未被正式推演、快照、球员贡献引用的版本永久删除；被引用版本归档并隐藏，模型血缘保留；必要时恢复上一有效活动版本 |
| 比赛永久删除 | Confirmation / Danger | 仅允许没有受保护 P4 血缘的比赛；删除关联阵容、赛果与普通复盘；普通模型运行和特征快照保留为历史记录并解除比赛关联；已有 P4 研究、冻结或正式赛后结算时拒绝永久删除 |
| `requestDatabaseReset` | Typed Danger | 清空应用数据并重建空白结构；数据库本身、结构和本机连接配置保留；必须输入完整数据库名 |
| `previewForceDeleteTeam` / `forceDeleteTeam` | Typed Danger 的名称门槛可复用 | 先取得后端预检影响范围和 `confirmation_text`，再精确校验；完整影响预览属于后续 Pattern，不能省略为单一输入框 |

来源定位：`src/main.ts` 数据库重置约 1394 行、球队强删预检约 3055 行、阵容/历史/比赛动作约 5487–5538 行。行号只作辅助，以以下源文件 SHA 和函数名为准。示例球队名与 `football_local` 为设计样例，不代表读取了用户数据库。

| 来源文件 | 本次读取的 Git blob SHA |
|---|---|
| [src/app/modal.ts](https://github.com/uniquenesssta/123/blob/main/src/app/modal.ts) | `750bed8b30083e27c6ec5d264d51ea18636e7717` |
| [src/main.ts](https://github.com/uniquenesssta/123/blob/main/src/main.ts) | `65bb1744a63015c56f8be8258406304535accb45` |
| [src/styles/visualSystem.css](https://github.com/uniquenesssta/123/blob/main/src/styles/visualSystem.css) | `56e44e574c4fc10667449c444d82bcf5b5e00072` |
| [src/styles/layout.css](https://github.com/uniquenesssta/123/blob/main/src/styles/layout.css) | `f1ef772d053cf6e0b6bf377516b3f0edb2f5c4ad` |
| [src/styles/app.css](https://github.com/uniquenesssta/123/blob/main/src/styles/app.css) | `6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214` |
| [src/styles/components.css](https://github.com/uniquenesssta/123/blob/main/src/styles/components.css) | `88f3d0ffae592bfa41f41a85a040667df1094e52` |

## 3. Tokens 与几何

所有新变量属于既有 `FIP Size`（`VariableCollectionId:17:4`）的 Value mode（`17:3`），无新 collection、颜色副本或 `ALL_SCOPES`。

| Token | Variable ID | Value | Scope | WEB code syntax |
|---|---|---:|---|---|
| `layout/dialog/width-compact` | `VariableID:275:2210` | 480 | WIDTH_HEIGHT | `var(--ui-dialog-width-compact)` |
| `layout/dialog/width-wide` | `VariableID:275:2211` | 640 | WIDTH_HEIGHT | `var(--ui-dialog-width-wide)` |
| `layout/dialog/header-min-height` | `VariableID:275:2212` | 46 | WIDTH_HEIGHT | `var(--ui-dialog-header-min-height)` |
| `layout/dialog/header-padding-y` | `VariableID:275:2213` | 9 | GAP | `var(--ui-dialog-header-padding-y)` |
| `layout/dialog/body-padding-y` | `VariableID:275:2214` | 11 | GAP | `var(--ui-dialog-body-padding-y)` |

- 480 / 640 为本次明确新增的 Compact / Wide 设计规格，不伪装成当前源码宽度。
- 最终 dense CSS 的 Header min-height 46、padding-y 9、Body padding-y 11 是几何来源；复用水平 padding 12、footer padding-y 8、gap 8 和 radius 9。
- 默认 Header 带副标题及 32px Close 实例，实际高度 54；46 是最小高度，不应把所有 Header 强制压成 46。
- 普通操作与关闭按钮为既有 32px 控件；文本输入沿用既有 40px Input/Text Field。表单不临时压缩既有输入框母版。
- 外框和 Header / Actions 分隔使用既有 1px 语义描边；不叠加两层 Focus 外框。
- Body 默认随内容增高；需要长内容时由容器约束 Body 高度并开启纵向滚动，Header / Actions 留在可视区。
- 真实视口接入应遵守左右各 12px 避让，并根据可用高度限制 Body；Figma 的固定画板不是浏览器响应式或移动端实现。

### 保留已确认的阴影

| 主题 | Effect style | 值 |
|---|---|---|
| Light | `S:f3fdd50cae221eac265bed3e9ad85427784b3563,` | 0 10 24 rgba(34,50,72,.045)，透明区域后方阴影关闭 |
| Dark | `S:3d867eeafb12e9a6ff85a325e7e4a76eb487dd32,` | 0 12 28 rgba(0,0,0,.26) |

不得恢复已被用户否定的旧 CSS 重阴影。暗色样例切换 FIP Color / Dark 并使用现有 Dark effect，仍引用同一套组件母版。

## 4. 组件与公开 API

属性以显示名列出；Figma 的 `#...` 后缀由 API 分配，不应硬编码到产品代码。

| 对象 | ID | 数量 | API |
|---|---|---:|---|
| Building Blocks/Dialog Header | `277:2341` | 6 variants | Tone Neutral/Danger × State Default/Focus/Busy；Title、Subtitle；Show subtitle、Show leading、Show close |
| Building Blocks/Dialog Facts | `278:2257` | single | Label 1/Value 1、Label 2/Value 2、Label 3/Value 3；Show second fact、Show third fact |
| Building Blocks/Dialog Body | `278:2354` | 4 variants | Content Message/Facts/Form/Typed；Description、Error；Show description、Show error；暴露 Facts/Field |
| Building Blocks/Dialog Actions | `278:2630` | 8 variants | Tone Primary/Danger × State Default/Focus/Disabled/Loading；Show cancel；暴露 Cancel/Confirm |
| Dialog/Standard | `279:2795` | 6 variants | Size Compact/Wide × State Default/Submitting/Error；Body content INSTANCE_SWAP |
| Dialog/Confirmation | `280:2814` | 6 variants | Tone Neutral/Danger × State Default/Submitting/Error；暴露 Header/Body/Actions |
| Dialog/Typed Danger | `280:3600` | 6 variants | State Empty/Editing/Valid/Mismatch/Submitting/Error；暴露 Header/Body/Actions |
| Dialog/Modal Layer | `281:2955` | single | Dialog content INSTANCE_SWAP；语义 overlay fill 与居中容器 |

### 嵌套编辑规则

- 标题、副标题和 Close/Leading 显隐从 Header 修改；小图标继续通过 Icon Slot 交换。
- Facts 默认两行、第三行关闭；第二、第三行均可隐藏。长值自动换行；第三行用于保护/保留信息，不占位撑高。
- 通用 Facts 默认后果为“请以操作前的影响检查为准”；具体业务实例必须覆盖成真实后果。
- Form/Typed 的标签、输入、辅助文字和错误态从暴露的 Field 实例修改。
- Cancel / Confirm 文案从暴露的 Button 实例修改。Actions 没有伪造顶层文字映射，也没有复制 Button 内部文字层。
- Standard 的 Body content 应交换为本组四种 Body 母版。更复杂的表单属于后续 Pattern，不拆开复制现有组件。
- Modal Layer 的 1120 × 450 仅为文档预览尺寸；Dialog content 支持替换，实际应用容器必须适配视口。

### 稳定变体节点

| 组件 | 状态 → ID |
|---|---|
| Header Neutral | Default `277:2212`；Focus `277:2233`；Busy `277:2254` |
| Header Danger | Default `277:2275`；Focus `277:2297`；Busy `277:2319` |
| Body | Message `278:2266`；Facts `278:2269`；Form `278:2279`；Typed `278:2319` |
| Actions Primary | Default `278:2357`；Focus `278:2405`；Disabled `278:2434`；Loading `278:2473` |
| Actions Danger | Default `278:2502`；Focus `278:2541`；Disabled `278:2570`；Loading `278:2609` |
| Standard Compact | Default `279:2337`；Submitting `279:2506`；Error `279:2673` |
| Standard Wide | Default `279:2445`；Submitting `279:2597`；Error `279:2734` |
| Confirmation Neutral | Default `280:2531`；Submitting `280:2658`；Error `280:2727` |
| Confirmation Danger | Default `280:2594`；Submitting `280:2692`；Error `280:2770` |
| Typed Danger | Empty `280:2817`；Editing `280:2933`；Valid `280:3038`；Mismatch `280:3141`；Submitting `280:3240`；Error `280:3339` |

## 5. 状态与操作合同

| 状态 | 确认 | 取消 / 关闭 | 输入和错误 |
|---|---|---|---|
| 普通 Default | 可用 | 可用 | 表单可编辑 |
| Confirmation Default | 可用，Danger 使用危险文案 | 可用；默认焦点落在 Cancel | 必须展示对象及后果 |
| Typed Empty / Editing | 禁用 | 可用 | 尚未满足名称条件 |
| Typed Mismatch | 禁用 | 可用 | 显示字段错误和期望名称 |
| Typed Valid | 可用 | 可用 | `trim(input) === expectedName`，大小写敏感 |
| Submitting | Loading，禁止重复调用 | 禁用；不响应 Esc / backdrop 关闭 | 保留内容；表单禁用 |
| 普通 Error | 可重试 | 可用 | 保留草稿并显示错误 |
| 破坏性操作 Error | 先检查结果与保护条件 | 可用 | 不能因网络失败推断后端未执行；Typed 示例先返回名称复核 |
| 已命中 P4 保护 | 永久删除禁用 | 只允许关闭 | 明确说明阻止原因，不把它当成可盲目重试的临时错误 |

键盘与运行时待实现要求：

- 真正 modal 使用合适的 dialog 语义、可访问名称和说明；在打开时设置初始焦点、约束 Tab/Shift+Tab，并在关闭后恢复触发器。
- Esc 只在非 Submitting 时允许关闭；默认不启用点击遮罩关闭，尤其是危险确认。
- Enter 不能绕过字段校验、Disabled 或 Busy。后端调用前再次验证精确名称与保护条件。
- 失败状态保留输入；未知提交结果先查询后端状态，再决定是否重试。
- 纯 Figma 原型不能证明浏览器焦点锁定、真实输入、请求幂等、后端保护或屏幕阅读器行为。上述项目需要后续实现及测试。

## 6. 实例与视觉 QA

### 母版实例关系

以下为每个 set/单组件内的后代 INSTANCE 数量，包含继承的多层 Icon Slot、图标和按钮；不是独立母版数量。

| 母版 | ID | 后代实例数 |
|---|---|---:|
| Building Blocks/Dialog Header | `277:2341` | 30 |
| Building Blocks/Dialog Body | `278:2354` | 4 |
| Building Blocks/Dialog Actions | `278:2630` | 18 |
| Dialog/Standard | `279:2795` | 68 |
| Dialog/Confirmation | `280:2814` | 68 |
| Dialog/Typed Danger | `280:3600` | 75 |
| Building Blocks/Dialog Facts | `278:2257` | 0 |
| Dialog/Modal Layer | `281:2955` | 12 |

总计 **275**；主组件缺失 **0**。Standard、Confirmation、Typed Danger 的每个变体直接组合 Header、Body、Actions；不复制其内部路径或字段。

### 检查结果

| 检查 | 结果 |
|---|---|
| 新 set / variant 数量 | 6 / 36，符合矩阵 |
| Controls 总数 | 47 sets / 398 variants |
| 新变量 scopes 与 WEB syntax | 5/5 |
| FIP collections | Primitives 53；Color 41；Size 50；Icon Scale 3；Control Scale 15；Icon Tone 1 |
| 缺失主组件 / 未解析变量绑定 | 0 / 0 |
| 硬编码视觉 fills/strokes | 0 |
| 默认命名 / 残留占位节点 | 0 / 0 |
| 意外溢出 / Auto Layout 重叠 / 根级重叠 | 0 / 0 / 0 |
| 明暗主题、危险保护、长内容 | 截图人工检查通过 |
| 已计算的文本/动作语义色对比 | Light 最低约 4.50:1；Dark 最低约 6.57:1 |

对比度数据仅覆盖已计算的主要文本、辅助文本、危险文本和危险按钮色对，不代表对所有像素或所有可访问性要求的认证。

唯一有意裁切为压力测试中的滚动 Body（`I285:3503;279:2459`）：640 × 138 内容视口，内部说明高 414；`overflowDirection=VERTICAL`、`clipsContent=true`。Header 和 Actions 不滚出可视区域。

### 视觉证据节点

| 内容 | 节点 |
|---|---|
| 同母版 Light / Dark 对照 | `283:3032`；Light `283:3033`；Dark `283:3418` |
| 长标题、滚动正文、长对象名、隐藏副标题 | `285:3500`；Wide `285:3503`；Compact `285:3610` |
| 阵容删除/归档与 P4 阻止示例 | `288:3928`；Lineup `288:3932`；Protected Match `288:4010` |
| Typed Submitting 与模拟结果入口 | `286:3971` |
| 整组缩略排布复查 | `276:2210` |

各母版在构建阶段分别进行了结构与截图检查；最终又复查主题、内容压力、Busy 和整组排布。整组缩略图只用于排布复核，不替代单组件可读性检查。

## 7. 原型测试连接

Figma NAVIGATE 要求目标为同页不同顶层 frame，因此测试场景放在 Controls 根节点右侧。没有新建页面或产品屏幕。

| 场景 | Frame ID |
|---|---|
| Launcher | `286:3364` |
| Standard Default / Submitting / Error | `286:3418` / `286:3458` / `286:3526` |
| Confirmation Default / Submitting / Error | `286:3566` / `286:3603` / `286:3670` |
| Typed Empty / Editing / Valid | `286:3707` / `286:3769` / `286:3910` |
| Typed Mismatch / Submitting / Error | `286:3844` / `286:3971` / `286:4048` |

- 47 条 reaction 的触发类型和目标回读一致；从 Launcher 可到达全部 13 个场景。
- 12 个不可操作入口没有 reaction：三个 Busy 场景的 Close/Cancel/Confirm，以及 Typed Empty/Editing/Mismatch 的 Confirm。
- 非 Busy 支持 Cancel、Close、Esc 回到入口；Typed Error 回到名称复核，而不是直接再次执行破坏性请求。
- 输入变化和服务端结果通过标注“模拟”的按钮演示，不是假装可输入的静态文本；没有连接任何真实删除 API。
- 这是 Figma 连接与禁用状态的验证，不是浏览器交互或应用端到端测试。

## 8. 实施顺序与 Code Connect 决策

暂不建立 Code Connect：当前应用不是这些可复用 modal 组件的等价实现，强行连接会造成错误映射。

后续实现依次完成：

1. 建立真正的 Dialog 容器与 Header/Body/Actions API，并明确与 workspace region 的职责边界。
2. 映射本组 token、输入/按钮/图标母版及 Light/Dark elevation。
3. 完成 focus trap / restore、Esc / backdrop / Enter 规则和可访问名称。
4. 完成保留草稿的 Submitting/Error、重复提交保护和未知结果复核。
5. 接入历史隐藏、阵容引用归档、比赛 P4 保护及名称确认前的后端预检。
6. 建立可复用代码组件后再添加 Code Connect，并测试真实键盘、请求和保护路径。

本组没有提前构建复杂表单、球队强删影响预览 Pattern 或产品 Screens。

## 9. 交接与下一组

- README、总组件记录和基础组件 backlog 与本文件同步更新。
- 8 个新 master 的 documentationLinks 指向本文件。
- 已保留此前完成的 41 个组件集；本次未修改应用源码或既有用户确认的 Dialog effect 值。
- 下一组固定为 **Toast / Inline Alert / Blocking Message**。当前底层组件阶段还剩 4 组，完成后再进入 Patterns / Screens。
