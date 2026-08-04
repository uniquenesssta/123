import { readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const main = read("src/main.ts");
const shell = read("src/app/shell.ts");
const navigation = read("src/app/navigation.ts");
const teams = read("src/pages/teams.ts");
const players = read("src/pages/players.ts");
const lineups = read("src/pages/lineups.ts");
const rules = read("src/pages/rules.ts");
const competition = read("src/components/competition.ts");
const footballText = read("src/components/footballText.ts");
const types = read("src/types.ts");
const domain = read("crates/domain/src/lib.rs");
const playerPersistence = read("crates/persistence-postgres/src/player_catalog.rs");
const teamPersistence = read("crates/persistence-postgres/src/team_catalog.rs");
const css = read("src/styles/entityCenter.css");

check(main.indexOf('import "./styles/entityCenter.css";') > main.indexOf('import "./styles/layout.css";'), "资源中心样式必须最后加载，避免旧阶段样式反向覆盖");
check(shell.includes("primary-rail") && shell.includes("secondary-sidebar") && navigation.includes('key: "management"'), "管理一级模块及上下文二级菜单必须常驻");
check(css.includes("--shell-sidebar-expanded") && css.includes("--shell-sidebar-collapsed"), "双层全局导航缺少展开/折叠双常驻宽度");
check(css.includes(".app-shell.dual-navigation.sidebar-collapsed"), "二级菜单折叠状态样式缺失");

for (const source of [teams, players]) {
  check(source.includes("workspaceSectionNavigation") && source.includes("core-local-navigation"), "球队/球员资源中心缺少固定三级任务导航");
  check(!source.includes("entity-mode-switch"), "球队/球员资源中心仍保留重复的旧模式切换入口");
  check(source.includes("entity-task-workspace"), "复杂编辑任务必须在主工作区直接展开");
  check(source.includes("entity-inspector"), "资源中心缺少不离开列表的速览检查器");
}
check(teams.includes("entity-directory-list") && teams.includes("entity-data-table roster-table"), "球队中心缺少球队目录与阵容主表");
check(players.includes("player-directory-table") && players.includes("clear-player-filters"), "球员中心缺少高密度名单或筛选复位");
check(teams.includes("previous-team-page") && teams.includes("next-team-page"), "球队目录缺少双向游标分页");
check(players.includes("previous-player-page") && players.includes("next-player-page"), "球员目录缺少双向游标分页");
check(main.includes("teamCursorHistory") && main.includes("playerCursorHistory"), "主控制器缺少游标历史，无法返回上一页");
check(main.includes("previewPlayerFromTeam") && main.includes('active_section: "directory"'), "球队阵容点击不得破坏主目录浏览状态");

for (const breakpoint of ["1480px", "1100px", "820px", "620px"]) {
  check(css.includes(`max-width: ${breakpoint}`), `资源中心缺少 ${breakpoint} 响应式断点`);
}
check(css.includes("grid-template-columns: var(--entity-directory-width) minmax(500px, 1fr) var(--entity-inspector-width)"), "宽屏三栏资源浏览布局缺失");
check(css.includes("grid-auto-flow: column") && css.includes("entity-task-workspace"), "中屏目录重排或主工作区详情缺失");
check(css.includes("overflow-x: auto") && css.includes("player-directory-table"), "窄屏表格内部滚动保护缺失");
check(!css.includes("container-type") && !css.includes("@container"), "不得再依赖修改容器自身布局的容器查询");

check(lineups.includes('const current = matches.filter') && lineups.includes('const history = matches.filter'), "比赛目录未拆分当前与历史比赛");
check(lineups.includes('<details class="match-history-group"'), "已结束/取消比赛未默认收纳");
check(css.includes(".match-list-item") && css.includes("min-height: 74px"), "比赛卡片未压缩到紧凑高度");

check(competition.includes("selected: CompetitionKind | null"), "赛事赛制选项不支持无默认选择");
check(rules.includes("competitionKindOptions(null)"), "赛事目录仍可能隐藏选中联赛过滤");
check(main.includes("const baseMatches = (row: HTMLElement)") && main.includes("button.disabled = value !== \"\" && count === 0"), "赛事目录计数与结果未使用同一筛选口径");
check(css.includes(".rules-scope-list button.active") && css.includes(".rules-region-list button.active"), "赛事一级/二级目录缺少精细选中态");

check(footballText.includes('GK: "门将"') && footballText.includes('ST: "前锋"') && footballText.includes("positionLabel"), "球员位置中文映射不完整");
check(footballText.includes("detailLocalizedName") && footballText.includes("hasChineseText"), "中文姓名识别链缺失");
check(types.match(/localized_name: string \| null;/g)?.length >= 2, "前端球队阵容/球员列表缺少中文姓名字段");
check(domain.match(/pub localized_name: Option<String>/g)?.length >= 2, "Rust领域层缺少中文姓名字段");
check(playerPersistence.includes("localized_name.name AS localized_name") && playerPersistence.includes('localized_name: row.try_get("localized_name")?'), "球员目录中文姓名查询或映射缺失");
check(teamPersistence.includes("localized_name.name AS localized_name") && teamPersistence.includes('localized_name: row.try_get("localized_name")?'), "球队阵容中文姓名查询或映射缺失");
check(!teamPersistence.match(/fn team_record_from_row[\s\S]{0,500}localized_name:/), "中文姓名字段被错误写入TeamRecord映射");

if (failures.length) {
  console.error("球队与球员资源中心专项验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("球队与球员资源中心专项验证通过：固定三级任务导航、三栏速览、完整档案页内闭环、分级响应式及中文姓名/位置链路完整。");
