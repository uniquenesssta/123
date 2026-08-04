import { escapeHtml } from "../components/format";
import { icon } from "../components/icons";
import {
  PRIMARY_NAVIGATION,
  navigationItemForPage,
  navigationModuleForPage,
  type PrimaryNavigationModule,
  type SecondaryNavigationItem,
} from "./navigation";
import type { BootstrapResponse, Page, Theme } from "../types";

export function pageTitle(current: Page): string {
  return navigationItemForPage(current).label;
}

interface ShellOptions {
  readonly state: BootstrapResponse;
  readonly page: Page;
  readonly theme: Theme;
  readonly content: string;
  readonly busy: boolean;
  readonly navigationPending: boolean;
  /**
   * Compatibility name retained for persisted workspace state.
   * In the dual-level shell this collapses only the secondary menu.
   */
  readonly sidebarCollapsed: boolean;
}

function primaryNavButton(module: PrimaryNavigationModule, current: PrimaryNavigationModule): string {
  const active = module.key === current.key;
  return `<button
    class="primary-nav-item ${active ? "active" : ""}"
    data-page="${module.default_page}"
    data-primary-module="${module.key}"
    title="${module.label} · ${module.description}"
    aria-label="${module.label}"
    ${active ? 'aria-current="true"' : ""}
  >${icon(module.icon, "primary-nav-svg")}<b>${module.label}</b></button>`;
}

function secondaryNavButton(item: SecondaryNavigationItem, current: Page): string {
  const active = item.page === current;
  return `<button
    class="secondary-nav-item ${active ? "active" : ""}"
    data-page="${item.page}"
    title="${item.description}"
    ${active ? 'aria-current="page"' : ""}
  ><span class="secondary-nav-icon">${icon(item.icon, "secondary-nav-svg")}</span><span><b>${item.label}</b><small>${item.description}</small></span></button>`;
}

export function renderShell(options: ShellOptions): string {
  const { state, page, theme, content, busy, navigationPending, sidebarCollapsed } = options;
  const activeModule = navigationModuleForPage(page);
  const activeItem = navigationItemForPage(page);
  const themeLabel = theme === "dark" ? "切换浅色主题" : "切换深色主题";
  const connected = Boolean(state.data.database_configured);
  const serviceLabel = navigationPending ? "正在载入" : connected ? "服务正常" : "等待数据库";
  const activeTitle = activeItem.label;
  const workspacePage = ["lineups", "teams", "players", "lineup_presets", "prediction", "workbooks", "rules", "release", "analytics", "api_workspace", "openai", "database", "logs", "architecture"].includes(page);

  return `
    <div
      class="app-shell dual-navigation ${sidebarCollapsed ? "sidebar-collapsed secondary-collapsed" : ""} ${workspacePage ? "workspace-page" : ""}"
      data-current-page="${page}"
      data-current-module="${activeModule.key}"
    >
      <aside class="primary-rail" aria-label="一级菜单">
        <div class="primary-brand" title="足球赛事模型平台" aria-label="足球赛事模型平台">
          <div class="brand-mark" aria-hidden="true"><span></span></div>
        </div>
        <nav class="primary-navigation">
          ${PRIMARY_NAVIGATION.map((module) => primaryNavButton(module, activeModule)).join("")}
        </nav>
        <div class="primary-rail-spacer"></div>
        <button class="primary-theme-control" data-action="toggle-theme" title="${themeLabel}" aria-label="${themeLabel}">
          <span class="theme-swatch" aria-hidden="true"></span><b>主题</b>
        </button>
      </aside>

      <aside class="secondary-sidebar" aria-label="${activeModule.label}二级菜单">
        <header class="secondary-sidebar-header">
          <div>
            <span>${activeModule.label}</span>
            <strong>${activeModule.description}</strong>
          </div>
          <button class="secondary-collapse-button icon-button" data-action="toggle-global-sidebar" title="收起二级菜单" aria-label="收起二级菜单">${icon("panel-left")}</button>
        </header>
        <nav class="secondary-navigation">
          <span class="secondary-nav-caption">功能入口</span>
          ${activeModule.items.map((item) => secondaryNavButton(item, page)).join("")}
        </nav>
        <div class="secondary-sidebar-spacer"></div>
        <footer class="secondary-sidebar-footer">
          <span class="status-dot ${connected ? "online" : "offline"}"></span>
          <div>
            <strong>${connected ? "数据库已连接" : "数据库未连接"}</strong>
            <small>v${escapeHtml(state.data.app_version ?? "0.23.0")}</small>
          </div>
        </footer>
      </aside>

      <main class="main-content" aria-busy="${navigationPending}">
        <header class="topbar">
          <div class="topbar-leading">
            ${sidebarCollapsed ? `<button class="secondary-reveal-button icon-button" data-action="toggle-global-sidebar" title="展开二级菜单" aria-label="展开二级菜单">${icon("panel-right")}</button>` : ""}
            <div class="topbar-history" aria-label="右侧工作区历史">
              <button class="icon-button" data-action="workspace-history-back" title="返回上一个右侧界面" aria-label="返回上一个右侧界面" disabled>←</button>
              <button class="icon-button" data-action="workspace-history-forward" title="前进到下一个右侧界面" aria-label="前进到下一个右侧界面" disabled>→</button>
            </div>
            <div class="topbar-title">
              <span>${activeModule.label} / ${activeTitle}</span>
              <strong>${activeTitle}</strong>
            </div>
          </div>
          <div class="topbar-actions">
            <div class="topbar-status ${connected ? "online" : "offline"} ${navigationPending ? "loading" : ""}"><i></i>${serviceLabel}</div>
            <button class="icon-button topbar-reset" data-action="reset-current-workspace" title="重置当前页面工作区" aria-label="重置当前页面工作区">${icon("reset")}</button>
          </div>
        </header>
        ${navigationPending ? '<div class="page-load-progress" role="status" aria-label="正在载入页面"></div>' : ""}
        <div class="page-container" ${navigationPending ? "inert" : ""}>${content}</div>
        <div id="modal-root" class="workspace-panel-root" aria-live="polite"></div>
      </main>
    </div>
    <div id="toast" class="toast" role="status" aria-live="polite"></div>
    <div id="busy" class="task-activity ${busy ? "visible" : ""}" aria-hidden="${!busy}"><div class="spinner"></div><span>正在处理</span></div>`;
}
