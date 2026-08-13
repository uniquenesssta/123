from __future__ import annotations

import hashlib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
STORE = ROOT / "crates/persistence-postgres/src/store/postgres_store.rs"
CONTRACT = ROOT / "architecture/database-baseline.json"
EXPECTED_OLD = "939445bec1e0e35ab28c25173d87e2ccf4dc79de"

raw = STORE.read_bytes().decode("utf-8").replace("\r\n", "\n").replace("\r", "\n").encode("utf-8")
header = f"blob {len(raw)}\0".encode("utf-8")
new_sha = hashlib.sha1(header + raw).hexdigest()
if new_sha == EXPECTED_OLD:
    raise RuntimeError("R4-04 expected postgres_store.rs fingerprint to change, but it stayed at the R4-03 baseline")

text = CONTRACT.read_text(encoding="utf-8")
old_token = (
    '{"path":"crates/persistence-postgres/src/store/postgres_store.rs",'
    f'"git_blob_sha1":"{EXPECTED_OLD}"}}'
)
new_token = (
    '{"path":"crates/persistence-postgres/src/store/postgres_store.rs",'
    f'"git_blob_sha1":"{new_sha}"}}'
)
if text.count(old_token) != 1:
    raise RuntimeError(f"database baseline postgres_store entry expected once, found {text.count(old_token)}")
text = text.replace(old_token, new_token, 1)
CONTRACT.write_text(text, encoding="utf-8", newline="\n")
print(f"R4-04 database runtime fingerprint synchronized: postgres_store.rs {EXPECTED_OLD} -> {new_sha}")
