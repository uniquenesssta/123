import { escapeHtml } from "./format";
import { icon } from "./icons";
import type { WorkspaceLayoutMode, WorkspaceTabState } from "../app/viewState";

export interface WorkspaceSectionItem {
  readonly id: string;
  readonly index: string;
  readonly label: string;
  readonly description?: string;
  readonly badge?: string;
  readonly disabled?: boolean;
}

export interface WorkspaceAnchorItem {
  readonly id: string;
  readonly label: string;
}

export function workspaceSectionNavigation(items: readonly WorkspaceSectionItem[], activeId: string): string {
  return `<nav class="workspace-section-nav" aria-label="模块功能">${items.map((item) => `<button class="workspace-section-nav-item ${item.id === activeId ? "active" : ""}" data-action="select-workspace-section" data-section-id="${escapeHtml(item.id)}" aria-current="${item.id === activeId ? "page" : "false"}" ${item.disabled ? 'disabled aria-disabled="true"' : ""}><span class="workspace-section-index">${escapeHtml(item.index)}</span><span class="workspace-section-copy"><strong>${escapeHtml(item.label)}</strong>${item.description ? `<small>${escapeHtml(item.description)}</small>` : ""}</span>${item.badge ? `<b>${escapeHtml(item.badge)}</b>` : ""}</button>`).join("")}</nav>`;
}


export function workspaceTaskAnchorNavigation(items: readonly WorkspaceSectionItem[]): string {
  return `<nav class="workspace-section-nav workspace-task-anchor-nav" aria-label="页面任务">${items.map((item, index) => `<button class="workspace-section-nav-item ${index === 0 ? "active" : ""}" data-action="jump-workspace-anchor" data-anchor-id="${escapeHtml(item.id)}" ${item.disabled ? 'disabled aria-disabled="true"' : ""}><span class="workspace-section-index">${escapeHtml(item.index)}</span><span class="workspace-section-copy"><strong>${escapeHtml(item.label)}</strong>${item.description ? `<small>${escapeHtml(item.description)}</small>` : ""}</span>${item.badge ? `<b>${escapeHtml(item.badge)}</b>` : ""}</button>`).join("")}</nav>`;
}

export function workspaceAnchorNavigation(label: string, items: readonly WorkspaceAnchorItem[]): string {
  return `<nav class="workspace-anchor-nav" aria-label="${escapeHtml(label)}"><span class="workspace-anchor-label">${escapeHtml(label)}</span><div>${items.map((item, index) => `<button class="${index === 0 ? "active" : ""}" data-action="jump-workspace-anchor" data-anchor-id="${escapeHtml(item.id)}">${escapeHtml(item.label)}</button>`).join("")}</div></nav>`;
}

export function workspaceTabs(scope: "teams" | "players", tabs: readonly WorkspaceTabState[], activeId: string | null): string {
  if (!tabs.length) return `<div class="workspace-tabs-empty"><span>尚未打开对象</span><small>从左侧目录选择一项开始</small></div>`;
  return `<div class="workspace-tabs" role="tablist">${tabs.map((tab) => `<div class="workspace-tab ${tab.id === activeId ? "active" : ""}"><button class="workspace-tab-main" role="tab" aria-selected="${tab.id === activeId}" data-action="activate-${scope}-tab" data-object-id="${escapeHtml(tab.id)}"><span>${escapeHtml(tab.label)}</span></button><button class="workspace-tab-close" data-action="close-${scope}-tab" data-object-id="${escapeHtml(tab.id)}" aria-label="关闭 ${escapeHtml(tab.label)}">${icon("close")}</button></div>`).join("")}</div>`;
}

export function workspaceLayoutControls(scope: "teams" | "players", mode: WorkspaceLayoutMode, tabCount: number): string {
  const buttons: Array<[WorkspaceLayoutMode, string, "detail" | "compare" | "cards"]> = [
    ["detail", "详情", "detail"],
    ["compare", "比较", "compare"],
    ["cards", "概览", "cards"],
  ];
  return `<div class="workspace-layout-controls" aria-label="显示模式">${buttons.map(([value, label, iconName]) => `<button class="${mode === value ? "active" : ""}" data-action="set-${scope}-workspace-mode" data-mode="${value}" title="${label}" aria-label="${label}" ${value !== "detail" && tabCount < 2 ? "disabled" : ""}>${icon(iconName)}<span>${label}</span></button>`).join("")}</div>`;
}

export function workspacePaneToggle(pane: "module-sidebar" | "inspector", collapsed: boolean): string {
  const isSidebar = pane === "module-sidebar";
  return `<button class="workspace-pane-toggle icon-button" data-action="toggle-workspace-pane" data-pane="${pane}" title="${collapsed ? "展开" : "收起"}${isSidebar ? "目录" : "检查器"}" aria-label="${collapsed ? "展开" : "收起"}${isSidebar ? "目录" : "检查器"}">${icon(isSidebar ? "panel-left" : "panel-right")}</button>`;
}
