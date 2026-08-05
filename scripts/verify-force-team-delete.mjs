import fs from "node:fs";

const read = (path) => fs.readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const failures = [];
const requireTrue = (condition, message) => { if (!condition) failures.push(message); };

const migration = read("crates/persistence-postgres/migrations/0028_force_team_purge.sql");
const persistence = read("crates/persistence-postgres/src/team_force_delete.rs");
const persistenceLib = read("crates/persistence-postgres/src/lib.rs");
const domain = read("crates/domain/src/lib.rs");
const application = read("crates/application/src/player_catalog.rs");
const commands = read("src-tauri/src/commands/catalog.rs");
const registry = read("src-tauri/src/bootstrap/command_registry.rs");
const client = read("src/api/client.ts");
const types = read("src/types.ts");
const main = read("src/main.ts");
const teams = read("src/pages/teams.ts");
const styles = read("src/styles/app.css");
const readme = fs.readFileSync(new URL("../README.md", import.meta.url), "utf8");

requireTrue(migration.includes("platform.force_purge_enabled"), "强制清除迁移缺少事务本地开关");
requireTrue(migration.includes("current_setting('football.force_purge', true)"), "强制清除开关不是事务可读设置");
requireTrue(migration.includes("TG_OP = 'DELETE' AND platform.force_purge_enabled()"), "不可变账本没有仅对DELETE开放强制清除");
requireTrue(!migration.includes("TG_OP = 'UPDATE' AND platform.force_purge_enabled()"), "强制清除不得绕过不可变账本UPDATE保护");
for (const guard of [
  "feature.reject_frozen_snapshot_delete",
  "analytics.guard_postmatch_evaluation_sample",
  "review.reject_postmatch_record_mutation",
]) {
  requireTrue(migration.includes(guard), `强制清除迁移未受控覆盖保护函数：${guard}`);
}

requireTrue(persistenceLib.includes("mod team_force_delete;"), "持久化层未注册球队强制清除模块");
requireTrue(persistence.includes("preview_force_delete_team") && persistence.includes("force_delete_team"), "持久化层缺少预检或执行入口");
requireTrue(persistence.includes("FOR UPDATE"), "强制清除未锁定球队主体");
requireTrue(persistence.includes("request.confirmation_text.trim() != label"), "强制清除未要求完整球队名称确认");
requireTrue(persistence.includes("set_config('football.force_purge', 'on', true)"), "强制清除没有使用事务本地维护权限");
requireTrue(persistence.includes("tx.commit().await?"), "强制清除未在事务成功后统一提交");
for (const target of [
  "purge_matches", "purge_players", "purge_coaches", "purge_snapshots",
  "purge_model_runs", "purge_research_runs", "purge_match_reviews",
  "purge_settlements", "purge_import_batches", "purge_ai_sessions",
]) {
  requireTrue(persistence.includes(`CREATE TEMP TABLE ${target}`), `强制清除缺少影响集合：${target}`);
}
for (const table of [
  "review.postmatch_settlements", "analytics.evaluation_samples", "platform.p4_freeze_tasks",
  "model.runs", "feature.snapshots", "research.runs", "review.match_reviews",
  "football.matches", "feature.player_dynamic_tags", "feature.player_ability_observations",
  "football.player_team_periods", "football.team_coach_periods",
  "feature.formation_usage_observations", "feature.team_tactical_observations",
  "feature.team_ability_observations", "catalog.import_batches", "ai_workspace.sessions",
  "football.players", "football.coaches", "football.teams",
]) {
  requireTrue(persistence.includes(`DELETE FROM ${table}`), `强制清除未覆盖表：${table}`);
}
requireTrue(persistence.includes("FROM review.match_events WHERE team_id=$1") && persistence.includes("related_player_id IN (SELECT id FROM purge_players)"), "强制清除影响集合未覆盖结构化比赛事件");
requireTrue(persistence.includes("UNION ALL SELECT 'match_events'") && persistence.includes("match_id IN (SELECT id FROM purge_matches)"), "强制清除预检未统计结构化比赛事件");
requireTrue(persistence.includes("FROM review.team_match_reviews team_review") && persistence.includes("team_review.team_id=$1"), "强制清除未通过球队复盘补齐相关比赛");
requireTrue(persistence.includes("session.metadata::text LIKE") && persistence.includes("DELETE FROM ai_workspace.sessions WHERE id IN"), "强制清除未清理API工作台实体上下文");
requireTrue(persistence.includes("team_force_deleted"), "强制清除完成后缺少最小审计墓碑");

