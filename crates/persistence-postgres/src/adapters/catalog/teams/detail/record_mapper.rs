use super::TeamRecordRow;
use crate::PersistenceResult;
use football_domain::TeamRecord;

pub(in crate::adapters::catalog::teams) fn map_team_record(
    row: TeamRecordRow,
) -> PersistenceResult<TeamRecord> {
    Ok(TeamRecord {
        id: row.id,
        canonical_name: row.canonical_name,
        normalized_name: row.normalized_name,
        country_code: row.country_code,
        is_active: row.is_active,
        created_at: row.created_at,
    })
}
