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
| 02 · Controls | 3:3 | planned foundation groups complete; 61 sets / 506 variants |
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
- Section · Switch & Toggle: 179:563
- Section · Tabs & Segmented Control: 199:595
- Section · Data Table & Pagination: 228:1090
- Section · Dropdown, Context & Overflow Menu: 249:1705
- Section · Dialog: 276:2210
- Dialog prototype launcher: 286:3364 (13 component test frames on Controls; not product Screens)
- Section · Toast / Inline Alert / Blocking Message: 298:3991
- Feedback prototype launcher: 309:4724 (18 component test frames on Controls; not product Screens)
- Section · Accordion & Disclosure: 320:5159
- Accordion / Disclosure prototype launcher: 328:12469 (16 component test frames on Controls; not product Screens)
- Section · Progress, Skeleton & Empty State: 337:6498
- Section · Avatar: 357:6965
- Current Controls root dimensions: 1440 × 34261

## 2. Foundations

### Variable collections

| Collection | ID | Modes | Count |
|---|---|---|---:|
| FIP Primitives | VariableCollectionId:17:2 | Value | 53 |
| FIP Color | VariableCollectionId:17:3 | Light, Dark | 43 |
| FIP Size | VariableCollectionId:17:4 | Value | 76 |
| FIP Icon Scale | VariableCollectionId:35:2 | 24, 20, 16 | 3 |
| FIP Control Scale | VariableCollectionId:46:2 | 32, 40, 48 | 15 |
| FIP Icon Tone | VariableCollectionId:50:206 | Default, Secondary, Active, On Accent, Success, Warning, Danger, Info | 1 |

All 191 variables have explicit scopes and WEB code syntax. Avatar adds four scoped size variables and one semantic color alias; see [Avatar record](FIGMA_AVATAR.md).

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
- FIP/Avatar/Compact — Inter Extra Bold, 11/16
- FIP/Avatar/Default — Inter Extra Bold, 12/16
- FIP/Avatar/Large — Inter Black, 22/28

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
- Preferred swaps: 15; Avatar adds Person and Shield while preserving all previous 13 choices and the default icon.

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
- Icon/Entity/Person — 356:212
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

