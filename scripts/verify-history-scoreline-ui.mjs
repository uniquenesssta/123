import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };

const modal = read("src/app/modal.ts");
const main = read("src/main.ts");
const runs = read("src/pages/runs.ts");
const lineups = read("src/pages/lineups.ts");
const styles = read("src/styles/app.css");
const workspacePanels = read("src/styles/workspacePanels.css");
const client = read("src/api/client.ts");
const predictionCommand = read("src-tauri/src/commands/prediction.rs");
const catalogCommand = read("src-tauri/src/commands/catalog.rs");
const registry = read("src-tauri/src/lib.rs");
const applicationPrediction = read("crates/application/src/prediction.rs");
const applicationCatalog = read("crates/application/src/player_catalog.rs");
const modelPersistence = read("crates/persistence-postgres/src/model_runs.rs");
const catalogPersistence = read("crates/persistence-postgres/src/player_catalog.rs");
const runHistoryMigration = read("crates/persistence-postgres/migrations/0032_model_run_history_visibility.sql");
const lineupHistoryMigration = read("crates/persistence-postgres/migrations/0036_lineup_history_visibility.sql");
const catalogMigration = [
  "crates/persistence-postgres/migrations/0005_pre_match_foundation.sql",
  "crates/persistence-postgres/migrations/0008_user_workflow_and_tuning.sql",
  "crates/persistence-postgres/migrations/0033_competition_catalog.sql",
  "crates/persistence-postgres/migrations/0034_mainstream_competition_catalog.sql",
  "crates/persistence-postgres/migrations/0037_comprehensive_competition_catalog.sql",
].map(read).join("\n");
const positionMigration = read("crates/persistence-postgres/migrations/0035_complete_position_catalog.sql");
const postgresIntegration = read("crates/persistence-postgres/tests/postgres_integration.rs");

requireTrue(modal.includes("panelClass") && main.includes('"prediction-result-modal"'), "推演结果右侧工作区没有独立样式类");
requireTrue(workspacePanels.includes(".workspace-detail-page.prediction-result-modal") && styles.includes("threshold-scroll"), "推演结果右侧工作区或固定高度滚动比分框样式缺失");
requireTrue(main.includes("item.probability >= 0.001") && main.includes("概率不低于 0.1% 的比分"), "比分列表没有按 0.1% 阈值过滤");
requireTrue(!main.includes(".slice(0, 5)"), "比分列表仍被固定截断为前五项");
requireTrue(main.includes("cumulativeProbability") && main.includes("累计"), "比分列表缺少累计概率");

requireTrue(runHistoryMigration.includes("history_hidden_at") && runHistoryMigration.includes("history_hidden_reason"), "推演历史软删除迁移缺失");
requireTrue(modelPersistence.includes("hide_run_from_history") && modelPersistence.includes("history_hidden_at IS NULL"), "推演历史软删除持久化链缺失");
for (const source of [applicationPrediction, predictionCommand, registry, client]) {
  requireTrue(source.includes("hide_model_run_history") || source.includes("hide_run_from_history"), "推演历史删除命令链未贯通");
}
requireTrue(runs.includes('data-context-kind="run"') && main.includes('document.addEventListener("contextmenu"'), "推演历史右键删除入口缺失");

requireTrue(lineupHistoryMigration.includes("history_hidden_at") && lineupHistoryMigration.includes("history_hidden_reason"), "阵容历史隐藏/归档迁移缺失");
requireTrue(catalogPersistence.includes("remove_lineup_history") && catalogPersistence.includes("restored_lineup_id"), "阵容历史删除、归档和活动版本恢复持久化缺失");
for (const source of [applicationCatalog, catalogCommand, registry, client]) {
  requireTrue(source.includes("remove_lineup_history"), "阵容历史删除命令链未贯通");
}
requireTrue(lineups.includes('data-context-kind="lineup"') && lineups.includes('data-action="request-remove-lineup-history"'), "阵容历史缺少可见删除和右键入口");
requireTrue(postgresIntegration.includes("集成测试删除未引用当前版本") && postgresIntegration.includes("集成测试归档已引用版本"), "阵容历史删除、恢复和归档缺少 PostgreSQL 集成回归");

