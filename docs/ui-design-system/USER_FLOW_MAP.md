# UI User Flow Map

## Goal

建立完整用户操作链路，确保每个入口、按钮、状态和结果均有 UI 对应。

## Global Flow

```
Launch App
  ↓
Application Shell
  ↓
Workspace Selection
  ↓
Feature Module
  ↓
Action
  ↓
Confirmation / Input
  ↓
Processing State
  ↓
Success / Error Recovery
```

## Dashboard / Workspace

入口：主工作台

Actions:

- 查看系统状态
- 进入功能模块
- 查看任务状态
- 查看最近操作

States:

- Normal
- Loading
- Empty
- Error

## Match Analysis Flow

```
选择赛事
 ↓
查看赛事信息
 ↓
选择分析参数
 ↓
执行分析
 ↓
生成分析结果
 ↓
保存/复盘
```

Required UI:

- Match selector
- Parameter panel
- Execute button
- Progress indicator
- Result panel
- Export action

## Team Management Flow

```
球队列表
 ↓
球队详情
 ↓
编辑资料
 ↓
保存
 ↓
刷新详情
```

Required UI:

- Search
- Filter
- Detail drawer
- Edit form
- Save confirmation

## Player Management Flow

```
球员列表
 ↓
球员详情
 ↓
关联球队
 ↓
修改资料
 ↓
保存
```

Required UI:

- Table
- Pagination
- Search
- Relationship selector
- Validation feedback

## API / Model Workspace Flow

```
API配置
 ↓
选择模型入口
 ↓
发送请求
 ↓
查看日志
 ↓
查看返回结果
```

Required UI:

- Configuration form
- Connection status
- Request button
- Runtime log
- Response viewer

## History / Review Flow

```
历史记录
 ↓
选择记录
 ↓
查看详情
 ↓
复盘分析
```

Required UI:

- History list
- Detail page
- Compare view
- Review actions

## Universal Button States

Every interactive element must define:

- Default
- Hover
- Active
- Disabled
- Loading
- Success
- Error
- Permission denied

## Permission Flow

```
User Action
 ↓
Permission Check
 ↓
Allowed
 ↓
Execute

or

Denied
 ↓
Explain reason
 ↓
Provide next action
```

## Figma Mapping Rule

Each flow node maps to:

- Screen
- Component
- Variant
- Interaction
- Prototype connection
