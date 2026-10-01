import fs from "node:fs";
const read = (p) => fs.readFileSync(new URL(`../${p}`, import.meta.url), "utf8").replace(/\r\n?/g, "\n");
const exists = (p) => fs.existsSync(new URL(`../${p}`, import.meta.url));
const req = (ok, msg) => { if (!ok) throw new Error(msg); };
const entityPath = "crates/persistence-postgres/src/entity_catalog.rs";
const entity = exists(entityPath) ? read(entityPath) : "";
const player = read("crates/persistence-postgres/src/player_catalog.rs");
const catalog = read("crates/persistence-postgres/src/adapters/catalog/mod.rs");
const matchingRoot = "crates/persistence-postgres/src/adapters/catalog/entity_matching/";
const referencesRoot = "crates/persistence-postgres/src/adapters/catalog/references/";
const files = [];
const walk = (base, rel="") => { for (const e of fs.readdirSync(new URL(`../${base}${rel}`, import.meta.url), {withFileTypes:true})) e.isDirectory() ? walk(base, `${rel}${e.name}/`) : files.push(`${base}${rel}${e.name}`); };
walk(matchingRoot); walk(referencesRoot);
const all = files.filter(p => p.endsWith(".rs")).map(read).join("\n");
req(catalog.includes("mod entity_matching;") && catalog.includes("pub(crate) mod references;"), "R6-08 catalog modules missing");
for (const name of ["list_entity_references", "resolve_entity_reference", "create_data_provider", "list_data_providers", "add_external_entity_id"]) {
  req((all.match(new RegExp(`pub async fn ${name}\\b`, "g")) || []).length === 1, `R6-08 owner count invalid: ${name}`);
}
for (const name of ["list_entity_references", "resolve_entity_reference"]) req(!entity.includes(`pub async fn ${name}`), `entity_catalog legacy owner remains: ${name}`);
for (const name of ["create_data_provider", "list_data_providers", "add_external_entity_id"]) req(!player.includes(`pub async fn ${name}`), `player_catalog legacy owner remains: ${name}`);
const deletionCheck = read("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/check.rs");
const deletionRefs = read("crates/persistence-postgres/src/adapters/catalog/deletion/preflight/references.rs");
const deletionArchive = read("crates/persistence-postgres/src/adapters/catalog/deletion/archive/bulk.rs");
req(!entity.includes("pub async fn check_entity_deletion") && !entity.includes("pub async fn bulk_archive_entities") && deletionCheck.includes("pub async fn check_entity_deletion") && deletionArchive.includes("pub async fn bulk_archive_entities"), "R6-09 AT1 deletion/archive ownership invalid");
req(!entity.includes("team_reference_counts") && deletionRefs.includes("team_reference_counts") && deletionRefs.includes("player_reference_counts") && deletionRefs.includes("coach_reference_counts"), "R6-09 AT1 reference-count ownership invalid");
const resolve = read(`${matchingRoot}resolve.rs`);
const list = read(`${referencesRoot}directory/list.rs`);
req(!resolve.includes("sqlx::") && !resolve.includes("SELECT ") && !list.includes("sqlx::") && !list.includes("SELECT "), "R6-08 coordinator owns SQL");
for (const path of [
  `${matchingRoot}existence.rs`, `${matchingRoot}external_id.rs`, `${matchingRoot}name_candidates.rs`,
  `${referencesRoot}directory/read.rs`, `${referencesRoot}directory/mapper.rs`,
  `${referencesRoot}providers/read.rs`, `${referencesRoot}providers/write.rs`, `${referencesRoot}providers/validation.rs`,
  `${referencesRoot}external_ids/write.rs`, `${referencesRoot}external_ids/validation.rs`,
]) req(exists(path), `R6-08 responsibility owner missing: ${path}`);
const matching = files.filter(p => p.includes("/entity_matching/")).map(read).join("\n");
for (const token of ["稳定实体 ID 精确匹配", "受信数据源外部 ID 精确匹配", "外部 ID 对应多条实体", "球队别名", "球员别名与出生日期", "教练别名与国籍", "status: \"ambiguous\""]) req(matching.includes(token), `matching contract missing: ${token}`);
const directoryRead = read(`${referencesRoot}directory/read.rs`);
req(directoryRead.includes("NameSearch::parse") && directoryRead.includes("push_name_search"), "reference directory lost unified NameSearch");
req(directoryRead.includes("query.limit.clamp(1, 500)") && directoryRead.includes("ORDER BY team.normalized_name, team.id") && directoryRead.includes("ORDER BY player.normalized_name, player.id") && directoryRead.includes("ORDER BY coach.normalized_name, coach.id"), "reference paging/order contract changed");
const providerValidation = read(`${referencesRoot}providers/validation.rs`);
const externalValidation = read(`${referencesRoot}external_ids/validation.rs`);
req(providerValidation.includes("数据源代码、名称和类型不能为空"), "provider validation error changed");
req(externalValidation.includes("外部 ID 实体类型无效") && externalValidation.includes("外部 ID 不能为空"), "external id validation errors changed");
const port = read("crates/application/src/ports/player/mod.rs");
for (const method of ["list_references", "resolve_reference", "check_deletion", "bulk_archive", "create_data_provider", "add_external_id"]) req(port.includes(`async fn ${method}`), `EntityReferencePort changed: ${method}`);
const externalWrite = read(`${referencesRoot}external_ids/write.rs`);
const spreadsheet = [read("crates/persistence-postgres/src/adapters/workbooks/player_catalog/commit.rs")].join("\n");
req(externalWrite.includes("pub(crate) async fn write_external_entity_id(") && externalWrite.includes("connection: &mut PgConnection"), "external ID transaction-aware owner missing");
req(externalWrite.includes("WHERE football.external_entity_ids.entity_id = EXCLUDED.entity_id") && externalWrite.includes(".fetch_optional(connection)") && externalWrite.includes("禁止自动改绑"), "external ID atomic conflict rejection missing");
req(!/SET[\s\S]*?entity_id\s*=\s*EXCLUDED\.entity_id\s*,/.test(externalWrite), "external ID target overwrite remains");
req(externalWrite.includes("write_external_entity_id(&mut connection, draft).await"), "direct external ID entry bypasses shared owner");
req(/write_external_entity_id\(\s*tx,[\s\S]*?\.await\?;/.test(spreadsheet), "import external ID entry lost shared transaction/error propagation");
req(!/INSERT\s+INTO\s+football\.external_entity_ids/i.test(spreadsheet), "import retains parallel external ID writer");
const contract = read("crates/persistence-postgres/tests/entity_matching_references_repository_contract.rs");
for (const token of ["external_id_identity_and_import_atomicity_are_preserved", "same entity retry", "Barrier::new(2)", "tokio::join!", "late conflict preview", "rolled back id count", "success audit count", "cleanup_external_id_fixture", "SELECT current_database()"])
  req(contract.includes(token), `external ID contract coverage missing: ${token}`);
console.log("R6-08 Entity Matching / References ownership and R7-02 external ID integrity verified.");
