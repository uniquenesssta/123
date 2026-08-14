use super::CompetitionRow;
use crate::{parse_competition_kind, PersistenceResult};
use football_domain::CompetitionRecord;

pub(in crate::adapters::competition) fn map_competition_row(
    row: CompetitionRow,
) -> PersistenceResult<CompetitionRecord> {
    Ok(CompetitionRecord {
        id: row.id,
        code: row.code,
        name: row.name,
        country_code: row.country_code,
        timezone: row.timezone,
        competition_kind: parse_competition_kind(&row.competition_kind)?,
        is_active: row.is_active,
        metadata: row.metadata,
        created_at: row.created_at,
    })
}
