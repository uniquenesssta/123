import { escapeHtml } from "../../../components/format";
import { PRIMARY_NAVIGATION, type PrimaryNavigationModule } from "../../router";
import { iconSlot } from "../../../ui/icons/iconSlot";
import { navigationIcon } from "./sidebarIcons";
import { iconButtonView } from "../../../ui/primitives/icon-button";
import { serviceStatusView } from "../topbar/serviceStatusView";
import type { ShellOptions } from "../application-shell/applicationShellState";

export function sidebarView(options: ShellOptions, activeModule: PrimaryNavigationModule): string {
  const themeLabel = options.theme === "dark" ? "切换浅色主题" : "切换深色主题";
  const version = options.state.data.app_version;
  return `<aside class="primary-rail" aria-label="一级菜单" data-shell-controls>
    <div class="primary-brand" title="足球赛事模型平台"><span>FIP</span></div>
    <nav class="primary-navigation" aria-label="业务模块">
      ${PRIMARY_NAVIGATION.map(module => `<button type="button" class="primary-nav-item ${module.key === activeModule.key ? "active" : ""}"
        data-page="${module.default_page}" data-primary-module="${module.key}" title="${module.label} · ${module.description}" aria-label="${module.label}" ${module.key === activeModule.key ? 'aria-current="true"' : ""}>
        ${iconSlot(navigationIcon(module.default_page), 20)}<b>${module.label}</b></button>`).join("")}
    </nav>
    <div class="primary-rail-spacer"></div>
    <div class="primary-theme-footer"><button type="button" class="primary-theme-control" data-action="toggle-theme" title="${themeLabel}" aria-label="${themeLabel}">
      ${iconSlot("contrast", 20)}<b>主题</b></button></div>
  </aside>
  <aside id="shell-secondary-navigation" class="secondary-sidebar" aria-label="${activeModule.label}二级菜单" data-shell-controls ${options.sidebarCollapsed ? "hidden inert" : ""}>
    <header class="secondary-sidebar-header"><span>${activeModule.label}</span>
      ${iconButtonView({ action: "toggle-global-sidebar", label: "收起二级菜单", icon: "panel-left", className: "secondary-collapse-button", attributes: {"aria-controls":"shell-secondary-navigation","aria-expanded":"true"}})}
    </header>
    <p class="secondary-module-description">${activeModule.description}</p>
    <nav class="secondary-navigation" aria-label="${activeModule.label}功能入口">
      ${activeModule.items.map(item => `<button type="button" class="secondary-nav-item ${item.page === options.page ? "active" : ""}"
        data-page="${item.page}" title="${item.description}" ${item.page === options.page ? 'aria-current="page"' : ""}>
        ${iconSlot(navigationIcon(item.page), 20)}<b>${item.label}</b></button>`).join("")}
    </nav>
    <div class="secondary-sidebar-spacer"></div>
    <footer class="secondary-sidebar-footer">${serviceStatusView(options.state.data.database_configured, false)}
      <small>${version ? escapeHtml(`v${version}`) : "版本未提供"}</small></footer>
  </aside>`;
}
