# Figma · Master–Detail–Inspector

**完成日期：2026-09-23**

**设计依据：视觉尺寸、比例、间距、字号、颜色与主题行为只以已确认的 Figma 为准。应用源码只用于核对 Master / Detail / Inspector 的职责、选择语义、检查器开合和数据上下文，不作为视觉尺寸参考。**

## 1. 本组范围

Patterns 第四组完成：

- `Entity Row` — Master 目录依赖
- `Master–Detail–Inspector` — 三栏实体工作区

Entity Row 使用真实交互状态，因此建立 1 个 Component Set / 10 variants；Master–Detail–Inspector 使用单组件 + SLOT + BOOLEAN property，不为了 Inspector 开/关制造重复 variants。

本组没有新增 Variable、Text Style 或 Effect Style。

## 2. Figma 节点

- Patterns page：`3:4`
- 本组文档根：`418:15663`
- 文档根尺寸：1600 × 5507
- Entity Row set：`419:15828`
- Master–Detail–Inspector：`420:15715`
- QA 根：`421:15881`
- QA / Light / Inspector Open：`421:15882`
- QA / Light / Inspector Collapsed：`421:16162`
- QA / Dark / Inspector Open：`421:16236`
- QA / Light / Narrow 760 / Collapsed：`421:16279`

新根位于 Card / Panel、Page Bars 和 PC App Shell 文档根下方，最终结构检查 overlap=0。

## 3. Phase 0 gap / reuse decision

本地 Figma 搜索 `Entity Row`、`Master Detail Inspector`、`List Item`、`Sidebar Item` 均无现成 FIP 组件。

Material 3 / Simple Design System 可以检索到 List item、List、Sidebar、Navigation Button 等，但不导入，原因：

- token 模型与 FIP 不一致；
- Material list density / mobile navigation 语义不等于 FIP 的实体 Master 目录；
- Simple Sidebar 面向通用导航，不包含 Selected + entity metadata + optional Badge 的 FIP 数据契约；
- 没有与当前 `layout/directory/width`、`layout/inspector/entity-width` 和 SLOT 工作区契约相匹配的 Master–Detail–Inspector。

因此本组使用现有 FIP Avatar / Badge / Panel / Metric Card / layout tokens 构建。

应用源码只用于语义确认：

- `src/pages/players.ts`：master filter → detail list/table → optional inspector；Inspector 有 open/collapsed 状态。
- `src/pages/teams.ts`：Master 目录或筛选区、Detail 主内容、Inspector 速览保持同一页面上下文。
- `src/components/workspace.ts`：Inspector 是可折叠 pane；业务选择不会重建一级/二级 Shell。

旧 CSS 几何不作为视觉尺寸依据。

## 4. Entity Row

### Component Set

- ID：`419:15828`
- 宽度绑定：`layout/directory/width` = 248
- 高度：54，由 Default Avatar 38 + 8px 上下 padding 形成
- 10 variants：
  - Selected=False × Default / Hover / Pressed / Focus / Disabled
  - Selected=True × Default / Hover / Pressed / Focus / Disabled

### API

- `Title` — TEXT
- `Supporting` — TEXT
- `Metadata` — TEXT
- `Show supporting` — BOOLEAN
- `Show metadata` — BOOLEAN
- `Show status` — BOOLEAN
- `Selected` — VARIANT
- `State` — VARIANT

真实嵌套依赖：

- Avatar → existing `Avatar` instance，暴露给父组件；
- Status → existing `Badge` instance，暴露给父组件。

Selected 使用：

- background → `color/action/accent-soft`
- 2px selection indicator → `color/action/accent`

Unselected 状态使用：

- Default → `color/bg/surface`
- Hover → `color/bg/raised`
- Pressed → `color/bg/subtle`
- Focus → `color/bg/surface` + 2px `color/border/focus`
- Disabled → `color/control/disabled-surface`

Selected 与 State 明确分离，不把 Selected 当成一个 Hover/Focus 替代状态。

