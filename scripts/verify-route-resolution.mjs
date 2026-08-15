import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const count = (source, token) => source.split(token).length - 1;

const base = "crates/persistence-postgres/src/adapters/competition/route_resolution";
const contextBase = `${base}/context`;
const routeBase = `${base}/route`;
const legacyRoutingPath = "crates/persistence-postgres/src/routing.rs";
const legacyRouting = exists(legacyRoutingPath) ? read(legacyRoutingPath) : "";
const competitionMod = read("crates/persistence-postgres/src/adapters/competition/mod.rs");
const lib = read("crates/persistence-postgres/src/lib.rs");

for (const relative of [
  `${base}/mod.rs`,
  `${contextBase}/mod.rs`,
  `${contextBase}/record_row.rs`,
  `${contextBase}/record_mapper.rs`,
  `${contextBase}/resolve_context.rs`,
  `${contextBase}/season_context.rs`,
  `${contextBase}/stage_context.rs`,
  `${contextBase}/validate_scope.rs`,
  `${routeBase}/mod.rs`,
  `${routeBase}/record_row.rs`,
  `${routeBase}/record_mapper.rs`,
  `${routeBase}/resolve_route.rs`,
  `${routeBase}/explicit_rule_package.rs`,
  `${routeBase}/binding_candidate.rs`,
  "crates/persistence-postgres/tests/route_resolution_repository_contract.rs",
]) check(exists(relative), `Missing R5-05 owner: ${relative}`);

check(competitionMod.includes("mod route_resolution;"), "competition adapter root must register route_resolution");
check(competitionMod.includes("model_run_identity"), "R5-05 gate must recognize the approved R5-06 model_run_identity owner");
check(!exists("crates/persistence-postgres/src/competitions.rs"), "legacy competitions.rs must be removed after R5-05 context owner switch");
check(!lib.includes("mod competitions;"), "lib.rs must not register removed legacy competitions.rs");
check(exists("crates/persistence-postgres/src/adapters/competition/model_run_identity"), "R5-06 model_run_identity owner must exist after the approved next-node switch");

for (const token of ["pub async fn resolve_route", "route_decision_from_row", "resolve_competition_context", "ensure_scope_id"]) {
  check(!legacyRouting.includes(token), `legacy routing.rs must not own R5-05 responsibility ${token}`);
}
check(!exists(legacyRoutingPath), "R5-06 must remove legacy routing.rs after model registration owner switch");

const contextCoordinator = read(`${contextBase}/resolve_context.rs`);
check(contextCoordinator.includes("pub async fn resolve_competition_context"), "context coordinator must own resolve_competition_context");
check(!contextCoordinator.includes("sqlx::query") && !contextCoordinator.includes("SELECT "), "context coordinator must orchestrate without SQL");
for (const token of ["read_stage_context", "read_season_context", "ensure_scope_id", "read_competition", "fallback_kind"]) {
  check(contextCoordinator.includes(token), `context coordinator missing ${token}`);
}

const stageRead = read(`${contextBase}/stage_context.rs`);
const seasonRead = read(`${contextBase}/season_context.rs`);
check(count(stageRead, "sqlx::query_as") === 1 && stageRead.includes("FROM football.competition_stages") && stageRead.includes("c.is_active = true"), "stage_context must own exactly one active stage hierarchy SELECT");
check(count(seasonRead, "sqlx::query_as") === 1 && seasonRead.includes("FROM football.seasons") && seasonRead.includes("c.is_active = true"), "season_context must own exactly one active season hierarchy SELECT");
for (const message of ["赛事阶段不存在或所属赛事已停用", "赛季不存在或所属赛事已停用"]) {
  check(stageRead.includes(message) || seasonRead.includes(message), `context reads must preserve error: ${message}`);
}

