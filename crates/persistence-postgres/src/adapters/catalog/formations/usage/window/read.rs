use crate::{PersistenceResult, PostgresStore};
use chrono::NaiveDate;
use uuid::Uuid;

impl PostgresStore {
    pub(crate) async fn read_recent_finished_match_dates(
        &self,
        team_id: Uuid,
        end_date: NaiveDate,
        limit: i64,
    ) -> PersistenceResult<Vec<NaiveDate>> {
        Ok(sqlx::query_scalar::<_, NaiveDate>(
            r#"
            SELECT fixture.kickoff_time::date
            FROM football.matches fixture
            WHERE (fixture.home_team_id=$1 OR fixture.away_team_id=$1)
              AND fixture.status='finished'
              AND fixture.kickoff_time::date <= $2
            ORDER BY fixture.kickoff_time DESC, fixture.id DESC
            LIMIT $3
            "#,
        )
        .bind(team_id)
        .bind(end_date)
        .bind(limit)
        .fetch_all(&self.pool)
        .await?)
    }

    pub(crate) async fn read_coach_term_window(
        &self,
        team_id: Uuid,
        coach_id: Uuid,
        as_of: NaiveDate,
    ) -> PersistenceResult<Option<(NaiveDate, Option<NaiveDate>)>> {
        Ok(sqlx::query_as::<_, (NaiveDate, Option<NaiveDate>)>(
            r#"
            SELECT valid_from, valid_to
            FROM football.team_coach_periods
            WHERE team_id=$1 AND coach_id=$2
              AND valid_from <= $3
            ORDER BY valid_from DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(team_id)
        .bind(coach_id)
        .bind(as_of)
        .fetch_optional(&self.pool)
        .await?)
    }

    pub(crate) async fn read_competition_season_range(
        &self,
        competition_id: Uuid,
        year: i32,
        as_of: NaiveDate,
    ) -> PersistenceResult<(Option<NaiveDate>, Option<NaiveDate>)> {
        Ok(sqlx::query_as::<_, (Option<NaiveDate>, Option<NaiveDate>)>(
            r#"
            SELECT min(kickoff_time::date), max(kickoff_time::date)
            FROM football.matches
            WHERE competition_id=$1
              AND extract(year from kickoff_time)=$2
              AND kickoff_time::date <= $3
            "#,
        )
        .bind(competition_id)
        .bind(year)
        .bind(as_of)
        .fetch_one(&self.pool)
        .await?)
    }

    pub(crate) async fn read_team_season_range(
        &self,
        team_id: Uuid,
        year: i32,
        as_of: NaiveDate,
    ) -> PersistenceResult<(Option<NaiveDate>, Option<NaiveDate>)> {
        Ok(sqlx::query_as::<_, (Option<NaiveDate>, Option<NaiveDate>)>(
            r#"
            SELECT min(kickoff_time::date), max(kickoff_time::date)
            FROM football.matches
            WHERE (home_team_id=$1 OR away_team_id=$1)
              AND extract(year from kickoff_time)=$2
              AND kickoff_time::date <= $3
            "#,
        )
        .bind(team_id)
        .bind(year)
        .bind(as_of)
        .fetch_one(&self.pool)
        .await?)
    }
}
