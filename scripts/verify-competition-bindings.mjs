import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const count = (source, token) => source.split(token).length - 1;

const base = "crates/persistence-postgres/src/adapters/competition/bindings";
const legacy = read("crates/persistence-postgres/src/routing.rs");
const contextOwner = read("crates/persistence-postgres/src/competitions.rs");
const competitionMod = read("crates/persistence-postgres/src/adapters/competition/mod.rs");

for (const relative of [
  `${base}/mod.rs`,
  `${base}/record_row.rs`,
  `${base}/record_mapper.rs`,
  `${base}/package_route_metadata.rs`,
  `${base}/list_bindings.rs`,
  `${base}/read_binding.rs`,
  `${base}/create_binding/mod.rs`,
  `${base}/create_binding/validation.rs`,
  `${base}/create_binding/insert_binding.rs`,
  `${base}/create_binding/transaction.rs`,
  `${base}/ensure_type_default/mod.rs`,
  `${base}/ensure_type_default/find_existing_binding.rs`,
  `${base}/ensure_type_default/insert_binding.rs`,
  `${base}/ensure_type_default/transaction.rs`,
  "crates/persistence-postgres/tests/competition_bindings_repository_contract.rs",
]) check(exists(relative), `Missing R5-04 owner: ${relative}`);

check(competitionMod.includes("mod bindings;"), "competition adapter root must register bindings");
check(!competitionMod.includes("route_resolution") && !competitionMod.includes("model_run_identity"), "R5-04 must not pre-implement R5-05/R5-06 owners");

for (const method of ["ensure_type_default_binding", "create_competition_binding", "list_competition_bindings", "read_binding", "package_route_metadata", "binding_summary_from_row", "binding_list_query"]) {
  check(!legacy.includes(method), `Legacy routing.rs still owns R5-04 responsibility ${method}`);
}
for (const required of ["pub async fn resolve_route", "fn route_decision_from_row", "pub async fn register_model", "pub(crate) async fn register_model_in_tx"]) {
  check(legacy.includes(required), `R5-04 must leave later owner in routing.rs: ${required}`);
}
check(contextOwner.includes("pub async fn resolve_competition_context"), "R5-04 must leave R5-05 competition context owner untouched");
check(contextOwner.includes("fn ensure_scope_id"), "R5-04 must leave R5-05 scope validation untouched");

const row = read(`${base}/record_row.rs`);
const mapper = read(`${base}/record_mapper.rs`);
check(row.includes("derive(Debug, FromRow)") && row.includes("struct BindingRow"), "Binding persistence must use typed sqlx::FromRow");
check(!row.includes("PgRow") && !mapper.includes("PgRow"), "R5-04 Binding mapping must not use dynamic PgRow");
check(mapper.includes("CompetitionBindingSummary") && mapper.includes("parse_competition_kind"), "Binding mapper must own typed Row -> Domain conversion and preserve CompetitionKind parsing");
check(!mapper.includes("sqlx::query"), "Binding mapper must not own SQL");

const packageMetadata = read(`${base}/package_route_metadata.rs`);
check(count(packageMetadata, "sqlx::query_as") === 1 && packageMetadata.includes("FROM model.rule_packages") && packageMetadata.includes("status = 'active'"), "package_route_metadata must own one active-rule-package SELECT");
for (const message of ["规则包缺少模型版本", "规则包缺少参数版本", "规则包缺少赛事类型"]) {
  check(packageMetadata.includes(message), `package_route_metadata must preserve error: ${message}`);
}

const list = read(`${base}/list_bindings.rs`);
check(count(list, "sqlx::query_as") === 1, "list_competition_bindings must own exactly one SQL purpose");
for (const token of ["b.is_active = true", "b.valid_from IS NULL OR b.valid_from <= now()", "b.valid_to IS NULL OR b.valid_to >= now()", "ORDER BY b.priority DESC, b.created_at DESC, b.id DESC"]) {
  check(list.includes(token), `Binding list must preserve ${token}`);
}
const detail = read(`${base}/read_binding.rs`);
check(count(detail, "sqlx::query_as") === 1 && detail.includes("WHERE b.id = $1"), "read_binding must own one detail SELECT");
check(!detail.includes("valid_from") && !detail.includes("valid_to"), "read_binding must preserve legacy no-validity-filter semantics");

