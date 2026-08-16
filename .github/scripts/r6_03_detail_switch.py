from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BASE = ROOT / "crates/persistence-postgres/src/adapters/catalog/players"
PLAYER = ROOT / "crates/persistence-postgres/src/player_catalog.rs"

FILES = {
    "detail/abilities/profile/mapper.rs": '''use super::row::PlayerAbilityProfileRow;
use football_domain::PlayerAbilityProfile;

pub(super) fn map_player_ability_profile(row: PlayerAbilityProfileRow) -> PlayerAbilityProfile {
    PlayerAbilityProfile {
        player_id: row.player_id,
        abilities: row.abilities,
        average_value: row.average_value,
        average_confidence: row.average_confidence,
        dimension_count: row.dimension_count,
        latest_observed_at: row.latest_observed_at,
        next_expiry_at: row.next_expiry_at,
        updated_at: row.updated_at,
    }
}
''',
    "detail/abilities/profile/read.rs": '''use super::{mapper::map_player_ability_profile, row::PlayerAbilityProfileRow};
use crate::PersistenceResult;
use football_domain::PlayerAbilityProfile;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_ability_profile(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Option<PlayerAbilityProfile>> {
    let row = sqlx::query_as::<_, PlayerAbilityProfileRow>(
        r#"
        SELECT player_id, abilities, average_value, average_confidence,
               dimension_count, latest_observed_at, next_expiry_at, updated_at
        FROM feature.player_ability_profiles
        WHERE player_id = $1
          AND (next_expiry_at IS NULL OR next_expiry_at >= now())
        "#,
    )
    .bind(player_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(map_player_ability_profile))
}
''',
    "detail/abilities/observations/mod.rs": '''mod mapper;
mod read;
mod row;

pub(super) use read::read_ability_observations;
''',
    "detail/abilities/observations/row.rs": '''use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct PlayerAbilityObservationRow {
    pub id: Uuid,
    pub player_id: Uuid,
    pub dimension_code: String,
    pub dimension_name: String,
    pub context_type: String,
    pub context_id: Option<Uuid>,
    pub value: f64,
    pub confidence: f64,
    pub sample_size: i32,
    pub observed_at: DateTime<Utc>,
    pub effective_from: DateTime<Utc>,
    pub effective_to: Option<DateTime<Utc>>,
    pub calculation_version: String,
}
''',
    "detail/abilities/observations/mapper.rs": '''use super::row::PlayerAbilityObservationRow;
use football_domain::PlayerAbilityObservationRecord;

pub(super) fn map_player_ability_observation(
    row: PlayerAbilityObservationRow,
) -> PlayerAbilityObservationRecord {
    PlayerAbilityObservationRecord {
        id: row.id,
        player_id: row.player_id,
        dimension_code: row.dimension_code,
        dimension_name: row.dimension_name,
        context_type: row.context_type,
        context_id: row.context_id,
        value: row.value,
        confidence: row.confidence,
        sample_size: row.sample_size,
        observed_at: row.observed_at,
        effective_from: row.effective_from,
        effective_to: row.effective_to,
        calculation_version: row.calculation_version,
    }
}
''',
    "detail/abilities/observations/read.rs": '''use super::{mapper::map_player_ability_observation, row::PlayerAbilityObservationRow};
use crate::PersistenceResult;
use football_domain::PlayerAbilityObservationRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_ability_observations(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Vec<PlayerAbilityObservationRecord>> {
    let rows = sqlx::query_as::<_, PlayerAbilityObservationRow>(
        r#"
        SELECT
            observation.id, observation.player_id, observation.dimension_code,
            dimension.name AS dimension_name, observation.context_type,
            observation.context_id, observation.value, observation.confidence,
            observation.sample_size, observation.observed_at,
            observation.effective_from, observation.effective_to,
            observation.calculation_version
        FROM feature.player_ability_observations observation
        JOIN feature.player_ability_dimensions dimension
          ON dimension.code = observation.dimension_code
        WHERE observation.player_id = $1
        ORDER BY observation.observed_at DESC, observation.id DESC
        LIMIT 250
        "#,
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(map_player_ability_observation).collect())
}
''',
    "detail/external_ids/mod.rs": '''mod mapper;
mod read;
mod row;

pub(super) use read::read_external_ids;
''',
    "detail/external_ids/row.rs": '''use serde_json::Value;
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, FromRow)]
pub(super) struct ExternalEntityIdRow {
    pub id: Uuid,
    pub provider_id: Uuid,
    pub provider_name: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub external_id: String,
    pub metadata: Value,
}
''',
    "detail/external_ids/mapper.rs": '''use super::row::ExternalEntityIdRow;
use football_domain::ExternalEntityIdRecord;

pub(super) fn map_external_entity_id(row: ExternalEntityIdRow) -> ExternalEntityIdRecord {
    ExternalEntityIdRecord {
        id: row.id,
        provider_id: row.provider_id,
        provider_name: row.provider_name,
        entity_type: row.entity_type,
        entity_id: row.entity_id,
        external_id: row.external_id,
        metadata: row.metadata,
    }
}
''',
    "detail/external_ids/read.rs": '''use super::{mapper::map_external_entity_id, row::ExternalEntityIdRow};
use crate::PersistenceResult;
use football_domain::ExternalEntityIdRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_external_ids(
    pool: &PgPool,
    player_id: Uuid,
) -> PersistenceResult<Vec<ExternalEntityIdRecord>> {
    let rows = sqlx::query_as::<_, ExternalEntityIdRow>(
        r#"
        SELECT external.id, external.provider_id, provider.name AS provider_name,
               external.entity_type, external.entity_id, external.external_id,
               external.metadata
        FROM football.external_entity_ids external
        JOIN catalog.data_providers provider ON provider.id = external.provider_id
        WHERE external.entity_type = 'player' AND external.entity_id = $1
        ORDER BY provider.name, external.external_id
        "#,
    )
    .bind(player_id)
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(map_external_entity_id).collect())
}
''',
    "detail/read_player.rs": '''use super::{
    abilities::{read_ability_observations, read_ability_profile},
    availability::read_availability,
    external_ids::read_external_ids,
    names::read_names,
    positions::read_positions,
    team_periods::read_team_periods,
};
use crate::{
    adapters::catalog::players::record::{map_player_record, PlayerRecordRow},
    PersistenceResult, PostgresStore,
};
use chrono::Utc;
use football_domain::PlayerDetail;
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_player(&self, player_id: Uuid) -> PersistenceResult<PlayerDetail> {
        let row = sqlx::query_as::<_, PlayerRecordRow>(
            r#"
            SELECT
                id, canonical_name, normalized_name, date_of_birth,
                nationality_code, preferred_foot, height_cm, status, created_at
            FROM football.players
            WHERE id = $1
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.pool)
        .await?;
        let player = map_player_record(row)?;
        let names = read_names(&self.pool, player_id).await?;
        let positions = read_positions(&self.pool, player_id).await?;
        let team_periods = read_team_periods(&self.pool, player_id).await?;
        let availability = read_availability(self, player_id).await?;
        let ability_profile = read_ability_profile(&self.pool, player_id).await?;
        let ability_observations = read_ability_observations(&self.pool, player_id).await?;
        let dynamic_tags = self.list_player_dynamic_tags(player_id, Utc::now()).await?;
        let external_ids = read_external_ids(&self.pool, player_id).await?;
        Ok(PlayerDetail {
            player,
            names,
            positions,
            team_periods,
            availability,
            ability_profile,
            ability_observations,
            dynamic_tags,
            external_ids,
        })
    }
}
''',
}

