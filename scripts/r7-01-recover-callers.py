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


branch = subprocess.check_output(
    ["git", "branch", "--show-current"], cwd=ROOT, text=True
).strip()
if branch != BRANCH:
    fail(f"expected branch {BRANCH}, got {branch}")

# These callers were not modified by the owner-switch script. Any local change here
# is treated as protected user work and must not be overwritten.
for relative in [
    "crates/persistence-postgres/src/match_prediction.rs",
    "crates/persistence-postgres/src/lineup_chain.rs",
]:
    result = git("diff", "--quiet", "--", relative)
    if result.returncode != 0:
        fail(f"protected file already has local changes: {relative}")

catalog_mod_path = SRC / "adapters/matches/catalog/mod.rs"
mapping_path = SRC / "adapters/matches/catalog/mapping.rs"
prediction_path = SRC / "match_prediction.rs"
lineup_chain_path = SRC / "lineup_chain.rs"
verify_path = ROOT / "scripts/verify-r7-match-catalog.mjs"

for path in [catalog_mod_path, mapping_path, prediction_path, lineup_chain_path, verify_path]:
    if not path.exists():
        fail(f"required file missing: {path.relative_to(ROOT)}")

catalog_mod = catalog_mod_path.read_text(encoding="utf-8")
mapping = mapping_path.read_text(encoding="utf-8")
prediction = prediction_path.read_text(encoding="utf-8")
lineup_chain = lineup_chain_path.read_text(encoding="utf-8")
verify = verify_path.read_text(encoding="utf-8")

# 1. Expose the canonical MatchRecord mapper to callers inside this crate only.
if "pub(crate) use mapping::match_record_from_row;" not in catalog_mod:
    marker = "mod mapping;\n"
    if marker not in catalog_mod:
        fail("catalog/mod.rs does not contain the expected mapping module")
    catalog_mod = catalog_mod.replace(
        marker,
        marker + "pub(crate) use mapping::match_record_from_row;\n",
        1,
    )

if "pub(super) fn match_record_from_row" in mapping:
    mapping = mapping.replace(
        "pub(super) fn match_record_from_row",
        "pub(crate) fn match_record_from_row",
        1,
    )
elif "pub(crate) fn match_record_from_row" not in mapping:
    fail("mapping.rs does not expose the expected MatchRecord mapper")

# 2. Switch the two missed legacy callers to the new Match Catalog owner.
old_prediction = "super::match_exchange::match_record_from_row(&row)?"
new_prediction = "crate::adapters::matches::catalog::match_record_from_row(&row)?"
if old_prediction in prediction:
    prediction = prediction.replace(old_prediction, new_prediction, 1)
elif new_prediction not in prediction:
    fail("match_prediction.rs has neither the expected legacy nor new mapper call")

old_chain = "self.read_match_exchange(match_id).await?"
new_chain = "self.read_match(match_id).await?"
if old_chain in lineup_chain:
    lineup_chain = lineup_chain.replace(old_chain, new_chain, 1)
elif new_chain not in lineup_chain:
    fail("lineup_chain.rs has neither the expected legacy nor new read call")

# 3. Strengthen the static ownership gate so these caller regressions cannot recur.
if 'const matchPrediction = read("crates/persistence-postgres/src/match_prediction.rs");' not in verify:
    marker = 'const exchange = read("crates/persistence-postgres/src/match_exchange.rs");\n'
    if marker not in verify:
        fail("ownership verifier caller declaration marker missing")
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

# All validation above happens before any write, so a failed recovery cannot leave
# a partially modified caller chain.
catalog_mod_path.write_text(catalog_mod, encoding="utf-8")
mapping_path.write_text(mapping, encoding="utf-8")
prediction_path.write_text(prediction, encoding="utf-8")
lineup_chain_path.write_text(lineup_chain, encoding="utf-8")
verify_path.write_text(verify, encoding="utf-8")

print("R7-01 caller recovery applied successfully.")
print("Recovered: Match mapper crate visibility, match_prediction caller, lineup_chain read caller, and verifier coverage.")
