import { escapeHtml } from "../../../components/format";
import { navigationItemForPage, type PrimaryNavigationModule } from "../../router";
import { iconButtonView } from "../../../ui/primitives/icon-button/iconButtonView";
import { serviceStatusView } from "./serviceStatusView";
import type { ShellOptions } from "../application-shell/applicationShellState";

export function topbarView(options: ShellOptions, activeModule: PrimaryNavigationModule): string {
  const activeTitle = navigationItemForPage(options.page).label;
  const location = `${activeModule.label} / ${activeTitle}`;
  return `<header class="topbar" data-shell-controls>
    ${options.sidebarCollapsed ? iconButtonView("toggle-global-sidebar", "展开二级菜单", "panel-right", "secondary-reveal-button", 'aria-controls="shell-secondary-navigation" aria-expanded="false"') : ""}
    <div class="topbar-history" aria-label="右侧工作区历史">
      ${iconButtonView("workspace-history-back", "返回上一个右侧界面", "arrow-left", "", "disabled")}
      ${iconButtonView("workspace-history-forward", "前进到下一个右侧界面", "arrow-right", "", "disabled")}
    </div>
    <span class="topbar-separator" aria-hidden="true"></span>
    <span class="topbar-title" title="${escapeHtml(location)}">${escapeHtml(location)}</span>
    ${serviceStatusView(options.state.data.database_configured, options.navigationPending)}
    <span data-warning-entry></span>
    ${iconButtonView("reset-current-workspace", "重置当前页面工作区", "reset", "topbar-reset")}
  </header>`;
}
