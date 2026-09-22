# Figma · Metric Card / Action Card / Panel

**完成日期：2026-09-23**

**设计依据：视觉尺寸、比例、间距、字号、颜色与主题行为只以已确认的 Figma 为准。应用源码只用于核对功能、交互和数据语义，不作为视觉尺寸参考。**

## 1. 本组范围

Patterns 第三组完成：

- `Metric Card`
- `Action Card`
- `Panel`

其中 Action Card 具有真实交互状态，因此建立 1 个 Component Set / 5 个 variants；Metric Card 与 Panel 都是生产单组件，不为业务内容差异制造视觉 variant。

本组没有新增 Variable、Text Style 或 Effect Style；继续复用现有 FIP Foundations、Icon Slot 与 Button Controls。

## 2. Figma 节点

- Patterns page：`3:4`
- 本组文档根：`410:15350`
- 文档根尺寸：1600 × 3967
- Metric Card：`410:15370`
- Action Card set：`411:15483`
  - Default：`411:15368`
  - Hover：`411:15391`
  - Pressed：`411:15414`
  - Focus：`411:15437`
  - Disabled：`411:15460`
- Panel：`412:15426`
- QA 根：`413:15472`
- QA / Light / 1120：`413:15473`
- QA / Dark / 1120：`414:15545`
- QA / Light / Narrow 760：`414:15637`

新根位于 Page Bars 文档根下方，与 Page Bars 和 PC App Shell 根均无重叠。

## 3. Phase 0 gap / reuse decision

本地文件内搜索 `Metric Card`、`Action Card`、`Panel` 均无现成资产。

Material 3 / Simple Design System 可检索到 Horizontal Card、Stacked Card、Stats Card、Card 和图片型 Panel，但其：

- token 模型不是 FIP；
- 几何与高密度桌面工作区不一致；
- Action Card 的整卡动作语义和本文件状态 API 不一致；
- Panel 以营销/图片内容为主，不匹配 FIP 的 arbitrary workspace content contract。

因此不导入远程组件，使用当前文件已有 Foundations / Controls 构建本地 Pattern。

源码只用于语义确认：

- `src/pages/dashboard.ts`：Metric = label / value / note；Action Card = 整卡导航动作。
- `src/components/taskWorkspace.ts`、players / teams / workbooks 等页面：业务容器普遍是 header + optional actions + arbitrary body。

旧 CSS 中任何 card/panel 尺寸、间距或阴影均不是本组视觉依据。

## 4. Metric Card

### 组件

- ID：`410:15370`
- Master specimen：248 × 88
- Read-only；没有交互 state

### API

- `Label` — TEXT
- `Value` — TEXT
- `Note` — TEXT
- `Show note` — BOOLEAN

Typography：

- Label → `FIP/Label`
- Value → `FIP/Data/Metric`
- Note → `FIP/Caption`

视觉全部绑定已有 token：

- `color/bg/surface`
- `color/border/default`
- `color/text/secondary`
- `color/text/strong`
- `color/text/tertiary`
- `layout/panel/padding`
- `radius/md`
- `spacing/xs`

Metric Card 不建立 success / warning / danger 业务 tone variants。业务含义由 Label / Value / Note 文案和页面上下文表达；若需要状态标识，应组合已有 Badge，而不是污染 Metric API。

## 5. Action Card

### Component Set

- Set：`411:15483`
- 5 variants：
  - Default
  - Hover
  - Pressed
  - Focus
  - Disabled
- 单个 specimen：320 × 84

这些 state 是整卡真实交互状态，不是为了展示而制造的 variants。

### API

- `Title` — TEXT
- `Description` — TEXT
- `Action` — TEXT
- `Show description` — BOOLEAN
- `State` — VARIANT

结构：

- Leading icon：真实 `Icon Slot` instance，默认使用 `Icon/Data/Chart`，并暴露给父组件；
- Arrow：真实 `Icon Slot` instance，使用既有 Arrow Right；
- Title / Description / Action label：FIP typography；
- 整张卡是一个 action target，内部不再嵌套第二个 Button target。

State 视觉继续绑定已有 surface / border / focus / disabled tokens。Disabled 必须在实现时同步取消 activation / keyboard semantics，不只改颜色。

## 6. Panel

### 组件

- ID：`412:15426`
- Master specimen：640 × 180
- 两个真正的 SLOT properties：
  - `Header actions`
  - `Body`

### API

- `Header actions` — SLOT
- `Body` — SLOT
- `Eyebrow` — TEXT
- `Title` — TEXT
- `Description` — TEXT
- `Show header` — BOOLEAN
- `Show description` — BOOLEAN
- `Show header actions` — BOOLEAN
- `Show divider` — BOOLEAN

Panel 只拥有 surface、padding、border、radius 与 header 结构；业务 loading / empty / error / form / table / list 等状态由 Body 中组合的真实组件负责。

QA 中没有 detach Panel。Light fixture 的 Header actions SLOT 被替换为真实 Secondary + Primary Buttons，Body SLOT 被替换为自定义 Frame，结构读回确认 slot replacement 正常。

## 7. QA

QA root：`413:15472`

三个真实组合：

- Light / 1120：`413:15473`
- Dark / 1120：`414:15545`
- Light / Narrow 760：`414:15637`

Narrow 760 的可用内容宽度为 718；最终：

- 两个 Metric Card：353 + 353 + 12 gap = 718
- 两个 Action Card：353 + 353 + 12 gap = 718
- Panel：718

因此窄宽测试不是截图缩放，而是真实组件重排 / resize。

最终结构审计：

- Metric Card duplicate：0
- Action Card set duplicate：0
- Panel duplicate：0
- Action Card variants：5 / 5
- Action Card unresolved nested main components：0
- Panel SLOT definitions：2 / 2
- production own hardcoded visible fills/strokes：0
- QA visible overflow：0
- 与 Page Bars / Shell roots overlap：0
- Variables / Text Styles / Effects：仍为 191 / 13 / 6

Patterns 当前 totals：

- **10 component sets / 65 variants**
- **8 production standalone components**
- Controls 保持 **61 sets / 506 variants**
- Screens 仍为空

本轮没有修改应用源码，也没有发布 speculative Code Connect。

## 8. 下一组

下一组进入：

**Master–Detail–Inspector 工作区**

随后为 Workflow Stepper / Timeline、AI Chat，最后才拼装产品 Screens。
