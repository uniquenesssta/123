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
const names = `${base}/names`;
const profiles = `${base}/profiles`;
const directory = `${base}/directory`;
const legacy = read("crates/persistence-postgres/src/team_catalog.rs");
const teamsMod = read(`${base}/mod.rs`);

for (const relative of [
  `${names}/mod.rs`,
  `${names}/add_team_name.rs`,
  `${names}/validation.rs`,
  `${names}/normalization.rs`,
  `${names}/row.rs`,
  `${names}/mapper.rs`,
  `${profiles}/mod.rs`,
  `${profiles}/upsert_team_profile.rs`,
  `${profiles}/input_policy.rs`,
  `${profiles}/row.rs`,
  `${profiles}/mapper.rs`,
  "crates/persistence-postgres/tests/team_names_profiles_repository_contract.rs",
]) check(exists(relative), `Missing R6-02 owner: ${relative}`);

check(teamsMod.includes("mod names;") && teamsMod.includes("mod profiles;"), "teams/mod.rs must register names/profiles");
check(!legacy.includes("pub async fn add_team_name"), "Legacy team_catalog.rs still owns add_team_name");
check(!legacy.includes("pub async fn upsert_team_profile"), "Legacy team_catalog.rs still owns upsert_team_profile");
for (const token of ["validate_team_profile", "trim_option", "normalize_name", "team_name_from_row", "team_profile_from_row"]) {
  check(!legacy.includes(token), `Legacy team_catalog.rs still owns R6-02 helper ${token}`);
}
check(legacy.includes("pub async fn bulk_delete_teams"), "R6-02 must not migrate R6-09 team deletion early");
check(legacy.includes("pub async fn bulk_delete_players"), "R6-02 must not disturb existing bulk player deletion path");

const addName = read(`${names}/add_team_name.rs`);
const nameValidation = read(`${names}/validation.rs`);
const normalization = read(`${names}/normalization.rs`);
const nameRow = read(`${names}/row.rs`);
const nameMapper = read(`${names}/mapper.rs`);
check(count(addName, "sqlx::query_as") === 1 && addName.includes("INSERT INTO football.team_names"), "add_team_name must own exactly one typed INSERT purpose");
check(addName.includes("self.pool.begin().await?") && addName.includes("tx.commit().await?"), "add_team_name must keep explicit transaction boundary");
check(addName.includes("team_name_added") && addName.includes("write_audit_event"), "add_team_name must keep audit event in transaction");
check(nameValidation.includes("球队别名不能为空") && nameValidation.includes("球队别名结束日期早于开始日期"), "team-name validation errors changed");
check(normalization.includes("split_whitespace") && normalization.includes("to_lowercase"), "team-name normalization semantics changed");
check(nameRow.includes("sqlx::FromRow") && !nameRow.includes("PgRow"), "team-name row must be typed");
check(!nameMapper.includes("sqlx::query") && nameMapper.includes("TeamNameRecord"), "team-name mapper must remain pure");

const upsertProfile = read(`${profiles}/upsert_team_profile.rs`);
const profilePolicy = read(`${profiles}/input_policy.rs`);
const profileRow = read(`${profiles}/row.rs`);
const profileMapper = read(`${profiles}/mapper.rs`);
check(count(upsertProfile, "sqlx::query_as") === 1 && upsertProfile.includes("INSERT INTO football.team_profiles"), "upsert_team_profile must own exactly one typed upsert purpose");
check(upsertProfile.includes("ON CONFLICT (team_id) DO UPDATE"), "profile upsert conflict contract missing");
check(upsertProfile.includes("head_coach=football.team_profiles.head_coach"), "profile upsert must preserve stored head coach");
check(upsertProfile.includes("metadata=football.team_profiles.metadata || EXCLUDED.metadata"), "profile metadata merge contract changed");
check(upsertProfile.includes("team_profile_updated") && upsertProfile.includes("write_audit_event"), "profile upsert must keep audit event");
check(upsertProfile.includes("self.pool.begin().await?") && upsertProfile.includes("tx.commit().await?"), "profile upsert must keep explicit transaction boundary");
for (const token of ["球队成立年份必须在1850到2100之间", "球队类型无效", "战术风格无效", "必须在0到100之间", "球队资料可信度必须在0到1之间"]) {
  check(profilePolicy.includes(token), `profile validation contract missing: ${token}`);
}
check(profileRow.includes("sqlx::FromRow") && !profileRow.includes("PgRow"), "team-profile row must be typed");
check(!profileMapper.includes("sqlx::query") && profileMapper.includes("TeamProfileRecord"), "team-profile mapper must remain pure");

const create = read(`${directory}/create_team.rs`);
const update = read(`${directory}/update_team.rs`);
check(create.includes("super::super::names::normalize_team_name") && update.includes("super::super::names::normalize_team_name"), "Team create/update must consume the unique name normalization owner");
check(!exists(`${directory}/name_policy.rs`), "Duplicate directory name policy must be removed");

const contract = read("crates/persistence-postgres/tests/team_names_profiles_repository_contract.rs");
for (const token of [
  "add_team_name",
  "upsert_team_profile",
  "Galaxy   Club",
  "galaxy club",
  "Persisted Coach",
  "球队别名不能为空",
  "球队别名结束日期早于开始日期",
  "球队类型无效",
  "进攻评分必须在0到100之间",
  "球队资料可信度必须在0到1之间",
]) check(contract.includes(token), `R6-02 PostgreSQL contract missing ${token}`);

const packageJson = JSON.parse(read("package.json"));
check(typeof packageJson.scripts["verify:team-names-profiles"] === "string", "package.json must expose verify:team-names-profiles");
check(packageJson.scripts["verify:architecture"].includes("verify-team-names-profiles.mjs"), "verify:architecture must include R6-02 gate");

if (failures.length) {
  console.error("R6-02 Team Names / Profiles verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R6-02 team names/profiles persistence verified: unique typed write owners, preserved validation/audit/metadata/head-coach semantics, shared normalization, and no early R6-09 migration.");
