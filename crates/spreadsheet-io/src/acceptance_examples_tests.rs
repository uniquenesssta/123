//! Check the exact files named by the Windows Full instructions with production readers.
//! Parsing fixture rows does not certify a database import or a formal model run.
use super::{read_match_lineup_workbook, read_player_catalog_workbook};
use football_domain::{SpreadsheetEntityType, SpreadsheetParsedWorkbook};
use std::path::PathBuf;

fn example(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/全链路模拟")
        .join(name)
}

fn count(workbook: &SpreadsheetParsedWorkbook, entity: SpreadsheetEntityType) -> usize {
    workbook
        .rows
        .iter()
        .filter(|row| row.entity_type == entity)
        .count()
}

#[test]
fn full_instructions_player_workbook_is_readable_and_nonempty() {
    let workbook = read_player_catalog_workbook(&example("01_球队球员模拟导入.xlsx"))
        .expect("the Full instructions must name an existing compatible workbook");
    assert_eq!(count(&workbook, SpreadsheetEntityType::Team), 4);
    assert_eq!(count(&workbook, SpreadsheetEntityType::Player), 72);
    assert_eq!(count(&workbook, SpreadsheetEntityType::PlayerPosition), 72);
    assert_eq!(
        count(&workbook, SpreadsheetEntityType::PlayerTeamPeriod),
        72
    );
    assert_eq!(count(&workbook, SpreadsheetEntityType::PlayerAbility), 864);
    assert_eq!(
        count(&workbook, SpreadsheetEntityType::PlayerAvailability),
        72
    );
}

#[test]
fn full_instructions_match_workbook_is_readable_and_nonempty() {
    let workbook = read_match_lineup_workbook(&example("02_比赛阵容模拟导入.xlsx"))
        .expect("the Full instructions must name an existing compatible workbook");
    assert_eq!(count(&workbook, SpreadsheetEntityType::Match), 3);
    assert_eq!(count(&workbook, SpreadsheetEntityType::Lineup), 12);
    assert_eq!(count(&workbook, SpreadsheetEntityType::LineupPlayer), 216);
    assert_eq!(
        count(&workbook, SpreadsheetEntityType::PlayerDynamicTag),
        576
    );
}
