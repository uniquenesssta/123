use crate::mapping::{optional_uuid, required_datetime, required_uuid};
use crate::{sha256_json, PersistenceError, PersistenceResult};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct PreparedRunInputAudit {
    pub(super) audit_version: String,
    pub(super) readiness_level: String,
    pub(super) readiness_score: Option<i16>,
    pub(super) manifest: Value,
    pub(super) manifest_sha256: String,
}

pub(super) fn prepared_run_input_audit(input: &Value) -> PersistenceResult<PreparedRunInputAudit> {
    let Some(audit) = input.get("input_audit") else {
        let manifest = json!({
            "audit_version": "runtime-input-audit-v0",
            "match_key": input.get("match_id"),
            "database_match_id": input.get("database_match_id"),
            "snapshot": input.get("snapshot"),
            "preparation_version": input.get("preparation_version"),
        });
        return Ok(PreparedRunInputAudit {
            audit_version: "runtime-input-audit-v0".to_string(),
            readiness_level: "not_assessed".to_string(),
            readiness_score: None,
            manifest_sha256: sha256_json(&manifest)?,
            manifest,
        });
    };
    let audit_version = audit
        .get("audit_version")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PersistenceError::InvalidState("input_audit.audit_version 不能为空".to_string())
        })?
        .to_string();
    let readiness = audit.get("readiness").ok_or_else(|| {
        PersistenceError::InvalidState("input_audit.readiness 不能为空".to_string())
    })?;
    let readiness_level = readiness
        .get("level")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PersistenceError::InvalidState("input_audit.readiness.level 不能为空".to_string())
        })?
        .to_string();
    if !matches!(
        readiness_level.as_str(),
        "formal_ready" | "ready_with_warnings" | "shadow_only" | "blocked"
    ) {
        return Err(PersistenceError::InvalidState(format!(
            "未知赛前输入完整度状态：{readiness_level}"
        )));
    }
    let readiness_score = readiness
        .get("score")
        .and_then(Value::as_u64)
        .map(|value| {
            i16::try_from(value).map_err(|_| {
                PersistenceError::InvalidState("input_audit.readiness.score 超出范围".to_string())
            })
        })
        .transpose()?;
    if readiness_score.is_some_and(|value| !(0..=100).contains(&value)) {
        return Err(PersistenceError::InvalidState(
            "input_audit.readiness.score 必须在 0..=100 范围内".to_string(),
        ));
    }
    let manifest = audit.get("manifest").cloned().ok_or_else(|| {
        PersistenceError::InvalidState("input_audit.manifest 不能为空".to_string())
    })?;
    let manifest_sha256 = audit
        .get("manifest_sha256")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PersistenceError::InvalidState("input_audit.manifest_sha256 不能为空".to_string())
        })?
        .to_string();
    let calculated = sha256_json(&manifest)?;
    if calculated != manifest_sha256 {
        return Err(PersistenceError::InvalidState(
            "赛前输入清单 SHA256 与实际清单不一致".to_string(),
        ));
    }
    Ok(PreparedRunInputAudit {
        audit_version,
        readiness_level,
        readiness_score,
        manifest,
        manifest_sha256,
    })
}

#[derive(Debug)]
pub(super) struct PreparedFeatureSnapshot {
    pub(super) id: Uuid,
    pub(super) data_cutoff_time: DateTime<Utc>,
    pub(super) frozen_at: DateTime<Utc>,
    pub(super) schema_version: String,
    pub(super) quality_score: f64,
}

