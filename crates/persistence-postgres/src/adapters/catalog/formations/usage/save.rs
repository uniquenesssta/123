use super::{
    key::FormationUsageGroupKey,
    preparation::{collect_usage_counts, complete_usage_counts},
    probability::calculate_probabilities,
    validation::validate_distribution_draft,
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::Utc;
use football_domain::{FormationUsageDistributionDraft, FormationUsageDistributionRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn save_formation_usage_distribution(
        &self,
        draft: &FormationUsageDistributionDraft,
    ) -> PersistenceResult<FormationUsageDistributionRecord> {
        validate_distribution_draft(draft)?;
        let (window_start, window_end) = self.resolve_formation_window(draft).await?;

        let counts = collect_usage_counts(draft)?;
        let formation_ids: Vec<Uuid> = counts.keys().copied().collect();
        self.ensure_active_formations(&formation_ids).await?;

        let counts = complete_usage_counts(counts, draft.observed_matches)?;
        let probabilities = calculate_probabilities(&counts, draft.observed_matches, draft.alpha)?;
        let observed_at = Utc::now();

        self.write_formation_usage_observations(
            draft,
            window_start,
            window_end,
            observed_at,
            &counts,
            &probabilities,
        )
        .await?;

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
