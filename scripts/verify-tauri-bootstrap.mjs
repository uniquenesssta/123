import { readFileSync, readdirSync, existsSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replace(/\r\n/g, "\n");
const failures = [];
const check = (condition, message) => {
  if (!condition) failures.push(message);
};

const bootstrapRoot = join(root, "src-tauri", "src", "bootstrap");
const expectedFiles = [
  "application.rs",
  "command_registry.rs",
  "error.rs",
  "mod.rs",
  "state.rs",
];
const actualFiles = readdirSync(bootstrapRoot, { withFileTypes: true })
  .filter((entry) => entry.isFile())
  .map((entry) => entry.name)
  .sort();
check(
  JSON.stringify(actualFiles) === JSON.stringify(expectedFiles),
  `Tauri bootstrap 文件集合不正确：${actualFiles.join(", ")}`,
);

const library = read("src-tauri/src/lib.rs");
const moduleRoot = read("src-tauri/src/bootstrap/mod.rs");
const application = read("src-tauri/src/bootstrap/application.rs");
const state = read("src-tauri/src/bootstrap/state.rs");
const registry = read("src-tauri/src/bootstrap/command_registry.rs");
const error = read("src-tauri/src/bootstrap/error.rs");
const commands = read("src-tauri/src/commands.rs");
const commandContract = JSON.parse(read("architecture/command-contract.json"));
const moduleContract = JSON.parse(read("architecture/module-boundaries.json"));
const stateContract = JSON.parse(read("architecture/state-ownership.json"));

check(library.includes("mod bootstrap;"), "src-tauri/src/lib.rs 未声明 bootstrap 模块");
check(library.includes("pub fn run()"), "src-tauri/src/lib.rs 未保留公共 run 入口");
check(library.includes("bootstrap::run();"), "公共 run 未委托 Tauri 组合根");
for (const forbidden of ["tauri::Builder", ".setup(", ".manage(", "generate_handler!"]) {
  check(!library.includes(forbidden), `src-tauri/src/lib.rs 仍承担组合职责：${forbidden}`);
}

const declaredModules = [...moduleRoot.matchAll(/^mod\s+([a-z_]+);$/gm)]
  .map((match) => match[1])
  .sort();
check(
  JSON.stringify(declaredModules) ===
    JSON.stringify(["application", "command_registry", "error", "state"]),
  `bootstrap/mod.rs 模块出口不正确：${declaredModules.join(", ")}`,
);
check(moduleRoot.includes("pub(crate) use state::AppState;"), "bootstrap 未公开唯一 AppState");
check(moduleRoot.includes("application::run();"), "bootstrap 未委托 application 组合入口");

check(application.includes("tauri::Builder::default()"), "application.rs 未构造 Tauri Builder");
check(application.includes("tauri_plugin_dialog::init()"), "application.rs 未保持 dialog 插件注册");
check(application.includes("state::install(app)?"), "application.rs 未委托状态安装");
check(application.includes("command_registry::register(builder)"), "application.rs 未委托命令注册");
check(application.includes("tauri::generate_context!()"), "application.rs 未保持 Tauri context");
check(application.includes("error::expect_startup"), "application.rs 未委托启动错误映射");

check(state.includes("pub struct AppState"), "state.rs 未定义 AppState");
for (const field of [
  "service",
  "config_path",
  "issue_log",
  "runtime_log",
  "openai_profiles",
  "workspace_state",
  "api_workspace_requests",
]) {
  check(new RegExp(`\\bpub ${field}:`).test(state), `AppState 缺少字段：${field}`);
}
for (const required of [
  "app_config_dir()",
  "RuntimeLogStore::discover",
  '"application_started"',
  "app.manage(AppState",
]) {
  check(state.includes(required), `state.rs 缺少原启动状态行为：${required}`);
}
check(!state.includes("#[tauri::command]"), "state.rs 不得实现 Tauri 命令");

const handlerMatches = [...registry.matchAll(/tauri::generate_handler!\s*\[([\s\S]*?)\]\s*\)/g)];
check(handlerMatches.length === 1, `命令注册表必须且只能有一个 generate_handler!，实际 ${handlerMatches.length}`);
const registered = handlerMatches.length
  ? [...handlerMatches[0][1].matchAll(/commands::([a-z0-9_]+)/g)].map((match) => match[1])
  : [];
