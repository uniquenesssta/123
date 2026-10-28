import { readFileSync, existsSync } from "node:fs";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(resolve(root, path), "utf8");

const domain = (read("crates/domain/src/lib.rs") + read("crates/domain/src/lineup/kind.rs") + read("crates/domain/src/lineup/player.rs") + read("crates/domain/src/lineup/snapshot.rs") + read("crates/domain/src/lineup/preset.rs") + read("crates/domain/src/lineup/chain.rs") + read("crates/domain/src/match_record/status.rs") + read("crates/domain/src/match_record/catalog.rs"));
const teamRecordMapper = read("crates/persistence-postgres/src/adapters/catalog/teams/detail/record_mapper.rs");
const playerRecordMapper = read("crates/persistence-postgres/src/adapters/catalog/players/record/mapper.rs");
const connection = read("crates/persistence-postgres/src/migrations/reset_to_pristine.rs");
const integration = read("crates/persistence-postgres/tests/postgres_integration.rs");
const command = [read("src-tauri/src/commands/database/reset/command.rs"), read("src-tauri/src/commands/database/reset/coordinator.rs")].join("\n");
const compactCommand = command.replace(/\s+/g, "");
const databaseFacade = read("crates/application/src/services/database/facade.rs");
const databaseService = read("crates/application/src/services/database/service.rs");
const resetUseCase = read("crates/application/src/use_cases/database/reset/mod.rs");
const lifecycleReset = read("crates/application/src/use_cases/application_facade/database_lifecycle/reset.rs");
const p4Facade = read("crates/application/src/services/p4_orchestration/facade.rs");
const tauri = read("src-tauri/src/bootstrap/command_registry.rs");
const client = read("src/api/client.ts");
const page = read("src/pages/database.ts");
const main = read("src/main.ts");

function requireText(source, token, label) {
  if (!source.includes(token)) throw new Error(`数据库彻底清空契约缺失：${label}`);
}

function functionBody(source, name) {
  const start = source.indexOf(`fn ${name}`);
  if (start < 0) throw new Error(`未找到 Rust 函数：${name}`);
  const brace = source.indexOf("{", start);
  let depth = 0;
  for (let index = brace; index < source.length; index += 1) {
    if (source[index] === "{") depth += 1;
    if (source[index] === "}") depth -= 1;
    if (depth === 0) return source.slice(start, index + 1);
  }
  throw new Error(`Rust 函数未闭合：${name}`);
}

const teamRecord = domain.match(/pub struct TeamRecord\s*\{([\s\S]*?)\n\}/)?.[1] ?? "";
const playerRecord = domain.match(/pub struct PlayerRecord\s*\{([\s\S]*?)\n\}/)?.[1] ?? "";
if (/localized_name/.test(teamRecord) || /localized_name/.test(playerRecord)) {
  throw new Error("TeamRecord / PlayerRecord 不应重复承载本地化姓名；本地化姓名属于列表或别名读取模型");
}
const teamMapper = functionBody(teamRecordMapper, "map_team_record");
const playerMapper = functionBody(playerRecordMapper, "map_player_record");
if (/localized_name/.test(teamMapper) || /localized_name/.test(playerMapper)) {
  throw new Error("基础 TeamRecord / PlayerRecord 行映射仍写入不存在的 localized_name 字段");
}

requireText(connection, "pub async fn reset_to_pristine", "持久化层彻底重建入口");
for (const schema of [
  "ai_workspace",
  "analytics",
  "audit",
  "catalog",
  "feature",
  "football",
  "model",
  "platform",
  "research",
  "review",
]) {
  requireText(connection, `\"${schema}\"`, `清理应用 schema：${schema}`);
}
requireText(connection, "DROP TABLE IF EXISTS public._sqlx_migrations", "清除迁移账本");
requireText(connection, "self.migrate().await", "从迁移重新建立空白结构");
requireText(connection, "football-platform-destructive-reset", "数据库级互斥锁");
requireText(integration, "destructive_reset_rebuilds_an_empty_migrated_database", "专用数据库集成测试");
requireText(integration, "assert_eq!(team_count, 0)", "清空后业务数据为空断言");