const contextRows = read(`${contextBase}/record_row.rs`);
const contextMapper = read(`${contextBase}/record_mapper.rs`);
check(count(contextRows, "derive(Debug, FromRow)") === 2, "context persistence must use typed sqlx::FromRow rows");
check(!contextRows.includes("PgRow") && !contextMapper.includes("PgRow"), "context mapping must not use dynamic PgRow");
check(contextMapper.includes("parse_competition_kind"), "context mapper must reuse shared CompetitionKind parser");
check(!contextMapper.includes("sqlx::query"), "context mapper must not own SQL");
const scope = read(`${contextBase}/validate_scope.rs`);
check(scope.includes("{label}层级不一致") && scope.includes("Some(Uuid::new_v4())"), "scope validation must preserve generic hierarchy mismatch semantics and mismatch coverage");
check(!scope.includes("sqlx::"), "scope validation must not own SQL");

const routeCoordinator = read(`${routeBase}/resolve_route.rs`);
check(routeCoordinator.includes("pub async fn resolve_route"), "route coordinator must own resolve_route");
check(!routeCoordinator.includes("sqlx::query") && !routeCoordinator.includes("SELECT "), "route coordinator must orchestrate without SQL");
for (const token of ["read_explicit_rule_package", "read_binding_candidate", "binding_source", "map_route_row"]) {
  check(routeCoordinator.includes(token), `route coordinator missing ${token}`);
}

const explicitRead = read(`${routeBase}/explicit_rule_package.rs`);
check(count(explicitRead, "sqlx::query_as") === 1 && explicitRead.includes("FROM model.rule_packages") && explicitRead.includes("rp.status = 'active'"), "explicit_rule_package must own exactly one active package SELECT");
check(explicitRead.includes("split_part(d.model_key, '_', 1)") && explicitRead.includes("d.model_key = $3"), "explicit package read must preserve family/exact model filters");
check(explicitRead.includes("RouteNotFound"), "explicit package read must preserve RouteNotFound");

const candidateRead = read(`${routeBase}/binding_candidate.rs`);
check(count(candidateRead, "sqlx::query_as") === 1 && candidateRead.includes("FROM model.competition_bindings") && candidateRead.includes("b.is_active = true"), "binding_candidate must own exactly one active binding SELECT");
for (const token of [
  "b.valid_from IS NULL OR b.valid_from <= $5",
  "b.valid_to IS NULL OR b.valid_to >= $5",
  "(b.stage_id IS NOT NULL) DESC",
  "(b.season_id IS NOT NULL) DESC",
  "(b.competition_id IS NOT NULL) DESC",
  "b.priority DESC",
  "b.created_at DESC",
  "b.id DESC",
  "split_part(d.model_key, '_', 1)",
]) check(candidateRead.includes(token), `binding candidate must preserve ${token}`);

const routeRows = read(`${routeBase}/record_row.rs`);
const routeMapper = read(`${routeBase}/record_mapper.rs`);
check(routeRows.includes("derive(Debug, FromRow)") && routeRows.includes("struct RouteRow"), "route persistence must use typed sqlx::FromRow RouteRow");
check(!routeRows.includes("PgRow") && !routeMapper.includes("PgRow"), "route mapping must not use dynamic PgRow");
check(!routeMapper.includes("sqlx::query"), "route mapper must not own SQL");
for (const token of ["StageBinding", "SeasonBinding", "CompetitionBinding", "CompetitionKindDefault", "RouteDecision", '"source"', '"binding_id"', '"rule_package_id"', '"preferred_model_family"', '"preferred_model_id"', '"competition_id"', '"season_id"', '"stage_id"', '"competition_kind"', '"priority"']) {
  check(routeMapper.includes(token), `route mapper must preserve ${token}`);
}

const contract = read("crates/persistence-postgres/tests/route_resolution_repository_contract.rs");
for (const token of ["resolve_competition_context", "resolve_route", "ExplicitRulePackage", "StageBinding", "SeasonBinding", "CompetitionBinding", "CompetitionKindDefault", "RouteNotFound", "preferred_model_family", "preferred_model_id", "valid_from", "valid_to", "reason"]) {
  check(contract.includes(token), `R5-05 PostgreSQL contract missing ${token}`);
}

if (failures.length) {
  console.error("R5-05 Route Resolution verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R5-05 Route Resolution verified: context and route reads retain their unique typed persistence owners, and the approved R5-06 model identity owner has replaced legacy routing.rs.");
