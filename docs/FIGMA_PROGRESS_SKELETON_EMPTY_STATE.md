# Figma · Progress / Skeleton / Empty State

> 本文件记录进度反馈、首次载入骨架与加载后无内容状态的完整设计合同。应用代码未在本轮修改。

- Repository: `uniquenesssta/123`
- Design record branch: `ui-design-system`
- Application source branch: `main`
- Source commit: `4c007be04f9fab4adf6399f7b216b50a88d650e4`
- Source tree: `60e58ccbc9ad1fd96a986a6604005d386cf270e3`
- Figma file: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- Controls page: `02 · Controls` (`3:3`)
- Section: `Section · Progress, Skeleton & Empty State` (`337:6498`)
- Direct link: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=337-6498

## 1. 范围与源码语义

本组覆盖三类不同阶段，不允许互相替代：

1. Progress：页面导航和后台任务仍在进行；可以有连续数值，也可以无法确定剩余量。
2. Skeleton：首次载入且没有旧内容可保留时，临时表达布局结构。
3. Empty State：加载已经完成，但结果为零、尚未开始或当前能力不可用。

错误继续使用 Inline Alert 或 Blocking Message；短时局部未知等待继续使用 Spinner。`probability-meter` 是数据可视化，不属于加载 Progress。

### 代码来源

| File | SHA | 使用事实 |
|---|---|---|
| `src/app/shell.ts` | `3633a7f42b7b337a25d6ad8dc3a2801307db1f1f` | 页面切换时输出 `page-load-progress`，主内容设置 `aria-busy` / `inert` |
| `src/styles/layout.css` | `f1ef772d053cf6e0b6bf377516b3f0edb2f5c4ad` | 2px 页面进度、38% 活动段、1.1s 位移动画 |
| `src/types.ts` | `e7c6d624eb443a3e727f709a285492ceda2055f4` | `BackgroundJob.progress` 为 number；状态 queued / running / succeeded / failed / cancelled |
| `src/pages/analytics.ts` | `6e87f0672a992f44c24425bd908724cb1e1874a3` | 后台任务按连续 0–100 输出宽度与百分比 |
| `src/main.ts` | `65bb1744a63015c56f8be8258406304535accb45` | 玩家分页将容器设为 Busy 并禁用分页按钮 |
| `src/styles/entityCenter.css` | `3a08b2f2a4d571d5760204a0d16149ed9e6c73c8` | 翻页保留当前行，只更新 footer Busy |
| `src/styles/visualSystem.css` | `56e44e574c4fc10667449c444d82bcf5b5e00072` | Empty State 最终覆盖值为 54 / 72 min-height、9 / 12 padding |

代码中未发现 Skeleton 实现；它是新增的设计系统能力。共审计 64 个非 CSS Empty State 使用点，但内联的“未记录 / 暂无内容”仍保留为局部文字，不强制升级为面板组件。

## 2. Token

本组在 `FIP Size` 新增 12 个变量；FIP Size 从 60 增至 72。颜色、字体、间距和焦点语义继续复用现有系统。

| Token | Variable ID | Value | Scope | WEB syntax |
|---|---|---:|---|---|
| `progress/min-width` | `VariableID:336:6498` | 120 | WIDTH_HEIGHT | `var(--ui-progress-min-width)` |
| `progress/track-height-compact` | `VariableID:336:6499` | 2 | WIDTH_HEIGHT | `var(--ui-progress-track-height-compact)` |
| `progress/track-height-default` | `VariableID:336:6500` | 4 | WIDTH_HEIGHT | `var(--ui-progress-track-height-default)` |
| `skeleton/text-height-compact` | `VariableID:336:6501` | 8 | WIDTH_HEIGHT | `var(--ui-skeleton-text-height-compact)` |
| `skeleton/text-height-default` | `VariableID:336:6502` | 12 | WIDTH_HEIGHT | `var(--ui-skeleton-text-height-default)` |
| `skeleton/block-height-compact` | `VariableID:336:6503` | 48 | WIDTH_HEIGHT | `var(--ui-skeleton-block-height-compact)` |
| `skeleton/block-height-default` | `VariableID:336:6504` | 72 | WIDTH_HEIGHT | `var(--ui-skeleton-block-height-default)` |
| `radius/full` | `VariableID:336:6505` | 999 | CORNER_RADIUS | `var(--radius-full)` |
| `layout/empty-state/min-height-compact` | `VariableID:336:6506` | 54 | WIDTH_HEIGHT | `var(--ui-empty-state-min-height-compact)` |
| `layout/empty-state/min-height-default` | `VariableID:336:6507` | 72 | WIDTH_HEIGHT | `var(--ui-empty-state-min-height-default)` |
| `layout/empty-state/padding-compact` | `VariableID:336:6508` | 9 | GAP | `var(--ui-empty-state-padding-compact)` |
| `layout/empty-state/padding-default` | `VariableID:336:6509` | 12 | GAP | `var(--ui-empty-state-padding-default)` |

12 / 12 变量有明确 Scope 与 WEB code syntax；重复变量 0，broken alias 0。源码 Empty State radius 6px 归一到现有 `radius/md = 7px`，避免制造单用途圆角。

## 3. 组件 API

### Progress Bar (`340:6522`)

