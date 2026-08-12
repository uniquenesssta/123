use super::{metadata::operation_metadata, payload::*};
use crate::{
    ports::{
        player::{PlayerCatalogPort, PlayerSignalPort},
        team::TeamCatalogPort,
    },
    ApplicationError, ApplicationResult,
};
use football_domain::{
    ApiWorkspaceOperationRecord, PlayerAbilityObservationDraft, PlayerAvailabilityDraft,
    PlayerDynamicTagDraft, PlayerNameDraft, PlayerPositionDraft, TeamNameDraft, TeamProfileDraft,
};
use serde_json::Value;

pub(super) async fn execute<P>(
    port: &P,
    operation: &ApiWorkspaceOperationRecord,
) -> ApplicationResult<Value>
where
    P: PlayerCatalogPort + PlayerSignalPort + TeamCatalogPort + ?Sized,
{
    match operation.operation_type.as_str() {
        "add_player_name" => {
            let draft = PlayerNameDraft {
                player_id: required_uuid(&operation.payload, "player_id")?,
                name: required_text(&operation.payload, "name")?,
                language_code: optional_text(&operation.payload, "language_code"),
                is_primary: false,
                valid_from: optional_date(&operation.payload, "valid_from")?,
                valid_to: optional_date(&operation.payload, "valid_to")?,
            };
            Ok(serde_json::to_value(port.add_player_name(&draft).await?)?)
        }
        "assign_player_position" => {
            let draft = PlayerPositionDraft {
                player_id: required_uuid(&operation.payload, "player_id")?,
                position_code: required_text(&operation.payload, "position_code")?,
                proficiency: required_f64(&operation.payload, "proficiency")?,
                default_role_code: optional_text(&operation.payload, "default_role_code"),
                is_primary: false,
                valid_from: optional_date(&operation.payload, "valid_from")?,
                valid_to: optional_date(&operation.payload, "valid_to")?,
                source_document_id: None,
            };
            Ok(serde_json::to_value(
                port.assign_player_position(&draft).await?,
            )?)
        }
        "add_player_availability" => {
            let draft = PlayerAvailabilityDraft {
                player_id: required_uuid(&operation.payload, "player_id")?,
                team_id: optional_uuid(&operation.payload, "team_id")?,
                competition_id: optional_uuid(&operation.payload, "competition_id")?,
                status: availability_status(required_text(&operation.payload, "status")?)?,
                reason: optional_text(&operation.payload, "reason"),
                confidence: optional_f64(&operation.payload, "confidence").unwrap_or(0.5),
                valid_from: required_datetime(&operation.payload, "valid_from")?,
                valid_to: optional_datetime(&operation.payload, "valid_to")?,
                source_document_id: None,
                metadata: operation_metadata(operation)?,
            };
            Ok(serde_json::to_value(port.add_availability(&draft).await?)?)
        }
        "add_player_dynamic_tag" => {
            let draft = PlayerDynamicTagDraft {
                player_id: required_uuid(&operation.payload, "player_id")?,
                tag_code: required_text(&operation.payload, "tag_code")?,
                value: required_f64(&operation.payload, "value")?,
                label: optional_text(&operation.payload, "label"),
                confidence: optional_f64(&operation.payload, "confidence").unwrap_or(0.5),
                observed_at: required_datetime(&operation.payload, "observed_at")?,
                valid_from: required_datetime(&operation.payload, "valid_from")?,
                valid_to: required_datetime(&operation.payload, "valid_to")?,
                competition_id: optional_uuid(&operation.payload, "competition_id")?,
                position_code: optional_text(&operation.payload, "position_code"),
                opponent_team_id: optional_uuid(&operation.payload, "opponent_team_id")?,
                sample_size: optional_i32(&operation.payload, "sample_size").unwrap_or(1),
                source_type: "api_workspace".to_string(),
                calculation_version: optional_text(&operation.payload, "calculation_version")
                    .unwrap_or_else(|| "api-workspace-v2".to_string()),
                source_document_id: None,
                metadata: operation_metadata(operation)?,
            };
            Ok(serde_json::to_value(port.add_dynamic_tag(&draft).await?)?)
        }
        "add_player_ability_observation" => {
            let observed_at = required_datetime(&operation.payload, "observed_at")?;
            let effective_from =
                optional_datetime(&operation.payload, "effective_from")?.unwrap_or(observed_at);
            let draft = PlayerAbilityObservationDraft {
                player_id: required_uuid(&operation.payload, "player_id")?,
                dimension_code: required_text(&operation.payload, "dimension_code")?,
                context_type: optional_text(&operation.payload, "context_type")
                    .unwrap_or_else(|| "general".to_string()),
                context_id: optional_uuid(&operation.payload, "context_id")?,
                value: required_f64(&operation.payload, "value")?,
                confidence: optional_f64(&operation.payload, "confidence").unwrap_or(0.5),
                sample_size: optional_i32(&operation.payload, "sample_size").unwrap_or(1),
                observed_at,
                effective_from,
                effective_to: optional_datetime(&operation.payload, "effective_to")?,
                calculation_version: optional_text(&operation.payload, "calculation_version")
                    .unwrap_or_else(|| "api-workspace-v2".to_string()),
                source_document_id: None,
                metadata: operation_metadata(operation)?,
            };
            Ok(serde_json::to_value(
                port.add_ability_observation(&draft).await?,
            )?)
        }
        "add_team_name" => {
            let draft = TeamNameDraft {
                team_id: required_uuid(&operation.payload, "team_id")?,
                name: required_text(&operation.payload, "name")?,
                language_code: optional_text(&operation.payload, "language_code"),
                valid_from: optional_date(&operation.payload, "valid_from")?,
                valid_to: optional_date(&operation.payload, "valid_to")?,
            };
            Ok(serde_json::to_value(port.add_team_name(&draft).await?)?)
        }
        "update_team_profile" => {
            let team_id = required_uuid(&operation.payload, "team_id")?;
            let current = port.read_team(team_id).await?.profile;
            let draft = TeamProfileDraft {
                short_name: optional_text(&operation.payload, "short_name")
                    .or_else(|| current.as_ref().and_then(|profile| profile.short_name.clone())),
                team_type: optional_text(&operation.payload, "team_type")
                    .or_else(|| current.as_ref().map(|profile| profile.team_type.clone()))
                    .unwrap_or_else(|| "club".to_string()),
                founded_year: optional_i16(&operation.payload, "founded_year")
                    .or_else(|| current.as_ref().and_then(|profile| profile.founded_year)),
                city: optional_text(&operation.payload, "city")
                    .or_else(|| current.as_ref().and_then(|profile| profile.city.clone())),
                stadium: optional_text(&operation.payload, "stadium")
                    .or_else(|| current.as_ref().and_then(|profile| profile.stadium.clone())),
                head_coach: optional_text(&operation.payload, "head_coach")
                    .or_else(|| current.as_ref().and_then(|profile| profile.head_coach.clone())),
                default_formation: optional_text(&operation.payload, "default_formation")
                    .or_else(|| {
                        current
                            .as_ref()
                            .and_then(|profile| profile.default_formation.clone())
                    }),
                tactical_style: optional_text(&operation.payload, "tactical_style")
                    .or_else(|| current.as_ref().map(|profile| profile.tactical_style.clone()))
                    .unwrap_or_else(|| "balanced".to_string()),
                attack_rating: optional_f64(&operation.payload, "attack_rating")
                    .or_else(|| current.as_ref().and_then(|profile| profile.attack_rating)),
                midfield_rating: optional_f64(&operation.payload, "midfield_rating")
                    .or_else(|| current.as_ref().and_then(|profile| profile.midfield_rating)),
                defence_rating: optional_f64(&operation.payload, "defence_rating")
                    .or_else(|| current.as_ref().and_then(|profile| profile.defence_rating)),
                goalkeeper_rating: optional_f64(&operation.payload, "goalkeeper_rating").or_else(
                    || current.as_ref().and_then(|profile| profile.goalkeeper_rating),
                ),
                reputation: optional_f64(&operation.payload, "reputation")
                    .or_else(|| current.as_ref().and_then(|profile| profile.reputation)),
                data_confidence: optional_f64(&operation.payload, "confidence")
                    .or_else(|| current.as_ref().map(|profile| profile.data_confidence))
                    .unwrap_or(operation.confidence),
                notes: optional_text(&operation.payload, "notes")
                    .or_else(|| current.as_ref().and_then(|profile| profile.notes.clone())),
                metadata: operation_metadata(operation)?,
            };
            Ok(serde_json::to_value(
                port.upsert_team_profile(team_id, &draft).await?,
            )?)
        }
        other => Err(ApplicationError::Validation(format!(
            "不允许的API数据库操作：{other}"
        ))),
    }
}
