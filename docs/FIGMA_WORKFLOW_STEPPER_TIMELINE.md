# Figma · Workflow Stepper / Timeline

**完成并复核：2026-09-30。**

**视觉依据：Figma 是尺寸、比例、间距、字体和颜色的唯一依据；应用源码只提供功能、状态和数据语义。** 本组没有修改应用源码，也没有修改既有 Controls 主组件。

## 1. 当前检查点

本组新增 **2 个组件集 / 29 个变体 / 2 个生产单组件**。Patterns 当前总计 **13 sets / 104 variants / 11 production singles**；另有既存 Shell QA-only 单组件，不计入生产总数。Controls 保持 **61 sets / 506 variants**。

Foundations 当前为 **193 variables / 13 text styles / 6 effects**。本组只新增两个尺寸变量，没有新增颜色、文字样式或效果样式。

下一组：**AI Chat：历史侧栏、消息、附件、Composer**。完成后再进入产品 Screens。本组的示例和 Workflow Lab 都位于 Patterns 页，不属于产品 Screens。

可机器读取的恢复依据：[本组状态文件](FIGMA_WORKFLOW_STEPPER_TIMELINE_STATE.json)。切换会话时先读取当前计划，再核对状态文件中的 Figma 节点；旧截图和旧阶段统计不能覆盖较新的检查点。

## 2. 节点与范围

- Figma file key：`PN0Whgu6HLWIHx4Mv6aHfu`
- Patterns page：`3:4`
- 文档根：`424:16193`，位置 `(200, 23627)`，尺寸 `1600 × 5968`
- Workflow Step 组件集：`424:16593`
- Timeline Item 组件集：`424:16815`
- Workflow Stepper：`425:16285`
- Timeline：`425:16427`
- Review 根：`425:16511`
- 原型入口：`427:17420`

本地画布检查未发现同名既有组件。对可用 Material 3 / Simple Design System 库的 Workflow Stepper 和 Timeline 搜索没有得到匹配资产；因此复用本文件的 Badge、Icon Slot、Button、Empty State 和 FIP tokens 构建，不引入外部 UI kit。

## 3. Workflow Step 与 Stepper

### 3.1 状态归属

**Progress、Active 与实际执行权限必须分开。**

`Progress` 表示业务阶段状态：Locked / Current / Done / Blocked。`Active` BOOLEAN 表示当前正在查看的步骤，不能因为回看已完成步骤而回退业务进度。`State` 表示行的交互外观：Default / Hover / Pressed / Focus / Disabled。

Locked / Blocked 不等于禁止查看：用户仍可以查看完成条件和阻断原因。只有显式的 `State=Disabled` 禁止该行的导航交互。实际写操作是否允许，由宿主依据后端门禁单独判断；点击 Stepper 不能直接触发写入、确认或结算。

### 3.2 Workflow Step API

Set：`Building Blocks/Workflow Step`（`424:16593`），20 variants。

| 属性 | 类型 | 契约 |
|---|---|---|
| Progress | VARIANT | Locked / Current / Done / Blocked |
| State | VARIANT | Default / Hover / Pressed / Focus / Disabled |
| Active | BOOLEAN | 只控制当前查看指示线 |
| Index | TEXT | 步骤编号；Current / Locked 显示编号 |
| Title | TEXT | 步骤名称，自然换行 |
| Summary | TEXT | 简短说明或阻断信息 |
| Show summary | BOOLEAN | 显示或隐藏说明 |

基础示例为 `320 × 58`；长标题允许增高，不截掉第 3 步的长名称。间距、圆角、边框和 32px 标记复用现有 tokens。Done 使用真实 Check Icon Slot，Blocked 使用真实 Alert Icon Slot。状态文字使用真实 Badge 实例；没有复制图标路径。

