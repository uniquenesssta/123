# Figma Button Components

本文件只记录按钮族与其直接依赖；基础 token、图标库和页面计划分别维护在其他文档。

- Figma file: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- Page: `02 · Controls`（`3:3`）
- Button section: `Section · Button`（`48:5`）
- Spinner section: `Section · Spinner`（`126:273`）
- Baseline date: 2026-08-23
- Status: PASS

## Source alignment

Application branch `main` remains the code reference; this design-system branch contains records only.

- `src/styles/app.css` — SHA `6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214`
  - primary/secondary button baseline
  - disabled interaction baseline
  - `.spinner` uses an 800ms linear infinite rotation
  - `button.danger-action` supplies the destructive action precedent
- `src/styles/components.css` — SHA `88f3d0ffae592bfa41f41a85a040667df1094e52`
  - `.icon-button` supplies the compact square action precedent

Figma preserves the code intent but uses the completed semantic tokens and the locked control scale rather than copying scattered CSS values.

## Button/Danger

- Component set: `Button/Danger`（`119:209`）
- Variants: 12
- Matrix: `Size=32/40/48 × State=Default/Hover/Pressed/Disabled`
- Public API:
  - `Label#51:0` — TEXT, default `Delete`
  - `Show leading#51:13` — BOOLEAN, default false
  - `Show trailing#51:26` — BOOLEAN, default false
  - `Size` and `State` — VARIANT
- Nested anatomy: two Icon Slot instances per variant; no direct or duplicated icon path.
- State bindings:
  - Default → `color/action/danger`（`VariableID:97:8`）
  - Hover → `color/action/danger-hover`（`VariableID:97:9`）
  - Pressed → `color/action/danger-pressed`（`VariableID:97:10`）
  - Disabled → `color/action/danger-disabled`（`VariableID:97:11`）
- Usage: delete, reset, revoke, disconnect, or another irreversible/high-consequence command only.
- Direct link: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=119-209

## Button/Icon

- Component set: `Button/Icon`（`123:209`）
- Variants: 12
- Matrix: `Size=32/40/48 × State=Default/Hover/Pressed/Disabled`
- Geometry: square click targets `32 × 32`, `40 × 40`, `48 × 48`.
- Icon mapping: size 32/40 → Icon Slot 16; size 48 → Icon Slot 20.
- Swap API: each variant exposes its nested Icon Slot; instance users receive `Icon#37:0` INSTANCE_SWAP without detaching the button.
- Disabled treatment: Icon Slot opacity 42%; state remains non-interactive in code.
- Accessibility contract: every implementation must supply an accessible name and tooltip.
- Direct link: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=123-209

## Spinner

- Component set: `Spinner`（`126:294`）
- Variants: 6
- Matrix: `Size=16/20/24 × Tone=Default/On Accent`
- Stroke scale: 16 → 1.25; 20 → 1.5; 24 → 1.75.
- Anatomy: token-bound full Track plus a 270° Active arc.
- Default tone:
  - Track → `color/border/default`（`VariableID:19:10`）
  - Active arc → `color/action/accent`（`VariableID:20:10`）
- On Accent tone:
  - Track and Active arc → `color/icon/on-accent`（`VariableID:20:5`）
  - Track opacity 34%
- Motion contract: Figma stores the static key shape; code applies `rotate(360deg)` over 800ms, linear, infinite.
- Direct link: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=126-294

### Spinner visual correction

- The first ellipse-arc attempt produced radial spokes and read like a clock rather than a progress indicator.
- Final anatomy keeps the full Ellipse Track and replaces the Active arc with a reusable open cubic Vector path; no wedge closure and no radial spokes.
- Final Active arc node IDs: `132:292`, `132:293`, `132:294`, `132:295`, `132:296`, `132:297`.
- The Spinner component-set ID and Loading Button dependency IDs did not change.
- Final Spinner and dependent Loading Button screenshots plus structure validation: PASS.

## Button/Loading

- Component set: `Button/Loading`（`128:343`）
- Variants: 9
- Matrix: `Size=32/40/48 × Style=Primary/Secondary/Danger`
- Public API:
  - `Label#128:0` — TEXT, default `Loading…`
  - `Size` and `Style` — VARIANT
- Nested dependency: one non-exposed Spinner instance per variant.
- Spinner mapping:
  - size 32/40 → Spinner 16
  - size 48 → Spinner 20
  - Primary/Danger → Tone On Accent
  - Secondary → Tone Default
- The label remains visible and editable; loading never collapses to an unlabelled spinner.
- Interaction contract:
  - render as disabled
  - set `aria-busy=true`
  - suppress hover/pressed interaction states while busy
  - preserve the pre-loading inline size in code so asynchronous label changes do not shift adjacent controls
- Direct link: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=128-343

## Validation

| Set | Variants | Instance test | Variable/dependency test | Visible overflow | Result |
|---|---:|---|---|---:|---|
| Button/Danger | 12 | PASS | 12/12 danger state bindings | 0 | PASS |
| Button/Icon | 12 | PASS | 12/12 exposed Icon Slot | 0 | PASS |
| Spinner | 6 | PASS | 6/6 tone/size bindings | 0 | PASS |
| Button/Loading | 9 | PASS | 9/9 nested Spinner; exposed nested count 0 | 0 | PASS |

Group total: 4 component sets, 39 variants, 0 visible overlap/overflow findings.

Controls inventory after this group:

- component sets: 13
- variants: 116
- nested Icon Slot instances: 193
- direct icon instances inside controls: 0
- nested Spinner instances: 9

## Code handoff notes

1. Keep the exact Figma property names when generating component APIs.
2. Danger Button must consume the four danger action tokens; do not recompute hover/pressed colors in component CSS.
3. Icon Button uses the shared AppIcon/Icon Slot mapping; never paste raw SVG paths into button markup.
4. Spinner is a dependency, not a copied vector; Loading Button composes it.
5. Loading is a busy state, not a clickable state. Do not ship hover, pressed, or repeated-submit behavior while `aria-busy=true`.
6. Preserve visible labels and accessible names on every button pathway.
