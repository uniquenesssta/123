use super::StageRow;
use crate::{parse_competition_kind, PersistenceResult};
use football_domain::StageRecord;

pub(super) fn map_stage_row(row: StageRow) -> PersistenceResult<StageRecord> {
    Ok(StageRecord {
        id: row.id,
        season_id: row.season_id,
        season_name: row.season_name,
        competition_id: row.competition_id,
        competition_name: row.competition_name,
        code: row.code,
        name: row.name,
        stage_kind: parse_competition_kind(&row.stage_kind)?,
        sequence_no: row.sequence_no,
    })
}
