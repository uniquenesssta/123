use crate::{PersistenceError, PersistenceResult};
use football_domain::{SpreadsheetImportRow, SpreadsheetParsedWorkbook};
use uuid::Uuid;

// UUID 定位已暂存的行；工作表/物理行/实体类型供账本插入。
// subrecord_key 仍由冻结的数据库生成列计算，不在 Rust 重建或去重。
pub(crate) struct ImportRowLocation<'a> {
    pub(crate) row_id: Uuid,
    pub(crate) batch_id: Uuid,
    pub(crate) sheet_name: &'a str,
    pub(crate) row_number: i32,
    pub(crate) entity_type: &'static str,
}

impl<'a> ImportRowLocation<'a> {
    pub(crate) fn from_row(
        batch_id: Uuid,
        row: &'a SpreadsheetImportRow,
    ) -> PersistenceResult<Self> {
        Ok(Self {
            row_id: row.id,
            batch_id,
            sheet_name: &row.sheet_name,
            row_number: persisted_worksheet_row_number(row.row_number)?,
            entity_type: row.entity_type.as_str(),
        })
    }
}

fn persisted_worksheet_row_number(row_number: u32) -> PersistenceResult<i32> {
    i32::try_from(row_number)
        .ok()
        .filter(|value| *value >= 2)
        .ok_or_else(|| {
            PersistenceError::InvalidState(format!(
                "导入工作表物理行号必须在 2..=2147483647 范围内：{row_number}"
            ))
        })
}

// 在同源批次查找/取消之前拒绝不能按现有 integer/CHECK 契约落库的行号。
// 相同物理行的多个实体和子记录交由原数据库唯一约束处理。
pub(crate) fn validate_workbook_row_locations(
    parsed: &SpreadsheetParsedWorkbook,
) -> PersistenceResult<()> {
    for row in &parsed.rows {
        persisted_worksheet_row_number(row.row_number)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::{
        SpreadsheetAction, SpreadsheetEntityType, SpreadsheetRawRow, SpreadsheetRowStatus,
        PLAYER_MONTHLY_FORMAT,
    };
    use serde_json::json;

    #[test]
    fn worksheet_row_number_rejects_headers_and_integer_wraparound() {
        for value in [0, 1, i32::MAX as u32 + 1, u32::MAX] {
            assert!(persisted_worksheet_row_number(value).is_err());
        }
        for value in [2, 4, 1_048_576, i32::MAX as u32] {
            assert_eq!(persisted_worksheet_row_number(value).unwrap(), value as i32);
        }
    }

    #[test]
    fn row_uuid_keeps_two_subrecords_on_one_physical_row_distinct() {
        let batch_id = Uuid::from_u128(1);
        let row = SpreadsheetImportRow {
            id: Uuid::from_u128(2),
            sheet_name: "球员与评分".into(),
            row_number: 4,
            entity_type: SpreadsheetEntityType::PlayerAbility,
            action: SpreadsheetAction::Skip,
            status: SpreadsheetRowStatus::Skip,
            message: None,
            payload: json!({"dimension_code":"attack"}),
            matched_entity_id: None,
            conflict_candidates: vec![],
        };
        let mut other = row.clone();
        other.id = Uuid::from_u128(3);
        other.payload = json!({"dimension_code":"defence"});
        let first = ImportRowLocation::from_row(batch_id, &row).unwrap();
        let second = ImportRowLocation::from_row(batch_id, &other).unwrap();
        assert_ne!(first.row_id, second.row_id);
        assert_eq!(first.batch_id, second.batch_id);
        assert_eq!(first.sheet_name, second.sheet_name);
        assert_eq!(first.row_number, 4);
        assert_eq!(first.row_number, second.row_number);
        assert_eq!(first.entity_type, second.entity_type);
        assert_eq!(row.payload["dimension_code"], "attack");
        assert_eq!(other.payload["dimension_code"], "defence");
    }

    #[test]
    fn preflight_preserves_same_row_entities_and_subrecords_for_database_rules() {
        let mut parsed = SpreadsheetParsedWorkbook {
            format_version: PLAYER_MONTHLY_FORMAT.into(),
            source_file_name: "原资料包.xlsx".into(),
            source_sha256: "source".into(),
            rows: vec![],
        };
        for entity_type in [
            SpreadsheetEntityType::Player,
            SpreadsheetEntityType::PlayerAbility,
            SpreadsheetEntityType::PlayerAbility,
            SpreadsheetEntityType::PlayerDynamicTag,
            SpreadsheetEntityType::PlayerTeamPeriod,
            SpreadsheetEntityType::PlayerTeamPeriod,
        ] {
            parsed.rows.push(SpreadsheetRawRow {
                sheet_name: "球员与评分".into(),
                row_number: 4,
                entity_type,
                action: SpreadsheetAction::Skip,
                values: json!({}),
            });
        }
        let before = serde_json::to_value(&parsed).unwrap();
        validate_workbook_row_locations(&parsed).unwrap();
        assert_eq!(serde_json::to_value(&parsed).unwrap(), before);
        parsed.rows[5].row_number = u32::MAX;
        assert!(validate_workbook_row_locations(&parsed).is_err());
    }
}
