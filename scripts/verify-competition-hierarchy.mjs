import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const count = (source, token) => source.split(token).length - 1;

const base = "crates/persistence-postgres/src/adapters/competition/hierarchy";
const legacyPath = "crates/persistence-postgres/src/competitions.rs";
const legacyExists = exists(legacyPath);
const legacy = legacyExists ? read(legacyPath) : "";
const competitionMod = read("crates/persistence-postgres/src/adapters/competition/mod.rs");

const hierarchyFiles = [
  `${base}/mod.rs`,
  `${base}/seasons/mod.rs`,
  `${base}/seasons/create_season.rs`,
  `${base}/seasons/read_season.rs`,
  `${base}/seasons/list_seasons.rs`,
  `${base}/seasons/record_row.rs`,
  `${base}/seasons/record_mapper.rs`,
  `${base}/stages/mod.rs`,
  `${base}/stages/create_stage.rs`,
  `${base}/stages/read_stage.rs`,
  `${base}/stages/list_stages.rs`,
  `${base}/stages/record_row.rs`,
  `${base}/stages/record_mapper.rs`,
  `${base}/rounds/mod.rs`,
  `${base}/rounds/create_round.rs`,
  `${base}/rounds/read_round.rs`,
  `${base}/rounds/list_rounds.rs`,
  `${base}/rounds/record_row.rs`,
  `${base}/rounds/record_mapper.rs`,
  "crates/persistence-postgres/tests/competition_hierarchy_repository_contract.rs",
];
for (const relative of hierarchyFiles) check(exists(relative), `Missing R5-02 owner: ${relative}`);

check(competitionMod.includes("mod hierarchy;"), "competition adapter root must register hierarchy");
const hierarchyMod = read(`${base}/mod.rs`);
for (const moduleName of ["seasons", "stages", "rounds"]) {
  check(hierarchyMod.includes(`mod ${moduleName};`), `hierarchy/mod.rs must register ${moduleName}`);
}
check(!hierarchyMod.includes("rule_packages") && !hierarchyMod.includes("bindings") && !hierarchyMod.includes("route_resolution") && !hierarchyMod.includes("model_run_identity"), "R5-02 hierarchy must not pre-implement later R5 responsibilities");

for (const method of [
  "create_season", "list_seasons", "read_season",
  "create_stage", "list_stages", "read_stage",
  "create_round", "list_rounds", "read_round",
]) {
  check(!new RegExp(`(?:pub\\s+)?async\\s+fn\\s+${method}\\b`).test(legacy), `Legacy competitions.rs still owns ${method}`);
}
for (const mapper of ["season_record_from_row", "stage_record_from_row", "round_record_from_row"]) {
  check(!legacy.includes(mapper), `Legacy competitions.rs still owns dynamic mapper ${mapper}`);
}
check(!legacyExists, "R5-05 must remove legacy competitions.rs after context owner switch");
check(exists("crates/persistence-postgres/src/adapters/competition/route_resolution/context/resolve_context.rs"), "R5-05 context coordinator must own resolve_competition_context");
check(exists("crates/persistence-postgres/src/adapters/competition/route_resolution/context/validate_scope.rs"), "R5-05 context owner must retain scope validation");

for (const [moduleName, rowName, recordName] of [
  ["seasons", "SeasonRow", "SeasonRecord"],
  ["stages", "StageRow", "StageRecord"],
  ["rounds", "RoundRow", "RoundRecord"],
]) {
  const row = read(`${base}/${moduleName}/record_row.rs`);
  const mapper = read(`${base}/${moduleName}/record_mapper.rs`);
  check(row.includes("derive(Debug, FromRow)") && row.includes(`struct ${rowName}`), `${moduleName} must use typed sqlx::FromRow`);
  check(!row.includes("PgRow") && !mapper.includes("PgRow"), `${moduleName} mapping must not use dynamic PgRow`);
  check(mapper.includes(recordName), `${moduleName} mapper must own typed Row -> Domain conversion`);
  check(!mapper.includes("sqlx::query"), `${moduleName} mapper must not own SQL`);
}
check(read(`${base}/stages/record_mapper.rs`).includes("parse_competition_kind"), "Stage mapper must preserve CompetitionKind parsing");

