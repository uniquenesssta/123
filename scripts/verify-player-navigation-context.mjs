import {readProjectSource} from "./styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../",import.meta.url));
import fs from "node:fs";
const read = (path) => readProjectSource(stylesheetRoot,new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const check = (ok, message) => { if (!ok) failures.push(message); };
const main = read("src/main.ts");
const catalog = read("src/features/players/playerCatalog.ts"), feature=read("src/features/players/playersFeature.ts"), compact=s=>s.replace(/\s/g,"");
const page = ["src/features/players/playerPresentation.ts","src/features/players/playerProfileView.ts","src/features/players/playerWorkbookView.ts","src/features/players/playerDirectoryView.ts"].map(read).join("\n");
const teams = read("src/features/teams/teamCatalog.ts");
const types = read("src/types.ts");
const styles = read("src/styles/entityCenter.css");
const between = (source, start, end) => source.slice(source.indexOf(start), source.indexOf(end, source.indexOf(start)));

check(types.includes("interface PlayerNavigationContext"), "缺少球队到球员的上下文类型");
for (const field of ['origin_page: "teams" | "lineups"', 'source: "team_roster" | "match_lineup"', "created_at: string", "updated_at: string"]) {
  check(types.includes(field), `来源上下文缺少 ${field}`);
}
check(!types.includes("locked: boolean"), "来源球队仍被建模为真实锁定状态");
for (const key of ["TEAM_QUERY_KEY", "PLAYER_QUERY_KEY", "PLAYER_NAV_CONTEXT_KEY"]) {
  check(main.includes(key), `缺少持久化键 ${key}`);
}
check(compact(main).includes("initialQuery:localPreferences.readObject<PlayerListQuery>(PLAYER_QUERY_KEY)") && catalog.includes("normalizePlayerQuery(query)"), "球员筛选未从持久状态恢复");
check(catalog.includes("if (this.sourceValue) this.queryValue.team_id = this.sourceValue.team_id"), "应用重启没有恢复来源球队预选");
check(compact(main).includes("persistSource:source=>") && compact(main).includes("localPreferences.writeJson(PLAYER_NAV_CONTEXT_KEY,source)") && compact(main).includes("persistQuery:query=>localPreferences.writeJson(PLAYER_QUERY_KEY,query)"), "来源上下文或查询没有接通原持久键");
const directDirectoryEntry=between(catalog,"  directEntry()", "  async open(");
check(directDirectoryEntry.includes("if (!teamId) return false"), "通用球员入口不应清除普通用户筛选");
check(directDirectoryEntry.includes("this.queryValue.team_id === teamId") && directDirectoryEntry.includes("team_id: null"), "通用入口没有清除来源预选");
check(directDirectoryEntry.includes("this.setSource(null)") && directDirectoryEntry.includes("this.adopt(null)"), "通用入口没有清除来源和旧详情");
check(compact(feature).includes('this.ports.section("directory",true)') && main.includes('playersFeature.directEntry()'), "通用入口未恢复目录或未接线");
const openFromTeam=between(main,"async function openPlayerFromTeam", "async function openPlayerProfileFromTeam");
check(compact(openFromTeam).includes("team_id:source.teamId") && !openFromTeam.includes("locked:"), "球队来源未携带稳定球队身份");
check(openFromTeam.includes('active_section: "directory"') && !openFromTeam.includes("appendWorkspaceTab"), "球队普通入口应进入目录而非完整档案");
check(compact(openFromTeam).includes("awaitteamsFeature.readPlayerFromTeam(playerId)") && openFromTeam.includes("!source.current()") && openFromTeam.includes("playersFeature.enterSource(") && openFromTeam.includes("source.player") && openFromTeam.includes("inspector_collapsed: false"), "普通来源入口未使用守卫后的速览响应");
const directProfile=between(main,"async function openPlayerProfileFromTeam", "async function openTeamApiWorkspace");
check(directProfile.includes('active_section: "profile"') && directProfile.includes("appendWorkspaceTab"), "显式完整档案入口失联");
const lineupEntry=between(main,"async function openPlayerFromLineup", "async function returnToLineupWorkspace");
check(compact(lineupEntry).includes('source:"match_lineup"') && compact(lineupEntry).includes('origin_page:"lineups"') && compact(lineupEntry).includes('team_id:teamId') && compact(lineupEntry).includes('active_section:"profile"'), "阵容入口未保留独立来源/稳定身份/完整档案");
check(page.includes("return-to-lineup-workspace") && page.includes("返回比赛阵容"), "缺少返回阵容入口");
const searchPlayers=between(feature,"  async search()", "  async clearFilters()");
check(searchPlayers.includes('team_id: field("player-filter-team")') && main.includes("playersFeature.search()"), "筛选未使用用户当前球队选择");
check(catalog.includes("this.sourceValue.team_id !== query.team_id") && catalog.includes("this.setSource(null)"), "更换球队后未清除旧来源");
check(!searchPlayers.includes("lockedTeamId") && !searchPlayers.includes("?? sourceTeamId"), "筛选仍被强制回退到来源球队");
check(compact(feature).includes("this.catalog.setSource(null)") && feature.includes("normalizePlayerQuery(null)"), "清除全部未清来源/球队过滤");
check(!main.includes("beginChangePlayerSourceTeam") && !main.includes('case "change-player-source-team"'), "仍保留多余的更换来源球队锁定流程");
check(!main.includes("clearPlayerSourceContext") && !main.includes('case "clear-player-source-context"'), "仍保留多余的解除来源球队锁定流程");

for (const label of ["球队筛选已带入", "从球队页带入", "已自动选中，可直接修改或清除", "清除全部"]) {
  check(page.includes(label), `球员页缺少可编辑预选语义：${label}`);
}
for (const removed of ["当前来源球队已锁定", "更换球队", "解除来源球队", "清除其他筛选"]) {
  check(!page.includes(removed), `球员页仍显示旧锁定语义：${removed}`);
}
check(page.includes('id="player-filter-team"') && !page.includes('id="player-filter-team" ${sourceLocked ? "disabled" : ""}'), "球队筛选器仍可能因为来源上下文被禁用");
check(page.includes("player-source-prefill") && styles.includes(".player-source-prefill"), "缺少紧凑的球队预选提示");

if (failures.length) {
  console.error("球队到球员筛选预选验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("球队到球员筛选预选验证通过：上下文入口自动预选球队，通用球员入口恢复全部球员目录，球队筛选可直接修改或清除。");
