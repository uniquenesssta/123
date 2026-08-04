import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const navigation = read("src/app/navigation.ts");
const types = read("src/types.ts");
const main = read("src/main.ts");
const page = read("src/pages/lineupPresets.ts");
const styles = read("src/styles/moduleWorkspaces.css");
const contract = JSON.parse(read("contracts/workspace-ui-contract.json"));
const pkg = JSON.parse(read("package.json"));

check(types.includes('| "lineup_presets"'), "页面类型缺少阵容预设入口");
check(navigation.includes('page: "lineup_presets"') && navigation.includes('label: "阵容预设"'), "资源二级菜单缺少阵容预设");
check(main.includes('case "lineup_presets"') && main.includes("lineupPresetsPage("), "主路由未渲染阵容预设页面");
check(main.includes('nextPage === "teams" || nextPage === "lineup_presets"'), "阵容预设页面未加载球队目录");
check(main.includes("selectLineupPresetTeam") && main.includes('case "select-lineup-preset-team"'), "阵容预设缺少球队选择链路");
check(main.includes('workspaceState.module("lineup_presets").active_tab_id') && main.includes('workspaceState.patchModule("lineup_presets", { active_tab_id:'), "阵容预设未保存或恢复当前球队");
check(main.includes("coachList = coaches") && main.includes("formationCatalog = formations"), "阵容预设编辑器未加载教练与阵型目录");
check(page.includes("球队阵容预设") && page.includes("lineup-preset-workspace"), "阵容预设页面骨架缺失");
check(page.includes('data-action="open-team-lineup-preset-editor"'), "阵容预设页面缺少新建或编辑入口");
check(page.includes('data-action="duplicate-team-lineup-preset"') && page.includes('data-action="archive-team-lineup-preset"'), "阵容预设页面缺少复制或归档入口");
check(page.includes('data-action="request-delete-team-lineup-preset"'), "阵容预设页面缺少归档记录永久删除入口");
check(styles.includes(".lineup-preset-workspace") && styles.includes(".lineup-preset-team-item.active"), "阵容预设页面布局或选中状态样式缺失");
check(contract.modules.includes("lineup_presets"), "工作区契约未登记阵容预设模块");
check(contract.secondary_navigation?.资源?.includes("阵容预设"), "工作区契约未登记资源二级入口");
check(pkg.scripts["verify:lineup-preset-navigation"] === "node scripts/verify-lineup-preset-navigation.mjs", "package.json缺少阵容预设导航专项验证命令");
check(read("scripts/verify-frontend.mjs").includes("verify-lineup-preset-navigation.mjs"), "前端全量门禁未接入阵容预设导航验证");

if (failures.length) {
  console.error("阵容预设二级入口专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("阵容预设二级入口专项验证通过：资源导航、球队选择、预设管理、编辑依赖与状态契约均已接通。");