requireText(command, "pub async fn reset_database", "Tauri 清空命令");
requireText(compactCommand, "state.service.preflight_database_reset(&options,&confirmation)", "后端数据库名称二次校验");
requireText(compactCommand, ".reset_database(options,confirmation)", "Tauri 委托 Application 清空用例");
if (command.includes("PostgresStore::connect") || command.includes("reset_store.reset_to_pristine")) {
  throw new Error("Tauri 数据库命令不得直接连接 PostgreSQL 或执行彻底重建");
}
requireText(databaseService, "self.disconnect().await?", "清空前停止活动数据库服务");
requireText(resetUseCase, "confirmation.trim() != health.database_name", "Application 清空用例数据库名称强确认");
requireText(resetUseCase, "保存的连接配置与当前数据库不一致，已拒绝清空", "清空目标一致性门禁");
requireText(resetUseCase, "port.reset_to_pristine().await", "通过 DatabaseLifecyclePort 执行彻底重建");
requireText(lifecycleReset, "connect::execute(application, options)", "清空后自动恢复连接");
requireText(command, "state.service.ensure_p4_orchestration_worker()", "清空后恢复P4后台工作器");
requireText(p4Facade, "pub fn ensure_p4_orchestration_worker", "P4后台工作器幂等恢复入口");
requireText(tauri, "commands::reset_database", "Tauri 命令注册");

requireText(client, 'invoke("reset_database", { confirmation })', "前端 API 调用");
requireText(page, 'data-action="request-reset-database"', "数据库页危险按钮");
requireText(main, "requestDatabaseReset", "输入数据库名称确认弹窗");
requireText(main, '"确认永久清空",executeDatabaseReset', "确认弹窗绑定原清空动作");
requireText(read("src/ui/overlays/confirmation-dialog/confirmationDialogView.ts"), "confirmationInputView(", "唯一强确认输入组件");
requireText(main, 'id:"database-reset-confirmation"', "真实数据库名称强确认配置");
requireText(main, 'confirmation !== expected', "业务执行前仍独立校验名称");
requireText(read("src/app/modal/workspacePanelController.ts"), "mountConfirmationInput(confirmation, confirmButton).destroy", "确认输入随工作区生命周期销毁");
requireText(main, "clear: () => api.clearWorkspaceState()", "唯一工作区清理adapter");
requireText(main, "await workspaceState.clearAll();\n  workspaceState.stop();", "清空后串行移除失效工作区引用并停止退出快照回写");
requireText(main, "window.location.reload()", "清空后重启界面状态");

