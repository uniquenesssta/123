import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (relative) => fs.readFileSync(path.join(root, relative), "utf8");
const walk = (dir) => fs.readdirSync(dir, { withFileTypes: true }).flatMap((entry) => {
  const full = path.join(dir, entry.name);
  return entry.isDirectory() ? walk(full) : [full];
});
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const persistenceAdapters = "crates/persistence-postgres/src/adapters";
const adapterFiles = fs.readdirSync(path.join(root, persistenceAdapters)).filter((name) => name.endsWith(".rs")).sort();
check(JSON.stringify(adapterFiles) === JSON.stringify(["mod.rs", "register_adapters.rs"]), `Persistence adapters root must contain only mod.rs/register_adapters.rs, got ${adapterFiles.join(", ")}`);
const pmod = read(`${persistenceAdapters}/mod.rs`);
const registration = read(`${persistenceAdapters}/register_adapters.rs`);
const plib = read("crates/persistence-postgres/src/lib.rs");
const store = read("crates/persistence-postgres/src/store/postgres_store.rs");
check(pmod.includes("mod register_adapters;") && pmod.includes("pub use register_adapters::register_adapters;"), "adapters/mod.rs must only expose register_adapters");
check(registration.includes("PostgresStore::connect(options).await"), "register_adapters must delegate concrete connection creation to PostgresStore");
check(!registration.includes("sqlx::") && !registration.includes("football_domain"), "register_adapters must not own SQL or domain behavior");
check(plib.includes("mod adapters;") && plib.includes("pub use adapters::register_adapters;"), "persistence lib must expose the adapter registration entry");
check(store.includes("redacted_url: String") && store.includes("pub fn redacted_url(&self) -> &str") && store.includes("let redacted_url = options.redacted_url();"), "PostgresStore must own active connection redaction metadata after wrapper removal");

const appRoot = path.join(root, "crates/application/src");
const appFiles = walk(appRoot).filter((file) => file.endsWith(".rs"));
const concreteImports = appFiles.filter((file) => fs.readFileSync(file, "utf8").includes("football_persistence_postgres"));
check(concreteImports.length === 1 && concreteImports[0].replaceAll("\\", "/").endsWith("crates/application/src/composition/port_registry.rs"), `Concrete persistence import must remain composition-only; got ${concreteImports.map((f) => path.relative(root, f)).join(", ")}`);
const combinedApplication = appFiles.map((file) => fs.readFileSync(file, "utf8")).join("\n");
check(!combinedApplication.includes("ActiveDatabase"), "ActiveDatabase wrapper must be fully removed");
check(!combinedApplication.includes("transition_store"), "transition_store forwarding path must be fully removed");
const nonCompositionConcreteLeaks = appFiles
  .filter((file) => !file.replaceAll("\\", "/").includes("crates/application/src/composition/"))
  .filter((file) => {
    const source = fs.readFileSync(file, "utf8");
    return ["PersistenceStore", "PostgresStore", "football_persistence_postgres"].some((token) => source.includes(token));
  });
check(nonCompositionConcreteLeaks.length === 0, `Application services/use_cases leak concrete persistence: ${nonCompositionConcreteLeaks.map((f) => path.relative(root, f)).join(", ")}`);
const databaseService = read("crates/application/src/services/database/service.rs");
check(databaseService.includes("RwLock<Option<DatabaseSession>>"), "DatabaseService must own the Application DatabaseSession boundary");

const portRegistry = read("crates/application/src/composition/port_registry.rs");
check(portRegistry.includes("use football_persistence_postgres::register_adapters;"), "PortRegistry must consume persistence register_adapters entry");
check(portRegistry.includes("pub(crate) type DatabaseSession = PersistenceStore;"), "PortRegistry must own the zero-cost DatabaseSession alias");
check(portRegistry.includes("-> PortResult<DatabaseSession>"), "PortRegistry must return the Application DatabaseSession boundary");
check(/register_adapters\(options\)\s*\.await\s*\.map_err\(map_persistence_error\)/s.test(portRegistry), "PortRegistry must use the unique adapter registration entry");
check(!portRegistry.includes("#[async_trait]") && !/impl\s+\w+Port\s+for/.test(portRegistry), "port_registry.rs must not own Port implementations");
check(!portRegistry.includes("PersistenceStore::connect"), "Application composition must not bypass register_adapters");
const persistenceError = read("crates/application/src/composition/adapters/persistence_error.rs");
check(persistenceError.includes("pub(in crate::composition) fn map_persistence_error"), "Persistence error mapper must be visible only across the composition boundary");
check(databaseService.includes("DatabaseLifecyclePort::close(&previous).await?;"), "Database replacement must preserve fallible lifecycle close semantics");
check(databaseService.includes("DatabaseLifecyclePort::close(&active).await?;"), "Database disconnect must preserve fallible lifecycle close semantics");

const adapterRoot = path.join(root, "crates/application/src/composition/adapters");
const applicationAdapterFiles = walk(adapterRoot).filter((file) => file.endsWith(".rs"));
const adapterCombined = applicationAdapterFiles.map((file) => fs.readFileSync(file, "utf8")).join("\n");
const implMatches = adapterCombined.match(/impl\s+(?:(?:crate::ports::prediction::)?[A-Za-z0-9_]+Port)\s+for\s+PersistenceStore/g) ?? [];
check(implMatches.length === 40, `Expected 40 PostgreSQL-backed Application Port impls targeting PersistenceStore, got ${implMatches.length}`);
check(!adapterCombined.includes("for ActiveDatabase"), "No adapter Port impl may target the removed ActiveDatabase wrapper");
check(!adapterCombined.includes("transition_store"), "No adapter may retain transition_store forwarding");
for (const required of ["database.rs", "competition.rs", "rules.rs", "persistence_error.rs"]) {
  check(fs.existsSync(path.join(adapterRoot, required)), `Missing extracted application adapter owner: ${required}`);
}

const packageJson = JSON.parse(read("package.json"));
check(typeof packageJson.scripts["verify:persistence-adapters"] === "string", "package.json must expose verify:persistence-adapters");
check(packageJson.scripts["verify:architecture"].includes("verify-persistence-adapters.mjs"), "verify:architecture must include R4-04 adapter gate");

if (failures.length) {
  console.error("R4-04 Port Adapter registration verification failed:");
  failures.forEach((failure) => console.error(`- ${failure}`));
  process.exit(1);
}
console.log("R4-04 Port Adapter registration verified: PostgresStore is the 40-Port concrete target, ActiveDatabase forwarding is removed, and register_adapters is the composition registration entry.");