requireTrue(lineups.includes('data-context-kind="match"') && lineups.includes('data-action="request-delete-match"'), "已创建比赛缺少可见删除和右键删除入口");
requireTrue(catalogPersistence.includes("protected_count") && catalogPersistence.includes("P4研究、冻结或正式赛后结算"), "比赛删除未保护不可变P4/赛后血缘");
requireTrue(catalogPersistence.includes("UPDATE ai_workspace.sessions SET match_id = NULL"), "比赛删除未解除AI会话的可空比赛引用");
requireTrue(main.includes('kind !== "run" && kind !== "match" && kind !== "lineup"') && main.includes("showAppContextMenu"), "应用右键菜单未覆盖推演、比赛和阵容历史");

for (const id of ["new-match-competition-scope", "new-match-competition-region", "new-match-competition"]) {
  requireTrue(lineups.includes(id), `赛事三级菜单缺失：${id}`);
}
for (const id of ["formation-level1", "formation-level2", "formation-id"]) {
  requireTrue(lineups.includes(id), `阵型三级菜单缺失：${id}`);
}
requireTrue(main.includes("updateCompetitionHierarchy") && main.includes("updateFormationHierarchy"), "三级菜单没有级联过滤控制器");
requireTrue(styles.includes(".hierarchy-selector") && styles.includes(".hierarchy-level"), "三级菜单视觉层级缺失");

for (const code of [
  "FIFA-WORLD-CUP", "UEFA-CHAMPIONS-LEAGUE", "UEFA-EUROPA-LEAGUE", "ENG-PREMIER-LEAGUE",
  "ENG-CHAMPIONSHIP", "ES-LALIGA", "IT-SERIE-A", "DE-BUNDESLIGA", "FR-LIGUE-1",
  "PT-PRIMEIRA-LIGA", "NL-EREDIVISIE", "BE-PRO-LEAGUE", "NO-ELITESERIEN", "FI-VEIKKAUSLIIGA",
  "US-MLS", "CA-PREMIER-LEAGUE", "BR-SERIE-A", "AR-PRIMERA", "CONMEBOL-LIBERTADORES",
]) {
  requireTrue(catalogMigration.includes(code), `内置主流赛事目录缺少：${code}`);
}
requireTrue(catalogMigration.includes("season_pattern") && catalogMigration.includes("menu_region"), "赛事目录缺少赛季模式或三级菜单元数据");
requireTrue(catalogPersistence.includes("AT TIME ZONE timezone") && catalogPersistence.includes("automatic_season_identity"), "赛季未按赛事本地时间自动判断/创建");

for (const code of ["SW", "LCB", "RCB", "LWB", "RWB", "LDM", "RDM", "LCM", "RCM", "LAM", "RAM", "SS", "CF", "LST", "RST"]) {
  requireTrue(positionMigration.includes(`'${code}'`) || positionMigration.includes(`\"${code}\"`), `完整位置目录缺少：${code}`);
}
requireTrue(lineups.includes("球队教练") && lineups.includes("本队数据可信度（0–1）") && lineups.includes("双方共用比赛"), "双方阵容缺少独立教练、可信度或统一版本信息");
requireTrue(lineups.includes("balanced-lineup-add") && lineups.includes("加入本次阵容") && lineups.includes("balanced-lineup-list"), "阵容编排没有采用下拉选人和纵向紧凑列表");

if (failures.length) {
  console.error("第一阶段比分、历史、赛事、位置与阵容链验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("第一阶段比分阈值、推演/阵容/比赛删除、三级目录、主流赛事、完整位置和本地赛季链验证通过。");
