import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const main = read("src/main.ts");
const workspace = read("src/components/workspace.ts");
const styles = read("src/styles/coreWorkspaces.css");
const lineups = read("src/pages/lineups.ts");
const prediction = read("src/pages/prediction.ts");
const teams = read("src/pages/teams.ts");
const players = read("src/pages/players.ts");
const pkg = JSON.parse(read("package.json"));

check(main.indexOf('import "./styles/coreWorkspaces.css";') > main.indexOf('import "./styles/taskWorkspace.css";'), "核心工作区样式必须最后加载，避免旧布局覆盖新层级");
check(workspace.includes("export interface WorkspaceAnchorItem") && workspace.includes("workspaceAnchorNavigation"), "缺少统一页内锚点导航组件");
check(workspace.includes("readonly disabled?: boolean") && workspace.includes('disabled aria-disabled="true"'), "三级入口缺少不可用状态语义");
check(main.includes('case "jump-workspace-anchor"') && main.includes("target instanceof HTMLDetailsElement") && main.includes("target.scrollIntoView"), "页内深层导航缺少自动展开与定位链路");

for (const [name, source, rootClass] of [
  ["比赛中心", lineups, "core-match-workspace"],
  ["赛事推演", prediction, "core-prediction-workspace"],
  ["球队中心", teams, "core-team-workspace"],
  ["球员中心", players, "core-player-workspace"],
]) {
  check(source.includes("core-workspace-page") && source.includes(rootClass), `${name}未接入统一核心工作区骨架`);
  check(source.includes('class="core-local-navigation"'), `${name}缺少固定三级导航容器`);
  check(source.includes('class="core-workspace-stage"'), `${name}缺少独立任务内容区`);
}

check(lineups.includes('workspaceAnchorNavigation("比赛编辑"') && lineups.includes('workspaceAnchorNavigation("双方阵容"'), "比赛中心未将编辑和阵容深层步骤收拢为页内导航");
for (const id of ["match-editor-competition", "match-editor-teams", "match-editor-status", "match-editor-actions", "lineup-builder-context", "lineup-builder-submit"]) {
  check(lineups.includes(`id="${id}"`), `比赛中心缺少页内定位目标：${id}`);
}
check(lineups.includes('id="lineup-builder-${side}"') && lineups.includes('lineupSideCard("home"') && lineups.includes('lineupSideCard("away"'), "双方阵容未通过统一卡片生成主客队定位目标");

check(prediction.includes('workspaceAnchorNavigation("正式推演"'), "赛事推演缺少正式推演页内导航");
for (const id of ["prediction-form-setup", "prediction-form-readiness", "prediction-result", "route-preview"]) {
  check(prediction.includes(`id="${id}"`), `赛事推演缺少页内定位目标：${id}`);
}

for (const label of ["球队目录", "完整档案", "资料工作包", "新增资料"]) {
  check(teams.includes(`label: "${label}"`), `球队中心三级导航缺少：${label}`);
}
check(teams.includes('disabled: !selectedTeam'), "球队完整档案入口未随选择状态禁用");
check(teams.includes('workspaceAnchorNavigation("球队档案"'), "球队完整档案缺少页内深层导航");
for (const id of ["team-profile-overview", "team-profile-identity", "team-profile-tactics", "team-profile-formations", "team-profile-coaches", "team-profile-players", "team-profile-presets", "team-profile-lineups", "team-profile-recent"]) {
  check(teams.includes(`id="${id}"`), `球队档案缺少页内定位目标：${id}`);
}

for (const label of ["球员目录", "完整档案", "球员工作包", "新增球员"]) {
  check(players.includes(`label: "${label}"`), `球员中心三级导航缺少：${label}`);
}
check(players.includes('disabled: !selectedPlayer'), "球员完整档案入口未随选择状态禁用");
check(players.includes('workspaceAnchorNavigation("球员档案"'), "球员完整档案缺少页内深层导航");
for (const id of ["player-profile-overview", "player-profile-actions", "player-profile-base", "player-profile-names", "player-profile-positions", "player-profile-teams", "player-profile-availability", "player-profile-ability", "player-profile-external"]) {
  check(players.includes(`id="${id}"`), `球员档案缺少页内定位目标：${id}`);
}

check(!teams.includes("entity-mode-switch") && !players.includes("entity-mode-switch"), "球队或球员页仍保留重复的旧模式切换入口");
check(styles.includes(".core-workspace-page") && styles.includes("grid-template-rows: auto auto auto minmax(0, 1fr)"), "核心工作区四层页面骨架样式缺失");
check(styles.includes(".core-local-navigation") && styles.includes(".workspace-anchor-nav"), "三级导航或页内深层导航样式缺失");
check(styles.includes("position: sticky") && styles.includes("scroll-margin-top"), "页内导航缺少固定定位与滚动偏移");
check(styles.includes("@media (max-width: 1100px)") && styles.includes("@media (max-width: 760px)"), "核心工作区缺少响应式收敛规则");
check(pkg.scripts["verify:core-workspace-hierarchy"] === "node scripts/verify-core-workspace-hierarchy.mjs", "package.json缺少核心工作区专项验证命令");
check(pkg.scripts.build.includes("verify-frontend.mjs") && read("scripts/verify-frontend.mjs").includes("verify-core-workspace-hierarchy.mjs"), "构建或前端全量门禁未接入核心工作区专项验证");

if (failures.length) {
  console.error("核心业务页内层级专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}

console.log("核心业务页内层级专项验证通过：比赛、推演、球队与球员均采用固定三级导航，深层任务在当前页面闭环完成。");
