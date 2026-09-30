use super::window::lineup_snapshot_window;
use crate::PersistenceResult;
use chrono::{DateTime, Utc};
use football_domain::FORMAL_LINEUP_SNAPSHOT_TYPES;
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

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
