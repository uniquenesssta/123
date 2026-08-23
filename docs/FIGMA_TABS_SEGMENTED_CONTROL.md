# Figma Tabs & Segmented Control

状态：完成并通过结构、变量绑定、Light/Dark 与真实数据视觉验收。

- Figma file: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- Controls page: `02 · Controls`（`3:3`）
- Documentation root: `Section · Tabs & Segmented Control`（`199:595`）
- Direct link: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=199-595
- Date: 2026-08-23

## 1. Semantic boundary

Tabs 与 Segmented Control 是两个独立组件族，不能互换：

- Tabs：可关闭的对象导航，对应工作区中已打开的球队、球员或其他对象。
- Segmented Control：同一工作区内的单选显示模式，对应 `detail | compare | cards`。
- 当前应用保留了 tabs、active tab 与 layout mode 的状态定义；现有 Teams / Players 页面暂未重新挂载旧 Tabs UI。本组先固定可复用底层 API，页面 Pattern 阶段再组合。

## 2. Component inventory

| Component set | Node ID | Variants | Public API | Composition |
|---|---:|---:|---|---|
| `Tabs/Close Button` | `202:832` | 5 | `State=Default/Hover/Pressed/Focus/Disabled` | 每个变体嵌套 1 个 16px Icon Slot |
| `Tabs/Item` | `209:690` | 20 | `Size=32/40`; `Selected=False/True`; `State=Default/Hover/Pressed/Focus/Disabled`; `Label`; `Show close` | 每个变体嵌套 1 个 `Tabs/Close Button` |
| `Tabs/Bar` | `213:940` | 6 | `Size=32/40`; `Active=1/2/3` | 每个变体只组合 3 个暴露的 `Tabs/Item` 实例 |
| `Segmented Control/Item` | `214:996` | 20 | `Size=32/40`; `Selected=False/True`; `State=Default/Hover/Pressed/Focus/Disabled`; `Label`; `Show leading` | 每个变体嵌套 1 个暴露的 Icon Slot |
| `Segmented Control` | `216:1036` | 6 | `Size=32/40`; `Selected=Detail/Compare/Cards` | 每个变体只组合 3 个暴露的 `Segmented Control/Item` 实例 |

合计：5 个组件集、57 个变体。

## 3. Tokens added

| Token | Variable ID | Collection / values | Scope | WEB syntax |
|---|---|---|---|---|
| `tabs/item-min-width` | `VariableID:198:595` | FIP Size / 88 | WIDTH_HEIGHT | `var(--tabs-item-min-width)` |
| `tabs/item-max-width` | `VariableID:198:596` | FIP Size / 200 | WIDTH_HEIGHT | `var(--tabs-item-max-width)` |
| `tabs/close-hit-area` | `VariableID:198:597` | FIP Size / 24 | WIDTH_HEIGHT | `var(--tabs-close-hit-area)` |
| `tabs/indicator-height` | `VariableID:198:598` | FIP Size / 2 | WIDTH_HEIGHT | `var(--tabs-indicator-height)` |
| `tabs/item-padding-x` | `VariableID:204:610` | FIP Size / 10 | GAP | `var(--tabs-item-padding-x)` |
| `segmented/item-min-width` | `VariableID:198:599` | FIP Size / 72 | WIDTH_HEIGHT | `var(--segmented-item-min-width)` |
| `segmented/container-padding` | `VariableID:198:600` | FIP Control Scale / 2, 3, 4 | GAP | `var(--segmented-control-padding)` |
| `segmented/item-padding-x` | `VariableID:204:611` | FIP Control Scale / 8, 12, 14 | GAP | `var(--segmented-item-padding-x)` |
| `segmented/item-height` | `VariableID:213:1133` | FIP Control Scale / 28, 34, 40 | WIDTH_HEIGHT | `var(--segmented-item-height)` |

9 / 9 变量唯一，均设置显式 scope 与 WEB code syntax。

## 4. Behavior contract

### Tabs

