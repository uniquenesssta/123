import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const navigation = read("src/app/navigation.ts");
const shell = read("src/app/shell.ts");
const viewState = read("src/app/viewState.ts");
const types = read("src/types.ts");
const layout = read("src/styles/layout.css");
const entityStyles = read("src/styles/entityCenter.css");
const contract = JSON.parse(read("contracts/workspace-ui-contract.json"));

const pageUnion = types.match(/export type Page\s*=([\s\S]*?);/)?.[1] ?? "";
const declaredPages = [...pageUnion.matchAll(/\|\s*"([^"]+)"/g)].map((match) => match[1]);
const configuredPages = [...navigation.matchAll(/\{ page: "([^"]+)", label:/g)].map((match) => match[1]);
const duplicatePages = configuredPages.filter((page, index) => configuredPages.indexOf(page) !== index);
const missingPages = declaredPages.filter((page) => !configuredPages.includes(page));
const unknownPages = configuredPages.filter((page) => !declaredPages.includes(page));

check(contract.contract_version === "1.3.0" && contract.ui_revision === 5, "双层导航契约版本或UI修订号错误");
check(navigation.includes("PRIMARY_NAVIGATION") && navigation.includes("navigationModuleForPage") && navigation.includes("navigationItemForPage"), "缺少集中式双层导航配置或页面映射");
check((navigation.match(/key: "/g) ?? []).length === 7, "一级菜单数量必须保持为7个清晰业务模块");
for (const label of ["首页", "比赛", "资源", "模型", "分析", "AI", "管理"]) {
  check(navigation.includes(`label: "${label}"`), `缺少一级菜单：${label}`);
}
check(duplicatePages.length === 0, `页面被重复分配到二级菜单：${duplicatePages.join(", ")}`);
check(missingPages.length === 0, `页面未分配到二级菜单：${missingPages.join(", ")}`);
check(unknownPages.length === 0, `二级菜单包含未知页面：${unknownPages.join(", ")}`);

check(shell.includes('class="primary-rail"') && shell.includes('class="secondary-sidebar"'), "Shell未渲染固定一级栏和上下文二级栏");
check(shell.includes('aria-label="一级菜单"') && shell.includes('data-current-module='), "Shell缺少一级菜单语义或当前模块标识");
check(shell.includes("activeModule.items.map") && shell.includes('aria-current="page"'), "二级菜单未按当前一级模块渲染或缺少活动页语义");
check(shell.includes("secondary-reveal-button") && shell.includes("secondary-collapse-button"), "二级菜单缺少收起与恢复入口");
check(shell.includes("${activeModule.label} / ${activeTitle}"), "顶部面包屑未显示一级/二级层级");
check(!shell.includes("nav-management"), "旧管理折叠菜单仍残留在Shell");

check(layout.includes(".primary-rail") && layout.includes(".secondary-sidebar") && layout.includes(".secondary-nav-item"), "双层导航视觉样式不完整");
check(layout.includes("position: fixed") && layout.includes("--shell-navigation-offset"), "一级与二级菜单未固定或主内容偏移链缺失");
check(layout.includes(".dual-navigation.secondary-collapsed") && layout.includes(".secondary-reveal-button"), "二级菜单折叠态样式不完整");
check(entityStyles.includes(".app-shell.dual-navigation") && !entityStyles.includes(".sidebar .nav-item"), "资源中心仍覆盖旧全局侧栏结构");
check(viewState.includes("ui_revision: 5") && viewState.includes("sidebar_collapsed") && viewState.includes("needsUiMigration ? false"), "工作区状态未迁移到双层导航修订版或首次升级未展开二级菜单");

if (failures.length) {
  console.error("双层全局导航专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log(`双层全局导航专项验证通过：7个一级模块、${configuredPages.length}个二级页面入口均唯一且完整。`);
