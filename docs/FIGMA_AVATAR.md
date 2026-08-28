# Figma · Avatar

> 球队、球员与默认占位的底层组件合同。本轮不修改应用源码，不开始 Patterns 或 Screens。

- Repository: `uniquenesssta/123`
- Design record branch: `ui-design-system`
- Application source: `main` at `4c007be04f9fab4adf6399f7b216b50a88d650e4`
- Completed: 2026-08-28
- Figma: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu
- Controls page: `3:3`
- Section · Avatar: `357:6965`
- Direct link: https://www.figma.com/design/PN0Whgu6HLWIHx4Mv6aHfu?node-id=357-6965
- Component set: `Avatar` — `358:6798`

## 1. 范围与源码事实

当前源码只显示姓名缩写，没有球队徽标、球员照片或图片加载字段。球队与球员共用相同头像外形，因此本组不为每种实体复制一套母版。

- Initials：已有能力；使用处理后的姓名缩写。
- Placeholder：新增设计能力；已知球队使用现有 Shield，球员或未知对象使用新增 Person。
- 图片、上传、图片加载／失败状态：当前数据合同不存在，本组没有用假照片或图标冒充已实现图片能力。
- Avatar 是身份展示，不是按钮。Default / Hover / Pressed / Focus / Disabled 属于外层按钮、目录项或表格行，不在 Avatar 重复创建。
- 伤病、可用性、选中、认证、在线状态均不由 Avatar 表达。盾牌图形不代表认证或数据可信度。

### 源码位置

| File | Blob SHA |
|---|---|
| `src/components/footballText.ts` | `edd7d148db7dd53f6952b077188757a92b9697ca` |
| `src/components/icons.ts` | `b0cef778236d596ab94ea0a6ff0083da0481d19f` |
| `src/pages/players.ts` | `b406293fd35163bcfae6266b9d083ea3b144d2e5` |
| `src/pages/teams.ts` | `677e4eb3b51f8efa73a3a5b1c148cafb30d93090` |
| `src/styles/app.css` | `6cb8405159a87c4ad5cf7c02dd4c1d3d7316c214` |
| `src/styles/entityCenter.css` | `3a08b2f2a4d571d5760204a0d16149ed9e6c73c8` |
| `src/styles/visualSystem.css` | `56e44e574c4fc10667449c444d82bcf5b5e00072` |
| `src/types.ts` | `e7c6d624eb443a3e727f709a285492ceda2055f4` |

`footballText.initials()` 当前行为：trim；中文取前两个 Unicode 码点；英文取前两个空白分词的首字母并大写；空值返回 `?`。旧球员档案 hero 另外使用 canonical name 的第一个 UTF-16 code unit，和目录存在不一致。

最终样式需按 CSS 层叠读取：

- 普通实体头像：34px；速览 large / xlarge 在 visualSystem 中合并为 38px；旧 player-profile hero 为 54px。
- `entityCenter.css` 中 8px 圆角使用 `!important`，覆盖后续非 important 的 7px 声明。本组保留实际 8px。
- `app.css` 的旧 54px player avatar 不可套回目录或速览。

## 2. Token 与文字样式

新增 5 个变量。FIP Size 72 → 76，FIP Color 42 → 43；所有集合合计 191 个变量。未新增 collection、primitive 或 effect style。

| Token | ID | Value / Alias | Scope | WEB syntax |
|---|---|---|---|---|
| avatar/size-compact | VariableID:355:6753 | 34 | WIDTH_HEIGHT | `var(--ui-avatar-size-compact)` |
| avatar/size-default | VariableID:355:6754 | 38 | WIDTH_HEIGHT | `var(--ui-avatar-size-default)` |
| avatar/size-large | VariableID:355:6755 | 54 | WIDTH_HEIGHT | `var(--ui-avatar-size-large)` |
| radius/avatar | VariableID:355:6756 | 8 | CORNER_RADIUS | `var(--ui-radius-avatar)` |
| color/text/avatar | VariableID:355:6757 | Light → blue/light-hover; Dark → blue/dark-active | TEXT_FILL | `var(--ui-avatar-text)` |

WEB 名称为本轮设计合同，尚未写入应用 CSS。语义颜色 alias 使用既有 primitive，不复制新色值。

