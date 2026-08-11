use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    SpreadsheetAction, SpreadsheetEntityType, SpreadsheetParsedWorkbook, SpreadsheetRawRow,
};
use std::collections::HashMap;

pub(super) fn collect_team_references(
    parsed: &SpreadsheetParsedWorkbook,
) -> ApplicationResult<HashMap<String, String>> {
    let mut references = HashMap::new();
    for row in parsed.rows.iter().filter(|row| {
        row.entity_type == SpreadsheetEntityType::Team && row.action != SpreadsheetAction::Skip
    }) {
        let Some(values) = row.values.as_object() else {
            continue;
        };
        let key = values
            .get("short_name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let name = values
            .get("official_name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let (Some(key), Some(name)) = (key, name) else {
            continue;
        };
        let normalized_key = key.to_ascii_uppercase();
        if let Some(existing) = references.insert(normalized_key, name.to_string()) {
            if existing != name {
                return Err(ApplicationError::Validation(format!(
                    "球队简称 {key} 在资料包中对应多个球队：{existing} / {name}"
                )));
            }
        }
    }
    Ok(references)
}

pub(super) fn team_rows(parsed: &SpreadsheetParsedWorkbook) -> Vec<SpreadsheetRawRow> {
    parsed
        .rows
        .iter()
        .filter(|row| is_team_entity(row.entity_type))
        .cloned()
        .collect()
}

pub(super) fn player_rows(parsed: &SpreadsheetParsedWorkbook) -> Vec<SpreadsheetRawRow> {
    parsed
        .rows
        .iter()
        .filter(|row| is_player_entity(row.entity_type))
        .cloned()
        .collect()
}

fn is_team_entity(entity_type: SpreadsheetEntityType) -> bool {
    matches!(
        entity_type,
        SpreadsheetEntityType::Team
            | SpreadsheetEntityType::TeamName
            | SpreadsheetEntityType::Coach
            | SpreadsheetEntityType::CoachName
            | SpreadsheetEntityType::TeamCoachPeriod
            | SpreadsheetEntityType::FormationUsage
            | SpreadsheetEntityType::TeamTacticalObservation
            | SpreadsheetEntityType::TeamAbilityObservation
    )
}

fn is_player_entity(entity_type: SpreadsheetEntityType) -> bool {
    matches!(
        entity_type,
        SpreadsheetEntityType::Player
            | SpreadsheetEntityType::PlayerName
            | SpreadsheetEntityType::PlayerPosition
            | SpreadsheetEntityType::PlayerTeamPeriod
            | SpreadsheetEntityType::PlayerAbility
            | SpreadsheetEntityType::PlayerAvailability
            | SpreadsheetEntityType::PlayerDynamicTag
            | SpreadsheetEntityType::ExternalEntityId
    )
}
