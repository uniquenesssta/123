use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{ParameterCandidateArtifactDraft, ParameterTuningCandidateRecord};
use serde_json::{json, Value};
use uuid::Uuid;

impl PostgresStore {
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
}
