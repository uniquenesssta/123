# Figma Icon Backlog

仅记录应用图标库的续作任务。组件事实清单与长期交接信息见 `FIGMA_COMPONENT_RECORD.md`。

## 当前状态

- 代码来源：`src/components/icons.ts`（`main`，SHA `b0cef778236d596ab94ea0a6ff0083da0481d19f`）。
- AppIcon 代码语义覆盖：25 / 25。
- P4.1 补充图标：18 / 18。
- 状态：已完成并通过统一审计。

## 已完成

- [x] shield — `Icon/Utility/Shield`（83:259）
- [x] users — `Icon/Entity/Users`（88:218）
- [x] sheet — `Icon/Utility/Sheet`（88:442）
- [x] chat — `Icon/Communication/Chat`（88:462）
- [x] chart — `Icon/Data/Chart`（88:482）
- [x] settings — `Icon/Utility/Settings`（88:502）
- [x] history — `Icon/Utility/History`（88:522）
- [x] database — `Icon/Data/Database`（88:542）
- [x] plug — `Icon/Utility/Plug`（88:562）
- [x] info — `Icon/Utility/Info`（88:582）
- [x] panel-left — `Icon/Layout/Panel Left`（88:602）
- [x] panel-right — `Icon/Layout/Panel Right`（88:622）
- [x] refresh — `Icon/Utility/Refresh`（88:643）
- [x] reset — `Icon/Utility/Reset`（88:664）
- [x] compare — `Icon/Utility/Compare`（88:684）
- [x] cards — `Icon/Layout/Cards`（88:706）
- [x] detail — `Icon/Layout/Detail`（88:727）
- [x] more — `Icon/Utility/More`（88:747）

## 组件前置图标

以下 4 个不是 `AppIcon` 代码键，而是底层组件真实链路触发的最小依赖：

- [x] plus — `Icon/Utility/Plus`（139:208）
- [x] calendar — `Icon/Utility/Calendar`（139:212）
- [x] chevron-up — `Icon/Utility/Chevron Up`（168:208）
- [x] person — `Icon/Entity/Person`（356:212）

Plus / Calendar 位于 `Section · Extended field icons`（139:213）；Chevron Up 文档卡为 `Utility · Chevron Up`（168:206）。前三个均为 24px / 1.75 stroke，并绑定图标尺寸、粗细与颜色变量；Chevron Up 已加入 Icon Slot preferred swaps。

Person 文档卡为 `356:206`，使用官方 [Lucide user-round](https://github.com/lucide-icons/lucide/blob/main/icons/user-round.svg) 的 Head / Shoulders 两条几何路径，绑定颜色与描边变量。24 / 20 / 16px、Light / Dark 六个样例已检查。Person 与复用的 Shield 加入 Icon Slot preferred swaps，保留原 13 项及默认图标，现为 15 项。完整记录见 [Avatar](FIGMA_AVATAR.md)。

Iconography 当前有 34 个独立 `Icon/*` 母版。Person 尚未加入代码 AppIcon 类型，不计入 25 / 25 的代码覆盖数。

## 制作规则

- 每个图标有独立 Figma 图标母版。
- 通过 Icon Slot 接入 16 / 20 / 24 三种尺寸。
- 使用已批准的细线条：24px=1.75、20px=1.5、16px=1.25。
- 统一圆角端点与连接，保持一致视觉重心。
- 已有 AppIcon 图标名称与代码语义一一对应；新增组件依赖单独列出，不冒充已有代码能力或使用近似图标替代。
- 图标颜色与描边粗细绑定 Figma 变量；页面和控件不得复制 SVG 路径。

## 最终验收

- 代码覆盖：25 / 25。
- 新 AppIcon 母版：18 / 18。
- 组件前置图标：4 / 4。
- 图标色变量绑定：18 / 18。
- 描边变量绑定：18 / 18。
- Icon Slot 实际替换：18 / 18。
- 展示网格：6 列 × 3 行，共 18 张文档卡片。
- 重叠：0。
- 网格溢出：0。
- 卡片子层溢出：0。
- 预期名称重复：0。
- 未命名母版：0。
- 结果：PASS。

P4.1 与当前组件前置图标均已关闭；后续新增图标仍必须由实际底层组件依赖触发。
