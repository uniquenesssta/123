import { readFileSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outer = resolve(root, "..");
const text = (path) => readFileSync(join(root, path), "utf8");
const json = (path) => JSON.parse(text(path));
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };
const contract = json("contracts/workspace-ui-contract.json");
const pkg = json("package.json");
const shell = text("src/app/shell.ts");
const navigation = text("src/app/navigation.ts");
const state = text("src/app/viewState.ts");
const main = text("src/main.ts");
const styles = `${text("src/styles/layout.css")}
${text("src/styles/app.css")}
${text("src/styles/entityCenter.css")}
${text("src/styles/moduleWorkspaces.css")}`;
const teams = text("src/pages/teams.ts");
const players = text("src/pages/players.ts");
const lineups = text("src/pages/lineups.ts");
const workbooks = text("src/pages/workbooks.ts");
const ai = text("src/pages/apiWorkspace.ts");
const prediction = text("src/pages/prediction.ts");
const release = text("src/pages/release.ts");
const backend = text("src-tauri/src/workspace_state.rs");
const commands = text("src-tauri/src/commands/workspace.rs");
const registry = text("src-tauri/src/bootstrap/command_registry.rs");
const client = text("src/api/client.ts");
const readme = readFileSync(join(root, "README.md"), "utf8");

requireTrue(contract.contract_id === "football.workspace-ui-contract.v1", "阶段6契约ID错误");
requireTrue(contract.contract_version === "1.3.0" && contract.ui_revision === 5, "双层全局导航契约未同步");
requireTrue(pkg.version === contract.release_version, "阶段6版本未同步");
requireTrue(readme.includes("## 0.19.0 变更记录"), "README缺少0.19.0变更记录");
for (const label of contract.global_navigation) requireTrue(navigation.includes(`label: "${label}"`), `一级菜单缺少：${label}`);
for (const labels of Object.values(contract.secondary_navigation ?? {})) {
  for (const label of labels) requireTrue(navigation.includes(`label: "${label}"`), `二级菜单缺少：${label}`);
}
requireTrue(shell.includes("primary-rail") && shell.includes("secondary-sidebar") && shell.includes("toggle-global-sidebar"), "双层全局导航或二级菜单折叠链缺失");
requireTrue(shell.includes('aria-label="一级菜单"') && shell.includes("activeModule.items.map"), "一级菜单与上下文二级菜单未同时常驻");
requireTrue(main.includes('app.querySelector<HTMLElement>(".page-container") ?? app') && !main.includes("workspaceState.restore(page, app)"), "工作区状态恢复必须限定在页面内容区，禁止改写全局导航折叠状态");
requireTrue(state.includes('details:not([data-workspace-persist="false"])'), "工作区详情状态未排除全局导航折叠组件");
requireTrue(state.includes("WorkspaceStateStore") && state.includes("schema_version: 1"), "缺少版本化WorkspaceStateStore");
requireTrue(state.includes("selected_object_ids") && state.includes("panel_widths") && state.includes("internal_scrolls") && state.includes("active_section"), "工作区状态字段不完整");
for (const key of contract.forbidden_state) requireTrue(backend.includes(key) || state.includes(key.replaceAll("_", "[-_]?")) || state.includes("FORBIDDEN_CONTROL_PATTERN"), `缺少敏感字段保护：${key}`);
requireTrue(backend.includes("MAX_WORKSPACE_STATE_BYTES") && backend.includes("write_atomic"), "后端状态文件缺少上限或原子写入");
for (const command of contract.commands) {
  requireTrue(commands.includes(`fn ${command}`), `后端缺少命令：${command}`);
  requireTrue(registry.includes(`commands::${command}`), `Tauri未注册命令：${command}`);
  requireTrue(client.includes(`"${command}"`), `前端缺少命令调用：${command}`);
}
requireTrue(main.includes("openSelectedWorkspaceObjects") && main.includes("setWorkspaceMode") && main.includes("resetCurrentWorkspace"), "主控制器缺少多选、模式或重置链");
requireTrue(main.includes("select-workspace-section") && main.includes("active_section: sectionId"), "主控制器缺少左侧模块切换状态链");
requireTrue(teams.includes("entity-browser") && teams.includes("entity-directory-list") && teams.includes("entity-data-table roster-table") && teams.includes("entity-selection-bar") && teams.includes("永久删除（无引用）"), "球队资源浏览工作区不完整");
requireTrue(players.includes("entity-browser player-browser") && players.includes("player-directory-table") && players.includes("entity-selection-bar") && players.includes("批量删除空对象"), "球员资源浏览工作区不完整");
requireTrue(lineups.includes("balanced-workspace") && lineups.includes("workspaceSectionNavigation") && lineups.includes("data-workspace-section") && lineups.includes("比赛、阵容与模型输入") && lineups.includes("balanced-lineup-add"), "比赛中心平衡布局、固定模块切换或阵容链路缺失");
requireTrue(workbooks.includes("workspaceSectionNavigation") && workbooks.includes("球队月度") && workbooks.includes("球员月度") && workbooks.includes("比赛与阵容") && workbooks.includes("workbook-flow"), "Excel工作包固定模块切换入口不完整");
requireTrue(ai.includes("module-workspace-page ai-module-workspace") && ai.includes("workspaceSectionNavigation") && ai.includes("ai-chat-layout") && ai.includes("ai-history-layout") && ai.includes("会话检查器"), "AI问答页内任务工作区不完整");
requireTrue(prediction.includes("balanced-workspace") && prediction.includes("workspaceSectionNavigation") && prediction.includes("workspace-module-view") && prediction.includes("外部提供器规则入口") && prediction.includes("参数版本"), "赛事推演未统一平衡信息密度排布逻辑");
requireTrue(release.includes("workspaceSectionNavigation") && release.includes("不可变发布验收记录"), "发布验收工作区不完整");
requireTrue(!teams.includes('workspacePaneToggle("module-sidebar"') && !players.includes('workspacePaneToggle("module-sidebar"') && !lineups.includes('workspacePaneToggle("module-sidebar"') && !workbooks.includes('workspacePaneToggle("module-sidebar"') && !ai.includes('workspacePaneToggle("module-sidebar"') && !prediction.includes('workspacePaneToggle("module-sidebar"') && !release.includes('workspacePaneToggle("module-sidebar"'), "桌面模块左栏不得继续提供收起入口");
requireTrue(styles.includes(".workspace-grid") && styles.includes(".workspace-section-nav") && styles.includes(".workspace-module-view.active") && styles.includes(".balanced-workspace") && styles.includes(".balanced-section-tabs") && styles.includes(".task-activity"), "工作区固定导航、平衡模块切换或局部异步样式不完整");
requireTrue(state.includes("ui_revision: 5") && state.includes("active_section: null") && state.includes("inspector_collapsed: true"), "双层导航工作区状态迁移或默认抽屉状态缺失");
requireTrue(styles.includes(".selection-commandbar") && styles.includes(".workspace-inspector") && styles.includes(".balanced-data-table") && styles.includes("overflow-y: auto"), "上下文批量操作、按需检查器或高密度列表滚动样式缺失");
requireTrue(!styles.includes("resize: horizontal"), "模块边栏不得保留原生横向拖拽");
requireTrue(existsSync(join(root, "src/components/icons.ts")), "缺少统一SVG图标组件");
requireTrue(existsSync(join(root, "src/components/workspace.ts")), "缺少工作区共享组件");

if (failures.length) {
  console.error("阶段6工作区契约验证失败：");
  failures.forEach((item) => console.error(`- ${item}`));
  process.exit(1);
}
console.log(`工作区契约验证通过：平衡信息密度模块切换、按需详情、持久状态与命令边界完整。`);
