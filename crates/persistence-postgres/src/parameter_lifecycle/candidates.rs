use super::{mapping::parameter_tuning_candidate_from_row, policy::tuning_decision_status};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{ParameterTuningCandidateRecord, ParameterTuningDecisionDraft};
use uuid::Uuid;

impl PostgresStore {
    pub async fn save_parameter_tuning_candidate(
        &self,
        candidate: &ParameterTuningCandidateRecord,
    ) -> PersistenceResult<ParameterTuningCandidateRecord> {
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
        .execute(&self.pool)
        .await?;
        self.read_parameter_tuning_candidate(candidate.id).await
    }

    pub async fn list_parameter_tuning_candidates(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<ParameterTuningCandidateRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT candidate.id, candidate.competition_id, competition.name AS competition_name,
                   candidate.competition_profile_id, candidate.partition_key,
                   candidate.model_key, candidate.model_version, candidate.parameter_version,
                   candidate.snapshot_type, candidate.target_module, candidate.sample_size,
                   candidate.baseline_model_version_id, candidate.baseline_parameter_set_id,
                   candidate.candidate_model_version_id, candidate.candidate_parameter_set_id,
                   candidate.candidate_model_version, candidate.candidate_parameter_version,
                   candidate.candidate_definition_sha256,
                   candidate.baseline_metrics, candidate.calibration_bias,
                   candidate.proposed_adjustments, candidate.constraints,
                   candidate.training_window, candidate.validation_window, candidate.holdout_window,
                   candidate.rationale, candidate.status, candidate.created_at,
                   candidate.decided_at, candidate.decision_note
            FROM analytics.parameter_tuning_candidates candidate
            LEFT JOIN football.competitions competition ON competition.id = candidate.competition_id
            ORDER BY candidate.created_at DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(parameter_tuning_candidate_from_row)
            .collect()
    }

    pub async fn decide_parameter_tuning_candidate(
        &self,
        draft: &ParameterTuningDecisionDraft,
    ) -> PersistenceResult<ParameterTuningCandidateRecord> {
        let status = tuning_decision_status(draft.decision);
        let updated = sqlx::query_scalar::<_, Uuid>(
            r#"
            UPDATE analytics.parameter_tuning_candidates
            SET status=$2, decided_at=now(), decision_note=$3
            WHERE id=$1 AND status='pending'
            RETURNING id
            "#,
        )
        .bind(draft.candidate_id)
        .bind(status)
        .bind(draft.decision_note.as_deref())
        .fetch_optional(&self.pool)
        .await?;
        if updated.is_none() {
            return Err(PersistenceError::InvalidState(
                "参数候选不存在或已经完成审核".to_string(),
            ));
        }
        self.read_parameter_tuning_candidate(draft.candidate_id)
            .await
    }

    pub async fn read_parameter_tuning_candidate(
        &self,
        id: Uuid,
    ) -> PersistenceResult<ParameterTuningCandidateRecord> {
        let row = sqlx::query(
            r#"
            SELECT candidate.id, candidate.competition_id, competition.name AS competition_name,
                   candidate.competition_profile_id, candidate.partition_key,
                   candidate.model_key, candidate.model_version, candidate.parameter_version,
                   candidate.snapshot_type, candidate.target_module, candidate.sample_size,
                   candidate.baseline_model_version_id, candidate.baseline_parameter_set_id,
                   candidate.candidate_model_version_id, candidate.candidate_parameter_set_id,
                   candidate.candidate_model_version, candidate.candidate_parameter_version,
                   candidate.candidate_definition_sha256,
                   candidate.baseline_metrics, candidate.calibration_bias,
                   candidate.proposed_adjustments, candidate.constraints,
                   candidate.training_window, candidate.validation_window, candidate.holdout_window,
                   candidate.rationale, candidate.status, candidate.created_at,
                   candidate.decided_at, candidate.decision_note
            FROM analytics.parameter_tuning_candidates candidate
            LEFT JOIN football.competitions competition ON competition.id = candidate.competition_id
            WHERE candidate.id=$1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("参数候选不存在".to_string()))?;
        parameter_tuning_candidate_from_row(&row)
    }
}
