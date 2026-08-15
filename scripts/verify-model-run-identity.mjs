import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const exists = (relative) => fs.existsSync(path.join(root, relative));
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };
const count = (source, token) => source.split(token).length - 1;

const base = "crates/persistence-postgres/src/adapters/competition/model_run_identity";
const registrationBase = `${base}/registration`;
const legacyRouting = "crates/persistence-postgres/src/routing.rs";

for (const relative of [
  `${base}/mod.rs`,
  `${base}/read_identity.rs`,
  `${base}/record.rs`,
  `${base}/record_row.rs`,
  `${registrationBase}/mod.rs`,
  `${registrationBase}/record.rs`,
  `${registrationBase}/transaction.rs`,
  `${registrationBase}/definition.rs`,
  `${registrationBase}/version.rs`,
  `${registrationBase}/parameter_set.rs`,
  "crates/persistence-postgres/tests/model_run_identity_repository_contract.rs",
]) check(exists(relative), `Missing R5-06 Model Run Identity owner: ${relative}`);

check(!exists(legacyRouting), "R5-06 must delete legacy routing.rs after model registration owner switch");

const competitionMod = read("crates/persistence-postgres/src/adapters/competition/mod.rs");
const adaptersMod = read("crates/persistence-postgres/src/adapters/mod.rs");
const library = read("crates/persistence-postgres/src/lib.rs");
check(competitionMod.includes("model_run_identity"), "competition adapter root must register model_run_identity");
check(competitionMod.includes("register_model_in_tx") && competitionMod.includes("ModelRegistration"), "competition adapter root must expose the R5-06 registration boundary");
check(adaptersMod.includes("competition::register_model_in_tx") && adaptersMod.includes("competition::ModelRegistration"), "adapter root must expose the shared registration boundary without legacy routing");
check(!library.includes("mod routing;"), "persistence lib must not register removed routing.rs");
check(library.includes("ModelRegistration"), "persistence public surface must preserve ModelRegistration export");

const identityMod = read(`${base}/mod.rs`);
check(!identityMod.includes("sqlx::") && !identityMod.includes("SELECT "), "model_run_identity/mod.rs must remain an explicit export-only boundary");
const identityRow = read(`${base}/record_row.rs`);
const identityRecord = read(`${base}/record.rs`);
const identityRead = read(`${base}/read_identity.rs`);
check(identityRow.includes("derive(Debug, FromRow)") && identityRow.includes("ModelRunIdentityRow"), "R5-06 identity read must use typed sqlx::FromRow");
check(!identityRow.includes("PgRow") && !identityRecord.includes("PgRow"), "R5-06 identity mapping must not use dynamic PgRow");
check(!identityRecord.includes("sqlx::query") && identityRecord.includes("From<ModelRunIdentityRow>"), "identity record mapper must remain pure typed Row -> record conversion");
check(count(identityRead, "sqlx::query_as") === 1 && identityRead.includes("JOIN model.versions") && identityRead.includes("JOIN model.definitions") && identityRead.includes("JOIN model.parameter_sets") && identityRead.includes("LEFT JOIN model.rule_packages"), "read_model_run_identity must own exactly one complete identity SELECT");
check(identityRead.includes("WHERE r.id = $1"), "identity read must preserve exact run-id lookup");

