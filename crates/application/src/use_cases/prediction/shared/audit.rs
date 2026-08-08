use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    MatchPredictionReadiness, PredictionInputAuditSummary, PREDICTION_INPUT_AUDIT_VERSION,
};
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

pub(crate) fn strip_runtime_prediction_input_identity(input: &mut Value) {
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

pub(crate) fn verify_prepared_input_matches_readiness(
    prepared: &football_domain::PreparedMatchPredictionInput,
    readiness: &MatchPredictionReadiness,
) -> ApplicationResult<()> {
    let expected_sha256 = readiness.input_manifest_sha256.as_deref().ok_or_else(|| {
        ApplicationError::Validation("完整度门禁没有生成输入指纹，禁止执行推演".to_string())
    })?;
    let manifest = build_prediction_input_manifest(
        &prepared.match_input,
        &prepared.data_quality,
        &prepared.match_record,
        &prepared.snapshot_type,
        readiness.route_identity.as_ref(),
    );
    let actual_sha256 = sha256_value(&manifest)?;
    if actual_sha256 != expected_sha256 {
        return Err(ApplicationError::Validation(
            "赛前数据在完整度检查后发生变化，请重新检查完整度再执行推演".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn attach_prediction_input_audit(
    input: &mut Value,
    readiness: &MatchPredictionReadiness,
) -> ApplicationResult<()> {
    let manifest = readiness.input_manifest.clone().ok_or_else(|| {
        ApplicationError::Validation("完整度门禁没有生成输入清单，禁止执行推演".to_string())
    })?;
    let manifest_sha256 = readiness.input_manifest_sha256.clone().ok_or_else(|| {
        ApplicationError::Validation("完整度门禁没有生成输入指纹，禁止执行推演".to_string())
    })?;
    let object = input
        .as_object_mut()
        .ok_or_else(|| ApplicationError::Model("模型输入必须是 JSON 对象".to_string()))?;
    object.insert(
        "input_audit".to_string(),
        json!({
            "audit_version": readiness.audit_version,
            "assessed_at": readiness.assessed_at,
            "readiness": {
                "level": readiness.level.as_str(),
                "score": readiness.score,
                "can_run_formal": readiness.can_run_formal,
                "can_run_shadow": readiness.can_run_shadow,
                "blockers": readiness.blockers,
                "warnings": readiness.warnings,
                "checks": readiness.checks,
            },
            "manifest": manifest,
            "manifest_sha256": manifest_sha256,
        }),
    );
    Ok(())
}

pub(crate) fn prediction_input_audit_summary(
    input: &Value,
    input_sha256: &str,
) -> ApplicationResult<Option<PredictionInputAuditSummary>> {
    let Some(audit) = input.get("input_audit") else {
        return Ok(None);
    };
    let audit_version = audit
        .get("audit_version")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| ApplicationError::Model("input_audit.audit_version 不能为空".to_string()))?;
    let manifest = audit
        .get("manifest")
        .ok_or_else(|| ApplicationError::Model("input_audit.manifest 不能为空".to_string()))?;
    let calculated_manifest_sha256 = sha256_value(manifest)?;
    let manifest_sha256 = audit
        .get("manifest_sha256")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            ApplicationError::Model("input_audit.manifest_sha256 不能为空".to_string())
        })?;
    if calculated_manifest_sha256 != manifest_sha256 {
        return Err(ApplicationError::Model(
            "赛前输入清单 SHA256 与实际清单不一致".to_string(),
        ));
    }
    Ok(Some(PredictionInputAuditSummary {
        audit_version: audit_version.to_string(),
        readiness_level: audit
            .get("readiness")
            .and_then(|value| value.get("level"))
            .and_then(Value::as_str)
            .unwrap_or("not_assessed")
            .to_string(),
        readiness_score: audit
            .get("readiness")
            .and_then(|value| value.get("score"))
            .and_then(Value::as_u64)
            .and_then(|value| u8::try_from(value).ok()),
        input_manifest_sha256: manifest_sha256.to_string(),
        input_sha256: input_sha256.to_string(),
    }))
}

pub(crate) fn sha256_value(value: &Value) -> ApplicationResult<String> {
    let bytes = serde_json::to_vec(value)?;
    Ok(hex::encode(Sha256::digest(bytes)))
}
