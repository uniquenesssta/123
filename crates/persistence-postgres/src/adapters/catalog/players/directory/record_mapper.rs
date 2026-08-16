use super::record_row::PlayerRecordRow;
use crate::{
    adapters::catalog::players::value_mapping::{player_status, preferred_foot},
    PersistenceResult,
};
use football_domain::PlayerRecord;

pub(super) fn map_player_record(row: PlayerRecordRow) -> PersistenceResult<PlayerRecord> {
    Ok(PlayerRecord {
        id: row.id,
        canonical_name: row.canonical_name,
        normalized_name: row.normalized_name,
        date_of_birth: row.date_of_birth,
        nationality_code: row.nationality_code,
        preferred_foot: preferred_foot(&row.preferred_foot)?,
        height_cm: row.height_cm,
        status: player_status(&row.status)?,
        created_at: row.created_at,
    })
}