const validation = read(`${base}/create_binding/validation.rs`);
check(!validation.includes("sqlx::"), "Binding validation must not own SQL");
for (const message of ["绑定范围不能为空；至少指定赛事、赛季、阶段或赛事类型", "绑定结束时间不能早于开始时间"]) {
  check(validation.includes(message), `Binding validation must preserve error: ${message}`);
}
const createInsert = read(`${base}/create_binding/insert_binding.rs`);
check(count(createInsert, "sqlx::query(") === 1 && createInsert.includes("INSERT INTO model.competition_bindings"), "Binding create insert helper must own exactly one INSERT");
const createTx = read(`${base}/create_binding/transaction.rs`);
check(!createTx.includes("sqlx::query"), "Binding create transaction must orchestrate without embedding SQL");
for (const token of ["validate_binding_draft", "resolve_competition_context", "package_route_metadata", "insert_binding", "write_audit_event", '"competition_binding_created"', "tx.commit().await?", "read_binding(self, id).await"]) {
  check(createTx.includes(token), `Binding create transaction missing ${token}`);
}
for (const message of ["绑定赛事类型 {} 与赛事层级解析结果 {} 不一致", "赛事类型默认绑定缺少赛事类型", "规则包赛事类型 {} 不能绑定到 {}", "赛事规则绑定-"]) {
  check(createTx.includes(message), `Binding create transaction must preserve behavior: ${message}`);
}

const defaultFind = read(`${base}/ensure_type_default/find_existing_binding.rs`);
check(count(defaultFind, "sqlx::query_scalar") === 1 && defaultFind.includes("competition_id IS NULL") && defaultFind.includes("season_id IS NULL") && defaultFind.includes("stage_id IS NULL") && defaultFind.includes("ORDER BY priority DESC, created_at DESC"), "Type-default lookup must preserve one idempotency SELECT");
const defaultInsert = read(`${base}/ensure_type_default/insert_binding.rs`);
check(count(defaultInsert, "sqlx::query(") === 1 && defaultInsert.includes("INSERT INTO model.competition_bindings"), "Type-default insert helper must own exactly one INSERT");
const defaultTx = read(`${base}/ensure_type_default/transaction.rs`);
check(!defaultTx.includes("sqlx::query"), "Type-default transaction must orchestrate without embedding SQL");
for (const token of ["package_route_metadata", "find_existing_binding", "insert_binding", "write_audit_event", '"type_default_binding_created"', "tx.commit().await?"]) {
  check(defaultTx.includes(token), `Type-default transaction missing ${token}`);
}
check(defaultTx.includes("规则包赛事类型 {} 不能作为 {} 的默认规则"), "Type-default transaction must preserve package-kind mismatch error");

const contract = read("crates/persistence-postgres/tests/competition_bindings_repository_contract.rs");
for (const token of ["ensure_type_default_binding", "create_competition_binding", "list_competition_bindings", "resolve_route", "StageBinding", "type_default_binding_created", "competition_binding_created", "valid_from", "valid_to"]) {
  check(contract.includes(token), `R5-04 PostgreSQL contract missing ${token}`);
}

const repositoryGate = read("scripts/verify-competition-repository.mjs");
const packageJson = JSON.parse(read("package.json"));
check(repositoryGate.includes('import "./verify-competition-bindings.mjs";'), "R5 competition gate must chain R5-04 Binding verifier");
check(packageJson.scripts["verify:architecture"].includes("verify-competition-repository.mjs"), "verify:architecture must reach the R5-04 verifier through competition gate chain");

if (failures.length) {
  console.error("R5-04 Competition Bindings verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R5-04 Competition Bindings verified: typed binding persistence has one owner, legacy Binding CRUD/helpers are removed, and R5-05 route/context plus R5-06 model registration remain untouched.");
