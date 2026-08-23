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

## 制作规则

- 每个图标有独立 Figma 图标母版。
- 通过 Icon Slot 接入 16 / 20 / 24 三种尺寸。
- 使用已批准的细线条：24px=1.75、20px=1.5、16px=1.25。
- 统一圆角端点与连接，保持一致视觉重心。
- 名称与代码 AppIcon 语义一一对应，不用近似图标替代。
- 图标颜色与描边粗细绑定 Figma 变量；页面和控件不得复制 SVG 路径。

## 最终验收

- 代码覆盖：25 / 25。
- 新图标母版：18 / 18。
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

P4.1 已关闭。下一顺序是补齐基础依赖 token 与桌面布局尺寸，再扩展底层控件。
