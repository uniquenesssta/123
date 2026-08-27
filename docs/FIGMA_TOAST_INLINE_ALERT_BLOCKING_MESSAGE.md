# Toast / Inline Alert / Blocking Message

- 状态：Figma 组件、属性、结构、Light / Dark 和组件原型回读已完成。
- 日期：2026-08-27。
- 仓库：uniquenesssta/123；文档分支：`ui-design-system`；代码审计来源：`main`。
- [Figma 组件区](https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=298-3991)
- [组件交互测试入口](https://www.figma.com/proto/PN0Whgu6HLWIHx4Mv6aHfu?node-id=309-4724)
- [组件总记录](FIGMA_COMPONENT_RECORD.md) · [待办](FIGMA_FOUNDATION_COMPONENT_BACKLOG.md)

本组新增 **6 个组件集、34 个变体、1 个 Task Activity 单组件**。Controls 从 47 sets / 398 variants 更新为 **53 sets / 432 variants**。只建设组件库和测试场景，没有实现应用代码或提前拼装产品 Screens。

## 1. 语义边界与代码来源

| 对象 | 当前代码事实 | Figma 采用规则 |
|---|---|---|
| Toast | `toast()` 使用单个 DOM 节点，normal / success / error，3200ms 后隐藏 | normal → Info，success → Success，error → Danger；Warning 为新增分类 |
| Inline Alert | `.alert`、`.package-message`、连接错误与阵容覆盖提示属于局部上下文 | 放在相关输入或内容附近；默认持续显示，关闭不解除校验或门禁 |
| Blocking Message | 数据服务不可用、readiness blockers、数据加载错误、启动失败会限制相关功能 | 区分 Unavailable / Blocked / Error / Retrying / Fatal；不把所有消息做成弹窗 |
| Task Activity | 当前 shell 的 `#busy` 实际 class 是 `task-activity`；`runBusy()` 控制显示与全局并发限制 | 非模态任务活动，不遮住页面；关闭关联 Toast 不会取消任务 |
| 旧 Busy 样式 | 代码仍保留旧 `.busy` overlay 样式与 overlay token | 本组不把未使用的旧全屏样式误建为新的任务活动组件 |

审计时检查了 CSS 的后续覆盖，不直接采用最早声明。当前 Toast max-width 380、padding 8 × 10、radius 5；Task Activity 最小宽 140、padding 10 × 13、gap 9、Spinner 16。Figma 保留这些紧凑尺寸，但文字使用现有 Body 12/18、Label 11/16。Toast 与 Task Activity 使用既有 Subtle elevation，旧 Float / 重阴影不继续作为新基准。

Material 3 与 Simple Design System 已读取并检索；没有直接满足 FIP 现有语义、紧凑尺寸及 API 的可复用资产，因此建立本地组合。所有小图标、按钮与 Spinner 复用本文件 master，不导入重复图标。

## 2. Token

新增 7 个 FIP Size 变量，以及 1 个 FIP Color 的语义 alias；没有新增 collection、原始颜色、文本样式或 effect style。

| Token | Variable ID | 值 / alias | Scope | WEB code syntax |
|---|---|---|---|---|
| `layout/feedback/toast-max-width` | `VariableID:297:3991` | 380 | `WIDTH_HEIGHT` | `var(--ui-toast-max-width)` |
| `layout/feedback/padding-x` | `VariableID:297:3992` | 10 | `GAP` | `var(--ui-feedback-padding-x)` |
| `layout/feedback/blocking-width` | `VariableID:297:3993` | 480 | `WIDTH_HEIGHT` | `var(--ui-blocking-message-width)` |
| `layout/task-activity/min-width` | `VariableID:297:3994` | 140 | `WIDTH_HEIGHT` | `var(--ui-task-activity-min-width)` |
| `layout/task-activity/padding-x` | `VariableID:297:3995` | 13 | `GAP` | `var(--ui-task-activity-padding-x)` |
| `layout/task-activity/padding-y` | `VariableID:297:3996` | 10 | `GAP` | `var(--ui-task-activity-padding-y)` |
| `layout/task-activity/gap` | `VariableID:297:3997` | 9 | `GAP` | `var(--ui-task-activity-gap)` |
| `color/border/focus-on-accent` | `VariableID:301:4042` | Light → 18:17 / Dark → 18:9 | `STROKE_COLOR` | `var(--ui-focus-on-accent)` |

Focus on Accent 的完整 alias ID 是 `VariableID:18:17` / `VariableID:18:9`，分别位于 FIP Color 的 Light `17:1` / Dark `17:2`。其 scope 仅为 STROKE_COLOR。

复用内容：

- 32px Control Scale、16px Icon Slot、8 / 12 / 20 间距、5 / 7 圆角、1px 描边。
- `color/bg/raised`、`color/bg/surface`、四种 feedback surface、text / icon / action 语义颜色。
- `FIP/Elevation/Subtle/Light`：`S:dfc687196b7ffc49fb0a7b42374b860a70568240,`。
- `FIP/Elevation/Subtle/Dark`：`S:6a5a20e1940dd0144f1dbe6a58fbe76e313c4fdb,`。
- 不修改用户确认的 Dialog Light 4.5% 阴影。

最终变量数量：Primitives 53、Color 42、Size 57、Icon Scale 3、Control Scale 15、Icon Tone 1，共 171。新增 8 / 8 具备明确 scope 与 WEB syntax；重复变量和未解析 alias 均为 0。

## 3. 组件 API 与默认值

| 组件 | Master / Set ID | 数量 | API |
|---|---|---:|---|
| Building Blocks/Feedback Content | `299:4040` | 4 | Tone=Info/Success/Warning/Danger；Title、Description、Show title/description/leading；exposed Icon Slot |
| Building Blocks/Feedback Action | `300:4242` | 12 | Style=Primary/Secondary × State=Default/Hover/Pressed/Focus/Disabled/Loading；exposed Button |
| Building Blocks/Feedback Close | `301:4094` | 5 | State=Default/Hover/Pressed/Focus/Disabled |
| Toast | `302:4210` | 4 | Tone=Info/Success/Warning/Danger；Show close=true；Show action=false |
| Inline Alert | `303:4259` | 4 | Tone=Info/Success/Warning/Danger；Show close=false；Show action=false |
| Blocking Message | `304:4348` | 5 | State=Unavailable/Blocked/Error/Retrying/Fatal |
| Task Activity | `305:4208` | 单组件 | Label；显示与隐藏由宿主任务控制 |

### 可编辑内容

Feedback Content 的实际属性键：

- `Title#299:20`，默认“操作提示”。
- `Description#299:21`，默认“请核对当前对象后继续操作。”。
- `Show title#299:22`、`Show description#299:23`、`Show leading#299:24`，默认 true。

Figma set 的 TEXT 默认值会共享；不能假定仅切 Tone 就会自动生成对应业务文案。业务实例须显式填写标题与说明。Toast 默认隐藏标题；本组 Inline 与 Blocking 示例已显式设置文案。

Toast 的显隐键为 `Show close#302:8` / `Show action#302:9`；Inline 为 `Show close#303:8` / `Show action#303:9`。正文通过 exposed Content 编辑，动作文案通过 exposed Action → Button 的 Label 编辑。没有伪造父级 TEXT 到实例子层的属性引用。

Feedback Action 只有 Style / State 两个直接属性；切换到 Loading 后应再次设置内部 Button Label，避免不同按钮母版的默认文字重置。Task Activity 的直接属性为 `Label#305:0`，默认“正在处理”。

### 几何与组成

| 对象 | 默认尺寸 | 布局规则 |
|---|---|---|
| Content | 480 × 38 预览 | 透明水平布局；Icon Slot 16 + 文本列；标题/正文间隔 4 |
| Action | 高 32、宽 Hug | 仅嵌套既有 Button / Loading Button；不复制按钮文本或 Spinner |
| Close | 32 × 32 | 中性 Button/Icon + Close Icon Slot；不沿用带 Danger Hover 的 Tabs Close |
| Toast | 380 × 48 | max-width 380；padding-x 10、padding-y 8、gap 8；宽可缩小、高 Hug |
| Inline Alert | 640 × 54 预览 | fluid width、高 Hug；padding 10 × 8；无固定高度约束 |
| Blocking | max-width 480 | padding 12、gap 12；真实恢复动作；长动作行可 Wrap |
| Task Activity | 140 × 36 | min-width 140、max-width 380；padding 13 × 10、gap 9；可调宽，长 Label 换行 |

Toast 的 Content alignment wrapper 最小高为 32：短文本垂直居中；长文本时 Close 仍在顶部。隐藏 Action 时不保留空行。Blocking Recovery actions 使用 WRAP，行间距绑定 8px。

所有非浮动表面仅使用 1px 细边框；Focus 只显示一条轮廓。Primary Focus 的轮廓贴住 Button 外沿，使用新增 on-accent alias；没有白色缝隙或双焦点圈。

Icon Slot 引用 Info `88:582`、Check `54:270`、Alert Circle `54:290`；Close 使用既有 Close master。Task Activity 使用 Spinner 16 master `126:276`。

## 4. 状态矩阵与稳定变体 ID

| 组件 | 状态 / 变体名 | ID |
|---|---|---|
| Feedback Content | Tone=Info | `299:3991` |
| Feedback Content | Tone=Success | `299:4003` |
| Feedback Content | Tone=Warning | `299:4014` |
| Feedback Content | Tone=Danger | `299:4027` |
| Feedback Action | Style=Primary, State=Default | `300:4004` |
| Feedback Action | Style=Primary, State=Hover | `300:4029` |
| Feedback Action | Style=Primary, State=Pressed | `300:4054` |
| Feedback Action | Style=Primary, State=Focus | `300:4079` |
| Feedback Action | Style=Primary, State=Disabled | `300:4094` |
| Feedback Action | Style=Primary, State=Loading | `300:4119` |
| Feedback Action | Style=Secondary, State=Default | `300:4125` |
| Feedback Action | Style=Secondary, State=Hover | `300:4149` |
| Feedback Action | Style=Secondary, State=Pressed | `300:4173` |
| Feedback Action | Style=Secondary, State=Focus | `300:4197` |
| Feedback Action | Style=Secondary, State=Disabled | `300:4212` |
| Feedback Action | Style=Secondary, State=Loading | `300:4236` |
| Feedback Close | State=Default | `301:4044` |
| Feedback Close | State=Hover | `301:4054` |
| Feedback Close | State=Pressed | `301:4064` |
| Feedback Close | State=Focus | `301:4074` |
| Feedback Close | State=Disabled | `301:4084` |
| Toast | Tone=Info | `302:4062` |
| Toast | Tone=Success | `302:4113` |
| Toast | Tone=Warning | `302:4144` |
| Toast | Tone=Danger | `302:4177` |
| Inline Alert | Tone=Info | `303:4111` |
| Inline Alert | Tone=Success | `303:4162` |
| Inline Alert | Tone=Warning | `303:4193` |
| Inline Alert | Tone=Danger | `303:4226` |
| Blocking Message | State=Unavailable | `304:4140` |
| Blocking Message | State=Blocked | `304:4186` |
| Blocking Message | State=Error | `304:4246` |
| Blocking Message | State=Retrying | `304:4287` |
| Blocking Message | State=Fatal | `304:4337` |

Standalone Task Activity：`305:4208`，Spinner instance `305:4209`，Label `305:4212`。

### Toast 生命周期合同

- Info / Success 且无 Action：3200ms 后可自动消失；鼠标悬停或键盘焦点在通知内时暂停，离开后按剩余时间继续。
- Warning / Danger，或任何带 Action 的通知：持续显示，直到用户关闭、相关问题解除或显式的新通知替换。
- v1 维持单个可见通知槽，不建 Toast Stack。新通知替换当前通知；不建立无限历史队列。相同事件 key 的重复通知合并，不重复播报。
- 每次替换、关闭或销毁都必须清理旧 timer，并校验消息 identity；旧 timer 不得隐藏后来的消息。
- 关闭只移除通知，不取消请求、不撤销成功操作、不清空输入、不解除 readiness blocker。
- 重要错误仍应在关联 Inline / Blocking / 日志中可找回；不能依赖会被替换的 Toast 作为唯一错误记录。
- 右/下偏移 20px，和 Task Activity 间隔 8px。不能把两者固定在同一个 bottom/right 坐标互相覆盖。
- 执行中的动作进入 Loading 并禁止重复触发；关闭该通知仍不等于取消动作。

当前 `toast()` 没有取消旧 timer；当前 `.toast` 的 `pointer-events:none` 不支持新关闭/操作入口。这些均是待代码修正项，原型未替代码实现修复。

### Inline 与 Blocking 的恢复合同

| 状态 | 可用动作 | 不允许发生 |
|---|---|---|
| Inline Error | 检查连接；可显式配置关闭 | 关闭即通过校验、清空当前草稿、自动重放失败写入 |
| Unavailable | 填写连接信息 | 把空凭据直接提交，或把正式功能伪装成可离线执行 |
| Blocked | 查看缺失项、重新检查 | readiness blockers 未解除即执行正式推演 |
| Error | 重新加载、查看问题日志 | 重试不可幂等写入、把“已发请求”当“已恢复” |
| Retrying | 主动作 Loading、辅助动作 Disabled | 重复检查、重复提交、主动释放门禁 |
| Fatal | 启动失败原因与恢复指引，无假按钮 | 凭空新增代码中不存在的重启 API / 操作 |

“填写连接信息”在真实实现中应聚焦当前页面的连接表单；这里的 connection 场景只是意图说明与显式模拟，不是完整表单 Pattern。“查看问题日志”场景同样只验证动作目的地。

Fatal 样例“现有数据不会被删除”只对应源码中数据库结构版本不一致的提示，不是对所有启动失败的保证；应按实际 `userFacingError` 输出适用说明。错误内容需脱敏，不暴露连接字符串、凭据、API key 或原始堆栈。

Task Activity 不遮挡界面，但现有 `runBusy()` 仍拒绝并发操作；非模态不能解释为允许所有动作同时执行。`runBusy()` 目前只切换 class，而 shell 渲染时才设置 `aria-hidden`，需将任务可见状态与 ARIA 同步。

## 5. 主题、长内容与对比度

| 验收对象 | Figma ID | 结果 |
|---|---|---|
| Light / Dark 同母版对照 | `306:4210`；Light `306:4211`、Dark `306:4560` | 全部 4 Tone、5 Blocking 状态和 Task Activity 已审阅 |
| 280px Toast 长内容 | `307:4636` | 280 × 148；Close 顶部，Action 不溢出 |
| 300px Inline 长内容 | `307:4699` | 300 × 130；正文换行，动作完整 |
| 320px Blocking 长动作 | `307:4742` | 320 × 168；动作行 Wrap，无溢出 |
| Toast + Task Activity 位置样例 | `307:4824` | 20px 右下偏移、8px 间隔，不相互覆盖 |
| 整组最终缩略图 | `298:3991` | 1312 × 3666 布局，顺序与边界通过 |

Dark 通过 FIP Color / Dark mode 和既有 Subtle/Dark effect 应用到同一组 master 实例，没有复制暗色母版。

本次实际修正：

1. Toast 长文案曾使关闭按钮垂直居中；通过 Content alignment wrapper + 顶对齐解决。
2. 320px Blocking 的长按钮曾溢出；改为 Wrap 并绑定 8px 行间距解决。
3. Primary Focus 原本与主按钮蓝色相近；改为贴边 1px on-accent 外轮廓。
4. Task Activity 次级文字在 Light raised 背景仅 4.43:1；改用现有 primary 文字色，主组件和两种主题实例绑定均已回读。

| 取样项 | Light | Dark |
|---|---:|---:|
| Toast / Task Activity primary text 对 raised | 10.91:1 | 14.78:1 |
| Inline body 对四色 surface 的最小值（canvas / surface 底色） | 9.82:1 | 12.44:1 |
| 四色语义图标对对应 surface 的最小值 | 3.09:1 | 5.53:1 |
| Blocking strong title 对 surface | 17.26:1 | 17.69:1 |
| Fatal secondary guidance 对 surface | 4.79:1 | 6.86:1 |
| Primary focus 外轮廓对其所在 raised 背景 | 15.96:1 | 16.65:1 |

以上为解析变量 alias 并进行 alpha 合成后的指定颜色取样，不是完整的应用可访问性认证。Disabled 控件、所有组合背景、缩放与真实键盘导航仍需实现阶段验证。

## 6. 组件交互原型

18 个测试场景位于 Controls 页右侧，为独立顶层 Frame；没有占用 Patterns / Screens 页。请求和输入结果都是显式模拟，不调用真实 API。

| 场景键 | Frame ID |
|---|---|
| `activityOnly` | `309:5016` |
| `blockError` | `311:5015` |
| `blockRecovered` | `311:5216` |
| `blockRetry` | `311:5073` |
| `blocked` | `311:5177` |
| `busyToast` | `309:4964` |
| `connection` | `311:5248` |
| `dismissed` | `309:5063` |
| `inlineDismissed` | `310:5145` |
| `inlineError` | `310:4876` |
| `inlineRecovered` | `310:5235` |
| `inlineRetry` | `310:5033` |
| `launcher` | `309:4724` |
| `logs` | `309:5089` |
| `persistent` | `309:4902` |
| `requirements` | `311:5290` |
| `transient` | `309:4851` |
| `unavailable` | `311:5143` |

实际回读：

- 45 条 reaction 连接；从 launcher 可到达全部 18 个 Frame。
- 只有 transient Frame 使用 AFTER_TIMEOUT = 3200；持续错误、任务活动与阻断场景没有自动消失定时器。
- 新错误通知可替换短暂成功通知；关闭 busyToast 后到达 activityOnly，任务指示仍在。
- inlineError → inlineRetry → 成功/失败；inlineError → inlineDismissed 后保存仍 Disabled。
- 四个 Inline 输入实例值均为“皇家社会 · 主力阵容”，不随提示关闭而变化。
- Unavailable 进入连接表单意图说明；Blocked 进入缺失项说明或重新检查；Retrying 显式模拟成功/失败。
- 7 个禁止操作的入口 reaction 均为 0：busyToast Action、inlineError Save、inlineRetry Action/Save、inlineDismissed Save、blockRetry Primary/Secondary。
- 18 个场景可见节点几何越界为 0。

Timeout 的单位按 [Figma Trigger 官方文档](https://developers.figma.com/docs/plugins/api/Trigger/) 为毫秒。这里验证的是 Figma 场景与连接数据，不是浏览器请求、键盘 Tab、计时器竞争或暂停机制的端到端运行测试。Hover / Pressed / Focus 为可选视觉变体；没有宣称全部鼠标/键盘状态自动转换已实现。

### 代码阶段的可访问性合同

Toast / Task Activity 不应抢焦点或建立 focus trap。当前 Toast 的 `role="status" aria-live="polite"` 可作为基础；成功/普通提示不重复播报。紧急错误的播报级别需按影响范围选择，不能把所有通知一律 assertive。

Close 必须有可理解的名称（如“关闭通知”），键盘可到达，关闭后焦点返回合理入口。非模态通知不注册全局 Esc 去干扰现有 Dialog / Menu。关联 Inline 错误通过说明关联字段；Blocking 的门禁与区域忙碌状态同步，不能只靠颜色表达。

## 7. 结构验收与复用证据

| 项目 | 回读结果 |
|---|---:|
| Controls sets / variants | 53 / 432 |
| 新 sets / variants / single | 6 / 34 / 1 |
| 新 master 的直接嵌套实例 | 58 |
| 本组 Section 的全部后代实例（含 inherited） | 337 |
| 无法解析 main component | 0 |
| 新 master 自有复制 vector 路径（不进入 instance） | 0 |
| 新 master 自有 hardcoded solid paints（不进入 instance） | 0 |
| 新 master 默认命名 / placeholder 名称 | 0 |
| Section 可见节点 overflow | 0 |
| Controls / Section 根级 overlap | 0 |
| 重复变量 / broken alias | 0 / 0 |

Controls 根 `48:2` 最终为 1440 × 26791；Section `298:3991` 为 1312 × 3666，位于 x=64 / y=23045。现有 Dialog 及以前组件未重建。原型场景 x 起点 3620，不与旧 Dialog 测试区交叠。

文档回链目标为本文件；6 个 set 与 Task Activity 共 7 个 master 使用相同 documentation link。各变体 ID 见第 4 节，测试 Frame ID 见第 6 节。

## 8. 代码映射与验收待办

暂不发布 Code Connect：当前应用反馈主要由函数与 HTML 字符串构建，尚无对应的可复用代码组件。不能把 Figma 名称映射到虚构组件文件。

后续实现顺序：

1. 建立 Feedback Content / Action / Close，复用已有 Button、Icon Slot、Spinner。
2. 实现 Toast 单槽生命周期，加入消息 identity、取消旧 timer、悬停/焦点暂停、关闭与动作点击能力。
3. 实现 Inline 与 Blocking 的局部状态；保留草稿与后端门禁，恢复仅重试安全读取/检查。
4. 同步 Task Activity 显示与 ARIA，保留并发限制；统一右下角 placement，避免和 Toast 重叠。
5. 脱敏错误文案；验证启动失败说明与真实原因匹配。
6. 使用现有测试体系覆盖：旧 timer 不隐藏新通知、关闭不取消任务、错误关闭不使 Save 可用、请求中不重复操作、只读恢复不重放写入。
7. 真实环境验证键盘、焦点、live region、窄容器与两种主题，再建立 Code Connect。

## 9. 审计源文件 SHA

以下为本次读取的 blob SHA；未修改应用分支。

| 文件 | SHA |
|---|---|
| `src/app/modal.ts` | `750bed8b30083e27c6ec5d264d51ea18636e7717` |
| `src/app/shell.ts` | `3633a7f42b7b337a25d6ad8dc3a2801307db1f1f` |
| `src/components/databaseSetup.ts` | `4972b0fa669877b63afde483b6a8b119415856a6` |
| `src/components/workspace.ts` | `d6c2ac64efa6ddbf88ca00445bd9d4c369cd7dcc` |
| `src/main.ts` | `65bb1744a63015c56f8be8258406304535accb45` |
| `src/pages/openai.ts` | `1326c3fda076f9f1bca7459c3eac34ac0054f78e` |
| `src/pages/prediction.ts` | `f08d6d0fd9157b9c25297e7c7fa6468d829ad4cc` |
| `src/pages/release.ts` | `e46d9bdf46f4c33523fc2cfbffbba893243f11d1` |
| `src/styles/app.css` | `6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214` |
| `src/styles/components.css` | `88f3d0ffae592bfa41f41a85a040667df1094e52` |
| `src/styles/coreWorkspaces.css` | `36cbf6e5a5296cb1de7258ac12e78e322e4cc614` |
| `src/styles/entityCenter.css` | `3a08b2f2a4d571d5760204a0d16149ed9e6c73c8` |
| `src/styles/layout.css` | `f1ef772d053cf6e0b6bf377516b3f0edb2f5c4ad` |
| `src/styles/moduleWorkspaces.css` | `70b1d2caf0c67f7244cde1c8adb4042d79795377` |
| `src/styles/taskWorkspace.css` | `054eb1d8e2aa96d9ee7eb84e5690d76ee6de0fe0` |
| `src/styles/visualSystem.css` | `56e44e574c4fc10667449c444d82bcf5b5e00072` |
| `src/styles/workspacePanels.css` | `a5427547f5620f54c14460b9cd0cb4bb95dfb1ff` |

## 10. 交接与下一组

本组只提交独立记录、README、组件总记录和 backlog。不要将本文件内容复制回单一总表。

底层组件剩余 3 组：Accordion / Disclosure；Progress / Skeleton / Empty State；Avatar。**下一组固定为 Accordion / Disclosure**。先完成底层组件，再进入 Patterns / Screens。
