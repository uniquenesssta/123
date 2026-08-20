use crate::{PersistenceError, PersistenceResult};
use chrono::{Datelike, NaiveDate};
use football_domain::MatchDraft;
use sqlx::Row;
use uuid::Uuid;

pub(super) async fn resolve_match_scope_draft(
    pool: &sqlx::PgPool,
    draft: &MatchDraft,
) -> PersistenceResult<MatchDraft> {
    let mut resolved = draft.clone();
    if let Some(round_id) = resolved.round_id {
        let row = sqlx::query(
            r#"
            SELECT round.stage_id, stage.season_id, season.competition_id
            FROM football.rounds round
            JOIN football.competition_stages stage ON stage.id = round.stage_id
            JOIN football.seasons season ON season.id = stage.season_id
            WHERE round.id = $1
            "#,
        )
        .bind(round_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛轮次不存在".to_string()))?;
        resolved.stage_id.get_or_insert(row.try_get("stage_id")?);
        resolved.season_id.get_or_insert(row.try_get("season_id")?);
        resolved
            .competition_id
            .get_or_insert(row.try_get("competition_id")?);
    } else if let Some(stage_id) = resolved.stage_id {
        let row = sqlx::query(
            r#"
            SELECT stage.season_id, season.competition_id
            FROM football.competition_stages stage
            JOIN football.seasons season ON season.id = stage.season_id
            WHERE stage.id = $1
            "#,
        )
        .bind(stage_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛阶段不存在".to_string()))?;
        resolved.season_id.get_or_insert(row.try_get("season_id")?);
        resolved
            .competition_id
            .get_or_insert(row.try_get("competition_id")?);
    } else if let Some(season_id) = resolved.season_id {
        let competition_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT competition_id FROM football.seasons WHERE id = $1",
        )
        .bind(season_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛赛季不存在".to_string()))?;
        resolved.competition_id.get_or_insert(competition_id);
    } else if let Some(competition_id) = resolved.competition_id {
        let competition = sqlx::query(
            r#"
            SELECT timezone,
                   ($2::timestamptz AT TIME ZONE timezone)::date AS local_kickoff_date,
                   COALESCE(NULLIF(metadata->>'season_pattern', ''), 'calendar') AS season_pattern
            FROM football.competitions
            WHERE id = $1 AND is_active
            "#,
        )
        .bind(competition_id)
        .bind(resolved.kickoff_time)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛赛事不存在或已停用".to_string()))?;
        let timezone: String = competition.try_get("timezone")?;
        let kickoff_date: NaiveDate = competition.try_get("local_kickoff_date")?;
        let season_pattern: String = competition.try_get("season_pattern")?;
        resolved.season_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id
            FROM football.seasons
            WHERE competition_id = $1
              AND (starts_on IS NULL OR starts_on <= $2)
              AND (ends_on IS NULL OR ends_on >= $2)
              AND status IN ('active', 'planned', 'completed')
            ORDER BY
              CASE status WHEN 'active' THEN 0 WHEN 'planned' THEN 1 ELSE 2 END,
              starts_on DESC NULLS LAST,
              id
            LIMIT 1
            "#,
        )
        .bind(competition_id)
        .bind(kickoff_date)
        .fetch_optional(pool)
        .await?;
        if resolved.season_id.is_none() {
            let (season_name, starts_on, ends_on) =
                automatic_season_identity(&season_pattern, kickoff_date)?;
            resolved.season_id = Some(
                sqlx::query_scalar::<_, Uuid>(
                    r#"
                    INSERT INTO football.seasons (
                        id, competition_id, name, starts_on, ends_on, status, metadata
                    ) VALUES (
                        $1, $2, $3, $4, $5, 'active',
                        jsonb_build_object(
                            'auto_created', true,
                            'season_pattern', $6,
                            'competition_timezone', $7,
                            'local_kickoff_date', $8::text
                        )
                    )
                    ON CONFLICT (competition_id, name) DO UPDATE SET
                        starts_on = COALESCE(football.seasons.starts_on, EXCLUDED.starts_on),
                        ends_on = COALESCE(football.seasons.ends_on, EXCLUDED.ends_on),
                        status = CASE
                            WHEN football.seasons.status = 'archived' THEN football.seasons.status
                            ELSE 'active'
                        END,
                        metadata = football.seasons.metadata || EXCLUDED.metadata
                    RETURNING id
                    "#,
                )
                .bind(Uuid::new_v4())
                .bind(competition_id)
                .bind(season_name)
                .bind(starts_on)
                .bind(ends_on)
                .bind(season_pattern)
                .bind(timezone)
                .bind(kickoff_date)
                .fetch_one(pool)
                .await?,
            );
        }
    }
    Ok(resolved)
}

fn automatic_season_identity(
    season_pattern: &str,
    kickoff_date: NaiveDate,
) -> PersistenceResult<(String, NaiveDate, NaiveDate)> {
    let year = kickoff_date.year();
    if season_pattern.eq_ignore_ascii_case("cross_year") {
        let start_year = if kickoff_date.month() >= 7 {
            year
        } else {
            year - 1
        };
        let end_year = start_year + 1;
        let starts_on = NaiveDate::from_ymd_opt(start_year, 7, 1)
            .ok_or_else(|| PersistenceError::InvalidState("自动赛季开始日期无效".to_string()))?;
        let ends_on = NaiveDate::from_ymd_opt(end_year, 6, 30)
            .ok_or_else(|| PersistenceError::InvalidState("自动赛季结束日期无效".to_string()))?;
        return Ok((
            format!("{start_year}/{}", end_year % 100),
            starts_on,
            ends_on,
        ));
    }
    let starts_on = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| PersistenceError::InvalidState("自动赛季开始日期无效".to_string()))?;
    let ends_on = NaiveDate::from_ymd_opt(year, 12, 31)
        .ok_or_else(|| PersistenceError::InvalidState("自动赛季结束日期无效".to_string()))?;
    Ok((year.to_string(), starts_on, ends_on))
}

