from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
APP_ROOT = ROOT / "crates/application/src"


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    (ROOT / path).write_text(text, encoding="utf-8", newline="\n")


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected 1 exact match, found {count}")
    return text.replace(old, new, 1)


# Keep the concrete type visible only inside composition. DatabaseSession is a zero-cost,
# crate-private alias: it restores the Application-owned session vocabulary without
# recreating the removed ActiveDatabase wrapper or transition_store forwarding layer.
registry_path = "crates/application/src/composition/port_registry.rs"
registry = read(registry_path)
registry = replace_once(
    registry,
    "use crate::ports::PortResult;\n\npub(crate) struct PortRegistry;",
    "use crate::ports::PortResult;\n\npub(crate) type DatabaseSession = PersistenceStore;\n\npub(crate) struct PortRegistry;",
    "DatabaseSession alias owner",
)
registry = replace_once(
    registry,
    ") -> PortResult<PersistenceStore> {",
    ") -> PortResult<DatabaseSession> {",
    "PortRegistry session return type",
)
write(registry_path, registry)

# All non-adapter Application code originally referenced ActiveDatabase as an Application
# session boundary. The earlier mechanical candidate rewrote those references to the
# concrete PersistenceStore; convert only those generated concrete references back to the
# Application-owned DatabaseSession alias.
changed_paths: list[str] = []
for path in APP_ROOT.rglob("*.rs"):
    relative = path.relative_to(ROOT).as_posix()
    if relative.startswith("crates/application/src/composition/adapters/"):
        continue
    if relative == registry_path:
        continue
    text = path.read_text(encoding="utf-8")
    if "PersistenceStore" not in text:
        continue
    updated = text.replace("PersistenceStore", "DatabaseSession")
    path.write_text(updated, encoding="utf-8", newline="\n")
    changed_paths.append(relative)

expected_boundary_files = {
    "crates/application/src/composition/mod.rs",
    "crates/application/src/services/analytics/facade.rs",
    "crates/application/src/services/prediction/facade.rs",
    "crates/application/src/services/release/facade.rs",
    "crates/application/src/services/teams/facade.rs",
    "crates/application/src/services/lineups/facade.rs",
    "crates/application/src/services/exchange/facade/mod.rs",
    "crates/application/src/services/postmatch/facade.rs",
    "crates/application/src/services/ai_workspace/facade/mod.rs",
    "crates/application/src/services/review/facade.rs",
    "crates/application/src/services/research/facade.rs",
    "crates/application/src/services/players/facade.rs",
    "crates/application/src/services/database/service.rs",
    "crates/application/src/use_cases/application_facade/database_lifecycle/initialize.rs",
}
actual = set(changed_paths)
if actual != expected_boundary_files:
    raise RuntimeError(
        "DatabaseSession boundary rewrite file set mismatch; "
        f"expected={sorted(expected_boundary_files)}, actual={sorted(actual)}"
    )

# composition/mod.rs must expose DatabaseSession, not the concrete PersistenceStore alias.
composition_mod = read("crates/application/src/composition/mod.rs")
if "DatabaseSession" not in composition_mod:
    raise RuntimeError("composition/mod.rs did not re-export DatabaseSession")
if "PersistenceStore" in composition_mod:
    raise RuntimeError("composition/mod.rs still exposes concrete PersistenceStore")

# Migrate the Database Service gate to the Application session vocabulary while preserving
# concrete adapter assertions in composition/adapters/database.rs.
db_gate_path = "scripts/verify-database-service.mjs"
db_gate = read(db_gate_path)
db_gate = replace_once(
    db_gate,
    'check(service.includes("pub(crate) session: RwLock<Option<PersistenceStore>>"), "活动数据库状态未归属 DatabaseService");',
    'check(service.includes("pub(crate) session: RwLock<Option<DatabaseSession>>"), "活动数据库状态未归属 DatabaseService");',
    "Database Service session gate",
)
db_gate = replace_once(
    db_gate,
    'check(!applicationService.includes("RwLock<Option<PersistenceStore>>"), "ApplicationService 仍直接持有活动数据库槽位");',
    'check(!applicationService.includes("RwLock<Option<DatabaseSession>>"), "ApplicationService 仍直接持有活动数据库槽位");',
    "ApplicationService session ownership gate",
)
insert_anchor = 'check(portRegistry.includes("PostgresStore as PersistenceStore"), "PostgreSQL 具体适配器入口缺失");\n'
insert = insert_anchor + 'check(portRegistry.includes("pub(crate) type DatabaseSession = PersistenceStore;"), "Application DatabaseSession 边界别名缺失");\n'
if db_gate.count(insert_anchor) != 1:
    raise RuntimeError(f"Database Service alias gate anchor expected once, found {db_gate.count(insert_anchor)}")
