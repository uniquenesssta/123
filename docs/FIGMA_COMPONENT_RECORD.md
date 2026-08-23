# FIP · Figma Component Record

> This branch is a design-system record branch. It intentionally contains no application source code.
> The record is the handoff source for implementing the Figma library back into code later.

- Repository: uniquenesssta/123
- Branch: ui-design-system
- Figma file: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- Figma file key: PN0Whgu6HLWIHx4Mv6aHfu
- Baseline date: 2026-08-23
- Design direction: desktop PC data workspace; icon-first; foundations before composed screens

## 1. Figma file structure

| Page | Node ID | Status |
|---|---:|---|
| 00 · Foundations | 0:1 | complete; desktop dependency extension verified |
| 01 · Iconography | 3:2 | application icon coverage complete and verified |
| 02 · Controls | 3:3 | baseline plus component extensions verified |
| 03 · Patterns | 3:4 | reserved; currently empty |
| 04 · Screens | 3:5 | reserved; currently empty |

Page documentation roots:

- Foundations · Icon-first: 23:4
- Desktop dependencies: 100:2
- Iconography · Production library: 27:2
- Code icon coverage: 83:206
- Controls: 48:2
- Section · Button: 48:5
- Section · Badge & Tag: 48:10
- Section · Spinner: 126:273
- Section · Extended Fields: 142:291
- Section · Switch & Toggle: 179:563\n- Section · Tabs & Segmented Control: 199:595

## 2. Foundations

### Variable collections

| Collection | ID | Modes | Count |
|---|---|---|---:|
| FIP Primitives | VariableCollectionId:17:2 | Value | 53 |
| FIP Color | VariableCollectionId:17:3 | Light, Dark | 41 |
| FIP Size | VariableCollectionId:17:4 | Value | 36 |
| FIP Icon Scale | VariableCollectionId:35:2 | 24, 20, 16 | 3 |
| FIP Control Scale | VariableCollectionId:46:2 | 32, 40, 48 | 15 |
| FIP Icon Tone | VariableCollectionId:50:206 | Default, Secondary, Active, On Accent, Success, Warning, Danger, Info | 1 |

All variables have explicit scopes and WEB code syntax.

### Text styles

- FIP/Page Title — Inter Semi Bold, 21/28
- FIP/Section Title — Inter Semi Bold, 14/20
- FIP/Body — Inter Regular, 12/18
- FIP/Label — Inter Medium, 11/16
- FIP/Caption — Inter Medium, 10/14
- FIP/Data/Metric — Inter Semi Bold, 20/24
- FIP/Data/Value — Inter Medium, 12/16
- FIP/Data/Table — Inter Regular, 11/16
- FIP/Technical/Code — Cascadia Mono Regular, 11/17
- FIP/Technical/Label — Cascadia Mono Semi Bold, 10/14

### Effect styles

- FIP/Elevation/Subtle/Light
- FIP/Elevation/Float/Light
- FIP/Elevation/Subtle/Dark
- FIP/Elevation/Float/Dark
- FIP/Elevation/Dialog/Light — 0 10px 24px rgba(34,50,72,.045); transparent-area shadow disabled
- FIP/Elevation/Dialog/Dark — 0 12px 28px rgba(0,0,0,.26)

### Desktop dependency extension audit

- Documentation section: `Section · Desktop dependencies` (`100:2`).
- Added variables: 26 total — 10 layout dimensions, 6 overlay variables, and 10 danger-state variables.
- Final desktop dimensions follow the current high-density CSS: topbar 44, primary rail 60, secondary sidebar 154, directory 248, entity inspector 300, workspace inspector 360, table row 36, page padding 14 × 10, panel padding 12.
- Added text styles: 5; added dialog effect styles: 2.
- Source SHAs: `visualSystem.css` `56e44e574c4fc10667449c444d82bcf5b5e00072`; `layout.css` `f1ef772d053cf6e0b6bf377516b3f0edb2f5c4ad`; `app.css` `6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214`.
- Design override: dialog elevation no longer follows the old source shadow values; see the correction record below.
- Validation: variables 26/26; broken aliases 0; duplicate variables/styles 0; documentation nodes 97; default names 0; placeholders 0; overlap 0; overflow 0.
- Result: PASS.

## 3. Iconography

### Icon Slot

- Component set: Icon Slot (37:44)
- Variants: Size=24, Size=20, Size=16
- Property: Icon#37:0 — INSTANCE_SWAP
- Default icon: Icon/Navigation/Dashboard (31:7)
- Locked geometry contract: 24px master grid, 20px optical area, round caps and joins
- Scaled strokes: 24px=1.75, 20px=1.5, 16px=1.25

