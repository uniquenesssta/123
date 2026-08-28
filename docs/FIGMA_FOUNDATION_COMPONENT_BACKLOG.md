# Figma Foundation & Component Backlog

只记录 tokens、底层组件及其验收顺序；页面与模式不在本文件处理。

## 基础依赖状态

状态：已完成并通过 Figma 结构、变量绑定与视觉检查。

Figma 记录：

- 页面：`00 · Foundations`（`0:1`）
- 总根节点：`Foundations · Icon-first`（`23:4`）
- 本轮文档区：`Section · Desktop dependencies`（`100:2`）
- 直接链接：https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=100-2

### 桌面布局尺寸

| Token | Variable ID | Value |
|---|---|---:|
| `layout/topbar/height` | `95:2` | 44 |
| `layout/nav/primary-width` | `95:3` | 60 |
| `layout/nav/secondary-width` | `95:4` | 154 |
| `layout/directory/width` | `95:5` | 248 |
| `layout/inspector/entity-width` | `95:6` | 300 |
| `layout/inspector/workspace-width` | `95:7` | 360 |
| `layout/table-row/height` | `95:8` | 36 |
| `layout/page/padding-x` | `95:9` | 14 |
| `layout/page/padding-y` | `95:10` | 10 |
| `layout/panel/padding` | `95:11` | 12 |

最终值以当前高密度桌面 CSS 为准；旧草案中的 64 / 68 / 188 等尺寸不再作为实现依据。

### Overlay / Busy

- `color/overlay/modal`（`96:6`）：Light `rgba(25,39,47,.46)`；Dark `rgba(2,7,10,.82)`
- `color/overlay/busy`（`96:7`）：Light `rgba(235,242,245,.72)`；Dark `rgba(4,9,13,.68)`
- 两个语义变量都以 Light / Dark alias 连接 primitive，并限定为填充用途。

### Danger Action

| State | Semantic variable |
|---|---|
| Default | `color/action/danger`（`97:8`） |
| Hover | `color/action/danger-hover`（`97:9`） |
| Pressed | `color/action/danger-pressed`（`97:10`） |
| Disabled | `color/action/danger-disabled`（`97:11`） |

四个状态均有 Light / Dark 值，适用于 fill 与 stroke；Disabled 使用 32% alpha，而不是临时降低组件整体透明度。

### Dialog 阴影

- `FIP/Elevation/Dialog/Light`（`S:f3fdd50cae221eac265bed3e9ad85427784b3563,`）：`0 10px 24px rgba(34,50,72,.045)`
- `FIP/Elevation/Dialog/Dark`（`S:3d867eeafb12e9a6ff85a325e7e4a76eb487dd32,`）：`0 12px 28px rgba(0,0,0,.26)`
- 2026-08-23 视觉纠偏：旧值 Light `0 26 70 / 22%`、Dark `0 28 80 / 42%` 扩散过大，正式作废。
- Light 最终由用户在 Figma 中调整为 4.5% alpha，并关闭“透明区域后方显示阴影”；保留该人工调整。
- 本项为已确认的设计覆盖；后续代码实现必须同步 Figma 新值，不得继续复制旧 CSS 阴影。

### 数据与技术文本

- `FIP/Data/Metric`（`S:b1aec1946253febd06f32d41e2ed98c30ab2a110,`）
- `FIP/Data/Value`（`S:2c5079f86251cec2cdeaf53b4a7c7ddf5e6beadd,`）
- `FIP/Data/Table`（`S:d57c2965cadbb03f04eddb40b216f03a250392c0,`）
- `FIP/Technical/Code`（`S:4c3672e04e1d45f7dadb92be7160b3e97aed588a,`）
- `FIP/Technical/Label`（`S:5520710a58f884a71af9c4f22d9528eba93ea39f,`）

### 来源与验收

代码基准：

- `src/styles/visualSystem.css`：`56e44e574c4fc10667449c444d82bcf5b5e00072`
- `src/styles/layout.css`：`f1ef772d053cf6e0b6bf377516b3f0edb2f5c4ad`
- `src/styles/app.css`：`6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214`

验收结果：

- 新变量 26 / 26 可解析，语义变量双模式完整；
- broken aliases 0，变量重名 0，新增样式重名 0；
- 文档节点 97，默认命名 0，占位节点 0；
- overlap 0，overflow 0；
- 10 个尺寸卡片均为 252 × 120；
- Foundations 全页视觉检查通过；
- Dialog effect style 与 Light/Dark 示例绑定一致，示例文字 overflow 0；Light 4.5% alpha 与 transparent-area shadow disabled 复核通过。

## 底层组件

### 已完成

