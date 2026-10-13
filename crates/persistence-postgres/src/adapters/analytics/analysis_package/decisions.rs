use super::policy::{
    ability_values, ensure_pending, json_string, json_uuid, AbilitySuggestionValues,
};
use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    AiAnalysisSuggestionRecord, AiSuggestionDecision, AiSuggestionDecisionDraft,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn decide_ai_suggestion(
        &self,
        draft: &AiSuggestionDecisionDraft,
    ) -> PersistenceResult<AiAnalysisSuggestionRecord> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("SELECT suggestion_type,payload,status FROM analytics.ai_suggestions WHERE id=$1 FOR UPDATE")
            .bind(draft.suggestion_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("AI 建议不存在".to_string()))?;
        let status: String = row.try_get("status")?;
        ensure_pending(&status)?;
        let suggestion_type: String = row.try_get("suggestion_type")?;
        let payload: Value = row.try_get("payload")?;
        let decided_by = draft.decided_by.as_deref().unwrap_or("desktop-user");
        let mut linked_candidate_id = None;
        let new_status = match draft.decision {
            AiSuggestionDecision::Reject => "rejected",
            AiSuggestionDecision::Accept if suggestion_type == "ability_update" => {
                let player_id = json_uuid(&payload, "player_id")?;
                let dimension_code = json_string(&payload, "dimension_code")?;
                let identity_valid: bool = sqlx::query_scalar(
                    r#"
                    SELECT EXISTS(
                        SELECT 1
                        FROM football.players player
                        CROSS JOIN feature.player_ability_dimensions dimension
                        WHERE player.id=$1 AND dimension.code=$2
                    )
                    "#,
                )
                .bind(player_id)
                .bind(&dimension_code)
                .fetch_one(&mut *tx)
                .await?;
                if !identity_valid {
                    return Err(PersistenceError::InvalidState(
                        "AI 能力建议引用了不存在的球员或能力维度".to_string(),
                    ));
                }
                let AbilitySuggestionValues {
                    proposed_value,
                    confidence,
                    sample_size,
                } = ability_values(&payload)?;
                let current_value: Option<f64> = sqlx::query_scalar(
                    "SELECT value FROM feature.player_current_abilities WHERE player_id=$1 AND dimension_code=$2",
                )
                .bind(player_id)
                .bind(&dimension_code)
                .fetch_optional(&mut *tx)
                .await?;
                let candidate_id = Uuid::new_v4();
                sqlx::query(
                    r#"
                    INSERT INTO review.ability_update_candidates (
                        id,player_id,dimension_code,current_value,proposed_value,confidence,sample_size,
                        evidence,calculation_version,status
                    ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,'ai-analysis-response-v1','pending')
                    "#,
                )
                .bind(candidate_id)
                .bind(player_id)
                .bind(&dimension_code)
                .bind(current_value)
                .bind(proposed_value)
                .bind(confidence)
                .bind(sample_size)
                .bind(json!({
                    "source": "ai_analysis_response",
                    "suggestion_id": draft.suggestion_id,
                    "accepted_by": decided_by,
                    "payload": payload
                }))
                .execute(&mut *tx)
                .await?;
                linked_candidate_id = Some(candidate_id);
                "accepted"
            }
            AiSuggestionDecision::Accept => "accepted",
        };
        sqlx::query("UPDATE analytics.ai_suggestions SET status=$2,linked_candidate_id=$3,decided_at=now(),decided_by=$4,decision_note=$5 WHERE id=$1")
            .bind(draft.suggestion_id)
            .bind(new_status)
            .bind(linked_candidate_id)
            .bind(decided_by)
            .bind(&draft.decision_note)
            .execute(&mut *tx)
            .await?;
        write_audit_event(
            &mut tx,
            "ai_suggestion_decided",
            "ai_suggestion",
            Some(draft.suggestion_id.to_string()),
            json!({"decision": new_status, "linked_candidate_id": linked_candidate_id}),
        )
        .await?;
        tx.commit().await?;
        self.read_ai_suggestion(draft.suggestion_id).await
    }
}