const registrationMod = read(`${registrationBase}/mod.rs`);
const registrationRecord = read(`${registrationBase}/record.rs`);
const registrationTx = read(`${registrationBase}/transaction.rs`);
const definition = read(`${registrationBase}/definition.rs`);
const version = read(`${registrationBase}/version.rs`);
const parameterSet = read(`${registrationBase}/parameter_set.rs`);
check(!registrationMod.includes("sqlx::") && !registrationMod.includes("INSERT "), "registration/mod.rs must remain export-only");
check(registrationRecord.includes("pub struct ModelRegistration") && registrationRecord.includes("model_version_id") && registrationRecord.includes("parameter_set_id"), "ModelRegistration public contract must remain unchanged");
check(!registrationTx.includes("sqlx::query") && registrationTx.includes("self.pool.begin().await?") && registrationTx.includes("tx.commit().await?") && registrationTx.includes("upsert_model_definition") && registrationTx.includes("register_model_version") && registrationTx.includes("register_parameter_set"), "model registration transaction must coordinate helpers without embedding SQL");
check(count(definition, "sqlx::query_scalar") === 1 && definition.includes("INSERT INTO model.definitions") && definition.includes("ON CONFLICT (model_key) DO UPDATE"), "model definition helper must own one upsert purpose");
check(count(version, "sqlx::query_scalar") === 1 && count(version, "sqlx::query_as") === 1 && version.includes("INSERT INTO model.versions") && version.includes("ON CONFLICT (model_id, version) DO NOTHING") && version.includes("derive(Debug, FromRow)"), "model version helper must own one insert plus one typed conflict read");
check(version.includes("模型版本 {model_version} 已存在但引擎或 Schema 不一致；请创建新模型版本"), "model version compatibility error semantics changed");
check(count(parameterSet, "sqlx::query_scalar") === 1 && count(parameterSet, "sqlx::query_as") === 1 && parameterSet.includes("INSERT INTO model.parameter_sets") && parameterSet.includes("ON CONFLICT (model_version_id, parameter_version) DO NOTHING") && parameterSet.includes("derive(Debug, FromRow)"), "parameter-set helper must own one insert plus one typed conflict read");
check(parameterSet.includes("sha256_json(parameters)?") && parameterSet.includes("参数版本 {parameter_version} 已存在但内容不同；请创建新参数版本"), "parameter-set hash/conflict semantics changed");

const ruleTransaction = read("crates/persistence-postgres/src/adapters/rules/packages/register_rule_package/transaction.rs");
const ruleInsert = read("crates/persistence-postgres/src/adapters/rules/packages/register_rule_package/insert_rule_package.rs");
check(ruleTransaction.includes("adapters::register_model_in_tx") && !ruleTransaction.includes("routing::register_model_in_tx"), "Rule Package transaction must consume the new R5-06 registration boundary");
check(ruleInsert.includes("adapters::ModelRegistration") && !ruleInsert.includes("routing::ModelRegistration"), "Rule Package insert must consume the new ModelRegistration export");

const modelRuns = read("crates/persistence-postgres/src/model_runs.rs");
const readRunStart = modelRuns.indexOf("pub async fn read_run");
const readRunEnd = modelRuns.indexOf("async fn save_model_details", readRunStart);
const readRun = readRunStart >= 0 && readRunEnd > readRunStart ? modelRuns.slice(readRunStart, readRunEnd) : "";
check(readRun.includes("read_model_run_identity(self, run_id).await?"), "read_run must delegate model identity lookup to R5-06 owner");
check(!readRun.includes("JOIN model.versions") && !readRun.includes("JOIN model.definitions") && !readRun.includes("JOIN model.parameter_sets") && !readRun.includes("JOIN model.rule_packages"), "read_run must not retain duplicate model identity joins");
for (const token of ["identity.model_key", "identity.model_version", "identity.parameter_version", "identity.rule_package_id", "identity.rule_package_key", "identity.rule_package_version", "identity.rule_package_name", "identity.route_binding_id"]) {
  check(readRun.includes(token), `read_run public JSON identity mapping missing ${token}`);
}

const contract = read("crates/persistence-postgres/tests/model_run_identity_repository_contract.rs");
for (const token of [
  "ModelRegistration", "read_run", "route_binding_id", "rule_package_key",
  "rule_package_version", "rule_package_name", "rule_package_id",
]) check(contract.includes(token), `R5-06 PostgreSQL read contract missing ${token}`);
const rulePackageContract = read("crates/persistence-postgres/tests/rule_package_repository_contract.rs");
check(rulePackageContract.includes("register_rule_package"), "Existing Rule Package PostgreSQL contract must continue exercising shared model registration transaction behavior");

if (failures.length) {
  console.error("R5-06 Model Run Identity verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R5-06 Model Run Identity verified: run identity reads and model registration have unique typed owners, Rule Package registration consumes the new boundary, and legacy routing.rs is removed.");
