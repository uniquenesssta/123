use super::mapping::promotion_decision_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::ParameterPromotionDecisionRecord;
use uuid::Uuid;

impl PostgresStore {
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

    pub(super) async fn read_parameter_promotion_decision(
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
