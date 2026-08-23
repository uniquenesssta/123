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
3. Search Field、Textarea、Number、Date/Datetime、Password、File Upload。

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

### 待执行顺序

1. Searchable Combobox（不能以现有 Select 替代）；
2. Switch / Toggle；
3. Tabs / Segmented Control；
4. Data Table、Row、Cell、排序、选择、空状态、Pagination；
5. Dropdown Menu / Context Menu / Overflow Menu；
6. Dialog：普通、确认、危险确认；
7. Toast、Inline Alert、Blocking Message；
8. Accordion / Disclosure；
9. Progress、Skeleton、Empty State；
10. Avatar：球队、球员、默认占位。

下一项固定为：`Searchable Combobox`。完成并验收该组件前，不开始页面拼装。
## 组件族验收

- 尺寸、文字、图标、内边距、点击区均无溢出；
- Default / Hover / Pressed / Disabled 等适用状态完整；
- Danger 状态必须引用上述四个语义变量；
- 图标只通过 Icon Slot 嵌套；
- Tag 的可删除语义与 Badge 的状态语义保持分离；
- 变体、属性和默认值稳定，页面可直接实例化，禁止临时复制拼接；
- 每组完成后必须记录组件集 ID、变体 API、嵌套实例数量及 overlap / overflow 结果。
