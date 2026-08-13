from __future__ import annotations

import importlib.util
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HELPER = ROOT / ".github/r4-04-apply.py"

spec = importlib.util.spec_from_file_location("r4_04_apply", HELPER)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load R4-04 implementation helper")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

module.apply_persistence_root()
module.write_application_adapter_owners()

error_path = ROOT / "crates/application/src/composition/adapters/persistence_error.rs"
error_text = error_path.read_text(encoding="utf-8")
old_visibility = "pub(super) fn map_persistence_error(error: PersistenceError) -> PortError {"
new_visibility = "pub(in crate::composition) fn map_persistence_error(error: PersistenceError) -> PortError {"
if error_text.count(old_visibility) != 1:
    raise RuntimeError(
        f"persistence error mapper visibility anchor expected once, found {error_text.count(old_visibility)}"
    )
error_path.write_text(
    error_text.replace(old_visibility, new_visibility, 1),
    encoding="utf-8",
    newline="\n",
)

adapter_root = ROOT / "crates/application/src/composition/adapters"
new_owners = {"database.rs", "competition.rs", "rules.rs", "persistence_error.rs"}
for path in adapter_root.rglob("*.rs"):
    if path.name in new_owners:
        continue
    text = path.read_text(encoding="utf-8")
    if "ActiveDatabase" not in text and "transition_store" not in text:
        continue
    relative = path.relative_to(adapter_root)
    if len(relative.parts) == 1:
        text = text.replace(
            "use super::super::port_registry::{map_persistence_error, ",
            "use super::map_persistence_error;\nuse super::super::port_registry::{",
        )
    else:
        text = text.replace(
            "use super::super::super::port_registry::{map_persistence_error, ",
            "use super::super::map_persistence_error;\nuse super::super::super::port_registry::{",
        )
    text = text.replace("ActiveDatabase", "PersistenceStore")
    text = re.sub(
        r"let store = self\s*\n\s*\.transition_store\(\);",
        "let store = PersistenceStore::clone(self);",
        text,
    )
    text = text.replace(
        "let store = self.transition_store();",
        "let store = PersistenceStore::clone(self);",
    )
    text = re.sub(r"self\s*\n\s*\.transition_store\(\)", "self", text)
    text = text.replace("self.transition_store()", "self")
    if "transition_store" in text or "ActiveDatabase" in text:
        raise RuntimeError(f"adapter transition owner remains after normalized rewrite in {relative}")
    path.write_text(text, encoding="utf-8", newline="\n")

mod_path = adapter_root / "mod.rs"
mod_text = mod_path.read_text(encoding="utf-8")
mod_text = module.replace_once(
    mod_text,
    "mod ai_workspace;\n",
    "mod ai_workspace;\nmod competition;\nmod database;\n",
    "application adapter domain modules",
)
mod_text = module.replace_once(
    mod_text,
    "mod players;\n",
    "mod persistence_error;\nmod players;\n",
    "application adapter error module",
)
mod_text = module.replace_once(
    mod_text,
    "mod research;\n",
    "mod research;\nmod rules;\n",
    "application rules adapter module",
)
mod_text = module.replace_once(
    mod_text,
    "pub(crate) use prediction::model_run_list_item_from_port;\n",
    "pub(crate) use database::{database_health_from_snapshot, database_stats_from_statistics};\npub(super) use persistence_error::map_persistence_error;\npub(crate) use prediction::model_run_list_item_from_port;\n",
    "application adapter exports",
)
mod_path.write_text(mod_text, encoding="utf-8", newline="\n")

module.rewrite_port_registry()
module.switch_application_sessions()

service_path = ROOT / "crates/application/src/services/database/service.rs"
service = service_path.read_text(encoding="utf-8")
for old, new, label in [
    (
        "previous.close().await?;",
        "DatabaseLifecyclePort::close(&previous).await?;",
        "database previous-session close",
    ),
    (
        "active.close().await?;",
        "DatabaseLifecyclePort::close(&active).await?;",
        "database active-session close",
    ),
]:
    if service.count(old) != 1:
        raise RuntimeError(f"{label} anchor expected once, found {service.count(old)}")
    service = service.replace(old, new, 1)
service_path.write_text(service, encoding="utf-8", newline="\n")

try:
    module.write_verifier_and_package()
except RuntimeError as error:
    expected = "architecture adapter gate: expected 1 exact match, found 0"
    if str(error) != expected:
        raise

    package_path = ROOT / "package.json"
    package = package_path.read_text(encoding="utf-8")
    old_arch = '&& node scripts/verify-persistence-mapping.mjs",\n    "verify:persistence-foundation"'
    new_arch = '&& node scripts/verify-persistence-mapping.mjs && node scripts/verify-persistence-adapters.mjs",\n    "verify:persistence-foundation"'
    if package.count(old_arch) != 1:
        raise RuntimeError(f"strict package architecture anchor expected once, found {package.count(old_arch)}")
    package = package.replace(old_arch, new_arch, 1)

    old_script = '    "verify:persistence-mapping": "node scripts/verify-persistence-mapping.mjs",\n'
    new_script = old_script + '    "verify:persistence-adapters": "node scripts/verify-persistence-adapters.mjs",\n'
    if package.count(old_script) != 1:
        raise RuntimeError(f"strict package script anchor expected once, found {package.count(old_script)}")
    package = package.replace(old_script, new_script, 1)
    package_path.write_text(package, encoding="utf-8", newline="\n")

