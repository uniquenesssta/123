import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const count = (source, token) => source.split(token).length - 1;

const base = "crates/persistence-postgres/src/adapters/rules/packages";
const legacyPath = "crates/persistence-postgres/src/routing.rs";
const legacy = exists(legacyPath) ? read(legacyPath) : "";
const adaptersMod = read("crates/persistence-postgres/src/adapters/mod.rs");
const rulesMod = read("crates/persistence-postgres/src/adapters/rules/mod.rs");

for (const relative of [
  "crates/persistence-postgres/src/adapters/rules/mod.rs",
  `${base}/mod.rs`,
  `${base}/list_rule_packages.rs`,
  `${base}/record_row.rs`,
  `${base}/record_mapper.rs`,
  `${base}/register_rule_package/mod.rs`,
  `${base}/register_rule_package/transaction.rs`,
  `${base}/register_rule_package/insert_rule_package.rs`,
  `${base}/register_rule_package/find_existing_package.rs`,
  `${base}/register_rule_package/attach_competition_profile.rs`,
  `${base}/source_documents/mod.rs`,
  `${base}/source_documents/upsert_source_document.rs`,
  "crates/persistence-postgres/tests/rule_package_repository_contract.rs",
]) check(exists(relative), `Missing R5-03 Rule Package owner: ${relative}`);

check(adaptersMod.includes("mod rules;"), "Persistence adapters root must register rules");
check(rulesMod.includes("mod packages;"), "rules adapter root must register packages");
for (const later of ["bindings", "route_resolution", "model_run_identity"]) {
  check(!exists(`crates/persistence-postgres/src/adapters/rules/${later}`), `R5-03 rules adapter must not own later responsibility ${later}`);
}

for (const method of ["register_rule_package", "list_rule_packages"]) {
  check(!new RegExp(`pub\\s+async\\s+fn\\s+${method}\\b`).test(legacy), `Legacy routing.rs still owns ${method}`);
}
check(!legacy.includes("register_rule_source_document"), "Legacy routing.rs still owns source-document registration");
check(!legacy.includes("rule_package_summary_from_row"), "Legacy routing.rs still owns dynamic RulePackage mapper");

for (const migrated of [
  "ensure_type_default_binding",
  "create_competition_binding",
  "list_competition_bindings",
  "package_route_metadata",
]) check(!legacy.includes(migrated), `R5-04 Binding responsibility must be removed from routing.rs: ${migrated}`);
check(exists("crates/persistence-postgres/src/adapters/competition/bindings/mod.rs"), "R5-04 Binding owner must exist in competition adapters");

for (const migrated of ["resolve_route", "route_decision_from_row"]) {
  check(!legacy.includes(migrated), `R5-05 route responsibility must be removed from routing.rs: ${migrated}`);
}
check(!exists(legacyPath), "R5-06 must remove legacy routing.rs after model registration owner switch");
check(exists("crates/persistence-postgres/src/adapters/competition/model_run_identity/registration/mod.rs"), "R5-06 model registration owner must exist under model_run_identity");

const row = read(`${base}/record_row.rs`);
const mapper = read(`${base}/record_mapper.rs`);
const list = read(`${base}/list_rule_packages.rs`);
check(row.includes("derive(Debug, FromRow)") && row.includes("struct RulePackageRow"), "Rule Package list must use typed sqlx::FromRow");
check(!row.includes("PgRow") && !mapper.includes("PgRow"), "Rule Package mapping must not use dynamic PgRow");
check(mapper.includes("RulePackageSummary") && mapper.includes("parse_competition_kind"), "Rule Package mapper must own typed Row -> Domain conversion and preserve CompetitionKind parsing");
check(!mapper.includes("sqlx::query"), "Rule Package mapper must not own SQL");
check(count(list, "sqlx::query_as") === 1 && list.includes("ORDER BY rp.created_at DESC, rp.package_key, rp.version"), "Rule Package list must preserve one SELECT purpose and ordering");

const transaction = read(`${base}/register_rule_package/transaction.rs`);
check(!transaction.includes("sqlx::query") && transaction.includes("self.pool.begin().await?") && transaction.includes("tx.commit().await?") && transaction.includes("upsert_rule_source_document") && transaction.includes("register_model_in_tx") && transaction.includes("register_competition_profile_in_tx") && transaction.includes("write_audit_event"), "Rule Package transaction must orchestrate helpers/audit without embedding SQL");
check(transaction.includes("adapters::register_model_in_tx") && !transaction.includes("routing::register_model_in_tx"), "Rule Package transaction must consume the approved R5-06 registration boundary");
check(transaction.includes("已存在但内容不同") && transaction.includes("已绑定不同赛事Profile版本") && transaction.includes("created_at: Utc::now()"), "Rule Package transaction must preserve conflict and returned-created_at semantics");

const insert = read(`${base}/register_rule_package/insert_rule_package.rs`);
const existing = read(`${base}/register_rule_package/find_existing_package.rs`);
const attach = read(`${base}/register_rule_package/attach_competition_profile.rs`);
const source = read(`${base}/source_documents/upsert_source_document.rs`);
check(count(insert, "sqlx::query_scalar") === 1 && insert.includes("INSERT INTO model.rule_packages") && insert.includes("ON CONFLICT (package_key, version) DO NOTHING"), "Rule Package insert helper must own exactly one INSERT purpose");
check(count(existing, "sqlx::query_as") === 1 && existing.includes("WHERE package_key = $1 AND version = $2") && existing.includes("derive(Debug, FromRow)"), "existing-package helper must own one typed conflict SELECT");
check(count(attach, "sqlx::query(") === 1 && attach.includes("competition_profile_id IS NULL"), "profile attach helper must own exactly one guarded UPDATE");
check(count(source, "sqlx::query_scalar") === 1 && source.includes("INSERT INTO catalog.source_documents") && source.includes("ON CONFLICT (content_sha256) DO UPDATE") && source.includes("COALESCE(catalog.source_documents.source_uri, EXCLUDED.source_uri)") && source.includes("catalog.source_documents.metadata || EXCLUDED.metadata"), "source-document helper must preserve content hash upsert semantics");

const contract = read("crates/persistence-postgres/tests/rule_package_repository_contract.rs");
for (const token of [
  "register_rule_package", "list_rule_packages", "已存在但内容不同",
  "source_documents", "rule_package_registered", "created_at", "content_sha256",
]) check(contract.includes(token), `R5-03 PostgreSQL contract missing ${token}`);

const repositoryGate = read("scripts/verify-competition-repository.mjs");
const packageJson = JSON.parse(read("package.json"));
check(repositoryGate.includes('import "./verify-rule-package-repository.mjs";'), "R5 competition gate must chain the R5-03 Rule Package gate");
check(packageJson.scripts["verify:architecture"].includes("verify-competition-repository.mjs"), "verify:architecture must reach the R5-03 gate through the existing R5 chain");

if (failures.length) {
  console.error("R5-03 Rule Package Repository verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R5-03 Rule Package Repository verified: package persistence remains unique and consumes the approved R5-06 model registration boundary after legacy routing removal.");
