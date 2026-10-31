import { escapeHtml } from "../../../components/format";
import type { WorkspaceSectionItem, WorkspaceAnchorItem } from "./sectionNavigationTypes";
export function workspaceSectionNavigation(items: readonly WorkspaceSectionItem[], activeId: string): string {
    return `<nav class="workspace-section-nav" data-ui-navigation="horizontal" aria-label="模块功能">${items.map((item) => `<button class="workspace-section-nav-item ${item.id === activeId ? "active" : ""}" data-action="select-workspace-section" data-section-id="${escapeHtml(item.id)}" aria-current="${item.id === activeId ? "page" : "false"}" ${item.disabled ? 'disabled aria-disabled="true"' : ""}><span class="workspace-section-copy"><strong>${escapeHtml(item.label)}</strong></span>${item.badge ? `<b>${escapeHtml(item.badge)}</b>` : ""}</button>`).join("")}</nav>`;
}
export function workspaceTaskAnchorNavigation(items: readonly WorkspaceSectionItem[], activeId = items[0]?.id): string {
    return `<nav class="workspace-section-nav workspace-task-anchor-nav" data-ui-navigation="horizontal" aria-label="页面任务">${items.map((item) => `<button class="workspace-section-nav-item ${item.id === activeId ? "active" : ""}" data-action="jump-workspace-anchor" data-anchor-id="${escapeHtml(item.id)}" aria-current="${item.id === activeId ? "page" : "false"}" ${item.disabled ? 'disabled aria-disabled="true"' : ""}><span class="workspace-section-copy"><strong>${escapeHtml(item.label)}</strong></span>${item.badge ? `<b>${escapeHtml(item.badge)}</b>` : ""}</button>`).join("")}</nav>`;
}
export function workspaceAnchorNavigation(label: string, items: readonly WorkspaceAnchorItem[], orientation: "horizontal" | "vertical" = "horizontal"): string {
    return `<nav class="workspace-anchor-nav" data-ui-navigation="${orientation}" aria-label="${escapeHtml(label)}"><span class="workspace-anchor-label">${escapeHtml(label)}</span><div>${items.map((item, index) => `<button class="${index === 0 ? "active" : ""}" data-action="jump-workspace-anchor" data-anchor-id="${escapeHtml(item.id)}">${escapeHtml(item.label)}</button>`).join("")}</div></nav>`;
}
