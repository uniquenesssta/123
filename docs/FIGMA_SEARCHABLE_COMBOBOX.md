# Figma Searchable Combobox

本文件只记录 Searchable Combobox 组件族。扩展字段、按钮、基础 Token、图标总库与后续组件分别读取对应文档。

- 状态：已完成并通过结构与视觉 QA
- 日期：2026-08-23
- Figma 文件：https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- 页面：`02 · Controls`（`3:3`）
- 文档根：`Component · Searchable Combobox`（`160:514`）
- Option 文档组：`Group · Combobox Option`（`160:517`）
- Master 文档组：`Group · Searchable Combobox`（`160:520`）

## 1. 代码来源与边界

本组件按应用分支真实实现校准：

- `src/components/searchableSelect.ts`，SHA `fec3425e8e9fc6f9234924246f7f87cabb1836bb`
- `src/styles/components.css`，SHA `88f3d0ffae592bfa41f41a85a040667df1094e52`

设计边界：

- 原生单选 `select` 继续作为权威值；输入框只负责模糊查询和键盘导航。
- 这是独立的 searchable combobox，不以 `Select/Dropdown`（`63:158`）替代。
- 不支持 `multiple` 或 `size > 1`；显式 native/off 标记继续跳过增强。
- 查询标准化、模糊匹配、最多 120 条结果、IME composition 与 remount 草稿恢复属于实现契约。

## 2. 前置依赖

### 新增图标

- `Icon/Utility/Chevron Up`：`168:208`
- 图标文档卡：`Utility · Chevron Up`（`168:206`）
- 24px master；1.75 stroke；颜色与粗细绑定变量；已加入 `Icon Slot` preferred swaps。
- 展开态通过 Icon Slot 正式替换 Chevron Up，不旋转实例、不 detach、不复制路径。

### 新增几何变量

位于 `FIP Control Scale`（`VariableCollectionId:46:2`）：

| Variable | ID | Modes 32 / 40 / 48 | WEB syntax |
|---|---:|---|---|
| `combobox/option-height` | `VariableID:161:514` | 32 / 36 / 40 | `var(--searchable-select-option-height)` |
| `combobox/option-padding-x` | `VariableID:161:515` | 8 / 10 / 12 | `var(--searchable-select-option-padding-x)` |

组件默认使用 40 模式：Control 40px、Option 36px、Option 横向 padding 10px。

## 3. 组件事实清单

| Component set | Node ID | Variants | Public API |
|---|---:|---:|---|
| `Building Blocks/Combobox Option` | `163:720` | 5 | State `Default/Hover/Keyboard Active/Selected/Disabled`; Label |
| `Input/Searchable Combobox` | `171:563` | 9 | State `Default/Focus/Open/Querying/Keyboard Active/Selected/Empty/Query Restored/Disabled`; Label; Empty message; Show label/helper |

状态专用的 Query/Value 与 Helper 文案保留在各母版中，没有暴露成组件集级 Text Property。原因：Figma 的组件集 Text Property 只有一个全局默认值，会把九态示例压成同一内容。实例仍可直接编辑对应文本层；未来 Code Connect 应把生产 `value/query/helper` props 映射到这些层。

## 4. 完整交互链

| State | Required meaning |
|---|---|
| Default | 未聚焦；显示占位；Chevron Down |
| Focus | Focus border；列表仍关闭 |
| Open | 当前选项可见；完整列表；Chevron Up |
| Querying | 查询草稿 `prem`；过滤后仅保留匹配结果 |
| Keyboard Active | 查询 `la`；La Liga 使用 Keyboard Active Option；保留下一候选 |
| Selected | La Liga 已提交到原生 select |
| Empty | 查询 `zzq`；显示 `No matching options`；Escape 可恢复 |
| Query Restored | remount 后恢复草稿 `prem`；权威选择不变 |
| Disabled | 输入与真实 Icon Button 均禁用；列表不可展开 |

键盘链：ArrowUp / ArrowDown 移动 active option；Enter 提交；Escape 取消查询并恢复已选值；`aria-activedescendant` 跟随 active option。

## 5. 复用关系

- Parent 每个变体包含 3 个 `Combobox Option` 实例；九态共 27 个 Option 实例。
- 每个变体包含 1 个真实 `Button/Icon` toggle；九态共 9 个 toggle button。
- Closed 使用 `Chevron Down`（`54:260`）；Open / Querying / Keyboard Active / Empty / Query Restored 使用 `Chevron Up`（`168:208`）。
- Selected Option 使用真实 `Icon Slot` + Check（`54:270`）。
- Listbox 统一绑定 `FIP/Elevation/Float/Light`，未误用 Dialog elevation。
- Parent 内不存在手绘 Option frame，也没有直接 Icon master 实例。

## 6. QA 结果

- Component Set：2 / 2。
- 变体：14 / 14（Option 5 + Combobox 9）。
- Component Set 变体网格重叠：0。
- Parent 中复制 Option frame：0。
- Parent Option 实例：27。
- Parent toggle button 实例：9。
- 直接 Icon master 实例：0。
- Listbox effect style 绑定：9 / 9（隐藏态也保留同一结构）。
- 控件展开方向：Down/Up 均为组件替换，不使用相对旋转覆盖。
- 文档占位：0。
- Controls 根：`1440 × 7966`；Extended Fields：`1312 × 3588`；新增文档完整包含。
- 整组截图检查：无文字换行错误、无边界越界、无变体重叠；结果 PASS。

## 7. 实现同步注意

1. 将 CSS 的 Option 高度与横向 padding 改为上述两个变量，避免继续硬编码 36 / 10。
2. 保持隐藏 native select 可提交、可触发 change；不要把可视 input 变成第二份权威状态。
3. Toggle 必须是 button，Input 必须使用 `role=combobox`、`aria-autocomplete=list`、`aria-expanded`、`aria-activedescendant`。
4. 保留查询草稿、composition 状态与 DOM remount 恢复诊断；不要在同步时用 selected label 覆盖活跃查询。
5. 空结果不可复用普通 disabled 文案；Selected 与 Keyboard Active 是不同状态。

## 8. 下一顺序

下一组固定为 `Switch / Toggle`。完成并验收该组件前，不开始页面拼装。
