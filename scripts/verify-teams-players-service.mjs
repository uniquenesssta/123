import { existsSync, readFileSync, readdirSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";
const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const read = (path) => readFileSync(join(root, path), "utf8").replaceAll("\r\n", "\n");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
function rustFiles(path) { const absolute = join(root, path); const result = []; for (const entry of readdirSync(absolute, {withFileTypes:true})) { const child=join(absolute,entry.name); if(entry.isDirectory()) result.push(...rustFiles(relative(root,child))); else if(entry.name.endsWith(".rs")) result.push(relative(root,child).replaceAll("\\","/")); } return result; }
const catalogRoot = "src-tauri/src/commands/catalog";
const splitCommandGroups = {
  "teams": [
    "create_team",
    "list_team_options",
    "list_teams",
    "read_team",
    "update_team",
    "add_team_name",
    "upsert_team_profile",
    "bulk_delete_teams",
    "preview_force_delete_team",
    "force_delete_team"
  ],
  "players": [
    "player_catalog_reference_data",
    "create_player",
    "update_player",
    "delete_player",
    "bulk_delete_players",
    "list_players",
    "read_player",
    "add_player_name",
    "assign_player_position",
    "add_player_team_period",
    "add_player_availability",
    "add_player_ability_observation",
    "add_player_dynamic_tag",
    "calculate_player_match_contribution"
  ],
  "coaches": [
    "create_coach",
    "list_coaches",
    "read_coach",
    "add_coach_name",
    "add_team_coach_period"
  ],
  "entities": [
    "list_entity_references",
    "resolve_entity_reference",
    "check_entity_deletion",
    "bulk_archive_entities",
    "create_data_provider",
    "add_external_entity_id"
  ]
};
const commandFiles = Object.fromEntries([
  ...Object.entries(splitCommandGroups).flatMap(([group, names]) => names.map(name => [name, `${catalogRoot}/${group}/${name}.rs`])),
]);
const teamMethods = ["create_team", "list_team_options", "list_teams", "read_team", "update_team", "add_team_name", "upsert_team_profile", "bulk_delete_teams", "preview_force_delete_team", "force_delete_team"];
const playerMethods = ["reference_data", "create_player", "update_player", "delete_player", "bulk_delete_players", "list_players", "read_player", "add_player_name", "assign_player_position", "add_player_team_period", "add_player_availability", "add_player_ability_observation", "add_player_dynamic_tag", "calculate_player_match_contribution", "create_coach", "list_coaches", "read_coach", "add_coach_name", "add_team_coach_period", "list_entity_references", "resolve_entity_reference", "check_entity_deletion", "bulk_archive_entities", "create_data_provider", "add_external_entity_id"];
const required = ["crates/application/src/services/teams/mod.rs","crates/application/src/services/teams/service.rs","crates/application/src/services/teams/facade.rs","crates/application/src/services/players/mod.rs","crates/application/src/services/players/service.rs","crates/application/src/services/players/facade.rs","crates/application/src/composition/adapters/mod.rs","crates/application/src/composition/adapters/teams.rs","crates/application/src/composition/adapters/players.rs", ...teamMethods.map((name)=>`crates/application/src/use_cases/teams/${name}/mod.rs`), ...playerMethods.map((name)=>`crates/application/src/use_cases/players/${name}/mod.rs`)];
for (const path of required) check(existsSync(join(root,path)), `缺少 R3-04 文件：${path}`);
const teamFacade=read("crates/application/src/services/teams/facade.rs"); const playerFacade=read("crates/application/src/services/players/facade.rs"); const app=read("crates/application/src/service/application_service.rs"); const composition=read("crates/application/src/composition/application_composition.rs"); const registry=read("crates/application/src/composition/port_registry.rs"); const teamAdapter=read("crates/application/src/composition/adapters/teams.rs"); const playerAdapter=read("crates/application/src/composition/adapters/players.rs"); const adapters=teamAdapter+"\n"+playerAdapter; const tauri=[...new Set(Object.values(commandFiles))].map(read).join("\n"); const packageJson=JSON.parse(read("package.json")); const frontend=read("scripts/verify-frontend.mjs");
check(app.includes("teams: TeamService"), "ApplicationService 缺少 Teams Service"); check(app.includes("players: PlayerService"), "ApplicationService 缺少 Players Service"); check(composition.includes("TeamService::new()"), "组合根未构造 TeamService"); check(composition.includes("PlayerService::new()"), "组合根未构造 PlayerService");
for (const implName of ["impl TeamCatalogPort for PersistenceStore","impl TeamLifecyclePort for PersistenceStore","impl PlayerCatalogPort for PersistenceStore","impl PlayerSignalPort for PersistenceStore","impl CoachCatalogPort for PersistenceStore","impl EntityReferencePort for PersistenceStore"]) check(adapters.includes(implName), `组合根职责适配器缺少：${implName}`); check(!registry.includes("impl TeamCatalogPort for PersistenceStore") && !registry.includes("impl PlayerCatalogPort for PersistenceStore"), "R3-04 具体适配实现重新堆叠到 port_registry.rs"); check(teamAdapter.includes("tokio::task::spawn_blocking") && teamAdapter.includes("runtime.block_on(store.force_delete_team"), "force-delete 非 Send 持久化边界未在组合适配器显式桥接");
for (const method of ["create_team","list_team_options","list_teams","read_team","update_team","add_team_name","upsert_team_profile","bulk_delete_teams","preview_force_delete_team","force_delete_team"]) check(teamFacade.includes(`pub async fn ${method}`), `Teams facade 缺少公共兼容方法：${method}`);
const r304Methods=["player_catalog_reference_data","create_coach","list_coaches","read_coach","add_coach_name","add_team_coach_period","list_entity_references","resolve_entity_reference","check_entity_deletion","bulk_archive_entities","create_team","list_team_options","list_teams","read_team","update_team","add_team_name","upsert_team_profile","bulk_delete_players","bulk_delete_teams","preview_force_delete_team","force_delete_team","create_data_provider","create_player","update_player","delete_player","list_players","read_player","add_player_name","assign_player_position","add_player_team_period","add_player_availability","add_player_ability_observation","add_player_dynamic_tag","calculate_player_match_contribution","add_external_entity_id"];
for (const method of r304Methods.filter((m)=>!["create_team","list_team_options","list_teams","read_team","update_team","add_team_name","upsert_team_profile","bulk_delete_teams","preview_force_delete_team","force_delete_team"].includes(m))) check(playerFacade.includes(`pub async fn ${method}`), `Players facade 缺少公共兼容方法：${method}`);
const serviceFiles=[...rustFiles("crates/application/src/services/teams"),...rustFiles("crates/application/src/services/players"),...rustFiles("crates/application/src/use_cases/teams"),...rustFiles("crates/application/src/use_cases/players")];
for(const path of serviceFiles) { const source=read(path); for(const token of ["football_persistence_postgres","PostgresStore","sqlx::","PgPool","PersistenceStore"]) check(!source.includes(token), `${path} 泄漏具体持久化实现：${token}`); }
check(read("crates/application/src/use_cases/players/add_player_team_period/mod.rs").includes("未知注册状态"), "球员注册状态校验未迁移到 Use Case"); check(read("crates/application/src/use_cases/players/add_player_ability_observation/mod.rs").includes("能力观察必须提供 calculation_version"), "能力观察版本校验未迁移到 Use Case");
for(const call of [".create_team(draft)",".list_teams(query)",".read_team(team_id)",".bulk_delete_players(ids)",".preview_force_delete_team(team_id)",".force_delete_team(request)",".create_player(draft)",".list_players(query)",".read_player(player_id)",".add_player_dynamic_tag(draft)"]) check(tauri.includes(call), `Tauri Team/Player 公共调用链变化：${call}`);
check(packageJson.scripts?.["verify:teams-players-service"] === "node scripts/verify-teams-players-service.mjs", "package.json 未登记 R3-04 专项门禁"); check(packageJson.scripts?.["verify:architecture"]?.includes("verify-teams-players-service.mjs"), "verify:architecture 未接入 R3-04 门禁"); check(frontend.includes('"verify-teams-players-service.mjs"'), "verify:frontend 未接入 R3-04 门禁");
// R11-04: original transport contracts, UUID gates, and unique module owners.
const commandContract = JSON.parse(read("architecture/command-contract.json"));
const boundaries = JSON.parse(read("architecture/module-boundaries.json"));
const catalogGroup = commandContract.command_groups.find(group => group.module === "catalog");
const commandsRoot = read("src-tauri/src/commands.rs");
check(!existsSync(join(root, `${catalogRoot}.rs`)), "旧catalog命令owner仍存在");
check(commandsRoot.includes("mod catalog;") && commandsRoot.includes("pub use catalog::*;"), "目录命令未使用原根入口");
check(catalogGroup?.definition_file === `${catalogRoot}/mod.rs` && catalogGroup?.parameter_and_return_signature_file === `${catalogRoot}/mod.rs`, "目录命令契约根出口漂移");
check(boundaries.tauri_commands?.groups?.catalog === `${catalogRoot}/mod.rs`, "目录命令边界owner漂移");
const commandSignatures = {
  "player_catalog_reference_data": "pub async fn player_catalog_reference_data(\n    state: State<'_, AppState>,\n) -> Result<PlayerCatalogReferenceData, String>",
  "create_coach": "pub async fn create_coach(\n    state: State<'_, AppState>,\n    draft: CoachDraft,\n) -> Result<CoachRecord, String>",
  "list_coaches": "pub async fn list_coaches(\n    state: State<'_, AppState>,\n    query: CoachListQuery,\n) -> Result<Vec<CoachListItem>, String>",
  "read_coach": "pub async fn read_coach(\n    state: State<'_, AppState>,\n    coach_id: String,\n) -> Result<CoachDetail, String>",
  "add_coach_name": "pub async fn add_coach_name(\n    state: State<'_, AppState>,\n    draft: CoachNameDraft,\n) -> Result<CoachNameRecord, String>",
  "add_team_coach_period": "pub async fn add_team_coach_period(\n    state: State<'_, AppState>,\n    draft: TeamCoachPeriodDraft,\n) -> Result<TeamCoachPeriodRecord, String>",
  "list_entity_references": "pub async fn list_entity_references(\n    state: State<'_, AppState>,\n    query: EntityReferenceQuery,\n) -> Result<Vec<EntityReferenceRecord>, String>",
  "resolve_entity_reference": "pub async fn resolve_entity_reference(\n    state: State<'_, AppState>,\n    request: EntityMatchRequest,\n) -> Result<EntityMatchResult, String>",
  "check_entity_deletion": "pub async fn check_entity_deletion(\n    state: State<'_, AppState>,\n    entity_type: String,\n    entity_id: String,\n) -> Result<EntityDeletionCheck, String>",
  "bulk_archive_entities": "pub async fn bulk_archive_entities(\n    state: State<'_, AppState>,\n    entity_type: String,\n    entity_ids: Vec<String>,\n) -> Result<BulkArchiveResult, String>",
  "create_team": "pub async fn create_team(\n    state: State<'_, AppState>,\n    draft: TeamDraft,\n) -> Result<TeamRecord, String>",
  "list_team_options": "pub async fn list_team_options(\n    state: State<'_, AppState>,\n    search: Option<String>,\n    limit: u32,\n) -> Result<Vec<TeamOption>, String>",
  "list_teams": "pub async fn list_teams(\n    state: State<'_, AppState>,\n    query: TeamListQuery,\n) -> Result<TeamListPage, String>",
  "read_team": "pub async fn read_team(state: State<'_, AppState>, team_id: String) -> Result<TeamDetail, String>",
  "update_team": "pub async fn update_team(\n    state: State<'_, AppState>,\n    team_id: String,\n    draft: TeamDraft,\n) -> Result<TeamRecord, String>",
  "add_team_name": "pub async fn add_team_name(\n    state: State<'_, AppState>,\n    draft: TeamNameDraft,\n) -> Result<TeamNameRecord, String>",
  "upsert_team_profile": "pub async fn upsert_team_profile(\n    state: State<'_, AppState>,\n    team_id: String,\n    draft: TeamProfileDraft,\n) -> Result<TeamProfileRecord, String>",
  "bulk_delete_players": "pub async fn bulk_delete_players(\n    state: State<'_, AppState>,\n    player_ids: Vec<String>,\n) -> Result<BulkDeleteResult, String>",
  "bulk_delete_teams": "pub async fn bulk_delete_teams(\n    state: State<'_, AppState>,\n    team_ids: Vec<String>,\n) -> Result<BulkDeleteResult, String>",
  "preview_force_delete_team": "pub async fn preview_force_delete_team(\n    state: State<'_, AppState>,\n    team_id: String,\n) -> Result<TeamForceDeletePreview, String>",
  "force_delete_team": "pub async fn force_delete_team(\n    state: State<'_, AppState>,\n    request: TeamForceDeleteRequest,\n) -> Result<TeamForceDeleteResult, String>",
  "create_data_provider": "pub async fn create_data_provider(\n    state: State<'_, AppState>,\n    draft: DataProviderDraft,\n) -> Result<DataProviderRecord, String>",
  "create_player": "pub async fn create_player(\n    state: State<'_, AppState>,\n    draft: PlayerDraft,\n) -> Result<PlayerRecord, String>",
  "delete_player": "pub async fn delete_player(state: State<'_, AppState>, player_id: String) -> Result<(), String>",
  "list_players": "pub async fn list_players(\n    state: State<'_, AppState>,\n    query: PlayerListQuery,\n) -> Result<PlayerListPage, String>",
  "read_player": "pub async fn read_player(\n    state: State<'_, AppState>,\n    player_id: String,\n) -> Result<PlayerDetail, String>",
  "add_player_name": "pub async fn add_player_name(\n    state: State<'_, AppState>,\n    draft: PlayerNameDraft,\n) -> Result<PlayerNameRecord, String>",
  "assign_player_position": "pub async fn assign_player_position(\n    state: State<'_, AppState>,\n    draft: PlayerPositionDraft,\n) -> Result<PlayerPositionRecord, String>",
  "add_player_team_period": "pub async fn add_player_team_period(\n    state: State<'_, AppState>,\n    draft: PlayerTeamPeriodDraft,\n) -> Result<PlayerTeamPeriodRecord, String>",
  "add_player_availability": "pub async fn add_player_availability(\n    state: State<'_, AppState>,\n    draft: PlayerAvailabilityDraft,\n) -> Result<PlayerAvailabilityRecord, String>",
  "add_player_ability_observation": "pub async fn add_player_ability_observation(\n    state: State<'_, AppState>,\n    draft: PlayerAbilityObservationDraft,\n) -> Result<PlayerAbilityObservationRecord, String>",
  "add_external_entity_id": "pub async fn add_external_entity_id(\n    state: State<'_, AppState>,\n    draft: ExternalEntityIdDraft,\n) -> Result<ExternalEntityIdRecord, String>",
  "update_player": "pub async fn update_player(\n    state: State<'_, AppState>,\n    player_id: String,\n    draft: PlayerDraft,\n) -> Result<PlayerRecord, String>",
  "add_player_dynamic_tag": "pub async fn add_player_dynamic_tag(\n    state: State<'_, AppState>,\n    draft: PlayerDynamicTagDraft,\n) -> Result<PlayerDynamicTagRecord, String>",
  "calculate_player_match_contribution": "pub async fn calculate_player_match_contribution(\n    state: State<'_, AppState>,\n    request: PlayerMatchContributionRequest,\n) -> Result<PlayerMatchContribution, String>"
};
const leadingValidation = {
  "read_coach": "let coach_id = parse_uuid(&coach_id, \"教练 ID\")?;",
  "check_entity_deletion": "let entity_id = parse_uuid(&entity_id, \"实体 ID\")?;",
  "bulk_archive_entities": "let ids = entity_ids\n        .iter()\n        .map(|value| parse_uuid(value, \"实体 ID\"))\n        .collect::<Result<Vec<_>, _>>()?;",
  "read_team": "let team_id = parse_uuid(&team_id, \"球队 ID\")?;",
  "update_team": "let team_id = parse_uuid(&team_id, \"球队 ID\")?;",
  "upsert_team_profile": "let team_id = parse_uuid(&team_id, \"球队 ID\")?;",
  "bulk_delete_players": "let ids = player_ids\n        .iter()\n        .map(|value| parse_uuid(value, \"球员 ID\"))\n        .collect::<Result<Vec<_>, _>>()?;",
  "bulk_delete_teams": "let ids = team_ids\n        .iter()\n        .map(|value| parse_uuid(value, \"球队 ID\"))\n        .collect::<Result<Vec<_>, _>>()?;",
  "preview_force_delete_team": "let team_id = parse_uuid(&team_id, \"球队 ID\")?;\n    let service =",
  "delete_player": "let player_id = parse_uuid(&player_id, \"球员 ID\")?;",
  "read_player": "let player_id = parse_uuid(&player_id, \"球员 ID\")?;",
  "update_player": "let parsed = parse_uuid(&player_id, \"球员 ID\")?;"
};
const applicationCalls = {
  "player_catalog_reference_data": "player_catalog_reference_data()",
  "create_coach": "create_coach(draft)",
  "list_coaches": "list_coaches(query)",
  "read_coach": "read_coach(coach_id)",
  "add_coach_name": "add_coach_name(draft)",
  "add_team_coach_period": "add_team_coach_period(draft)",
  "list_entity_references": "list_entity_references(query)",
  "resolve_entity_reference": "resolve_entity_reference(request)",
  "check_entity_deletion": "check_entity_deletion(entity_type, entity_id)",
  "bulk_archive_entities": "bulk_archive_entities(entity_type, ids)",
  "create_team": "create_team(draft)",
  "list_team_options": "list_team_options(search, limit)",
  "list_teams": "list_teams(query)",
  "read_team": "read_team(team_id)",
  "update_team": "update_team(team_id, draft)",
  "add_team_name": "add_team_name(draft)",
  "upsert_team_profile": "upsert_team_profile(team_id, draft)",
  "bulk_delete_players": "bulk_delete_players(ids)",
  "bulk_delete_teams": "bulk_delete_teams(ids)",
  "preview_force_delete_team": "preview_force_delete_team(team_id)",
  "force_delete_team": "force_delete_team(request)",
  "create_data_provider": "create_data_provider(draft)",
  "create_player": "create_player(draft)",
  "delete_player": "delete_player(player_id)",
  "list_players": "list_players(query)",
  "read_player": "read_player(player_id)",
  "add_player_name": "add_player_name(draft)",
  "assign_player_position": "assign_player_position(draft)",
  "add_player_team_period": "add_player_team_period(draft)",
  "add_player_availability": "add_player_availability(draft)",
  "add_player_ability_observation": "add_player_ability_observation(draft)",
  "add_external_entity_id": "add_external_entity_id(draft)",
  "update_player": "update_player(parsed, draft)",
  "add_player_dynamic_tag": "add_player_dynamic_tag(draft)",
  "calculate_player_match_contribution": "calculate_player_match_contribution(request)"
};
const rustTokens = source => (source.match(/"(?:\\.|[^"\\])*"|[A-Za-z_][A-Za-z_0-9]*|[^\s]/g) ?? []).join("\0");
const signatureTokens = source => rustTokens(source.replace(/,\s*\)/g, ")"));
const moduleTokens = source => rustTokens(source.replace(/,\s*}/g, "}"));
const exports = (module, names) => `pub use ${module}::{${names.flatMap(name => [`__cmd__${name}`, `__tauri_command_name_${name}`, name]).sort().join(",")}};`;
const rootModules = Object.keys(splitCommandGroups).sort();
const expectedRoot = [
  ...rootModules.map(name => `mod ${name};`),
  ...rootModules.map(name => exports(name, splitCommandGroups[name])),
].join("\n");
check(moduleTokens(read(`${catalogRoot}/mod.rs`)) === moduleTokens(expectedRoot), "目录出口必须仅显式登记实际职责和全部双宏");
for (const [group, names] of Object.entries(splitCommandGroups)) {
  const ordered = [...names].sort();
  const expected = [...ordered.map(name => `mod ${name};`), ...ordered.map(name => exports(name, [name]))].join("\n");
  check(moduleTokens(read(`${catalogRoot}/${group}/mod.rs`)) === moduleTokens(expected), `目录子出口不纯或双宏/命令缺失：${group}`);
}
for (const [name, file] of Object.entries(commandFiles)) {
  check(catalogGroup?.command_definition_files?.[name] === file, `目录命令定义归属漂移：${name}`);
}
for (const [name, expected] of Object.entries(commandSignatures)) {
  const source = read(commandFiles[name]);
  const signature = source.match(new RegExp(`pub async fn ${name}\\([\\s\\S]*?\\{`))?.[0].slice(0, -1) ?? "";
  check(signatureTokens(signature) === signatureTokens(expected), `球队/球员命令公开签名漂移：${name}`);
  check((source.match(/#\[tauri::command\]/g) ?? []).length === 1 && source.includes("bootstrap::AppState"), `命令未直接使用原State/唯一Tauri定义：${name}`);
  check(source.includes(`.${applicationCalls[name]}`), `命令未调用原Application能力/原参数：${name}`);
  check(source.includes(".map_err(|error| error.to_string())"), `命令未保留原服务错误字符串：${name}`);
  const prefix = leadingValidation[name];
  if (prefix) {
    const body = source.slice(source.indexOf("{", source.indexOf(`pub async fn ${name}`)) + 1);
    check(rustTokens(body).startsWith(rustTokens(prefix)), `UUID校验必须在服务调用/阻塞任务前完整失败返回：${name}`);
    check(source.includes("commands::parse_uuid"), `命令未复用原UUID解析owner：${name}`);
  }
  for (const forbidden of ["sqlx::", "football_persistence_postgres", "PostgresStore", "PgPool", "ApplicationService::new", "Mutex<", "static ", "macro_rules!", "filter_map", ".unwrap()", ".expect("]) {
    check(!source.includes(forbidden), `命令越过原边界或跳过校验：${name}/${forbidden}`);
  }
}
for (const [name, message] of [["preview_force_delete_team", "球队强制删除预检任务执行失败："], ["force_delete_team", "球队强制删除任务执行失败："]]) {
  const source = read(commandFiles[name]);
  check(source.includes("let service = state.service.clone();") && source.includes("let runtime = tauri::async_runtime::handle();"), `强删缺失原拥有所有权的句柄：${name}`);
  const blocking = source.slice(source.indexOf("tauri::async_runtime::spawn_blocking"));
  check(blocking.startsWith("tauri::async_runtime::spawn_blocking(move ||") && blocking.includes(".block_on(service.") && !blocking.includes("state."), `强删非Send Future隔离或State捕获漂移：${name}`);
  check(source.includes(`format!("${message}{error}")`) && /\}\)\s*\.await\s*\.map_err/.test(source), `强删join错误必须保持原上下文和传播：${name}`);
}

if(failures.length) throw new Error(`Teams/Players Service 验证失败\n${failures.map((item)=>`- ${item}`).join("\n")}`);
console.log(`Teams/Players Service 验证通过：${serviceFiles.length} 个 Service/Use Case Rust 文件，35 个公开 Application 职责保持 Ports 边界。`);