- `Selected` 与交互 `State` 独立；不能用 Hover 代替选中态。
- 选中指示线固定 2px，并绑定 accent 语义色。
- Close Button 的点击区固定 24 × 24；Hover / Pressed 使用 danger 语义，Focus 使用 focus border。
- 标签使用横向 FILL、单行、ENDING ellipsis。长对象名不会挤走关闭按钮。
- `Tabs/Bar` 不复制 Label、Close Button 或 Indicator；只允许实例化 `Tabs/Item`。
- 当前 Bar 固定 3 个对象位，`Active=1/2/3` 与代码的 active tab 索引映射。

### Segmented Control

- `Selected` 与交互 `State` 独立；每次只能有一个选中项。
- Item 内层高度为 28 / 34，容器 padding 为 2 / 3，因此组合后总高严格为 32 / 40。
- 选中默认使用 accent-soft；Pressed 使用 accent + on-accent；Focus 使用 inside focus stroke。
- `Show leading=false` 生成文字模式；Icon Slot 仍保持暴露以切换 Detail / Compare / Cards 图标。
- 当打开对象不足 2 个时，Compare 与 Cards 通过嵌套 Item 的 `State=Disabled` 禁用，Detail 保持可用。
- `Selected=Detail/Compare/Cards` 必须映射代码的 `detail | compare | cards`，不得改成含义不明确的 1 / 2 / 3。

## 5. Code mapping

Source baseline:

- `src/components/workspace.ts`: `d6c2ac64efa6ddbf88ca00445bd9d4c369cd7dcc`
- `src/app/viewState.ts`: `379b0b6d06b71980611fa90e5f289b4f4ac2df07`
- `src/styles/layout.css`: `f1ef772d053cf6e0b6bf377516b3f0edb2f5c4ad`
- `src/styles/entityCenter.css`: `3a08b2f2a4d571d5760204a0d16149ed9e6c73c8`

| Figma | Code |
|---|---|
| `Tabs/Bar.Active` | active tab position derived from `active_tab_id` |
| `Tabs/Item.Label` | workspace tab label |
| `Tabs/Item.Selected` | tab id equals `active_tab_id` |
| `Tabs/Close Button` | close action nested inside each object tab |
| `Segmented Control.Selected=Detail` | `layout_mode = "detail"` |
| `Segmented Control.Selected=Compare` | `layout_mode = "compare"` |
| `Segmented Control.Selected=Cards` | `layout_mode = "cards"` |
| Compare / Cards disabled | `tabCount < 2` |

Code Connect is intentionally deferred: the current source functions emit string templates and the production Teams / Players pages no longer render the legacy bar. Implement reusable code components first, then attach exact mappings; do not map these Figma sets to dead legacy markup.

## 6. Validation

- Tabs/Close Button: 5 variants; Icon Slot 5 / 5; hardcoded visual paints 0; overflow 0.
- Tabs/Item: 20 variants; Label / Show close references 20 / 20; nested Close Button 20 / 20; hardcoded visual paints 0; overflow 0.
- Tabs/Bar: 6 variants; nested Tabs/Item 18 / 18; exposed 18 / 18; wrong main component 0; hardcoded visual paints 0; overflow 0.
- Segmented Item: 20 variants; height token bindings 20 / 20; exposed Icon Slot 20 / 20; wrong Icon Slot source 0; hardcoded visual paints 0; overflow 0.
- Segmented Control: 6 variants; nested Items 18 / 18; exposed 18 / 18; height bindings 6 / 6; padding bindings 6 / 6; wrong main component 0; hardcoded visual paints 0; overflow 0.
- Real usage: long-label ellipsis PASS; one-tab contextual disable PASS; text-only mode PASS.
- Light / Dark screenshots PASS.
- Section size: 1312 × 1324; default layer names 0.
- Controls final count after this group: 28 component sets, 251 variants.

## 7. Decision log

- Material 3 and Simple Design System Tabs / Segmented assets were inspected but not imported: close affordance, compact desktop geometry, variant API and token model do not match FIP.
- Figma supersedes legacy approximately 34px segmented CSS: supported total heights are now 32 / 40.
- Tabs and Segmented Control remain separate families even when they appear in the same workspace toolbar.
- Next foundation group: Data Table, Row, Cell, selection, sorting, empty state, and Pagination.
