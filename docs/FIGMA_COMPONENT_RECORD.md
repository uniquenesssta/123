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
| 02 · Controls | 3:3 | complete baseline |
| 03 · Patterns | 3:4 | reserved; currently empty |
| 04 · Screens | 3:5 | reserved; currently empty |

Page documentation roots:

- Foundations · Icon-first: 23:4
- Desktop dependencies: 100:2
- Iconography · Production library: 27:2
- Code icon coverage: 83:206
- Controls: 48:2
- Section · Badge & Tag: 48:10

## 2. Foundations

### Variable collections

| Collection | ID | Modes | Count |
|---|---|---|---:|
| FIP Primitives | VariableCollectionId:17:2 | Value | 53 |
| FIP Color | VariableCollectionId:17:3 | Light, Dark | 41 |
| FIP Size | VariableCollectionId:17:4 | Value | 25 |
| FIP Icon Scale | VariableCollectionId:35:2 | 24, 20, 16 | 3 |
| FIP Control Scale | VariableCollectionId:46:2 | 32, 40, 48 | 10 |
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
- FIP/Elevation/Dialog/Light
- FIP/Elevation/Dialog/Dark

### Desktop dependency extension audit

- Documentation section: `Section · Desktop dependencies` (`100:2`).
- Added variables: 26 total — 10 layout dimensions, 6 overlay variables, and 10 danger-state variables.
- Final desktop dimensions follow the current high-density CSS: topbar 44, primary rail 60, secondary sidebar 154, directory 248, entity inspector 300, workspace inspector 360, table row 36, page padding 14 × 10, panel padding 12.
- Added text styles: 5; added dialog effect styles: 2.
- Source SHAs: `visualSystem.css` `56e44e574c4fc10667449c444d82bcf5b5e00072`; `layout.css` `f1ef772d053cf6e0b6bf377516b3f0edb2f5c4ad`; `app.css` `6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214`.
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

Current baseline: 9 component sets, 77 variants, 157 nested Icon Slot instances, 0 direct icon instances, 0 placeholders, 0 overflow findings.

| Component set | ID | Variants | API |
|---|---:|---:|---|
| Button/Primary | 51:414 | 12 | Size 32/40/48 × State Default/Hover/Pressed/Disabled; Label; Show leading; Show trailing |
| Button/Secondary | 52:2 | 12 | same API |
| Button/Ghost | 52:255 | 12 | same API |
| Input/Text Field | 57:313 | 4 | State Default/Focus/Error/Disabled; Label; Value/placeholder; Helper; label/helper/leading/trailing/clear booleans |
| Checkbox | 59:329 | 6 | Unchecked/Hover/Checked/Indeterminate/Disabled Unchecked/Disabled Checked |
| Radio | 60:333 | 5 | Unchecked/Hover/Selected/Disabled Unchecked/Disabled Selected |
| Select/Dropdown | 63:158 | 6 | Default/Hover/Focus/Open/Error/Disabled; label/value/helper/options; label/helper/leading booleans |
| Badge | 66:410 | 10 | Style Filled/Outline × Tone Neutral/Info/Success/Warning/Danger |
| Tag | 66:411 | 10 | Style Filled/Outline × Tone Neutral/Info/Success/Warning/Danger; Label; Show remove |

## 5. Corrections already made

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

Foundation dependency extension P4.2 is complete and verified. Exact tokens, IDs, source SHAs, and QA evidence are maintained separately in `docs/FIGMA_FOUNDATION_COMPONENT_BACKLOG.md`.

### Required component families

- Danger Button, Icon Button, and Loading state.
- Extended fields: Search, Textarea, Number, Date/Datetime, Password, File Upload.
- Searchable Combobox with open, keyboard, empty, and query-restoration states.
- Switch/Toggle.
- Tabs/Segmented Control.
- Data Table, selection/sort states, Pagination.
- Menu/Context Menu/Overflow Menu.
- Dialog, Toast, Inline Alert, Disclosure.
- Spinner, Progress, Skeleton, Empty State.
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
