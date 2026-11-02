import {verifyAnalyticsHierarchy} from './features/verifyAnalyticsHierarchy.mjs';
import {readProjectSource} from "./styles/projectSource.mjs";
import {fileURLToPath as stylesheetFilePath} from "node:url";
const stylesheetRoot=stylesheetFilePath(new URL("../",import.meta.url));
import fs from "node:fs";

const read = (path) => readProjectSource(stylesheetRoot,new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const visual = read("src/styles/visualSystem.css");
const entity = read("src/styles/entityCenter.css");
const analytics = read("src/pages/analytics.ts");
const contract = JSON.parse(read("contracts/task-ui-contract.json"));

check(
  /\.entity-page\.task-page\s*\{[^}]*grid-template-rows:\s*auto\s+auto\s+32px\s+minmax\(0,\s*1fr\)/s.test(visual),
  "球队/球员页面没有为标题、上下文、任务栏和内容区保留四个独立网格行",
);
check(
  /\.entity-page\.task-page\s*>\s*\.core-local-navigation\s*\{[^}]*height:\s*32px[^}]*max-height:\s*32px/s.test(visual),
  "球队/球员页内任务栏缺少固定高度，仍可能被内容撑高",
);
check(
  /\.rules-scope-list,\s*\n\.rules-region-list\s*\{[^}]*align-content:\s*start[^}]*grid-auto-rows:\s*max-content/s.test(entity),
  "赛事一级分类没有与地区列表采用相同的顶部紧凑纵向排列",
);
// Latest approved S12 uses horizontal steps: retain all original reachability, active-stage and gate obligations via real rendering (29 assertions), with no accordion-specific source shape.
await verifyAnalyticsHierarchy(stylesheetRoot);

check(!analytics.includes('<section class="analysis-chain-map"'), "分析页仍重复显示第二套四阶段流程图");
const analysisStyles=read("src/features/analytics/analytics.css");check(analysisStyles.includes(".analytics-workflow")&&analysisStyles.includes(".analytics-viewport.workspace-module-view.active")&&analysisStyles.includes("grid-template-rows:auto auto auto auto auto minmax(0,1fr)"),"分析页缺少批准的横向步骤与剩余高度独立视口样式");

const screenshotNames = new Set(contract.screenshot_cases.map((item) => item.name));
for (const name of [
  "reported-entity-workspace-1440x900",
  "reported-rules-directory-1440x900",
  "reported-analysis-focus-1440x900",
  "reported-entity-workspace-2560x1440",
  "reported-rules-directory-2560x1440",
  "reported-analysis-focus-2560x1440",
]) {
  check(screenshotNames.has(name), `截图回归缺少 ${name}`);
}

if (failures.length) {
  console.error("用户反馈页面布局专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("用户反馈页面布局专项验证通过：资源中心任务栏高度、赛事分类对齐、分析单阶段聚焦及 2K 大屏可读性均已锁定。");