| State | Locked | Current | Done | Blocked |
|---|---|---|---|---|
| Default | 424:16208 | 424:16227 | 424:16247 | 424:16265 |
| Hover | 424:16285 | 424:16304 | 424:16324 | 424:16342 |
| Pressed | 424:16362 | 424:16381 | 424:16401 | 424:16419 |
| Focus | 424:16439 | 424:16458 | 424:16478 | 424:16496 |
| Disabled | 424:16516 | 424:16535 | 424:16555 | 424:16573 |

### 3.3 Workflow Stepper API

单组件 `425:16285`：Title / Summary / Show heading，加真正的 `Steps` SLOT（节点 `425:16289`）。默认示例 `320 × 612`，包含九个真实 Workflow Step 实例；步骤数量可以通过 SLOT 内容调整，不为每种步骤数量或 Active 位置生成变体。

九步顺序来自复盘工作流：

1. 选择比赛。
2. 导出赛后复盘资料包。
3. 在外部补充真实比赛事实和球员量化数据。
4. 导入并预检。
5. 人工确认。
6. 写入真实赛后事实。
7. 生成正式复盘。
8. 正式结算。
9. 进入分析与历史。

默认示例：1–3 Done、4 Current 且 Active、5–9 Locked。窄宽示例：2 Done 且 Active，4 仍为 Blocked；这验证回看与业务进度独立。

## 4. Timeline Item 与 Timeline

### 4.1 事件状态

`Verification`：Verified / Unverified / Disputed。

`Revision`：Original / Corrected / Cancelled。

两轴各自保留，形成 9 个组合；“已修订”不自动变成“已核验”，“已取消”不删除原始记录。Cancelled 标题有删除线，并保留状态文字和正文，不把整条事件做成低透明度的禁用控件。

时间支持 `90+3′`。Score 是可选的事件比分快照；缺失比分不能显示为 0–0。宿主负责有效事件统计与最终比分，不能把取消事件计入有效比分。示例中的取消事件隐藏比分。

### 4.2 Timeline Item API

Set：`Building Blocks/Timeline Item`（`424:16815`），9 variants。

属性：Verification、Revision、Time、Title、Score、Show score、Description、Metadata、Show metadata、Show connector。

基础示例 `640 × 100`；时间列 64，主体填满余宽并允许换行。Verification / Revision 保持真实 Badge，事件标记保持真实 Icon Slot。宿主为末项设置 `Show connector=false`，单项或末项不延伸连接线。

| Revision | Verified | Unverified | Disputed |
|---|---|---|---|
| Original | 424:16594 | 424:16618 | 424:16643 |
| Corrected | 424:16669 | 424:16693 | 424:16718 |
| Cancelled | 424:16744 | 424:16768 | 424:16791 |

### 4.3 Timeline API

单组件 `425:16427`：Title / Summary / Show heading，加真正的 `Events` SLOT（节点 `425:16431`）。默认示例 `640 × 450`，包含四条事件：已核验进球、待核验换人、有争议的修订记录、已取消的记录。

无事件时在 Events SLOT 放入既有 Empty State（`343:6521`），不 detach Timeline，不另做一个空状态父组件变体。加载失败、真实事件排序、时间格式化和数据恢复由应用负责，不把预置示例冒充运行逻辑。

## 5. Tokens 与局部对比度修正

本组新增 FIP Size 变量：

| 名称 | ID | 值 | Scope | WEB 映射 |
|---|---|---:|---|---|
| layout/workflow/rail-width | VariableID:424:16191 | 320 | WIDTH_HEIGHT | var(--ui-workflow-rail-width) |
| layout/timeline/time-width | VariableID:424:16192 | 64 | WIDTH_HEIGHT | var(--ui-timeline-time-width) |

这两个值来自本轮 Figma 设计，不是从旧 CSS 推导。WEB 名称是后续实现应对接的新 token 契约。

对比度复核发现两类问题：既有 Success / Warning Badge 的文字在本组浅色背景上最低约 3.40:1；部分 secondary 文字在 canvas / hover surface 上约 4.43–4.47:1。

