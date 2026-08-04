import fs from "node:fs";

function read(path) {
  return fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
}
function requireTrue(value, message) {
  if (!value) throw new Error(message);
}
function body(source, name) {
  const start = source.indexOf(`async function ${name}`);
  requireTrue(start >= 0, `未找到函数：${name}`);
  const next = source.indexOf("\nasync function ", start + 1);
  return source.slice(start, next < 0 ? source.length : next);
}

const main = read("src/main.ts");
const loaders = read("src/controllers/pageLoaders.ts");
const teams = read("src/pages/teams.ts");
const css = read("src/styles/entityCenter.css");

requireTrue(loaders.includes("cachedReferences: PlayerCatalogReferenceData | null = null"), "球员目录加载器未支持复用参考数据");
requireTrue(loaders.includes("cachedReferences") && loaders.includes("Promise.resolve(cachedReferences)"), "翻页仍会重复加载球员参考目录");

const nextPage = body(main, "nextPlayerPage");
const previousPage = body(main, "previousPlayerPage");
for (const [label, source] of [["下一页", nextPage], ["上一页", previousPage]]) {
  requireTrue(source.includes("loadPlayerPage"), `${label}未使用无闪烁分页加载链`);
  requireTrue(!source.includes("runBusy"), `${label}仍触发全屏忙碌遮罩`);
  requireTrue(source.includes("render({ preserveForm: true })"), `${label}未保留当前筛选与滚动上下文`);
}
requireTrue(main.includes("playerPageLoading") && main.includes("setPlayerPageLoading"), "球员分页缺少并发保护和局部加载状态");
requireTrue(main.includes("refreshPlayerPageDom") && main.includes("playerTableRows"), "球员分页仍依赖整页重绘，未切换为表格局部更新");
requireTrue(css.includes('.player-browser .entity-main[aria-busy="true"]'), "球员分页缺少局部加载样式");

requireTrue(teams.includes("teamDetailFilterPanel"), "球队详情未恢复独立筛选器");
requireTrue(teams.includes("search-teams-from-detail") && teams.includes("clear-team-filters-from-detail"), "球队详情筛选器未接入目录返回链");
requireTrue(teams.includes("${teamDetailFilterPanel}${teamDetailPanel}${teamInspectorPanel}"), "球队详情筛选器未放在主名单左侧");
requireTrue(main.includes('case "search-teams-from-detail"') && main.includes("searchTeams(true)"), "球队详情筛选未在应用后返回结果目录");
requireTrue(main.includes('case "clear-team-filters-from-detail"') && main.includes("clearTeamFilters(true)"), "球队详情清除筛选未返回完整目录");
requireTrue(css.includes('grid-template-columns: minmax(220px, 248px) minmax(0, 1fr)'), "球队详情筛选器和主名单未形成稳定双列布局");
requireTrue(css.includes("grid-column: 2"), "关闭速览后球队主名单未保留筛选器列");

console.log("球员无闪烁分页与球队详情筛选器专项验证通过。");
