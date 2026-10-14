use super::policy::non_negative_u64;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    ParameterCandidateBaseline, ParameterLifecycleReadiness, ParameterLifecycleReadinessRequest,
};
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
}