Current controls state: **61 component sets and 506 variants**. The final foundation group, Avatar, adds one set and six variants for team/player initials and neutral placeholders at 34 / 38 / 54px. All planned foundation groups are complete; Patterns and Screens remain empty. Details: [Avatar record](FIGMA_AVATAR.md). Previous group records remain indexed in `docs/README.md`; their audit totals describe the state at completion of each group.

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
| Building Blocks/Table Header Cell | 230:1170 | 24 | Type Text/Number × Sort None/Ascending/Descending × State Default/Hover/Focus/Disabled; Label |
| Building Blocks/Table Cell | 235:1242 | 20 | Type Text/Metadata/Number/Status/Action × State Default/Hover/Selected/Disabled; Primary; Supporting |
| Data Table/Row | 236:1331 | 4 | State Default/Hover/Selected/Disabled; nested Checkbox and Table Cell instances |
| Data Table | 238:1663 | 3 | State Default/Loading/Empty; nested Header Cell, Row and Spinner |
| Pagination/Page Button | 239:1439 | 6 | Default/Hover/Pressed/Focus/Current/Disabled; Label |
| Pagination/Nav Button | 240:1447 | 10 | Direction Previous/Next × State Default/Hover/Pressed/Focus/Disabled; Label |
| Pagination | 241:1477 | 4 | State First/Middle/Last/Busy; Summary; nested Nav/Page Button and Spinner |
| Building Blocks/Menu Item | 254:1755 | 20 | Tone Neutral/Danger × Selected False/True × State Default/Hover/Pressed/Focus/Disabled; Label; Description; Shortcut; visibility booleans; Leading Icon swap |
| Building Blocks/Menu Separator | 255:1762 | 2 | Inset None/Leading |
| Menu/Surface | 259:1861 | 3 | Content Actions/Selection/Empty; nested Menu Item, Separator and Note |
| Dropdown Menu | 263:1938 | 6 | State Closed/Hover/Pressed/Focus/Open/Disabled; nested Button/Secondary and Menu Surface |
| Context Menu | 265:1924 | 3 | Target Run/Lineup/Match; exact source action and note semantics |
| Overflow Menu | 267:2007 | 6 | State Closed/Hover/Pressed/Focus/Open/Disabled; nested Button/Icon, More and Menu Surface |
| Building Blocks/Dialog Header | 277:2341 | 6 | Tone Neutral/Danger × State Default/Focus/Busy; Title; Subtitle; visibility booleans |
| Building Blocks/Dialog Body | 278:2354 | 4 | Content Message/Facts/Form/Typed; Description; Error; visibility booleans; exposed Facts/Field |
| Building Blocks/Dialog Actions | 278:2630 | 8 | Tone Primary/Danger × State Default/Focus/Disabled/Loading; Show cancel; exposed Cancel/Confirm |
| Dialog/Standard | 279:2795 | 6 | Size Compact/Wide × State Default/Submitting/Error; Body content swap |
| Dialog/Confirmation | 280:2814 | 6 | Tone Neutral/Danger × State Default/Submitting/Error; exposed Header/Body/Actions |
| Dialog/Typed Danger | 280:3600 | 6 | State Empty/Editing/Valid/Mismatch/Submitting/Error; exposed Header/Body/Actions |
| Building Blocks/Feedback Content | 299:4040 | 4 | Tone Info/Success/Warning/Danger; Title; Description; Show title/description/leading; exposed Icon Slot |
| Building Blocks/Feedback Action | 300:4242 | 12 | Style Primary/Secondary × State Default/Hover/Pressed/Focus/Disabled/Loading; exposed Button Label |
| Building Blocks/Feedback Close | 301:4094 | 5 | State Default/Hover/Pressed/Focus/Disabled; nested Button/Icon |
| Toast | 302:4210 | 4 | Tone Info/Success/Warning/Danger; Show close true; Show action false; exposed Content/Action/Close |
| Inline Alert | 303:4259 | 4 | Tone Info/Success/Warning/Danger; Show close false; Show action false; exposed Content/Action/Close |
| Blocking Message | 304:4348 | 5 | State Unavailable/Blocked/Error/Retrying/Fatal; exposed Content and applicable recovery Actions |
| Building Blocks/Disclosure Trigger | 322:5430 | 10 | Expanded False/True × State Default/Hover/Pressed/Focus/Disabled; Label; Description; visibility booleans; exposed Count |
| Disclosure | 323:5557 | 20 | Style Panel/Inline × Expanded False/True × State five states; Content instance swap; exposed Header/Content |
| Accordion | 325:5983 | 12 | Mode Multiple/Single × legal Open combinations; three exposed Disclosure Item instances |
| Progress Bar | 340:6522 | 12 | Size Compact/Default × Value 0/25/50/75/100/Indeterminate; runtime values remain continuous |
| Building Blocks/Skeleton Block | 341:6504 | 6 | Density Compact/Default × Shape Text/Avatar/Rectangle |
| Skeleton | 342:6572 | 6 | Density Compact/Default × Layout List/Card/Table; true Skeleton Block instances; first-load only |
| Empty State | 343:6571 | 2 | Size Compact/Default; Title; Description; Show description/icon/action; exposed Icon Slot and Secondary Button |
| Avatar | 358:6798 | 6 | Size Compact/Default/Large × Content Initials/Placeholder; Initials text; exposed Fallback icon via Icon Slot |

Standalone components: `Building Blocks/Menu Note` (`255:1763`); `Building Blocks/Dialog Facts` (`278:2257`, up to three editable facts); `Dialog/Modal Layer` (`281:2955`, Dialog content swap); `Task Activity` (`305:4208`, editable Label; host-controlled visibility); `Building Blocks/Disclosure Content` (`323:5179`, editable Text). These do not add variants to the totals above.

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

### Data Table / Pagination architecture

