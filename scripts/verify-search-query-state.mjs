import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");
const requireTrue = (condition, message) => {
  if (!condition) throw new Error(message);
};

const players = read("src/pages/players.ts");
const teams = read("src/pages/teams.ts");
const presets = read("src/pages/lineupPresets.ts");
const main = read("src/main.ts");
const viewState = read("src/app/viewState.ts");

requireTrue(
  players.includes('data-workspace-panel="players-filter" data-workspace-persist="false"'),
  "球员筛选器仍可能被旧工作区控件快照覆盖",
);
requireTrue(
  players.includes('class="entity-search wide" data-workspace-persist="false"')
    && players.includes('id="player-search" value="${escapeHtml(query.search ?? "")}"'),
  "球员搜索框没有以当前已应用查询作为唯一显示值",
);
requireTrue(
  players.includes('搜索“${escapeHtml(query.search)}”'),
  "球员结果区未回显当前实际搜索词",
);
requireTrue(
  teams.includes('data-workspace-panel="teams-directory" data-workspace-persist="false"')
    && teams.includes('data-workspace-panel="teams-filter" data-workspace-persist="false"'),
  "球队搜索与筛选仍可能被旧工作区控件快照覆盖",
);
requireTrue(
  presets.includes('data-workspace-scroll-key="lineup-preset-team-directory" data-workspace-persist="false"'),
  "阵容预设球队搜索仍可能被旧工作区控件快照覆盖",
);
requireTrue(
  viewState.includes("control.closest('[data-workspace-persist=\"false\"]')"),
  "工作区状态恢复没有尊重受控查询控件的持久化边界",
);
requireTrue(
  main.includes('search: nullableValue("player-search")')
    && main.indexOf('search: nullableValue("player-search")') < main.indexOf('await runBusy(() => loadPlayerCatalog(false))'),
  "球员搜索词没有在请求前提交到查询状态",
);
requireTrue(
  !players.includes('value="Marlon"') && !teams.includes('value="Marlon"') && !presets.includes('value="Marlon"'),
  "生产页面仍残留固定示例搜索词 Marlon",
);

console.log("搜索词状态同步契约验证通过：已应用查询始终覆盖旧工作区快照，并在结果区回显真实搜索词。");
