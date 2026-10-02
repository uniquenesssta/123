use super::input_manifest::{
    attach_prediction_input_audit, verify_prepared_input_matches_readiness,
};
use super::shared::routing::normalize_model_selection;
use super::PredictionAccess;
use crate::model_registry::ModelRegistry;
use crate::ports::prediction::PredictionInputPort;
use crate::{ApplicationError, ApplicationResult, PredictionCommand, StoredMatchPredictionCommand};
use football_domain::MatchPredictionReadiness;

pub(crate) async fn execute<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
    persist_run: bool,
) -> ApplicationResult<PredictionCommand> {
    let readiness = super::readiness::execute(port, registry, command.clone()).await?;
    from_assessment(port, command, readiness, persist_run).await
}

async fn from_assessment<P: PredictionInputPort + ?Sized>(
    port: &P,
    command: StoredMatchPredictionCommand,
    readiness: MatchPredictionReadiness,
    persist_run: bool,
) -> ApplicationResult<PredictionCommand> {
    let allowed = if persist_run {
        readiness.can_run_formal
    } else {
        readiness.can_run_shadow
    };
    if !allowed {
        let reasons = if readiness.blockers.is_empty() {
            readiness.warnings.join("；")
        } else {
            readiness.blockers.join("；")
        };
        let mode = if persist_run { "正式" } else { "影子" };
        return Err(ApplicationError::Validation(format!(
            "赛前数据完整度门禁未允许{mode}推演（{}，{} 分）：{}",
            readiness.level.as_str(),
            readiness.score,
            reasons
        )));
    }
    let model_family = normalize_model_selection(&command.model_family)?
        .family
        .to_string();
    let store = port;
    let mut prepared = store
        .prepare_match_input_at(
            command.match_id,
            &command.snapshot_type,
            &model_family,
            readiness.assessed_at,
        )
        .await?;
    verify_prepared_input_matches_readiness(&prepared, &readiness)?;
    attach_prediction_input_audit(&mut prepared.match_input, &readiness)?;
    Ok(PredictionCommand {
        match_input: prepared.match_input,
        snapshot_type: prepared.snapshot_type,
        competition_id: prepared.match_record.competition_id,
        season_id: prepared.match_record.season_id,
        stage_id: prepared.match_record.stage_id,
        competition_kind: prepared.competition_kind,
        model_family: command.model_family,
        explicit_rule_package_id: command.explicit_rule_package_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::PortErrorKind;
    use crate::use_cases::prediction::input_manifest::{
        build_prediction_input_manifest, sha256_value,
    };
    use crate::use_cases::prediction::tests::Probe;
    use chrono::{DateTime, Utc};
    use football_domain::{
        CompetitionKind, MatchRecord, MatchStatus, PredictionReadinessLevel,
        PreparedMatchPredictionInput, PREDICTION_INPUT_AUDIT_VERSION,
    };
    use serde_json::json;
    use uuid::Uuid;

    fn fixture(
        model: &str,
    ) -> (
        StoredMatchPredictionCommand,
        PreparedMatchPredictionInput,
        MatchPredictionReadiness,
    ) {
        let assessed_at = DateTime::parse_from_rfc3339("2026-10-01T12:00:00.123456789Z")
            .unwrap()
            .with_timezone(&Utc);
        let prepared = PreparedMatchPredictionInput {
            match_record: MatchRecord {
                id: Uuid::from_u128(1),
                external_key: "R8-INPUT".into(),
                competition_id: Some(Uuid::from_u128(2)),
                competition_name: Some("Test".into()),
                season_id: Some(Uuid::from_u128(3)),
                stage_id: Some(Uuid::from_u128(4)),
                round_id: None,
                home_team_id: Uuid::from_u128(5),
                home_team_name: "Home".into(),
                away_team_id: Uuid::from_u128(6),
                away_team_name: "Away".into(),
                kickoff_time: assessed_at + chrono::Duration::hours(1),
                status: MatchStatus::Scheduled,
                venue: None,
            },
            competition_kind: CompetitionKind::League,
            snapshot_type: "T-1h".into(),
            match_input: json!({"match_id":"R8-INPUT", "feature_snapshot_id":Uuid::from_u128(7), "team_a":{"strength":12}, "team_b":{"strength":11}, "snapshot":{"data_cutoff_time":assessed_at,"snapshot_id":Uuid::from_u128(8),"frozen_at":assessed_at}}),
            data_quality: json!({"complete":true}),
        };
        let command = StoredMatchPredictionCommand {
            match_id: prepared.match_record.id,
            snapshot_type: prepared.snapshot_type.clone(),
            model_family: model.into(),
            explicit_rule_package_id: Some(Uuid::from_u128(9)),
        };
        let mut readiness = MatchPredictionReadiness {
            audit_version: PREDICTION_INPUT_AUDIT_VERSION.into(),
            match_id: command.match_id,
            match_key: "R8-INPUT".into(),
            snapshot_type: command.snapshot_type.clone(),
            model_family: model.into(),
            assessed_at,
            data_cutoff_at: Some(assessed_at),
            level: PredictionReadinessLevel::FormalReady,
            score: 100,
            can_run_formal: true,
            can_run_shadow: true,
            blockers: vec![],
            warnings: vec![],
            checks: vec![],
            input_manifest: None,
            input_manifest_sha256: None,
            route_identity: Some(
                json!({"rule_package_id":command.explicit_rule_package_id,"model_id":model}),
            ),
        };
        refresh_manifest(&prepared, &mut readiness);
        (command, prepared, readiness)
    }

    fn refresh_manifest(
        prepared: &PreparedMatchPredictionInput,
        readiness: &mut MatchPredictionReadiness,
    ) {
        let manifest = build_prediction_input_manifest(
            &prepared.match_input,
            &prepared.data_quality,
            &prepared.match_record,
            &prepared.snapshot_type,
            readiness.route_identity.as_ref(),
        );
        readiness.input_manifest_sha256 = Some(sha256_value(&manifest).unwrap());
        readiness.input_manifest = Some(manifest);
    }

    fn prepared_probe(prepared: PreparedMatchPredictionInput) -> Probe {
        let probe = Probe::new();
        probe.state.lock().unwrap().prepared_input = Some(prepared);
        probe
    }

    #[tokio::test]
    async fn audited_input_preserves_clock_family_route_and_manifest() {
        for (model, family) in [("P4_LEAGUE", "p4"), ("P7_KNOCKOUT_90", "p7")] {
            for persist_run in [true, false] {
                let (command, prepared, readiness) = fixture(model);
                let probe = prepared_probe(prepared.clone());
                let input =
                    from_assessment(&probe, command.clone(), readiness.clone(), persist_run)
                        .await
                        .unwrap();
                assert_eq!(probe.calls(), vec!["prepare_input_at"]);
                assert_eq!(
                    probe.state.lock().unwrap().input_requests,
                    vec![(
                        command.match_id,
                        command.snapshot_type.clone(),
                        family.into(),
                        readiness.assessed_at
                    )]
                );
                assert_eq!(input.model_family, command.model_family);
                assert_eq!(
                    input.explicit_rule_package_id,
                    command.explicit_rule_package_id
                );
                assert_eq!(input.competition_id, prepared.match_record.competition_id);
                assert_eq!(input.season_id, prepared.match_record.season_id);
                assert_eq!(input.stage_id, prepared.match_record.stage_id);
                assert_eq!(input.competition_kind, prepared.competition_kind);
                assert_eq!(input.snapshot_type, prepared.snapshot_type);
                let mut expected = prepared.match_input;
                attach_prediction_input_audit(&mut expected, &readiness).unwrap();
                assert_eq!(input.match_input, expected);
                assert_eq!(
                    input.match_input["input_audit"]["assessed_at"],
                    json!(readiness.assessed_at)
                );
                assert_eq!(
                    input.match_input["input_audit"]["manifest"],
                    readiness.input_manifest.unwrap()
                );
                assert_eq!(
                    input.match_input["input_audit"]["manifest_sha256"],
                    json!(readiness.input_manifest_sha256)
                );
            }
        }
    }

    #[tokio::test]
    async fn formal_and_shadow_permissions_block_input_io() {
        for (level, persist_run, allowed) in [
            (PredictionReadinessLevel::ShadowOnly, true, false),
            (PredictionReadinessLevel::ShadowOnly, false, true),
            (PredictionReadinessLevel::Blocked, true, false),
            (PredictionReadinessLevel::Blocked, false, false),
        ] {
            let (command, prepared, mut readiness) = fixture("p4");
            readiness.level = level;
            readiness.score = 42;
            readiness.can_run_formal = level.can_run_formal();
            readiness.can_run_shadow = level.can_run_shadow();
            readiness.blockers = vec!["缺少阵容".into()];
            readiness.warnings = vec!["警告".into()];
            let probe = prepared_probe(prepared);
            let result = from_assessment(&probe, command, readiness, persist_run).await;
            if allowed {
                assert!(result.is_ok());
                assert_eq!(probe.calls(), vec!["prepare_input_at"]);
            } else {
                let mode = if persist_run { "正式" } else { "影子" };
                assert!(
                    matches!(result,Err(ApplicationError::Validation(message)) if message==format!("赛前数据完整度门禁未允许{mode}推演（{}，42 分）：缺少阵容",level.as_str()))
                );
                assert!(probe.calls().is_empty());
            }
        }
    }

    #[tokio::test]
    async fn changed_input_manifest_blocks_construction() {
        for change_quality in [false, true] {
            let (command, mut prepared, readiness) = fixture("p4");
            if change_quality {
                prepared.data_quality["complete"] = json!(false);
            } else {
                prepared.match_input["team_a"]["strength"] = json!(13);
            }
            let probe = prepared_probe(prepared);
            assert!(
                matches!(from_assessment(&probe,command,readiness,true).await,Err(ApplicationError::Validation(message)) if message=="赛前数据在完整度检查后发生变化，请重新检查完整度再执行推演")
            );
            assert_eq!(probe.calls(), vec!["prepare_input_at"]);
        }
    }

    #[tokio::test]
    async fn runtime_identity_changes_preserve_assessed_manifest() {
        let (command, mut prepared, readiness) = fixture("p4");
        prepared.match_input["feature_snapshot_id"] = json!(Uuid::from_u128(100));
        prepared.match_input["snapshot"]["snapshot_id"] = json!(Uuid::from_u128(101));
        prepared.match_input["snapshot"]["frozen_at"] = json!("2026-10-01T12:01:00Z");
        let probe = prepared_probe(prepared);
        let input = from_assessment(&probe, command, readiness.clone(), false)
            .await
            .unwrap();
        assert_eq!(
            input.match_input["input_audit"]["manifest_sha256"],
            json!(readiness.input_manifest_sha256)
        );
        assert_eq!(probe.calls(), vec!["prepare_input_at"]);
    }

    #[tokio::test]
    async fn input_port_failure_and_missing_audit_do_not_return_command() {
        let (command, prepared, readiness) = fixture("p4");
        let probe = prepared_probe(prepared.clone());
        probe.fail_at(Some("prepare_input_at"));
        assert!(
            matches!(from_assessment(&probe,command.clone(),readiness.clone(),true).await,Err(ApplicationError::Port(error)) if error.kind==PortErrorKind::Unavailable)
        );
        assert_eq!(probe.calls(), vec!["prepare_input_at"]);
        for missing_hash in [true, false] {
            let mut report = readiness.clone();
            let message = if missing_hash {
                report.input_manifest_sha256 = None;
                "完整度门禁没有生成输入指纹，禁止执行推演"
            } else {
                report.input_manifest = None;
                "完整度门禁没有生成输入清单，禁止执行推演"
            };
            let probe = prepared_probe(prepared.clone());
            assert!(
                matches!(from_assessment(&probe,command.clone(),report,false).await,Err(ApplicationError::Validation(actual)) if actual==message)
            );
            assert_eq!(probe.calls(), vec!["prepare_input_at"]);
        }
    }

    #[tokio::test]
    async fn invalid_family_and_non_object_input_keep_existing_errors() {
        let (mut command, prepared, readiness) = fixture("p4");
        command.model_family = "unknown".into();
        let probe = prepared_probe(prepared);
        assert!(
            matches!(from_assessment(&probe,command,readiness,true).await,Err(ApplicationError::Validation(message)) if message.starts_with("不支持的模型：unknown"))
        );
        assert!(probe.calls().is_empty());
        let (command, mut prepared, mut readiness) = fixture("p4");
        prepared.match_input = json!([]);
        refresh_manifest(&prepared, &mut readiness);
        let probe = prepared_probe(prepared);
        assert!(
            matches!(from_assessment(&probe,command,readiness,true).await,Err(ApplicationError::Model(message)) if message=="模型输入必须是 JSON 对象")
        );
        assert_eq!(probe.calls(), vec!["prepare_input_at"]);
    }
}