1. Danger Button、Icon Button、Loading Button；
2. Spinner（作为 Loading Button 的直接依赖提前完成）；
3. Search Field、Textarea、Number、Date/Datetime、Password、File Upload；
4. Searchable Combobox 与其 Option building block；
5. Switch / Toggle 与其 Switch Control building block；
6. Tabs / Segmented Control 与 Close Button、Item building blocks；
7. Data Table / Pagination 与 Header Cell、Cell、Row、Page Button、Nav Button building blocks；
8. Dropdown / Context / Overflow Menu 与 Menu Item、Separator、Note、Surface building blocks；
9. Dialog：普通、确认、名称校验危险确认，与 Header、Body、Facts、Actions、Modal Layer；
10. Toast / Inline Alert / Blocking Message，与 Feedback Content、Action、Close、Task Activity；
11. Accordion / Disclosure，与 Disclosure Trigger、Content building blocks；
12. Progress / Skeleton / Empty State，与 Skeleton Block building block；
13. Avatar：球队、球员缩写与默认占位，复用 Icon Slot。

按钮首组的组件 ID、变体 API、嵌套关系、代码约束与验收证据单独记录在 `docs/FIGMA_BUTTON_COMPONENTS.md`，本文件不重复展开。

完成结果：

- `Button/Danger`（`119:209`）：12 变体；
- `Button/Icon`（`123:209`）：12 变体；
- `Spinner`（`126:294`）：6 变体；
- `Button/Loading`（`128:343`）：9 变体；
- 合计 4 个组件集、39 个变体，实例化检查全部通过，可见 overlap / overflow 为 0。

扩展字段组记录见 `docs/FIGMA_EXTENDED_FIELDS.md`：

- 前置图标：Plus（`139:208`）、Calendar（`139:212`）；
- 组件集：Search（`142:526`）、Textarea（`143:410`）、Number（`144:437`）、Date & Datetime（`146:488`）、Password（`147:616`）、File Upload（`149:906`）；
- 合计 6 个组件集、44 个变体；68 个 Icon Slot wrapper、72 个真实按钮实例、直接 icon master 实例 0；
- 组件集越界 0、根级重叠 0、截图检查 PASS。

Searchable Combobox 记录见 `docs/FIGMA_SEARCHABLE_COMBOBOX.md`：

- 新变量：`combobox/option-height`（`VariableID:161:514`）与 `combobox/option-padding-x`（`VariableID:161:515`）；
- 新图标：Chevron Up（`168:208`）；
- 组件集：`Building Blocks/Combobox Option`（`163:720`）与 `Input/Searchable Combobox`（`171:563`）；
- 合计 2 个组件集、14 个变体；Parent 内 27 个 Option 实例、9 个真实 `Button/Icon` toggle、直接 icon master 实例 0；
- 组件集网格重叠 0、复制 Option frame 0、截图检查 PASS。

Switch / Toggle 记录见 `docs/FIGMA_SWITCH_TOGGLE.md`：

- 新变量：`switch/track-width`（`178:563`）、`switch/track-height`（`178:564`）、`switch/thumb-size`（`178:565`）、`switch/track-padding`（`178:566`）、`switch/min-width`（`178:567`）；
- 组件集：`Building Blocks/Switch Control`（`183:563`）与 `Switch/Toggle`（`186:583`）；
- 合计 2 个组件集、20 个变体；Parent 内 10 个真实 Switch Control 实例，复制 Track / Thumb frame 0；
- Checked Off / On 与 Default / Hover / Pressed / Focus / Disabled 状态矩阵完整；
- 真实 API Readonly Context 样例 2 个；hardcoded visual paints 0；overlap / overflow 0；Light / Dark 截图检查 PASS。

Tabs / Segmented Control 记录见 `docs/FIGMA_TABS_SEGMENTED_CONTROL.md`：

- 新变量 9 个：Tabs 宽度、关闭点击区、指示线与水平 padding；Segmented 最小宽度、容器 padding、Item padding 与 inner height；
- 组件集：`Tabs/Close Button`（`202:832`）、`Tabs/Item`（`209:690`）、`Tabs/Bar`（`213:940`）、`Segmented Control/Item`（`214:996`）、`Segmented Control`（`216:1036`）；
- 合计 5 个组件集、57 个变体；Parent 内 18 个 Tabs/Item 与 18 个 Segmented Item 真实实例，全部保持暴露；
- Tabs 长名称单行省略、Segmented 总高 32 / 40、仅一个标签时 Compare / Cards 禁用；
- hardcoded visual paints 0；overflow 0；Light / Dark 截图检查 PASS。

Data Table / Pagination 记录见 `docs/FIGMA_DATA_TABLE_PAGINATION.md`：

