use crate::{
    adapters::catalog::coaches::{
        existence::{ensure_coach_exists, ensure_team_exists},
        validation::validate_team_coach_period,
    },
    write_audit_event, PersistenceResult, PostgresStore,
};
use football_domain::{TeamCoachPeriodDraft, TeamCoachPeriodRecord};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;
impl PostgresStore {
    pub async fn add_team_coach_period(
        &self,
        draft: &TeamCoachPeriodDraft,
    ) -> PersistenceResult<TeamCoachPeriodRecord> {
        validate_team_coach_period(draft)?;
        let mut tx = self.pool.begin().await?;
        ensure_team_exists(&mut tx, draft.team_id).await?;
        ensure_coach_exists(&mut tx, draft.coach_id).await?;
        if draft.end_previous {
            sqlx::query(
                r#"
                    UPDATE football.team_coach_periods
                    SET valid_to = $3 - 1
                    WHERE team_id=$1 AND role=$2 AND valid_to IS NULL
                      AND valid_from < $3 AND coach_id <> $4
                    "#,
            )
            .bind(draft.team_id)
            .bind(draft.role.trim())
            .bind(draft.valid_from)
            .bind(draft.coach_id)
            .execute(&mut *tx)
            .await?;
        }
        let row = sqlx::query(
            r#"
                INSERT INTO football.team_coach_periods (
                    id, team_id, coach_id, role, valid_from, valid_to, is_interim,
                    source_document_id, confidence, metadata
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)
                ON CONFLICT (team_id, coach_id, role, valid_from)
                DO UPDATE SET valid_to=EXCLUDED.valid_to, is_interim=EXCLUDED.is_interim,
                              source_document_id=EXCLUDED.source_document_id,
                              confidence=EXCLUDED.confidence,
                              metadata=football.team_coach_periods.metadata || EXCLUDED.metadata
                RETURNING id
                "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.team_id)
        .bind(draft.coach_id)
        .bind(draft.role.trim())
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .bind(draft.is_interim)
        .bind(draft.source_document_id)
        .bind(draft.confidence)
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        let id: Uuid = row.try_get("id")?;
        write_audit_event(
            &mut tx,
            "team_coach_period_upserted",
            "team",
            Some(draft.team_id.to_string()),
            json!({"coach_id": draft.coach_id, "role": draft.role, "valid_from": draft.valid_from}),
        )
        .await?;
        tx.commit().await?;
        self.read_team_coach_period(id).await
    }
}
