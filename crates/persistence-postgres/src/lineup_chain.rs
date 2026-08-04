use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Duration, Utc};
use football_domain::{
    MatchLineupChain, MatchLineupTeamChain, TeamMatchLineupHistoryItem,
    FORMAL_LINEUP_SNAPSHOT_TYPES,
};
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Copy)]
pub(crate) struct LineupSnapshotWindow {
    pub start_time: Option<DateTime<Utc>>,
    pub cutoff_time: DateTime<Utc>,
}

pub(crate) fn normalize_lineup_snapshot_type(value: &str) -> PersistenceResult<&'static str> {
    match value.trim() {
        "T-N" => Ok("T-N"),
        "T-24h" => Ok("T-24h"),
        "T-6h" => Ok("T-6h"),
        "T-1h" => Ok("T-1h"),
        "T-90m" => Err(PersistenceError::InvalidState(
            "T-90m 已停止用于新阵容和新推演；请选择 T-N、T-24h、T-6h 或 T-1h"
                .to_string(),
        )),
        other => Err(PersistenceError::InvalidState(format!(
            "不支持的阵容时间窗口：{other}"
        ))),
    }
}

pub(crate) fn lineup_snapshot_window(
    kickoff_time: DateTime<Utc>,
    snapshot_type: &str,
) -> PersistenceResult<LineupSnapshotWindow> {
    lineup_snapshot_window_at(kickoff_time, snapshot_type, Utc::now())
}

pub(crate) fn lineup_snapshot_window_at(
    kickoff_time: DateTime<Utc>,
    snapshot_type: &str,
    reference_time: DateTime<Utc>,
) -> PersistenceResult<LineupSnapshotWindow> {
    let snapshot_type = normalize_lineup_snapshot_type(snapshot_type)?;
    let cutoff_time = reference_time.min(kickoff_time - Duration::seconds(1));
    let start_time = match snapshot_type {
        "T-N" => None,
        "T-24h" => Some(kickoff_time - Duration::hours(24)),
        "T-6h" => Some(kickoff_time - Duration::hours(6)),
        "T-1h" => Some(kickoff_time - Duration::hours(1)),
        _ => unreachable!(),
    };
    if let Some(start_time) = start_time {
        if cutoff_time < start_time {
            return Err(PersistenceError::InvalidState(format!(
                "{snapshot_type} 数据窗口尚未开启；窗口从 {} 开始",
                start_time.to_rfc3339()
            )));
        }
    }
    Ok(LineupSnapshotWindow {
        start_time,
        cutoff_time,
    })
}

