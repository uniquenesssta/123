import { navigationItemForPage, navigationModuleForPage } from "../../router";
import type { Page } from "../../../types";
import { sidebarView } from "../sidebar/sidebarView";
import { topbarView } from "../topbar/topbarView";
import { pageFrameView, workspacePages } from "../page-frame/pageFrameView";
import type { ShellOptions } from "./applicationShellState";

export function pageTitle(page: Page): string { return navigationItemForPage(page).label; }

export function renderShell(options: ShellOptions): string {
  const activeModule = navigationModuleForPage(options.page);
  return `<div class="app-shell dual-navigation application-shell ${options.sidebarCollapsed ? "sidebar-collapsed secondary-collapsed" : ""} ${workspacePages.has(options.page) ? "workspace-page" : ""}"
    data-current-page="${options.page}" data-current-module="${activeModule.key}">
    ${sidebarView(options, activeModule)}
    ${pageFrameView(options, topbarView(options, activeModule))}
  </div>
  <div data-confirmation-host></div>
  <div id="toast" class="toast" role="status" aria-live="polite"></div>
  <div id="busy" class="task-activity ${options.busy ? "visible" : ""}" aria-hidden="${!options.busy}"><div class="spinner"></div><span>正在处理</span></div>`;
}