- Header Cell（`230:1170`）分离 Type、Sort 与交互 State；默认仅 1px 底线，Focus 才显示完整焦点边框。
- Table Cell（`235:1242`）按 Text / Metadata / Number / Status / Action 拆分；Status 与 Action 分别引用 Badge 和 Button/Secondary master。
- Row（`236:1331`）同步 Default / Hover / Selected / Disabled；Selected 勾选 Checkbox 并显示 2px accent 左边缘。
- Data Table（`238:1663`）覆盖 Default / Loading / Empty；Loading 保留已有行且禁用交互，不闪空。
- Pagination（`241:1477`）与 Data Table 保持 1120px 同宽但独立；First / Last 禁用边界按钮，Busy 禁止重复翻页。
- 新尺寸 token 为 34px header、9px cell padding-x、27px compact page control；继续复用 36px row。
- 代码当前没有排序行为；Figma Sort 是待实现契约，必须同步 `aria-sort`。Code Connect 在可复用代码组件建立后再连接。
- 七个 sets、71 variants；Controls 总计 35 sets、322 variants；Light / Dark 真实球员目录与状态截图 QA PASS。Details: `docs/FIGMA_DATA_TABLE_PAGINATION.md`.

### Dropdown / Context / Overflow Menu architecture

- `Building Blocks/Menu Item`（`254:1755`）把 Tone、Selected 与 State 分离为 20 个变体；Focus 只使用单一 1px focus stroke，Disabled 不叠加整体透明度。
- `Menu/Surface`（`259:1861`）覆盖 Actions / Selection / Empty；Empty 不保留隐藏交互项。
- `Dropdown Menu`（`263:1938`）和 `Overflow Menu`（`267:2007`）仅在 Open 变体组合真实 Surface；closed states 没有不可见菜单。
- `Context Menu`（`265:1924`）直接映射代码中的 Run / Lineup / Match 行为；Run 为 neutral，Lineup / Match 为 danger，首项 Focus 对应代码打开后聚焦首按钮。
- Surface 使用 border + Subtle elevation。旧 CSS `0 18px 50px rgba(0,0,0,.24)` 被视为待同步的旧实现，不作为新设计基准。
- 六个 menu tokens 形成 240 / 8 / 5 / 11 的紧凑桌面几何；Menu Item 内容区宽 224，padding 为 11 × 10。
- 六个 sets、40 variants、1 single component；Controls 总计 41 sets、362 variants；hardcoded paints 0，placeholders 0，Light / Dark visual QA PASS. Details: `docs/FIGMA_DROPDOWN_CONTEXT_OVERFLOW_MENU.md`.

### Dialog architecture and safety contract

- Six sets / 36 variants plus Facts and Modal Layer single components. All eight masters retain true nested instances; 275 descendant instances resolve to their main components (includes inherited nested icons and controls).
- Compact / Wide widths are 480 / 640. Header minimum 46 with 9px vertical padding; Body 11px vertical padding; horizontal padding 12; Actions 8px vertical padding and existing 32px buttons.
- The accepted Light Dialog shadow remains 4.5% alpha; Dark uses the existing 26% style. Same masters are used for both themes.
- Neutral confirmation only hides history. Lineup versions are deleted or archived according to references. Match deletion preserves ordinary model runs/snapshots, but protected P4 lineage blocks permanent deletion.
- Typed Danger compares `trim(input) === expectedName` before enabling the destructive action. Empty, partial and mismatched input remain disabled; submitting blocks close, cancel and repeat submit.
- Long-body overflow is intentionally constrained to a vertically scrollable Body; Header and Actions remain visible.
- Current `ModalController` renders a workspace `role="region"`, and `runPendingAction` closes before awaiting the action. Overlay semantics, focus trap/restore and inline pending/error persistence are implementation work, not existing code behavior. No speculative Code Connect was published.
- Final QA: 47 sets / 398 variants; unexpected overflow 0; overlap 0; hardcoded visual paints 0; broken main-component references 0; five new scoped tokens verified. Light / Dark, source protection and long-content screenshots reviewed.
- Prototype: 13 reachable test frames, 47 verified reaction links, 12 blocked controls. Input/server results use explicit simulation buttons; this is not an application end-to-end test.
- Details and stable IDs: [Dialog record](FIGMA_DIALOG.md).

### Toast / Inline Alert / Blocking Message architecture