pub(super) async fn validate_match_scope(
    pool: &sqlx::PgPool,
    draft: &MatchDraft,
) -> PersistenceResult<()> {
    if let Some(round_id) = draft.round_id {
        let row = sqlx::query(
            r#"
            SELECT round.stage_id, stage.season_id, season.competition_id
            FROM football.rounds round
            JOIN football.competition_stages stage ON stage.id = round.stage_id
            JOIN football.seasons season ON season.id = stage.season_id
            WHERE round.id = $1
            "#,
        )
        .bind(round_id)
        .fetch_one(pool)
        .await?;
        let stage_id: Uuid = row.try_get("stage_id")?;
        let season_id: Uuid = row.try_get("season_id")?;
        let competition_id: Uuid = row.try_get("competition_id")?;
        if draft.stage_id.is_some_and(|value| value != stage_id)
            || draft.season_id.is_some_and(|value| value != season_id)
            || draft
                .competition_id
                .is_some_and(|value| value != competition_id)
        {
            return Err(PersistenceError::InvalidState(
                "比赛轮次、阶段、赛季或赛事层级不一致".to_string(),
            ));
        }
    } else if let Some(stage_id) = draft.stage_id {
        let row = sqlx::query(
            r#"
            SELECT stage.season_id, season.competition_id
            FROM football.competition_stages stage
            JOIN football.seasons season ON season.id = stage.season_id
            WHERE stage.id = $1
            "#,
        )
        .bind(stage_id)
        .fetch_one(pool)
        .await?;
        let season_id: Uuid = row.try_get("season_id")?;
        let competition_id: Uuid = row.try_get("competition_id")?;
        if draft.season_id.is_some_and(|value| value != season_id)
            || draft
                .competition_id
                .is_some_and(|value| value != competition_id)
        {
            return Err(PersistenceError::InvalidState(
                "比赛阶段、赛季或赛事层级不一致".to_string(),
            ));
        }
    } else if let Some(season_id) = draft.season_id {
        let competition_id: Uuid =
            sqlx::query_scalar("SELECT competition_id FROM football.seasons WHERE id = $1")
                .bind(season_id)
                .fetch_one(pool)
                .await?;
        if draft
            .competition_id
            .is_some_and(|value| value != competition_id)
        {
            return Err(PersistenceError::InvalidState(
                "比赛赛季不属于所选赛事".to_string(),
            ));
        }
    }
    Ok(())
}