for (const typeName of ["TeamForceDeleteRequest", "TeamForceDeletePreview", "TeamForceDeleteResult"]) {
  requireTrue(domain.includes(`struct ${typeName}`), `领域层缺少：${typeName}`);
  requireTrue(types.includes(`interface ${typeName}`), `前端类型缺少：${typeName}`);
}
for (const command of ["preview_force_delete_team", "force_delete_team"]) {
  requireTrue(commands.includes(`fn ${command}`), `Tauri命令源码缺失：${command}`);
  requireTrue(registry.includes(`commands::${command}`), `Tauri命令未注册：${command}`);
  requireTrue(client.includes(`"${command}"`), `前端API未调用：${command}`);
}
const previewCommand = commands.slice(
  commands.indexOf("pub async fn preview_force_delete_team"),
  commands.indexOf("pub async fn force_delete_team"),
);
const executeCommand = commands.slice(
  commands.indexOf("pub async fn force_delete_team"),
  commands.indexOf("pub async fn create_data_provider"),
);
for (const [label, source] of [["预检", previewCommand], ["执行", executeCommand]]) {
  requireTrue(source.includes("state.service.clone()"), `强制清除${label}命令未取得拥有所有权的服务句柄`);
  requireTrue(source.includes("tauri::async_runtime::handle()"), `强制清除${label}命令未绑定Tauri运行时句柄`);
  const blockingStart = source.indexOf("tauri::async_runtime::spawn_blocking");
  const blockingSource = blockingStart >= 0 ? source.slice(blockingStart) : "";
  requireTrue(source.includes("tauri::async_runtime::spawn_blocking(move ||"), `强制清除${label}命令未隔离非Send长事务Future`);
  requireTrue(
    blockingSource.includes("service.") && !blockingSource.includes("state."),
    `强制清除${label}阻塞事务仍捕获State借用`,
  );
  requireTrue(source.includes("runtime") && source.includes(".block_on("), `强制清除${label}命令未在隔离线程驱动数据库Future`);
}
requireTrue(application.includes("preview_force_delete_team") && application.includes("force_delete_team"), "应用服务未接入强制清除");
requireTrue(teams.includes('data-action="request-force-delete-team"') && teams.includes("强制删除全部资料"), "球队详情缺少强制清除按钮");
requireTrue(main.includes("requestForceDeleteTeam") && main.includes("confirmForceDeleteTeam"), "前端缺少强制清除预检与确认流程");
requireTrue(main.includes("force-delete-team-confirmation") && main.includes("confirmation !== preview.confirmation_text"), "前端没有严格校验完整球队名称");
requireTrue(main.includes('removeWorkspaceObjects("teams", [result.team_id])') && main.includes('removeWorkspaceObjects("players", result.deleted_player_ids)'), "强制清除后未同步移除球队和球员工作区对象");
requireTrue(styles.includes(".force-delete-warning") && styles.includes(".force-delete-impact-grid"), "强制清除对话框缺少危险提示样式");
requireTrue(readme.includes("强制删除全部资料") && readme.includes("football.force_purge"), "README未记录强制清除能力和安全边界");

if (failures.length) {
  console.error("球队强制清除契约验证失败：");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("球队强制清除契约通过：完整名称确认、Tauri非Send事务隔离、事务本地权限、球队/球员/教练/比赛/P4/导入/API上下文清理及工作区同步均已锁定。");
