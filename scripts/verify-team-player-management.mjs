import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { isVersionAtLeast } from "./version.mjs";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outerRoot = resolve(root, "..");
const failures = [];
const assert = (condition, message) => { if (!condition) failures.push(message); };
const text = (path) => readFileSync(join(root, path), "utf8").replace(/\r\n?/g, "\n");
const json = (path) => JSON.parse(text(path));
const hash = (path) => createHash("sha256").update(text(path), "utf8").digest("hex");

const contractPath = "contracts/team-player-management-contract.json";
const contract = json(contractPath);
const packageJson = json("package.json");
const tauri = json("src-tauri/tauri.conf.json");
const workspaceCargo = text("Cargo.toml");
const cargoLock = text("Cargo.lock");
const migration = text("crates/persistence-postgres/migrations/0020_team_player_management.sql");
const domain = text("crates/domain/src/lib.rs");
const appCatalog = text("crates/application/src/player_catalog.rs");
const persistenceCatalog = text("crates/persistence-postgres/src/team_catalog.rs");
const playerPersistence = text("crates/persistence-postgres/src/player_catalog.rs");
const spreadsheetPersistence = text("crates/persistence-postgres/src/spreadsheet_exchange.rs");
const spreadsheetIo = text("crates/spreadsheet-io/src/lib.rs");
const commands = text("src-tauri/src/commands/catalog.rs");
const registry = text("src-tauri/src/bootstrap/command_registry.rs");
const client = text("src/api/client.ts");
const shell = text("src/app/shell.ts");
const navigation = text("src/app/navigation.ts");
const main = text("src/main.ts");
const teamsPage = text("src/pages/teams.ts");
const playersPage = text("src/pages/players.ts");
const types = text("src/types.ts");
const styles = text("src/styles/components.css");
const entityStyles = text("src/styles/entityCenter.css");
const readme = text("README.md");

assert(contract.contract_id === "football.team-player-management-contract.v1", "球队与球员管理契约ID错误");
assert(contract.contract_version === "1.0.0", "球队与球员管理契约版本错误");
assert(contract.baseline_source_version === "0.13.1", "球队与球员管理基线必须是0.13.1");
assert(contract.release_version === "0.13.2", "球队与球员管理发布版本必须是0.13.2");
assert(contract.stage === "G_PRE_H", "本次仍必须保持H前置阶段");
assert(isVersionAtLeast(packageJson.version, "0.15.0"), "当前项目版本早于H前置阶段2版本0.15.0");
assert(tauri.version === packageJson.version, "Tauri版本未同步");
assert(workspaceCargo.includes(`version = "${packageJson.version}"`), "Cargo workspace版本未同步");
assert(cargoLock.includes(`name = "football-application"\nversion = "${packageJson.version}"`), "Cargo.lock本地包版本未同步");
assert(readme.includes(`当前版本 **${packageJson.version}**`), "根README当前版本未同步");
assert(readme.includes("## 0.13.2 变更记录"), "根README缺少0.13.2变更记录");
assert(readme.includes("## 0.13.3 变更记录"), "根README缺少0.13.3变更记录");
assert(readme.includes("## 0.13.4 变更记录"), "根README缺少0.13.4变更记录");
assert(readme.includes("## 0.13.5 变更记录"), "根README缺少0.13.5变更记录");
assert(readme.includes("## 0.14.0 变更记录"), "根README缺少0.14.0变更记录");
assert(readme.includes("## 0.15.0 变更记录"), "根README缺少0.15.0变更记录");

for (const artifact of contract.artifacts) {
  assert(existsSync(join(root, artifact)), `球队与球员管理制品不存在：${artifact}`);
}
const contractHash = hash(contractPath);
assert(migration.includes(`TEAM_PLAYER_CONTRACT_SHA256 = ${contractHash}`), "0020迁移顶部球队契约哈希错误");
assert(migration.includes(`'${contractHash}'`), "0020迁移登记球队契约哈希错误");
assert(migration.includes("CREATE TABLE football.team_profiles"), "0020缺少球队档案表");
assert(migration.includes("operation_proposals_operation_type_check"), "0020未升级API操作类型约束");
assert(migration.includes("'add_team_name'"), "0020未允许球队别名提案");
assert(migration.includes("'update_team_profile'"), "0020未允许球队档案提案");
assert(migration.includes("'api_workspace'") && migration.includes("player_dynamic_tags_source_type_check"), "0020未允许人工确认后的API动态标签来源");

