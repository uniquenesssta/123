import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { runInNewContext } from "node:vm";
import assert from "node:assert/strict";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const navigation = read("src/app/router/routeRegistry.ts");
const shell = read("src/app/shell.ts");
const viewState = read("src/app/viewState.ts");
const types = read("src/types.ts");
const layout = read("src/styles/layout.css");
const entityStyles = read("src/styles/entityCenter.css");
const contract = JSON.parse(read("contracts/workspace-ui-contract.json"));

const pageUnion = types.match(/export type Page\s*=([\s\S]*?);/)?.[1] ?? "";
const declaredPages = [...pageUnion.matchAll(/\|\s*"([^"]+)"/g)].map((match) => match[1]);
const configuredPages = [...navigation.matchAll(/\{ page: "([^"]+)", label:/g)].map((match) => match[1]);
const duplicatePages = configuredPages.filter((page, index) => configuredPages.indexOf(page) !== index);
const missingPages = declaredPages.filter((page) => !configuredPages.includes(page));
const unknownPages = configuredPages.filter((page) => !declaredPages.includes(page));

check(contract.contract_version === "1.3.0" && contract.ui_revision === 5, "双层导航契约版本或UI修订号错误");
check(navigation.includes("PRIMARY_NAVIGATION") && navigation.includes("navigationModuleForPage") && navigation.includes("navigationItemForPage"), "缺少集中式双层导航配置或页面映射");
check((navigation.match(/key: "/g) ?? []).length === 7, "一级菜单数量必须保持为7个清晰业务模块");
for (const label of ["首页", "比赛", "资源", "模型", "分析", "AI", "管理"]) {
  check(navigation.includes(`label: "${label}"`), `缺少一级菜单：${label}`);
}
check(duplicatePages.length === 0, `页面被重复分配到二级菜单：${duplicatePages.join(", ")}`);
check(missingPages.length === 0, `页面未分配到二级菜单：${missingPages.join(", ")}`);
check(unknownPages.length === 0, `二级菜单包含未知页面：${unknownPages.join(", ")}`);

check(shell.includes('class="primary-rail"') && shell.includes('class="secondary-sidebar"'), "Shell未渲染固定一级栏和上下文二级栏");
check(shell.includes('aria-label="一级菜单"') && shell.includes('data-current-module='), "Shell缺少一级菜单语义或当前模块标识");
check(shell.includes("activeModule.items.map") && shell.includes('aria-current="page"'), "二级菜单未按当前一级模块渲染或缺少活动页语义");
check(shell.includes("secondary-reveal-button") && shell.includes("secondary-collapse-button"), "二级菜单缺少收起与恢复入口");
check(shell.includes("${activeModule.label} / ${activeTitle}"), "顶部面包屑未显示一级/二级层级");
check(!shell.includes("nav-management"), "旧管理折叠菜单仍残留在Shell");

check(layout.includes(".primary-rail") && layout.includes(".secondary-sidebar") && layout.includes(".secondary-nav-item"), "双层导航视觉样式不完整");
check(layout.includes("position: fixed") && layout.includes("--shell-navigation-offset"), "一级与二级菜单未固定或主内容偏移链缺失");
check(layout.includes(".dual-navigation.secondary-collapsed") && layout.includes(".secondary-reveal-button"), "二级菜单折叠态样式不完整");
check(entityStyles.includes(".app-shell.dual-navigation") && !entityStyles.includes(".sidebar .nav-item"), "资源中心仍覆盖旧全局侧栏结构");
check(viewState.includes("ui_revision: 5") && viewState.includes("sidebar_collapsed") && viewState.includes("needsUiMigration ? false"), "工作区状态未迁移到双层导航修订版或首次升级未展开二级菜单");

// R12-03 uses this existing entry to exercise actual production modules and page loaders.
const ts = createRequire(import.meta.url)("typescript");
const cache = new Map();
function load(file) {
  file = resolve(root, file);
  if (cache.has(file)) return cache.get(file);
  const exports = {};
  cache.set(file, exports);
  const output = ts.transpileModule(read(file), { compilerOptions: { module: ts.ModuleKind.CommonJS, target: ts.ScriptTarget.ES2022 } }).outputText;
  const require = specifier => {
    assert.ok(specifier.startsWith("."), `Router dependency must stay local: ${specifier}`);
    const target = resolve(root, dirname(file), specifier);
    return load(existsSync(`${target}.ts`) ? `${target}.ts` : `${target}/index.ts`);
  };
  new Function("exports", "require", output)(exports, require);
  return exports;
}
const { Router, RouteHistory, NavigationContext, PRIMARY_NAVIGATION, isRoute, navigationItemForPage, navigationModuleForPage } = load("src/app/router/index.ts");
const deferred = () => { let resolve, reject; const promise = new Promise((a, b) => { resolve = a; reject = b; }); return { promise, resolve, reject }; };
let behaviorCases = 0;
async function test(name, run) {
  try { await run(); behaviorCases++; }
  catch (error) { failures.push(`R12-03 ${name}: ${error.stack ?? error}`); }
}
await test("registered routes are complete, immutable, defaults stay within their module", () => {
  assert.equal(PRIMARY_NAVIGATION.length, 7);
  assert.deepEqual(PRIMARY_NAVIGATION.flatMap(module => module.items.map(item => item.page)).sort(), declaredPages.sort());
  for (const module of PRIMARY_NAVIGATION) {
    assert.equal(navigationModuleForPage(module.default_page), module);
    assert.ok(Object.isFrozen(module) && Object.isFrozen(module.items));
    for (const item of module.items) { assert.equal(navigationItemForPage(item.page), item); assert.equal(isRoute(item.page), true); }
  }
  assert.equal(isRoute("missing"), false);
  assert.throws(() => navigationItemForPage("missing"), /未配置二级菜单/);
});
await test("invalid route does not replace current route or pending lease", () => {
  const router = new Router(), pending = router.begin("teams");
  assert.throws(() => router.begin("missing"));
  assert.equal(router.page, "teams"); assert.equal(router.pendingPage, "teams"); assert.equal(router.isCurrent(pending), true);
  assert.throws(() => new Router("missing")); router.destroy();
});
await test("only latest request commits or settles", () => {
  const router = new Router(), a = router.begin("teams"), b = router.begin("players");
  assert.equal(a.signal.aborted, true); assert.equal(a.commit(() => assert.fail("stale")), false);
  assert.equal(router.complete(a), false); assert.equal(router.pendingPage, "players");
  let value; assert.equal(b.commit(() => { value = 7; }), true); assert.equal(value, 7);
  assert.equal(router.complete(b), true); assert.equal(router.pendingPage, null); assert.equal(router.complete(b), false);
  assert.equal(b.commit(() => assert.fail("finished")), false); router.destroy();
});
await test("rapid navigation suppresses old completion and keeps new pending UI", async () => {
  const router = new Router(), a = deferred(), b = deferred(), applied = [], effects = [];
  const start = (page, task) => router.navigate(page, async request => { const value = await task.promise; request.commit(() => applied.push(value)); }, { started: () => effects.push(`start:${page}`), settled: () => effects.push(`end:${page}`) });
  const first = start("teams", a), second = start("players", b);
  a.resolve("old"); assert.equal(await first, false); assert.equal(router.pendingPage, "players");
  b.resolve("new"); assert.equal(await second, true); assert.deepEqual(applied, ["new"]);
  assert.deepEqual(effects, ["start:teams", "start:players", "end:players"]); router.destroy();
});
await test("stale error is ignored, current error surfaces and loading ends", async () => {
  const router = new Router(), old = deferred(); let settled = 0;
  const first = router.navigate("teams", () => old.promise, { started() {}, settled() { assert.fail("stale settle"); } });
  const error = new Error("current");
  await assert.rejects(router.navigate("players", async () => { throw error; }, { started() {}, settled() { settled++; } }), value => value === error);
  old.reject(new Error("stale")); assert.equal(await first, false); assert.equal(settled, 1); assert.equal(router.pendingPage, null); router.destroy();
});
await test("synchronous start failure still releases loading", async () => {
  const router = new Router(); let settled = 0;
  await assert.rejects(router.navigate("logs", async () => assert.fail("load"), { started() { throw new Error("render"); }, settled() { settled++; } }), /render/);
  assert.equal(settled, 1); assert.equal(router.pendingPage, null); router.destroy();
});
await test("destruction invalidates pending result and prohibits restart", async () => {
  const router = new Router(), task = deferred(); let request;
  const pending = router.navigate("teams", async value => { request = value; await task.promise; value.commit(() => assert.fail("destroyed")); }, { started() {}, settled() { assert.fail("destroyed render"); } });
  router.destroy(); router.destroy(); task.resolve(); assert.equal(await pending, false);
  assert.equal(request.signal.aborted, true); assert.equal(router.pendingPage, null); assert.throws(() => router.begin("players"), /生命周期/);
});
await test("abort listener navigation cannot be overwritten by outer begin", () => {
  const router = new Router(), first = router.begin("teams"); let inner;
  first.signal.addEventListener("abort", () => { inner = router.begin("logs"); });
  const outer = router.begin("players");
  assert.equal(router.isCurrent(outer), false); assert.equal(router.isCurrent(inner), true); assert.equal(router.page, "logs"); assert.equal(router.pendingPage, "logs"); router.destroy();
});
await test("same-page refresh supersedes prior lease, cancel permits later navigation", () => {
  const router = new Router(), first = router.begin("teams"), next = router.begin("teams");
  assert.equal(first.signal.aborted, true); router.cancel(); assert.equal(router.isCurrent(next), false);
  assert.equal(router.pendingPage, null); assert.equal(router.page, "teams"); assert.equal(router.isCurrent(router.begin("logs")), true); router.destroy();
});
await test("history base, boundaries, close and forward preserve entry ownership", () => {
  const history = new RouteHistory(), a = { draft: "A" }, b = { draft: "B" };
  history.back(); history.forward(); assert.equal(history.current, null); assert.equal(history.canBack, false); assert.equal(history.canForward, false);
  history.push(a); history.push(b); history.back(); assert.equal(history.current, a);
  history.back(); assert.equal(history.current, null); assert.equal(history.canForward, true); history.back(); history.forward(); assert.equal(history.current, a);
  history.close(); history.forward(); assert.equal(history.current, a); history.forward(); assert.equal(history.current, b); history.forward(); assert.equal(history.current, b);
  history.reset(); assert.equal(history.current, null); assert.equal(history.canForward, false);
});
await test("history branch truncates abandoned forward entries", () => {
  const history = new RouteHistory(); history.push("A"); history.push("B"); history.back(); history.push("C");
  assert.equal(history.canForward, false); history.back(); assert.equal(history.current, "A"); history.forward(); assert.equal(history.current, "C");
  history.close(); history.push("D"); history.back(); history.forward(); assert.equal(history.current, "D"); assert.equal(history.canForward, false);
});
await test("return context contains only location and is consumed once", () => {
  const context = new NavigationContext(), input = { page: "teams", section: "profile", matchId: "feature-owned" };
  context.set(input); input.section = "changed";
  assert.deepEqual(context.target, { page: "teams", section: "profile" }); assert.ok(Object.isFrozen(context.target));
  assert.equal(context.take().page, "teams"); assert.equal(context.take(), null); context.set({ page: "players", section: null }); context.clear(); assert.equal(context.target, null);
});

// Compile only the named real feature loading functions; inject I/O, never a copied loader.
const main = read("src/main.ts");
const source = ts.createSourceFile("main.ts", main, ts.ScriptTarget.Latest, true);
const functions = source.statements.filter(node => ts.isFunctionDeclaration(node) && ["loadNavigationPage", "loadApiWorkspace"].includes(node.name?.text)).map(node => node.getText(source)).join("\n");
assert.equal(source.statements.filter(node => ts.isFunctionDeclaration(node) && ["loadNavigationPage", "loadApiWorkspace"].includes(node.name?.text)).length, 2);
const loaderJs = ts.transpileModule(functions, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None } }).outputText;
function loaderContext() {
  const applied = [], router = new Router();
  const context = {
    navigation: router, browserLifecycleController: { signal: { aborted: false } }, state: { data: { database_configured: true } },
    issueLogs: "before", releaseAcceptanceRuns: "before", selectedReleaseAcceptanceRun: null, openAiProfiles: "before", creatingOpenAiProfile: false, selectedOpenAiProfileId: null,
    playerQuery: {}, selectedPlayer: null, playerListPage: null, teamQuery: {}, selectedTeam: null, teamListPage: null, playerReferences: null, selectedP4MatchId: null, p4TaskWorkspace: null,
    selectedReviewMatchId: null, apiWorkspaceDetail: null, apiWorkspacePresets: "before", selectedApiWorkspaceProfileId: "", selectedApiWorkspacePresetKey: "", selectedApiWorkspaceMatchId: null,
    workspaceState: { module: () => ({}), patchModule: () => applied.push("patch") },
    api: {},
  };
  for (const name of ["applyPlayerCatalog", "applyTeamCatalog", "applyLineups", "applyReviewCenter", "applyAnalysisCenter"]) context[name] = () => applied.push(name);
  runInNewContext(loaderJs, context);
  return { context, router, applied };
}
for (const [page, operation, apply] of [
  ["logs", "listIssueLogs", "issueLogs"], ["openai", "listOpenAiProfiles", "openAiProfiles"],
  ["players", "fetchPlayerCatalog", "applyPlayerCatalog"], ["lineups", "fetchLineups", "applyLineups"],
  ["review", "fetchReviewCenter", "applyReviewCenter"], ["analytics", "fetchAnalysisCenter", "applyAnalysisCenter"],
]) {
  await test(`actual ${page} loader rejects stale data and accepts current data`, async () => {
    for (const stale of [true, false]) {
      const { context, router, applied } = loaderContext(), task = deferred();
      (operation.startsWith("fetch") ? context : context.api)[operation] = () => task.promise;
      const request = router.begin(page), pending = context.loadNavigationPage(request);
      if (stale) router.begin("dashboard"); task.resolve(page === "openai" ? { profiles: [] } : ["loaded"]); await pending;
      if (apply.startsWith("apply")) assert.equal(applied.includes(apply), !stale);
      else assert.equal(context[apply] === "before", stale);
      router.destroy();
    }
  });
}
await test("actual release loader is atomic across both awaits", async () => {
  for (const stage of ["list", "detail", "current"]) {
    const { context, router } = loaderContext(), list = deferred(), detail = deferred(), detailStarted = deferred(); let reads = 0;
    context.api.listReleaseAcceptanceRuns = () => list.promise; context.api.readReleaseAcceptanceRun = () => { reads++; detailStarted.resolve(); return detail.promise; };
    const pending = context.loadNavigationPage(router.begin("release")); if (stage === "list") router.begin("dashboard");
    list.resolve([{ id: "run" }]); if (stage !== "list") await detailStarted.promise;
    if (stage === "detail") router.begin("dashboard"); detail.resolve({ id: "run" }); await pending;
    assert.equal(reads, stage === "list" ? 0 : 1); assert.equal(context.releaseAcceptanceRuns === "before", stage !== "current"); router.destroy();
  }
});
await test("actual AI workspace loader guards each await including session detail", async () => {
  for (const stage of ["profiles", "sessions", "detail", "current"]) {
    const { context, router } = loaderContext(), profiles = deferred(), sessions = deferred(), detail = deferred(), sessionsStarted = deferred(), detailStarted = deferred();
    context.apiWorkspaceDetail = { session: { id: "session" } };
    context.api.listOpenAiProfiles = () => profiles.promise; context.api.listApiWorkspacePresets = async () => [{ key: "plain_chat" }];
    context.api.listApiWorkspaceSessions = () => { sessionsStarted.resolve(); return sessions.promise; }; context.api.playerCatalogReferenceData = async () => ({ managed_matches: [] });
    context.api.readApiWorkspaceSession = () => { detailStarted.resolve(); return detail.promise; };
    const pending = context.loadNavigationPage(router.begin("api_workspace"));
    if (stage === "profiles") router.begin("dashboard"); profiles.resolve({ profiles: [] });
    if (stage !== "profiles") await sessionsStarted.promise;
    if (stage === "sessions") router.begin("dashboard"); sessions.resolve([{ id: "session" }]);
    if (stage === "detail" || stage === "current") await detailStarted.promise;
    if (stage === "detail") router.begin("dashboard"); detail.resolve({ session: { id: "new", metadata: {} } }); await pending;
    assert.equal(context.apiWorkspaceDetail.session.id, stage === "current" ? "new" : "session");
    assert.equal(context.apiWorkspaceSessions === undefined, stage !== "current");
    if (stage === "profiles") assert.equal(context.openAiProfiles, "before"); router.destroy();
  }
});
await test("actual teams and prediction loaders cannot apply stale multi-step results", async () => {
  for (const page of ["teams", "lineup_presets", "prediction"]) {
    const { context, router, applied } = loaderContext(), final = deferred(), finalStarted = deferred();
    context.fetchTeamCatalog = async () => ({ list: { items: [] }, selected: { team: { id: "team" } } });
    context.api.listCoaches = async () => []; context.api.listFormations = async () => [];
    context.api.listTeamMatchLineups = () => { finalStarted.resolve(); return final.promise; }; context.api.listTeamLineupPresets = async () => [];
    context.api.playerCatalogReferenceData = async () => ({ managed_matches: [{ id: "match" }], upcoming_matches: [] });
    context.api.readP4MatchWorkspace = () => { finalStarted.resolve(); return final.promise; };
    const pending = context.loadNavigationPage(router.begin(page));
    await finalStarted.promise; router.begin("dashboard"); final.resolve(page === "prediction" ? { tasks: [] } : []); await pending;
    assert.deepEqual(applied, []); assert.equal(context.playerReferences, null); router.destroy();
  }
});
check(!existsSync(join(root, "src/app/navigation.ts")), "旧navigation.ts实现未移除");
check(!/\blet page\s*:|navigationPendingPage|NavigationCoordinator/.test(main), "页面或pending状态仍存在第二所有者");
for (const name of ["route", "routeRegistry", "router", "routeTransition", "navigationRequest", "navigationContext", "routeHistory", "index"]) {
  const code = read(`src/app/router/${name}.ts`);
  check(!/from ["'][^"']*(?:pages|api\/client|viewState|modal)["']/.test(code), `Router不得反向依赖业务或UI：${name}`);
}
check(main.includes("navigation.destroy()") && main.includes("workflowNavigation.clear()"), "停止时未释放路由及返回位置");
check(main.includes("targetPage === navigation.page") && main.includes("!isRoute(targetPage)"), "通用入口未保护无效/已选中页面");

if (failures.length) {
  console.error("双层全局导航专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log(`双层全局导航专项验证通过：7个一级模块、${configuredPages.length}个二级页面入口；${behaviorCases}项真实路由/历史/页面加载行为测试。`);
