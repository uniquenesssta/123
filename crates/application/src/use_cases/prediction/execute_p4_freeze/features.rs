use crate::use_cases::prediction::shared::p4_planning::canonical_fact_keys;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    EvidenceVerificationState, P4FreezeReadiness, P4FreezeTaskRecord, P4RoutedFact,
    SnapshotFeatureDraft,
};
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn snapshot_features(
    task: &P4FreezeTaskRecord,
    data_quality: &Value,
    readiness: &P4FreezeReadiness,
    routes: &[P4RoutedFact],
) -> ApplicationResult<Vec<SnapshotFeatureDraft>> {
    let mut by_field = BTreeMap::<String, Vec<&P4RoutedFact>>::new();
    for route in routes {
        by_field
            .entry(route.field_key.clone())
            .or_default()
            .push(route);
    }
    let mut features = Vec::with_capacity(31);
    for (index, field_key) in canonical_fact_keys().into_iter().enumerate() {
        let field_routes = by_field.get(&field_key).cloned().unwrap_or_default();
        let mut evidence_ids = field_routes
            .iter()
            .flat_map(|route| route.selected_evidence_ids.iter().copied())
            .collect::<Vec<_>>();
        evidence_ids.sort_unstable();
        evidence_ids.dedup();
        let verification_state = aggregate_verification_state(&field_routes)?;
        let value = if field_routes.is_empty() {
            Value::Null
        } else if field_routes.len() == 1 {
            field_routes[0].selected_value.clone()
        } else {
            Value::Array(
                field_routes
                    .iter()
                    .map(|route| {
                        json!({
                            "route_key": route.route_key,
                            "entity_target": {
                                "module": route.target_module,
                                "slot": route.target_slot,
                            },
                            "value": route.selected_value,
                        })
                    })
                    .collect(),
            )
        };
        features.push(SnapshotFeatureDraft {
            field_order: u8::try_from(index + 1)
                .map_err(|_| ApplicationError::Validation("P4快照字段序号溢出".to_string()))?,
            field_key,
            value,
            verification_state,
            evidence_ids,
            metadata: json!({
                "orchestration_task_id": task.id,
                "route_count": field_routes.len(),
            }),
        });
    }
    features.push(SnapshotFeatureDraft {
        field_order: 30,
        field_key: "database_pre_match_features".to_string(),
        value: data_quality.clone(),
        verification_state: EvidenceVerificationState::NotApplicable,
        evidence_ids: Vec::new(),
        metadata: json!({"source": "PostgreSQL cutoff-aware deterministic preparation"}),
    });
    features.push(SnapshotFeatureDraft {
        field_order: 31,
        field_key: "orchestration_readiness".to_string(),
        value: serde_json::to_value(readiness)?,
        verification_state: EvidenceVerificationState::NotApplicable,
        evidence_ids: Vec::new(),
        metadata: json!({
            "contract_version": football_domain::P4_ORCHESTRATION_CONTRACT_VERSION,
            "state": "READY_TO_FREEZE",
        }),
    });
    Ok(features)
}

fn aggregate_verification_state(
    routes: &[&P4RoutedFact],
) -> ApplicationResult<EvidenceVerificationState> {
    if routes.is_empty() {
        return Ok(EvidenceVerificationState::NotFound);
    }
    let mut states = BTreeSet::new();
    for route in routes {
        states.insert(route.verification_state.as_str());
    }
    if states.contains("CONFLICT") || states.contains("STALE") {
        return Err(ApplicationError::Validation(
            "READY_TO_FREEZE任务仍包含CONFLICT或STALE路由".to_string(),
        ));
    }
    if states.contains("PROBABLE") {
        Ok(EvidenceVerificationState::Probable)
    } else if states.contains("CONFIRMED") {
        Ok(EvidenceVerificationState::Confirmed)
    } else if states.contains("NOT_FOUND") {
        Ok(EvidenceVerificationState::NotFound)
    } else {
        Ok(EvidenceVerificationState::NotApplicable)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::p4_orchestration::tests::task_fixture;
    use football_domain::P4FreezeTaskState;
    use uuid::Uuid;

    fn readiness(task: &P4FreezeTaskRecord) -> P4FreezeReadiness {
        P4FreezeReadiness {
            task_id: task.id,
            ready: true,
            research_status: Some("SUCCEEDED".into()),
            requested_fact_count: 29,
            routed_fact_count: 29,
            missing_fact_count: 0,
            ignored_fact_count: 0,
            blocked_fact_count: 0,
            blockers: vec![],
        }
    }
    fn route(key: &str, state: &str, ids: Vec<Uuid>, value: Value) -> P4RoutedFact {
        P4RoutedFact {
            route_key: key.into(),
            field_key: canonical_fact_keys()[0].clone(),
            target_module: "personnel".into(),
            target_slot: key.into(),
            route_status: "routed".into(),
            verification_state: state.into(),
            selected_evidence_ids: ids,
            selected_value: value,
            reason: "fixture".into(),
        }
    }
    #[test]
    fn feature_projection_preserves_31_fields_route_order_values_and_deduplicated_provenance() {
        let task = task_fixture(P4FreezeTaskState::Freezing);
        let ready = readiness(&task);
        let a = Uuid::from_u128(1);
        let b = Uuid::from_u128(2);
        let routes = [
            route("second", "CONFIRMED", vec![b, a, b], json!({"raw":"原文"})),
            route("first", "PROBABLE", vec![a], json!(false)),
        ];
        let fields = snapshot_features(&task, &json!({"cutoff": "raw"}), &ready, &routes).unwrap();
        assert_eq!(
            fields.iter().map(|f| f.field_order).collect::<Vec<_>>(),
            (1..=31).collect::<Vec<u8>>()
        );
        assert_eq!(
            fields[0].verification_state,
            EvidenceVerificationState::Probable
        );
        assert_eq!(fields[0].evidence_ids, [a, b]);
        assert_eq!(fields[0].value[0]["route_key"], "second");
        assert_eq!(fields[0].value[0]["value"], routes[0].selected_value);
        assert_eq!(fields[0].value[1]["value"], json!(false));
        assert_eq!(fields[1].value, Value::Null);
        assert_eq!(
            fields[1].verification_state,
            EvidenceVerificationState::NotFound
        );
        assert_eq!(fields[29].value, json!({"cutoff":"raw"}));
        assert_eq!(fields[30].value, serde_json::to_value(ready).unwrap());
        let single =
            snapshot_features(&task, &Value::Null, &readiness(&task), &routes[..1]).unwrap();
        assert_eq!(
            single[0].value, routes[0].selected_value,
            "single route keeps its raw shape"
        );
    }
    #[test]
    fn feature_projection_rejects_conflict_or_stale_without_inventing_values() {
        let task = task_fixture(P4FreezeTaskState::Freezing);
        for state in ["CONFLICT", "STALE"] {
            let routes = [
                route("ok", "CONFIRMED", vec![], json!(7)),
                route("bad", state, vec![], Value::Null),
            ];
            assert!(
                matches!(snapshot_features(&task, &Value::Null, &readiness(&task), &routes), Err(ApplicationError::Validation(ref message)) if message == "READY_TO_FREEZE任务仍包含CONFLICT或STALE路由")
            );
        }
    }
}
