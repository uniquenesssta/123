# Figma Data Table & Pagination

状态：完成并通过结构、变量绑定、真实数据、Light/Dark 与链路视觉验收。

- Figma file: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- Controls page: `02 · Controls`（`3:3`）
- Documentation root: `Section · Data Table & Pagination`（`228:1090`）
- Direct link: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=228-1090
- Date: 2026-08-23

## 1. Semantic boundary

- Data Table 负责表头、排序、选择、单元格、行状态以及 Loaded / Loading / Empty。
- Pagination 是独立 footer 组件；与 Data Table 保持 1120px 同宽，但不嵌入表格 master。
- Loading 不清空已有数据：行保持可见并禁止交互，进度提示放在状态区；这与现有 `aria-busy` 行为一致。
- Selection 使用现有 Checkbox master；Status 使用 Badge；操作使用 Button/Secondary；任何父组件都不复制这些底层路径。
- 代码当前表头仍是静态文本；Figma 的 Sort 属性是待实现的正式契约，不冒充已存在的代码行为。

## 2. Component inventory

| Component set | Node ID | Variants | Public API | Composition |
|---|---:|---:|---|---|
| `Building Blocks/Table Header Cell` | `230:1170` | 24 | `Type=Text/Number`; `Sort=None/Ascending/Descending`; `State=Default/Hover/Focus/Disabled`; `Label` | Asc/Desc 只引用 Chevron Up / Down master |
| `Building Blocks/Table Cell` | `235:1242` | 20 | `Type=Text/Metadata/Number/Status/Action`; `State=Default/Hover/Selected/Disabled`; `Primary`; `Supporting` | Status 嵌套 Badge；Action 嵌套 Button/Secondary |
| `Data Table/Row` | `236:1331` | 4 | `State=Default/Hover/Selected/Disabled` | Checkbox + 6 个 Table Cell instance |
| `Data Table` | `238:1663` | 3 | `State=Default/Loading/Empty` | Header Cell、Row、Checkbox、Spinner 的组合 |
| `Pagination/Page Button` | `239:1439` | 6 | `State=Default/Hover/Pressed/Focus/Current/Disabled`; `Label` | 无复制路径 |
| `Pagination/Nav Button` | `240:1447` | 10 | `Direction=Previous/Next`; `State=Default/Hover/Pressed/Focus/Disabled`; `Label` | 保留代码中的文字按钮，不新增无来源图标 |
| `Pagination` | `241:1477` | 4 | `State=First/Middle/Last/Busy`; `Summary` | Nav Button、Page Button、Spinner instance |

合计：7 个组件集、71 个变体。Controls 完成后总计 35 个组件集、322 个变体。

## 3. Tokens added

| Token | Variable ID | Value | Scope | WEB syntax |
|---|---|---:|---|---|
| `layout/table-header/height` | `VariableID:226:1090` | 34 | WIDTH_HEIGHT | `var(--ui-table-header-height)` |
| `layout/table-cell/padding-x` | `VariableID:226:1091` | 9 | GAP | `var(--ui-table-cell-padding-x)` |
| `layout/pagination/control-height` | `VariableID:226:1092` | 27 | WIDTH_HEIGHT | `var(--ui-control-height-sm)` |

已有 `layout/table-row/height`（`VariableID:95:8`）继续作为 36px 行高唯一来源。颜色、边框、radius、disabled、accent 与 spinner 均复用现有 semantic tokens。

## 4. Behavior contract

### Header and cells

- Header 高度固定 34px，默认仅保留 1px 底部分隔线；Focus 才显示完整 focus border。
- `Type=Number` 右对齐；其他类型左对齐。Sort 与 State 是独立轴。
- Ascending / Descending 必须映射到真实 `aria-sort`，不能只显示图标。
- Metadata 为双行信息；Number 使用主数值 + supporting confidence。
- Disabled 不叠加两次 opacity；文字保持可辨认，交互才被禁用。

### Rows and table