for (const command of contract.commands) {
  assert(commands.includes(`fn ${command}`), `Tauri命令缺少${command}`);
  assert(registry.includes(`commands::${command}`), `Tauri注册表缺少${command}`);
  assert(client.includes(`"${command}"`), `前端客户端缺少${command}`);
}
assert(domain.includes("pub struct TeamListQuery"), "领域层缺少球队列表查询");
assert(domain.includes("pub struct TeamProfileDraft"), "领域层缺少球队档案草稿");
assert(domain.includes("pub struct TeamDetail"), "领域层缺少球队详情");
assert(domain.includes("pub struct BulkDeleteResult"), "领域层缺少批量删除结果");
assert(appCatalog.includes("pub async fn list_teams"), "应用层缺少球队列表入口");
assert(appCatalog.includes("pub async fn bulk_delete_players"), "应用层缺少球员批量删除入口");
assert(appCatalog.includes("pub async fn bulk_delete_teams"), "应用层缺少球队批量删除入口");
assert(persistenceCatalog.includes("FROM football.team_profiles"), "球队详情未读取球队档案");
assert(persistenceCatalog.includes("football.player_team_periods"), "球队详情未读取当前阵容");
assert(persistenceCatalog.includes("review.team_match_reviews"), "球队删除未保护赛后复盘历史");
assert(persistenceCatalog.includes("football.matches WHERE home_team_id=$1 OR away_team_id=$1"), "球队删除未保护比赛历史");
assert(persistenceCatalog.includes("team_deleted"), "球队删除缺少审计事件");
assert(playerPersistence.includes("player_deleted"), "批量球员删除必须复用已有审计删除链");

assert(spreadsheetPersistence.includes("_auto_create_team"), "球员球队履历导入缺少自动建队标记");
assert(spreadsheetPersistence.includes("resolve_or_create_import_team"), "球员球队履历导入缺少自动建队事务");
assert(spreadsheetPersistence.includes("pg_advisory_xact_lock"), "自动建队缺少并发锁");
assert(spreadsheetPersistence.includes("FROM football.team_names alias") && spreadsheetPersistence.includes("alias.normalized_name = $1"), "提交时未再次核对球队别名");
assert(spreadsheetPersistence.includes("team_created_from_player_import"), "自动建队缺少审计事件");
assert(spreadsheetPersistence.includes("匹配到多条记录，不能自动创建或关联"), "球队名称歧义未阻断导入");
assert(spreadsheetIo.includes("自动创建球队") && spreadsheetIo.includes("同名多条"), "Excel说明页未解释自动建队与歧义阻断规则");

assert(navigation.includes('key: "ai"') && navigation.includes('page: "api_workspace"') && navigation.includes('label: "AI 问答"'), "AI问答未进入AI一级模块");
assert(navigation.includes('key: "resources"') && navigation.includes('page: "teams"') && navigation.includes('label: "球队中心"'), "球队中心未进入资源一级模块");
assert(navigation.includes('page: "players"') && navigation.includes('label: "球员中心"'), "球员中心未作为资源二级入口");
assert(shell.includes("primary-rail") && shell.includes("secondary-sidebar"), "球队与球员管理未接入双层全局导航");
assert(types.includes('| "teams"'), "页面类型缺少球队中心");
assert(main.includes('case "teams"'), "主路由缺少球队中心");
assert(teamsPage.includes("球队基础身份"), "球队中心缺少身份编辑");
assert(teamsPage.includes("球队档案与战术") && teamsPage.includes("能力"), "球队中心缺少战术和能力档案");
assert(teamsPage.includes("当前阵容"), "球队中心缺少当前阵容");
assert(teamsPage.includes("近期比赛"), "球队中心缺少近期比赛");
assert(teamsPage.includes("进入 AI 问答"), "球队中心缺少AI问答入口");
assert(playersPage.includes("批量删除"), "球员中心缺少批量删除入口");
assert(teamsPage.includes("永久删除（无引用）"), "球队与人员工作区缺少无引用永久删除入口");
assert(main.includes("openTeamApiWorkspace"), "球队未与AI问答联动");
assert(main.includes("openPlayerApiWorkspace"), "球员未与AI问答联动");
const apiWorkspaceApplication = text("crates/application/src/api_workspace.rs");
assert(apiWorkspaceApplication.includes("let current = self.read_team(team_id).await?.profile"), "API球队档案提案未读取现有档案");
assert(apiWorkspaceApplication.includes("or_else(|| current.as_ref()"), "API球队档案提案未采用增量合并");
assert(entityStyles.includes(".entity-browser") && entityStyles.includes("@media (max-width: 1100px)") && entityStyles.includes("@media (max-width: 620px)"), "球队与球员资源中心缺少分级响应式布局");
assert(entityStyles.includes("--shell-sidebar-expanded") && entityStyles.includes(".app-shell.dual-navigation.sidebar-collapsed"), "双层全局导航缺少展开与折叠常驻状态");

assert(contract.team_center.organization_model === "generic_fm_fifa_inspired_without_proprietary_data", "球队组织逻辑必须是通用借鉴而非专有复制");
assert(contract.bulk_delete.explicit_confirmation_required === true, "批量删除必须显式确认");
assert(contract.bulk_delete.historical_match_team_delete_blocked === true, "球队历史比赛删除保护必须开启");
assert(contract.spreadsheet_import.ambiguous_team_match_blocks_commit === true, "球队名称歧义必须阻断提交");
assert(!main.includes("delete without confirmation"), "不得出现绕过确认的批量删除路径");

if (failures.length) {
  console.error("球队与球员管理契约验证失败：");
  for (const failure of failures) console.error(`- ${failure}`);
  process.exit(1);
}
console.log("球队与球员管理验证通过：统一主导航、双向目录、Excel自动关联、AI只读问答联动和受保护批量删除均已锁定。");
