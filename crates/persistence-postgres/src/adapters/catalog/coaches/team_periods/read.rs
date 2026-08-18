use crate::{
    adapters::catalog::coaches::mapping::team_coach_period_from_row, PersistenceResult,
    PostgresStore,
};
use football_domain::TeamCoachPeriodRecord;
use uuid::Uuid;
impl PostgresStore {
    pub(crate) async fn list_team_coach_periods(
        &self,
        team_id: Uuid,
    ) -> PersistenceResult<Vec<TeamCoachPeriodRecord>> {
        sqlx::query(
            r#"
                SELECT period.id, period.team_id, team.canonical_name AS team_name,
                       period.coach_id, coach.canonical_name AS coach_name, period.role,
                       period.valid_from, period.valid_to, period.is_interim, period.confidence
                FROM football.team_coach_periods period
                JOIN football.teams team ON team.id=period.team_id
                JOIN football.coaches coach ON coach.id=period.coach_id
                WHERE period.team_id=$1
                ORDER BY period.valid_from DESC, period.valid_to DESC NULLS FIRST, period.id DESC
                "#,
        )
        .bind(team_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(team_coach_period_from_row)
        .collect()
    }
}

impl PostgresStore {
    pub(crate) async fn list_coach_team_periods(
        &self,
        coach_id: Uuid,
    ) -> PersistenceResult<Vec<TeamCoachPeriodRecord>> {
        sqlx::query(
            r#"
                SELECT period.id, period.team_id, team.canonical_name AS team_name,
                       period.coach_id, coach.canonical_name AS coach_name, period.role,
                       period.valid_from, period.valid_to, period.is_interim, period.confidence
                FROM football.team_coach_periods period
                JOIN football.teams team ON team.id=period.team_id
                JOIN football.coaches coach ON coach.id=period.coach_id
                WHERE period.coach_id=$1
                ORDER BY period.valid_from DESC, period.valid_to DESC NULLS FIRST, period.id DESC
                "#,
        )
        .bind(coach_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(team_coach_period_from_row)
        .collect()
    }
}

impl PostgresStore {
    pub(crate) async fn read_team_coach_period(
        &self,
        period_id: Uuid,
    ) -> PersistenceResult<TeamCoachPeriodRecord> {
        let row = sqlx::query(
            r#"
                SELECT period.id, period.team_id, team.canonical_name AS team_name,
                       period.coach_id, coach.canonical_name AS coach_name, period.role,
                       period.valid_from, period.valid_to, period.is_interim, period.confidence
                FROM football.team_coach_periods period
                JOIN football.teams team ON team.id=period.team_id
                JOIN football.coaches coach ON coach.id=period.coach_id
                WHERE period.id=$1
                "#,
        )
        .bind(period_id)
        .fetch_one(&self.pool)
        .await?;
        team_coach_period_from_row(&row)
    }
}
