use super::{
    mapping::match_record_from_row,
    scope::{resolve_match_scope_draft, validate_match_scope},
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{MatchDraft, MatchRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_match(&self, draft: &MatchDraft) -> PersistenceResult<MatchRecord> {
        let resolved = resolve_match_scope_draft(&self.pool, draft).await?;
        let draft = &resolved;
        if draft.home_team_id == draft.away_team_id {
            return Err(PersistenceError::InvalidState(
                "主队和客队不能相同".to_string(),
            ));
        }
        validate_match_scope(&self.pool, draft).await?;
        let external_key = if draft.external_key.trim().is_empty() {
            let kickoff = draft.kickoff_time.format("%Y%m%dT%H%MZ");
            let home = draft.home_team_id.simple().to_string();
            let away = draft.away_team_id.simple().to_string();
            format!("MATCH-{kickoff}-{}-{}", &home[..8], &away[..8])
        } else {
            draft.external_key.trim().to_string()
        };
        let generated_id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            WITH upserted AS (
                INSERT INTO football.matches (
                    id, external_key, competition_id, season_id, stage_id, round_id,
                    home_team_id, away_team_id, kickoff_time, status, venue, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (external_key) DO UPDATE SET
                    competition_id = EXCLUDED.competition_id,
                    season_id = EXCLUDED.season_id,
                    stage_id = EXCLUDED.stage_id,
                    round_id = EXCLUDED.round_id,
                    home_team_id = EXCLUDED.home_team_id,
                    away_team_id = EXCLUDED.away_team_id,
                    kickoff_time = EXCLUDED.kickoff_time,
                    status = EXCLUDED.status,
                    venue = EXCLUDED.venue,
                    metadata = football.matches.metadata || EXCLUDED.metadata,
                    updated_at = now()
                RETURNING *
            )
            SELECT
                upserted.id, upserted.external_key,
                upserted.competition_id, competition.name AS competition_name,
                upserted.season_id, upserted.stage_id, upserted.round_id,
                upserted.home_team_id, home.canonical_name AS home_team_name,
                upserted.away_team_id, away.canonical_name AS away_team_name,
                upserted.kickoff_time, upserted.status, upserted.venue
            FROM upserted
            LEFT JOIN football.competitions competition ON competition.id = upserted.competition_id
            JOIN football.teams home ON home.id = upserted.home_team_id
            JOIN football.teams away ON away.id = upserted.away_team_id
            "#,
        )
        .bind(generated_id)
        .bind(&external_key)
        .bind(draft.competition_id)
        .bind(draft.season_id)
        .bind(draft.stage_id)
        .bind(draft.round_id)
        .bind(draft.home_team_id)
        .bind(draft.away_team_id)
        .bind(draft.kickoff_time)
        .bind(draft.status.as_str())
        .bind(
            draft
                .venue
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        match_record_from_row(&row)
    }
}