const expectedCommands = commandContract.commands_in_registration_order ?? [];
check(registered.length === 171, `Tauri 注册命令数量必须为 171，实际 ${registered.length}`);
check(new Set(registered).size === registered.length, "Tauri 注册表存在重复命令");
check(
  JSON.stringify(registered) === JSON.stringify(expectedCommands),
  "Tauri 注册命令名称或顺序与冻结命令契约不一致",
);
check(!registry.includes("#[tauri::command]"), "command_registry.rs 只能登记命令，不能实现命令");

check(error.includes('"足球赛事模型平台启动失败"'), "启动错误提示语义发生变化");
check(
  error.includes("io::Error::other") || error.includes("std::io::Error::other"),
  "启动 I/O 错误映射发生变化",
);
// R11-09 retired state re-export BEGIN
check(!/\bAppState\b/.test(commands), "commands.rs 不得残留无消费者的 AppState 重导出或替代状态定义");
const commandDefinitionFiles = new Set(commandContract.command_groups.flatMap(group =>
  Object.values(group.command_definition_files ?? { root: group.definition_file }),
));
for (const file of commandDefinitionFiles) {
  const source = read(file);
  if (/\bAppState\b/.test(source)) {
    check(/\bbootstrap\s*::\s*AppState\b/.test(source), `命令必须直接借用组合根唯一 AppState，不得恢复 commands 中转：${file}`);
  }
}
// R11-09 retired state re-export END
check(!commands.includes("pub struct AppState"), "commands.rs 仍重复定义 AppState");
check(commands.includes("use uuid::Uuid;"), "commands.rs 丢失 UUID 解析依赖");

check(
  commandContract.sources?.tauri_registration?.file ===
    "src-tauri/src/bootstrap/command_registry.rs",
  "命令契约未指向新的注册所有者",
);
check(
  moduleContract.rust?.tauri_host?.public_entry === "src-tauri/src/lib.rs::run",
  "模块边界契约未保留公共 Tauri 入口",
);
check(
  moduleContract.rust?.tauri_host?.state === "src-tauri/src/bootstrap/state.rs::AppState",
  "模块边界契约未指向新的 AppState 所有者",
);
check(
  moduleContract.tauri_commands?.registration_owner ===
    "src-tauri/src/bootstrap/command_registry.rs",
  "模块边界契约未指向新的命令注册所有者",
);
const tauriAppState = stateContract.states?.find((entry) => entry.id === "tauri.app-state");
check(
  tauriAppState?.owner === "src-tauri/src/bootstrap/state.rs::AppState",
  "状态所有权契约未切换 tauri.app-state owner",
);
check(tauriAppState?.transition === null, "tauri.app-state 仍处于过渡状态");

