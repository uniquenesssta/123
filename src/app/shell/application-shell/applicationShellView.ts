import { applicationFrameView } from "../../../ui/layout";
import { navigationItemForPage, navigationModuleForPage } from "../../router";
import type { Page } from "../../../types";
import { sidebarView } from "../sidebar/sidebarView";
import { topbarView } from "../topbar/topbarView";
import { pageFrameView, workspacePages } from "../page-frame/pageFrameView";
import type { ShellOptions } from "./applicationShellState";

export function pageTitle(page: Page): string { return navigationItemForPage(page).label; }

export function renderShell(options: ShellOptions): string {
  const activeModule = navigationModuleForPage(options.page);
  return applicationFrameView({navigation:sidebarView(options,activeModule),main:pageFrameView(options,topbarView(options,activeModule)),collapsed:options.sidebarCollapsed,workspace:workspacePages.has(options.page),page:options.page,module:activeModule.key});
}
