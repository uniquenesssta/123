import {releaseFixture,releaseReport} from "./features/releaseFixtures.mjs";
import {analyticsFixture} from './features/analyticsFixtures.mjs';
import { loadTeamsProduction } from "./features/loadTeamsProduction.mjs";
import {readProjectSource} from "./styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../",import.meta.url));
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";
import { runInNewContext } from "node:vm";
import assert from "node:assert/strict";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readProjectSource(stylesheetRoot,resolve(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const navigation = read("src/app/router/routeRegistry.ts");
const shell = JSON.parse(read("contracts/global-visual-system-contract.json")).shell_source_entries.map(read).join("\n");
const viewState = read("src/app/workspace/workspaceVersion.ts");
const types = read("src/types.ts");
const layout = ["src/ui/layout/application-frame/applicationFrame.css","src/ui/navigation/sidebar/sidebar.css","src/ui/navigation/topbar/topbar.css","src/ui/layout/page-frame/pageFrame.css","src/ui/layout/content-region/contentRegion.css"].map(read).join("\n");
const entityStyles = read("src/styles/entityCenter.css") + read("src/ui/layout/application-frame/applicationFrame.css");
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
check(shell.includes("activeModule.items.map") && shell.includes('level === "primary" ? "true" : "page"'), "二级菜单未按当前一级模块渲染或缺少活动页语义");
check(shell.includes("secondary-reveal-button") && shell.includes("secondary-collapse-button"), "二级菜单缺少收起与恢复入口");
check(shell.includes('activeModule.label+" / "+navigationItemForPage(options.page).label'), "顶部面包屑未显示一级/二级层级");
check(!shell.includes("nav-management"), "旧管理折叠菜单仍残留在Shell");

check(layout.includes(".primary-rail") && layout.includes(".secondary-sidebar") && layout.includes(".secondary-nav-item"), "双层导航视觉样式不完整");
check(layout.includes("height: 100dvh") && layout.includes("--shell-navigation-offset"), "一级与二级菜单未固定或主内容偏移链缺失");
check(layout.includes(".application-shell.secondary-collapsed") && read("src/ui/primitives/icon-button/iconButton.css").includes("var(--fip-action-size)") && shell.includes("secondary-reveal-button"), "二级菜单折叠态样式不完整");
check(entityStyles.includes(".application-shell") && !entityStyles.includes(".sidebar .nav-item"), "资源中心仍覆盖旧全局侧栏结构");
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
const functions = source.statements.filter(node => ts.isFunctionDeclaration(node) && ["loadNavigationPage", "loadAnalysisCenter"].includes(node.name?.text)).map(node => node.getText(source)).join("\n");
assert.equal(source.statements.filter(node => ts.isFunctionDeclaration(node) && ["loadNavigationPage", "loadAnalysisCenter"].includes(node.name?.text)).length, 2);
const loaderJs = ts.transpileModule(functions, { compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None } }).outputText;
const { TeamsFeature } = loadTeamsProduction(root)("src/features/teams/index.ts");
const { PlayersFeature }=loadTeamsProduction(root)("src/features/players/index.ts");
const { PredictionFeature }=loadTeamsProduction(root)("src/features/prediction/predictionFeature.ts");
const {ResearchFeature}=loadTeamsProduction(root)("src/features/prediction/research/researchFeature.ts");
const {ReviewFeature}=loadTeamsProduction(root)("src/features/review/reviewFeature.ts");
const {PostmatchFeature}=loadTeamsProduction(root)("src/features/postmatch/index.ts");
function loaderContext(retainedTeam = null) {
  const applied = [], router = new Router();
  const context = {
    navigation: router, browserLifecycleController: { signal: { aborted: false } }, session: { bootstrap: { data: { database_configured: true } } },
    issueLogs: "before", releaseAcceptanceRuns: "before", selectedReleaseAcceptanceRun: null, openAiProfiles: "before", creatingOpenAiProfile: false, selectedOpenAiProfileId: null,
    playerQuery: {}, selectedPlayer: null, playerListPage: null, teamQuery: {}, selectedTeam: null, teamListPage: null, playerReferences: null,
    selectedReviewMatchId: null, apiWorkspaceDetail: null, apiWorkspacePresets: "before", selectedApiWorkspaceProfileId: "", selectedApiWorkspacePresetKey: "", selectedApiWorkspaceMatchId: null,
    workspaceState: { module: () => ({}), patchModule: () => applied.push("patch") },
    api: {},
  };
  for (const name of ["applyPlayerCatalog", "applyTeamCatalog", "applyLineups", "applyReviewCenter"]) context[name] = () => applied.push(name);
  context.teamsFeature = new TeamsFeature({ api: context.api, initialQuery: null, snapshot: () => context.session.bootstrap, page: () => router.page, activeTab: () => retainedTeam, publishOptions: () => applied.push("options"), persistQuery: () => applied.push("teamQuery"), setPresetTab: () => applied.push("presetTab") });
  context.playersFeature=new PlayersFeature({api:context.api,initialQuery:null,initialSource:null,snapshot:()=>context.session.bootstrap,page:()=>router.page,persistQuery:()=>applied.push("playerQuery"),persistSource:()=>{},root:()=>({}),busy:fn=>fn()});
  context.predictionFeature=new PredictionFeature({api:context.api,recovery:{read:()=>null,save:()=>{},clear:()=>{}},scope:()=>()=>true,view:()=>()=>true,changed:()=>{},busy:fn=>fn()});
  context.researchFeature=new ResearchFeature({api:context.api,context:context.predictionFeature.context,recovery:{read:()=>null,save:()=>{},clear:()=>{}},scope:()=>()=>true,view:()=>{const page=router.page;return()=>router.page===page;},changed:()=>{}});
  context.api.postmatchOverview=async()=>({settlement_count:0,pending_evidence_count:0,scored_evidence_count:0,settlements:[],evidence_queue:[],provider_scores:[],drift_runs:[]});
  context.postmatchFeature=new PostmatchFeature({api:context.api,recovery:{capture:()=>({read:()=>null,save:()=>{},clear:()=>{}})},scope:()=>()=>true,view:()=>{const page=router.page;return()=>router.page===page;},changed:()=>{},formatError:String});
  const releaseModule=loadTeamsProduction(root)("src/features/release/index.ts"),release=releaseFixture(releaseModule.ReleaseFeature,releaseModule.releaseStorage);Object.assign(context.api,release.api);context.releaseFeature=new releaseModule.ReleaseFeature({...release.ports,api:context.api,view:()=>{const page=router.page;return()=>router.page===page;}});
  const analyticsModule=loadTeamsProduction(root)("src/features/analytics/index.ts"),analytics=analyticsFixture(analyticsModule.AnalyticsFeature,analyticsModule.analyticsStorage);
  analytics.ports.view=()=>{const page=router.page;return()=>router.page===page;};Object.assign(context.api,analytics.api);context.analyticsFeature=new analyticsModule.AnalyticsFeature({...analytics.ports,api:context.api});
  context.reviewFeature=new ReviewFeature({postmatch:context.postmatchFeature.settlement,api:context.api,scope:()=>()=>true,view:()=>{const page=router.page;return()=>router.page===page;},changed:()=>{},recovery:{read:()=>null,save:()=>{},clear:()=>{}},snapshot:()=>context.session.bootstrap,formatError:String});
  runInNewContext(loaderJs, context);
  return { context, router, applied };
}
for (const [page, operation, apply] of [

  ["lineups", "fetchLineups", "applyLineups"],

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
await test("actual logs loader rejects stale history without a business database",async()=>{for(const stale of [true,false]){const {context,router}=loaderContext(),task=deferred(),{LogsFeature}=loadTeamsProduction(root)("src/features/logs/LogsFeature.ts");context.api.listIssueLogs=()=>task.promise;context.logsFeature=new LogsFeature({api:context.api,storage:{read:()=>({section:"overview",expanded:[],unknownClear:false,operationId:null}),save:()=>{}},view:()=>()=>true,changed:()=>{},busy:run=>run()});context.session.bootstrap.data.database_configured=false;const pending=context.loadNavigationPage(router.begin("logs"));if(stale)router.begin("dashboard");task.resolve([]);await pending;assert.equal(context.logsFeature.state.entries===null,stale);router.destroy();}});
await test("actual profile loader rejects stale directory and works without business database",async()=>{for(const stale of [true,false]){const {context,router}=loaderContext(),task=deferred(),{ProfilesFeature}=loadTeamsProduction(root)("src/features/openai-profiles/ProfilesFeature.ts");context.api.listOpenAiProfiles=()=>task.promise;context.profilesFeature=new ProfilesFeature({api:context.api,storage:{read:()=>null,save:()=>{}},view:()=>{const p=router.page;return()=>p===router.page;},changed:()=>{},formatError:String});context.workspaceState.module=()=>({controls:{}});context.session.bootstrap.data.database_configured=false;const pending=context.loadNavigationPage(router.begin("openai"));if(stale)router.begin("dashboard");task.resolve({active_profile_id:null,profiles:[],config_path:"fixture"});await pending;assert.equal(context.profilesFeature.state.catalog===null,stale);router.destroy();}});
await test("actual analytics loader rejects stale data and keeps current source identity",async()=>{for(const stale of [true,false]){const {context,router}=loaderContext(),task=deferred();context.api.analyticsOverview=()=>task.promise;const pending=context.loadNavigationPage(router.begin("analytics"));if(stale)router.begin("dashboard");task.resolve(analyticsFixture(loadTeamsProduction(root)("src/features/analytics/index.ts").AnalyticsFeature,loadTeamsProduction(root)("src/features/analytics/index.ts").analyticsStorage).overview());await pending;assert.equal(context.analyticsFeature.overview!==null,!stale);router.destroy();}});
await test("actual review module loading rejects stale data and publishes current identity",async()=>{for(const stale of [true,false]){const {context,router}=loaderContext(),task=deferred();context.api.listReviewableMatches=()=>task.promise;context.api.listMatchReviews=async()=>[];const pending=context.loadNavigationPage(router.begin("review"));if(stale)router.begin("dashboard");task.resolve([{match_record:{id:"original-match"}}]);await pending;assert.equal(context.reviewFeature.state.matches.length,stale?0:1);assert.equal(context.reviewFeature.state.selectedMatchId,null);router.destroy();}});
await test("actual players loader rejects stale data and publishes only current identity",async()=>{for(const stale of [true,false]){const {context,router}=loaderContext(),task=deferred();context.api.playerCatalogReferenceData=async()=>({teams:[]});context.api.listPlayers=()=>task.promise;const pending=context.loadNavigationPage(router.begin("players"));if(stale)router.begin("dashboard");task.resolve({items:[],has_more:false});await pending;assert.equal(context.playersFeature.catalog.list!==null,!stale);router.destroy();}});
await test("actual release loader and explicit history detail guard each await",async()=>{
 for(const stage of ["list","detail","current"]){const {context,router}=loaderContext(),list=deferred(),detail=deferred();let reads=0;const row=releaseReport("qa-nav");context.api.listReleaseAcceptanceRuns=()=>list.promise;context.api.readReleaseAcceptanceRun=()=>{reads++;return detail.promise;};const loading=context.loadNavigationPage(router.begin("release"));if(stage==="list")router.begin("dashboard");list.resolve([row]);await loading;if(stage==="list"){assert.equal(context.releaseFeature.state.loaded,false);assert.equal(reads,0);}else{const selecting=context.releaseFeature.select(row.id);if(stage==="detail")router.begin("dashboard");detail.resolve(row);await selecting;assert.equal(reads,1);assert.equal(context.releaseFeature.state.selected?.id??null,stage==="current"?row.id:null);}router.destroy();}
});
await test("actual AI workspace loader guards profiles catalog selection and each await",async()=>{
 for(const stage of ["profiles","sessions","detail","current"]){const {context,router}=loaderContext(),profiles=deferred(),sessions=deferred(),detail=deferred(),started=deferred(),reading=deferred();const {AiWorkspaceFeature}=loadTeamsProduction(root)("src/features/ai-workspace/aiWorkspaceFeature.ts");Object.assign(context.api,{listOpenAiProfiles:()=>profiles.promise,listApiWorkspacePresets:async()=>[{key:"plain_chat"}],listApiWorkspaceSessions:()=>{started.resolve();return sessions.promise;},playerCatalogReferenceData:async()=>({managed_matches:[]}),readApiWorkspaceSession:()=>{reading.resolve();return detail.promise;}});context.aiWorkspaceFeature=new AiWorkspaceFeature({api:context.api,scope:()=>()=>true,view:()=>{const page=router.page;return()=>page===router.page;},storage:{capture:()=>({read:()=>({selected:"session:A",drafts:{},receipt:null,archives:{}}),save:()=>{}})},changed:()=>{},formatError:String});context.workspaceState.module=()=>({controls:{}});const task=context.loadNavigationPage(router.begin("api_workspace"));if(stage==="profiles")router.begin("dashboard");profiles.resolve({profiles:[],active_profile_id:null});if(stage!=="profiles")await started.promise;if(stage==="sessions")router.begin("dashboard");sessions.resolve([{id:"A"}]);if(stage==="detail"||stage==="current")await reading.promise;if(stage==="detail")router.begin("dashboard");detail.resolve({session:{id:"A",metadata:{},profile_id:"p",preset_key:"plain_chat",match_id:null},messages:[],operations:[],files:[]});await task;assert.equal(context.aiWorkspaceFeature.state.detail?.session.id??null,stage==="current"?"A":null);if(stage==="profiles")assert.equal(context.aiWorkspaceFeature.state.profiles,null);router.destroy();}
});
await test("actual teams and prediction loaders cannot apply stale multi-step results", async () => {
  for (const page of ["teams", "lineup_presets", "prediction"]) {
    const { context, router, applied } = loaderContext(page === "prediction" ? null : "team"), final = deferred(), finalStarted = deferred();
    context.api.listTeams = async () => ({ items: [{ id: "team" }] }); context.api.readTeam = async () => ({ team: { id: "team" } });
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
check(main.includes("targetPage === navigation.page") && main.includes("if (isRoute(targetPage)) activatePage(targetPage)"), "通用入口未保护无效/已选中页面");

if (failures.length) {
  console.error("双层全局导航专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log(`双层全局导航专项验证通过：7个一级模块、${configuredPages.length}个二级页面入口；${behaviorCases}项真实路由/历史/页面加载行为测试。`);