### Existing icon masters

Navigation:

- Icon/Navigation/Dashboard — 31:7
- Icon/Navigation/Match Center — 39:48
- Icon/Navigation/Prediction — 40:74
- Icon/Navigation/Lineup Preset — 41:111
- Icon/Navigation/Release Acceptance — 43:148
- Icon/Navigation/Issue Log — 44:172

Entity and communication:

- Icon/Entity/Users — 88:218
- Icon/Communication/Chat — 88:462

Data:

- Icon/Data/Chart — 88:482
- Icon/Data/Database — 88:542

Layout:

- Icon/Layout/Panel Left — 88:602
- Icon/Layout/Panel Right — 88:622
- Icon/Layout/Cards — 88:706
- Icon/Layout/Detail — 88:727

Utilities:

- Icon/Utility/Search — 54:208
- Icon/Utility/Close — 54:249
- Icon/Utility/Chevron Down — 54:260
- Icon/Utility/Chevron Up — 168:208
- Icon/Utility/Check — 54:270
- Icon/Utility/Minus — 54:280
- Icon/Utility/Alert Circle — 54:290
- Icon/Utility/Shield — 83:259
- Icon/Utility/Sheet — 88:442
- Icon/Utility/Settings — 88:502
- Icon/Utility/History — 88:522
- Icon/Utility/Plug — 88:562
- Icon/Utility/Info — 88:582
- Icon/Utility/Refresh — 88:643
- Icon/Utility/Reset — 88:664
- Icon/Utility/Compare — 88:684
- Icon/Utility/More — 88:747
- Icon/Utility/Plus — 139:208
- Icon/Utility/Calendar — 139:212

### Code icon coverage audit

- Source: `src/components/icons.ts` on application branch `main`, source SHA `b0cef778236d596ab94ea0a6ff0083da0481d19f`.
- AppIcon code keys covered: 25 / 25.
- P4.1 new masters: 18 / 18.
- Documentation layout: section 83:206, grid 83:209, 6 columns × 3 rows.
- Binding checks: icon-tone variable 18 / 18; icon-stroke variable 18 / 18.
- Icon Slot replacement checks: 18 / 18.
- Geometry audit: overlap 0; grid overflow 0; card-child overflow 0.
- Naming audit: duplicate expected names 0; unnamed masters 0.
- Result: PASS.

## 4. Controls

Current controls state: 28 component sets and 251 variants. The Tabs / Segmented group adds 5 sets, 57 variants, 18 nested Tabs/Item instances and 18 nested Segmented Item instances; all new nested controls remain exposed, hardcoded visual paints are 0, overflow is 0, and Light/Dark QA passes. Detailed records live in `docs/FIGMA_BUTTON_COMPONENTS.md`, `docs/FIGMA_EXTENDED_FIELDS.md`, `docs/FIGMA_SEARCHABLE_COMBOBOX.md`, `docs/FIGMA_SWITCH_TOGGLE.md`, and `docs/FIGMA_TABS_SEGMENTED_CONTROL.md`.

