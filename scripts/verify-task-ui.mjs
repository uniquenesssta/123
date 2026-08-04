import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const json = (relative) => JSON.parse(read(relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const contract = json("contracts/task-ui-contract.json");
const styles = read("src/styles/taskWorkspace.css");
const components = read("src/components/taskWorkspace.ts");
const main = read("src/main.ts");
const legacyStyles = read("src/styles/components.css");

check(contract.contract_id === "football.task-ui-contract.v1", "阶段 B UI 契约 ID 错误");
check(contract.contract_version === "1.5.0", "阶段 B UI 契约版本错误");
check(fs.existsSync(path.join(root, "src/components/taskWorkspace.ts")), "缺少任务型页面共享组件");
check(fs.existsSync(path.join(root, "src/styles/taskWorkspace.css")), "缺少任务型页面共享样式");
check(components.includes("export function taskPageHeader") && components.includes("export function taskContextRibbon"), "共享页面标题或上下文条组件缺失");

for (const pageName of contract.core_pages) {
  const page = read(`src/pages/${pageName}.ts`);
  for (const component of contract.shared_components) {
    check(page.includes(component), `${pageName} 页面没有接入 ${component}`);
  }
  check(!page.includes("page-heading simple-heading"), `${pageName} 的未连接或空状态仍使用旧页面标题`);
}
for (const pageName of contract.master_detail_pages) {
  const page = read(`src/pages/${pageName}.ts`);
  check(page.includes("master-detail-workspace"), `${pageName} 页面没有接入统一主从详情语义`);
}

const players = read("src/pages/players.ts");
check(contract.player_team_prefill?.mode === "prefill" && contract.player_team_prefill?.filter_editable === true && contract.player_team_prefill?.clear_all_removes_prefill === true, "球员球队上下文契约没有定义为可编辑预选");
check(players.includes("球队筛选已带入") && players.includes("从球队页带入") && players.includes("已自动选中，可直接修改或清除"), "球员页缺少可见的球队预选语义");
check(!players.includes("当前来源球队已锁定") && !players.includes("解除来源球队"), "球员页仍残留来源球队锁定语义");

const review = read("src/pages/review.ts");
check(review.includes(contract.review.rail_class) && review.includes(contract.review.workspace_class), "赛后复盘缺少固定步骤轨道或当前步骤工作区");
check(review.includes(`data-action=\"${contract.review.selection_action}\"`), "赛后复盘缺少步骤选择动作");
for (let step = 1; step <= contract.review.step_count; step += 1) {
  check(review.includes(`no: ${step},`), `赛后复盘缺少第 ${step} 步`);
}
for (const label of ["本步用途", "当前状态", "完成条件", "阻塞原因", "下一步动作"]) {
  check(review.includes(label), `赛后复盘当前步骤缺少：${label}`);
}
check(main.includes(`case \"${contract.review.selection_action}\"`) && main.includes("active_section: `step-${step}`"), "步骤选择没有持久化到工作区状态");
check(!review.includes("review-package-steps") && !legacyStyles.includes(".review-package-steps"), "旧九卡片复盘布局仍有残留");

for (const className of [".task-page-header", ".task-context-ribbon", ".task-empty-workspace", ".master-detail-workspace", ".review-command-center", ".review-stage-rail", ".review-stage-workspace"]) {
  check(styles.includes(className), `阶段 B 样式缺少 ${className}`);
}
for (const breakpoint of contract.responsive_breakpoints) {
  check(styles.includes(`@media (max-width: ${breakpoint}px)`), `阶段 B 缺少 ${breakpoint}px 响应式断点`);
}
check(styles.includes("--task-control-height") && styles.includes("--task-rail-width") && styles.includes("--task-gap-5"), "阶段 B 密度与尺寸令牌不完整");
check(styles.includes("grid-template-columns: minmax(270px, var(--task-rail-width)) minmax(0, 1fr)"), "九步复盘没有采用轨道与任务区双栏结构");
check(styles.includes("grid-template-columns: 1fr") && styles.includes(".review-stage-facts { grid-template-columns: 1fr; }"), "窄屏单栏降级规则缺失");

for (const item of contract.screenshot_cases) {
  check(fs.existsSync(path.join(root, "tests/ui", item.fixture)), `缺少截图用例：${item.fixture}`);
  check(fs.existsSync(path.join(root, "tests/ui/baselines", `${item.name}.png`)), `缺少截图基线：${item.name}.png`);
}

if (failures.length) {
  console.error("阶段 B 任务型页面与 UI 体系验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log(`阶段 B UI 验证通过：${contract.core_pages.length} 个核心页面、${contract.review.step_count} 步复盘链路与 ${contract.screenshot_cases.length} 个截图基线完整。`);