- Row 高度固定 36px；Selected 勾选 Checkbox，并显示 2px accent 左边缘。
- Hover / Selected / Disabled 同步到所有嵌套 Table Cell。
- Loading 保留最后一次数据，禁用行与排序交互，并显示 Spinner 文案。
- Empty 保留完整表头，让用户仍能理解字段；正文显示“没有匹配球员”和恢复建议。
- 1120px player-directory sample 列宽：44 / 280 / 220 / 120 / 120 / 140 / 196。

### Pagination

- Page Button 的 Current 与 Pressed 分离，Current 是持久状态。
- First 禁用 Previous；Last 禁用 Next；Busy 同时禁用 Previous / Next，避免重复翻页。
- Nav Button 保持“上一页 / 下一页”文字，与现有产品代码一致。
- Pagination footer 高度 48px；内部紧凑控件高度 27px。

## 5. Code mapping

Source baseline:

- `src/styles/entityCenter.css`: `3a08b2f2a4d571d5760204a0d16149ed9e6c73c8`
- `src/styles/visualSystem.css`: `56e44e574c4fc10667449c444d82bcf5b5e00072`
- `src/styles/app.css`: `6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214`
- `src/pages/players.ts`: `b406293fd35163bcfae6266b9d083ea3b144d2e5`
- `src/pages/teams.ts`: `677e4eb3b51f8efa73a3a5b1c148cafb30d93090`
- `src/pages/runs.ts`: `c4374872117775c17570a405afe63f73842d860a3`

| Figma | Code |
|---|---|
| Header 34 | final `.entity-data-table th { height: 34px }` |
| Row 36 | `--ui-table-row-height: 36px` |
| Cell X 9 | final `th/td { padding: 7px 9px }` |
| Row Hover | `tbody tr:hover` |
| Row Selected | `tr.active` / `tr.selected` plus accent inset edge |
| Table Loading | player main `aria-busy=true`; rows remain visible |
| Table Empty | `playerTableRows` loading / no-match rows |
| Pagination First | `pageNumber <= 1` disables previous |
| Pagination Last | `has_more === false` disables next |
| Pagination Busy | footer displays “正在加载下一页…” |

Code Connect 暂缓：当前页面函数输出字符串模板，没有可安全映射的可复用 Table / Pagination 代码组件。实现可复用组件后，再按上述同名属性连接；不得把 Figma set 绑定到整页字符串函数。

## 6. Accessibility contract

- 使用真实 `table / thead / tbody / th / td` 语义。
- Sort 状态同步 `aria-sort=none/ascending/descending`，表头必须可键盘聚焦。
- 全选与行选择 Checkbox 都必须有可访问名称。
- Disabled / Busy 同步原生 `disabled` 或 `aria-disabled`；Busy 容器同步 `aria-busy=true`。
- Empty 信息放在跨列 cell 内，不删除表头。
- “上一页 / 下一页”使用真实 button；边界状态不能只靠颜色表达。

## 7. Validation

- 新组件集 7 / 7；新变体 71 / 71。
- Header Cell 24 / 24；Table Cell 20 / 20；Row 4 / 4；Data Table 3 / 3。
- Page Button 6 / 6；Nav Button 10 / 10；Pagination 4 / 4。
- 嵌套 master 审计通过：Chevron、Checkbox、Badge、Button/Secondary、Spinner、Table Cell、Row、Page Button、Nav Button 均为 instance。
- Dark QA frame `242:1587` 显式绑定 `FIP Color / Dark`（mode `17:2`）。
- Light QA frame：`242:1441`；Dark QA frame：`242:1587`。
- Documentation section：1312 × 3031；Controls root：1440 × 12955；section 完整落在 root 内。
- 结构、真实数据、Loaded / Loading / Empty、First / Middle / Last / Busy 与 Light / Dark 截图检查 PASS。

## 8. Decision log

- 未导入 Material 3 / Simple Design System：当前文件没有订阅库，搜索也没有匹配资产；FIP 的紧凑桌面尺寸与代码语义优先。
- 分页继续使用文字 Previous / Next，因为现有代码没有左右箭头图标，不为视觉方便伪造实现来源。
- 排序能力在 Figma 中补齐，但明确记录为后续代码工作。
- 下一组：Dropdown Menu / Context Menu / Overflow Menu。