- Six sets / 34 variants plus Task Activity. New section `298:3991`; Controls root is 1440 × 26791. Seven scoped size variables and one semantic focus alias were added; no new raw color values.
- Toast is max-width 380; Inline Alert is fluid; Blocking Message is max-width 480. All use 1px borders. Floating Toast and Task Activity use existing Subtle Light/Dark effects; blocking surfaces have no shadow.
- Short Toast content is centered against the 32px close target, while long content keeps Close at the top. Recovery Actions wrap at narrow widths; the 320px long-label blocking example has no overflow.
- Primary Focus uses a single flush outside 1px `color/border/focus-on-accent` alias, not a separated double ring. Task Activity text was changed from secondary to primary after a Light contrast result of 4.43:1; final Light/Dark values are 10.91:1 / 14.78:1.
- Source `#busy` is a nonmodal `task-activity`, not the legacy full-screen `.busy` overlay. Closing a related Toast does not stop the task; the existing global concurrency guard remains.
- Only Info/Success without actions may auto-dismiss after 3200ms. Warning/Danger and actionable notices persist. New notices must invalidate old timers. Inline dismissal never clears validation or readiness blockers.
- Recovery retries only repeat safe reads/checks, never failed writes. Fatal has guidance without an invented restart button. Connection action means focusing the current page's connection form, not submitting missing credentials.
- Final QA: 53 sets / 432 variants, 337 descendant instances in this section resolve to masters, 58 direct nested instances in the new masters. Unexpected overflow, root overlap, copied own vector paths, hardcoded own visual paints and broken aliases are all 0. Light/Dark and stress screenshots reviewed.
- Prototype: 18 reachable test frames, 45 verified reaction links, 7 blocked controls; four inline states retain the same draft. This is explicit simulation, not an application end-to-end test.
- Details, state IDs, source SHAs and implementation checklist: [Feedback record](FIGMA_TOAST_INLINE_ALERT_BLOCKING_MESSAGE.md).

### Accordion / Disclosure architecture

- Three sets / 42 variants plus Content single component. Section `320:5159`; Controls root 1440 × 30703. Three scoped size tokens were added: trigger minimum 34, trigger padding-y 11, group gap 7; FIP Size is now 60.
- Trigger is 38px for a single 16px line plus 11px vertical padding. Count is an existing Badge instance with a local 0px vertical-padding override, preserving 16px height. Long titles and descriptions wrap without displacing Count or Chevron.
- Disclosure composes Trigger and replaceable Content; Panel and Inline share the same semantics. Accordion composes 36 Disclosure instances across 12 legal combinations. Multiple is the current native-details default; Single is a design addition and allows none open.
- Parent Expanded/State and Mode/Open are the state owners. Multiple-to-Single normalizes to the first open item before selecting a variant; do not rely on Figma's nearest-variant fallback for illegal combinations.
- Disabled locks the header toggle only, leaving visible content readable; nested action permissions remain independent. Error content can still be expanded/collapsed without clearing business blockers.
- Source audit risks: distinguish missing snapshot from explicitly all-closed, replace index-based fallback keys, preserve permitted drafts, exclude sensitive controls, and open hidden ancestors before anchor focus. No application changes or Code Connect files were created.
- QA: Light/Dark, 280/300px long content, Input and Inline Alert body swaps, no overflow, default names, placeholders, copied own paths or hardcoded own visual paints. Non-disabled visible text minimum 4.789:1; Focus minimum 4.228:1 against canvas/surface.
- Prototype: 16 reachable component test frames, 70 internal reaction links, header-only toggles, disabled headers without reactions and identical draft values across collapse/reopen. Explicit simulation, not application end-to-end verification.
- Full APIs, stable variant IDs, source SHAs and implementation checklist: [Accordion / Disclosure record](FIGMA_ACCORDION_DISCLOSURE.md).

### Progress / Skeleton / Empty State architecture

