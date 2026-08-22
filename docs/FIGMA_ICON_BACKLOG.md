# Figma Icon Backlog

仅记录应用图标库的续作任务。完成后更新 FIGMA_COMPONENT_RECORD.md 的“图标”事实清单。

## 当前状态

代码在 src/components/icons.ts 中定义 25 个 AppIcon。Figma 已覆盖 7 个对应的应用语义；以下 18 个尚未建立：

shield、users、sheet、chat、chart、settings、history、database、plug、info、panel-left、panel-right、refresh、reset、compare、cards、detail、more。

## 制作规则

- 每个图标有独立 Figma 图标母版；
- 接入 Icon Slot 的 16 / 20 / 24 三种尺寸；
- 与已批准的小图标保持细线条、圆角端点和一致视觉重心；
- 名称必须与代码 AppIcon 语义一一对应，不以近似图标代替；
- 逐个检查 16px 下的辨识度、描边粗细与负空间。

## 验收

18 个图标全部成为可交换的 Icon Slot 内容，并更新组件记录。
