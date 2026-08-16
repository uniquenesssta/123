from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
PLAYER = ROOT / "crates/persistence-postgres/src/player_catalog.rs"
CATALOG_MOD = ROOT / "crates/persistence-postgres/src/adapters/catalog/mod.rs"


def remove_between(text: str, start: str, end: str) -> str:
    start_index = text.find(start)
    if start_index < 0:
        raise SystemExit(f"missing start marker: {start!r}")
    end_index = text.find(end, start_index)
    if end_index < 0:
        raise SystemExit(f"missing end marker: {end!r}")
    return text[:start_index] + text[end_index:]


text = PLAYER.read_text(encoding="utf-8")

# Move only the R6-03 Directory operations; deletion and later-node writes remain here.
text = remove_between(
    text,
    "    pub async fn create_player(&self, draft: &PlayerDraft) -> PersistenceResult<PlayerRecord> {",
    "    pub async fn delete_player(&self, player_id: Uuid) -> PersistenceResult<()> {",
)
text = remove_between(
    text,
    "    pub async fn list_players(&self, query: &PlayerListQuery) -> PersistenceResult<PlayerListPage> {",
    "    pub async fn read_player(&self, player_id: Uuid) -> PersistenceResult<PlayerDetail> {",
)

# Shared persisted-value/name policies get one owner under adapters/catalog/players.
text = remove_between(
    text,
    "fn normalize_name(value: &str) -> String {",
    "fn match_status(value: &str) -> PersistenceResult<MatchStatus> {",
)

# Only the list mapper moves in the Directory substage. PlayerRecord mapping must remain until
# read_player is moved by the Detail substage.
text = remove_between(
    text,
    "fn player_list_item_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PlayerListItem> {",
    "fn player_name_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PlayerNameRecord> {",
)

old_import = "use crate::{\n    name_search::{push_name_search, NameSearch, NameSearchColumns},\n    role_resolution::{"
new_import = "use crate::{\n    adapters::catalog::players::{\n        normalization::normalize_name,\n        value_mapping::{availability_status, player_status, preferred_foot},\n    },\n    role_resolution::{"
if text.count(old_import) != 1:
    raise SystemExit("unexpected crate import layout")
text = text.replace(old_import, new_import, 1)

old_domain = (
    "    PlayerCatalogReferenceData, PlayerDetail, PlayerDraft, PlayerListItem, PlayerListPage,\n"
    "    PlayerListQuery, PlayerNameDraft, PlayerNameRecord, PlayerPositionDraft, PlayerPositionRecord,\n"
    "    PlayerRecord, PlayerStatus, PlayerTeamPeriodDraft, PlayerTeamPeriodRecord, PositionReference,\n"
    "    PreferredFoot, SeasonTeamMembershipOption,\n"
)
new_domain = (
    "    PlayerCatalogReferenceData, PlayerDetail, PlayerNameDraft, PlayerNameRecord,\n"
    "    PlayerPositionDraft, PlayerPositionRecord, PlayerRecord, PlayerTeamPeriodDraft,\n"
    "    PlayerTeamPeriodRecord, PositionReference, SeasonTeamMembershipOption,\n"
)
if text.count(old_domain) != 1:
    raise SystemExit("unexpected football_domain import layout")
text = text.replace(old_domain, new_domain, 1)

text = text.replace(
    "use sqlx::{Postgres, QueryBuilder, Row, Transaction};",
    "use sqlx::{Postgres, Row, Transaction};",
    1,
)

test_import = "mod tests {\n    use super::*;\n"
if text.count(test_import) != 1:
    raise SystemExit("unexpected player_catalog test module layout")
text = text.replace(
    test_import,
    "mod tests {\n    use super::*;\n    use football_domain::{PlayerStatus, PreferredFoot};\n",
    1,
)

PLAYER.write_text(text, encoding="utf-8")

catalog = CATALOG_MOD.read_text(encoding="utf-8")
if catalog != "pub(crate) mod teams;\n":
    raise SystemExit(f"unexpected catalog mod content: {catalog!r}")
CATALOG_MOD.write_text("pub(crate) mod players;\npub(crate) mod teams;\n", encoding="utf-8")