- Four sets / 26 variants. Section `337:6498`; Controls root 1440 × 33309. Twelve scoped size variables were added; FIP Size is now 72.
- Progress Bar (`340:6522`) has Size Compact / Default and Value 0 / 25 / 50 / 75 / 100 / Indeterminate. Code remains continuous 0–100; Figma values are snapshots.
- Progress internals use responsive equal segments. At 120px the ratios remain 0.25 / 0.5 / 0.75 / 1; Indeterminate is 0.375, matching the source 38% snapshot.
- Skeleton Block (`341:6504`) provides Text / Avatar / Rectangle at two densities. Skeleton (`342:6572`) composes 48 true block instances across List / Card / Table.
- Skeleton is first-load-only. The retained pagination example references Data Table Loading (`238:1324`) and contains no Skeleton.
- Empty State (`343:6571`) has Compact / Default with Title, Description, Show description, Show icon and Show action. Icon Slot and Secondary Button stay exposed. Source radius 6 is normalized to the existing radius/md token at 7.
- Light/Dark, 120px progress, 126px long-copy Empty State, first-use, no-results, unavailable and retained-data screenshots passed.
- QA: global 60 sets / 500 variants; hardcoded own visual paints 0; own vectors 0; broken masters 0; placeholders 0; default names 0. Light progress contrast 3.780:1; Dark 6.120:1.
- Code follow-up: add prefers-reduced-motion fallback, extract stable Progress/Skeleton/Empty State implementations, and preserve existing error and retained-data boundaries. No speculative Code Connect was published.
- Full API, stable IDs, source SHAs and implementation checklist: [Progress / Skeleton / Empty State record](FIGMA_PROGRESS_SKELETON_EMPTY_STATE.md).

### Avatar architecture

- One set / six variants. Section `357:6965`; Controls root 1440 × 34261. Four size variables, one semantic color alias and three Avatar text styles added; current totals: 191 variables and 13 text styles.
- `Avatar` (`358:6798`) shares geometry for teams and players: 34 / 38 / 54px, radius 8, Initials / Placeholder. Each master exposes one true Icon Slot; no icon paths are copied into Avatar.
- New `Icon/Entity/Person` (`356:212`) uses the official Lucide user-round geometry. Team placeholders reuse Shield (`83:259`). Iconography has 34 independent icon masters; code AppIcon coverage stays 25 / 25.
- Initials stay legible on Light/Dark with a semantic text alias. Large horizontal padding is 2px so 22px Black `WW` fits completely; other sizes retain 4px padding.
- QA: 23 Avatar usage/test instances; 81 descendant instances resolve. No unbound own paints, paint cache mismatch, unintended overflow, sibling/root overlap, broken masters, unnamed nodes or residual placeholders. Text contrast minimum Light 4.867:1 / Dark 5.225:1.
- Content/Size switching preserves initials and the chosen fallback icon. Empty strings do not automatically switch Figma variants; the caller must select Placeholder. Avatar itself is non-interactive.
- Current source renders initials only. Placeholder selection, grapheme-safe normalization and a stable Avatar implementation remain code follow-up; photos and image-loading states are not claimed. No application source or speculative Code Connect was changed.
- Full API, stable IDs, source SHAs and implementation checklist: [Avatar record](FIGMA_AVATAR.md).

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

The planned foundation component groups are complete. Product Patterns and Screens have not been built; the file does not yet contain finished product pages.

Foundation dependency extension P4.2, the first button group, the extended-field group, Searchable Combobox, Switch / Toggle, Tabs / Segmented Control, Data Table / Pagination, Dropdown / Context / Overflow Menu, Dialog, Toast / Inline Alert / Blocking Message (including Task Activity), Accordion / Disclosure, Progress / Skeleton / Empty State, and Avatar are complete and verified. Evidence is split across the separate records linked from `docs/README.md`.

### Foundation completion

- Remaining planned foundation groups: **0**. Current Controls inventory: **61 sets / 506 variants**.
- Next: **PC App Shell** in Patterns, before product Screens. New dependencies must still be recorded and validated if discovered during composition.

### Required pattern families

- PC application shell: primary rail, secondary navigation, topbar, collapse states.
- Page heading, toolbar, filter bar, and selection command bar.
- Metric cards, action cards, panels, and entity rows.
- Master–detail–inspector workspace.
- Workflow stepper/timeline.
- AI chat workspace.

## 8. Cleanup record

Before this record was created, the branch contained application code, Rust crates, contracts, scripts, tests, generated assets, and legacy documentation. Those files were intentionally removed from this design-system branch in the cleanup commit. The main/application branches are not changed by this cleanup.
