use super::RoundRow;
use football_domain::RoundRecord;

pub(super) fn map_round_row(row: RoundRow) -> RoundRecord {
    RoundRecord {
        id: row.id,
        stage_id: row.stage_id,
        stage_name: row.stage_name,
        code: row.code,
        name: row.name,
        sequence_no: row.sequence_no,
        starts_at: row.starts_at,
        ends_at: row.ends_at,
    }
}
