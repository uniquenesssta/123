from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
HELPER = ROOT / ".github/r4-04-apply.py"

spec = importlib.util.spec_from_file_location("r4_04_apply", HELPER)
if spec is None or spec.loader is None:
    raise RuntimeError("unable to load R4-04 implementation helper")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

try:
    module.apply()
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
else:
    raise RuntimeError("R4-04 helper unexpectedly bypassed the known strict package-anchor recovery; remove wrapper recovery before publication")

print("R4-04 helper applied with the single known package-anchor recovery and no other suppressed error")