pub(super) fn prepared_feature_snapshot(
    input: &Value,
    expected_snapshot_type: &str,
) -> PersistenceResult<Option<PreparedFeatureSnapshot>> {
    let Some(id) = optional_uuid(input, "feature_snapshot_id")? else {
        return Ok(None);
    };
    let snapshot = input
        .get("snapshot")
        .and_then(Value::as_object)
        .ok_or_else(|| PersistenceError::InvalidState("缺少特征快照元数据".to_string()))?;
    let snapshot_id = snapshot
        .get("snapshot_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            PersistenceError::InvalidState("snapshot.snapshot_id 必须是 UUID 字符串".to_string())
        })?;
    let snapshot_id = required_uuid(snapshot_id, "snapshot.snapshot_id")?;
    if snapshot_id != id {
        return Err(PersistenceError::InvalidState(
            "feature_snapshot_id 与 snapshot.snapshot_id 不一致".to_string(),
        ));
    }
    let snapshot_type = snapshot
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| PersistenceError::InvalidState("snapshot.type 不能为空".to_string()))?;
    if snapshot_type != expected_snapshot_type {
        return Err(PersistenceError::InvalidState(format!(
            "snapshot.type 与运行快照类型不一致：{snapshot_type} != {expected_snapshot_type}"
        )));
    }
    let data_cutoff_time = required_datetime(
        snapshot.get("data_cutoff_time"),
        "snapshot.data_cutoff_time",
    )?;
    let frozen_at = required_datetime(snapshot.get("frozen_at"), "snapshot.frozen_at")?;
    let schema_version = input
        .get("preparation_version")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| PersistenceError::InvalidState("preparation_version 不能为空".to_string()))?
        .to_string();
    let quality_score = input
        .get("feature_quality_score")
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            PersistenceError::InvalidState("feature_quality_score 必须是数值".to_string())
        })?;
    if !quality_score.is_finite() || !(0.0..=1.0).contains(&quality_score) {
        return Err(PersistenceError::InvalidState(
            "feature_quality_score 必须在 0..=1 范围内".to_string(),
        ));
    }
    Ok(Some(PreparedFeatureSnapshot {
        id,
        data_cutoff_time,
        frozen_at,
        schema_version,
        quality_score,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prepared_input_audit_validates_manifest_hash() {
        let manifest = json!({"match_key": "MATCH-1", "snapshot_type": "T-1h"});
        let sha = sha256_json(&manifest).unwrap();
        let input = json!({
            "input_audit": {
                "audit_version": "prematch-input-audit-v1",
                "readiness": {"level": "formal_ready", "score": 96},
                "manifest": manifest,
                "manifest_sha256": sha
            }
        });
        let prepared = prepared_run_input_audit(&input).unwrap();
        assert_eq!(prepared.readiness_level, "formal_ready");
        assert_eq!(prepared.readiness_score, Some(96));

        let mut modified = input;
        modified["input_audit"]["manifest"]["snapshot_type"] = json!("T-6h");
        assert!(prepared_run_input_audit(&modified).is_err());
    }

    fn audited_input() -> Value {
        let manifest = json!({"match_key": "MATCH-1", "snapshot_type": "T-1h"});
        json!({"input_audit": {
            "audit_version": " prematch-input-audit-v1 ",
            "readiness": {"level": " formal_ready ", "score": 100},
            "manifest_sha256": format!(" {} ", sha256_json(&manifest).unwrap()),
            "manifest": manifest
        }})
    }

    fn snapshot_input() -> Value {
        let id = Uuid::from_u128(7);
        json!({
            "feature_snapshot_id": id,
            "preparation_version": " preparation-v1 ",
            "feature_quality_score": 1.0,
            "snapshot": {
                "snapshot_id": id, "type": "T-1h",
                "data_cutoff_time": "2026-10-08T12:00:00.123456789Z",
                "frozen_at": "2026-10-08T12:30:00Z"
            }
        })
    }

    #[test]
    fn legacy_audit_preserves_fallback_shape_hash_and_missing_values() {
        let input = json!({"match_id": "MATCH-1", "snapshot": {"type": "T-1h"}});
        let audit = prepared_run_input_audit(&input).unwrap();
        assert_eq!(audit.audit_version, "runtime-input-audit-v0");
        assert_eq!(audit.readiness_level, "not_assessed");
        assert_eq!(audit.readiness_score, None);
        assert_eq!(
            audit.manifest,
            json!({
                "audit_version": "runtime-input-audit-v0", "match_key": "MATCH-1",
                "database_match_id": null, "snapshot": {"type": "T-1h"},
                "preparation_version": null
            })
        );
        assert_eq!(audit.manifest_sha256, sha256_json(&audit.manifest).unwrap());
        assert!(prepared_run_input_audit(&json!({"input_audit": null})).is_err());
    }

    #[test]
    fn audited_input_preserves_trimmed_identity_levels_and_optional_score() {
        for level in [
            "formal_ready",
            "ready_with_warnings",
            "shadow_only",
            "blocked",
        ] {
            for score in [json!(0), json!(100), Value::Null, json!(-1), json!("99")] {
                let mut input = audited_input();
                input["input_audit"]["readiness"]["level"] = json!(format!(" {level} "));
                input["input_audit"]["readiness"]["score"] = score.clone();
                let audit = prepared_run_input_audit(&input).unwrap();
                assert_eq!(audit.audit_version, "prematch-input-audit-v1");
                assert_eq!(audit.readiness_level, level);
                assert_eq!(audit.readiness_score, score.as_u64().map(|v| v as i16));
                assert_eq!(audit.manifest_sha256, sha256_json(&audit.manifest).unwrap());
            }
        }
        for score in [101_u64, 32768, u64::MAX] {
            let mut input = audited_input();
            input["input_audit"]["readiness"]["score"] = json!(score);
            assert!(prepared_run_input_audit(&input).is_err());
        }
    }

    #[test]
    fn audit_field_errors_keep_priority_and_null_manifest_semantics() {
        let mut input = audited_input();
        input["input_audit"]["audit_version"] = json!(" ");
        input["input_audit"]["readiness"]["level"] = json!("unknown");
        assert!(
            matches!(prepared_run_input_audit(&input), Err(PersistenceError::InvalidState(m)) if m == "input_audit.audit_version 不能为空")
        );
        input["input_audit"]["audit_version"] = json!("v1");
        assert!(
            matches!(prepared_run_input_audit(&input), Err(PersistenceError::InvalidState(m)) if m == "未知赛前输入完整度状态：unknown")
        );
        input["input_audit"]["readiness"]["level"] = json!("formal_ready");
        input["input_audit"]["manifest"] = Value::Null;
        input["input_audit"]["manifest_sha256"] = json!(sha256_json(&Value::Null).unwrap());
        assert!(prepared_run_input_audit(&input).unwrap().manifest.is_null());
        input["input_audit"]
            .as_object_mut()
            .unwrap()
            .remove("manifest");
        assert!(
            matches!(prepared_run_input_audit(&input), Err(PersistenceError::InvalidState(m)) if m == "input_audit.manifest 不能为空")
        );
    }

    #[test]
    fn feature_snapshot_keeps_original_identity_window_and_schema() {
        let input = snapshot_input();
        let snapshot = prepared_feature_snapshot(&input, "T-1h").unwrap().unwrap();
        assert_eq!(snapshot.id, Uuid::from_u128(7));
        assert_eq!(snapshot.schema_version, " preparation-v1 ");
        assert_eq!(snapshot.quality_score, 1.0);
        assert_eq!(
            snapshot.data_cutoff_time.timestamp_subsec_nanos(),
            123456789
        );
        assert_eq!(
            snapshot.frozen_at,
            chrono::DateTime::parse_from_rfc3339("2026-10-08T12:30:00Z").unwrap()
        );
        assert!(prepared_feature_snapshot(&json!({}), "T-1h")
            .unwrap()
            .is_none());
        assert!(prepared_feature_snapshot(&input, " T-1h ").is_err());
    }

    #[test]
    fn feature_snapshot_rejects_missing_metadata_and_changed_identity() {
        for field in ["snapshot", "preparation_version", "feature_quality_score"] {
            let mut input = snapshot_input();
            input.as_object_mut().unwrap().remove(field);
            assert!(
                prepared_feature_snapshot(&input, "T-1h").is_err(),
                "{field}"
            );
        }
        for field in ["snapshot_id", "type", "data_cutoff_time", "frozen_at"] {
            let mut input = snapshot_input();
            input["snapshot"].as_object_mut().unwrap().remove(field);
            assert!(
                prepared_feature_snapshot(&input, "T-1h").is_err(),
                "{field}"
            );
        }
        let mut input = snapshot_input();
        input["snapshot"]["snapshot_id"] = json!(Uuid::from_u128(8));
        assert!(
            matches!(prepared_feature_snapshot(&input, "T-1h"), Err(PersistenceError::InvalidState(m)) if m == "feature_snapshot_id 与 snapshot.snapshot_id 不一致")
        );
        input["feature_snapshot_id"] = json!("invalid-uuid");
        assert!(prepared_feature_snapshot(&input, "T-1h").is_err());
    }

    #[test]
    fn feature_snapshot_quality_keeps_closed_unit_interval_and_type_checks() {
        for quality in [0.0, 0.25, 1.0] {
            let mut input = snapshot_input();
            input["feature_quality_score"] = json!(quality);
            assert_eq!(
                prepared_feature_snapshot(&input, "T-1h")
                    .unwrap()
                    .unwrap()
                    .quality_score,
                quality
            );
        }
        for quality in [json!(-0.0001), json!(1.0001), json!("1"), Value::Null] {
            let mut input = snapshot_input();
            input["feature_quality_score"] = quality;
            assert!(prepared_feature_snapshot(&input, "T-1h").is_err());
        }
    }
}
