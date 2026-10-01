use crate::{PersistenceError, PersistenceResult};
use football_domain::{
    SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportMode, SpreadsheetImportRow,
    SpreadsheetRowStatus,
};
use serde_json::Value;
use sqlx::Row;

pub(crate) fn player_import_row_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<SpreadsheetImportRow> {
    let candidates: Value = row.try_get("conflict_candidates")?;
    Ok(SpreadsheetImportRow {
        id: row.try_get("id")?,
        sheet_name: row.try_get("sheet_name")?,
        row_number: row.try_get::<i32, _>("row_number")? as u32,
        entity_type: player_parse_entity_type(&row.try_get::<String, _>("entity_type")?)?,
        action: player_parse_action(&row.try_get::<String, _>("requested_action")?)?,
        status: player_parse_row_status(&row.try_get::<String, _>("status")?)?,
        message: row.try_get("message")?,
        payload: row.try_get("payload")?,
        matched_entity_id: row.try_get("matched_entity_id")?,
        conflict_candidates: serde_json::from_value(candidates)?,
    })
}

pub(crate) fn player_parse_entity_type(v: &str) -> PersistenceResult<SpreadsheetEntityType> {
    match v {
        "team" => Ok(SpreadsheetEntityType::Team),
        "team_name" => Ok(SpreadsheetEntityType::TeamName),
        "coach" => Ok(SpreadsheetEntityType::Coach),
        "coach_name" => Ok(SpreadsheetEntityType::CoachName),
        "team_coach_period" => Ok(SpreadsheetEntityType::TeamCoachPeriod),
        "formation_usage" => Ok(SpreadsheetEntityType::FormationUsage),
        "team_tactical_observation" => Ok(SpreadsheetEntityType::TeamTacticalObservation),
        "team_ability_observation" => Ok(SpreadsheetEntityType::TeamAbilityObservation),
        "player" => Ok(SpreadsheetEntityType::Player),
        "player_name" => Ok(SpreadsheetEntityType::PlayerName),
        "player_position" => Ok(SpreadsheetEntityType::PlayerPosition),
        "player_team_period" => Ok(SpreadsheetEntityType::PlayerTeamPeriod),
        "player_ability" => Ok(SpreadsheetEntityType::PlayerAbility),
        "player_availability" => Ok(SpreadsheetEntityType::PlayerAvailability),
        "player_dynamic_tag" => Ok(SpreadsheetEntityType::PlayerDynamicTag),
        "external_entity_id" => Ok(SpreadsheetEntityType::ExternalEntityId),
        _ => Err(PersistenceError::InvalidState(format!("未知导入实体：{v}"))),
    }
}

pub(crate) fn player_parse_action(v: &str) -> PersistenceResult<SpreadsheetAction> {
    match v {
        "add" => Ok(SpreadsheetAction::Add),
        "update" => Ok(SpreadsheetAction::Update),
        "clear" => Ok(SpreadsheetAction::Clear),
        "skip" => Ok(SpreadsheetAction::Skip),
        _ => Err(PersistenceError::InvalidState(format!("未知导入动作：{v}"))),
    }
}

pub(crate) fn player_parse_row_status(v: &str) -> PersistenceResult<SpreadsheetRowStatus> {
    match v {
        "ready_add" => Ok(SpreadsheetRowStatus::ReadyAdd),
        "ready_update" => Ok(SpreadsheetRowStatus::ReadyUpdate),
        "ready_end_previous" => Ok(SpreadsheetRowStatus::ReadyEndPrevious),
        "conflict" => Ok(SpreadsheetRowStatus::Conflict),
        "error" => Ok(SpreadsheetRowStatus::Error),
        "skip" => Ok(SpreadsheetRowStatus::Skip),
        "imported" => Ok(SpreadsheetRowStatus::Imported),
        _ => Err(PersistenceError::InvalidState(format!("未知导入状态：{v}"))),
    }
}

pub(crate) fn player_parse_import_mode(
    v: Option<&str>,
) -> PersistenceResult<SpreadsheetImportMode> {
    match v.unwrap_or("add_and_update") {
        "add_only" => Ok(SpreadsheetImportMode::AddOnly),
        "add_and_update" => Ok(SpreadsheetImportMode::AddAndUpdate),
        other => Err(PersistenceError::InvalidState(format!(
            "未知导入模式：{other}"
        ))),
    }
}