- 新变量 3 个：表头高度 34、单元格横向 padding 9、分页控件高度 27；继续复用 36px table row；
- 组件集：`Building Blocks/Table Header Cell`（`230:1170`）、`Building Blocks/Table Cell`（`235:1242`）、`Data Table/Row`（`236:1331`）、`Data Table`（`238:1663`）、`Pagination/Page Button`（`239:1439`）、`Pagination/Nav Button`（`240:1447`）、`Pagination`（`241:1477`）；
- 合计 7 个组件集、71 个变体；排序、选择、Loaded / Loading / Empty、First / Middle / Last / Busy 状态闭合；
- Checkbox、Badge、Button/Secondary、Spinner、Header Cell、Table Cell、Row 与分页子组件全部保持 instance composition；
- Controls 总计 35 个组件集、322 个变体；Light / Dark 真实球员目录截图检查 PASS。


Dropdown / Context / Overflow Menu 记录见 `docs/FIGMA_DROPDOWN_CONTEXT_OVERFLOW_MENU.md`：

- 新变量 6 个：Menu 宽度 240、gap 5、Item padding 11 × 10、Note padding-x 6、Menu radius 11；
- 组件集：`Building Blocks/Menu Item`（`254:1755`）、`Building Blocks/Menu Separator`（`255:1762`）、`Menu/Surface`（`259:1861`）、`Dropdown Menu`（`263:1938`）、`Context Menu`（`265:1924`）、`Overflow Menu`（`267:2007`）；单组件 `Building Blocks/Menu Note`（`255:1763`）；
- 合计 6 个组件集、40 个变体、1 个单组件；Neutral / Danger、Selection、Empty、Open/Closed 与 Run/Lineup/Match 语义闭合；
- Button、Icon Slot、More、Chevron、Menu Item、Separator、Note 与 Surface 全部保持 instance composition；
- Controls 总计 41 个组件集、362 个变体；hardcoded visual paints 0，placeholder 0，Light / Dark 截图检查 PASS。

Dialog 记录见 [FIGMA_DIALOG.md](FIGMA_DIALOG.md)：

- 新增 5 个尺寸 token：Compact 480、Wide 640、Header min-height 46、Header padding-y 9、Body padding-y 11；保留用户确认的 Light 4.5% 阴影。
- 组件集：Header（`277:2341`）、Body（`278:2354`）、Actions（`278:2630`）、Standard（`279:2795`）、Confirmation（`280:2814`）、Typed Danger（`280:3600`）；单组件 Facts（`278:2257`）与 Modal Layer（`281:2955`）。
- 合计 6 个组件集、36 个变体、2 个单组件；275 个后代嵌套实例全部可解析。Controls 总计 47 个组件集、398 个变体。
- 真实语义覆盖历史隐藏、阵容版本删除/归档、比赛 P4 保护与精确名称校验；Busy 禁止关闭、取消和重复提交。
- Light / Dark、长标题/长对象名、滚动 Body 与危险操作保护截图检查 PASS；意外 overflow 0、overlap 0、hardcoded visual paints 0。
- 13 个原型测试状态可到达，47 条 reaction 连接和 12 个禁用入口回读通过；不等同于代码中的键盘或请求流程已实现。
- 当前代码仍是 workspace region；后续实现需补模态语义、焦点锁定/恢复、异步状态与后端保护复核。

Toast / Inline Alert / Blocking Message 记录见 [FIGMA_TOAST_INLINE_ALERT_BLOCKING_MESSAGE.md](FIGMA_TOAST_INLINE_ALERT_BLOCKING_MESSAGE.md)：

- 新增 7 个尺寸 token、1 个 Focus on Accent 语义颜色 alias；FIP Size 57、FIP Color 42。
- 组件集：Content（`299:4040`）、Action（`300:4242`）、Close（`301:4094`）、Toast（`302:4210`）、Inline Alert（`303:4259`）、Blocking Message（`304:4348`）；单组件 Task Activity（`305:4208`）。
- 合计 6 个组件集、34 个变体、1 个单组件；Controls 总计 53 个组件集、432 个变体。
- 短暂通知、持续通知、局部错误、恢复中、启动失败和非模态任务活动分别建模；关闭提示不取消任务，也不解除保存门禁。
- 280px Toast、300px Inline、320px Blocking 长文案与动作换行检查通过；Toast 和 Task Activity 在右下角间隔 8px 共存。
- Light / Dark 截图与整组截图已审阅；意外 overflow 0、root overlap 0、复制自有图标路径 0、hardcoded own visual paints 0。
- 18 个原型场景可达、45 条连接、7 个禁用入口回读通过；4 个局部错误场景的草稿值保持一致。
- Task Activity 正文改用现有 primary token，修正 Light 4.43:1 的不足；最终 Light / Dark 为 10.91:1 / 14.78:1。
- 当前代码需要补定时器身份检查、悬停/焦点暂停、点击区、ARIA 同步、只读恢复处理与错误脱敏；原型验证不等同于运行时已实现。

