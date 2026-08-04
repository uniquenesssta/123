import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };

const layout = read("src/styles/layout.css");
const entityStyles = read("src/styles/entityCenter.css");
const styles = read("src/styles/app.css");
const shell = read("src/app/shell.ts");
const navigation = read("src/app/navigation.ts");
const lineups = read("src/pages/lineups.ts");
const prediction = read("src/pages/prediction.ts");
const runs = read("src/pages/runs.ts");
const teams = read("src/pages/teams.ts");
const main = read("src/main.ts");

requireTrue(entityStyles.includes("--shell-sidebar-expanded") && entityStyles.includes(".app-shell.dual-navigation.sidebar-collapsed"), "双层全局导航没有展开与折叠常驻结构");
requireTrue(shell.includes("primary-rail") && shell.includes("secondary-sidebar") && navigation.includes('key: "management"'), "一级、二级菜单或管理模块结构缺失");
requireTrue(styles.includes(".balanced-page-heading") && styles.includes(".balanced-section-tabs") && styles.includes(".balanced-workspace-main"), "平衡信息密度公共页面结构样式缺失");

requireTrue(lineups.includes("taskPageHeader") && lineups.includes("taskContextRibbon") && lineups.includes("core-local-navigation") && lineups.includes("master-detail-workspace"), "比赛与阵容页没有接入任务型平衡布局");
requireTrue(lineups.includes("balanced-lineup-add") && lineups.includes("balanced-lineup-list") && lineups.includes("加入本次阵容"), "阵容页没有接入下拉选人与纵向阵容列表");
for (const label of ["比赛", "阵容类型", "数据窗口", "记录时间", "来源网址", "球队教练", "本队数据可信度（0–1）"]) {
  requireTrue(lineups.includes(label), `阵容主界面缺少必要字段：${label}`);
}
requireTrue(styles.includes(".balanced-lineup-row") && styles.includes("overflow-x: hidden"), "阵容列表仍可能产生横向滚动");
requireTrue(lineups.includes("open-lineup-player-settings") && main.includes("savePairedLineupPlayerSettings"), "阵容高级字段已隐藏但没有可访问的球员设置入口");
requireTrue(main.includes("lineup-player-expected-minutes") && main.includes("lineup-player-starting-probability"), "球员分钟与首发概率设置链路缺失");

requireTrue(prediction.includes("taskPageHeader") && prediction.includes("taskContextRibbon") && prediction.includes("core-local-navigation") && prediction.includes("master-detail-workspace"), "正式推演页没有接入任务型平衡布局");
for (const label of ["P4/P7 的入口、路由与数据准备链已保留", "外部提供器规则入口", "数据窗口", "参数版本", "正式运行与重新校准分开管理"]) {
  requireTrue(prediction.includes(label), `正式推演主界面缺少必要信息：${label}`);
}
requireTrue(!prediction.includes("P4 Golden Master固定回归规则"), "正式推演页面仍直接暴露 Golden Master 回归资产");

requireTrue(runs.includes("balanced-data-table") && runs.includes("模型 / 规则") && runs.includes("胜平负"), "推演历史没有接入必要信息表格");
requireTrue(teams.includes("entity-directory-list") && teams.includes("entity-data-table roster-table") && teams.includes("当前阵容"), "球队资源中心没有接入目录与高密度阵容表格");
requireTrue(styles.includes(".balanced-data-table") && entityStyles.includes(".entity-data-table") && entityStyles.includes(".entity-directory-list"), "历史或球队高密度表格样式缺失");

requireTrue(main.includes("item.probability >= 0.001") && styles.includes(".threshold-scroll"), "推演比分没有采用 0.1% 阈值和固定滚动区");
requireTrue(!main.includes('data-action="show-route-json">高级矩阵与技术信息在详细设置中查看'), "比分阈值说明错误复用了路由详情动作");
requireTrue(main.includes("request-remove-lineup-history") && main.includes("request-hide-run-history"), "阵容或推演历史删除交互没有接入");

if (failures.length) {
  console.error("第一阶段平衡信息密度 UI 验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("平衡信息密度 UI、固定三级任务导航、阵容选择、公开模型入口、历史和球队目录验证通过。");
