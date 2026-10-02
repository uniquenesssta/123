use crate::ApplicationResult;
use football_domain::PREDICTION_INPUT_AUDIT_VERSION;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

pub(crate) fn build_prediction_input_manifest(
    input: &Value,
    data_quality: &Value,
    match_record: &football_domain::MatchRecord,
    snapshot_type: &str,
    route_identity: Option<&Value>,
) -> Value {
    let mut canonical_input = input.clone();
    strip_runtime_prediction_input_identity(&mut canonical_input);
    json!({
        "audit_version": PREDICTION_INPUT_AUDIT_VERSION,
        "match": {
            "database_match_id": match_record.id,
            "match_key": match_record.external_key,
            "competition_id": match_record.competition_id,
            "season_id": match_record.season_id,
            "stage_id": match_record.stage_id,
            "home_team_id": match_record.home_team_id,
            "away_team_id": match_record.away_team_id,
            "kickoff_time": match_record.kickoff_time,
        },
        "snapshot_type": snapshot_type,
        "route_identity": route_identity,
        "model_input": canonical_input,
        "data_quality": data_quality,
    })
}

fn strip_runtime_prediction_input_identity(input: &mut Value) {
    let Some(object) = input.as_object_mut() else {
        return;
    };
    object.remove("feature_snapshot_id");
    object.remove("input_audit");
    if let Some(snapshot) = object.get_mut("snapshot").and_then(Value::as_object_mut) {
        snapshot.remove("snapshot_id");
        snapshot.remove("frozen_at");
    }
    if let Some(sources) = object.get_mut("sources").and_then(Value::as_array_mut) {
        for source in sources {
            if let Some(source) = source.as_object_mut() {
                source.remove("accessed_at");
            }
        }
    }
}

pub(crate) fn sha256_value(value: &Value) -> ApplicationResult<String> {
    let bytes = serde_json::to_vec(value)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}