pub(crate) async fn refresh_lineup_validation_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    lineup_id: Uuid,
) -> PersistenceResult<()> {
    let row = sqlx::query(
        r#"
        SELECT lineup.match_id, lineup.team_id, lineup.lineup_type, lineup.snapshot_type,
               lineup.formation_id, lineup.captured_at, fixture.kickoff_time,
               count(player.player_id)::bigint AS player_count,
               count(player.player_id) FILTER (WHERE player.is_starter)::bigint AS starter_count
        FROM football.lineups lineup
        JOIN football.matches fixture ON fixture.id = lineup.match_id
        LEFT JOIN football.lineup_players player ON player.lineup_id = lineup.id
        WHERE lineup.id = $1
        GROUP BY lineup.id, fixture.kickoff_time
        "#,
    )
    .bind(lineup_id)
    .fetch_one(&mut **tx)
    .await?;

    let match_id: Uuid = row.try_get("match_id")?;
    let team_id: Uuid = row.try_get("team_id")?;
    let lineup_type: String = row.try_get("lineup_type")?;
    let snapshot_type: String = row.try_get("snapshot_type")?;
    let formation_id: Option<Uuid> = row.try_get("formation_id")?;
    let captured_at: DateTime<Utc> = row.try_get("captured_at")?;
    let kickoff_time: DateTime<Utc> = row.try_get("kickoff_time")?;
    let player_count: i64 = row.try_get("player_count")?;
    let starter_count: i64 = row.try_get("starter_count")?;

    let mut errors = Vec::<String>::new();
    let mut warnings = Vec::<String>::new();
    let model_snapshot = FORMAL_LINEUP_SNAPSHOT_TYPES.contains(&snapshot_type.as_str());

    if !model_snapshot {
        warnings.push("旧阵容未绑定可识别赛前时点，仅保留历史读取".to_string());
    }
    if !(11..=30).contains(&player_count) {
        errors.push(format!("阵容人数必须为 11–30，当前为 {player_count}"));
    }
    if starter_count != 11 {
        errors.push(format!(
            "正式模型阵容必须恰好 11 名首发，当前为 {starter_count}"
        ));
    }
    if formation_id.is_none() {
        errors.push("阵容必须绑定内置阵型 ID".to_string());
    }
    if lineup_type == "actual" {
        warnings.push("实际阵容只用于赛后复盘，不进入赛前模型输入".to_string());
    } else if model_snapshot {
        let window = lineup_snapshot_window(kickoff_time, &snapshot_type)?;
        if let Some(start_time) = window.start_time {
            if captured_at < start_time {
                errors.push(format!(
                    "记录时间早于 {snapshot_type} 窗口起点 {}",
                    start_time.to_rfc3339()
                ));
            }
        }
        if captured_at > window.cutoff_time {
            errors.push(format!(
                "记录时间晚于当前可用截止时间 {}",
                window.cutoff_time.to_rfc3339()
            ));
        }
        if captured_at >= kickoff_time {
            errors.push("预计或确认阵容的记录时间必须早于开球时间".to_string());
        }
    }

    let players = sqlx::query(
        r#"
        SELECT lineup_player.player_id, player.canonical_name,
               lineup_player.membership_override,
               EXISTS (
                   SELECT 1 FROM football.player_team_periods period
                   WHERE period.player_id = lineup_player.player_id
                     AND period.team_id = $2
                     AND period.valid_from <= $3::date
                     AND (period.valid_to IS NULL OR period.valid_to >= $3::date)
               ) AS belongs_to_team,
               EXISTS (
                   SELECT 1 FROM football.player_team_periods period
                   WHERE period.player_id = lineup_player.player_id
                     AND period.valid_from <= $3::date
                     AND (period.valid_to IS NULL OR period.valid_to >= $3::date)
               ) AS has_active_membership
        FROM football.lineup_players lineup_player
        JOIN football.players player ON player.id = lineup_player.player_id
        WHERE lineup_player.lineup_id = $1
        ORDER BY lineup_player.sequence_no, player.normalized_name
        "#,
    )
    .bind(lineup_id)
    .bind(team_id)
    .bind(kickoff_time)
    .fetch_all(&mut **tx)
    .await?;

    for player in players {
        let player_id: Uuid = player.try_get("player_id")?;
        let name: String = player.try_get("canonical_name")?;
        let membership_override: bool = player.try_get("membership_override")?;
        let belongs_to_team: bool = player.try_get("belongs_to_team")?;
        let has_active_membership: bool = player.try_get("has_active_membership")?;
        let warning = if belongs_to_team {
            None
        } else if membership_override {
            Some("已人工确认球员履历例外".to_string())
        } else if has_active_membership {
            errors.push(format!("{name} 在开球时点不属于该球队"));
            Some("球员在开球时点登记于其他球队".to_string())
        } else {
            warnings.push(format!("{name} 缺少开球时点的球队履历"));
            Some("缺少开球时点球队履历，建议补全或人工确认".to_string())
        };
        sqlx::query(
            "UPDATE football.lineup_players SET validation_warning=$3 WHERE lineup_id=$1 AND player_id=$2",
        )
        .bind(lineup_id)
        .bind(player_id)
        .bind(warning)
        .execute(&mut **tx)
        .await?;
    }

    let model_eligible = errors.is_empty() && lineup_type != "actual" && model_snapshot;
    let validation_status = if errors.is_empty() {
        "valid"
    } else {
        "invalid"
    };
    sqlx::query(
        r#"
        UPDATE football.lineups
        SET model_validation_status=$2, model_eligible=$3,
            validation_errors=$4, validation_warnings=$5, updated_at=now(),
            metadata = metadata || $6
        WHERE id=$1
        "#,
    )
    .bind(lineup_id)
    .bind(validation_status)
    .bind(model_eligible)
    .bind(json!(errors))
    .bind(json!(warnings))
    .bind(json!({
        "validation_version": "lineup-chain-v2",
        "validated_match_id": match_id,
        "validated_at": Utc::now(),
    }))
    .execute(&mut **tx)
    .await?;
    Ok(())
}

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
        if active.iter().all(|lineup| lineup.captured_at < window_start) {
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
        let match_record = self.read_match_exchange(match_id).await?;
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

    pub async fn list_team_match_lineups(
        &self,
        team_id: Uuid,
        limit: u32,
    ) -> PersistenceResult<Vec<TeamMatchLineupHistoryItem>> {
        let rows = sqlx::query(
            r#"
            SELECT lineup.id, fixture.id AS match_id, fixture.external_key AS match_key,
                   fixture.kickoff_time,
                   CASE WHEN fixture.home_team_id=$1 THEN fixture.away_team_id ELSE fixture.home_team_id END AS opponent_team_id,
                   CASE WHEN fixture.home_team_id=$1 THEN away.canonical_name ELSE home.canonical_name END AS opponent_team_name,
                   CASE WHEN fixture.home_team_id=$1 THEN 'home' ELSE 'away' END AS venue_side
            FROM football.lineups lineup
            JOIN football.matches fixture ON fixture.id=lineup.match_id
            JOIN football.teams home ON home.id=fixture.home_team_id
            JOIN football.teams away ON away.id=fixture.away_team_id
            WHERE lineup.team_id=$1
              AND lineup.history_hidden_at IS NULL
            ORDER BY fixture.kickoff_time DESC, lineup.captured_at DESC, lineup.id DESC
            LIMIT $2
            "#,
        )
        .bind(team_id)
        .bind(i64::from(limit.clamp(1, 200)))
        .fetch_all(&self.pool)
        .await?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let lineup_id: Uuid = row.try_get("id")?;
            result.push(TeamMatchLineupHistoryItem {
                match_id: row.try_get("match_id")?,
                match_key: row.try_get("match_key")?,
                opponent_team_id: row.try_get("opponent_team_id")?,
                opponent_team_name: row.try_get("opponent_team_name")?,
                venue_side: row.try_get("venue_side")?,
                kickoff_time: row.try_get("kickoff_time")?,
                lineup: self.read_lineup(lineup_id).await?,
            });
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::lineup_snapshot_window_at;
    use chrono::{Duration, TimeZone, Utc};

    #[test]
    fn latest_window_uses_reference_time_before_kickoff() {
        let kickoff = Utc.with_ymd_and_hms(2026, 7, 20, 12, 0, 0).single().unwrap();
        let reference = kickoff - Duration::hours(2);
        let window = lineup_snapshot_window_at(kickoff, "T-N", reference).unwrap();
        assert_eq!(window.start_time, None);
        assert_eq!(window.cutoff_time, reference);
    }

    #[test]
    fn fixed_window_means_within_declared_duration() {
        let kickoff = Utc.with_ymd_and_hms(2026, 7, 20, 12, 0, 0).single().unwrap();
        let reference = kickoff - Duration::hours(2);
        let window = lineup_snapshot_window_at(kickoff, "T-6h", reference).unwrap();
        assert_eq!(window.start_time, Some(kickoff - Duration::hours(6)));
        assert_eq!(window.cutoff_time, reference);
    }

    #[test]
    fn fixed_window_rejects_requests_before_window_opens() {
        let kickoff = Utc.with_ymd_and_hms(2026, 7, 20, 12, 0, 0).single().unwrap();
        let reference = kickoff - Duration::hours(7);
        assert!(lineup_snapshot_window_at(kickoff, "T-6h", reference).is_err());
    }

    #[test]
    fn latest_window_never_crosses_kickoff() {
        let kickoff = Utc.with_ymd_and_hms(2026, 7, 20, 12, 0, 0).single().unwrap();
        let reference = kickoff + Duration::hours(2);
        let window = lineup_snapshot_window_at(kickoff, "T-N", reference).unwrap();
        assert_eq!(window.cutoff_time, kickoff - Duration::seconds(1));
    }

    #[test]
    fn legacy_t90m_is_not_available_for_new_requests() {
        let kickoff = Utc.with_ymd_and_hms(2026, 7, 20, 12, 0, 0).single().unwrap();
        assert!(lineup_snapshot_window_at(kickoff, "T-90m", kickoff).is_err());
    }
}