- 12 variants。
- `Size=Compact | Default`。
- `Value=0 | 25 | 50 | 75 | 100 | Indeterminate`。
- Compact 为 2px，Default 为 4px；母版展示宽度 240，最小宽度合同为 120。
- 设计变体是检查快照；代码 API 必须继续接受任意连续 0–100 数值，禁止把代码限制为六档枚举。
- 25 / 50 / 75 / 100 使用等分 Auto Layout 段，缩到 120px 仍保持精确比例。
- Indeterminate 以 3 / 8 = 37.5% 静态段表达源码 38% 活动段；运行时继续位移动画。
- Determinate 使用 `role=progressbar`、名称、`aria-valuemin=0`、`aria-valuemax=100`、`aria-valuenow`；Indeterminate 使用 `role=status` 与名称并省略 `aria-valuenow`。

### Building Blocks/Skeleton Block (`341:6504`)

- 6 variants。
- `Density=Compact | Default`。
- `Shape=Text | Avatar | Rectangle`。
- Text 高 8 / 12，Rectangle 高 48 / 72；Avatar 复用 Control Scale 32 / 40。
- Text 使用 `radius/sm`，Avatar 使用 `radius/full`，Rectangle 使用 `radius/md`。
- 组件为中性装饰占位，不承载可见文案或业务状态。

### Skeleton (`342:6572`)

- 6 variants。
- `Layout=List | Card | Table`。
- `Density=Compact | Default`。
- 仅组合 Skeleton Block；共 48 个真实嵌套实例，母版来源唯一为 `341:6504`。
- 只用于首次载入且没有旧内容时；宿主设置 `aria-busy=true`，Skeleton 子树 `aria-hidden=true`。
- 分页、刷新或后台重算能保留旧数据时禁止使用 Skeleton。

### Empty State (`343:6571`)

- 2 variants：`Size=Compact | Default`。
- 公共属性：`Title`、`Description`、`Show description`、`Show icon`、`Show action`。
- 默认显示说明，默认隐藏图标与操作；匹配源码中最常见的纯文字空状态。
- Icon Slot 与 Secondary Button 均为暴露实例；图标与动作标签继续通过嵌套组件 API 修改。
- 仅支持 0 或 1 个恢复 / 首次操作；多个阻断操作应升级为 Blocking Message 或页面 Pattern。
- Title 使用 strong，Description 使用 primary，避免低对比度 secondary。

## 4. 状态边界与真实样例

- Light / Dark 对照：`345:6501`；Light `345:6502`，Dark `345:6562`。
- 后台任务 / 页面导航 / 首次载入：`346:6534`。
- 首次使用 / 暂不可用 Empty State：`346:6553`。
- 翻页保留旧数据：`346:6638`，直接引用 `Data Table / State=Loading` (`238:1324`)；Skeleton 数量为 0。
- 120px、Reduced Motion 与长文案压力样例：`347:7175`。

状态合同：

| 情况 | 组件 | 说明 |
|---|---|---|
| 短时局部未知等待 | Spinner | 保持局部尺寸，不铺开布局骨架 |
| 页面导航、后台任务 | Progress Bar | 有数值用 determinate；未知剩余量用 indeterminate |
| 首次载入、无旧内容 | Skeleton | 表达布局结构，不显示真实数据 |
| 翻页 / 刷新、旧内容可保留 | Data Table Loading + Progress | 保留内容，不使用 Skeleton |
| 加载完成但没有内容 | Empty State | no data / no results / first use / unavailable |
| 错误或阻断 | Inline Alert / Blocking Message | 不伪装成 Empty State |

## 5. 可访问性与视觉验收

- Progress 120px 实测比例：25%=0.25、50%=0.5、75%=0.75、100%=1、Indeterminate=0.375。
- Empty State 长标题、长说明与单操作在 Compact 下增高到 126px，无裁切。
- Light 对比度：Title 16.385:1，Description 11.201:1，Progress 3.780:1。
- Dark 对比度：Title 18.071:1，Description 16.044:1，Progress 6.120:1。
- 4 个组件集、26 个变体；hardcoded own visual paints 0，own vectors 0，broken masters 0。
- placeholders 0，default names 0，根级 overlap 0，意外 overflow 0。
- Controls 最终 60 个组件集、500 个变体；Controls root 1440 × 33309。

Reduced Motion：设计合同为停止 Indeterminate 位移并保留静态活动段。当前代码未找到专用 `prefers-reduced-motion` 规则，必须在实现阶段补齐。

## 6. Code Connect 决策

本组未发布 Code Connect：

- Progress 分散在 shell、analytics 与 CSS，不是单一代码组件。
- Skeleton 尚无代码实现。
- Empty State 目前由多个页面模板直接输出，没有稳定的一对一组件路径。

因此只保留 12 个变量的 WEB syntax；待代码抽取为稳定组件后再建立 Javascript / Web Components 映射，避免把多个临时模板错误连接到一个 Figma master。

## 7. 后续代码实现清单

1. 抽取接受连续 `value` 的 Progress 组件，并区分 determinate / indeterminate ARIA。
2. 为页面进度补 `prefers-reduced-motion` 静态退化。
3. 新增 Skeleton Block 与 List / Card / Table 组合；确保 skeleton 子树 aria-hidden。
4. 抽取 Empty State，保持 title / description / icon / single action 合同。
5. 分页继续保留旧数据；不要在 `setPlayerPageLoading` 链路切换成 Skeleton。
6. 错误继续路由到 Inline Alert / Blocking Message；不要用 Empty State 吞掉错误。

## 8. 完成状态

- Figma：完成。
- Light / Dark：通过。
- 最小宽度与长文案：通过。
- 真实数据与保留旧数据：通过。
- GitHub 记录：本文件。
- 应用代码：未修改。
- 下一组：Avatar——球队、球员、默认占位。