for relative, content in FILES.items():
    target = BASE / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8")

# Shared PlayerRecord projection is owned once above Directory and Detail.
players_mod = BASE / "mod.rs"
text = players_mod.read_text(encoding="utf-8")
expected = "mod directory;\npub(crate) mod normalization;\npub(crate) mod value_mapping;\n"
if text != expected:
    raise SystemExit(f"unexpected players mod content: {text!r}")
players_mod.write_text(
    "mod detail;\nmod directory;\npub(crate) mod normalization;\nmod record;\npub(crate) mod value_mapping;\n",
    encoding="utf-8",
)

directory_mod = BASE / "directory/mod.rs"
text = directory_mod.read_text(encoding="utf-8")
text = text.replace("mod record_mapper;\n", "").replace("mod record_row;\n", "")
directory_mod.write_text(text, encoding="utf-8")

for name in ["create_player.rs", "update_player.rs"]:
    target = BASE / "directory" / name
    text = target.read_text(encoding="utf-8")
    text = text.replace(
        "use super::{\n    input_policy::validate_player_draft, record_mapper::map_player_record,\n    record_row::PlayerRecordRow,\n};\n",
        "use super::input_policy::validate_player_draft;\n",
    )
    text = text.replace(
        "    adapters::catalog::players::normalization::normalize_name, PersistenceResult, PostgresStore,\n",
        "    adapters::catalog::players::{\n        normalization::normalize_name,\n        record::{map_player_record, PlayerRecordRow},\n    },\n    PersistenceResult, PostgresStore,\n",
    )
    text = text.replace(
        "    adapters::catalog::players::normalization::normalize_name, PersistenceError, PersistenceResult,\n    PostgresStore,\n",
        "    adapters::catalog::players::{\n        normalization::normalize_name,\n        record::{map_player_record, PlayerRecordRow},\n    },\n    PersistenceError, PersistenceResult, PostgresStore,\n",
    )
    target.write_text(text, encoding="utf-8")

