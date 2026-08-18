use crate::{
    adapters::catalog::formations::constants::UNKNOWN_FORMATION_ID, PersistenceResult,
    PostgresStore,
};
use chrono::{DateTime, NaiveDate, Utc};
use football_domain::FormationUsageEntryRecord;
use sqlx::Row;
use uuid::Uuid;

pub(crate) struct MatchFormationObservation {
    pub(crate) formation_id: Uuid,
    pub(crate) formation_code: String,
    pub(crate) formation_name: String,
    pub(crate) quality_score: Option<f64>,
    pub(crate) lineup_type: String,
    pub(crate) competition_id: Option<Uuid>,
}

impl PostgresStore {
    pub(crate) async fn read_match_formation_observation(
        &self,
        match_id: Uuid,
        team_id: Uuid,
        as_of: DateTime<Utc>,
    ) -> PersistenceResult<Option<MatchFormationObservation>> {
        let row = sqlx::query(
            r#"
            SELECT lineup.formation_id, formation.code, formation.name,
                   lineup.quality_score, lineup.lineup_type,
                   fixture.competition_id
            FROM football.lineups lineup
            JOIN football.matches fixture ON fixture.id = lineup.match_id
            JOIN football.formations formation ON formation.id = lineup.formation_id
            WHERE lineup.match_id = $1
              AND lineup.team_id = $2
              AND lineup.status = 'active'
              AND lineup.lineup_type IN ('actual','confirmed')
              AND lineup.captured_at <= $3
            ORDER BY CASE lineup.lineup_type WHEN 'actual' THEN 0 ELSE 1 END,
                     lineup.captured_at DESC, lineup.id DESC
            LIMIT 1
            "#,
        )
        .bind(match_id)
        .bind(team_id)
        .bind(as_of)
        .fetch_optional(&self.pool)
        .await?;

        row.map(|row| {
            Ok(MatchFormationObservation {
                formation_id: row.try_get("formation_id")?,
                formation_code: row.try_get("code")?,
                formation_name: row.try_get("name")?,
                quality_score: row.try_get("quality_score")?,
                lineup_type: row.try_get("lineup_type")?,
                competition_id: row.try_get("competition_id")?,
            })
        })
        .transpose()
    }

    pub(crate) async fn read_match_competition_id(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<Option<Uuid>> {
        Ok(
            sqlx::query_scalar::<_, Option<Uuid>>(
                "SELECT competition_id FROM football.matches WHERE id=$1",
            )
            .bind(match_id)
            .fetch_optional(&self.pool)
            .await?
            .flatten(),
        )
    }

    pub(crate) async fn read_current_head_coach_id(
        &self,
        team_id: Uuid,
        as_of: NaiveDate,
    ) -> PersistenceResult<Option<Uuid>> {
        Ok(sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT coach_id
            FROM football.team_coach_periods
            WHERE team_id=$1
              AND role IN ('head_coach','interim_head_coach','caretaker')
              AND valid_from <= $2
              AND (valid_to IS NULL OR valid_to >= $2)
            ORDER BY CASE role WHEN 'head_coach' THEN 0 WHEN 'interim_head_coach' THEN 1 ELSE 2 END,
                     valid_from DESC, id DESC
            LIMIT 1
            "#,
        )
        .bind(team_id)
        .bind(as_of)
        .fetch_optional(&self.pool)
        .await?)
    }

    pub(crate) async fn read_unknown_formation_entry(
        &self,
    ) -> PersistenceResult<FormationUsageEntryRecord> {
        let row = sqlx::query("SELECT id, code, name FROM football.formations WHERE id=$1")
            .bind(UNKNOWN_FORMATION_ID)
            .fetch_one(&self.pool)
            .await?;
        Ok(FormationUsageEntryRecord {
            id: Uuid::nil(),
            formation_id: row.try_get("id")?,
            formation_code: row.try_get("code")?,
            formation_name: row.try_get("name")?,
            usage_count: 0,
            raw_probability: 1.0,
            smoothed_probability: 1.0,
        })
    }
}
