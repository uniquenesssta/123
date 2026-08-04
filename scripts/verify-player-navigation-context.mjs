import fs from "node:fs";
const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const check = (ok, message) => { if (!ok) failures.push(message); };
const main = read("src/main.ts");
const page = read("src/pages/players.ts");
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
check(main.includes("initialTeamQuery()") && main.includes("initialPlayerQuery()"), "球队/球员筛选未从持久状态恢复");
check(main.includes("if (playerNavigationContext) playerQuery.team_id = playerNavigationContext.team_id"), "应用重启后没有恢复从球队页带入的预选球队");
check(main.includes("setPlayerNavigationContext") && main.includes("persistPlayerQuery"), "球员来源上下文未持久保存");

const directDirectoryEntry = between(main, "function prepareDirectPlayerDirectoryEntry", "const appRoot");
check(directDirectoryEntry.includes("if (!sourceTeamId) return"), "直接进入球员目录时不应清除普通用户筛选上下文");
check(directDirectoryEntry.includes("playerQuery.team_id === sourceTeamId") && directDirectoryEntry.includes("team_id: null"), "直接进入全部球员时没有清除球队页带入的球队预选");
check(directDirectoryEntry.includes("setPlayerNavigationContext(null)"), "直接进入全部球员时没有清除球队来源上下文");
check(directDirectoryEntry.includes('active_section: "directory"') && directDirectoryEntry.includes("selectedPlayer = null"), "直接进入全部球员时没有恢复目录模式或清除来源球员选择");
check(main.includes('if (targetPage === "players") prepareDirectPlayerDirectoryEntry();'), "左侧或通用球员入口没有切换为全部球员目录");

const openFromTeam = between(main, "async function openPlayerFromTeam", "async function openPlayerProfileFromTeam");
check(openFromTeam.includes("team_id: teamId") && !openFromTeam.includes("locked:"), "从球队进入球员页时没有只带入球队筛选，或仍写入锁定状态");
check(openFromTeam.includes('active_section: "directory"') && !openFromTeam.includes('active_section: "profile"'), "球队跳转仍直接进入完整球员档案，而不是目录");
check(!openFromTeam.includes("appendWorkspaceTab"), "球队跳转仍自动打开完整档案标签");
check(openFromTeam.includes("selectedPlayer = await") && openFromTeam.includes("inspector_collapsed: false"), "普通球队跳转未在目录中保持来源球员速览");
const directProfile = between(main, "async function openPlayerProfileFromTeam", "async function openTeamApiWorkspace");

const lineupEntry = between(main, "async function openPlayerFromLineup", "async function returnToLineupWorkspace");
check(lineupEntry.includes('source: "match_lineup"') && lineupEntry.includes('origin_page: "lineups"'), "阵容球员入口没有独立来源上下文");
check(lineupEntry.includes('active_section: "profile"') && lineupEntry.includes("team_id: teamId"), "阵容球员入口没有直达完整档案或携带球队身份");
check(page.includes("return-to-lineup-workspace") && page.includes("返回比赛阵容"), "阵容来源球员档案没有返回阵容入口");
check(directProfile.includes('active_section: "profile"') && directProfile.includes("appendWorkspaceTab"), "显式完整档案动作没有直接进入球员主工作区");

const searchPlayers = between(main, "async function searchPlayers", "async function clearPlayerFilters");
check(searchPlayers.includes("team_id: selectedTeamId"), "应用筛选没有采用用户当前选择的球队");
check(searchPlayers.includes("playerNavigationContext.team_id !== selectedTeamId") && searchPlayers.includes("setPlayerNavigationContext(null)"), "用户更换或清空球队筛选后仍残留旧来源提示");
check(!searchPlayers.includes("lockedTeamId") && !searchPlayers.includes("?? sourceTeamId"), "应用筛选仍会强制回退到来源球队");

const clearFilters = between(main, "async function clearPlayerFilters", "async function nextPlayerPage");
check(clearFilters.includes("team_id: null") && clearFilters.includes("setPlayerNavigationContext(null)"), "清除全部没有清除球队筛选与来源提示");
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
