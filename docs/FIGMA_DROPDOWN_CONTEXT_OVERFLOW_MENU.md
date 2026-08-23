# Dropdown / Context / Overflow Menu

本文件只记录 Dropdown、Context 与 Overflow Menu 这一组底层组件，不展开其他 Controls、Patterns 或 Screens。

## Figma 位置

- 文件：https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- 页面：`02 · Controls`（`3:3`）
- 文档区：`Section · Dropdown, Context & Overflow Menu`（`249:1705`）
- 直接链接：https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=249-1705
- Controls 根：`48:2`

文档分组：

- Tokens：`249:1708`
- Menu Item：`249:1709`
- Separator & Note：`249:1710`
- Menu Surface：`249:1711`
- Dropdown Menu：`249:1712`
- Context Menu：`249:1713`
- Overflow Menu：`249:1714`
- Usage / Theme QA：`249:1715`

## 新增尺寸变量

均加入既有 `FIP Size` collection（`VariableCollectionId:17:4`），Value mode 为 `17:3`，并设置专用 scope 与 WEB code syntax。

| Token | Variable ID | Value | WEB syntax |
|---|---|---:|---|
| `layout/menu/width` | `248:1705` | 240 | `var(--ui-menu-width)` |
| `layout/menu/gap` | `248:1706` | 5 | `var(--ui-menu-gap)` |
| `layout/menu/item-padding-x` | `248:1707` | 11 | `var(--ui-menu-item-padding-x)` |
| `layout/menu/item-padding-y` | `248:1708` | 10 | `var(--ui-menu-item-padding-y)` |
| `layout/menu/note-padding-x` | `248:1709` | 6 | `var(--ui-menu-note-padding-x)` |
| `radius/menu` | `248:1710` | 11 | `var(--ui-menu-radius)` |

几何契约：Surface 宽 240、内边距 8、内容 gap 5、Item 宽 224、Item padding 11 × 10、Note 横向 padding 6、圆角 11。

## 组件清单

| Component | ID | Variants | API / purpose |
|---|---:|---:|---|
| `Building Blocks/Menu Item` | `254:1755` | 20 | Tone Neutral/Danger × Selected False/True × State Default/Hover/Pressed/Focus/Disabled；Label、Description、Shortcut、显示开关、Leading Icon swap |
| `Building Blocks/Menu Separator` | `255:1762` | 2 | Inset None/Leading |
| `Building Blocks/Menu Note` | `255:1763` | — | 单组件；Note 文本属性 |
| `Menu/Surface` | `259:1861` | 3 | Content Actions/Selection/Empty |
| `Dropdown Menu` | `263:1938` | 6 | State Closed/Hover/Pressed/Focus/Open/Disabled |
| `Context Menu` | `265:1924` | 3 | Target Run/Lineup/Match |
| `Overflow Menu` | `267:2007` | 6 | State Closed/Hover/Pressed/Focus/Open/Disabled |

新增合计：6 个 component sets、40 个 variants、1 个单组件。Controls 页面总计：41 个 component sets、362 个 variants。

## 结构与语义

### Menu Item

- 20 个变体完整覆盖中性/危险、未选/已选及五种交互状态。
- Focus 使用单一 1px focus stroke，不叠加双层描边。
- Disabled 使用禁用语义色与图标透明度，不再叠加整个组件透明度。
- 前导图标只通过 INSTANCE_SWAP 与既有图标 master；Check、Info 等继续保持真实 nested instance。

### Menu Surface

- Actions、Selection、Empty 三种内容状态闭合。
- Actions 组合 Open details、Duplicate、Archive、Delete；Delete 使用 Danger tone。
- Selection 组合 All results selected、Ready、Blocked。
- Empty 是真实空状态，不保留不可见幽灵按钮。
- Surface 仅使用 `FIP/Elevation/Subtle/Light` / `Dark`；不使用 Dialog elevation。

### Dropdown 与 Overflow

- Closed、Hover、Pressed、Focus、Open、Disabled 六态完整。
- Closed 状态不存在隐藏 Surface；Open 才组合真实 `Menu/Surface` instance。
- Dropdown 复用 `Button/Secondary`、Icon Slot、Chevron Down/Up。
- Overflow 复用 `Button/Icon`、Icon Slot 与 `Icon/Utility/More`（`88:747`），不复制 SVG path。
- Dropdown 左对齐，Overflow 右对齐。

### Context Menu

上下文行为以现有代码为语义基准：

| Target | Action | Note | Tone |
|---|---|---|---|
| Run | 从历史列表删除 | 底层运行、快照和收敛数据保留 | Neutral |
| Lineup | 删除或归档阵容版本 | 已引用版本保留模型血缘，未引用版本永久删除 | Danger |
| Match | 删除比赛 | 将同时删除该比赛的阵容和复盘数据 | Danger |

代码契约：`role=menu`、`role=menuitem`，打开后首项获得焦点；Escape、滚动与外部点击关闭；视口边缘保留 8px clamp。Figma 三个 Target 变体与该语义一一对应。

## 主题与视觉决定

- Light QA frame：`268:1993`
- Dark QA frame：`268:1994`
- 两个 frame 使用同一组真实组件 instance，并显式绑定 FIP Color 的 Light / Dark mode。
- Dark 下嵌套 Surface 使用 Dark Subtle elevation。
- 对比行采用 AUTO 高度，已修复初版固定高度造成的底部裁切。
- 现有代码 `.app-context-menu` 使用 `0 18px 50px rgba(0,0,0,.24)`；该阴影与已确认的细描边、低扩散方向不符。Figma 的 border + Subtle elevation 是正式设计覆盖，代码实现时应同步，而不是复制旧阴影。

## 来源

- `src/main.ts`：`65bb1744a63015c56f8be8258406304535accb45`
- `src/styles/app.css`：`6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214`
- `src/components/icons.ts`：`b0cef778236d596ab94ea0a6ff0083da0481d19f`

## QA

- 变体计数：20 + 2 + 3 + 6 + 3 + 6 = 40，全部与预期一致。
- hardcoded visual paints：0。
- placeholder nodes：0。
- duplicate component-set names：0。
- 新变量：6 / 6；`FIP Size` 总数 45。
- nested masters 复核通过：Menu Item、Separator、Note、Button/Secondary、Button/Icon、Icon Slot、Chevron、More 与 Menu Surface 均保持 instance composition。
- Light / Dark、Dropdown open、Context Lineup focus、Overflow open 截图检查通过。
- 整组文档区为 1312 × 2676；Controls 根已扩展至 1440 × 15671。
- 结果：PASS。

## 下一组

固定为：Dialog——普通、确认、危险确认。完成并验收底层组件前，不开始页面拼装。
