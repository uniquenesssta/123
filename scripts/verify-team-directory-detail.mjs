import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const count = (source, token) => source.split(token).length - 1;

const base = "crates/persistence-postgres/src/adapters/catalog/teams";
const directory = `${base}/directory`;
const detail = `${base}/detail`;
const names = `${base}/names`;
const profiles = `${base}/profiles`;
const legacyTeam = exists("crates/persistence-postgres/src/team_catalog.rs") ? read("crates/persistence-postgres/src/team_catalog.rs") : "";
const r609Deletion = read("crates/persistence-postgres/src/adapters/catalog/deletion/bulk_delete.rs");
const legacyPlayer = read("crates/persistence-postgres/src/player_catalog.rs");
const adapters = read("crates/persistence-postgres/src/adapters/mod.rs");

for (const relative of [
  "crates/persistence-postgres/src/adapters/catalog/mod.rs",
  `${base}/mod.rs`,
  `${directory}/mod.rs`,
  `${directory}/create_team.rs`,
  `${directory}/update_team.rs`,
  `${directory}/list_team_options.rs`,
  `${directory}/option_row.rs`,
  `${directory}/option_mapper.rs`,
  `${directory}/list_teams.rs`,
  `${directory}/list_row.rs`,
  `${directory}/list_mapper.rs`,
  `${names}/normalization.rs`,
  `${detail}/mod.rs`,
  `${detail}/read_team.rs`,
  `${detail}/read_team_record.rs`,
  `${detail}/record_row.rs`,
  `${detail}/record_mapper.rs`,
  `${detail}/read_names.rs`,
  `${detail}/name_row.rs`,
  `${detail}/name_mapper.rs`,
  `${detail}/read_profile.rs`,
  `${detail}/profile_row.rs`,
  `${detail}/profile_mapper.rs`,
  `${detail}/read_squad.rs`,
  `${detail}/squad_row.rs`,
  `${detail}/squad_mapper.rs`,
  `${detail}/read_recent_matches.rs`,
  `${detail}/recent_match_row.rs`,
  `${detail}/recent_match_mapper.rs`,
  "crates/persistence-postgres/tests/team_directory_detail_repository_contract.rs",
]) check(exists(relative), `Missing R6-01 owner: ${relative}`);

check(adapters.includes("pub(crate) mod catalog;"), "Persistence adapters root must register catalog");
const catalogMod = read("crates/persistence-postgres/src/adapters/catalog/mod.rs");
const teamsMod = read(`${base}/mod.rs`);
check(catalogMod.includes("mod teams;") || catalogMod.includes("pub(crate) mod teams;"), "catalog/mod.rs must register teams");
check(teamsMod.includes("mod directory;") && teamsMod.includes("mod detail;"), "teams/mod.rs must register directory/detail owners");
check(teamsMod.includes("mod names;") && teamsMod.includes("mod profiles;"), "teams/mod.rs must retain later R6 team owner registrations");

for (const method of ["list_teams", "read_team", "update_team"]) {
  check(!new RegExp(`pub\\s+async\\s+fn\\s+${method}\\b`).test(legacyTeam), `Legacy team_catalog.rs still owns ${method}`);
}
for (const method of ["create_team", "list_team_options"]) {
  check(!new RegExp(`pub\\s+async\\s+fn\\s+${method}\\b`).test(legacyPlayer), `Legacy player_catalog.rs still owns ${method}`);
}
for (const token of ["team_record_from_row", "team_list_item_from_row", "team_squad_player_from_row", "team_recent_match_from_row"]) {
  check(!legacyTeam.includes(token), `Legacy team_catalog.rs still owns ${token}`);
}
for (const token of ["team_record_from_row", "team_option_from_row"]) {
  check(!legacyPlayer.includes(token), `Legacy player_catalog.rs still owns ${token}`);
}
check(!legacyTeam.includes("pub async fn add_team_name"), "R6-02 must remove team-name write from legacy owner");
check(!legacyTeam.includes("pub async fn upsert_team_profile"), "R6-02 must remove profile write from legacy owner");
check(read(`${names}/add_team_name.rs`).includes("pub async fn add_team_name"), "R6-02 names owner must expose add_team_name");
check(read(`${profiles}/upsert_team_profile.rs`).includes("pub async fn upsert_team_profile"), "R6-02 profiles owner must expose upsert_team_profile");
check(!exists("crates/persistence-postgres/src/team_catalog.rs") && r609Deletion.includes("pub async fn bulk_delete_teams"), "R6-09 permanent team deletion owner advancement missing");

