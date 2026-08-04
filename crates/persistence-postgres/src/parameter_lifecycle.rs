use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    ParameterCandidateArtifactDraft, ParameterCandidateBaseline, ParameterLifecycleReadiness,
    ParameterLifecycleReadinessRequest, ParameterPromotionDecisionRecord,
    ParameterPromotionRequest, ParameterReplayFixture, ParameterRollbackRequest,
    ParameterShadowValidationRecord, ParameterTuningCandidateRecord,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn parameter_lifecycle_readiness(
        &self,
        request: &ParameterLifecycleReadinessRequest,
    ) -> PersistenceResult<ParameterLifecycleReadiness> {
        let h_contract = sqlx::query(
            r#"
            SELECT contract_version
            FROM platform.integration_contracts
            WHERE contract_key = 'p4-postmatch-settlement'
              AND stage = 'H'
              AND status = 'locked'
              AND COALESCE((metadata->>'settlement_ready')::boolean, false) = true
              AND COALESCE((metadata->>'evidence_queue_ready')::boolean, false) = true
              AND COALESCE((metadata->>'drift_metrics_ready')::boolean, false) = true
            ORDER BY locked_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&self.pool)
        .await?;
        let h_contract_version = h_contract
            .as_ref()
            .map(|row| row.try_get::<String, _>("contract_version"))
            .transpose()?;
        let h_contract_ready = h_contract_version.is_some();

        let baseline = if let Some(competition_id) = request.competition_id {
            match self.parameter_candidate_baseline(competition_id).await {
                Ok(value) => Some(value),
                Err(PersistenceError::InvalidState(message))
                    if message == "未找到该赛事的活动 P4 模型绑定" =>
                {
                    None
                }
                Err(error) => return Err(error),
            }
        } else {
            None
        };

        let settled_sample_count = if let Some(baseline) = baseline.as_ref() {
            let count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)::bigint
                FROM review.postmatch_settlements settlement
                WHERE settlement.competition_id = $1
                  AND settlement.competition_profile_id = $2
                  AND settlement.horizon = $3
                  AND settlement.model_version_id = $4
                  AND settlement.parameter_set_id = $5
                "#,
            )
            .bind(baseline.competition_id)
            .bind(baseline.competition_profile_id)
            .bind(&request.snapshot_type)
            .bind(baseline.model_version_id)
            .bind(baseline.parameter_set_id)
            .fetch_one(&self.pool)
            .await?;
            non_negative_u64(count, "已结算样本数")?
        } else {
            0
        };

        let eligible_sample_count = if let Some(baseline) = baseline.as_ref() {
            let count: i64 = sqlx::query_scalar(
                r#"
                SELECT count(*)::bigint
                FROM analytics.evaluation_samples sample
                JOIN review.postmatch_settlements settlement
                  ON settlement.id = sample.settlement_id
                WHERE settlement.competition_id = $1
                  AND settlement.competition_profile_id = $2
                  AND settlement.horizon = $3
                  AND settlement.model_version_id = $4
                  AND settlement.parameter_set_id = $5
                  AND sample.competition_profile_id = settlement.competition_profile_id
                  AND sample.model_version_id = settlement.model_version_id
                  AND sample.parameter_set_id = settlement.parameter_set_id
                  AND sample.snapshot_type = settlement.horizon
                  AND sample.calculation_version = 'postmatch-monitoring-v1'
                "#,
            )
            .bind(baseline.competition_id)
            .bind(baseline.competition_profile_id)
            .bind(&request.snapshot_type)
            .bind(baseline.model_version_id)
            .bind(baseline.parameter_set_id)
            .fetch_one(&self.pool)
            .await?;
            non_negative_u64(count, "可验证样本数")?
        } else {
            0
        };

        let competition_name = baseline
            .as_ref()
            .map(|value| value.competition_name.clone());
        let competition_profile_id = baseline.as_ref().map(|value| value.competition_profile_id);
        let partition_key = baseline.as_ref().map_or_else(
            || format!("unresolved:all:{}", request.snapshot_type),
            |value| {
                format!(
                    "{}:{}:{}",
                    value.model_version, value.competition_profile_id, request.snapshot_type
                )
            },
        );

        let mut blocked_reasons = Vec::new();
        if request.competition_id.is_none() {
            blocked_reasons.push("必须选择一个具体赛事，禁止跨赛事混合晋升".to_string());
        }
        if !h_contract_ready {
            blocked_reasons.push("接入点 H 的赛果结算、证据队列与漂移契约尚未就绪".to_string());
        }
        if eligible_sample_count < request.minimum_sample_size {
            blocked_reasons.push(format!(
                "可验证样本只有 {eligible_sample_count} 场，低于最低门槛 {} 场",
                request.minimum_sample_size
            ));
        }
        if baseline.is_none() {
            blocked_reasons.push("未找到该赛事的活动 P4 模型绑定".to_string());
        }
        let ready = blocked_reasons.is_empty();
        Ok(ParameterLifecycleReadiness {
            partition_key,
            competition_id: request.competition_id,
            competition_name,
            competition_profile_id,
            snapshot_type: request.snapshot_type.clone(),
            h_contract_ready,
            h_contract_version,
            settled_sample_count,
            eligible_sample_count,
            minimum_sample_size: request.minimum_sample_size,
            active_model_version_id: baseline.as_ref().map(|value| value.model_version_id),
            active_parameter_set_id: baseline.as_ref().map(|value| value.parameter_set_id),
            active_model_version: baseline.as_ref().map(|value| value.model_version.clone()),
            active_parameter_version: baseline
                .as_ref()
                .map(|value| value.parameter_version.clone()),
            blocked_reasons,
            ready_for_shadow_validation: ready,
            ready_for_promotion: ready,
        })
    }

    pub async fn parameter_candidate_baseline(
        &self,
        competition_id: Uuid,
    ) -> PersistenceResult<ParameterCandidateBaseline> {
        let row = sqlx::query(
            r#"
            SELECT
                competition.id AS competition_id,
                competition.name AS competition_name,
                package.competition_profile_id,
                binding.id AS binding_id,
                package.id AS rule_package_id,
                package.version AS rule_package_version,
                definition.model_key,
                version.id AS model_version_id,
                version.version AS model_version,
                version.engine_version,
                version.input_schema_version,
                version.output_schema_version,
                parameter.id AS parameter_set_id,
                parameter.parameter_version,
                parameter.definition AS parameters
            FROM football.competitions competition
            JOIN model.competition_bindings binding
              ON binding.competition_id = competition.id
              OR (
                  binding.competition_id IS NULL
                  AND binding.season_id IS NULL
                  AND binding.stage_id IS NULL
                  AND binding.competition_kind = competition.competition_kind
              )
            JOIN model.rule_packages package ON package.id = binding.rule_package_id
            JOIN model.versions version ON version.id = binding.model_version_id
            JOIN model.definitions definition ON definition.id = version.model_id
            JOIN model.parameter_sets parameter ON parameter.id = binding.parameter_set_id
            WHERE binding.is_active = true
              AND competition.id = $1
              AND package.status = 'active'
              AND version.status IN ('active', 'draft')
              AND parameter.status IN ('active', 'draft')
              AND (definition.model_key = 'p4' OR definition.model_key LIKE 'p4\_%' ESCAPE '\')
            ORDER BY (binding.competition_id IS NOT NULL) DESC,
                     binding.priority DESC,
                     binding.id DESC
            LIMIT 1
            "#,
        )
        .bind(competition_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState("未找到该赛事的活动 P4 模型绑定".to_string())
        })?;
        Ok(ParameterCandidateBaseline {
            competition_id: row.try_get("competition_id")?,
            competition_name: row.try_get("competition_name")?,
            competition_profile_id: row.try_get("competition_profile_id")?,
            binding_id: row.try_get("binding_id")?,
            rule_package_id: row.try_get("rule_package_id")?,
            rule_package_version: row.try_get("rule_package_version")?,
            model_key: row.try_get("model_key")?,
            model_version_id: row.try_get("model_version_id")?,
            model_version: row.try_get("model_version")?,
            engine_version: row.try_get("engine_version")?,
            input_schema_version: row.try_get("input_schema_version")?,
            output_schema_version: row.try_get("output_schema_version")?,
            parameter_set_id: row.try_get("parameter_set_id")?,
            parameter_version: row.try_get("parameter_version")?,
            parameters: row.try_get("parameters")?,
        })
    }

    pub async fn read_parameter_set_definition(
        &self,
        parameter_set_id: Uuid,
    ) -> PersistenceResult<Value> {
        sqlx::query_scalar("SELECT definition FROM model.parameter_sets WHERE id = $1")
            .bind(parameter_set_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("参数版本不存在".to_string()))
    }

    pub async fn save_parameter_tuning_candidate_with_artifacts(
        &self,
        candidate: &ParameterTuningCandidateRecord,
        artifact: &ParameterCandidateArtifactDraft,
    ) -> PersistenceResult<ParameterTuningCandidateRecord> {
        let mut tx = self.pool.begin().await?;
        let inserted_model = sqlx::query(
            r#"
            INSERT INTO model.versions (
                id, model_id, version, engine_version,
                input_schema_version, output_schema_version, source_sha256, status
            )
            SELECT $1, model_id, $2, engine_version,
                   input_schema_version, output_schema_version, source_sha256, 'draft'
            FROM model.versions
            WHERE id = $3
            "#,
        )
        .bind(artifact.candidate_model_version_id)
        .bind(&artifact.candidate_model_version)
        .bind(artifact.baseline.model_version_id)
        .execute(&mut *tx)
        .await?;
        if inserted_model.rows_affected() != 1 {
            return Err(PersistenceError::InvalidState(
                "候选模型无法继承基线模型版本".to_string(),
            ));
        }
        sqlx::query(
            r#"
            INSERT INTO model.parameter_sets (
                id, model_version_id, parameter_version, name,
                definition, definition_sha256, status
            ) VALUES ($1,$2,$3,$4,$5,$6,'draft')
            "#,
        )
        .bind(artifact.candidate_parameter_set_id)
        .bind(artifact.candidate_model_version_id)
        .bind(&artifact.candidate_parameter_version)
        .bind(format!(
            "{} / {}",
            artifact.baseline.competition_name, candidate.target_module
        ))
        .bind(&artifact.candidate_parameters)
        .bind(&artifact.candidate_definition_sha256)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO analytics.parameter_tuning_candidates (
                id, competition_id, competition_profile_id, partition_key,
                model_key, model_version, parameter_version, snapshot_type,
                target_module, sample_size,
                baseline_model_version_id, baseline_parameter_set_id,
                candidate_model_version_id, candidate_parameter_set_id,
                candidate_model_version, candidate_parameter_version, candidate_definition_sha256,
                baseline_metrics, calibration_bias, proposed_adjustments, constraints,
                training_window, validation_window, holdout_window,
                rationale, status
            ) VALUES (
                $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,
                $14,$15,$16,$17,$18,$19,$20,$21,$22,$23,$24,$25,$26
            )
            "#,
        )
        .bind(candidate.id)
        .bind(candidate.competition_id)
        .bind(candidate.competition_profile_id)
        .bind(candidate.partition_key.as_deref())
        .bind(&candidate.model_key)
        .bind(&candidate.model_version)
        .bind(&candidate.parameter_version)
        .bind(&candidate.snapshot_type)
        .bind(&candidate.target_module)
        .bind(candidate.sample_size as i64)
        .bind(candidate.baseline_model_version_id)
        .bind(candidate.baseline_parameter_set_id)
        .bind(candidate.candidate_model_version_id)
        .bind(candidate.candidate_parameter_set_id)
        .bind(candidate.candidate_model_version.as_deref())
        .bind(candidate.candidate_parameter_version.as_deref())
        .bind(candidate.candidate_definition_sha256.as_deref())
        .bind(&candidate.baseline_metrics)
        .bind(&candidate.calibration_bias)
        .bind(&candidate.proposed_adjustments)
        .bind(&candidate.constraints)
        .bind(&candidate.training_window)
        .bind(&candidate.validation_window)
        .bind(&candidate.holdout_window)
        .bind(&candidate.rationale)
        .bind(&candidate.status)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "parameter_candidate_created",
            "parameter_tuning_candidate",
            Some(candidate.id.to_string()),
            json!({
                "partition_key": candidate.partition_key,
                "baseline_model_version_id": candidate.baseline_model_version_id,
                "baseline_parameter_set_id": candidate.baseline_parameter_set_id,
                "candidate_model_version_id": candidate.candidate_model_version_id,
                "candidate_parameter_set_id": candidate.candidate_parameter_set_id,
                "automatic_promotion": false,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_parameter_tuning_candidate(candidate.id).await
    }

    pub async fn load_parameter_replay_fixtures(
        &self,
        competition_id: Uuid,
        competition_profile_id: Uuid,
        snapshot_type: &str,
        baseline_model_version_id: Uuid,
        baseline_parameter_set_id: Uuid,
    ) -> PersistenceResult<Vec<ParameterReplayFixture>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (run.id)
                review.id AS review_id,
                run.id AS run_id,
                fixture.id AS match_id,
                fixture.external_key AS match_key,
                fixture.competition_id,
                fixture.season_id,
                fixture.stage_id,
                COALESCE(stage.stage_kind, competition.competition_kind, 'custom') AS competition_kind,
                package.competition_profile_id,
                fixture.kickoff_time,
                home.canonical_name AS home_team_name,
                away.canonical_name AS away_team_name,
                run.snapshot_type,
                run.input_payload,
                package.version AS rule_package_version,
                result.home_goals_90 AS actual_home_goals,
                result.away_goals_90 AS actual_away_goals,
                (run.summary->>'home_win')::double precision AS baseline_home_win,
                (run.summary->>'draw')::double precision AS baseline_draw,
                (run.summary->>'away_win')::double precision AS baseline_away_win,
                scoreline.probability AS baseline_scoreline_probability,
                review.data_coverage::double precision AS data_coverage
            FROM review.postmatch_settlements settlement
            JOIN review.match_reviews review ON review.id = settlement.match_review_id
            JOIN football.match_results result ON result.match_id = settlement.match_id
            JOIN football.matches fixture ON fixture.id = settlement.match_id
            JOIN football.competitions competition ON competition.id = fixture.competition_id
            LEFT JOIN football.competition_stages stage ON stage.id = fixture.stage_id
            LEFT JOIN football.teams home ON home.id = fixture.home_team_id
            LEFT JOIN football.teams away ON away.id = fixture.away_team_id
            JOIN model.runs run ON run.id = settlement.model_run_id AND run.status = 'succeeded'
            JOIN model.rule_packages package ON package.id = settlement.rule_package_id
            LEFT JOIN model.run_scorelines scoreline
              ON scoreline.run_id = run.id
             AND scoreline.home_goals = result.home_goals_90
             AND scoreline.away_goals = result.away_goals_90
            WHERE review.status = 'finalized'
              AND COALESCE((review.prediction_evaluation->>'available')::boolean, false) = true
              AND settlement.competition_id = $1
              AND settlement.competition_profile_id = $2
              AND settlement.horizon = $3
              AND settlement.model_version_id = $4
              AND settlement.parameter_set_id = $5
              AND package.competition_profile_id = settlement.competition_profile_id
              AND run.snapshot_type = settlement.horizon
              AND run.model_version_id = settlement.model_version_id
              AND run.parameter_set_id = settlement.parameter_set_id
            ORDER BY run.id, review.created_at DESC, review.id DESC
            "#,
        )
        .bind(competition_id)
        .bind(competition_profile_id)
        .bind(snapshot_type)
        .bind(baseline_model_version_id)
        .bind(baseline_parameter_set_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(replay_fixture_from_row).collect()
    }

    pub async fn save_parameter_shadow_validation(
        &self,
        record: &ParameterShadowValidationRecord,
    ) -> PersistenceResult<ParameterShadowValidationRecord> {
        let candidate_status = match record.status.as_str() {
            "passed" => "shadow_passed",
            "failed" => "shadow_failed",
            _ => "blocked_by_h",
        };
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO analytics.parameter_shadow_validations (
                id, candidate_id, validation_key, partition_key, sample_count,
                baseline_metrics, candidate_metrics, metric_deltas, gate_results,
                status, generated_at
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            ON CONFLICT (candidate_id, validation_key) DO NOTHING
            "#,
        )
        .bind(record.id)
        .bind(record.candidate_id)
        .bind(&record.validation_key)
        .bind(&record.partition_key)
        .bind(record.sample_count as i64)
        .bind(&record.baseline_metrics)
        .bind(&record.candidate_metrics)
        .bind(&record.metric_deltas)
        .bind(&record.gate_results)
        .bind(&record.status)
        .bind(record.generated_at)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            UPDATE analytics.parameter_tuning_candidates
            SET status = $2,
                decided_at = now(),
                decision_note = CASE
                    WHEN $2 = 'shadow_passed' THEN '影子验证全部门禁通过，等待人工晋升'
                    WHEN $2 = 'shadow_failed' THEN '影子验证未通过，正式绑定保持不变'
                    ELSE '接入点 H 前置门禁未通过'
                END
            WHERE id = $1
              AND status IN ('accepted_for_backtest', 'shadow_running', 'blocked_by_h', 'shadow_failed')
            "#,
        )
        .bind(record.candidate_id)
        .bind(candidate_status)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "parameter_shadow_validation_recorded",
            "parameter_tuning_candidate",
            Some(record.candidate_id.to_string()),
            json!({
                "validation_id": record.id,
                "validation_key": record.validation_key,
                "status": record.status,
                "sample_count": record.sample_count,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_parameter_shadow_validation(record.candidate_id, &record.validation_key)
            .await
    }

    pub async fn list_parameter_shadow_validations(
        &self,
        candidate_id: Uuid,
    ) -> PersistenceResult<Vec<ParameterShadowValidationRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, candidate_id, validation_key, partition_key, sample_count,
                   baseline_metrics, candidate_metrics, metric_deltas, gate_results,
                   status, generated_at
            FROM analytics.parameter_shadow_validations
            WHERE candidate_id = $1
            ORDER BY generated_at DESC, id DESC
            "#,
        )
        .bind(candidate_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(shadow_validation_from_row).collect()
    }

    async fn read_parameter_shadow_validation(
        &self,
        candidate_id: Uuid,
        validation_key: &str,
    ) -> PersistenceResult<ParameterShadowValidationRecord> {
        let row = sqlx::query(
            r#"
            SELECT id, candidate_id, validation_key, partition_key, sample_count,
                   baseline_metrics, candidate_metrics, metric_deltas, gate_results,
                   status, generated_at
            FROM analytics.parameter_shadow_validations
            WHERE candidate_id = $1 AND validation_key = $2
            "#,
        )
        .bind(candidate_id)
        .bind(validation_key)
        .fetch_one(&self.pool)
        .await?;
        shadow_validation_from_row(&row)
    }

    pub async fn promote_parameter_candidate(
        &self,
        request: &ParameterPromotionRequest,
    ) -> PersistenceResult<ParameterPromotionDecisionRecord> {
        let candidate = self
            .read_parameter_tuning_candidate(request.candidate_id)
            .await?;
        if candidate.status != "shadow_passed" {
            return Err(PersistenceError::InvalidState(
                "只有影子验证通过的候选可以人工晋升".to_string(),
            ));
        }
        let competition_id = required_uuid(candidate.competition_id, "候选缺少赛事范围")?;
        let competition_profile_id =
            required_uuid(candidate.competition_profile_id, "候选缺少赛事 Profile")?;
        let baseline_model_version_id =
            required_uuid(candidate.baseline_model_version_id, "候选缺少基线模型版本")?;
        let baseline_parameter_set_id =
            required_uuid(candidate.baseline_parameter_set_id, "候选缺少基线参数版本")?;
        let candidate_model_version_id =
            required_uuid(candidate.candidate_model_version_id, "候选缺少候选模型版本")?;
        let candidate_parameter_set_id =
            required_uuid(candidate.candidate_parameter_set_id, "候选缺少候选参数版本")?;
        let mut tx = self.pool.begin().await?;
        let bindings = sqlx::query(
            r#"
            SELECT binding.id, binding.model_version_id, binding.parameter_set_id
            FROM model.competition_bindings binding
            JOIN model.rule_packages package ON package.id = binding.rule_package_id
            WHERE binding.is_active = true
              AND binding.competition_id = $1
              AND binding.model_version_id = $2
              AND binding.parameter_set_id = $3
              AND package.competition_profile_id = $4
            FOR UPDATE OF binding
            "#,
        )
        .bind(competition_id)
        .bind(baseline_model_version_id)
        .bind(baseline_parameter_set_id)
        .bind(competition_profile_id)
        .fetch_all(&mut *tx)
        .await?;
        if bindings.is_empty() {
            return Err(PersistenceError::InvalidState(
                "活动绑定已经变化，拒绝覆盖；请重新生成候选".to_string(),
            ));
        }
        let decision_id = Uuid::new_v4();
        let previous_state = Value::Array(
            bindings
                .iter()
                .map(|row| {
                    json!({
                        "binding_id": row.try_get::<Uuid, _>("id").ok(),
                        "model_version_id": row.try_get::<Uuid, _>("model_version_id").ok(),
                        "parameter_set_id": row.try_get::<Uuid, _>("parameter_set_id").ok(),
                    })
                })
                .collect(),
        );
        let new_state = Value::Array(
            bindings
                .iter()
                .map(|row| {
                    json!({
                        "binding_id": row.try_get::<Uuid, _>("id").ok(),
                        "model_version_id": candidate_model_version_id,
                        "parameter_set_id": candidate_parameter_set_id,
                    })
                })
                .collect(),
        );
        sqlx::query(
            r#"
            INSERT INTO analytics.parameter_promotion_decisions (
                id, candidate_id, decision, previous_binding_state, new_binding_state,
                decided_by, decision_note
            ) VALUES ($1,$2,'promote',$3,$4,$5,$6)
            "#,
        )
        .bind(decision_id)
        .bind(candidate.id)
        .bind(&previous_state)
        .bind(&new_state)
        .bind(request.decided_by.as_deref())
        .bind(&request.decision_note)
        .execute(&mut *tx)
        .await?;
        for row in &bindings {
            let binding_id: Uuid = row.try_get("id")?;
            sqlx::query(
                r#"
                INSERT INTO analytics.parameter_binding_changes (
                    decision_id, binding_id,
                    previous_model_version_id, previous_parameter_set_id,
                    new_model_version_id, new_parameter_set_id
                ) VALUES ($1,$2,$3,$4,$5,$6)
                "#,
            )
            .bind(decision_id)
            .bind(binding_id)
            .bind(baseline_model_version_id)
            .bind(baseline_parameter_set_id)
            .bind(candidate_model_version_id)
            .bind(candidate_parameter_set_id)
            .execute(&mut *tx)
            .await?;
            let updated = sqlx::query(
                r#"
                UPDATE model.competition_bindings
                SET model_version_id = $2, parameter_set_id = $3
                WHERE id = $1
                  AND model_version_id = $4
                  AND parameter_set_id = $5
                "#,
            )
            .bind(binding_id)
            .bind(candidate_model_version_id)
            .bind(candidate_parameter_set_id)
            .bind(baseline_model_version_id)
            .bind(baseline_parameter_set_id)
            .execute(&mut *tx)
            .await?;
            if updated.rows_affected() != 1 {
                return Err(PersistenceError::InvalidState(format!(
                    "绑定 {binding_id} 已被其他版本修改，拒绝覆盖式晋升"
                )));
            }
        }
        sqlx::query("UPDATE model.versions SET status='active' WHERE id=$1")
            .bind(candidate_model_version_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE model.parameter_sets SET status='active' WHERE id=$1")
            .bind(candidate_parameter_set_id)
            .execute(&mut *tx)
            .await?;
        let promoted = sqlx::query(
            "UPDATE analytics.parameter_tuning_candidates SET status='promoted', decided_at=now(), decision_note=$2 WHERE id=$1 AND status='shadow_passed'",
        )
        .bind(candidate.id)
        .bind(&request.decision_note)
        .execute(&mut *tx)
        .await?;
        if promoted.rows_affected() != 1 {
            return Err(PersistenceError::InvalidState(
                "候选状态已变化，拒绝提交晋升决策".to_string(),
            ));
        }
        write_audit_event(
            &mut tx,
            "parameter_candidate_promoted",
            "parameter_tuning_candidate",
            Some(candidate.id.to_string()),
            json!({
                "decision_id": decision_id,
                "binding_count": bindings.len(),
                "manual": true,
                "automatic_promotion": false,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_parameter_promotion_decision(decision_id).await
    }

    pub async fn rollback_parameter_candidate(
        &self,
        request: &ParameterRollbackRequest,
    ) -> PersistenceResult<ParameterPromotionDecisionRecord> {
        let candidate = self
            .read_parameter_tuning_candidate(request.candidate_id)
            .await?;
        if candidate.status != "promoted" {
            return Err(PersistenceError::InvalidState(
                "只有已晋升且未回滚的候选可以回滚".to_string(),
            ));
        }
        let candidate_model_version_id =
            required_uuid(candidate.candidate_model_version_id, "候选缺少候选模型版本")?;
        let candidate_parameter_set_id =
            required_uuid(candidate.candidate_parameter_set_id, "候选缺少候选参数版本")?;
        let promotion = sqlx::query(
            r#"
            SELECT id
            FROM analytics.parameter_promotion_decisions
            WHERE candidate_id = $1 AND decision = 'promote'
            ORDER BY created_at DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(candidate.id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("未找到该候选的晋升决策记录".to_string()))?;
        let promotion_id: Uuid = promotion.try_get("id")?;
        let mut tx = self.pool.begin().await?;
        let changes = sqlx::query(
            r#"
            SELECT binding_id, previous_model_version_id, previous_parameter_set_id,
                   new_model_version_id, new_parameter_set_id
            FROM analytics.parameter_binding_changes
            WHERE decision_id = $1
            ORDER BY binding_id
            "#,
        )
        .bind(promotion_id)
        .fetch_all(&mut *tx)
        .await?;
        if changes.is_empty() {
            return Err(PersistenceError::InvalidState(
                "晋升决策没有可回滚的绑定快照".to_string(),
            ));
        }
        let rollback_id = Uuid::new_v4();
        let previous_state = Value::Array(
            changes
                .iter()
                .map(|row| {
                    json!({
                        "binding_id": row.try_get::<Uuid, _>("binding_id").ok(),
                        "model_version_id": candidate_model_version_id,
                        "parameter_set_id": candidate_parameter_set_id,
                    })
                })
                .collect(),
        );
        let new_state = Value::Array(
            changes
                .iter()
                .map(|row| {
                    json!({
                        "binding_id": row.try_get::<Uuid, _>("binding_id").ok(),
                        "model_version_id": row.try_get::<Uuid, _>("previous_model_version_id").ok(),
                        "parameter_set_id": row.try_get::<Uuid, _>("previous_parameter_set_id").ok(),
                    })
                })
                .collect(),
        );
        sqlx::query(
            r#"
            INSERT INTO analytics.parameter_promotion_decisions (
                id, candidate_id, decision, previous_binding_state, new_binding_state,
                decided_by, decision_note
            ) VALUES ($1,$2,'rollback',$3,$4,$5,$6)
            "#,
        )
        .bind(rollback_id)
        .bind(candidate.id)
        .bind(&previous_state)
        .bind(&new_state)
        .bind(request.decided_by.as_deref())
        .bind(&request.decision_note)
        .execute(&mut *tx)
        .await?;
        for row in &changes {
            let binding_id: Uuid = row.try_get("binding_id")?;
            let previous_model_version_id: Uuid = row.try_get("previous_model_version_id")?;
            let previous_parameter_set_id: Uuid = row.try_get("previous_parameter_set_id")?;
            let updated = sqlx::query(
                r#"
                UPDATE model.competition_bindings
                SET model_version_id = $2, parameter_set_id = $3
                WHERE id = $1
                  AND model_version_id = $4
                  AND parameter_set_id = $5
                "#,
            )
            .bind(binding_id)
            .bind(previous_model_version_id)
            .bind(previous_parameter_set_id)
            .bind(candidate_model_version_id)
            .bind(candidate_parameter_set_id)
            .execute(&mut *tx)
            .await?;
            if updated.rows_affected() != 1 {
                return Err(PersistenceError::InvalidState(format!(
                    "绑定 {binding_id} 已被其他版本修改，拒绝覆盖式回滚"
                )));
            }
        }
        sqlx::query("UPDATE model.versions SET status='deprecated' WHERE id=$1")
            .bind(candidate_model_version_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE model.parameter_sets SET status='deprecated' WHERE id=$1")
            .bind(candidate_parameter_set_id)
            .execute(&mut *tx)
            .await?;
        let rolled_back = sqlx::query(
            "UPDATE analytics.parameter_tuning_candidates SET status='rolled_back', decided_at=now(), decision_note=$2 WHERE id=$1 AND status='promoted'",
        )
        .bind(candidate.id)
        .bind(&request.decision_note)
        .execute(&mut *tx)
        .await?;
        if rolled_back.rows_affected() != 1 {
            return Err(PersistenceError::InvalidState(
                "候选状态已变化，拒绝提交回滚决策".to_string(),
            ));
        }
        write_audit_event(
            &mut tx,
            "parameter_candidate_rolled_back",
            "parameter_tuning_candidate",
            Some(candidate.id.to_string()),
            json!({
                "promotion_decision_id": promotion_id,
                "rollback_decision_id": rollback_id,
                "binding_count": changes.len(),
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_parameter_promotion_decision(rollback_id).await
    }

    pub async fn list_parameter_promotion_decisions(
        &self,
        candidate_id: Uuid,
    ) -> PersistenceResult<Vec<ParameterPromotionDecisionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, candidate_id, decision, previous_binding_state, new_binding_state,
                   decided_by, decision_note, created_at
            FROM analytics.parameter_promotion_decisions
            WHERE candidate_id = $1
            ORDER BY created_at DESC, id DESC
            "#,
        )
        .bind(candidate_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(promotion_decision_from_row).collect()
    }

    async fn read_parameter_promotion_decision(
        &self,
        id: Uuid,
    ) -> PersistenceResult<ParameterPromotionDecisionRecord> {
        let row = sqlx::query(
            r#"
            SELECT id, candidate_id, decision, previous_binding_state, new_binding_state,
                   decided_by, decision_note, created_at
            FROM analytics.parameter_promotion_decisions
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        promotion_decision_from_row(&row)
    }
}

fn replay_fixture_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ParameterReplayFixture> {
    Ok(ParameterReplayFixture {
        review_id: row.try_get("review_id")?,
        run_id: row.try_get("run_id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        competition_id: row.try_get("competition_id")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        competition_kind: row.try_get("competition_kind")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        kickoff_time: row.try_get("kickoff_time")?,
        home_team_name: row
            .try_get::<Option<String>, _>("home_team_name")?
            .unwrap_or_else(|| "主队".to_string()),
        away_team_name: row
            .try_get::<Option<String>, _>("away_team_name")?
            .unwrap_or_else(|| "客队".to_string()),
        snapshot_type: row.try_get("snapshot_type")?,
        input_payload: row.try_get("input_payload")?,
        rule_package_version: row.try_get("rule_package_version")?,
        actual_home_goals: row.try_get("actual_home_goals")?,
        actual_away_goals: row.try_get("actual_away_goals")?,
        baseline_home_win: row.try_get("baseline_home_win")?,
        baseline_draw: row.try_get("baseline_draw")?,
        baseline_away_win: row.try_get("baseline_away_win")?,
        baseline_scoreline_probability: row.try_get("baseline_scoreline_probability")?,
        data_coverage: row.try_get("data_coverage")?,
    })
}

fn shadow_validation_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ParameterShadowValidationRecord> {
    Ok(ParameterShadowValidationRecord {
        id: row.try_get("id")?,
        candidate_id: row.try_get("candidate_id")?,
        validation_key: row.try_get("validation_key")?,
        partition_key: row.try_get("partition_key")?,
        sample_count: non_negative_u64(row.try_get("sample_count")?, "影子验证样本数")?,
        baseline_metrics: row.try_get("baseline_metrics")?,
        candidate_metrics: row.try_get("candidate_metrics")?,
        metric_deltas: row.try_get("metric_deltas")?,
        gate_results: row.try_get("gate_results")?,
        status: row.try_get("status")?,
        generated_at: row.try_get("generated_at")?,
    })
}

fn promotion_decision_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ParameterPromotionDecisionRecord> {
    Ok(ParameterPromotionDecisionRecord {
        id: row.try_get("id")?,
        candidate_id: row.try_get("candidate_id")?,
        decision: row.try_get("decision")?,
        previous_binding_state: row.try_get("previous_binding_state")?,
        new_binding_state: row.try_get("new_binding_state")?,
        decided_by: row.try_get("decided_by")?,
        decision_note: row.try_get("decision_note")?,
        created_at: row.try_get("created_at")?,
    })
}

fn required_uuid(value: Option<Uuid>, message: &str) -> PersistenceResult<Uuid> {
    value.ok_or_else(|| PersistenceError::InvalidState(message.to_string()))
}

fn non_negative_u64(value: i64, label: &str) -> PersistenceResult<u64> {
    u64::try_from(value).map_err(|_| PersistenceError::InvalidState(format!("{label}不能为负数")))
}