verifier_path = ROOT / "scripts/verify-persistence-adapters.mjs"
verifier = verifier_path.read_text(encoding="utf-8")
old_check = 'check(portRegistry.includes("register_adapters(options).await.map_err(map_persistence_error)"), "PortRegistry must use the unique adapter registration entry");'
new_check = 'check(/register_adapters\\(options\\)\\s*\\.await\\s*\\.map_err\\(map_persistence_error\\)/s.test(portRegistry), "PortRegistry must use the unique adapter registration entry");'
if verifier.count(old_check) != 1:
    raise RuntimeError(f"registration verifier format anchor expected once, found {verifier.count(old_check)}")
verifier = verifier.replace(old_check, new_check, 1)

insert_anchor = 'check(!portRegistry.includes("PersistenceStore::connect"), "Application composition must not bypass register_adapters");\n'
insert = insert_anchor + '''const persistenceError = read("crates/application/src/composition/adapters/persistence_error.rs");
const databaseService = read("crates/application/src/services/database/service.rs");
check(persistenceError.includes("pub(in crate::composition) fn map_persistence_error"), "Persistence error mapper must be visible only across the composition boundary");
check(databaseService.includes("DatabaseLifecyclePort::close(&previous).await?;"), "Database replacement must preserve fallible lifecycle close semantics");
check(databaseService.includes("DatabaseLifecyclePort::close(&active).await?;"), "Database disconnect must preserve fallible lifecycle close semantics");
'''
if verifier.count(insert_anchor) != 1:
    raise RuntimeError(f"adapter verifier close-semantics anchor expected once, found {verifier.count(insert_anchor)}")
verifier = verifier.replace(insert_anchor, insert, 1)
verifier_path.write_text(verifier, encoding="utf-8", newline="\n")

# R4-04 changes the concrete database session from the transitional ActiveDatabase wrapper
# to PostgresStore/PersistenceStore while keeping DatabaseService as the unique state owner.
# Migrate the R3-02 static gate to the new owner locations instead of weakening/removing it.
db_gate_path = ROOT / "scripts/verify-database-service.mjs"
db_gate = db_gate_path.read_text(encoding="utf-8")
db_gate = module.replace_once(
    db_gate,
    'const adapter = read("crates/application/src/composition/port_registry.rs");',
    'const portRegistry = read("crates/application/src/composition/port_registry.rs");\nconst databaseAdapter = read("crates/application/src/composition/adapters/database.rs");',
    "database service gate adapter owners",
)
db_gate = module.replace_once(
    db_gate,
    'check(service.includes("pub(crate) session: RwLock<Option<ActiveDatabase>>"), "活动数据库状态未归属 DatabaseService");',
    'check(service.includes("pub(crate) session: RwLock<Option<PersistenceStore>>"), "活动数据库状态未归属 DatabaseService");',
    "database service active session type",
)
db_gate = module.replace_once(
    db_gate,
    'check(!applicationService.includes("RwLock<Option<ActiveDatabase>>"), "ApplicationService 仍直接持有活动数据库槽位");',
    'check(!applicationService.includes("RwLock<Option<PersistenceStore>>"), "ApplicationService 仍直接持有活动数据库槽位");',
    "database service facade state guard",
)
db_gate = module.replace_once(
    db_gate,
    'check(adapter.includes("impl DatabaseLifecyclePort for ActiveDatabase"), "PostgreSQL adapter 未实现 DatabaseLifecyclePort");\ncheck(adapter.includes("impl DatabaseObservabilityPort for ActiveDatabase"), "PostgreSQL adapter 未实现 DatabaseObservabilityPort");\ncheck(adapter.includes("PostgresStore as PersistenceStore"), "PostgreSQL 具体适配器入口缺失");',
    'check(databaseAdapter.includes("impl DatabaseLifecyclePort for PersistenceStore"), "PostgreSQL adapter 未实现 DatabaseLifecyclePort");\ncheck(databaseAdapter.includes("impl DatabaseObservabilityPort for PersistenceStore"), "PostgreSQL adapter 未实现 DatabaseObservabilityPort");\ncheck(portRegistry.includes("PostgresStore as PersistenceStore"), "PostgreSQL 具体适配器入口缺失");\ncheck(portRegistry.includes("use football_persistence_postgres::register_adapters;"), "PostgreSQL adapter 注册入口未收敛到 register_adapters");\ncheck(!portRegistry.includes("impl DatabaseLifecyclePort") && !portRegistry.includes("impl DatabaseObservabilityPort"), "port_registry.rs 仍承载数据库 Port 实现");',
    "database service port implementation owners",
)
db_gate = module.replace_once(
    db_gate,
    'console.log("Database Service 验证通过：连接、迁移、健康、统计、清空均已进入 Service/Use Case/Port 边界，活动数据库由 DatabaseService 单一持有，Tauri 不再直接执行 PostgreSQL 清空流程，内置 P4 artifact 初始化通过 ResearchService/ResearchArtifactPort 保持可验证链路。");',
    'console.log("Database Service 验证通过：连接、迁移、健康、统计、清空均已进入 Service/Use Case/Port 边界，活动 PostgresStore 由 DatabaseService 单一持有，Lifecycle/Observability Port 实现已迁入具名 database adapter，Tauri 不直接执行 PostgreSQL 清空流程，内置 P4 artifact 初始化通过 ResearchService/ResearchArtifactPort 保持可验证链路。");',
    "database service gate success message",
)
db_gate_path.write_text(db_gate, encoding="utf-8", newline="\n")

print("R4-04 helper applied: forwarding normalized, mapper visibility/close semantics preserved, and Database Service architecture gate migrated to PersistenceStore + named adapter owners")
