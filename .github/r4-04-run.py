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
verifier_path.write_text(verifier, encoding="utf-8", newline="\n")

print("R4-04 helper applied: forwarding normalized, package gate updated, and registration verifier made rustfmt-format agnostic")
