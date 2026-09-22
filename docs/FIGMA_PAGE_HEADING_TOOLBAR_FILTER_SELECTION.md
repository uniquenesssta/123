# Figma · Page Heading / Toolbar / Filter Bar / Selection Command Bar

**完成日期：2026-09-23**

**设计依据：视觉尺寸、比例、间距、字号、颜色与主题行为只以已确认的 Figma 为准。应用源码只用于核对功能、交互和数据语义，不作为视觉尺寸参考。**

## 1. 本组范围

Patterns 第二组完成四个生产 Pattern：

- `Page Heading`
- `Toolbar`
- `Filter Bar`
- `Selection Command Bar`

本组没有制造无业务意义的 variant。四个 Pattern 均保持为单组件，条件区域使用 BOOLEAN / TEXT component properties，具体控件状态继续由已验收的 Controls 子组件负责。这样避免把“显示/隐藏某区域”与按钮、搜索、筛选状态交叉成组合爆炸。

本组没有新增 Variable、Text Style 或 Effect Style；继续复用现有 FIP Foundations 和 Controls。

## 2. Figma 节点

- Patterns page: `3:4`
- 本组文档根：`403:14581`
- 文档根尺寸：1600 × 3845
- Page Heading：`404:14589`
- Toolbar：`405:14634`
- Filter Bar：`406:14802`
- Selection Command Bar：`407:14987`
- QA 根：`408:15035`
- QA / Light / 1120：`408:15036`
- QA / Dark / 1120：`408:15576`
- QA / Light / Narrow 760：`408:15880`

现有 PC App Shell 根 `370:2` 未修改；新根位于其下方且结构检查无重叠。

## 3. Page Heading

### 组件

- ID：`404:14589`
- Master specimen：1120 × 80
- 左侧：Eyebrow / Title / Description
- 右侧：可选 Status + 可选页面级 Actions

### API

- `Eyebrow` — TEXT
- `Title` — TEXT
- `Description` — TEXT
- `Show status` — BOOLEAN
- `Show actions` — BOOLEAN

嵌套实例均保持真实引用并暴露给父组件：

- Status → `Badge`
- Secondary action → `Button/Secondary`
- Primary action → `Button/Primary`

Status 的 Tone / Style / Label 继续由 Badge 自身 API 控制；页面动作的 Label / State / Size 继续由 Button API 控制，不在 Page Heading 复制第二套状态系统。

语义边界与当前 `taskPageHeader` 一致：页面上下文、标题、说明、状态和页面级动作属于 Heading；Toolbar、Filter 与 Selection 不进入 Heading。

## 4. Toolbar

### 组件

- ID：`405:14634`
- Master specimen：1120 × 40
- 左侧：可选 Submit Search
- 右侧：可选结果摘要、View switch、局部动作

### API

- `Summary` — TEXT
- `Show search` — BOOLEAN
- `Show summary` — BOOLEAN
- `Show view` — BOOLEAN
- `Show action` — BOOLEAN

暴露的真实子组件：

- Search → `Input/Search Field`
- View → `Segmented Control`
- Local action → `Button/Secondary`

默认 Search 使用 Submit 模式并隐藏 Field label/helper；这仍然是现有 Search Field 实例，不复制搜索框 anatomy。Toolbar 只负责当前结果区域的搜索、结果级视图和局部动作，不承担 Page Heading、Filter Bar 或 Selection 的职责。

## 5. Filter Bar

### 组件

- ID：`406:14802`
- Master specimen：1120 × 102
- 1 个必需 Filter + 最多 2 个可选 Filter
- Apply / Clear
- 可选 Active filters 行

### API

- `Active summary` — TEXT
- `Show filter 2` — BOOLEAN
- `Show filter 3` — BOOLEAN
- `Show active filters` — BOOLEAN
- `Show active filter 2` — BOOLEAN
- `Show active filter 3` — BOOLEAN

暴露的真实子组件：

- Filter 1 / 2 / 3 → `Select/Dropdown`
- Active filter 1 / 2 / 3 → `Tag`
- Apply filters → `Button/Primary`
- Clear filters → `Button/Secondary`

布局使用现有 `layout/panel/padding`、`radius/md`、`color/bg/surface`、`color/border/default`。Filter Bar 不持有业务 query；宿主页面负责应用、清空和恢复筛选状态。

创建过程中发现 Active filters 行在初始固定高度下会超出 Master；已改回真正的纵向 AUTO sizing。最终 Master 高度 102，结构复检 overflow=0。

## 6. Selection Command Bar

### 组件

- ID：`407:14987`
- Master specimen：1120 × 56
- 左侧：Selection + 可选 Description
- 右侧：最多两项普通批量动作 + 一项危险动作

### API

- `Selection` — TEXT
- `Description` — TEXT
- `Show description` — BOOLEAN
- `Show action 1` — BOOLEAN
- `Show action 2` — BOOLEAN
- `Show danger` — BOOLEAN

暴露的真实子组件：

- Action 1 / Action 2 → `Button/Secondary`
- Danger action → `Button/Danger`

宿主只在 selection count > 0 时显示本组件；“0 项选择”不是另一个视觉 variant。危险操作仍必须遵守后续业务确认和权限约束，Command Bar 本身不绕过 Dialog / destructive-confirmation 契约。

视觉使用现有 `layout/panel/padding`、`radius/md`、`color/bg/subtle`、`color/border/strong`。

## 7. 语义取证边界

本组只从应用源码核对现有业务契约：

- `src/components/taskWorkspace.ts`：确认 Page Heading 具有 eyebrow / title / description / optional status / actions。
- `src/pages/players.ts` 与 `src/pages/teams.ts`：确认目录存在搜索、筛选应用/清除、结果摘要和 selection batch actions。
- 源码 CSS、旧布局宽高、旧间距和旧阴影均未作为本组视觉尺寸依据。

Master 的 1120 specimen 以及 760 窄宽 QA 都是本次 Figma 设计结果，不从旧 CSS 反推。

## 8. QA

完成三组真实实例组合：

- Light / 1120
- Dark / 1120
- Light / Narrow 760

窄宽度实例通过 FILL 重排；Page Heading 的长说明允许换行，不缩字号；Filter Bar 在窄宽时关闭第三筛选；Toolbar 和 Selection Command Bar 保持可读。

最终结构审计：

- 新生产组件：4 / 4
- 重名生产组件：0
- 新 Component Set / Variant：0 / 0
- 嵌套实例 unresolved main component：0
- 新组件 own hardcoded visible fills/strokes：0
- QA visible overflow：0
- 与 PC App Shell 文档根重叠：0
- 新 Variable / Text Style / Effect Style：0 / 0 / 0

共享 totals 保持：

- Controls：61 sets / 506 variants
- Foundations：191 variables / 13 text styles / 6 effects
- Patterns：PC App Shell 仍为 9 sets / 60 variants；生产 standalone 从 2 增至 6
- Screens：仍为空

本组没有修改应用源码，也没有发布 speculative Code Connect。

## 9. 下一组

下一组进入：

**Metric Card / Action Card / Panel**

完成依赖和 QA 后，再进入 Master–Detail–Inspector、Workflow Stepper / Timeline、AI Chat，最后才拼装产品 Screens。
