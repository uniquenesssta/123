import {verifyProfileHierarchy} from "./features/verifyProfileHierarchy.mjs";
import {verifyAiWorkspaceHierarchy} from "./features/verifyAiWorkspaceHierarchy.mjs";
import {verifyReleaseHierarchy} from "./features/verifyReleaseHierarchy.mjs";
import {verifyAnalyticsHierarchy} from './features/verifyAnalyticsHierarchy.mjs';
import {productionStyleFiles} from "./styles/styleGraph.mjs";
import {readProjectSource} from "./styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../",import.meta.url));
import { existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readProjectSource(stylesheetRoot,join(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const main = read("src/main.ts");
const shell = JSON.parse(read("contracts/global-visual-system-contract.json")).shell_source_entries.map(read).join("\n");
const workspace = read("src/ui/navigation/section-navigation/sectionNavigationView.ts");
const rules = ["src/features/rules/rulesView.ts", "src/features/rules/bindingView.ts", "src/features/competitions/catalogueView.ts", "src/features/competitions/competitionView.ts"].map(read).join("\n");


const apiWorkspace = ["aiWorkspacePage","aiWorkspaceMessages"].map(name=>read(`src/features/ai-workspace/${name}.ts`)).join("\n");

const database = read("src/features/database/databaseView.ts");
const logs = read("src/pages/logs.ts");
const architecture = read("src/features/architecture/architectureView.ts");
const styles = read("src/styles/moduleWorkspaces.css");
const pkg = JSON.parse(read("package.json"));
const taskUiContract = JSON.parse(read("contracts/task-ui-contract.json"));

check(
  productionStyleFiles(root).indexOf("src/styles/moduleWorkspaces.css") > productionStyleFiles(root).indexOf("src/styles/coreWorkspaces.css"),
  "扩展工作区样式必须在核心工作区样式之后加载",
);
check(
  workspace.includes("export function workspaceTaskAnchorNavigation") && workspace.includes("workspace-task-anchor-nav"),
  "缺少统一的页面任务锚点导航组件",
);
check(
  main.includes('.workspace-anchor-nav, .workspace-task-anchor-nav') && main.includes('case "jump-workspace-anchor"'),
  "统一锚点处理器未覆盖页面任务导航",
);

for (const page of ["rules", "release", "analytics", "api_workspace", "openai", "database", "logs", "architecture"]) {
  check(shell.includes(`"${page}"`), `应用壳未将 ${page} 纳入工作区页面`);
}

check(main.includes('rulesPage(session.bootstrap, rulesFeature.packages.pending, workspace.active_section ?? "catalog", rulesFeature.packages.reading)'), "规则页未从会话摘要及工作区状态恢复当前任务");
for (const [id, label] of [["catalog", "赛事目录"], ["structure", "赛事结构"], ["routing", "模型路由"], ["packages", "规则包"]]) {
  check(rules.includes(`id: "${id}"`) && rules.includes(`label: "${label}"`), `规则页缺少任务：${label}`);
  check(rules.includes(`data-workspace-section="${id}"`), `规则页任务 ${id} 未接入页面内容区`);
}
for (const id of ["rules-custom-competition", "rules-season-structure"]) {
  check(rules.includes(`id="${id}"`) && rules.includes(`{ id: "${id}"`), `规则页缺少深层定位目标：${id}`);
}
check(rules.includes("module-workspace-page model-rules-workspace"), "规则页未接入扩展工作区骨架");

await verifyReleaseHierarchy(root);

await verifyAnalyticsHierarchy(root);

await verifyAiWorkspaceHierarchy(root);

await verifyProfileHierarchy(root);

for (const id of ["database-overview", "database-connection", "database-statistics", "database-danger"]) {
  check(database.includes(`id:"${id}"`) && database.includes('id="${panel.id}"') && database.includes("data-navigation-panel"), `数据库页缺少任务定位目标：${id}`);
}
for (const id of ["issue-summary", "issue-processing", "issue-records"]) {
  check(logs.includes(`{ id: "${id}"`) && logs.includes(`id="${id}"`), `问题日志页缺少任务定位目标：${id}`);
}
for (const id of ["architecture-flow", "architecture-principles", "architecture-modules"]) {
  check(architecture.includes(`id:"${id}"`) && architecture.includes('id="${panel.id}"') && architecture.includes("data-navigation-panel"), `系统说明页缺少任务定位目标：${id}`);
}
for (const [name, source] of [["数据库", database], ["问题日志", logs], ["系统说明", architecture]]) {
  check(source.includes("module-workspace-page management-module-workspace"), `${name}未接入管理模块统一工作区`);
}

for (const selector of [".module-workspace-page", ".module-workspace-stage", ".workspace-task-anchor-nav", ".release-core-layout", ".ai-chat-layout", ".ai-history-layout", ".openai-module-stage", ".management-module-stage"]) {
  check(styles.includes(selector), `扩展工作区样式缺少：${selector}`);
}
check(styles.includes("@media (max-width: 1180px)") && styles.includes("@media (max-width: 900px)"), "扩展工作区缺少桌面收敛和窄屏响应规则");
check(taskUiContract.screenshot_cases.some((item) => item.name === "extended-workspace-hierarchy-1440x900" && item.fixture === "extended-workspace-hierarchy.html"), "任务型 UI 契约缺少扩展工作区视觉回归");
check(existsSync(join(root, "tests/ui/extended-workspace-hierarchy.html")) && existsSync(join(root, "tests/ui/baselines/extended-workspace-hierarchy-1440x900.png")), "扩展工作区视觉 fixture 或基线缺失");
check(pkg.scripts["verify:extended-workspace-hierarchy"] === "node scripts/verify-extended-workspace-hierarchy.mjs", "package.json 缺少扩展工作区专项验证命令");
check(pkg.scripts.build.includes("verify-frontend.mjs") && read("scripts/verify-frontend.mjs").includes("verify-extended-workspace-hierarchy.mjs"), "构建或前端全量门禁未接入扩展工作区专项验证");

if (failures.length) {
  console.error("模型、分析、AI与管理页内层级专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}

console.log("模型、分析、AI与管理页内层级专项验证通过：全局双层导航之外，深层任务均在当前页面闭环完成。\n");
