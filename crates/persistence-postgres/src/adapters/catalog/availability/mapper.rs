use super::row::PlayerAvailabilityRow;
use crate::{adapters::catalog::players::value_mapping::availability_status, PersistenceResult};
use football_domain::PlayerAvailabilityRecord;

pub(in crate::adapters::catalog) fn map_player_availability(
    row: PlayerAvailabilityRow,
) -> PersistenceResult<PlayerAvailabilityRecord> {
    Ok(PlayerAvailabilityRecord {
        id: row.id,
        player_id: row.player_id,
        team_id: row.team_id,
        team_name: row.team_name,
        competition_id: row.competition_id,
        status: availability_status(&row.status)?,
        reason: row.reason,
        confidence: row.confidence,
        valid_from: row.valid_from,
        valid_to: row.valid_to,
        created_at: row.created_at,
    })
}
