import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => readFileSync(join(root, relative), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const contract = JSON.parse(read("contracts/global-visual-system-contract.json"));
const styles = read(contract.style_entry);
const main = read("src/main.ts");
const pkg = JSON.parse(read("package.json"));
const taskUi = JSON.parse(read("contracts/task-ui-contract.json"));

check(contract.contract_id === "football.global-visual-system.v1", "全局视觉契约 ID 错误");
check(contract.contract_version === "1.2.0", "全局视觉契约版本错误");
check(existsSync(join(root, contract.style_entry)), "缺少全局视觉样式入口");
check(
  main.lastIndexOf('import "./styles/visualSystem.css";') > main.lastIndexOf('import "./styles/moduleWorkspaces.css";'),
  "全局视觉样式必须最后加载，避免被历史页面样式覆盖",
);

for (const token of contract.required_tokens) {
  check(styles.includes(token), `全局视觉令牌缺失：${token}`);
}
for (const [group, selectors] of Object.entries(contract.required_component_groups)) {
  for (const selector of selectors) check(styles.includes(selector), `${group} 视觉规则缺少：${selector}`);
}
for (const breakpoint of contract.responsive_breakpoints) {
  check(styles.includes(`@media (max-width: ${breakpoint}px)`), `缺少 ${breakpoint}px 响应式断点`);
}
check(styles.includes(":focus-visible"), "缺少键盘焦点可见性规则");
check(styles.includes("@media (prefers-reduced-motion: reduce)"), "缺少减少动态效果规则");
check(styles.includes("@media (forced-colors: active)"), "缺少高对比度模式规则");
check(styles.includes("scrollbar-gutter: stable"), "页面滚动条出现时仍可能造成布局横向跳动");
check(styles.includes("font-variant-numeric: tabular-nums"), "指标数字缺少等宽数字规则");
check(styles.includes("position: sticky") && styles.includes("th {"), "数据表头缺少固定阅读规则");
check(styles.includes("button:disabled") && styles.includes('[aria-disabled="true"]'), "禁用态没有同时覆盖原生和 ARIA 语义");
const density = contract.density_profile;
check(density?.desktop_reference === "high-density-admin", "高密度后台参考类型错误");
const densityExpectations = {
  "--shell-primary-rail-width": `${density.primary_rail_width}px`,
  "--shell-secondary-sidebar-width": `${density.secondary_sidebar_width}px`,
  "height: 44px": `${density.topbar_height}px`,
  "--ui-font-size": `${density.body_font_size}px`,
  "--ui-page-title-size": `${density.page_title_size}px`,
  "--ui-section-title-size": `${density.section_title_size}px`,
  "--ui-control-height": `${density.control_height}px`,
  "--ui-table-row-height": `${density.table_row_height}px`,
  "--ui-panel-padding": `${density.panel_padding}px`,
  "--ui-page-padding-x": `${density.page_padding_x}px`,
  "--ui-page-padding-y": `${density.page_padding_y}px`,
};
for (const [token, value] of Object.entries(densityExpectations)) {
  check(styles.includes(token) && styles.includes(value), `高密度视觉规格缺失：${token}=${value}`);
}
check(styles.includes("Dense tables: 36px standard rows"), "缺少高密度表格规则");
check(styles.includes("QianNiu-inspired high-density desktop information system"), "缺少高密度后台视觉系统入口");
const largeDesktop = contract.large_desktop_readability;
check(largeDesktop?.min_width === 1680 && largeDesktop?.min_height === 900, "2K 大屏可读性断点错误");
check(styles.includes(`@media (min-width: ${largeDesktop.min_width}px) and (min-height: ${largeDesktop.min_height}px)`), "缺少 2K 大屏可读性媒体查询");
for (const [token, value] of Object.entries({
  "--ui-font-size": `${largeDesktop.body_font_size}px`,
  "--ui-font-size-sm": `${largeDesktop.small_font_size}px`,
  "--ui-font-size-xs": `${largeDesktop.extra_small_font_size}px`,
  "--ui-page-title-size": `${largeDesktop.page_title_size}px`,
  "--ui-section-title-size": `${largeDesktop.section_title_size}px`,
  "--ui-control-height": `${largeDesktop.control_height}px`,
  "--ui-control-height-sm": `${largeDesktop.small_control_height}px`,
  "--ui-table-row-height": `${largeDesktop.table_row_height}px`,
})) {
  check(styles.includes(token) && styles.includes(value), `2K 大屏可读性规格缺失：${token}=${value}`);
}

for (const viewport of contract.visual_viewports) {
  check(taskUi.screenshot_cases.some((item) => item.name === viewport.name && item.fixture === viewport.fixture && item.width === viewport.width && item.height === viewport.height), `任务型 UI 契约缺少视觉视口：${viewport.name}`);
  check(existsSync(join(root, "tests/ui", viewport.fixture)), `缺少视觉 fixture：${viewport.fixture}`);
  check(existsSync(join(root, "tests/ui/baselines", `${viewport.name}.png`)), `缺少视觉基线：${viewport.name}.png`);
}
for (const fixture of new Set(taskUi.screenshot_cases.map((item) => item.fixture))) {
  const html = read(`tests/ui/${fixture}`);
  check(html.includes("../../src/styles/visualSystem.css"), `${fixture} 未加载全局视觉样式`);
}

check(pkg.scripts["verify:global-visual-system"] === "node scripts/verify-global-visual-system.mjs", "package.json 缺少全局视觉专项验证命令");
check(pkg.scripts.build.includes("verify-frontend.mjs") && read("scripts/verify-frontend.mjs").includes("verify-global-visual-system.mjs"), "构建门禁未接入全局视觉专项验证");
check(read("scripts/verify-frontend.mjs").includes("verify-global-visual-system.mjs"), "前端全量门禁未接入全局视觉专项验证");

if (failures.length) {
  console.error("全局视觉与交互体系验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}

console.log(`全局视觉与交互体系验证通过：${contract.required_tokens.length} 个核心令牌、${Object.values(contract.required_component_groups).flat().length} 组组件规则、${contract.responsive_breakpoints.length} 个响应式断点、${contract.visual_viewports.length} 个视觉视口与高密度后台规格完整。`);
