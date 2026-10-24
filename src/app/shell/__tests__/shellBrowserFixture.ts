import { ApplicationShellController, renderShell } from "../index";
import { AppSession, SessionPersistence } from "../../session";
import { Router, PRIMARY_NAVIGATION, type NavigationRequest } from "../../router";
import { ModalController } from "../../modal";
import { WorkspaceCoordinator } from "../../workspace";
import { verifyWorkspaceRecovery } from "../../workspace/__tests__/workspaceBrowserFixture";
import type { BootstrapResponse, Page, Theme } from "../../../types";

/** Browser fixture injects data adapters only; all Shell/route/session/workspace code is real. */
export async function installShellFixture(theme: Theme, collapsed: boolean): Promise<{ checks: string[]; finish(): void }> {
  const root = document.querySelector<HTMLElement>("#shell-host")!;
  const checks: string[] = [];
  function expect(condition: unknown, message: string): void {
    if (!condition) throw new Error(message);
    checks.push(message);
  }
  await verifyWorkspaceRecovery(root, expect);
  const frames = () => new Promise<void>(resolve => requestAnimationFrame(() => requestAnimationFrame(() => resolve())));
  const summary = { data: { database_configured: true, app_version: "0.23.0", recent_runs: [] } } as unknown as BootstrapResponse;
  const preferences = new Map<string, string>([["football-model-platform.theme", theme]]);
  const session = new AppSession({ loadBootstrap: async () => summary, loadRecentRuns: async () => [], applyTheme(value) { document.documentElement.dataset.theme = value; } },
    new SessionPersistence(() => ({ getItem: key => preferences.get(key) ?? null, setItem(key, value) { preferences.set(key, value); } })));
  await session.start();
  const workspace = new WorkspaceCoordinator<Page>({ read: async () => ({ schema_version: 1, global: { sidebar_collapsed: false, ui_revision: 5 }, modules: {} }), save: async () => {}, clear: async () => ({ schema_version: 1, global: { sidebar_collapsed: false, ui_revision: 5 }, modules: {} }) });
  await workspace.initialize();
  const router = new Router();
  const modal = new ModalController();
  let painted: Page | null = null;
  let activations = 0;
  let pending: Promise<boolean> | null = null;
  let resolveLoad: (() => void) | null = null;
  let deferLoad = false;
  let failLoad = false;
  let actionController: ApplicationShellController;
  const query = <T extends HTMLElement>(selector: string) => root.querySelector<T>(selector)!;
  const button = (action: string) => Array.from(root.querySelectorAll<HTMLButtonElement>(`[data-shell-controls] [data-action="${action}"]`)).find(item => !item.closest("[hidden]"))!;
  const field = () => query<HTMLInputElement>("#shell-draft");
  const pageRoot = () => query<HTMLElement>(".page-container");
  function paint(): void {
    const focus = actionController.captureFocus();
    modal.capture();
    if (painted) workspace.capture(painted, pageRoot(), true);
    root.innerHTML = renderShell({ state: session.bootstrap!, page: router.page, theme: session.theme, content: `<section class="shell-test-content"><h1>${router.page}</h1><label>工作区草稿 <input id="shell-draft" value="初始输入"></label><div style="height:1600px"></div></section>`, navigationPending: router.pendingPage === router.page, navigationFailed: controller.navigationFailed, sidebarCollapsed: workspace.sidebarCollapsed() });
    modal.restore();
    painted = router.page;
    workspace.restore(router.page, pageRoot());
    actionController.restoreFocus(focus);
  }
  function reset(): void {
    actionController.confirmWorkspaceReset(router.page, () => {
      modal.reset(); workspace.clear(router.page); painted = null; paint();
    });
  }
  const actions = {
    navigate(page: Page) {
      if (router.page === page) return;
      activations++;
      readPage(page);
    },
    action(action: string) {
      activations++;
      void actionController.handleAction(action);
    },
  };
  const operations = {
    sidebarCollapsed: () => workspace.sidebarCollapsed(), setSidebarCollapsed: (value: boolean) => workspace.setSidebarCollapsed(value),
    toggleTheme: () => { session.toggleTheme(); }, renderPreservingWorkspace: paint,
    historyBack: () => modal.back(), historyForward: () => modal.forward(), resetWorkspace: () => { if (!router.pendingPage) reset(); },
    retryPageRead: () => readPage(router.page), openLogs: () => readPage("logs"),
  };
  const controller = new ApplicationShellController(root, actions, operations);
  actionController = controller;
  function readPage(page: Page): void {
    pending = router.navigate(page, async (request: NavigationRequest) => {
      try {
        if (deferLoad) await new Promise<void>(resolve => { resolveLoad = resolve; });
        if (failLoad) throw new Error("fixture page read failure");
        if (router.isCurrent(request)) controller.clearNavigationFailure();
      } catch (error) { if (router.isCurrent(request)) controller.showNavigationFailure(); throw error; }
    }, { started() { modal.reset(); controller.clearNavigationFailure(); paint(); }, settled: paint });
  }
  controller.start(); controller.start(); paint(); await frames();
  expect(query(".primary-rail").getBoundingClientRect().width === 60, "primary rail 60");
  expect(query(".secondary-sidebar").getBoundingClientRect().width === 154, "secondary rail 154");
  expect(query(".topbar").getBoundingClientRect().height === 44, "topbar 44");
  expect(query(".application-shell").getBoundingClientRect().right === innerWidth && query(".application-shell").getBoundingClientRect().bottom === innerHeight, "shell fills viewport without a reserved document gutter");
  expect(query(".primary-nav-item").getBoundingClientRect().width === 52 && query(".primary-nav-item").getBoundingClientRect().height === 48, "primary target 52x48");
  expect(query(".secondary-nav-item").getBoundingClientRect().width === 138 && query(".secondary-nav-item").getBoundingClientRect().height === 40, "secondary target 138x40");
  expect(button("workspace-history-back").getBoundingClientRect().width === 32, "icon target 32");
  expect(getComputedStyle(button("workspace-history-back")).backgroundColor === "rgba(0, 0, 0, 0)" && getComputedStyle(button("workspace-history-back")).borderRadius === "7px", "icon primitive is isolated from legacy important styles");
  expect(root.querySelectorAll(".primary-nav-item").length === 7 && root.querySelectorAll(".secondary-nav-item").length === 1, "seven modules and contextual leaves");
  expect(root.textContent!.includes("数据库已配置") && !root.textContent!.includes("服务正常") && !root.textContent!.includes("数据库已连接"), "configuration is not connection health");
  expect(document.documentElement.scrollHeight === innerHeight && pageRoot().scrollHeight > pageRoot().clientHeight, "workspace scroll independent of chrome");
  field().value = "未保存的输入"; field().focus(); pageRoot().scrollTop = 321;
  button("toggle-theme").click(); await frames();
  expect(field().value === "未保存的输入" && pageRoot().scrollTop === 321, "theme preserves draft and scroll");
  const beforeCollapse = activations;
  button("toggle-global-sidebar").focus(); button("toggle-global-sidebar").click(); await frames();
  expect(activations === beforeCollapse + 1, "start idempotent and one action per click");
  expect(workspace.sidebarCollapsed() && query(".secondary-sidebar").hidden && query(".main-content").getBoundingClientRect().left === 60, "only secondary collapses");
  expect(document.activeElement === button("toggle-global-sidebar"), `collapse transfers focus to reveal (active=${document.activeElement?.tagName}:${(document.activeElement as HTMLElement)?.dataset.action}; expected=${button("toggle-global-sidebar").className})`);
  expect(field().value === "未保存的输入" && pageRoot().scrollTop === 321 && router.page === "dashboard", "collapse preserves route draft scroll");
  button("toggle-global-sidebar").click(); await frames();
  expect(!workspace.sidebarCollapsed() && query(".main-content").getBoundingClientRect().left === 214, "reveal restores secondary width");
  const detail = (title: string) => modal.showHtml(title, "详情", '<input id="detail-draft"><div style="height:1500px"></div>');
  detail("详情A"); query<HTMLInputElement>("#detail-draft").value = "详情输入A"; query(".workspace-detail-body").scrollTop = 180;
  detail("详情B"); query<HTMLInputElement>("#detail-draft").value = "详情输入B";
  button("workspace-history-back").click();
  expect(query<HTMLInputElement>("#detail-draft").value === "详情输入A" && query(".workspace-detail-body").scrollTop === 180, "back restores detail input and scroll");
  button("workspace-history-forward").click();
  expect(query<HTMLInputElement>("#detail-draft").value === "详情输入B", "forward restores detail input");
  button("reset-current-workspace").click();
  expect(!!root.querySelector("dialog:modal") && document.activeElement === query("dialog [autofocus]"), "native confirmation opens with Cancel focused");
  const lockedActivations = activations; button("workspace-history-back").click();
  expect(activations === lockedActivations, "confirmation guards underlying history actions");
  paint(); await frames();
  expect(!!root.querySelector("dialog:modal"), "confirmation survives a shell redraw");
  query<HTMLButtonElement>('dialog footer [data-confirmation-action="cancel"]').click();
  expect(query<HTMLInputElement>("#detail-draft").value === "详情输入B", "reset cancellation restores original detail");
  button("workspace-history-back").click(); button("workspace-history-back").click();
  expect(!root.querySelector("#detail-draft") && !pageRoot().hasAttribute("aria-hidden"), "first detail back returns to base");
  const emptyBackCount = activations; button("workspace-history-back").dispatchEvent(new MouseEvent("click", { bubbles: true }));
  expect(activations === emptyBackCount, "disabled action cannot dispatch");
  button("workspace-history-forward").click(); button("toggle-theme").click(); await frames();
  expect(query<HTMLInputElement>("#detail-draft").value === "详情输入A", "theme preserves detail history");
  const beforeSelected = activations;
  query<HTMLButtonElement>('.primary-nav-item[data-page="dashboard"]').click();
  expect(activations === beforeSelected && !!root.querySelector("#detail-draft"), "selected route is a no-op");
  const resetTheme = session.theme, resetPage = router.page, resetCollapsed = workspace.sidebarCollapsed();
  workspace.patchModule("players", { controls: { other: { value: "保留其他页面" } } });
  button("reset-current-workspace").click(); query<HTMLButtonElement>('[data-confirmation-action="confirm"]').click(); await frames();
  expect(field().value === "初始输入" && pageRoot().scrollTop === 0 && button("workspace-history-forward").disabled, "confirmed reset cannot recapture discarded draft or detail");
  expect(session.theme === resetTheme && router.page === resetPage && workspace.sidebarCollapsed() === resetCollapsed && workspace.module("players").controls.other.value === "保留其他页面", "reset is scoped to current workspace");
  detail("再次打开的详情");
  query<HTMLButtonElement>('.primary-nav-item[data-page="lineups"]').click(); await pending; await frames();
  expect(router.page === "lineups" && button("workspace-history-forward").disabled && !root.querySelector("#detail-draft"), "real page navigation clears detail history");
  for (const module of PRIMARY_NAVIGATION) {
    query<HTMLButtonElement>(`.primary-nav-item[data-primary-module="${module.key}"]`).click(); await pending; await frames();
    expect(router.page === module.default_page && root.querySelectorAll(".secondary-nav-item").length === module.items.length, `module default ${module.key}`);
    for (const item of module.items) { query<HTMLButtonElement>(`.secondary-nav-item[data-page="${item.page}"]`).click(); await pending; await frames(); expect(router.page === item.page && root.querySelectorAll('.secondary-nav-item[aria-current="page"]').length === 1, `selected route ${item.page}`); }
  }
  deferLoad = true;
  query<HTMLButtonElement>('.primary-nav-item[data-page="lineups"]').click(); await frames();
  expect(pageRoot().inert && query("#modal-root").inert && query(".page-container").getAttribute("aria-busy") === "true", "loading locks only workspace");
  expect(!query(".primary-rail").inert && !button("toggle-theme").disabled && !!root.querySelector(".page-load-progress"), "navigation remains usable while loading");
  button("toggle-global-sidebar").click(); await frames();
  expect(workspace.sidebarCollapsed() && pageRoot().inert, "collapse remains available during load");
  resolveLoad!(); await pending; await frames(); deferLoad = false;
  expect(!pageRoot().inert && !query("#modal-root").inert && !root.querySelector(".page-load-progress"), "settled navigation unlocks workspace");
  failLoad = true;
  field().value = "读取失败前的草稿";
  readPage(router.page); await pending!.catch(() => {}); await frames();
  expect(!!root.querySelector('.ui-blocking-message[role="alert"]') && !pageRoot().inert && !button("toggle-theme").disabled, "current read failure blocks workspace with recovery actions");
  button("toggle-theme").click(); await frames();
  failLoad = false;
  query<HTMLButtonElement>('[data-action="retry-shell-page"]').click(); await pending; await frames();
  expect(!controller.navigationFailed && !!field() && !root.querySelector(".ui-blocking-message"), "retry reloads current page read and clears failure");
  expect(field().value === "读取失败前的草稿", "loading/error redraw cannot erase a usable workspace snapshot");
  controller.showNavigationFailure(); paint(); query<HTMLButtonElement>('[data-action="shell-open-logs"]').click(); await pending; await frames();
  expect(router.page === "logs" && !controller.navigationFailed, "failure recovery opens registered logs page");
  const beforeDestroyed = activations;
  controller.destroy(); controller.destroy(); controller.start(); button("toggle-theme").click();
  expect(activations === beforeDestroyed, "destroy removes handlers and cannot restart");
  // Return the retained visual specimen to the requested accepted Shell state.
  workspace.setSidebarCollapsed(collapsed);
  if (session.theme !== theme) session.toggleTheme();
  const request = router.begin("dashboard"); router.complete(request); modal.reset(); painted = null; paint(); await frames();
  expect(getComputedStyle(query(".primary-rail")).backgroundColor === (theme === "dark" ? "rgb(17, 25, 35)" : "rgb(255, 255, 255)"), "resolved surface theme");
  expect(getComputedStyle(query(".secondary-sidebar")).backgroundColor === (theme === "dark" ? "rgb(11, 17, 24)" : "rgb(246, 247, 249)"), "resolved canvas theme");
  const icons = Array.from(root.querySelectorAll<HTMLElement>(".ui-icon-slot"));
  expect(icons.every(icon => getComputedStyle(icon).maskImage.startsWith('url("data:image/svg+xml;base64,')), "exact local vector assets resolved");
  const keyboardController = new ApplicationShellController(root, actions, operations);
  actionController = keyboardController;
  keyboardController.start();
  return { checks, finish() { keyboardController.destroy(); controller.destroy(); router.destroy(); workspace.stop(); session.stop(); void workspace.destroy(); void session.destroy(); } };
}
