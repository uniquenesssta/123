import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const count = (source, token) => source.split(token).length - 1;

const base = "crates/persistence-postgres/src/adapters/competition";
const legacy = read("crates/persistence-postgres/src/competitions.rs");
const adapterMod = read("crates/persistence-postgres/src/adapters/mod.rs");

for (const relative of [
  `${base}/mod.rs`,
  `${base}/detail/mod.rs`,
  `${base}/detail/record_row.rs`,
  `${base}/detail/record_mapper.rs`,
  `${base}/detail/read_competition.rs`,
  `${base}/directory/mod.rs`,
  `${base}/directory/create_competition.rs`,
  `${base}/directory/list_competitions.rs`,
  `${base}/directory/delete_competition/mod.rs`,
  `${base}/directory/delete_competition/lock_active_competition.rs`,
  `${base}/directory/delete_competition/soft_delete_competition.rs`,
  `${base}/directory/delete_competition/delete_external_entity_ids.rs`,
  `${base}/directory/delete_competition/deactivate_bindings.rs`,
  `${base}/directory/delete_competition/transaction.rs`,
  "crates/persistence-postgres/tests/competitions_repository_contract.rs",
]) check(exists(relative), `Missing R5-01 owner: ${relative}`);

check(adapterMod.includes("mod competition;"), "Persistence adapters root must register the competition module");
const competitionMod = read(`${base}/mod.rs`);
check(competitionMod.includes("mod detail;") && competitionMod.includes("mod directory;"), "competition/mod.rs must retain R5-01 directory/detail owners");
check(competitionMod.includes("mod hierarchy;"), "competition/mod.rs must register the approved R5-02 hierarchy owner");
check(!competitionMod.includes("rule_packages") && !competitionMod.includes("bindings") && !competitionMod.includes("route_resolution") && !competitionMod.includes("model_run_identity"), "R5-02 must not pre-implement R5-03 through R5-06 responsibilities");

for (const method of ["create_competition", "read_competition", "list_competitions", "delete_competition"]) {
  check(!new RegExp(`pub\\s+async\\s+fn\\s+${method}\\b`).test(legacy), `Legacy competitions.rs still owns ${method}`);
}
check(!legacy.includes("competition_record_from_row"), "Legacy CompetitionRecord PgRow mapper must be removed");
check(new RegExp(`pub\\s+async\\s+fn\\s+resolve_competition_context\\b`).test(legacy), "R5-02 must leave R5-05 resolve_competition_context in its current owner");

const row = read(`${base}/detail/record_row.rs`);
const mapper = read(`${base}/detail/record_mapper.rs`);
const detailRead = read(`${base}/detail/read_competition.rs`);
check(row.includes("derive(Debug, FromRow)") && row.includes("struct CompetitionRow"), "Competition detail must use a typed sqlx::FromRow record");
check(!row.includes("PgRow") && !mapper.includes("PgRow"), "R5-01 Competition mapping must not use dynamic PgRow");
check(mapper.includes("CompetitionRecord") && mapper.includes("parse_competition_kind"), "Competition mapper must own typed Row -> Domain conversion and preserve CompetitionKind parsing");
check(!mapper.includes("sqlx::query"), "Competition mapper must not own SQL");
check(count(detailRead, "sqlx::query_as") === 1 && detailRead.includes("WHERE id = $1 AND is_active = true"), "read_competition must own exactly one detail SELECT purpose");

const create = read(`${base}/directory/create_competition.rs`);
const list = read(`${base}/directory/list_competitions.rs`);
check(count(create, "sqlx::query_as") === 1 && create.includes("INSERT INTO football.competitions") && create.includes("draft.code.trim()") && create.includes("draft.name.trim()") && create.includes("draft.timezone.trim()"), "create_competition must own exactly one INSERT purpose and preserve trimming semantics");
check(count(list, "sqlx::query_as") === 1 && list.includes("WHERE is_active = true") && list.includes("sort_order"), "list_competitions must own exactly one active-directory SELECT and preserve ordering");

const tx = read(`${base}/directory/delete_competition/transaction.rs`);
check(!tx.includes("sqlx::query") && tx.includes("self.pool.begin().await?") && tx.includes("tx.commit().await?") && tx.includes("write_audit_event"), "delete_competition transaction owner must coordinate helpers/audit without embedding SQL");
for (const [file, token] of [
  ["lock_active_competition.rs", "sqlx::query_scalar"],
  ["soft_delete_competition.rs", "sqlx::query("],
  ["delete_external_entity_ids.rs", "sqlx::query("],
  ["deactivate_bindings.rs", "sqlx::query("],
]) {
  const source = read(`${base}/directory/delete_competition/${file}`);
  check(count(source, token) === 1, `${file} must own exactly one SQL purpose`);
}
check(read(`${base}/directory/delete_competition/soft_delete_competition.rs`).includes("'original_code', code"), "soft delete metadata must preserve original_code");
check(tx.includes('"competition_deleted"') && tx.includes('"deletion_mode": "soft_delete"'), "delete audit event contract must be preserved");

const contract = read("crates/persistence-postgres/tests/competitions_repository_contract.rs");
for (const token of ["create_competition", "read_competition", "list_competitions", "delete_competition", "original_code", "deleted_at"]) {
  check(contract.includes(token), `R5-01 PostgreSQL contract missing ${token}`);
}

const packageJson = JSON.parse(read("package.json"));
check(typeof packageJson.scripts["verify:competition-repository"] === "string", "package.json must expose verify:competition-repository");
check(packageJson.scripts["verify:architecture"].includes("verify-competition-repository.mjs"), "verify:architecture must include the R5-01 gate");

if (failures.length) {
  console.error("R5-01 Competitions Repository verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R5-01 Competitions Repository verified: directory/detail remain unique Competition CRUD owners, typed Row/Mapper and delete SQL boundaries remain intact, and only the approved R5-02 hierarchy module has been added.");
