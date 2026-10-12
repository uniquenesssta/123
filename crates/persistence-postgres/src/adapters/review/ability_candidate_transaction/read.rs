use super::mapping::ability_candidate_from_row;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{AbilityCandidateStatus, AbilityUpdateCandidateRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_ability_candidates(
        &self,
        status: Option<AbilityCandidateStatus>,
        limit: u32,
        match_review_id: Option<Uuid>,
    ) -> PersistenceResult<Vec<AbilityUpdateCandidateRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                candidate.id, candidate.match_review_id, candidate.player_match_review_id,
                candidate.player_id, player.canonical_name AS player_name,
                candidate.dimension_code, dimension.name AS dimension_name,
                candidate.current_value, candidate.proposed_value,
                candidate.confidence, candidate.sample_size, candidate.evidence,
                candidate.calculation_version, candidate.status,
                candidate.created_at, candidate.decided_at,
                candidate.decided_by, candidate.decision_note,
                candidate.accepted_observation_id
            FROM review.ability_update_candidates candidate
            JOIN football.players player ON player.id = candidate.player_id
            JOIN feature.player_ability_dimensions dimension ON dimension.code = candidate.dimension_code
            WHERE ($1::text IS NULL OR candidate.status = $1)
              AND ($2::uuid IS NULL OR candidate.match_review_id = $2)
            ORDER BY
                CASE candidate.status WHEN 'pending' THEN 0 ELSE 1 END,
                candidate.confidence DESC, candidate.created_at DESC
            LIMIT $3
            "#,
        )
        .bind(status.map(AbilityCandidateStatus::as_str))
        .bind(match_review_id)
        .bind(i64::from(limit.clamp(1, 2000)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(ability_candidate_from_row).collect()
    }
}

pub(super) async fn read_ability_candidate(
    pool: &sqlx::PgPool,
    candidate_id: Uuid,
) -> PersistenceResult<AbilityUpdateCandidateRecord> {
    let row = sqlx::query(
        r#"
        SELECT
            candidate.id, candidate.match_review_id, candidate.player_match_review_id,
            candidate.player_id, player.canonical_name AS player_name,
            candidate.dimension_code, dimension.name AS dimension_name,
            candidate.current_value, candidate.proposed_value,
            candidate.confidence, candidate.sample_size, candidate.evidence,
            candidate.calculation_version, candidate.status,
            candidate.created_at, candidate.decided_at,
            candidate.decided_by, candidate.decision_note,
            candidate.accepted_observation_id
        FROM review.ability_update_candidates candidate
        JOIN football.players player ON player.id = candidate.player_id
        JOIN feature.player_ability_dimensions dimension ON dimension.code = candidate.dimension_code
        WHERE candidate.id = $1
        "#,
    )
    .bind(candidate_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("能力更新候选不存在".to_string()))?;
    ability_candidate_from_row(&row)
}