const seasonCreate = read(`${base}/seasons/create_season.rs`);
check(count(seasonCreate, "sqlx::query(") === 1 && seasonCreate.includes("INSERT INTO football.seasons") && seasonCreate.includes("draft.name.trim()") && seasonCreate.includes("draft.status.trim()") && seasonCreate.includes("read_season(self, id).await"), "Season create must preserve single INSERT, trim semantics and post-insert read");
const seasonRead = read(`${base}/seasons/read_season.rs`);
check(count(seasonRead, "sqlx::query_as") === 1 && seasonRead.includes("WHERE s.id = $1"), "Season detail read must own exactly one SELECT purpose");
const seasonList = read(`${base}/seasons/list_seasons.rs`);
check(count(seasonList, "sqlx::query_as") === 1 && seasonList.includes("WHERE c.is_active = true") && seasonList.includes("ORDER BY c.name, s.starts_on DESC NULLS LAST, s.name"), "Season list must preserve active filtering and ordering");

const stageCreate = read(`${base}/stages/create_stage.rs`);
check(count(stageCreate, "sqlx::query(") === 1 && stageCreate.includes("INSERT INTO football.competition_stages") && stageCreate.includes("draft.code.trim()") && stageCreate.includes("draft.name.trim()") && stageCreate.includes("draft.stage_kind.as_str()") && stageCreate.includes("read_stage(self, id).await"), "Stage create must preserve single INSERT, trim/kind semantics and post-insert read");
const stageRead = read(`${base}/stages/read_stage.rs`);
check(count(stageRead, "sqlx::query_as") === 1 && stageRead.includes("WHERE st.id = $1"), "Stage detail read must own exactly one SELECT purpose");
const stageList = read(`${base}/stages/list_stages.rs`);
check(count(stageList, "sqlx::query_as") === 1 && stageList.includes("WHERE c.is_active = true") && stageList.includes("ORDER BY c.name, s.name, st.sequence_no, st.name"), "Stage list must preserve active filtering and ordering");

const roundCreate = read(`${base}/rounds/create_round.rs`);
check(count(roundCreate, "sqlx::query(") === 1 && roundCreate.includes("INSERT INTO football.rounds") && roundCreate.includes("draft.code.trim()") && roundCreate.includes("draft.name.trim()") && roundCreate.includes("read_round(self, id).await"), "Round create must preserve single INSERT, trim semantics and post-insert read");
const roundRead = read(`${base}/rounds/read_round.rs`);
check(count(roundRead, "sqlx::query_as") === 1 && roundRead.includes("WHERE r.id = $1"), "Round detail read must own exactly one SELECT purpose");
const roundList = read(`${base}/rounds/list_rounds.rs`);
check(count(roundList, "sqlx::query_as") === 1 && roundList.includes("WHERE c.is_active = true") && roundList.includes("ORDER BY st.name, r.sequence_no, r.starts_at NULLS LAST"), "Round list must preserve active filtering and ordering");

const contract = read("crates/persistence-postgres/tests/competition_hierarchy_repository_contract.rs");
for (const token of ["create_season", "list_seasons", "create_stage", "list_stages", "create_round", "list_rounds", "metadata", "rules", "KnockoutTwoLeg"]) {
  check(contract.includes(token), `R5-02 PostgreSQL contract missing ${token}`);
}

const repositoryGate = read("scripts/verify-competition-repository.mjs");
const packageJson = JSON.parse(read("package.json"));
check(repositoryGate.includes('import "./verify-competition-hierarchy.mjs";'), "R5-01 competition gate must chain the R5-02 hierarchy gate");
check(packageJson.scripts["verify:architecture"].includes("verify-competition-repository.mjs"), "verify:architecture must reach the R5-02 hierarchy gate through the competition gate chain");

if (failures.length) {
  console.error("R5-02 Competition Hierarchy verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R5-02 Competition Hierarchy verified: seasons/stages/rounds remain unique typed owners, legacy competitions.rs is removed, and R5-05 context resolution is owned by route_resolution/context.");