Accordion / Disclosure 记录见 [FIGMA_ACCORDION_DISCLOSURE.md](FIGMA_ACCORDION_DISCLOSURE.md)：

- 新增 3 个尺寸 token：Trigger min-height 34、padding-y 11、Accordion gap 7；单行实际高度 38。FIP Size 60，FIP Color 保持 42。
- 组件集：Trigger（`322:5430`）、Disclosure（`323:5557`）、Accordion（`325:5983`）；单组件 Content（`323:5179`）。
- 合计 3 个组件集、42 个变体、1 个单组件；Controls 总计 56 个组件集、474 个变体。
- Panel / Inline、五种交互状态、Single / Multiple 合法展开组合完整；Accordion 只引用 36 个 Disclosure Item 实例。
- 同一母版的 Light / Dark、280px 长标题与数量、300px Input/Inline Alert 内容替换、禁用正文可读性检查通过；无溢出、默认命名或残留占位。
- 16 个组件原型场景、70 条内部连接可达且可返回；标题单独控制 toggle，正文不误触折叠，两个禁用标题无反应；草稿值为静态模拟。
- 非禁用可见文字最低 4.789:1；Focus 对 canvas/surface 最低 4.228:1，继续复用 1px 细线，无双圈或阴影。
- Single 为新增设计能力；代码尚需验证初始 open 与空快照的区别、稳定 key、键盘/焦点、正文草稿及敏感字段排除。原型不等于运行时验证。

Progress / Skeleton / Empty State 记录见 [FIGMA_PROGRESS_SKELETON_EMPTY_STATE.md](FIGMA_PROGRESS_SKELETON_EMPTY_STATE.md)：

- 新增 12 个尺寸 token；FIP Size 72，颜色、字体与效果样式不新增。
- 组件集：Progress Bar（`340:6522`）、Skeleton Block（`341:6504`）、Skeleton（`342:6572`）、Empty State（`343:6571`）。
- 合计 4 个组件集、26 个变体；Controls 总计 60 个组件集、500 个变体。
- Progress 在 120px 保持 25 / 50 / 75 / 100 精确比例，Indeterminate 37.5%；代码仍接受连续 0–100。
- Skeleton 仅用于首次载入；翻页保留旧数据样例直接引用 Data Table Loading，Skeleton 数量为 0。
- Empty State 支持标题、说明、图标、单动作；长文案 Compact 增高到 126px，无裁切。
- Light / Dark、真实状态、长文案与对比度检查通过；hardcoded own visual paints、own vectors、broken masters、placeholder、default names 均为 0。
- 代码需补 Reduced Motion、稳定组件抽取和 ARIA；本轮未修改应用源码，未发布推测性 Code Connect。

Avatar 记录见 [FIGMA_AVATAR.md](FIGMA_AVATAR.md)：

- 新增 4 个尺寸 token、1 个头像文字语义 alias、3 个文字样式；FIP Size 76、FIP Color 43，变量总数 191。
- 新图标 Person（`356:212`）；球队占位复用 Shield。两者加入 Icon Slot preferred swaps，原 13 项保留，现为 15 项。
- 组件集 `Avatar`（`358:6798`），分区 `357:6965`；1 个组件集、6 个变体，Size 34 / 38 / 54 × Content Initials / Placeholder。
- 每个母版暴露 1 个真实 Icon Slot；23 个使用／压力测试 Avatar 实例，81 个后代实例母版均可解析。
- Light / Dark、宽字母与中文、248px 目录、36px 表格行、300px 速览检查通过；属性切换保留缩写与图标选择。
- 意外 overflow、overlap、未绑定自有颜色、paint 缓存不一致、broken masters、默认命名及残留占位均为 0；文字对比度最低 Light 4.867:1 / Dark 5.225:1。
- Controls 总计 **61 个组件集、506 个变体**。当前代码只支持缩写；默认占位与字符簇处理列为后续实现合同，本轮未修改应用源码。

### 完成状态与下一阶段

既定底层组件剩余 **0 组**，已全部完成并验收。

下一阶段为 [Patterns](FIGMA_PATTERN_SCREEN_PLAN.md)，先做 **PC App Shell**。Patterns 与 Screens 仍为空；先完成模式，再拼装产品页面。组合时若发现新依赖，仍须先归档并验收。

## 组件族验收

- 尺寸、文字、图标、内边距、点击区均无溢出；
- Default / Hover / Pressed / Focus / Disabled 等适用状态完整；
- Danger 状态必须引用上述四个语义变量；
- 图标只通过 Icon Slot 嵌套；
- Tag 的可删除语义与 Badge 的状态语义保持分离；
- 变体、属性和默认值稳定，页面可直接实例化，禁止临时复制拼接；
- 每组完成后必须记录组件集 ID、变体 API、嵌套实例数量及 overlap / overflow 结果。