pub(crate) fn match_row_from_db(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<SpreadsheetImportRow> {
    let candidates: Value = row.try_get("conflict_candidates")?;
    Ok(SpreadsheetImportRow {
        id: row.try_get("id")?,
        sheet_name: row.try_get("sheet_name")?,
        row_number: row.try_get::<i32, _>("row_number")? as u32,
        entity_type: match_parse_entity(&row.try_get::<String, _>("entity_type")?)?,
        action: match_parse_action(&row.try_get::<String, _>("requested_action")?)?,
        status: match_parse_status(&row.try_get::<String, _>("status")?)?,
        message: row.try_get("message")?,
        payload: row.try_get("payload")?,
        matched_entity_id: row.try_get("matched_entity_id")?,
        conflict_candidates: serde_json::from_value(candidates)?,
    })
}

pub(crate) fn match_parse_entity(value: &str) -> PersistenceResult<SpreadsheetEntityType> {
    match value {
        "match" => Ok(SpreadsheetEntityType::Match),
        "lineup" => Ok(SpreadsheetEntityType::Lineup),
        "lineup_player" => Ok(SpreadsheetEntityType::LineupPlayer),
        "player_dynamic_tag" => Ok(SpreadsheetEntityType::PlayerDynamicTag),
        _ => Err(PersistenceError::InvalidState(format!(
            "未知比赛导入实体：{value}"
        ))),
    }
}

pub(crate) fn match_parse_action(value: &str) -> PersistenceResult<SpreadsheetAction> {
    match value {
        "add" => Ok(SpreadsheetAction::Add),
        "update" => Ok(SpreadsheetAction::Update),
        "skip" => Ok(SpreadsheetAction::Skip),
        _ => Err(PersistenceError::InvalidState("未知导入动作".to_string())),
    }
}

pub(crate) fn match_parse_status(value: &str) -> PersistenceResult<SpreadsheetRowStatus> {
    match value {
        "ready_add" => Ok(SpreadsheetRowStatus::ReadyAdd),
        "ready_update" => Ok(SpreadsheetRowStatus::ReadyUpdate),
        "conflict" => Ok(SpreadsheetRowStatus::Conflict),
        "error" => Ok(SpreadsheetRowStatus::Error),
        "skip" => Ok(SpreadsheetRowStatus::Skip),
        "imported" => Ok(SpreadsheetRowStatus::Imported),
        _ => Err(PersistenceError::InvalidState("未知导入状态".to_string())),
    }
}

pub(crate) fn match_parse_mode(value: Option<&str>) -> PersistenceResult<SpreadsheetImportMode> {
    match value.unwrap_or("add_and_update") {
        "add_only" => Ok(SpreadsheetImportMode::AddOnly),
        "add_and_update" => Ok(SpreadsheetImportMode::AddAndUpdate),
        _ => Err(PersistenceError::InvalidState("未知导入模式".to_string())),
    }
}

pub(crate) fn match_mode_text(mode: SpreadsheetImportMode) -> &'static str {
    match mode {
        SpreadsheetImportMode::AddOnly => "add_only",
        SpreadsheetImportMode::AddAndUpdate => "add_and_update",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strict_family_codecs_preserve_allowed_entities_and_error_messages() {
        assert!(player_parse_entity_type("player_position").is_ok());
        assert!(match_parse_entity("lineup_player").is_ok());
        assert!(
            matches!(player_parse_entity_type("lineup"), Err(PersistenceError::InvalidState(message)) if message == "未知导入实体：lineup")
        );
        assert!(
            matches!(match_parse_entity("player_position"), Err(PersistenceError::InvalidState(message)) if message == "未知比赛导入实体：player_position")
        );
        assert!(player_parse_action("clear").is_ok());
        assert!(
            matches!(match_parse_action("clear"), Err(PersistenceError::InvalidState(message)) if message == "未知导入动作")
        );
    }

    #[test]
    fn strict_row_status_mapping_preserves_end_previous_family_boundary() {
        for status in [
            "ready_add",
            "ready_update",
            "conflict",
            "error",
            "skip",
            "imported",
        ] {
            assert_eq!(player_parse_row_status(status).unwrap().as_str(), status);
            assert_eq!(match_parse_status(status).unwrap().as_str(), status);
        }
        assert_eq!(
            player_parse_row_status("ready_end_previous")
                .unwrap()
                .as_str(),
            "ready_end_previous"
        );
        assert!(match_parse_status("ready_end_previous").is_err());
        assert!(
            matches!(player_parse_row_status("bad"), Err(PersistenceError::InvalidState(message)) if message == "未知导入状态：bad")
        );
        assert!(
            matches!(match_parse_status("bad"), Err(PersistenceError::InvalidState(message)) if message == "未知导入状态")
        );
    }

    #[test]
    fn import_mode_defaults_and_strict_rejections_are_preserved() {
        for mode in [None, Some("add_only"), Some("add_and_update")] {
            let player = player_parse_import_mode(mode).unwrap();
            let fixture = match_parse_mode(mode).unwrap();
            assert_eq!(match_mode_text(player), match_mode_text(fixture));
            assert_eq!(match_mode_text(fixture), mode.unwrap_or("add_and_update"));
        }
        assert!(
            matches!(player_parse_import_mode(Some("bad")), Err(PersistenceError::InvalidState(message)) if message == "未知导入模式：bad")
        );
        assert!(
            matches!(match_parse_mode(Some("bad")), Err(PersistenceError::InvalidState(message)) if message == "未知导入模式")
        );
    }
}