修正仅发生在本组：35 个嵌套 Badge 文字覆盖，以及 34 个新建文字节点，改为绑定已有 `color/text/primary`。状态边框、语义图标、状态文字、字号、行高均保留；没有改动全局 Badge / Controls，也没有为此增加颜色变量。

复核范围内排除显式 Disabled 控件的普通文字，共检查 691 个可见文字层，最低对比度为：

- Light：**4.5326557047:1**。
- Dark：**6.8625187908:1**。

本组自有文字层均关联现有 FIP Text Styles。嵌套 Badge 沿用既有 Controls 的排版设置；不宣称已修复整个历史组件库的文字样式关联。

## 6. Review 与交互验证

| Review | 节点 |
|---|---|
| Light / 1120 | 425:16512 |
| Dark / 1120 | 425:16706 |
| Narrow / 760 | 425:16712 |
| Empty timeline | 425:16930 |
| Workflow 20-state matrix | 425:17064 |
| Verification × Revision 9-state matrix | 425:17357 |

Review 中展示的是组件实例，主组件和构建资产位于其他分区。Light / Dark、长标题、760px 真实重排、隐藏比分、末项连接线和 Empty State 内容替换均检查。窄宽示例内部为 320px Stepper + 20px 间距 + 380px Timeline，没有缩小整体截图来代替响应布局。

### 四个预置原型场景

| 场景 | 节点 | 关键状态 |
|---|---|---|
| Current step 04 | 427:17420 | 4 Current + Active；5–9 Locked |
| Blocked step 04 | 427:17614 | 4 Blocked + Active；后续保持锁定 |
| Revisit completed step 02 | 427:17806 | 2 Done + Active；4 仍 Blocked |
| Preview passed / confirm pending | 427:17980 | 4 Done；5 Current + Active；6–9 Locked |

已接线并读回核对 **8 条原生连接**：模拟预检失败、模拟通过、回看第 2 步、返回第 4 步以及重置。四个未解锁的写入按钮使用真实 Disabled Primary Button，**反应连接均为 0**。

原型是明确标记的预置状态演示。只有说明中的 02 / 04 回看路径与模拟按钮接线；不宣称九步业务都已接通，不执行后端读写，不替代实际键盘焦点、权限、网络竞态与应用端到端测试。

## 7. 最终验收与恢复

深层审计包含 INSTANCE 和 SLOT 内部可见子节点，不仅检查外围 frame。检查结果：

- 新组件命名唯一，重复数量 0。
- 469 个可见嵌套实例引用可解析，失效引用 0。
- 可见内容越界 0；本组文档根和原型与现有顶层内容重叠 0。
- 检查范围内未绑定的可见 solid fill / stroke 0。
- 本组自有文字缺失 Text Style 0。
- 原型连接 8，失效目标 0；禁止写入按钮 4，连接 0。
- 20 个 Workflow Step 与 9 个 Timeline Item 变体完整。
- 532 个实际存在的本组主节点 ID 已以无损区间格式记录在状态文件中；实例内覆盖与原型连接单独记录。

恢复步骤：读取当前计划 → 本组状态文件 → 核对 Figma 根和组件 ID → 从 AI Chat 继续。不要以早期历史统计或截图回退进度。独立记录、状态文件和主索引应共同保存；Create State handoff 是附加入口，不替代这些明确内容。

## 8. 语义来源与实现后续

语义来源：`main/src/pages/review.ts`，blob SHA `4a18002bdd6937cec1d1d076f521dcee8e2f4b5e`。参考 `stepState`、`workflowStepButton`、九步 `reviewPackageWorkspace` 与 `matchEventTimeline`，没有使用 CSS 尺寸。

实现阶段须保持：后端动作门禁独立于查看步骤；复检和确认不跳过；Cancelled 不计入有效事件；null 比分不补零；Events 顺序与来源一致；仅末项隐藏连接线；需要真实键盘、焦点、屏幕阅读和异步恢复测试。

**下一组：AI Chat。**
