import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };

const migration = read("crates/persistence-postgres/migrations/0037_comprehensive_competition_catalog.sql");
const rules = read("src/pages/rules.ts");
const lineups = read("src/pages/lineups.ts");
const main = read("src/main.ts");
const styles = read("src/styles/app.css");
const layout = read("src/styles/layout.css");
const modal = read("src/app/modal.ts");
const loaders = read("src/controllers/pageLoaders.ts");

const catalogCount = (migration.match(/'catalog_version', '2026\.07-stage2'/g) ?? []).length;
requireTrue(catalogCount >= 350, `完整内置赛事目录不足 350 项，实际 ${catalogCount} 项`);
for (const scope of ["'scope', 'national'", "'scope', 'club'"]) {
  requireTrue(migration.includes(scope), `赛事目录缺少范围：${scope}`);
}
for (const confederation of ["FIFA", "UEFA", "CONMEBOL", "CONCACAF", "AFC", "CAF", "OFC"]) {
  requireTrue(migration.includes(`'confederation', '${confederation}'`), `赛事目录缺少足联：${confederation}`);
}
for (const marker of [
  "FIFA-WORLD-CUP", "UEFA-CHAMPIONS-LEAGUE", "CONMEBOL-LIBERTADORES",
  "CONCACAF-CHAMPIONS-CUP", "AFC-CHAMPIONS-LEAGUE-ELITE", "CAF-CHAMPIONS-LEAGUE",
  "OFC-CHAMPIONS-LEAGUE", "ENG-PREMIER-LEAGUE", "西班牙甲级联赛", "德国甲级联赛",
  "意大利甲级联赛", "法国甲级联赛", "美国职业足球大联盟", "墨西哥超级联赛",
  "巴西甲级联赛", "阿根廷甲级联赛", "JP-J1", "KR-KLEAGUE1",
]) {
  requireTrue(migration.includes(marker), `完整赛事目录缺少关键赛事：${marker}`);
}
requireTrue(migration.includes("ON CONFLICT (code) DO UPDATE") && migration.includes("football.competitions.metadata || EXCLUDED.metadata"), "完整赛事迁移未采用非破坏性合并");
requireTrue(migration.includes("season_pattern") && migration.includes("menu_region") && migration.includes("sort_order"), "完整赛事迁移缺少三级目录或赛季元数据");

requireTrue(rules.includes("data-rules-directory") && rules.includes("rules-directory-grid") && rules.includes("rules-table-scroll"), "赛事设置没有改成三级紧凑目录");
requireTrue(!rules.includes('<details class="catalogue-country"'), "赛事设置仍使用整页巨型国家折叠卡片");
for (const marker of ["data-rules-scope", "data-rules-region", "data-rules-competition-row", "rules-competition-search", "rules-competition-kind"]) {
  requireTrue(rules.includes(marker), `赛事目录筛选结构缺失：${marker}`);
}
requireTrue(main.includes("filterRulesCompetitionCatalogue") && main.includes("selectRulesDirectoryLevel"), "赛事三级目录缺少前端筛选控制器");
requireTrue(lineups.includes("competitionRegionOrder") && lineups.includes("全球 / FIFA") && lineups.includes("大洋洲 / OFC") && lineups.includes("国家队与俱乐部赛事严格分开"), "新建比赛的赛事地区/足联菜单或国家队/俱乐部隔离缺失");
requireTrue(main.includes("competition.metadata?.scope") && main.includes("inferCompetitionTeamScope"), "比赛球队范围没有优先读取赛事目录元数据");

requireTrue(layout.includes(".app-shell.workspace-page .balanced-workspace {") && layout.includes("grid-template-rows: auto minmax(0, 1fr);") && layout.includes("flex: 1 1 auto;"), "平衡工作区没有取得页面剩余高度");
requireTrue(layout.includes(".app-shell.workspace-page .balanced-workspace-main > .workspace-module-view.active") && layout.includes("overflow-y: auto;") && layout.includes("scrollbar-gutter: stable;"), "平衡工作区活动内容没有独立纵向滚动");
requireTrue(layout.includes('data-workspace-section="matches"') && layout.includes(".match-browser-detail {") && layout.includes("grid-template-rows: auto auto minmax(0, 1fr) auto;"), "比赛中心左右分栏没有独立滚动所有权");
requireTrue(layout.includes("scrollbar-width: none;") && layout.includes("::-webkit-scrollbar") && layout.includes("display: none;"), "业务滚动能力未保留为隐藏滚动条模式");
requireTrue(layout.includes(".app-shell.workspace-page .balanced-section-tabs {") && layout.includes("position: static;") && layout.includes("top: auto;"), "顶部模块导航仍使用粘性定位压住业务内容");
requireTrue(layout.includes("max-width: none") && layout.includes("clamp(16px, 1.45vw, 28px)"), "页面容器没有按窗口宽度自适应");
requireTrue(styles.includes("height: clamp(280px, calc(100dvh - 545px), 620px)") && styles.includes("overscroll-behavior-y: contain"), "阵容列表没有按窗口高度自适应滚动");
requireTrue(layout.includes("font-size: clamp(16px, .92vw, 18px)") && layout.includes("font-size: 15px;") && layout.includes("font-size: 14px;"), "业务界面没有建立新的可读字号下限");
requireTrue(styles.includes("height: clamp(450px, calc(100dvh - 300px), 720px)") && styles.includes(".rules-competition-table th"), "赛事设置目录没有自适应内部滚动与固定表头");
requireTrue(styles.includes("@media (max-height: 780px)") && styles.includes("@media (min-width: 1700px)"), "全局UI缺少窗口宽度和高度响应规则");

requireTrue(modal.includes("panelClass = \"\"") && modal.includes("safeClass") && modal.includes("workspace-detail-page ${entry.panelClass}"), "自定义详情页尺寸类没有贯通右侧工作区控制器");
requireTrue(loaders.includes("readonly coaches: CoachListItem[]") && loaders.includes("api.listCoaches"), "阵容页面加载器没有同步教练列表");
requireTrue(main.includes('target.id === "new-match-kickoff" && target instanceof HTMLInputElement'), "开球时间改变时没有触发赛季自动判断");

if (failures.length) {
  console.error("第二阶段自适应UI与完整赛事目录验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log(`第二阶段自适应滚动、隐藏滚动条、非重叠模块导航、可读字号与 ${catalogCount} 项完整内置赛事目录验证通过。`);
