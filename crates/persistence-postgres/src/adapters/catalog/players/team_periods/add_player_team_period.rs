use super::{
    input_policy::validate_player_team_period_draft, mapper::map_player_team_period,
    row::PlayerTeamPeriodRow,
};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{PlayerTeamPeriodDraft, PlayerTeamPeriodRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn add_player_team_period(
        &self,
        draft: &PlayerTeamPeriodDraft,
    ) -> PersistenceResult<PlayerTeamPeriodRecord> {
        let validated = validate_player_team_period_draft(draft)?;
        let row = sqlx::query_as::<_, PlayerTeamPeriodRow>(
            r#"
            WITH inserted AS (
                INSERT INTO football.player_team_periods (
                    id, player_id, team_id, season_id, squad_number,
                    valid_from, valid_to, registration_status, source_document_id
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
            )
            SELECT
                inserted.id, inserted.player_id, inserted.team_id,
                team.canonical_name AS team_name,
                inserted.season_id, season.name AS season_name,
                inserted.squad_number, inserted.valid_from, inserted.valid_to,
                inserted.registration_status
            FROM inserted
            JOIN football.teams team ON team.id = inserted.team_id
            LEFT JOIN football.seasons season ON season.id = inserted.season_id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.player_id)
        .bind(draft.team_id)
        .bind(draft.season_id)
        .bind(draft.squad_number)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .bind(validated.registration_status)
        .bind(draft.source_document_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(map_player_team_period(row))
    }
}
