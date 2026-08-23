# Figma Switch / Toggle

本文件只记录 Switch / Toggle 组件，不混入后续 Tabs、Table 或页面模式。

## 1. 设计目标与来源

- 用途：明确表达会立即生效的二元设置；不是普通 Checkbox，也不用于多选列表。
- 代码来源：`src/pages/apiWorkspace.ts`（SHA `79f4ee4e505ac9da12e6579c1a7729cdeb981bc1`）中的“附加当前只读上下文”。
- 样式来源：`src/styles/app.css`（SHA `6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214`）中的 `.api-context-toggle`。
- 当前代码仍为原生 checkbox。实现阶段应迁移为语义 Switch，同时保留整行点击、标题、说明和 Disabled 能力。
- 参考库审计：Material 3 Switch（key `1f6ff0e7afa1181d166c5dbd3856ba6135b75019`）与 Simple Design System Switch Field（key `7a8dc511d7173ec44c06ef0e8b1a5a6947c4a19c`）。两者 API 与本项目 token/state 契约不一致，因此未直接导入，改为本地重建。

## 2. Figma 位置

- 文件：https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- 页面：`02 · Controls`（`3:3`）
- 文档区：`Section · Switch & Toggle`（`179:563`）
- 直接链接：https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=179-563
- Building Block 文档卡：`Component · Switch Control`（`179:566`）
- Parent 文档卡：`Component · Switch Toggle`（`179:567`）

## 3. 尺寸变量

| Token | Variable ID | Value | 用途 |
|---|---:|---:|---|
| `switch/track-width` | `178:563` | 36 | Track 宽度 |
| `switch/track-height` | `178:564` | 20 | Track 高度 |
| `switch/thumb-size` | `178:565` | 16 | Thumb 尺寸 |
| `switch/track-padding` | `178:566` | 2 | Track 内边距与位移基准 |
| `switch/min-width` | `178:567` | 120 | Parent 最小宽度 |

五个变量均位于 `FIP Size`，绑定尺寸/间距 scope，并提供 WEB code syntax。

## 4. Building Blocks/Switch Control

- Component set：`Building Blocks/Switch Control`（`183:563`）
- 变体：10
- API：`Checked=Off|On` × `State=Default|Hover|Pressed|Focus|Disabled`
- 单变体点击区：40 × 24
- Track：36 × 20；Thumb：16；Padding：2
- 结构固定为 Track + Thumb；不包含标签文本。

| Checked | Default | Hover | Pressed | Focus | Disabled |
|---|---|---|---|---|---|
| Off | `181:566` | `181:569` | `181:572` | `181:575` | `181:578` |
| On | `182:564` | `182:567` | `182:570` | `182:573` | `182:576` |

Focus 不改变开关值；轮廓直接绑定在 Track 的 OUTSIDE 外沿，不再放在 40 × 24 外层 frame 上，从而避免背景从间隙透出形成白色双线。Disabled 使用专用 surface/opacity 语义，不把 Hover/Pressed 伪装成 Disabled。

## 5. Switch/Toggle

- Component set：`Switch/Toggle`（`186:583`）
- 变体：10
- API：`Checked=Off|On` × `State=Default|Hover|Pressed|Focus|Disabled`
- Parent 默认宽度：240；高度：40；最小宽度绑定 `switch/min-width`。
- 每个变体只有两个直接子节点：`Control` 实例（40 × 24）与 `Content` frame（192 × 36）。
- 10 / 10 变体嵌套 `Building Blocks/Switch Control`；复制 Track/Thumb frame 数量为 0。

公开属性：

| Property | Type | Default |
|---|---|---|
| `Label#186:0` | TEXT | Enable setting |
| `Show label#186:1` | BOOLEAN | true |
| `Description#186:2` | TEXT | Changes apply immediately. |
| `Show description#186:3` | BOOLEAN | true |
| `Checked` | VARIANT | Off |
| `State` | VARIANT | Default |

| Checked | Default | Hover | Pressed | Focus | Disabled |
|---|---|---|---|---|---|
| Off | `184:566` | `184:573` | `184:582` | `184:591` | `184:600` |
| On | `185:574` | `185:583` | `185:592` | `185:601` | `185:610` |

## 6. 真实使用样例

- 样例区：`Usage · API Readonly Context`（`188:584`）
- 可用实例：`188:585`
- 不可用实例：`188:592`
- 标题：附加当前只读上下文
- 可用说明：将当前球队、球员或比赛摘要附加到下一条问题。
- Disabled 说明：当前没有可附加的球队、球员或比赛上下文。
- 两个实例均为 520 × 40；文字区 472 宽；overflow 0。

## 7. 代码实现契约

推荐保留原生表单语义，不用只有视觉的 div：

~~~html
<label class="switch-toggle">
  <input type="checkbox" role="switch" aria-describedby="context-help" />
  <span>
    <strong>附加当前只读上下文</strong>
    <small id="context-help">将当前上下文附加到下一条问题。</small>
  </span>
</label>
~~~

- `checked` 映射到 `Checked=Off|On`。
- `:hover`、`:active`、`:focus-visible`、`:disabled` 映射到同名 State。
- 整行 label 保持可点击；键盘 Space 切换；Focus 必须可见。
- Disabled 时同时提供视觉状态和原生 `disabled`，不可只降透明度。
- 不把它映射到现有 Checkbox，也不直接复制 Figma 内部 Track/Thumb。
- 当前代码库尚无独立 Switch 组件，因此暂不创建 Code Connect；代码组件落地后再按上述 API 建立映射。

## 8. 验收结果

- Switch Control：10 / 10 变体。
- Switch/Toggle：10 / 10 变体。
- Parent 中 Control 实例：10；Label / Description 属性关联：10 / 10。
- 真实使用实例：2。
- hardcoded visual paints：0。
- Component set overlap：0；component set overflow：0；section overflow：0。
- placeholders：0。
- Light 视觉 QA：PASS；Dark 视觉 QA：PASS。
- Dark 检查中发现的 7 个文档文字节点已改为语义颜色绑定；Controls 根节点最终恢复 Light。
- 2026-08-23 Focus 视觉纠偏：原外层 frame 描边与 Track 之间露出背景，形成错误的白色双线；Off/Focus（`181:575`）与 On/Focus（`182:573`）已改为 Track OUTSIDE 描边，复检 PASS。

## 9. 后续约束

下一组固定为 `Tabs / Segmented Control`。在实现页面拼装前，继续保持 building block → parent component → real usage → structure QA → Light/Dark visual QA → docs 的顺序。