| Text style | ID | Font / Size / Line height |
|---|---|---|
| FIP/Avatar/Compact | S:386d11f4c1c0e5813bada13beaa045ff0ade7243, | Inter Extra Bold · 11 / 16 |
| FIP/Avatar/Default | S:b9ab74a387faa7ce28e554b6cbed537d7cb4c3db, | Inter Extra Bold · 12 / 16 |
| FIP/Avatar/Large | S:4e9838d4825a7a1c835b908453f91fd56bd1aaac, | Inter Black · 22 / 28 |

总 text styles 为 13。已有 Dialog Light 4.5% 阴影未改动；Avatar 没有阴影。

## 3. 组件 API

`Avatar` — `358:6798`，共 6 个变体：

| Size | Content | Variant ID |
|---|---|---|
| Compact | Initials | 357:6973 |
| Default | Initials | 358:6756 |
| Large | Initials | 358:6765 |
| Compact | Placeholder | 358:6774 |
| Default | Placeholder | 358:6780 |
| Large | Placeholder | 358:6789 |

公共属性：

- `Size = Compact | Default | Large`；默认 Compact。
- `Content = Initials | Placeholder`；默认 Initials。
- `Initials#358:0`：TEXT，默认 `FC`。
- `Fallback icon`：暴露的真实 Icon Slot 实例，使用其 `Icon#37:0` INSTANCE_SWAP。
- 球队／球员身份通过业务语义、缩写和图标选择传入，不制造每个图标一个变体。

解剖合同：

| Size | Avatar | Initials 区域宽度 | Padding X / Y | Icon Slot |
|---|---:|---:|---|---:|
| Compact | 34 × 34 | 26 | 4 / 4 | 16 |
| Default | 38 × 38 | 30 | 4 / 4 | 20 |
| Large | 54 × 54 | 50 | 2 / 4 | 24 |

- 缩写底色复用 `color/action/accent-soft`，文字使用 `color/text/avatar`。
- 默认占位复用 `color/bg/subtle` 和 Icon Tone Default。
- 缩写和图标互斥显示；隐藏图标仍保持真实实例及可替换属性。
- 6 个母版内各有 1 个暴露 Icon Slot，没有直接复制自有图标路径。
- 缩写固定字号、单行、省略保护；正常数据必须先生成 1–2 个字符簇的缩写。
- Figma 不会根据空字符串自动选择 Placeholder；运行时由调用方决定 Content。

## 4. 图标依赖

