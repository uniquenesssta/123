use super::reference_row::PositionReferenceRow;
use football_domain::PositionReference;

pub(super) fn map_position_reference(row: PositionReferenceRow) -> PositionReference {
    PositionReference {
        code: row.code,
        name: row.name,
        position_group: row.position_group,
        sort_order: row.sort_order,
    }
}