db_gate = db_gate.replace(insert_anchor, insert, 1)
db_gate = db_gate.replace("活动 PostgresStore 由 DatabaseService 单一持有", "活动 DatabaseSession 由 DatabaseService 单一持有")
write(db_gate_path, db_gate)

# Migrate Application composition state assertion to the same alias. Adapter target checks
# remain PersistenceStore and are intentionally not changed here.
composition_gate_path = "scripts/verify-application-composition.mjs"
composition_gate = read(composition_gate_path)
composition_gate = replace_once(
    composition_gate,
    'databaseService.includes("pub(crate) session: RwLock<Option<PersistenceStore>>"),',
    'databaseService.includes("pub(crate) session: RwLock<Option<DatabaseSession>>"),',
    "Application composition session gate",
)
write(composition_gate_path, composition_gate)

# Strengthen R4-04's own verifier: only composition adapters/port_registry may see the
# concrete PersistenceStore name; services/use_cases must use DatabaseSession.
adapter_verifier_path = "scripts/verify-persistence-adapters.mjs"
verifier = read(adapter_verifier_path)
registry_anchor = 'check(portRegistry.includes("use football_persistence_postgres::register_adapters;"), "PortRegistry must consume persistence register_adapters entry");\n'
registry_insert = registry_anchor + 'check(portRegistry.includes("pub(crate) type DatabaseSession = PersistenceStore;"), "PortRegistry must own the zero-cost DatabaseSession alias");\n'
if verifier.count(registry_anchor) != 1:
    raise RuntimeError(f"R4-04 registry alias verifier anchor expected once, found {verifier.count(registry_anchor)}")
verifier = verifier.replace(registry_anchor, registry_insert, 1)
verifier = replace_once(
    verifier,
    'check(portRegistry.includes("-> PortResult<PersistenceStore>"), "PortRegistry must return the concrete PostgresStore adapter session");',
    'check(portRegistry.includes("-> PortResult<DatabaseSession>"), "PortRegistry must return the Application DatabaseSession boundary");',
    "R4-04 registry return verifier",
)
source_anchor = 'check(!combinedApplication.includes("transition_store"), "transition_store forwarding path must be fully removed");\n'
source_insert = source_anchor + '''const nonCompositionConcreteLeaks = appFiles
  .filter((file) => !file.replaceAll("\\\\", "/").includes("crates/application/src/composition/"))
  .filter((file) => {
    const source = fs.readFileSync(file, "utf8");
    return ["PersistenceStore", "PostgresStore", "football_persistence_postgres"].some((token) => source.includes(token));
  });
check(nonCompositionConcreteLeaks.length === 0, `Application services/use_cases leak concrete persistence: ${nonCompositionConcreteLeaks.map((f) => path.relative(root, f)).join(", ")}`);
const databaseService = read("crates/application/src/services/database/service.rs");
check(databaseService.includes("RwLock<Option<DatabaseSession>>"), "DatabaseService must own the Application DatabaseSession boundary");
'''
if verifier.count(source_anchor) != 1:
    raise RuntimeError(f"R4-04 concrete leak verifier anchor expected once, found {verifier.count(source_anchor)}")
verifier = verifier.replace(source_anchor, source_insert, 1)
# r4-04-run.py already added a databaseService const later. Remove the duplicate local declaration.
verifier = verifier.replace(
    'const persistenceError = read("crates/application/src/composition/adapters/persistence_error.rs");\nconst databaseService = read("crates/application/src/services/database/service.rs");\n',
    'const persistenceError = read("crates/application/src/composition/adapters/persistence_error.rs");\n',
    1,
)
write(adapter_verifier_path, verifier)

# Final source-level guard: concrete persistence is allowed only under composition.
violations: list[str] = []
for path in APP_ROOT.rglob("*.rs"):
    relative = path.relative_to(ROOT).as_posix()
    text = path.read_text(encoding="utf-8")
    if "ActiveDatabase" in text or "transition_store" in text:
        violations.append(f"legacy:{relative}")
    if not relative.startswith("crates/application/src/composition/"):
        for token in ("PersistenceStore", "PostgresStore", "football_persistence_postgres"):
            if token in text:
                violations.append(f"concrete:{relative}:{token}")
if violations:
    raise RuntimeError("Application session boundary violations remain: " + ", ".join(violations))

print("R4-04 Application session boundary fixed: DatabaseSession alias replaces the removed wrapper outside composition; 40 concrete Port impls remain on PersistenceStore")
