use super::{
    key::FormationUsageGroupKey, probability::calculate_probabilities,
    validation::validate_distribution_draft,
};
use crate::{
    adapters::catalog::formations::constants::UNKNOWN_FORMATION_ID, write_audit_event,
    PersistenceError, PersistenceResult, PostgresStore,
};
use chrono::Utc;
use football_domain::{FormationUsageDistributionDraft, FormationUsageDistributionRecord};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

impl PostgresStore {
    pub async fn save_formation_usage_distribution(
        &self,
        draft: &FormationUsageDistributionDraft,
    ) -> PersistenceResult<FormationUsageDistributionRecord> {
        validate_distribution_draft(draft)?;
        let (window_start, window_end) = self.resolve_formation_window(draft).await?;

        let mut counts: HashMap<Uuid, i32> = HashMap::new();
        for entry in &draft.entries {
            if entry.usage_count < 0 || entry.usage_count > draft.observed_matches {
                return Err(PersistenceError::InvalidState(
                    "阵型使用次数必须位于 0 到观察场数之间".to_string(),
                ));
            }
            if counts
                .insert(entry.formation_id, entry.usage_count)
                .is_some()
            {
                return Err(PersistenceError::InvalidState(
                    "同一阵型在一个观察窗口中不能重复".to_string(),
                ));
            }
        }

        let formation_ids: Vec<Uuid> = counts.keys().copied().collect();
        if !formation_ids.is_empty() {
            let valid_count: i64 = sqlx::query_scalar(
                "SELECT count(*)::bigint FROM football.formations WHERE id = ANY($1) AND is_active",
            )
            .bind(&formation_ids)
            .fetch_one(&self.pool)
            .await?;
            if valid_count != formation_ids.len() as i64 {
                return Err(PersistenceError::InvalidState(
                    "阵型目录中存在无效或已停用的阵型".to_string(),
                ));
            }
        }

        let total: i32 = counts.values().sum();
        if total > draft.observed_matches {
            return Err(PersistenceError::InvalidState(
                "阵型使用次数合计不能超过观察场数".to_string(),
            ));
        }
        let missing = draft.observed_matches - total;
        if draft.observed_matches == 0 {
            counts.clear();
            counts.insert(UNKNOWN_FORMATION_ID, 0);
        } else if missing > 0 {
            *counts.entry(UNKNOWN_FORMATION_ID).or_insert(0) += missing;
        }
        counts.retain(|formation_id, count| *count > 0 || *formation_id == UNKNOWN_FORMATION_ID);
        let probabilities = calculate_probabilities(&counts, draft.observed_matches, draft.alpha)?;
        let observed_at = Utc::now();
        let mut tx = self.pool.begin().await?;

        for (formation_id, usage_count) in counts {
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

        self.read_exact_distribution(&FormationUsageGroupKey {
            scope_type: draft.scope_type.trim(),
            team_id: draft.team_id,
            coach_id: draft.coach_id,
            competition_id: draft.competition_id,
            window_start,
            window_end,
            observed_at,
        })
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("阵型概率保存后无法读取".to_string()))
    }
}
