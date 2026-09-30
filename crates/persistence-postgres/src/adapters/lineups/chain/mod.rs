mod validation;
mod window;

pub(crate) use validation::refresh_lineup_validation_in_tx;
pub(crate) use window::{
    lineup_snapshot_window_at, normalize_lineup_snapshot_type, LineupSnapshotWindow,
};

use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{MatchLineupChain, MatchLineupTeamChain};
use uuid::Uuid;

fn lineup_team_blocking_issues(
    versions: &[football_domain::LineupRecord],
    selected_lineup_id: Option<Uuid>,
    window_start: Option<DateTime<Utc>>,
    cutoff: DateTime<Utc>,
    snapshot_type: &str,
) -> Vec<String> {
    if selected_lineup_id.is_some() {
        return Vec::new();
    }
    if versions.is_empty() {
        return vec!["尚未创建任何阵容版本".to_string()];
    }
    let mut issues = Vec::new();
    let active = versions
        .iter()
        .filter(|lineup| lineup.status == "active")
        .collect::<Vec<_>>();
    if active.is_empty() {
        issues.push("已有阵容版本均已被后续修订替代".to_string());
        return issues;
    }
    if active.iter().all(|lineup| lineup.captured_at > cutoff) {
        issues.push(format!(
            "已有阵容记录均晚于 {snapshot_type} 当前可用截止时间 {}",
            cutoff.to_rfc3339()
        ));
    }
    if let Some(window_start) = window_start {
        if active
            .iter()
            .all(|lineup| lineup.captured_at < window_start)
        {
            issues.push(format!(
                "已有阵容记录均早于 {snapshot_type} 窗口起点 {}",
                window_start.to_rfc3339()
            ));
        }
    }
    if let Some(latest) = active
        .iter()
        .max_by_key(|lineup| (lineup.captured_at, lineup.id))
    {
        if latest.lineup_type.as_str() == "actual" {
            issues.push("最新版本是实际阵容，只用于赛后复盘".to_string());
        }
        for error in &latest.validation_errors {
            if !issues.contains(error) {
                issues.push(error.clone());
            }
        }
    }
    if issues.is_empty() {
        issues.push("当前时点没有通过完整性校验的预计或确认阵容".to_string());
    }
    issues
}

impl PostgresStore {
    pub(crate) async fn preferred_lineup_id(
        &self,
        match_id: Uuid,
        team_id: Uuid,
        window: LineupSnapshotWindow,
    ) -> PersistenceResult<Option<Uuid>> {
        let id = sqlx::query_scalar(
            r#"
            SELECT lineup.id
            FROM football.lineups lineup
            WHERE lineup.match_id=$1 AND lineup.team_id=$2
              AND lineup.status='active' AND lineup.history_hidden_at IS NULL AND lineup.model_eligible
              AND lineup.lineup_type IN ('confirmed','expected')
              AND lineup.snapshot_type IN ('T-N','T-24h','T-6h','T-1h')
              AND lineup.captured_at <= $3
              AND ($4::timestamptz IS NULL OR lineup.captured_at >= $4)
            ORDER BY lineup.captured_at DESC,
                     CASE lineup.lineup_type WHEN 'confirmed' THEN 2 ELSE 1 END DESC,
                     lineup.created_at DESC, lineup.id DESC
            LIMIT 1
            "#,
        )
        .bind(match_id)
        .bind(team_id)
        .bind(window.cutoff_time)
        .bind(window.start_time)
        .fetch_optional(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn read_match_lineup_chain(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
    ) -> PersistenceResult<MatchLineupChain> {
        self.read_match_lineup_chain_at(match_id, snapshot_type, Utc::now())
            .await
    }

    pub async fn read_match_lineup_chain_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        reference_time: DateTime<Utc>,
    ) -> PersistenceResult<MatchLineupChain> {
        let snapshot_type = normalize_lineup_snapshot_type(snapshot_type)?.to_string();
        let match_record = self.read_match(match_id).await?;
        let window =
            lineup_snapshot_window_at(match_record.kickoff_time, &snapshot_type, reference_time)?;
        let summaries = self.list_lineups(Some(match_id), 500).await?;
        let mut home_versions = Vec::new();
        let mut away_versions = Vec::new();
        for summary in summaries {
            let record = self.read_lineup(summary.id).await?;
            if record.team_id == match_record.home_team_id {
                home_versions.push(record);
            } else if record.team_id == match_record.away_team_id {
                away_versions.push(record);
            }
        }
        let home_selected = self
            .preferred_lineup_id(match_id, match_record.home_team_id, window)
            .await?;
        let away_selected = self
            .preferred_lineup_id(match_id, match_record.away_team_id, window)
            .await?;
        let home_issues = lineup_team_blocking_issues(
            &home_versions,
            home_selected,
            window.start_time,
            window.cutoff_time,
            &snapshot_type,
        );
        let away_issues = lineup_team_blocking_issues(
            &away_versions,
            away_selected,
            window.start_time,
            window.cutoff_time,
            &snapshot_type,
        );
        let mut blocking = Vec::new();
        blocking.extend(
            home_issues
                .iter()
                .map(|issue| format!("{}：{issue}", match_record.home_team_name)),
        );
        blocking.extend(
            away_issues
                .iter()
                .map(|issue| format!("{}：{issue}", match_record.away_team_name)),
        );
        Ok(MatchLineupChain {
            match_record: match_record.clone(),
            snapshot_type,
            data_window_start_time: window.start_time,
            data_cutoff_time: window.cutoff_time,
            home: MatchLineupTeamChain {
                team_id: match_record.home_team_id,
                team_name: match_record.home_team_name.clone(),
                team_side: "home".to_string(),
                selected_lineup_id: home_selected,
                versions: home_versions,
                blocking_issues: home_issues,
            },
            away: MatchLineupTeamChain {
                team_id: match_record.away_team_id,
                team_name: match_record.away_team_name.clone(),
                team_side: "away".to_string(),
                selected_lineup_id: away_selected,
                versions: away_versions,
                blocking_issues: away_issues,
            },
            ready_for_model: blocking.is_empty(),
            blocking_issues: blocking,
        })
    }
}
