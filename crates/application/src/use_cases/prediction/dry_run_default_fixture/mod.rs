use super::shared::routing::{nested_required_string, parse_kickoff, required_string};
use crate::model_registry::ModelRegistry;
use crate::model_shell::P4_MODEL_ID;
use crate::{p4_default_match, p4_default_parameters, ApplicationError, ApplicationResult};
use football_domain::{CompetitionKind, MatchContext, ModelIdentity};
use football_model_api::{ModelOutput, ModelRequest};
use serde_json::Value;

pub(crate) fn execute(registry: &ModelRegistry) -> ApplicationResult<ModelOutput> {
    let model = registry
        .get(P4_MODEL_ID)
        .ok_or_else(|| ApplicationError::ModelNotFound(P4_MODEL_ID.to_string()))?;
    let parameters = p4_default_parameters();
    let match_input = p4_default_match();
    let request = ModelRequest {
        context: MatchContext {
            match_key: required_string(&match_input, "match_id")?,
            kickoff_time: parse_kickoff(&required_string(&match_input, "kickoff_time")?)?,
            competition_id: None,
            season_id: None,
            stage_id: None,
            competition_kind: CompetitionKind::Custom,
            home_team_name: nested_required_string(&match_input, "team_a", "name")?,
            away_team_name: nested_required_string(&match_input, "team_b", "name")?,
            metadata: Value::Null,
        },
        identity: ModelIdentity {
            model_id: P4_MODEL_ID.to_string(),
            model_version: required_string(&parameters, "model_version")?,
            parameter_version: required_string(&parameters, "parameter_version")?,
            rule_package_version: None,
        },
        snapshot_type: "T-1h".to_string(),
        input: match_input,
        parameters,
    };
    model
        .predict(&request)
        .map_err(|error| ApplicationError::Model(error.to_string()))
}
