import {readProjectSource} from "./styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../",import.meta.url));
import { renderResourceMarkup } from "./patterns/renderResourceMarkup.mjs";
import {fileURLToPath} from "node:url";
import fs from "node:fs";

function read(path) {
  return readProjectSource(stylesheetRoot,new URL(`../${path}`, import.meta.url), "utf8");
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
const catalog=read("src/features/players/playerCatalog.ts"),feature=read("src/features/players/playersFeature.ts"),dom=read("src/features/players/playerDirectoryDom.ts");
const teams = ["src/features/teams/teamPresentation.ts","src/features/teams/profileView.ts","src/features/teams/packageView.ts","src/features/teams/teamsView.ts"].map(read).join("\n");
const css = read("src/styles/entityCenter.css");

requireTrue(catalog.includes("cachedReferences?: boolean") && catalog.includes("options.cachedReferences && this.referencesValue"), "翻页未复用参考目录");
for(const name of ["next","previous"]){const start=catalog.indexOf("  "+name+"("),end=catalog.indexOf(name==="next"?"\n  previous(":"\n  async refreshDetail(",start+3),method=catalog.slice(start,end<0?catalog.length:end);requireTrue(method.includes("this.loading"),name+"缺少并发门禁");}
requireTrue(catalog.includes("cachedReferences: true") && catalog.includes("this.history.slice(0, -1)"),"分页未使用原缓存/游标历史");
const paginate=feature.slice(feature.indexOf("  async paginate("),feature.indexOf("  changeImportMode("));
requireTrue(paginate.includes("this.catalog[direction]") && !paginate.includes("this.ports.busy"),"分页仍触发全屏忙碌或未接真实catalog");
requireTrue(paginate.includes("refreshPlayerPageDom") && paginate.includes("setPlayerPageLoading") && main.includes("render({preserveForm:true})"),"未保留局部更新/表单回退");
requireTrue(dom.includes("playerTableRows(") && dom.includes("tbody.innerHTML") && dom.includes("aria-busy"),"真实分页DOM更新与局部加载状态失联");
requireTrue(css.includes('.player-browser .entity-main[aria-busy="true"]'), "球员分页缺少局部加载样式");

requireTrue(teams.includes("teamDetailFilterPanel"), "球队详情未恢复独立筛选器");
requireTrue(teams.includes("search-teams-from-detail") && teams.includes("clear-team-filters-from-detail"), "球队详情筛选器未接入目录返回链");
const rendered=renderResourceMarkup(fileURLToPath(new URL("..",import.meta.url)),"teams","directory",null,true);
const filterIndex=rendered.indexOf('class="entity-filter-panel panel team-detail-filter'),listIndex=rendered.indexOf('<main class="entity-main panel detail-pane"'),inspectorIndex=rendered.indexOf('<aside class="entity-inspector panel inspector-pane"');
requireTrue(filterIndex>=0 && listIndex>filterIndex && inspectorIndex>listIndex, "球队详情筛选器未放在主名单左侧");
requireTrue(main.includes('case "search-teams-from-detail"') && main.includes("teamsFeature.search(true)"), "球队详情筛选未在应用后返回结果目录");
requireTrue(main.includes('case "clear-team-filters-from-detail"') && main.includes("teamsFeature.clearFilters(true)"), "球队详情清除筛选未返回完整目录");
requireTrue(css.includes('grid-template-columns: minmax(220px, 248px) minmax(0, 1fr)'), "球队详情筛选器和主名单未形成稳定双列布局");
requireTrue(css.includes("grid-column: 2"), "关闭速览后球队主名单未保留筛选器列");

console.log("球员无闪烁分页与球队详情筛选器专项验证通过。");
