use super::{mapper::map_membership_option, row::SeasonTeamMembershipRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::SeasonTeamMembershipOption;

impl PostgresStore {
    pub(crate) async fn list_season_team_memberships(
        &self,
    ) -> PersistenceResult<Vec<SeasonTeamMembershipOption>> {
        let rows = sqlx::query_as::<_, SeasonTeamMembershipRow>(
            r#"
            SELECT season_id, team_id, registration_status
            FROM football.team_season_memberships
            WHERE registration_status IN ('registered', 'guest')
            ORDER BY season_id, team_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(map_membership_option).collect())
    }
}