- 新增 `Icon/Entity/Person`：`356:212`。
- 图标文档：`356:206`，位于 Iconography，未增加 Controls 组件集数量。
- 来源：[Lucide user-round](https://github.com/lucide-icons/lucide/blob/main/icons/user-round.svg)；许可见上游仓库 LICENSE。
- 复用球队图标：`Icon/Utility/Shield` — `83:259`。
- Icon Slot：`37:44`；16 / 20 / 24px 分别为 `37:38` / `37:32` / `37:26`。
- Person 只有 Head / Shoulders 两个 vector；均绑定 `icon/color` 和 Icon Scale 的 `icon/stroke`。
- 24 / 20 / 16px 描边为 1.75 / 1.5 / 1.25；Light / Dark 六个样例检查通过。
- Person 和 Shield 已加入 Icon Slot preferred swaps，原 13 项完整保留，现为 15 项。
- Iconography 独立 Icon master 共 34，代码 AppIcon 覆盖仍为 25 / 25；Person 是设计依赖，尚未加入 AppIcon 类型。

## 5. 检查场景与节点

| 场景 | Node ID | 验证事实 |
|---|---|---|
| Light / Dark 总览 | 359:6767 | 同母版覆盖六个变体及两类实体 |
| Light | 359:6768 | 曼城 / EH / 哈兰与三个占位例 |
| Dark | 359:6769 | 颜色和 alpha 随模式解析 |
| 缩写压力卡 | 360:6806 | WW、张三、李、ÉM、7、空名称 |
| Large / 异常输入 | 360:6852 | 54px WW 完整；长串 Initials 单行截断 |
| 248px 目录 | 360:6871 | 34px 头像固定；长名称省略 |
| 36px 表格行 | 360:6882 | 34px 头像在 y=1；不挤大行高 |
| 300px 速览 | 360:6891 | 38px 头像固定；长姓名换行 |

人物／球队名称只作组件内容样例，不代表来自实时数据源。上述容器为 Controls 内的组件测试，不是产品 Patterns 或 Screens。

属性回读测试按顺序执行 Content → Size → Content → Size → Content 五步：`国米` 缩写和 Shield 选择始终保留；图标尺寸随 34 / 54 / 38 头像切换为 16 / 24 / 20；始终仅 Initials 或 Fallback icon 一项可见。临时测试实例已删除。

## 6. 发现与修正

1. 原浅色 accent 在头像底色上对比度约 4.037:1。新增头像文字 alias，复用已有较深蓝色，未修改全局 accent。
2. 首轮变量绑定的 alpha paint 缓存显示为黑色。重新写入由 token 解析的 RGB 和 alpha，再保留 variable binding；最终缓存与变量解析不一致为 0。
3. Large 的 22px Black 字体下 `WW` 自然宽度为 49px，原 46px 内容宽度会省略。Large 横向 padding 从 4 调整到现有 spacing/2xs = 2，内容宽度达到 50；字号和头像外框不变。

## 7. 验收结果

| 项目 | 结果 |
|---|---|
| 新 Avatar 组件集 / 变体 | 1 / 6 |
| 母版直接 Icon Slot 实例 | 6，全部暴露 |
| 使用／压力测试 Avatar 实例 | 23 |
| 本分区后代实例 | 81，母版全部可解析 |
| 自有 vector / 未绑定自有 paint | 0 / 0 |
| paint 缓存与绑定解析不一致 | 0 |
| 意外 overflow / sibling overlap | 0 / 0 |
| 变体网格 overlap / Controls 根级 overlap | 0 / 0 |
| 未命名节点 / 残留构建占位标记 | 0 / 0 |
| Avatar 自身 reaction | 0，符合非交互身份展示语义 |
| Controls 合计 | 61 组件集 / 506 变体 |
| Controls root | 1440 × 34261 |
| Avatar section | 1312 × 896 |

对比度使用 canvas / surface / raised / subtle 四种宿主底色与实际 alpha 合成：

| Theme | 头像文字最低值 | 占位图标 |
|---|---:|---:|
| Light | 4.867:1 | 11.201:1 |
| Dark | 5.225:1 | 16.044:1 |

已审阅：母版、双主题、宽字母、窄宿主以及整个 Avatar 分区的最终截图。之前各组继续以各自独立验收记录为依据，本轮没有宣称重新执行所有既有组件的完整回归。

## 8. 后续代码合同

1. 抽取稳定的 Avatar 实现，统一目录／速览／档案的入口；尺寸对应 34 / 38 / 54，不重新套用旧 CSS 大头像。
2. 统一缩写生成：trim、Unicode normalization、字符簇安全截取；中文优先使用显示名称，英文按姓名分词。处理空、全空白和异常输入后选择 Placeholder。
3. 新增 Person 图标与中性默认占位；Shield 仅表示球队，不表达认证或可信度。
4. 在名称旁将头像子树设为 `aria-hidden=true`；单独显示时提供可访问名称。头像不单独增加 tab stop。
5. 加载或禁用外层操作时保留现有身份信息；伤病／可用性继续使用其他状态组件。
6. 保持 flex 不收缩；宿主名称使用省略或换行，不能靠缩小头像字号塞入完整姓名。
7. 图片若以后进入数据模型，单独补齐资产来源、裁切、加载失败与回退合同；本组没有预先宣称这条链路已实现。

未发布推测性 Code Connect：现有代码是多处 span/div 模板及缩写 helper，不是稳定的一对一 Avatar 组件。变量 WEB syntax 作为后续实现合同保留。

## 9. 完成状态

既定底层组件组剩余 **0 组**。下一阶段为 [Patterns](FIGMA_PATTERN_SCREEN_PLAN.md)，先做 PC App Shell。

Patterns `3:4` 与 Screens `3:5` 已回读确认仍为空；应用源码和业务链路未在本轮修改。
