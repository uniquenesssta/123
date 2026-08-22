# UI Design System Component Registry

Branch: ui-design-system

## Purpose

记录软件 UI 重构过程中的所有组件、状态、交互规则和页面依赖关系。

## Design Principles

- Every user flow must have a complete UI path.
- Every interactive control must define its state.
- No visual-only components without behavior specification.
- Loading, empty, error, disabled and permission states are required.

## Core Components

### Navigation
- App Shell
- Sidebar
- Top Bar
- Breadcrumb

### Action Components
- Primary Button
- Secondary Button
- Danger Button
- Icon Button
- Loading Button

States:
- Default
- Hover
- Active
- Disabled
- Loading

### Form Components
- Input
- Select
- Checkbox
- Switch
- Slider

### Feedback Components
- Toast
- Modal
- Drawer
- Dialog
- Empty State
- Error State
- Skeleton

### Data Components
- Table
- Card
- List
- Pagination

## Pending Audit

- Application modules
- User workflows
- Existing screens
- Component dependencies
- Figma node mapping
