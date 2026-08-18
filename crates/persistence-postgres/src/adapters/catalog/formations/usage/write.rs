use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, NaiveDate, Utc};
use football_domain::FormationUsageDistributionDraft;
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

impl PostgresStore {
    pub(crate) async fn ensure_active_formations(
        &self,
        formation_ids: &[Uuid],
    ) -> PersistenceResult<()> {
        if formation_ids.is_empty() {
            return Ok(());
        }
        let valid_count: i64 = sqlx::query_scalar(
            "SELECT count(*)::bigint FROM football.formations WHERE id = ANY($1) AND is_active",
        )
        .bind(formation_ids)
        .fetch_one(&self.pool)
        .await?;
        if valid_count != formation_ids.len() as i64 {
            return Err(PersistenceError::InvalidState(
                "阵型目录中存在无效或已停用的阵型".to_string(),
            ));
        }
        Ok(())
    }

    pub(crate) async fn write_formation_usage_observations(
        &self,
        draft: &FormationUsageDistributionDraft,
        window_start: NaiveDate,
        window_end: NaiveDate,
        observed_at: DateTime<Utc>,
        counts: &HashMap<Uuid, i32>,
        probabilities: &HashMap<Uuid, (f64, f64)>,
    ) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        for (&formation_id, &usage_count) in counts {
            let (raw_probability, smoothed_probability) =
                probabilities.get(&formation_id).copied().ok_or_else(|| {
                    PersistenceError::InvalidState("阵型概率计算结果缺失".to_string())
                })?;
            sqlx::query(
                r#"
                INSERT INTO feature.formation_usage_observations (
                    id, scope_type, team_id, coach_id, competition_id, formation_id,
                    window_preset, window_start, window_end, observed_matches,
                    usage_count, raw_probability, smoothed_probability, confidence,
                    smoothing_alpha, source_document_id, observed_at, metadata
                ) VALUES (
                    $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18
                )
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(draft.scope_type.trim())
            .bind(draft.team_id)
            .bind(draft.coach_id)
            .bind(draft.competition_id)
            .bind(formation_id)
            .bind(draft.window_preset.trim())
            .bind(window_start)
            .bind(window_end)
            .bind(draft.observed_matches)
            .bind(usage_count)
            .bind(raw_probability)
            .bind(smoothed_probability)
            .bind(draft.confidence)
            .bind(draft.alpha)
            .bind(draft.source_document_id)
            .bind(observed_at)
            .bind(&draft.metadata)
            .execute(&mut *tx)
            .await?;
        }
        write_audit_event(
            &mut tx,
            "formation_usage_saved",
            "formation_usage",
            draft
                .team_id
                .or(draft.coach_id)
                .or(draft.competition_id)
                .map(|id| id.to_string()),
            json!({
                "scope_type": draft.scope_type,
                "team_id": draft.team_id,
                "coach_id": draft.coach_id,
                "competition_id": draft.competition_id,
                "window_start": window_start,
                "window_end": window_end,
                "observed_matches": draft.observed_matches,
                "alpha": draft.alpha,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
