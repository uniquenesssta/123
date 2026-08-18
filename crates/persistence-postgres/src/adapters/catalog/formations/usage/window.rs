use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{Datelike, NaiveDate, Utc};
use football_domain::FormationUsageDistributionDraft;

impl PostgresStore {
    pub(crate) async fn resolve_formation_window(
        &self,
        draft: &FormationUsageDistributionDraft,
    ) -> PersistenceResult<(NaiveDate, NaiveDate)> {
        let today = Utc::now().date_naive();
        match draft.window_preset.trim() {
            "custom" => {
                let start = draft.window_start.ok_or_else(|| {
                    PersistenceError::InvalidState("自定义窗口必须填写开始日期".to_string())
                })?;
                let end = draft.window_end.ok_or_else(|| {
                    PersistenceError::InvalidState("自定义窗口必须填写结束日期".to_string())
                })?;
                if end < start {
                    return Err(PersistenceError::InvalidState(
                        "观察窗口结束日期不能早于开始日期".to_string(),
                    ));
                }
                Ok((start, end))
            }
            "last_5" | "last_10" | "last_20" => {
                let team_id = draft.team_id.ok_or_else(|| {
                    PersistenceError::InvalidState("最近场次窗口必须选择球队".to_string())
                })?;
                let limit = match draft.window_preset.as_str() {
                    "last_5" => 5_i64,
                    "last_10" => 10_i64,
                    _ => 20_i64,
                };
                let dates = sqlx::query_scalar::<_, NaiveDate>(
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
                .bind(today)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?;
                let end = dates.first().copied().ok_or_else(|| {
                    PersistenceError::InvalidState(
                        "数据库中没有可用于最近场次窗口的已结束比赛".to_string(),
                    )
                })?;
                let start = dates.last().copied().unwrap_or(end);
                Ok((start, end))
            }
            "current_coach_term" => {
                let team_id = draft.team_id.ok_or_else(|| {
                    PersistenceError::InvalidState("当前教练任期必须选择球队".to_string())
                })?;
                let coach_id = draft.coach_id.ok_or_else(|| {
                    PersistenceError::InvalidState("当前教练任期必须选择教练".to_string())
                })?;
                sqlx::query_as::<_, (NaiveDate, Option<NaiveDate>)>(
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
                .bind(today)
                .fetch_optional(&self.pool)
                .await?
                .map(|(start, end)| (start, end.unwrap_or(today).min(today)))
                .ok_or_else(|| PersistenceError::InvalidState("没有找到对应的教练任期".to_string()))
            }
            "current_season" => {
                let range = if let Some(competition_id) = draft.competition_id {
                    sqlx::query_as::<_, (Option<NaiveDate>, Option<NaiveDate>)>(
                        r#"
                            SELECT min(kickoff_time::date), max(kickoff_time::date)
                            FROM football.matches
                            WHERE competition_id=$1
                              AND extract(year from kickoff_time)=$2
                              AND kickoff_time::date <= $3
                            "#,
                    )
                    .bind(competition_id)
                    .bind(today.year())
                    .bind(today)
                    .fetch_one(&self.pool)
                    .await?
                } else if let Some(team_id) = draft.team_id {
                    sqlx::query_as::<_, (Option<NaiveDate>, Option<NaiveDate>)>(
                        r#"
                            SELECT min(kickoff_time::date), max(kickoff_time::date)
                            FROM football.matches
                            WHERE (home_team_id=$1 OR away_team_id=$1)
                              AND extract(year from kickoff_time)=$2
                              AND kickoff_time::date <= $3
                            "#,
                    )
                    .bind(team_id)
                    .bind(today.year())
                    .bind(today)
                    .fetch_one(&self.pool)
                    .await?
                } else {
                    (None, None)
                };
                if let (Some(start), Some(end)) = range {
                    return Ok((start, end));
                }
                let start = NaiveDate::from_ymd_opt(today.year(), 1, 1).ok_or_else(|| {
                    PersistenceError::InvalidState("无法生成当前赛季日期".to_string())
                })?;
                Ok((start, today))
            }
            other => Err(PersistenceError::InvalidState(format!(
                "未知阵型观察窗口：{other}"
            ))),
        }
    }
}
