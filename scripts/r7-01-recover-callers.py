from __future__ import annotations

from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[1]
SRC = ROOT / "crates/persistence-postgres/src"
BRANCH = "rewrite/r7-match-lineup-workbook-persistence"


def fail(message: str) -> None:
    raise SystemExit(f"R7-01 recovery aborted: {message}")


def git(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(["git", *args], cwd=ROOT, text=True, capture_output=True)


branch = subprocess.check_output(["git", "branch", "--show-current"], cwd=ROOT, text=True).strip()
if branch != BRANCH:
    fail(f"expected branch {BRANCH}, got {branch}")

# These three files were not changed by the first owner-switch attempt. Protect any user edits.
for relative in [
    "crates/persistence-postgres/src/match_prediction.rs",
    "crates/persistence-postgres/src/lineup_chain.rs",
    "scripts/r7-01-apply.py",
]:
    result = git("diff", "--quiet", "--", relative)
    if result.returncode != 0:
        fail(f"protected file already has local changes: {relative}")

catalog_mod_path = SRC / "adapters/matches/catalog/mod.rs"
mapping_path = SRC / "adapters/matches/catalog/mapping.rs"
prediction_path = SRC / "match_prediction.rs"
lineup_chain_path = SRC / "lineup_chain.rs"
verify_path = ROOT / "scripts/verify-r7-match-catalog.mjs"
apply_path = ROOT / "scripts/r7-01-apply.py"

for path in [catalog_mod_path, mapping_path, prediction_path, lineup_chain_path, verify_path, apply_path]:
    if not path.exists():
        fail(f"required file missing: {path.relative_to(ROOT)}")

catalog_mod = catalog_mod_path.read_text(encoding="utf-8")
mapping = mapping_path.read_text(encoding="utf-8")
prediction = prediction_path.read_text(encoding="utf-8")
lineup_chain = lineup_chain_path.read_text(encoding="utf-8")
verify = verify_path.read_text(encoding="utf-8")
apply = apply_path.read_text(encoding="utf-8")

old_prediction = "super::match_exchange::match_record_from_row(&row)?"
new_prediction = "crate::adapters::matches::catalog::match_record_from_row(&row)?"
old_chain = "self.read_match_exchange(match_id).await?"
new_chain = "self.read_match(match_id).await?"

if "pub(crate) use mapping::match_record_from_row;" not in catalog_mod:
    if "mod mapping;\n" not in catalog_mod:
        fail("catalog/mod.rs does not contain the expected mapping module")
    catalog_mod = catalog_mod.replace(
        "mod mapping;\n",
        "mod mapping;\npub(crate) use mapping::match_record_from_row;\n",
        1,
    )

if "pub(super) fn match_record_from_row" not in mapping:
    fail("mapping.rs no longer has the expected pub(super) mapper")
mapping = mapping.replace(
    "pub(super) fn match_record_from_row",
    "pub(crate) fn match_record_from_row",
    1,
)

if old_prediction not in prediction:
    fail("match_prediction.rs no longer has the expected legacy mapper call")
prediction = prediction.replace(old_prediction, new_prediction, 1)

if old_chain not in lineup_chain:
    fail("lineup_chain.rs no longer has the expected read_match_exchange call")
lineup_chain = lineup_chain.replace(old_chain, new_chain, 1)

# Extend the ownership verifier so these two caller regressions cannot recur.
if 'const matchPrediction = read("crates/persistence-postgres/src/match_prediction.rs");' not in verify:
    marker = 'const exchange = read("crates/persistence-postgres/src/match_exchange.rs");\n'
    if marker not in verify:
        fail("ownership verifier caller marker missing")
    verify = verify.replace(
        marker,
        marker
        + 'const matchPrediction = read("crates/persistence-postgres/src/match_prediction.rs");\n'
        + 'const lineupChain = read("crates/persistence-postgres/src/lineup_chain.rs");\n',
        1,
    )

if "match_prediction.rs 仍依赖旧 Match Exchange mapper" not in verify:
    marker = 'requireTrue(exchange.includes("self.read_match("), "match_exchange.rs 未切换到 Match Catalog read owner");\n'
    if marker not in verify:
        fail("ownership verifier assertion marker missing")
    verify = verify.replace(
        marker,
        marker
        + 'requireTrue(!matchPrediction.includes("match_exchange::match_record_from_row"), "match_prediction.rs 仍依赖旧 Match Exchange mapper");\n'
        + 'requireTrue(matchPrediction.includes("adapters::matches::catalog::match_record_from_row"), "match_prediction.rs 未切换到 Match Catalog mapper");\n'
        + 'requireTrue(!lineupChain.includes("read_match_exchange("), "lineup_chain.rs 仍依赖旧 read_match_exchange");\n'
        + 'requireTrue(lineupChain.includes("self.read_match("), "lineup_chain.rs 未切换到 Match Catalog read owner");\n',
        1,
    )

# Repair the original one-shot migration so a fresh replay performs the complete caller switch.
apply_replacements = [
    (
        'exchange_path = SRC / "match_exchange.rs"\nadapters_path = SRC / "adapters/mod.rs"\n',
        'exchange_path = SRC / "match_exchange.rs"\nprediction_path = SRC / "match_prediction.rs"\nlineup_chain_path = SRC / "lineup_chain.rs"\nadapters_path = SRC / "adapters/mod.rs"\n',
    ),
    (
        'if not player_path.exists() or not exchange_path.exists() or not adapters_path.exists():\n',
        'if (\n    not player_path.exists()\n    or not exchange_path.exists()\n    or not prediction_path.exists()\n    or not lineup_chain_path.exists()\n    or not adapters_path.exists()\n):\n',
    ),
    (
        'exchange = exchange_path.read_text(encoding="utf-8")\nadapters = adapters_path.read_text(encoding="utf-8")\n',
        'exchange = exchange_path.read_text(encoding="utf-8")\nprediction = prediction_path.read_text(encoding="utf-8")\nlineup_chain = lineup_chain_path.read_text(encoding="utf-8")\nadapters = adapters_path.read_text(encoding="utf-8")\n',
    ),
    (
        '"mod create;\\nmod delete;\\nmod list;\\nmod mapping;\\nmod read;\\nmod scope;\\n",\n',
        '"mod create;\\nmod delete;\\nmod list;\\nmod mapping;\\npub(crate) use mapping::match_record_from_row;\\nmod read;\\nmod scope;\\n",\n',
    ),
    (
        '"fn match_record_from_row", "pub(super) fn match_record_from_row", 1\n',
        '"fn match_record_from_row", "pub(crate) fn match_record_from_row", 1\n',
    ),
    (
        'exchange_path.write_text(exchange, encoding="utf-8")\n\n# Register the new R7 adapter namespace.\n',
        'exchange_path.write_text(exchange, encoding="utf-8")\n\n# Switch remaining Match consumers to the new catalog owner.\nold_prediction = "super::match_exchange::match_record_from_row(&row)?"\nif old_prediction not in prediction:\n    fail("match_prediction.rs legacy Match mapper call missing")\nprediction = prediction.replace(\n    old_prediction,\n    "crate::adapters::matches::catalog::match_record_from_row(&row)?",\n    1,\n)\nold_chain = "self.read_match_exchange(match_id).await?"\nif old_chain not in lineup_chain:\n    fail("lineup_chain.rs legacy read_match_exchange call missing")\nlineup_chain = lineup_chain.replace(old_chain, "self.read_match(match_id).await?", 1)\nprediction_path.write_text(prediction, encoding="utf-8")\nlineup_chain_path.write_text(lineup_chain, encoding="utf-8")\n\n# Register the new R7 adapter namespace.\n',
    ),
    (
        'const exchange = read("crates/persistence-postgres/src/match_exchange.rs");\\n',
        'const exchange = read("crates/persistence-postgres/src/match_exchange.rs");\\nconst matchPrediction = read("crates/persistence-postgres/src/match_prediction.rs");\\nconst lineupChain = read("crates/persistence-postgres/src/lineup_chain.rs");\\n',
    ),
    (
        'requireTrue(exchange.includes("self.read_match("), "match_exchange.rs 未切换到 Match Catalog read owner");\\n',
        'requireTrue(exchange.includes("self.read_match("), "match_exchange.rs 未切换到 Match Catalog read owner");\\nrequireTrue(!matchPrediction.includes("match_exchange::match_record_from_row"), "match_prediction.rs 仍依赖旧 Match Exchange mapper");\\nrequireTrue(matchPrediction.includes("adapters::matches::catalog::match_record_from_row"), "match_prediction.rs 未切换到 Match Catalog mapper");\\nrequireTrue(!lineupChain.includes("read_match_exchange("), "lineup_chain.rs 仍依赖旧 read_match_exchange");\\nrequireTrue(lineupChain.includes("self.read_match("), "lineup_chain.rs 未切换到 Match Catalog read owner");\\n',
    ),
]

for old, new in apply_replacements:
    if old not in apply:
        fail(f"original migration no longer matches expected text: {old[:72]!r}")
    apply = apply.replace(old, new, 1)

# Compile the repaired migration before touching disk with it.
compile(apply, str(apply_path), "exec")

catalog_mod_path.write_text(catalog_mod, encoding="utf-8")
mapping_path.write_text(mapping, encoding="utf-8")
prediction_path.write_text(prediction, encoding="utf-8")
lineup_chain_path.write_text(lineup_chain, encoding="utf-8")
verify_path.write_text(verify, encoding="utf-8")
apply_path.write_text(apply, encoding="utf-8")

print("R7-01 caller recovery applied successfully.")
print("Recovered: Match mapper crate visibility, match_prediction caller, lineup_chain read caller, verifier coverage, and replayable migration.")