(BASE / "directory/record_mapper.rs").unlink()
(BASE / "directory/record_row.rs").unlink()


def remove_between(text: str, start: str, end: str) -> str:
    start_index = text.find(start)
    if start_index < 0:
        raise SystemExit(f"missing start marker: {start!r}")
    end_index = text.find(end, start_index)
    if end_index < 0:
        raise SystemExit(f"missing end marker: {end!r}")
    return text[:start_index] + text[end_index:]

text = PLAYER.read_text(encoding="utf-8")
text = remove_between(
    text,
    "    pub async fn read_player(&self, player_id: Uuid) -> PersistenceResult<PlayerDetail> {",
    "    pub async fn add_player_name(",
)
text = remove_between(
    text,
    "fn player_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PlayerRecord> {",
    "fn player_name_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PlayerNameRecord> {",
)
text = remove_between(
    text,
    "fn player_ability_profile_from_row(\n    row: &sqlx::postgres::PgRow,\n) -> PersistenceResult<PlayerAbilityProfile> {",
    "fn external_entity_id_from_row(",
)
text = text.replace(
    "    adapters::catalog::players::{\n        normalization::normalize_name,\n        value_mapping::{availability_status, player_status, preferred_foot},\n    },\n",
    "    adapters::catalog::players::{\n        normalization::normalize_name,\n        value_mapping::availability_status,\n    },\n",
    1,
)
text = text.replace(
    "    MatchRecord, MatchStatus, PlayerAbilityObservationDraft, PlayerAbilityObservationRecord,\n    PlayerAbilityProfile, PlayerAvailabilityDraft, PlayerAvailabilityRecord,\n    PlayerCatalogReferenceData, PlayerDetail, PlayerNameDraft, PlayerNameRecord,\n    PlayerPositionDraft, PlayerPositionRecord, PlayerRecord, PlayerTeamPeriodDraft,\n",
    "    MatchRecord, MatchStatus, PlayerAbilityObservationDraft, PlayerAbilityObservationRecord,\n    PlayerAvailabilityDraft, PlayerAvailabilityRecord, PlayerCatalogReferenceData, PlayerNameDraft,\n    PlayerNameRecord, PlayerPositionDraft, PlayerPositionRecord, PlayerTeamPeriodDraft,\n",
    1,
)
text = text.replace(
    "mod tests {\n    use super::*;\n    use football_domain::{PlayerStatus, PreferredFoot};\n",
    "mod tests {\n    use super::*;\n    use crate::adapters::catalog::players::value_mapping::{player_status, preferred_foot};\n    use football_domain::{PlayerStatus, PreferredFoot};\n",
    1,
)
PLAYER.write_text(text, encoding="utf-8")
