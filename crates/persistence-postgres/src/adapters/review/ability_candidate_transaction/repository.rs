use super::{
    read::read_ability_candidate,
    validation::{decision_actor, require_current_projection, require_pending_candidate},
};
use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{
    AbilityCandidateDecision, AbilityCandidateDecisionDraft, AbilityUpdateCandidateRecord,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn decide_ability_candidate(
        &self,
        draft: &AbilityCandidateDecisionDraft,
    ) -> PersistenceResult<AbilityUpdateCandidateRecord> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            SELECT
                candidate.id, candidate.match_review_id, candidate.player_match_review_id,
                candidate.player_id, candidate.dimension_code,
                candidate.current_value, candidate.proposed_value,
                candidate.confidence, candidate.sample_size, candidate.evidence,
                candidate.calculation_version, candidate.status,
                review.match_id, fixture.kickoff_time
            FROM review.ability_update_candidates candidate
            LEFT JOIN review.match_reviews review ON review.id = candidate.match_review_id
            LEFT JOIN football.matches fixture ON fixture.id = review.match_id
            WHERE candidate.id = $1
            FOR UPDATE OF candidate
            "#,
        )
        .bind(draft.candidate_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("能力更新候选不存在".to_string()))?;
        let status: String = row.try_get("status")?;
        require_pending_candidate(&status)?;
        let decided_by = decision_actor(draft.decided_by.as_deref());
        let player_id: Uuid = row.try_get("player_id")?;
        let proposed_value: f64 = row.try_get("proposed_value")?;
        let current_value: Option<f64> = row.try_get("current_value")?;
        let calculation_version: String = row.try_get("calculation_version")?;
        let match_id: Option<Uuid> = row.try_get("match_id")?;
        let observed_at: Option<DateTime<Utc>> = row.try_get("kickoff_time")?;
        let evidence: Value = row.try_get("evidence")?;
        let confidence: f64 = row.try_get("confidence")?;
        let sample_size: i32 = row.try_get("sample_size")?;
        let dimension_code: String = row.try_get("dimension_code")?;

        let (decision_value, candidate_status, observation_id) = match draft.decision {
            AbilityCandidateDecision::Accept => {
                let current_projection: Option<f64> = sqlx::query_scalar(
                    r#"
                    SELECT value
                    FROM feature.player_current_abilities
                    WHERE player_id = $1 AND dimension_code = $2
                    "#,
                )
                .bind(player_id)
                .bind(&dimension_code)
                .fetch_optional(&mut *tx)
                .await?;
                require_current_projection(current_value, current_projection)?;
                let observation_id = Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO feature.player_ability_observations (
                        id, player_id, dimension_code, context_type, context_id,
                        value, confidence, sample_size, observed_at,
                        effective_from, effective_to, calculation_version,
                        source_document_id, metadata
                    ) VALUES (
                        $1, $2, $3, 'match_review', $4,
                        $5, $6, $7, $8,
                        now(), NULL, $9, NULL, $10
                    )
                    "#,
                )
                .bind(observation_id)
                .bind(player_id)
                .bind(&dimension_code)
                .bind(match_id)
                .bind(proposed_value)
                .bind(confidence)
                .bind(sample_size)
                .bind(observed_at.unwrap_or_else(Utc::now))
                .bind(format!("{calculation_version}:accepted"))
                .bind(json!({
                    "candidate_id": draft.candidate_id,
                    "evidence": evidence,
                    "decision_note": draft.decision_note,
                    "decided_by": decided_by,
                }))
                .execute(&mut *tx)
                .await?;
                ("accepted", "accepted", Some(observation_id))
            }
            AbilityCandidateDecision::Reject => ("rejected", "rejected", None),
        };

        sqlx::query(
            r#"
            UPDATE review.ability_update_candidates
            SET status = $2, decided_at = now(), decided_by = $3,
                decision_note = $4, accepted_observation_id = $5
            WHERE id = $1
            "#,
        )
        .bind(draft.candidate_id)
        .bind(candidate_status)
        .bind(decided_by)
        .bind(draft.decision_note.as_deref())
        .bind(observation_id)
        .execute(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO review.ability_update_decisions (
                id, candidate_id, decision, previous_value, proposed_value,
                applied_observation_id, decided_by, decision_note
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.candidate_id)
        .bind(decision_value)
        .bind(current_value)
        .bind(proposed_value)
        .bind(observation_id)
        .bind(decided_by)
        .bind(draft.decision_note.as_deref())
        .execute(&mut *tx)
        .await?;

        write_audit_event(
            &mut tx,
            "ability_update_candidate_decided",
            "ability_update_candidate",
            Some(draft.candidate_id.to_string()),
            json!({
                "decision": decision_value,
                "player_id": player_id,
                "dimension_code": dimension_code,
                "previous_value": current_value,
                "proposed_value": proposed_value,
                "observation_id": observation_id,
                "decided_by": decided_by,
            }),
        )
        .await?;
        tx.commit().await?;

        read_ability_candidate(&self.pool, draft.candidate_id).await
    }
}
