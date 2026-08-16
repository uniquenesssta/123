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
text = remove_between(text, "fn normalize_name(value: &str) -> String {", "fn match_status(value: &str) -> PersistenceResult<MatchStatus> {")

# Directory-specific dynamic row mappers are replaced by typed Row + pure Mapper modules.
text = remove_between(
    text,
    "fn player_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PlayerRecord> {",
    "fn player_name_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PlayerNameRecord> {",
)

old_import = "use crate::{\n    name_search::{push_name_search, NameSearch, NameSearchColumns},\n    role_resolution::{"
new_import = "use crate::{\n    adapters::catalog::players::{\n        normalization::normalize_name,\n        value_mapping::{availability_status, player_status, preferred_foot},\n    },\n    role_resolution::{"
if text.count(old_import) != 1:
    raise SystemExit("unexpected crate import layout")
text = text.replace(old_import, new_import, 1)

text = text.replace(
    "    PlayerCatalogReferenceData, PlayerDetail, PlayerDraft, PlayerListItem, PlayerListPage,\n    PlayerListQuery, PlayerNameDraft, PlayerNameRecord, PlayerPositionDraft, PlayerPositionRecord,\n    PlayerRecord, PlayerStatus, PlayerTeamPeriodDraft, PlayerTeamPeriodRecord, PositionReference,\n",
    "    PlayerCatalogReferenceData, PlayerDetail, PlayerNameDraft, PlayerNameRecord,\n    PlayerPositionDraft, PlayerPositionRecord, PlayerStatus, PlayerTeamPeriodDraft,\n    PlayerTeamPeriodRecord, PositionReference,\n",
    1,
)
text = text.replace("use sqlx::{Postgres, QueryBuilder, Row, Transaction};", "use sqlx::{Postgres, Row, Transaction};", 1)
PLAYER.write_text(text, encoding="utf-8")

catalog = CATALOG_MOD.read_text(encoding="utf-8")
if catalog != "pub(crate) mod teams;\n":
    raise SystemExit(f"unexpected catalog mod content: {catalog!r}")
CATALOG_MOD.write_text("pub(crate) mod players;\npub(crate) mod teams;\n", encoding="utf-8")
