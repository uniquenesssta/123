import { escapeHtml } from "../../../components/format";
import { iconSlot } from "../../icons/iconSlot";
import { iconButtonView } from "../../primitives/icon-button";
import { navigationItemView, type NavigationItem } from "./navigationItemView";
export interface SidebarOptions {
    title: string;
    description: string;
    themeLabel: string;
    collapsed: boolean;
    primary: readonly NavigationItem[];
    secondary: readonly NavigationItem[];
    footer: string;
}
/** Trusted footer is composed by the host; route and service ownership stay there. */
export function navigationSidebarView(options: SidebarOptions): string {
    return `<aside class="primary-rail" aria-label="一级菜单" data-shell-controls><div class="primary-brand" title="足球赛事模型平台"><span>FIP</span></div><nav class="primary-navigation" aria-label="业务模块" data-ui-navigation="vertical">${options.primary.map(i => navigationItemView(i, "primary")).join("")}</nav><div class="primary-rail-spacer"></div><div class="primary-theme-footer"><button type="button" class="primary-theme-control" data-action="toggle-theme" title="${escapeHtml(options.themeLabel)}" aria-label="${escapeHtml(options.themeLabel)}">${iconSlot("contrast", 20)}<b>主题</b></button></div></aside>
 <aside id="shell-secondary-navigation" class="secondary-sidebar" aria-label="${escapeHtml(options.title)}二级菜单" data-shell-controls ${options.collapsed ? "hidden inert" : ""}><header class="secondary-sidebar-header"><span>${escapeHtml(options.title)}</span>${iconButtonView({ action: "toggle-global-sidebar", label: "收起二级菜单", icon: "panel-left", className: "secondary-collapse-button", attributes: { "aria-controls": "shell-secondary-navigation", "aria-expanded": "true" } })}</header><p class="secondary-module-description">${escapeHtml(options.description)}</p><nav class="secondary-navigation" aria-label="${escapeHtml(options.title)}功能入口" data-ui-navigation="vertical">${options.secondary.map(i => navigationItemView(i, "secondary")).join("")}</nav><div class="secondary-sidebar-spacer"></div><footer class="secondary-sidebar-footer">${options.footer}</footer></aside>`;
}
