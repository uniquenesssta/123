import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const main = read("src/main.ts");
const shell = read("src/app/shell.ts");
const workspace = read("src/components/workspace.ts");
const rules = read("src/pages/rules.ts");
const release = read("src/pages/release.ts");
const analytics = read("src/pages/analytics.ts");
const apiWorkspace = read("src/pages/apiWorkspace.ts");
const openai = read("src/pages/openai.ts");
const database = read("src/pages/database.ts");
const logs = read("src/pages/logs.ts");
const architecture = read("src/pages/architecture.ts");
const styles = read("src/styles/moduleWorkspaces.css");
const pkg = JSON.parse(read("package.json"));
const taskUiContract = JSON.parse(read("contracts/task-ui-contract.json"));

check(
  main.indexOf('import "./styles/moduleWorkspaces.css";') > main.indexOf('import "./styles/coreWorkspaces.css";'),
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

check(main.includes('rulesPage(state, pendingRulePackage, workspace.active_section ?? "catalog")'), "规则页未从工作区状态恢复当前任务");
for (const [id, label] of [["catalog", "赛事目录"], ["structure", "赛事结构"], ["routing", "模型路由"], ["packages", "规则包"]]) {
  check(rules.includes(`id: "${id}"`) && rules.includes(`label: "${label}"`), `规则页缺少任务：${label}`);
  check(rules.includes(`data-workspace-section="${id}"`), `规则页任务 ${id} 未接入页面内容区`);
}
for (const id of ["rules-custom-competition", "rules-season-structure"]) {
  check(rules.includes(`id="${id}"`) && rules.includes(`{ id: "${id}"`), `规则页缺少深层定位目标：${id}`);
}
check(rules.includes("module-workspace-page model-rules-workspace"), "规则页未接入扩展工作区骨架");

for (const [id, label] of [["overview", "发布总览"], ["chain", "全链路"], ["performance", "性能"], ["security", "安全"], ["cost", "成本"], ["history", "历史报告"]]) {
  check(release.includes(`id: "${id}"`) && release.includes(`label: "${label}"`), `发布验收缺少任务：${label}`);
  check(release.includes(`data-workspace-section="${id}"`), `发布验收任务 ${id} 未接入页面内容区`);
}
check(release.includes('class="release-core-layout'), "发布验收缺少主工作区与检查器布局");
check(!release.includes('<aside class="panel module-sidebar"'), "发布验收仍保留重复的页面级左侧目录");

for (const [id, label] of [["analysis-history-step", "历史样本"], ["analysis-model-step", "完整分析"], ["analysis-quality-step", "质量门禁"], ["analysis-lifecycle-step", "受控校准"]]) {
  check(analytics.includes(`{ id: "${id}"`) && analytics.includes(`label: "${label}"`), `分析链缺少任务：${label}`);
  check(analytics.includes(`id="${id}"`) && analytics.includes("workspace-anchor-target"), `分析链缺少定位目标：${id}`);
}
check(analytics.includes("historyReady") && analytics.includes("analysisReady") && analytics.includes("reviewGateReady") && analytics.includes("lifecycleCompleted"), "分析链依赖状态未完整保留");

for (const [id, label] of [["chat", "对话工作台"], ["history", "会话历史"]]) {
  check(apiWorkspace.includes(`id: "${id}"`) && apiWorkspace.includes(`label: "${label}"`), `AI 问答缺少任务：${label}`);
  check(apiWorkspace.includes(`data-workspace-section="${id}"`), `AI 问答任务 ${id} 未接入页面内容区`);
}
check(main.includes('active_section ?? "chat"'), "AI 问答未恢复默认对话任务");
check((main.match(/patchModule\("api_workspace", \{ active_section: "chat" \}\)/g) ?? []).length >= 2, "新建或选择历史会话后未统一返回对话工作台");
check(apiWorkspace.includes('data-action="select-workspace-section" data-section-id="chat"'), "历史预览缺少继续对话入口");

for (const id of ["openai-profile-list-section", "openai-request-settings", "openai-model-settings", "openai-test-security"]) {
  check(openai.includes(`{ id: "${id}"`) && openai.includes(`id="${id}"`), `兼容 API 缺少任务定位目标：${id}`);
}
check(openai.includes("workspaceTaskAnchorNavigation") && openai.includes("openai-module-stage"), "兼容 API 未采用单页编辑与任务导航结构");
check(openai.includes("支持多个 OpenAI-compatible 配置档案"), "兼容 API 原有产品说明被意外删除");

for (const id of ["database-overview", "database-connection", "database-statistics", "database-danger"]) {
  check(database.includes(`{ id: "${id}"`) && database.includes(`id="${id}"`), `数据库页缺少任务定位目标：${id}`);
}
for (const id of ["issue-summary", "issue-processing", "issue-records"]) {
  check(logs.includes(`{ id: "${id}"`) && logs.includes(`id="${id}"`), `问题日志页缺少任务定位目标：${id}`);
}
for (const id of ["architecture-flow", "architecture-principles", "architecture-modules"]) {
  check(architecture.includes(`{ id: "${id}"`) && architecture.includes(`id="${id}"`), `系统说明页缺少任务定位目标：${id}`);
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