| Component set | ID | Variants | API |
|---|---:|---:|---|
| Button/Primary | 51:414 | 12 | Size 32/40/48 × State Default/Hover/Pressed/Disabled; Label; Show leading; Show trailing |
| Button/Secondary | 52:2 | 12 | same API |
| Button/Ghost | 52:255 | 12 | same API |
| Button/Danger | 119:209 | 12 | Size 32/40/48 × State Default/Hover/Pressed/Disabled; Label; Show leading; Show trailing |
| Button/Icon | 123:209 | 12 | square Size 32/40/48 × State Default/Hover/Pressed/Disabled; exposed Icon Slot |
| Spinner | 126:294 | 6 | Size 16/20/24 × Tone Default/On Accent |
| Button/Loading | 128:343 | 9 | Size 32/40/48 × Style Primary/Secondary/Danger; Label; nested Spinner |
| Input/Text Field | 57:313 | 4 | State Default/Focus/Error/Disabled; Label; Value/placeholder; Helper; label/helper/leading/trailing/clear booleans |
| Checkbox | 59:329 | 6 | Unchecked/Hover/Checked/Indeterminate/Disabled Unchecked/Disabled Checked |
| Radio | 60:333 | 5 | Unchecked/Hover/Selected/Disabled Unchecked/Disabled Selected |
| Select/Dropdown | 63:158 | 6 | Default/Hover/Focus/Open/Error/Disabled; label/value/helper/options; label/helper/leading booleans |
| Badge | 66:410 | 10 | Style Filled/Outline × Tone Neutral/Info/Success/Warning/Danger |
| Tag | 66:411 | 10 | Style Filled/Outline × Tone Neutral/Info/Success/Warning/Danger; Label; Show remove |
| Input/Search Field | 142:526 | 8 | Mode Live/Submit × State Default/Focus/Filled/Disabled |
| Input/Textarea | 143:410 | 8 | Height 96/160 × State Default/Focus/Error/Disabled |
| Input/Number | 144:437 | 4 | State Default/Focus/Error/Disabled; Unit and stepper booleans |
| Input/Date & Datetime | 146:488 | 8 | Type Date/Datetime × State Default/Focus/Error/Disabled |
| Input/Password | 147:616 | 8 | Visibility Hidden/Shown × State Default/Focus/Error/Disabled |
| Input/File Upload | 149:906 | 8 | Empty/Drag Over/Selected/Validating/Ready/Complete/Error/Disabled |
| Building Blocks/Combobox Option | 163:720 | 5 | Default/Hover/Keyboard Active/Selected/Disabled; Label |
| Input/Searchable Combobox | 171:563 | 9 | Default/Focus/Open/Querying/Keyboard Active/Selected/Empty/Query Restored/Disabled; Label; Empty message; Show label/helper |
| Building Blocks/Switch Control | 183:563 | 10 | Checked Off/On × State Default/Hover/Pressed/Focus/Disabled |
| Switch/Toggle | 186:583 | 10 | Checked Off/On × State Default/Hover/Pressed/Focus/Disabled; Label; Description; Show label/description |
| Tabs/Close Button | 202:832 | 5 | State Default/Hover/Pressed/Focus/Disabled |
| Tabs/Item | 209:690 | 20 | Size 32/40 × Selected False/True × State Default/Hover/Pressed/Focus/Disabled; Label; Show close |
| Tabs/Bar | 213:940 | 6 | Size 32/40 × Active 1/2/3; three exposed Tabs/Item instances |
| Segmented Control/Item | 214:996 | 20 | Size 32/40 × Selected False/True × State Default/Hover/Pressed/Focus/Disabled; Label; Show leading; exposed Icon Slot |
| Segmented Control | 216:1036 | 6 | Size 32/40 × Selected Detail/Compare/Cards; three exposed Item instances |

## 5. Corrections already made

### Dialog elevation correction

- The initial code-derived shadows were rejected as too large: Light 0/26/70/22%, Dark 0/28/80/42%.
- The existing effect style IDs were retained and updated in place.
- Current source of truth: Light 0/10/24/4.5% with transparent-area shadow disabled; Dark 0/12/28/26%.
- The user's direct Light style adjustment was preserved; documentation samples 106:5 and 106:8 remain bound to the two styles; label overflow 0; visual QA PASS.
- This accepted Figma correction supersedes the old CSS shadow values; implementation must sync from Figma.

### Search Field compact-width correction

- Submit 模式 Value 使用 12px / 16px，保持单行结尾省略并横向 FILL；Live 模式继续使用 14px / 18px。
- 修正节点：`142:431`、`142:456`、`142:481`、`142:506`。
- Figma 不具备 CSS 容器查询，故 `Mode=Submit` 是短宽度排版契约；不把任意拉窄行为伪装成自动字体缩放。
- Search Field 文档视觉复检 PASS。

### Searchable Combobox architecture correction

- `Select/Dropdown` 不足以表达查询、键盘、空结果和草稿恢复；正式组件为独立 `Input/Searchable Combobox`（`171:563`）。
- Parent 只组合 `Building Blocks/Combobox Option` 实例；不复制 Option frame。
- Figma 禁止覆盖实例内部的相对旋转，因此新增 `Icon/Utility/Chevron Up`（`168:208`）并通过 Icon Slot 替换；没有 detach。
- Listbox 使用 Float elevation，不使用 Dialog elevation。
- 结构与整组截图 QA PASS；详见 `docs/FIGMA_SEARCHABLE_COMBOBOX.md`。

### Switch / Toggle architecture correction