const create = read(`${directory}/create_team.rs`);
const update = read(`${directory}/update_team.rs`);
const options = read(`${directory}/list_team_options.rs`);
const list = read(`${directory}/list_teams.rs`);
check(count(create, "sqlx::query_as") === 1 && create.includes("INSERT INTO football.teams"), "create_team must own one typed INSERT purpose");
check(count(update, "sqlx::query_as") === 1 && update.includes("UPDATE football.teams"), "update_team must own one typed UPDATE purpose");
check(create.includes("super::super::names::normalize_team_name") && update.includes("super::super::names::normalize_team_name"), "Team create/update must use the unique names normalization owner");
check(!exists(`${directory}/name_policy.rs`), "R6-02 must remove duplicate directory name policy");
check(count(options, "build_query_as::<TeamOptionRow>") === 1 && options.includes("football.team_names"), "list_team_options must retain typed alias-aware search");
check(count(list, "build_query_as::<TeamListRow>") === 1 && list.includes("football.team_coach_periods") && list.includes("football.player_availability"), "list_teams must retain typed directory aggregation");
check(list.includes("NameSearch::parse") && options.includes("NameSearch::parse"), "Team directory searches must retain shared NameSearch policy");
check(list.includes("球队分页游标必须同时包含名称和 ID"), "Team pagination cursor error contract must be preserved");

for (const file of ["record_row.rs", "name_row.rs", "profile_row.rs", "squad_row.rs", "recent_match_row.rs"]) {
  const source = read(`${detail}/${file}`);
  check(source.includes("sqlx::FromRow"), `${file} must define a typed sqlx::FromRow record`);
  check(!source.includes("PgRow"), `${file} must not use dynamic PgRow`);
}
for (const file of ["record_mapper.rs", "name_mapper.rs", "profile_mapper.rs", "squad_mapper.rs", "recent_match_mapper.rs"]) {
  const source = read(`${detail}/${file}`);
  check(!source.includes("sqlx::query"), `${file} mapper must not own SQL`);
}
const detailCoordinator = read(`${detail}/read_team.rs`);
check(!detailCoordinator.includes("sqlx::query"), "read_team coordinator must orchestrate without embedding SQL");
for (const file of ["read_team_record.rs", "read_names.rs", "read_profile.rs", "read_squad.rs", "read_recent_matches.rs"]) {
  const source = read(`${detail}/${file}`);
  check(count(source, "sqlx::query_as") === 1, `${file} must own exactly one typed SELECT purpose`);
}
check(read(`${detail}/read_squad.rs`).includes("default_role_code"), "Team detail squad read must preserve default tactical role projection");
check(read(`${detail}/read_squad.rs`).includes("zh-cn") && read(`${detail}/read_squad.rs`).includes("[一-龥]"), "Team detail squad read must preserve localized Chinese name preference");

const contract = read("crates/persistence-postgres/tests/team_directory_detail_repository_contract.rs");
for (const token of ["create_team", "list_team_options", "list_teams", "read_team", "update_team", "银河俱乐部", "current_player_count", "recent_matches"]) {
  check(contract.includes(token), `R6-01 PostgreSQL contract missing ${token}`);
}

const packageJson = JSON.parse(read("package.json"));
check(typeof packageJson.scripts["verify:team-directory-detail"] === "string", "package.json must expose verify:team-directory-detail");
check(packageJson.scripts["verify:architecture"].includes("verify-team-directory-detail.mjs"), "verify:architecture must include R6-01 team directory/detail gate");

if (failures.length) {
  console.error("R6-01 Team Directory / Detail verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R6-01 team directory/detail persistence verified: unique catalog owner, typed rows/mappers, preserved search/pagination/detail aggregation, and R6-02/R6-09 owner advancement.");
