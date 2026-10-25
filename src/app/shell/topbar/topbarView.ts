import { escapeHtml } from "../../../components/format";
import { navigationItemForPage, type PrimaryNavigationModule } from "../../router";
import { iconButtonView } from "../../../ui/primitives/icon-button";
import { serviceStatusView } from "./serviceStatusView";
import type { ShellOptions } from "../application-shell/applicationShellState";

export function topbarView(options: ShellOptions, activeModule: PrimaryNavigationModule): string {
  const activeTitle = navigationItemForPage(options.page).label;
  const location = `${activeModule.label} / ${activeTitle}`;
  return `<header class="topbar" data-shell-controls>
    ${options.sidebarCollapsed ? iconButtonView({ action: "toggle-global-sidebar", label: "展开二级菜单", icon: "panel-right", className: "secondary-reveal-button", attributes: {"aria-controls":"shell-secondary-navigation","aria-expanded":"false"}}) : ""}
    <div class="topbar-history" aria-label="右侧工作区历史">
      ${iconButtonView({ action: "workspace-history-back", label: "返回上一个右侧界面", icon: "arrow-left", disabled: true})}
      ${iconButtonView({ action: "workspace-history-forward", label: "前进到下一个右侧界面", icon: "arrow-right", disabled: true})}
    </div>
    <span class="topbar-separator" aria-hidden="true"></span>
    <span class="topbar-title" title="${escapeHtml(location)}">${escapeHtml(location)}</span>
    ${serviceStatusView(options.state.data.database_configured, options.navigationPending)}
    <span data-warning-entry></span>
    ${iconButtonView({ action: "reset-current-workspace", label: "重置当前页面工作区", icon: "reset", className: "topbar-reset"})}
  </header>`;
}