- A binary immediate-effect setting is represented by `Switch/Toggle` (`186:583`), not by the existing Checkbox family.
- The parent composes 10 instances of `Building Blocks/Switch Control` (`183:563`); Track and Thumb are never copied into parent variants.
- Five size tokens define the 36 × 20 track, 16 thumb, 2 padding, and 120 minimum parent width.
- The current native-checkbox code usage remains the semantic source; implementation should use `<input type="checkbox" role="switch">` and map checked/pseudo-states to the exact Figma axes.
- Material 3 and Simple Design System library assets were audited but not imported because their public API and token contract do not match this file.
- Focus 视觉纠偏：原先的外层 frame 描边与 Track 之间露出背景，造成白色双线；Off/Focus（`181:575`）和 On/Focus（`182:573`）现均使用 Track OUTSIDE 描边。
- Structure QA and Light/Dark visual QA PASS; documentation text discovered in Dark review was corrected to semantic fills. Details: `docs/FIGMA_SWITCH_TOGGLE.md`.

### Tabs / Segmented Control architecture

- Tabs are closeable object navigation; Segmented Control is a single-select workspace display mode. The families remain separate.
- `Tabs/Bar` (`213:940`) composes only exposed `Tabs/Item` instances; labels use single-line ending ellipsis so long object names never displace the 24px close action.
- `Segmented Control` (`216:1036`) composes only exposed `Segmented Control/Item` instances; inner heights 28 / 34 plus container padding 2 / 3 produce exact 32 / 40 totals.
- The parent Selected values map directly to `detail | compare | cards`; Compare and Cards can be disabled independently when fewer than two workspace objects are open.
- Material 3 and Simple Design System assets were audited but not imported because the compact desktop geometry, close semantics, variant API, and token model do not match FIP.
- Five sets, 57 variants, nine tokens, hardcoded visual paints 0, overflow 0, and Light/Dark visual QA PASS. Details: `docs/FIGMA_TABS_SEGMENTED_CONTROL.md`.

### Badge and Tag semantic correction

- Badge is status-only and never carries a remove action.
- Tag default label is Tag.
- Tag Show remove default is false.
- Removable close icons are enabled only on explicit filter/selection instances.
- Usage examples were separated into Status and Filters groups.
- Blocked is now a Danger Badge, not a removable Tag.
- Premier League is the long-label removable Tag test case.
- All 20 Badge/Tag variants use content-hugging horizontal sizing.
- Validation: status remove icons=0; removable tag icons=2; long-label instance overflow=0; section overflow=0.

### Existing component quality constraints

- Components use Icon Slot for nested icon anatomy.
- Icon paths are not duplicated in controls or page instances.
- Variant matrices remain below the 30-variant split threshold per set.
- Every created/mutated Figma node ID is tracked in the local design-system state ledger and this branch record.

## 6. Figma-to-code handoff contract

When implementation starts:

1. Treat this record and the Figma file as the design source of truth for component structure.
2. Map each component set to a code component; map variant properties by the exact API names above.
3. Reuse the existing repository token names and preserve Light/Dark semantics.
4. Keep icons as slot/instance substitutions; do not paste SVG paths into page markup.
5. Implement states before composing screens: default, hover, pressed, focus, disabled, error, empty, loading, and destructive confirmation where applicable.
6. Build Patterns before Screens. A screen is not a new master component when it is only a composition of existing patterns.

## 7. Known next work

The Figma file is intentionally not considered page-ready yet.

Foundation dependency extension P4.2, the first button group, the extended-field group, Searchable Combobox, Switch / Toggle, and Tabs / Segmented Control are complete and verified. Evidence is split across `docs/FIGMA_FOUNDATION_COMPONENT_BACKLOG.md`, `docs/FIGMA_BUTTON_COMPONENTS.md`, `docs/FIGMA_EXTENDED_FIELDS.md`, `docs/FIGMA_SEARCHABLE_COMBOBOX.md`, `docs/FIGMA_SWITCH_TOGGLE.md`, and `docs/FIGMA_TABS_SEGMENTED_CONTROL.md`.

### Required component families

- Tabs/Segmented Control.
- Data Table, selection/sort states, Pagination.
- Menu/Context Menu/Overflow Menu.
- Dialog, Toast, Inline Alert, Disclosure.
- Progress, Skeleton, Empty State.
- Avatar for team/player entities.

### Required pattern families

- PC application shell: primary rail, secondary navigation, topbar, collapse states.
- Page heading, toolbar, filter bar, and selection command bar.
- Metric cards, action cards, panels, and entity rows.
- Master–detail–inspector workspace.
- Workflow stepper/timeline.
- AI chat workspace.

## 8. Cleanup record

Before this record was created, the branch contained application code, Rust crates, contracts, scripts, tests, generated assets, and legacy documentation. Those files were intentionally removed from this design-system branch in the cleanup commit. The main/application branches are not changed by this cleanup.
