mod read;

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
                let dates = self
                    .read_recent_finished_match_dates(team_id, today, limit)
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
                self.read_coach_term_window(team_id, coach_id, today)
                    .await?
                    .map(|(start, end)| (start, end.unwrap_or(today).min(today)))
                    .ok_or_else(|| {
                        PersistenceError::InvalidState("没有找到对应的教练任期".to_string())
                    })
            }
            "current_season" => {
                let range = if let Some(competition_id) = draft.competition_id {
                    self.read_competition_season_range(competition_id, today.year(), today)
                        .await?
                } else if let Some(team_id) = draft.team_id {
                    self.read_team_season_range(team_id, today.year(), today)
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
