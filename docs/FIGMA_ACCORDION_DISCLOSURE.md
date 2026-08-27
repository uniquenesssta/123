# Accordion / Disclosure

状态：Figma 母版、主题、内容替换与组件交互原型已完成并验收。应用代码未修改。

- 验收日期：2026-08-27
- Repository / branch：`uniquenesssta/123` / `ui-design-system`（仅设计记录）
- [Figma 组件区域](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=320-5159)
- [交互原型入口](https://www.figma.com/proto/PN0Whgu6HLWIHx4Mv6aHfu?node-id=328-12469&starting-point-node-id=328%3A12469)
- Controls：`3:3`；根节点：`48:2`，1440 × 30703
- 本组 Section：`320:5159`，1312 × 3872，位于 x=64 / y=26751
- 本组新增：3 个组件集、42 个变体、1 个单组件、3 个尺寸 token
- Controls 总计：56 个组件集、474 个变体；单组件不计入变体总数
- 下一组：Progress / Skeleton / Empty State；底层组件还剩 2 组

## 1. 范围与复用

Disclosure 是独立内容的展开/收起；Accordion 是一组 Disclosure 的展开策略。默认 Multiple 保留当前原生 details 的独立展开语义；Single 是本轮新增的设计能力，不能声称现有应用已经实现。

复用现有 Icon Slot、Chevron Down / Up、Badge、Button、Input/Text Field、Inline Alert、Light/Dark 颜色、字体、圆角及细描边。没有 detach、复制图标路径或另建暗色母版。

已查询当前文件 Libraries，并搜索本地及可用 Material 3 / Simple Design System 的 Accordion 参考；没有导入外部组件。本轮采用本地 FIP 基础与 API，未将搜索结果等同于对外部完整属性 API 的验证。

不在本轮处理：侧栏折叠（Application Shell）、复杂嵌套业务表单（Patterns）、产品 Screens、应用代码或 Code Connect 文件。现有代码尚无稳定的通用 Accordion 导出组件，不创建虚构的组件映射。

## 2. 代码来源与采用规则

| 代码链路 | 现有行为 | Figma / 交接约定 |
|---|---|---|
| `src/pages/players.ts` | 基础资料、名称与别名、位置与角色使用 details；基础资料初始 open；名称/位置显示数量 | 三个真实标题与 3 / 2 数量用于 Accordion 样例；保存、添加等操作仍在正文内独立执行 |
| `src/pages/database.ts` | 连接配置及技术说明包含 details；连接状态影响默认展开，并存在嵌套 | 本轮建立可替换 Content，嵌套业务结构留给 Patterns |
| `src/pages/logs.ts` | “查看技术详情” 展示 technical_message | Inline Disclosure；示例仅用脱敏摘要 |
| `src/pages/lineups.ts` | 历史版本 details 根据当前历史选择展开，带数量 | 标题数量是 Badge 信息，不是独立按钮 |
| `src/pages/rules.ts` | 多组绑定信息独立展开 | Multiple 为默认策略 |
| `src/app/viewState.ts` | `open_details: string[]` 保存展开 key，capture/restore 处理 details | 稳定 key、初始值与显式全部关闭必须区分，见第 8 节 |
| `src/main.ts` | 渲染前 capture、渲染后 restore；jump-workspace-anchor 将直接目标 details 打开后滚动 | 原型入口模拟打开第二项；祖先展开与焦点定位仍待代码补齐 |

### 尺寸与视觉决策

- 最终紧凑样式的 summary 最小高度为 34；`editor-details` summary 纵向 padding 为 11，横向为 12。
- FIP/Label 为 11/16，因此单行标准 Trigger 实际高度为 **38 = 16 + 11 × 2**，34 是下限，不是强制总高。
- `profile-section-stack` 最终 gap 为 7，Panel 圆角为 5；正文 padding 为 12。
- 较早 spacious disclosure/dashboard 规则的 70 / 54 高度不成为本组标准；需要说明的标题通过 Hug 增高。
- 默认边界为 1px。Focus 只有紧贴边界的 1px 轮廓，无 offset 白缝。非浮层组件不使用阴影。
- 不修改全局 Badge master：Trigger 中的 Badge 保留实例，垂直 padding 覆盖为 0 / Hug，高 16；显示数量不会把单行 Trigger 撑高。
- 长标题与说明换行，Count 和 16px Chevron 保持独立空间；没有把长标题强行截断。

## 3. Token

Collection：`FIP Size`（`VariableCollectionId:17:4`），Value mode `17:3`。

| Token | Variable ID | 值 | Scope | WEB code syntax |
|---|---|---:|---|---|
| disclosure/trigger-min-height | VariableID:318:5159 | 34 | WIDTH_HEIGHT | `var(--ui-disclosure-trigger-min-height)` |
| disclosure/trigger-padding-y | VariableID:318:5160 | 11 | GAP | `var(--ui-disclosure-trigger-padding-y)` |
| accordion/item-gap | VariableID:318:5161 | 7 | GAP | `var(--ui-accordion-item-gap)` |

复用 `spacing/md=12`、`spacing/sm=8`、`radius/sm=5`、`control/stroke=1`、`control/disabled-opacity=.42` 及现有语义颜色。没有新增 raw color 或 collection。WEB syntax 是后续代码映射，不代表 CSS 已添加。

最终变量：Primitives 53、Color 42、Size 60、Icon Scale 3、Control Scale 15、Icon Tone 1，共 174；文字样式 10、效果样式 6，均未新增。

## 4. 母版与公共 API

| 组件 | ID | 数量 | API / 职责 |
|---|---|---:|---|
| Building Blocks/Disclosure Trigger | 322:5430 | 10 variants | Expanded=False/True × State=Default/Hover/Pressed/Focus/Disabled |
| Building Blocks/Disclosure Content | 323:5179 | single | Text；默认透明正文，可被替换 |
| Disclosure | 323:5557 | 20 variants | Style=Panel/Inline × Expanded=False/True × State 五态；Content instance swap |
| Accordion | 325:5983 | 12 variants | Mode=Multiple/Single；Open 合法组合；三个暴露的 Disclosure 实例 |

### Trigger

- `Label#322:0`：TEXT，默认“编辑基础资料”。
- `Description#322:1`：TEXT，默认“收起后仍保留当前输入。”
- `Show description#322:2`：BOOLEAN，默认 false。
- `Show count#322:3`：BOOLEAN，默认 false。
- Count 文本通过暴露的 Badge Label 修改；Chevron 使用 Icon Slot 16（`37:38`），折叠 Down（`54:260`）、展开 Up（`168:208`），不旋转/复制路径。
- Default 透明；Hover subtle；Pressed accent-soft；Focus 单条 1px 完整轮廓。展开的非 Focus 状态显示 1px 底线。
- Disabled 使用禁用文字和图标/数量透明度；仅锁定 toggle。不是对所有正文操作的授权判断。

### Content 与 Disclosure

- Content：`Text#323:0`；正文节点 `323:5180`，名字为 Body text，FIP/Body 12/18。
- Disclosure：`Content#323:2` 为 INSTANCE_SWAP，默认 `323:5179`；Header 和 Content 保持暴露。
- Panel 为 surface + 1px border + radius 5，Inline 无外框。Header 横向 FILL、高度 HUG；Body padding 12；内容由自身高度决定。
- Expanded=False 隐藏 Body，Expanded=True 显示 Body。480px 默认样本为收起 38 / 展开 80；不是任意内容的固定高度。
- 父级 Expanded / State 是唯一的展开与交互状态来源，不独立覆盖嵌套 Header 的 Expanded / State。
- 标题的禁用不使已展开正文不可读；正文内的按钮/字段是否禁用，由各操作状态与权限决定。
- 正文出现错误也仍允许展开/收起；折叠错误提示不解除验证或业务门禁。

### Accordion

12 个母版仅组合 **36 个真实 Disclosure Item 实例**，不复制 Header、Body 或图标。三项名字为 Item 1 / Item 2 / Item 3，全部暴露；间距绑定 7。

| Mode | 合法 Open 值 | 默认 |
|---|---|---|
| Multiple | None, 1, 2, 3, 1+2, 1+3, 2+3, All | Multiple / None |
| Single | None, 1, 2, 3 | 允许全部收起 |

Figma Open 属性选择器会显示两种 Mode 的值并集，但只存在上述 **12 个合法组合**。不要依赖 Figma 对非法组合的最近变体匹配：Multiple 切 Single 时，先将 Open 规范化为第一个已展开项；无展开项则保留 None。Single 切 Multiple 时保留当前项。

父级 Mode / Open 控制各 Item 的 Expanded；不另外覆盖某个 Item 的 Expanded 造成双重来源。各 Item 的 Label、Count、Content 及独立 State 可通过嵌套属性修改。

三个 slot 是组件库 v1 示例合同，不是应用最多只能有三个条目。业务数据超过三项时，在 Patterns 使用同一 Disclosure 按稳定数据 key 重复组合；不得为了条数创建全组合变体爆炸。

## 5. 变体稳定 ID

### Trigger

| Variant | ID |
|---|---|
| Expanded=False, State=Default | 321:5159 |
| Expanded=False, State=Hover | 322:5161 |
| Expanded=False, State=Pressed | 322:5169 |
| Expanded=False, State=Focus | 322:5177 |
| Expanded=False, State=Disabled | 322:5185 |
| Expanded=True, State=Default | 322:5193 |
| Expanded=True, State=Hover | 322:5202 |
| Expanded=True, State=Pressed | 322:5211 |
| Expanded=True, State=Focus | 322:5220 |
| Expanded=True, State=Disabled | 322:5229 |

### Disclosure

| Variant | ID |
|---|---|
| Style=Panel, Expanded=False, State=Default | 323:5181 |
| Style=Panel, Expanded=False, State=Hover | 323:5193 |
| Style=Panel, Expanded=False, State=Pressed | 323:5214 |
| Style=Panel, Expanded=False, State=Focus | 323:5234 |
| Style=Panel, Expanded=False, State=Disabled | 323:5254 |
| Style=Panel, Expanded=True, State=Default | 323:5274 |
| Style=Panel, Expanded=True, State=Hover | 323:5294 |
| Style=Panel, Expanded=True, State=Pressed | 323:5314 |
| Style=Panel, Expanded=True, State=Focus | 323:5334 |
| Style=Panel, Expanded=True, State=Disabled | 323:5354 |
| Style=Inline, Expanded=False, State=Default | 323:5374 |
| Style=Inline, Expanded=False, State=Hover | 323:5386 |
| Style=Inline, Expanded=False, State=Pressed | 323:5405 |
| Style=Inline, Expanded=False, State=Focus | 323:5424 |
| Style=Inline, Expanded=False, State=Disabled | 323:5443 |
| Style=Inline, Expanded=True, State=Default | 323:5462 |
| Style=Inline, Expanded=True, State=Hover | 323:5481 |
| Style=Inline, Expanded=True, State=Pressed | 323:5500 |
| Style=Inline, Expanded=True, State=Focus | 323:5519 |
| Style=Inline, Expanded=True, State=Disabled | 323:5538 |

### Accordion

| Variant | ID |
|---|---|
| Mode=Multiple, Open=None | 324:5329 |
| Mode=Multiple, Open=1 | 325:5360 |
| Mode=Multiple, Open=2 | 325:5415 |
| Mode=Multiple, Open=3 | 325:5467 |
| Mode=Multiple, Open=1+2 | 325:5519 |
| Mode=Multiple, Open=1+3 | 325:5583 |
| Mode=Multiple, Open=2+3 | 325:5647 |
| Mode=Multiple, Open=All | 325:5711 |
| Mode=Single, Open=None | 325:5787 |
| Mode=Single, Open=1 | 325:5827 |
| Mode=Single, Open=2 | 325:5879 |
| Mode=Single, Open=3 | 325:5931 |

## 6. 文档区、主题与压力样例

| 对象 | ID | 验收内容 |
|---|---|---|
| Token / Trigger / Content 文档区 | 320:5162 / 320:5163 / 320:5164 | 尺寸样本及底层母版 |
| Disclosure / Accordion 文档区 | 320:5165 / 320:5166 | 分组排列与合法变体 |
| Light/Dark 文档区 | 320:5167 | 同一母版切换 Color mode |
| QA 文档区 | 320:5168 | 窄宽、内容替换与原型入口 |
| Light / Dark 对照 | 326:5712 / 326:5821 | Color modes 17:1 / 17:2 |
| Light Multiple / Single / Inline | 326:5715 / 326:5760 / 326:5806 | 球员资料与技术摘要 |
| Dark Multiple / Single / Inline | 326:5824 / 326:5826 / 326:5828 | 分别使用同一源母版 |
| 长标题 + 说明 + Count 128 | 327:5877 | 280px；标题和正文换行，数量/箭头不溢出 |
| 输入正文替换 | 327:5891 | 300px；Content 指向 Input/Text Field 56:46 |
| Disabled expanded | 327:5941 | 锁定标题，正文仍可阅读 |
| 错误正文替换 | 327:5957 | 300px；Content 指向 Inline Alert/Danger 303:4226 |
| 原型入口按钮 | 330:10902 | 从本组文档区导航到 Launcher 328:12469 |

输入样例中的“奥亚萨瓦尔”是已填值，使用 primary 文字色，不沿用 placeholder 色。Light 文档说明文字的原 secondary 色比值 4.468:1，已经改为现有 primary；Dark 对照说明也使用同一语义绑定，没有改全局颜色变量。

## 7. 交互原型

16 个 Controls 页上的组件测试 frame，不是产品 Screens。共 **70 条内部 ON_CLICK / NAVIGATE 连接**，另有文档入口按钮 1 条连接。没有向母版添加跨测试场景导航；原型反应位于测试实例。

| 场景 | Frame ID |
|---|---|
| Launcher | 328:12469 |
| Multiple / None | 328:10785 |
| Multiple / 1 | 328:11050 |
| Multiple / 2 | 328:11162 |
| Multiple / 3 | 328:11274 |
| Multiple / 1+2 | 328:11386 |
| Multiple / 1+3 | 328:11498 |
| Multiple / 2+3 | 328:11610 |
| Multiple / All | 328:11722 |
| Single / None | 328:11834 |
| Single / 1 | 328:11946 |
| Single / 2 | 328:12058 |
| Single / 3 | 328:12170 |
| Draft open | 328:12282 |
| Draft closed | 328:12336 |
| Disabled（含已收起及已展开） | 328:12425 |

### 已回读的交互规则

- Multiple：点击任一标题只改变该项；支持 8 种展开集合。
- Single：点击另一项会关闭前项；再次点击当前项可全部关闭。
- 切换为 Single：多项展开时保留序号最小的已展开项；切回 Multiple 保持该项。
- 导航入口“跳到名称与别名”进入 Multiple / 2，是打开目标的显式模拟，不声称 Figma 已验证 DOM 滚动或焦点。
- Draft open ↔ closed 的标题连接往返；两个场景均保留相同 Input master 和“奥亚萨瓦尔”示例值，已收起场景 Body 不可见。
- Draft 根节点、Body 和 Field 上没有折叠反应；所有 toggle 仅绑定 Header。
- Disabled 的两个 Header 反应均为 0；可返回入口，不能通过标题展开/收起。
- 所有 16 场景可从入口到达，也可返回入口；无缺失目标。

**验收边界：**以上是 Figma reaction 数据、可见性、母版、值与截图检查。输入值为静态模拟，不接受真实输入，不调用 API，不保存数据；未运行应用浏览器或键盘端到端测试。不能把这份验证描述成应用的真实表单持久化已通过。

## 8. 代码实施与状态恢复风险

以下来自源码审计，尚未通过运行时测试；本轮不修改应用。

1. **初始默认展开与已保存的全部关闭。** 当前默认 `open_details=[]`，restore 对每个 details 赋值 `snapshot.open_details.includes(key)`。推断首次空快照可能覆盖 markup 中的 `open`。实现需区分“无快照”与“用户明确全部收起”，不能把空数组一律当作未初始化。
2. **稳定 key。** 当前 `detailsKey` 顺序为 id → data-workspace-key → data-view-key → `details:index`。排序或插入条目会使 index 回退绑定错误；采用稳定且有模块/对象作用域的数据 key，跨球员/连接配置不得互相继承展开状态。
3. **保存时机。** 当前可见 capture 点包括重渲染前及 beforeunload，随后 restore；没有观察到统一的 details toggle 持久化监听。不要承诺每次折叠立即持久化，或 beforeunload 异步 flush 一定完成。根据状态存储合同补同步时机。
4. **保留正文草稿。** 普通收起仅隐藏，不提交、不删除、不清空。若使用会卸载正文的实现，必须另行保存允许保留的草稿；原生 details 折叠本身不卸载 DOM。折叠后隐藏内容不可继续获得 Tab 焦点。
5. **敏感字段例外。** control capture 仅在 includeControls 为 true 时进行；safeControl 要求 id，排除 disabled、位于 data-workspace-persist=false 祖先内的控件、password/file 以及匹配敏感名的字段。不要把 API key、密码、文件内容等写入展开状态或通用草稿持久化。details 自身的排除选择器是 `details:not([data-workspace-persist="false"])`；它与控件的祖先排除规则不同。
6. **Keyboard / focus。** 优先保留原生 details/summary 的 Enter / Space 语义；若用 button+panel，实现 aria-expanded、aria-controls、唯一 ID 与实际 open 同步。Disabled 不能只变灰：原生 summary 没有 disabled 属性，需要阻止实际激活并表达 aria-disabled；不要误禁用正文的独立操作。程序关闭包含焦点的正文前，将焦点移到仍可见且可用的标题或合理目标。
7. **Anchor jump。** 当前 jump-workspace-anchor 只打开直接目标为 HTMLDetailsElement 的节点，然后滚动。若目标在已收起祖先内，需先展开祖先链，并验证可见性与焦点落点；当前代码没有完整实现该链路。
8. **Single 是新策略。** 现有页面原生 details 没有统一 Single 管理器。本组 Mode / Open 需要以稳定 key 实现；重排后不要按序号恢复业务对象。Figma 的 1/2/3 仅为三槽样例标记。
9. **业务动作隔离。** 保存/新增/删除按钮留在正文内，不冒泡为 toggle。折叠错误或加载结果不改变保存门禁，不自动重试、不越过权限；Busy/Disabled 状态由对应动作负责。
10. **后续测试。** 覆盖首次载入、显式全部关闭、页面往返、对象切换、列表重排、Single/Multiple 规范化、正文焦点关闭、原生键盘、禁用激活、非敏感草稿恢复与敏感字段排除。

## 9. 验收结果

| 检查项 | 结果 |
|---|---|
| 新组件集 / 变体 / 单组件 | 3 / 42 / 1，均符合预期 |
| Controls 总数 | 56 sets / 474 variants |
| Accordion 中直接 Item 实例 | 36，均来源 Disclosure master |
| 本组 Section 后代实例 | 484（含嵌套与隐藏实例），全部解析到真实 master |
| 新变量 | 3/3 明确 scope、值与 WEB syntax；重复 / broken aliases 0 |
| 组件公共属性引用 | 缺失引用 0 |
| 自有图标路径 / 未绑定自有可见 solid paint | 0 / 0；复用实例内部路径不计为复制 |
| 默认命名 / placeholder | 0 / 0 |
| 可见内容 overflow / Controls 根级重叠 / 原型互相重叠 | 0 / 0 / 0 |
| 非禁用可见文字对比度 | 检查 346 处，最低 4.789:1，无低于 4.5:1 的文字 |
| 箭头语义色 / Focus 色对比度 | Light/Dark 对 canvas/surface 最低 4.468:1 / 4.228:1，均高于 3:1 检查目标 |
| 原型场景 / 内部连接 / 缺失或无法返回场景 | 16 / 70 / 0 |
| 视觉 QA | 母版、Light/Dark、280/300px 压力样例、展开/收起/禁用原型、整组缩略图均审阅 |

本检查不等于完整无障碍认证或应用运行时测试。固定的 master 展示宽度与原型画布尺寸是文档尺寸，不是新增的产品尺寸 token。

## 10. 源文件快照

代码读取范围为 main 上 46 个 `src/**/*.ts|css` 文件；以下列出决定本组语义与尺寸的主要文件。SHA 均为 **文件 blob SHA**，不是 commit SHA。

| File | Blob SHA |
|---|---|
| src/app/viewState.ts | 379b0b6d06b71980611fa90e5f289b4f4ac2df07 |
| src/main.ts | 65bb1744a63015c56f8be8258406304535accb45 |
| src/pages/players.ts | b406293fd35163bcfae6266b9d083ea3b144d2e5 |
| src/pages/database.ts | 09f00b517715540df6977beac4ac663a54c8d979 |
| src/pages/logs.ts | cabb4829f6f19101e3118446852d00cb7f4c4613 |
| src/pages/lineups.ts | d35d01fb28ca13b6fb5d07423d11112507ffb243 |
| src/pages/rules.ts | 22b042a920f9841fe759d8d8e834d47b683c8c9b |
| src/styles/app.css | 6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214 |
| src/styles/coreWorkspaces.css | 36cbf6e5a5296cb1de7258ac12e78e322e4cc614 |
| src/styles/visualSystem.css | 56e44e574c4fc10667449c444d82bcf5b5e00072 |

源 main tree SHA：`4c007be04f9fab4adf6399f7b216b50a88d650e4`（tree，不是 commit）。

## 11. 后续顺序

1. Progress / Skeleton / Empty State。
2. Avatar：球队、球员、默认占位。

全部底层组件验收后，才进入 Patterns 与 Screens。