const databaseRoot = "src-tauri/src/commands/database";
const definitions = {
  configure_database: "configure/command.rs",
  disconnect_database: "disconnect_database.rs",
  reset_database: "reset/command.rs",
};
const commandContract = JSON.parse(read("architecture/command-contract.json"));
const databaseGroup = commandContract.command_groups.find(group => group.module === "database");
const boundaries = JSON.parse(read("architecture/module-boundaries.json"));
if (existsSync(resolve(root, "src-tauri/src/commands/database.rs"))) throw new Error("旧数据库命令owner仍存在");
requireText(read("src-tauri/src/commands.rs"), "mod database;", "数据库命令原根登记");
if (databaseGroup?.definition_file !== `${databaseRoot}/mod.rs` || boundaries.tauri_commands?.groups?.database !== `${databaseRoot}/mod.rs`) throw new Error("数据库职责清单未指向唯一出口");
for (const file of ["mod.rs", "configure/mod.rs", "reset/mod.rs"]) {
  if (/\b(?:fn|struct|impl|enum)\s/.test(read(`${databaseRoot}/${file}`))) throw new Error(`数据库出口包含业务：${file}`);
}
const signatures = {
  configure_database: "pub async fn configure_database(state: State<'_, AppState>, options: DatabaseOptions) -> Result<DatabaseHealth, String>",
  disconnect_database: "pub async fn disconnect_database(state: State<'_, AppState>) -> Result<(), String>",
  reset_database: "pub async fn reset_database(state: State<'_, AppState>, confirmation: String) -> Result<DatabaseHealth, String>",
};
const compact = text => text.replace(/,\s*\)/g, ")").replace(/\s+/g, "");
for (const [name, file] of Object.entries(definitions)) {
  const source = read(`${databaseRoot}/${file}`);
  const signature = source.match(new RegExp(`pub async fn ${name}\\([\\s\\S]*?\\{`))?.[0].slice(0, -1) ?? "";
  if (compact(signature) !== compact(signatures[name])) throw new Error(`数据库公共签名漂移：${name}`);
  const delegate = name === "disconnect_database" ? "execute(&state).await" : `coordinator::execute(&state, ${name === "configure_database" ? "options" : "confirmation"}).await`;
  requireText(compact(source), compact(delegate), `数据库实际命令委托：${name}`);
  if (databaseGroup?.command_definition_files?.[name] !== `${databaseRoot}/${file}`) throw new Error(`数据库命令定义归属漂移：${name}`);
  const exit = read(`${databaseRoot}/mod.rs`);
  for (const token of [`__cmd__${name}`, `__tauri_command_name_${name}`]) requireText(exit, token, `数据库Tauri宏出口：${token}`);
  if (file.includes("/")) {
    const groupExit = read(`${databaseRoot}/${file.split("/")[0]}/mod.rs`);
    for (const token of [`__cmd__${name}`, `__tauri_command_name_${name}`]) requireText(groupExit, token, `数据库子模块宏出口：${token}`);
  }
}
function ordered(source, tokens, label) {
  const text = compact(source);
  let previous = -1;
  for (const token of tokens) {
    const position = text.indexOf(compact(token));
    if (position < 0 || position <= previous) throw new Error(`数据库原顺序漂移：${label}/${token}`);
    previous = position;
  }
}
const configure = read(`${databaseRoot}/configure/coordinator.rs`);
ordered(configure, ['"connection_attempt_started"', '.connect_database(options.clone())', '"connection_attempt_succeeded"', '.save(&state.config_path)', 'state.service.disconnect_database().await', 'return Err(error);'], "连接/保存失败断开");
requireText(configure, '"connection_attempt_failed"', "连接失败原日志");
const disconnect = functionBody(read(`${databaseRoot}/disconnect_database.rs`), "execute");
ordered(disconnect, ['DesktopConfig::default().save(&state.config_path)?', 'state.service.disconnect_database().await', 'Ok(())'], "先清配置再断开");
const reset = read(`${databaseRoot}/reset/coordinator.rs`);
ordered(reset, ['DesktopConfig::load(&state.config_path)?', '.ok_or_else(', '.preflight_database_reset(&options, &confirmation)', '"destructive_reset_started"', '.reset_database(options, confirmation)', 'tokio::task::yield_now().await', 'state.service.ensure_p4_orchestration_worker()', '"destructive_reset_succeeded"'], "强确认/重置/工作器恢复");
requireText(reset, 'message.starts_with("彻底清空数据库失败：")', "仅原重置阶段错误写critical");
requireText(reset, '"destructive_reset_failed"', "重置失败日志");
for (const source of [configure, disconnect, reset]) {
  requireText(source, "state: &AppState", "保留原AppState服务Arc句柄");
  if (/sqlx::|PostgresStore::connect|reset_to_pristine\(|static\s/.test(source)) throw new Error("数据库命令协调器越过原服务/状态边界");
}
for (const source of [configure, reset]) if ((source.match(/let _\s*=\s*state\.runtime_log\.record\(/g) ?? []).length !== 3) throw new Error("数据库诊断必须保持原三次尽力写入，不得变为业务失败");
if (!/pub async fn reset_database\(\s*self: &Arc<Self>,/.test(databaseFacade)) throw new Error("重置必须保留原ApplicationService的Arc接收者");
requireText(read("src-tauri/src/bootstrap/state.rs"), "pub service: Arc<ApplicationService>", "唯一AppState原服务句柄");
const tests = read(`${databaseRoot}/tests.rs`);
for (const name of ["configure_parse_failure_keeps_config_and_original_error_events", "configure_logging_failure_does_not_replace_connection_error", "disconnect_clears_saved_metadata_and_credentials_through_original_store", "disconnect_save_failure_preserves_credentials_and_returns_file_error", "reset_missing_or_empty_saved_connection_stops_before_backend_and_logs", "reset_config_errors_preserve_original_strings_without_reset_side_effects", "reset_missing_credential_stops_before_reset_events_and_keeps_metadata", "reset_preflight_parse_failure_keeps_config_without_destructive_events"]) requireText(tests, `fn ${name}(`, `原Tauri unit注册：${name}`);
requireText(read(`${databaseRoot}/mod.rs`), "#[cfg(test)]\nmod tests;", "数据库原unit target登记");
requireText(tests, "runtime_log::tests::test_store", "隔离复用原日志fixture，禁止discover污染真实运行目录");
if (/RuntimeLogStore::discover|set_var\(|set_current_dir\(/.test(tests)) throw new Error("数据库unit不得发现真实日志目录或修改进程环境");
requireText(read("src-tauri/src/runtime_log.rs"), "#[cfg(test)]\npub(crate) mod tests", "日志fixture只在测试编译中可见");

console.log("数据库编译修复与彻底清空契约通过：记录映射、后端重建、强确认、自动重连和前端状态重置均已锁定。");
