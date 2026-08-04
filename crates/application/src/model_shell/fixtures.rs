use serde_json::{json, Value};

pub fn default_match() -> Value {
    json!({
        "match_id": "PUBLIC-SHELL-DEMO",
        "kickoff_time": "2030-01-01T12:00:00Z",
        "team_a": { "name": "Home Team" },
        "team_b": { "name": "Away Team" },
        "public_shell": true
    })
}

pub fn p4_default_match() -> Value {
    default_match()
}

pub fn default_parameters() -> Value {
    external_parameters("PUBLIC_P7_EXTERNAL_PROFILE")
}

pub fn p4_default_parameters() -> Value {
    external_parameters("PUBLIC_P4_EXTERNAL_PROFILE")
}

fn external_parameters(profile_id: &str) -> Value {
    json!({
        "model_version": "external-provider",
        "parameter_version": "external-provider",
        "provider": {
            "kind": "external",
            "bundled": false
        },
        "profile": {
            "profile_id": profile_id,
            "competition_type": "custom",
            "runtime": "external"
        }
    })
}