### Acceptance correction

第一次状态审计发现构建脚本中 Selected 使用了字符串 truthiness，导致 `"False"` 也被当作 true，Unselected variants 错误绑定了 accent-soft，selection indicator 也错误显示。

验收前已修复并重新读回：

- Selected=False 5 variants indicator opacity = 0
- Selected=True 5 variants indicator opacity = 1
- Unselected fill bindings恢复为 surface / raised / subtle / disabled
- Selected fill bindings保持 accent-soft
- Unselected Pressed stroke 恢复 border/strong
- Selected Pressed stroke 使用 accent-pressed
- Focus 两组均使用 2px focus border

最终 state map 再审计通过。

## 5. Master–Detail–Inspector

### 组件

- ID：`420:15715`
- Master：248
- Detail：fill
- Inspector：300
- gutter：`spacing/sm` = 8
- specimen：1120 × 600

### API

- `Master` — SLOT
- `Detail` — SLOT
- `Inspector` — SLOT
- `Show inspector` — BOOLEAN

尺寸绑定：

- Master → `layout/directory/width`
- Inspector → `layout/inspector/entity-width`
- Detail → auto-layout FILL

本组针对 players / teams 这类实体工作区，因此 Inspector 使用 entity width 300。已有 `layout/inspector/workspace-width` 360 保留给后续非实体工作区；这里不做一个无实际语义的宽度 variant。

Inspector collapsed 直接通过 `Show inspector=false` 隐藏 Inspector SLOT，Detail 自动扩展。Collapse 是布局可见性，不是单独 Component variant。

## 6. Default composition

Master 默认内容使用 Entity Row：

- Selected：首尔 FC
- Default：全北现代
- Default：蔚山 HD

Detail 默认内容组合既有 Panel；Inspector 默认内容展示只读 facts。

这些只是 Pattern specimen，不是产品 Screens。

## 7. QA / SLOT replacement

QA root：`421:15881`

### Light / Inspector Open — 421:15882

三个 SLOT 均实际替换，无 detach：

- Master → custom directory frame + 4 个真实 Entity Row instances
- Detail → custom detail frame + 2 个 Metric Cards + Panel
- Inspector → custom inspector + Avatar Large + Badge + facts

该 fixture 的实例宽度为 1078（1120 frame 内左右 20 padding），布局读回：

- Master = 248
- Detail = 514
- Inspector = 300
- 两个 gutter = 8 + 8

### Light / Inspector Collapsed — 421:16162

`Show inspector=false`，Inspector 不占布局宽度，Detail 自动扩展；没有复制另一套 MDI master。

### Dark / Inspector Open — 421:16236

与 Light Open 使用同一结构，只切换 FIP Color mode。

### Narrow 760 / Collapsed — 421:16279

fixture 内可用宽度 720：

- Master = 248
- gutter = 8
- Detail = 464
- Inspector hidden

Panel 内长 Body copy 在 412px content width 下自然换行，不缩小字体。

## 8. Final QA

最终验收：

- Entity Row duplicate set：0
- Master–Detail–Inspector duplicate component：0
- Entity Row variants：10 / 10
- Entity Row unresolved nested instances：0
- MDI unresolved nested instances：0
- MDI top-level SLOT definitions：3 / 3
- Master width binding：`layout/directory/width`
- Inspector width binding：`layout/inspector/entity-width`
- production own hardcoded visible fills/strokes：0
- QA visible overflow：0
- 与前三组 Pattern roots overlap：0

共享 totals 保持：

- Variables：191
- Text styles：13
- Effects：6
- Controls：61 sets / 506 variants

Patterns 当前：

- **11 component sets / 75 variants**
- **9 production standalone components**

Screens 仍为空。

本轮没有修改应用源码，也没有发布 speculative Code Connect。

## 9. 下一组

下一组进入：

**Workflow Stepper / Timeline**

随后为 AI Chat，最后才拼装产品 Screens。
