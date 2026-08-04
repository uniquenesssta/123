import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const between = (source, start, end) => {
  const from = source.indexOf(start);
  const to = source.indexOf(end, from + start.length);
  return from >= 0 && to > from ? source.slice(from, to) : "";
};

const main = read("src/main.ts");
const players = read("src/pages/players.ts");
const teams = read("src/pages/teams.ts");
const css = read("src/styles/entityCenter.css");

for (const source of [players, teams]) {
  check(source.includes("entity-task-workspace"), "球队/球员完整档案必须在主工作区直接展开");
  check(!source.includes("entity-task-backdrop") && !source.includes("entity-task-drawer"), "球队/球员页面仍包含侧边抽屉完整档案");
}
check(!css.includes(".entity-task-backdrop") && !css.includes(".entity-task-drawer"), "旧完整档案遮罩或侧边抽屉样式仍残留");
check(css.includes(".entity-task-workspace"), "缺少主工作区完整档案样式");

check(players.includes('id="edit-player-localized-name"') && players.includes("currentLocalizedPlayerName"), "球员完整档案缺少可回显的中文姓名字段");
check(teams.includes('id="team-localized-name"') && teams.includes("currentLocalizedTeamName"), "球队完整档案缺少可回显的中文名称字段");
check(players.includes("playerNameHistory(detail)"), "球员已保存名称历史未持续显示");
check(teams.includes('class="tag-row">${aliases}'), "球队已保存名称未持续显示");

const updatePlayer = between(main, "async function updatePlayer", "async function exportPlayerTemplate");
check(updatePlayer.includes('value("edit-player-localized-name").trim()'), "保存球员时没有读取中文姓名");
check(updatePlayer.includes("if (localizedName && localizedName !== previousLocalizedName)"), "空白或未变化中文姓名仍可能新增记录");
check(updatePlayer.includes('language_code: "zh-CN"') && updatePlayer.includes("valid_from: todayDate()"), "球员中文姓名没有按明确语言和生效日保存");
check(updatePlayer.includes("await reloadSelectedPlayer()") && updatePlayer.includes('active_section: "profile"'), "保存球员后没有重新读取数据库并留在完整档案");

const updateTeam = between(main, "async function updateTeam", "async function addTeamName");
check(updateTeam.includes('value("team-localized-name").trim()'), "保存球队时没有读取中文名称");
check(updateTeam.includes("if (localizedName && localizedName !== previousLocalizedName)"), "空白或未变化球队中文名仍可能新增记录");
check(updateTeam.includes('language_code: "zh-CN"') && updateTeam.includes("valid_from: todayDate()"), "球队中文名称没有按明确语言和生效日保存");
check(updateTeam.includes("await reloadSelectedTeam()") && updateTeam.includes('active_section: "profile"'), "保存球队后没有重新读取数据库并留在完整档案");

const addPlayerName = between(main, "async function addPlayerName", "async function assignPlayerPosition");
const addTeamName = between(main, "async function addTeamName", "async function saveTeamProfile");
for (const [source, label] of [[addPlayerName, "球员"], [addTeamName, "球队"]]) {
  check(source.includes('throw new Error("请选择名称语言")'), `${label}新增名称仍允许无语言记录`);
  check(source.includes("valid_from: todayDate()"), `${label}新增名称没有明确当前生效日`);
}

for (const action of ["open-player-profile", "open-team-profile", "open-player-profile-from-team"]) {
  check(main.includes(`case "${action}"`), `主控制器缺少直接完整档案动作：${action}`);
}
check(players.includes('data-action="open-player-profile"'), "球员名单缺少直接打开完整档案入口");
check(teams.includes('data-action="open-team-profile"'), "球队目录或主区缺少直接打开完整档案入口");
check(teams.includes('data-action="open-player-profile-from-team"'), "球队阵容缺少直接打开球员完整档案入口");
check(players.includes("playerQuickInspector") && teams.includes("playerInspector"), "速览能力被错误删除；速览应保留但不得成为必经步骤");

const squadGroup = between(teams, "function squadGroup", "function profileValue");
check(squadGroup.includes('data-action="open-player-profile-from-team"'), "球队完整档案当前阵容点击球员没有直接进入球员完整档案");
check(!squadGroup.includes('data-action="open-player-from-team"'), "球队完整档案当前阵容仍先跳转球员目录");
const teamDetailPanel = between(teams, "function detailPanel", "function teamPackagePanel");
check(!teamDetailPanel.includes('data-action="open-player-from-team"'), "球队完整档案球员履历仍先跳转球员目录");
check(teamDetailPanel.includes('data-action="open-player-profile-from-team"'), "球队完整档案缺少球员完整档案直达动作");

const playerRows = between(players, "function playerTableRows", "function playerQuickInspector");
check(playerRows.includes('data-action="open-player-profile"'), "球员目录缺少完整档案主操作");
check(!playerRows.includes('>速览</button><button'), "球员目录操作列仍重复放置速览与完整档案按钮");
check(playerRows.match(/table-row-action/g)?.length === 2, "球员目录每行应只保留一个完整档案按钮及其容器");
check(css.includes('.player-directory-table th:nth-child(8) { width: 88px; }'), "球员目录操作列没有预留稳定宽度");
check(css.includes('.roster-table th:last-child { width: 88px; }'), "球队阵容操作列没有预留稳定宽度");

check(players.includes('data-action="return-to-source-team-profile"'), "从球队完整档案进入球员后缺少返回来源球队完整档案入口");
check(main.includes("async function returnToSourceTeamProfile") && main.includes('case "return-to-source-team-profile"'), "主控制器缺少返回来源球队完整档案链路");

if (failures.length) {
  console.error("球队/球员完整档案持久显示专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("球队/球员完整档案持久显示专项验证通过：保存后回读、中文名空白保护、名称历史回显及主工作区直达均已锁定。");
