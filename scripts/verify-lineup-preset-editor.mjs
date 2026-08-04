import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const main = read("src/main.ts");
const css = read("src/styles/components.css");
const workspaceCss = read("src/styles/workspacePanels.css");
const footballText = read("src/components/footballText.ts");
const pkg = JSON.parse(read("package.json"));

for (const token of [
  'class="preset-member-position" data-native-select',
  'class="preset-member-tactical-role" data-native-select',
  'aria-label="战术位置"',
  'aria-label="战术角色"',
  '按阵型自动分配',
  'formationSlotCodes',
  'assignPresetFormationSlots',
  'validateTeamLineupPresetDraft',
  '首发位置必须完整匹配当前阵型',
]) check(main.includes(token), `阵容预设编辑器缺少：${token}`);

check(!main.includes('class="preset-member-position" value='), "战术位置仍是手工文本输入");
check(!main.includes('class="preset-member-tactical-role" value='), "战术角色仍是手工文本输入");
check(main.includes('role_origin === "lineup_override"') && main.includes('role_origin === "player_position_default"'), "战术角色未区分显式覆盖和资料继承");
check(main.includes('target.id === "lineup-preset-formation"') && main.includes('refreshPresetPositionOptions(true)'), "切换阵型后未刷新并自动分配战术位置");
check(main.includes('target.classList.contains("preset-member-position")') && main.includes('refreshPresetTacticalRoleSelect(row)'), "切换战术位置后未刷新角色选项");
check(main.includes('memberList.scrollTop = 0'), "打开编辑器时没有重置球员列表滚动位置");

for (const token of [
  ".preset-member-table-head",
  "grid-template-columns: minmax(250px, 1.65fr)",
  ".preset-member-tactical-role",
  "height: 100%",
  "max-height: none",
  "scrollbar-gutter: stable",
  ".preset-editor-validation",
]) check(css.includes(token), `阵容预设布局缺少：${token}`);
check(workspaceCss.includes(".workspace-detail-page.lineup-preset-modal .workspace-detail-body") && workspaceCss.includes("overflow: hidden"), "右侧完整工作区未锁定为内部列表滚动");
for (const code of ["LDM", "RDM", "LAM", "RAM", "LST", "RST"]) check(footballText.includes(`${code}:`), `中文位置映射缺少 ${code}`);

check(pkg.scripts["verify:lineup-preset-editor"] === "node scripts/verify-lineup-preset-editor.mjs", "package.json缺少阵容预设编辑器专项命令");
check(read("scripts/verify-frontend.mjs").includes("verify-lineup-preset-editor.mjs"), "前端全量门禁未接入阵容预设编辑器专项验证");

if (failures.length) {
  console.error("阵容预设编辑器专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("阵容预设编辑器专项验证通过：右侧全屏布局、阵型槽位下拉、战术角色选择、自动分配、结构校验和滚动边界均已锁定。");