const platformRoot = "src-tauri/src/commands/platform";
const platform = read(`${platformRoot}/mod.rs`);
const platformCommands = {
  bootstrap: "bootstrap/command.rs",
  read_workspace_state: "read_workspace_state.rs",
  save_workspace_state: "save_workspace_state.rs",
  clear_workspace_state: "clear_workspace_state.rs",
  record_issue: "record_issue.rs",
  record_runtime_operation: "runtime_operation/command.rs",
  list_issue_logs: "list_issue_logs.rs",
  clear_issue_logs: "clear_issue_logs.rs",
  export_issue_logs: "export_issue_logs.rs",
};
const platformSignatures = {
  bootstrap: "pub async fn bootstrap(state: State<'_, AppState>) -> Result<BootstrapResponse, String>",
  read_workspace_state: "pub fn read_workspace_state(state: State<'_, AppState>) -> Result<Value, String>",
  save_workspace_state: "pub fn save_workspace_state(state: State<'_, AppState>, document: Value) -> Result<(), String>",
  clear_workspace_state: "pub fn clear_workspace_state(state: State<'_, AppState>) -> Result<Value, String>",
  record_issue: "pub fn record_issue(state: State<'_, AppState>, draft: IssueLogDraft) -> Result<IssueLogEntry, String>",
  record_runtime_operation: "pub fn record_runtime_operation(state: State<'_, AppState>, draft: RuntimeOperationDraft) -> Result<(), String>",
  list_issue_logs: "pub async fn list_issue_logs(state: State<'_, AppState>, limit: Option<u32>) -> Result<Vec<IssueLogEntry>, String>",
  clear_issue_logs: "pub fn clear_issue_logs(state: State<'_, AppState>) -> Result<(), String>",
  export_issue_logs: "pub fn export_issue_logs(state: State<'_, AppState>, output_path: String) -> Result<(), String>",
};
const signatureTokens = source => source.replace(/,\s*\)/g, ")").replace(/\s+/g, "");
const platformGroup = commandContract.command_groups.find(group => group.module === "platform");
check(commands.includes("mod platform;") && commands.includes("pub use platform::*;"), "平台命令未接入唯一根出口");
check(platformGroup?.definition_file === `${platformRoot}/mod.rs`, "平台命令清单根出口未同步");
for (const [name, file] of Object.entries(platformCommands)) {
  const source = read(`${platformRoot}/${file}`);
  check(source.includes(`fn ${name}(`), `平台命令缺失：${name}`);
  const signature = source.match(new RegExp(`pub (?:async )?fn ${name}\\([\\s\\S]*?\\{`))?.[0].slice(0, -1) ?? "";
  check(signatureTokens(signature) === signatureTokens(platformSignatures[name]), `平台命令公共签名漂移：${name}`);
  check(platform.includes(`__cmd__${name}`) && platform.includes(`__tauri_command_name_${name}`), `平台命令缺少Tauri 2.6.3宏出口：${name}`);
  check(platformGroup?.command_definition_files?.[name] === `${platformRoot}/${file}`, `平台命令归属不准确：${name}`);
  for (const forbidden of ["sqlx::", "PostgresStore::", "static ", "Arc<Mutex", "CREATE TABLE", "SELECT ", "UPDATE "]) {
    check(!source.includes(forbidden), `平台命令越过原基础设施边界：${name}/${forbidden}`);
  }
}
for (const file of ["mod.rs", "bootstrap/mod.rs", "runtime_operation/mod.rs"]) {
  check(!/\b(?:fn|struct|impl|enum)\s/.test(read(`${platformRoot}/${file}`)), `平台出口仍承载业务：${file}`);
}
for (const old of ["workspace", "issues", "logging"]) {
  check(!existsSync(join(root, `src-tauri/src/commands/${old}.rs`)), `旧平台owner未删除：${old}`);
  check(!commands.includes(`mod ${old};`), `旧平台模块仍登记：${old}`);
}
check(!["mod.rs", "configure/command.rs", "configure/coordinator.rs", "disconnect_database.rs", "reset/command.rs", "reset/coordinator.rs"].some(file => read(`src-tauri/src/commands/database/${file}`).includes("fn bootstrap(")), "数据库命令仍持有旧bootstrap");
const connection = read(`${platformRoot}/bootstrap/connection.rs`);
const databaseFacade = read("crates/application/src/services/database/facade.rs");
check(/pub async fn connect_database\(\s*self: &Arc<Self>,/.test(databaseFacade), "连接恢复必须核对原ApplicationService的Arc接收者");
check(connection.includes("service: &Arc<ApplicationService>") && connection.includes("sync::Arc"), "恢复连接必须借用原Arc服务句柄，不得退化为普通ApplicationService引用");
check(connection.includes("if service.is_database_connected().await") && connection.indexOf("is_database_connected().await") < connection.indexOf("DesktopConfig::load(config_path)"), "恢复连接前必须保留已连接短路");
check(connection.includes("let options = config.database?;") && connection.includes(".connect_database(options)"), "恢复连接必须保持原可选配置/连接入口");
const coordinator = read(`${platformRoot}/bootstrap/coordinator.rs`);
check(coordinator.includes("ensure_saved_connection(&state.service, &state.config_path).await"), "恢复连接必须直接借用AppState的原Arc，不得另造服务或连接");
check(coordinator.includes("ensure_saved_connection(") && coordinator.indexOf("ensure_saved_connection(") < coordinator.indexOf("state.service.bootstrap().await"), "bootstrap必须先恢复保存连接再读初始化数据");
for (const token of ["saved_connection_restore_failed", "bootstrap_failed", "return Err(message);", "state.runtime_log.relative_display_path()", "config_path: state.config_path.to_string_lossy().to_string()"])
  check(coordinator.includes(token), `bootstrap丢失原错误/路径行为：${token}`);
const operation = read(`${platformRoot}/runtime_operation/command.rs`);
check(operation.includes("normalize_phase(&draft.phase)?") && operation.includes('validate_text("操作名称"') && operation.indexOf("normalize_phase(&draft.phase)?") < operation.indexOf('validate_text("操作名称"'), "运行日志必须保留phase先于operation的校验顺序");
check(operation.includes('validate_text("链路 ID"') && operation.indexOf('validate_text("链路 ID"') < operation.indexOf("state.runtime_log.record("), "运行日志必须在完整校验后写入");
check(operation.includes('"failed" => "error"') && operation.includes('"frontend.operation"') && operation.includes('format!("operation_{phase}")') && operation.includes("draft.context.unwrap_or(Value::Null)"), "运行日志级别/事件/显式null投影漂移");
const issueList = read(`${platformRoot}/list_issue_logs.rs`);
check(issueList.includes("list_background_jobs(500).await") && issueList.includes("state.issue_log.list(limit.unwrap_or(500) as usize)"), "问题列表后台扫描或原limit投影漂移");
check(issueList.includes("if state.service.is_database_connected().await") && issueList.indexOf("is_database_connected().await") < issueList.indexOf("list_background_jobs(500).await"), "断开数据库时不得扫描后台任务");
check(read("src-tauri/src/commands/analytics/jobs/list_background_jobs.rs").includes("use crate::commands::record_failed_background_jobs;"), "Analytics未切换共享问题投影owner");
const background = read(`${platformRoot}/background_jobs.rs`);
for (const token of ["job.status == JobStatus::Failed", "if let Some(message)", 'format!("{}:{}", job.id, job.attempts)', "let _ = issue_log.record("])
  check(background.includes(token), `后台问题投影原语义丢失：${token}`);
const newTests = [
  ["bootstrap/tests.rs", ["missing_saved_connection_does_not_create_config_or_connect", "saved_config_errors_are_returned_unchanged_without_repair_or_connection", "empty_current_config_is_read_without_rewriting_or_connecting", "legacy_empty_config_keeps_original_load_migration", "bootstrap_response_keeps_original_wire_fields_nulls_and_paths"]],
  ["runtime_operation/tests.rs", ["phase_validation_rejects_unknown_values", "operation_name_validation_is_bounded", "phases_keep_case_normalization_and_unknown_phase_error", "text_limits_count_unicode_scalars_and_preserve_exact_errors", "runtime_draft_keeps_optional_nulls_unknown_fields_and_raw_input", "runtime_draft_keeps_required_fields_and_original_wire_types"]],
  ["tests.rs", ["background_scan_keeps_failed_only_filter_and_original_issue_projection", "repeated_scan_deduplicates_same_attempt_but_records_a_new_attempt"]],
];
for (const [file, names] of newTests) for (const name of names) check(read(`${platformRoot}/${file}`).includes(`fn ${name}(`), `原Tauri unit target缺少平台覆盖：${name}`);
for (const file of ["mod.rs", "bootstrap/mod.rs", "runtime_operation/mod.rs"]) check(read(`${platformRoot}/${file}`).includes("#[cfg(test)]\nmod tests;"), `平台测试未登记：${file}`);

if (failures.length) {
  throw new Error(`Tauri 组合根验证失败\n${failures.map((item) => `- ${item}`).join("\n")}`);
}

console.log(
  `Tauri 组合根验证通过：5 个职责模块、1 个 AppState 所有者、${registered.length} 个冻结命令及启动错误语义全部一致。`,
);
