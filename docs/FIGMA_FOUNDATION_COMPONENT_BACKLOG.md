# Figma Foundation & Component Backlog

只记录 tokens 与底层组件，页面与模式不在本文件处理。

## 基础依赖

- 桌面布局尺寸：Topbar、一级导航、二级侧栏、Inspector、Table Row；
- Overlay / Busy 遮罩颜色；
- Danger Action：Default、Hover、Pressed、Disabled；
- Dialog 专用阴影；
- 数据数字与技术文本样式。

## 底层组件

- Danger Button、Icon Button、Loading Button；
- Search Field、Textarea、Number、Date、Datetime、Password、File Upload；
- Searchable Combobox（不能以现有 Select 替代）；
- Switch / Toggle；
- Tabs / Segmented Control；
- Data Table、Row、Cell、排序、选择、空状态、Pagination；
- Dropdown Menu / Context Menu / Overflow Menu；
- Dialog：普通、确认、危险确认；
- Toast、Inline Alert、Blocking Message；
- Accordion / Disclosure；
- Spinner、Progress、Skeleton、Empty State；
- Avatar：球队、球员、默认占位。

## 组件族验收

- 尺寸、文字、图标、内边距、点击区均无溢出；
- Default / Hover / Pressed / Disabled 等适用状态完整；
- 图标只通过 Icon Slot 嵌套；
- Tag 的可删除语义与 Badge 的状态语义保持分离；
- 变体、属性和默认值稳定，页面可直接实例化，禁止临时复制拼接。
