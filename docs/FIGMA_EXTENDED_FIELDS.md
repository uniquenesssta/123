# Figma Extended Fields

本文件只记录扩展字段组；按钮、基础 Token、图标主库与后续 Patterns 分别读取对应文档。

- 状态：已完成并通过结构与视觉 QA
- 日期：2026-08-23
- Figma 文件：https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- 页面：`02 · Controls`（`3:3`）
- Controls 根：`48:2`，当前尺寸 `1440 × 7966`
- 组件文档区：`Section · Extended Fields`（`142:291`），当前尺寸 `1312 × 3588`
- 图标前置文档区：`Section · Extended field icons`（`139:213`）

## 1. 代码校准

本组不是按通用 UI Kit 猜测，而是按 `main` 分支真实控件链路补齐：

- `src/styles/app.css`（`6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214`）：通用 input / select / textarea、Focus ring、Textarea vertical resize。
- `src/styles/components.css`（`88f3d0ffae592bfa41f41a85a040667df1094e52`）：`secret-input-wrap` 的显示/隐藏文字按钮、Searchable Select 状态。
- `src/components/searchableSelect.ts`（`fec3425e8e9fc6f9234924246f7f87cabb1836bb`）：combobox 查询、展开、空结果、恢复链；该组件留到下一组单独制作。
- `src/pages/rules.ts`（`22b042a920f9841fe759d8d8e834d47b683c8c9b`）：Search、Date、Number、File 选择→校验→注册/取消/详情。
- `src/pages/openai.ts`（`1326c3fda076f9f1bca7459c3eac34ac0054f78e`）：Password + Show/Hide、Textarea、Number。
- `src/pages/players.ts`（`b406293fd35163bcfae6266b9d083ea3b144d2e5`）、`src/pages/teams.ts`（`677e4eb3b51f8efa73a3a5b1c148cafb30d93090`）、`src/pages/lineups.ts`（`d35d01fb28ca13b6fb5d07423d11112507ffb243`）：Date、Datetime-local、Number 与短/长 Textarea。

设计决策：Password 保留代码中的文字 `Show / Hide`，不虚构 Eye 图标；Date / Datetime 使用原生选择器语义；File Upload 同时覆盖拖放和代码中的文件选择链。

2026-08-23 Search Field 纠偏：Submit 模式的 Value 使用 12/16 紧凑排版、单行结尾省略并保持横向 FILL；Live 模式继续使用 14/18。Figma 不支持 CSS 容器查询，因此以 `Mode=Submit` 作为短宽度契约，而不是在任意拉伸宽度下自动缩放字体。修正节点：`142:431`、`142:456`、`142:481`、`142:506`。

## 2. 前置图标

| Master | Node ID | Contract |
|---|---:|---|
| `Icon/Utility/Plus` | `139:208` | 24px master；1.75 stroke；颜色与粗细绑定变量 |
| `Icon/Utility/Calendar` | `139:212` | 24px master；1.75 stroke；颜色与粗细绑定变量 |

两个图标只通过 `Icon Slot` 使用；Controls 内没有直接粘贴 SVG 路径。

## 3. 组件事实清单

| Component set | Node ID | Variants | Public API |
|---|---:|---:|---|
| `Input/Search Field` | `142:526` | 8 | Mode `Live/Submit` × State `Default/Focus/Filled/Disabled`; Label; Query/placeholder; Helper; Show label/helper |
| `Input/Textarea` | `143:410` | 8 | Height `96/160` × State `Default/Focus/Error/Disabled`; Label; Value/placeholder; Helper; Show label/helper/resize |
| `Input/Number` | `144:437` | 4 | State `Default/Focus/Error/Disabled`; Label; Value; Unit; Helper; Show label/helper/unit/stepper |
| `Input/Date & Datetime` | `146:488` | 8 | Type `Date/Datetime` × State `Default/Focus/Error/Disabled`; Label; Value/placeholder; Helper; Show label/helper/picker |
| `Input/Password` | `147:616` | 8 | Visibility `Hidden/Shown` × State `Default/Focus/Error/Disabled`; Label; Value; Helper; Show label/helper |
| `Input/File Upload` | `149:906` | 8 | State `Empty/Drag Over/Selected/Validating/Ready/Complete/Error/Disabled`; Label; File name; Support; Show label |

合计：6 个 Component Set，44 个变体。

## 4. 完整交互链

| Family | Required chain | Figma expression |
|---|---|---|
| Search · Live | Default → Focus → Filled → Clear → Default | Filled 才显示 Close；Close 经 Icon Slot |
| Search · Submit | Default → Focus/Filled → Search → Disabled | Submit 为 `Button/Secondary` 实例；短宽度 Value 为 12/16、单行省略 |
| Textarea | Default/Focus → content → Error or Disabled; compact/editor height | 96/160 两档；可选 native resize affordance |
| Number | Default/Focus → decrement/increment → Error or Disabled | `− / +` 为 32px `Button/Icon` 实例；Helper 存 min/max/step |
| Date / Datetime | Default/Focus → OS picker → value; required Error; Disabled | Calendar 为 `Button/Icon` 内的 Icon Slot |
| Password | Hidden → Show → Shown → Hide；Error/Disabled | Show/Hide 为 `Button/Secondary` 实例 |
| File Upload | Empty/Drag Over → Selected → Validating → Ready → Complete | 包含 Choose, Validate, Cancel, Details, Register, Replace, Retry, Remove |

File Upload 的危险语义已纠偏：`Remove` 使用 Danger；`Cancel` 保持 Secondary。Validating 中 Loading 动作位于 Cancel 前。

## 5. 依赖与复用

- Input 基础尺寸、Focus/Error/Disabled 颜色与现有 `Input/Text Field`（`57:313`）一致。
- Search、Textarea、Date、File feedback 图标均经 `Icon Slot`。
- Number 步进器与 Date picker 复用 `Button/Icon`。
- Search Submit、Password Show/Hide、File Upload 普通动作复用 `Button/Primary` / `Button/Secondary`。
- File Remove 复用 `Button/Danger`；Validating 复用 `Button/Loading`。
- 未创建重复按钮母版，未复制控件内图标路径。

## 6. QA 结果

- Component Set：6 / 6。
- 变体：44 / 44。
- 组件集边界越界：0。
- 默认组件命名：0。
- Controls 根级重叠：0。
- Controls 根尺寸覆盖内容：PASS（根高度 `7966`，新增 Searchable Combobox 文档完整包含）。
- Extended Fields 内 Icon Slot wrapper：68。
- Extended Fields 内真实按钮实例：72。
- Controls 直接 Icon master 实例：0。
- 截图检查：Search、Textarea、Number、Date/Datetime、Password、File Upload 全部通过；无文字裁切、重叠、重阴影或错误危险语义。

## 7. 后续衔接

Searchable Combobox 已作为独立组件完成，未用现有 `Select/Dropdown` 代替；完整节点、状态、依赖与 QA 见 `docs/FIGMA_SEARCHABLE_COMBOBOX.md`。下一底层组件为 `Switch / Toggle`。