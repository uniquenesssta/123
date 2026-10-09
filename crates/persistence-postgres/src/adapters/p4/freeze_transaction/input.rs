use crate::adapters::p4::idempotency::validate_idempotency_key;
use crate::{sha256_json, PersistenceError, PersistenceResult};
use football_domain::{
    PrematchSnapshotDraft, SnapshotFeatureDraft, SnapshotProbabilityDraft, P4_FEATURE_FIELD_COUNT,
};
use serde_json::json;
use std::collections::{BTreeSet, HashSet};

const PROBABILITY_TOLERANCE: f64 = 1e-9;

pub(super) struct PreparedSnapshot {
    pub(super) payload_sha256: String,
    pub(super) feature_set_sha256: String,
    pub(super) evidence_set_sha256: String,
    pub(super) probability_set_sha256: String,
    pub(super) snapshot_fingerprint: String,
    pub(super) features: Vec<SnapshotFeatureDraft>,
    pub(super) probabilities: Vec<SnapshotProbabilityDraft>,
}

impl PreparedSnapshot {
    pub(super) fn new(draft: &PrematchSnapshotDraft) -> PersistenceResult<Self> {
        validate_snapshot_draft(draft)?;
        let mut features = draft.features.clone();
        features.sort_by_key(|feature| feature.field_order);
        let mut probabilities = draft.probabilities.clone();
        probabilities.sort_by(|left, right| left.chain_key.cmp(&right.chain_key));
        let payload_sha256 = sha256_json(&draft.input_payload)?;
        let feature_set_sha256 = sha256_json(&features)?;
        let evidence_ids = features
            .iter()
            .flat_map(|feature| feature.evidence_ids.iter().copied())
            .collect::<BTreeSet<_>>();
        let evidence_set_sha256 = sha256_json(&evidence_ids)?;
        let probability_set_sha256 = sha256_json(&probabilities)?;
        let snapshot_fingerprint = sha256_json(&json!({
            "contract": football_domain::P4_PERSISTENCE_CONTRACT_VERSION,
            "match_id": draft.match_id,
            "match_key": draft.match_key,
            "horizon": draft.horizon.as_str(),
            "data_cutoff_at": draft.data_cutoff_at,
            "model_version_id": draft.model_version_id,
            "parameter_set_id": draft.parameter_set_id,
            "competition_profile_id": draft.competition_profile_id,
            "schema_version_id": draft.schema_version_id,
            "schema_version": draft.schema_version,
            "source_kind": draft.source_kind.as_str(),
            "evidence_scope": draft.source_kind.evidence_scope(),
            "payload_sha256": payload_sha256,
            "feature_set_sha256": feature_set_sha256,
            "evidence_set_sha256": evidence_set_sha256,
            "probability_set_sha256": probability_set_sha256,
        }))?;
        Ok(Self {
            payload_sha256,
            feature_set_sha256,
            evidence_set_sha256,
            probability_set_sha256,
            snapshot_fingerprint,
            features,
            probabilities,
        })
    }
}
fn validate_snapshot_draft(draft: &PrematchSnapshotDraft) -> PersistenceResult<()> {
    validate_idempotency_key(&draft.idempotency_key)?;
    if !draft.horizon.is_canonical() {
        return Err(PersistenceError::InvalidState(
            "P4计划冻结只接受T-24h、T-6h或T-1h；T-N由正式推演按需读取".to_string(),
        ));
    }
    if draft.match_key.trim().is_empty() || draft.schema_version.trim().is_empty() {
        return Err(PersistenceError::InvalidState(
            "match_key和schema_version不能为空".to_string(),
        ));
    }
    if draft.frozen_at < draft.data_cutoff_at {
        return Err(PersistenceError::InvalidState(
            "frozen_at不能早于data_cutoff_at".to_string(),
        ));
    }
    if !draft.quality_score.is_finite() || !(0.0..=1.0).contains(&draft.quality_score) {
        return Err(PersistenceError::InvalidState(
            "quality_score必须在0..=1范围内".to_string(),
        ));
    }
    if draft.features.len() != P4_FEATURE_FIELD_COUNT {
        return Err(PersistenceError::InvalidState(format!(
            "P4赛前快照必须包含{P4_FEATURE_FIELD_COUNT}个A:AE语义字段，实际{}个",
            draft.features.len()
        )));
    }
    let orders = draft
        .features
        .iter()
        .map(|feature| feature.field_order)
        .collect::<BTreeSet<_>>();
    let expected_orders = (1..=P4_FEATURE_FIELD_COUNT as u8).collect::<BTreeSet<_>>();
    if orders != expected_orders {
        return Err(PersistenceError::InvalidState(
            "P4字段顺序必须完整覆盖1..=31且不得重复".to_string(),
        ));
    }
    let field_keys = draft
        .features
        .iter()
        .map(|feature| feature.field_key.trim())
        .collect::<HashSet<_>>();
    if field_keys.len() != P4_FEATURE_FIELD_COUNT || field_keys.contains("") {
        return Err(PersistenceError::InvalidState(
            "P4字段键不能为空或重复".to_string(),
        ));
    }
    let chains = draft
        .probabilities
        .iter()
        .map(|probability| probability.chain_key.trim())
        .collect::<BTreeSet<_>>();
    if chains.is_empty() || chains.contains("") || chains.len() != draft.probabilities.len() {
        return Err(PersistenceError::InvalidState(
            "冻结快照至少需要一条名称非空且不重复的外部模型概率链".to_string(),
        ));
    }
    for probability in &draft.probabilities {
        let values = [probability.home_win, probability.draw, probability.away_win];
        if values
            .iter()
            .any(|value| !value.is_finite() || !(0.0..=1.0).contains(value))
            || (values.iter().sum::<f64>() - 1.0).abs() > PROBABILITY_TOLERANCE
        {
            return Err(PersistenceError::InvalidState(format!(
                "{}链1X2概率必须有限、位于0..=1且和为1",
                probability.chain_key
            )));
        }
        if probability.matrix_cell_count == 0
            || probability.matrix_sha256.len() != 64
            || !probability
                .matrix_sha256
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        {
            return Err(PersistenceError::InvalidState(format!(
                "{}链矩阵必须非空并带小写SHA-256",
                probability.chain_key
            )));
        }
        for optional in [
            probability.btts,
            probability.over_2_5,
            probability.clean_sheet_home,
            probability.clean_sheet_away,
        ]
        .into_iter()
        .flatten()
        {
            if !optional.is_finite() || !(0.0..=1.0).contains(&optional) {
                return Err(PersistenceError::InvalidState(
                    "概率扩展指标必须在0..=1范围内".to_string(),
                ));
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Duration, Utc};
    use football_domain::{EvidenceVerificationState, P4Horizon, SnapshotSourceKind};
    use uuid::Uuid;

    fn feature(order: u8) -> SnapshotFeatureDraft {
        SnapshotFeatureDraft {
            field_order: order,
            field_key: format!("field_{order:02}"),
            value: json!({"value": order}),
            verification_state: EvidenceVerificationState::Confirmed,
            evidence_ids: Vec::new(),
            metadata: json!({}),
        }
    }

    fn probability(chain: &str) -> SnapshotProbabilityDraft {
        SnapshotProbabilityDraft {
            chain_key: chain.to_string(),
            home_win: 0.47,
            draw: 0.30,
            away_win: 0.23,
            btts: Some(0.46),
            over_2_5: Some(0.42),
            clean_sheet_home: Some(0.38),
            clean_sheet_away: Some(0.23),
            matrix_sha256: "a".repeat(64),
            matrix_cell_count: 1,
            metadata: json!({}),
        }
    }

    fn snapshot() -> PrematchSnapshotDraft {
        let cutoff = Utc::now();
        PrematchSnapshotDraft {
            match_id: Uuid::new_v4(),
            match_key: "TEST-MATCH".to_string(),
            horizon: P4Horizon::T6h,
            data_cutoff_at: cutoff,
            frozen_at: cutoff + Duration::seconds(1),
            model_version_id: Uuid::new_v4(),
            parameter_set_id: Uuid::new_v4(),
            competition_profile_id: Uuid::new_v4(),
            research_run_id: None,
            schema_version_id: Uuid::new_v4(),
            schema_version: "snapshot-v1".to_string(),
            trace_id: Uuid::new_v4(),
            idempotency_key: "snapshot:test".to_string(),
            source_kind: SnapshotSourceKind::SyntheticFixture,
            quality_score: 1.0,
            input_payload: json!({"test": true}),
            features: (1..=31).map(feature).collect(),
            probabilities: ["primary", "secondary"]
                .iter()
                .map(|chain| probability(chain))
                .collect(),
            metadata: json!({}),
        }
    }

    #[test]
    fn snapshot_fingerprint_is_deterministic_and_order_independent() {
        let first = snapshot();
        let mut second = first.clone();
        second.features.reverse();
        second.probabilities.reverse();
        assert_eq!(
            PreparedSnapshot::new(&first).unwrap().snapshot_fingerprint,
            PreparedSnapshot::new(&second).unwrap().snapshot_fingerprint
        );
    }

    #[test]
    fn snapshot_fingerprint_preserves_submicrosecond_input_identity() {
        let mut first = snapshot();
        first.data_cutoff_at = DateTime::parse_from_rfc3339("2026-09-30T12:00:00.123456789Z")
            .unwrap()
            .with_timezone(&Utc);
        first.frozen_at = first.data_cutoff_at + Duration::seconds(1);
        let mut changed = first.clone();
        changed.data_cutoff_at += Duration::nanoseconds(1);
        assert_ne!(
            PreparedSnapshot::new(&first).unwrap().snapshot_fingerprint,
            PreparedSnapshot::new(&changed)
                .unwrap()
                .snapshot_fingerprint
        );
    }

    #[test]
    fn snapshot_requires_all_31_fields_and_at_least_one_chain() {
        let mut draft = snapshot();
        draft.features.pop();
        assert!(validate_snapshot_draft(&draft).is_err());

        let mut draft = snapshot();
        draft.probabilities.clear();
        assert!(validate_snapshot_draft(&draft).is_err());
    }

    #[test]
    fn duplicate_provider_chain_is_rejected() {
        let mut draft = snapshot();
        draft.probabilities[1].chain_key = draft.probabilities[0].chain_key.clone();
        assert!(validate_snapshot_draft(&draft).is_err());
    }

    #[test]
    fn retry_fingerprint_keeps_original_exclusions_for_delivery_metadata() {
        let first = snapshot();
        let mut retry = first.clone();
        retry.frozen_at += Duration::seconds(10);
        retry.idempotency_key = "another-delivery-key".into();
        retry.trace_id = Uuid::new_v4();
        retry.research_run_id = Some(Uuid::new_v4());
        retry.quality_score = 0.3;
        retry.metadata = json!({"delivery":2});
        assert_eq!(
            PreparedSnapshot::new(&first).unwrap().snapshot_fingerprint,
            PreparedSnapshot::new(&retry).unwrap().snapshot_fingerprint
        );
    }
    #[test]
    fn fingerprint_binds_each_immutable_identity_and_raw_payload_set() {
        let first = snapshot();
        let fingerprint = PreparedSnapshot::new(&first).unwrap().snapshot_fingerprint;
        for field in [
            "match",
            "match_key",
            "horizon",
            "cutoff",
            "model",
            "parameter",
            "profile",
            "schema_id",
            "schema",
            "source",
            "payload",
            "feature",
            "evidence",
            "probability",
        ] {
            let mut changed = first.clone();
            match field {
                "match" => changed.match_id = Uuid::new_v4(),
                "match_key" => changed.match_key.push(' '),
                "horizon" => changed.horizon = P4Horizon::T1h,
                "cutoff" => changed.data_cutoff_at += Duration::nanoseconds(1),
                "model" => changed.model_version_id = Uuid::new_v4(),
                "parameter" => changed.parameter_set_id = Uuid::new_v4(),
                "profile" => changed.competition_profile_id = Uuid::new_v4(),
                "schema_id" => changed.schema_version_id = Uuid::new_v4(),
                "schema" => changed.schema_version.push(' '),
                "source" => changed.source_kind = SnapshotSourceKind::Manual,
                "payload" => changed.input_payload = json!({"test":false}),
                "feature" => changed.features[0].value = json!("原文"),
                "evidence" => changed.features[0].evidence_ids.push(Uuid::new_v4()),
                "probability" => changed.probabilities[0].metadata = json!({"formal":true}),
                _ => unreachable!(),
            }
            assert_ne!(
                fingerprint,
                PreparedSnapshot::new(&changed)
                    .unwrap()
                    .snapshot_fingerprint,
                "{field}"
            );
        }
    }
    #[test]
    fn preparation_sorts_copies_without_normalizing_raw_keys_or_evidence_lists() {
        let mut draft = snapshot();
        draft.features[0].field_key = " field_01 ".into();
        let evidence = Uuid::from_u128(1);
        draft.features[0].evidence_ids = vec![evidence, evidence];
        draft.features.reverse();
        draft.probabilities.reverse();
        let before = serde_json::to_value(&draft).unwrap();
        let prepared = PreparedSnapshot::new(&draft).unwrap();
        assert_eq!(serde_json::to_value(&draft).unwrap(), before);
        assert_eq!(prepared.features[0].field_key, " field_01 ");
        assert_eq!(prepared.features[0].evidence_ids, [evidence, evidence]);
        assert_eq!(prepared.probabilities[0].chain_key, "primary");
        assert_eq!(
            prepared.evidence_set_sha256,
            sha256_json(&BTreeSet::from([evidence])).unwrap()
        );
    }
    #[test]
    fn draft_validation_requires_complete_unique_orders_and_trimmed_nonempty_keys() {
        for kind in [
            "order_zero",
            "order_duplicate",
            "order_32",
            "key_empty",
            "key_duplicate",
            "chain_empty",
            "chain_duplicate",
        ] {
            let mut draft = snapshot();
            match kind {
                "order_zero" => draft.features[0].field_order = 0,
                "order_duplicate" => draft.features[0].field_order = 2,
                "order_32" => draft.features[0].field_order = 32,
                "key_empty" => draft.features[0].field_key = " \t".into(),
                "key_duplicate" => {
                    draft.features[0].field_key = format!(" {} ", draft.features[1].field_key)
                }
                "chain_empty" => draft.probabilities[0].chain_key = " ".into(),
                "chain_duplicate" => {
                    draft.probabilities[0].chain_key =
                        format!(" {} ", draft.probabilities[1].chain_key)
                }
                _ => unreachable!(),
            }
            assert!(validate_snapshot_draft(&draft).is_err(), "{kind}");
        }
    }
    #[test]
    fn draft_validation_preserves_finite_probability_sum_matrix_and_optional_boundaries() {
        for value in [f64::NAN, f64::INFINITY, -0.1, 1.1] {
            let mut draft = snapshot();
            draft.quality_score = value;
            assert!(validate_snapshot_draft(&draft).is_err());
            let mut draft = snapshot();
            draft.probabilities[0].home_win = value;
            assert!(validate_snapshot_draft(&draft).is_err());
            let mut draft = snapshot();
            draft.probabilities[0].btts = Some(value);
            assert!(validate_snapshot_draft(&draft).is_err());
        }
        for kind in ["sum", "zero_cells", "short_hash", "upper_hash", "nonhex"] {
            let mut draft = snapshot();
            match kind {
                "sum" => draft.probabilities[0].home_win += 1e-8,
                "zero_cells" => draft.probabilities[0].matrix_cell_count = 0,
                "short_hash" => {
                    let _ = draft.probabilities[0].matrix_sha256.pop();
                }
                "upper_hash" => draft.probabilities[0].matrix_sha256 = "A".repeat(64),
                "nonhex" => draft.probabilities[0].matrix_sha256 = "g".repeat(64),
                _ => unreachable!(),
            }
            assert!(validate_snapshot_draft(&draft).is_err(), "{kind}");
        }
        let mut draft = snapshot();
        draft.quality_score = 0.0;
        draft.probabilities.truncate(1);
        draft.probabilities[0].home_win = 1.0;
        draft.probabilities[0].draw = 0.0;
        draft.probabilities[0].away_win = 0.0;
        draft.probabilities[0].btts = None;
        draft.probabilities[0].matrix_cell_count = u16::MAX;
        assert!(validate_snapshot_draft(&draft).is_ok());
    }
    #[test]
    fn draft_validation_preserves_formal_horizons_inclusive_freeze_time_and_key_bytes() {
        for horizon in P4Horizon::CANONICAL {
            let mut draft = snapshot();
            draft.horizon = horizon;
            draft.frozen_at = draft.data_cutoff_at;
            draft.idempotency_key = "键".repeat(80);
            assert!(validate_snapshot_draft(&draft).is_ok());
            draft.idempotency_key.push('a');
            assert!(validate_snapshot_draft(&draft).is_err());
            draft = snapshot();
            draft.frozen_at = draft.data_cutoff_at - Duration::nanoseconds(1);
            assert!(validate_snapshot_draft(&draft).is_err());
        }
        for horizon in [P4Horizon::T90m, P4Horizon::LegacyTN] {
            let mut draft = snapshot();
            draft.horizon = horizon;
            assert!(validate_snapshot_draft(&draft).is_err());
        }
    }
}
