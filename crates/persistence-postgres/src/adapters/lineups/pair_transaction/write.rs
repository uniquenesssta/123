use super::validation::ValidatedLineupDraft;
use crate::{
    role_resolution::{
        metadata_with_role_resolution, resolve_default_tactical_role_in_tx, resolve_tactical_role,
    },
    PersistenceError, PersistenceResult,
};
use football_domain::{AvailabilityStatus, LineupDraft};
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

pub(super) async fn insert_lineup_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    draft: &LineupDraft,
    validated: &ValidatedLineupDraft,
) -> PersistenceResult<Uuid> {
    let valid_team: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM football.matches fixture
            WHERE fixture.id = $1
              AND (fixture.home_team_id = $2 OR fixture.away_team_id = $2)
        )
        "#,
    )
    .bind(draft.match_id)
    .bind(draft.team_id)
    .fetch_one(&mut **tx)
    .await?;
    if !valid_team {
        return Err(PersistenceError::InvalidState(
            "阵容球队不是该场比赛的参赛队".to_string(),
        ));
    }

    let requested_formation = draft
        .formation
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let formation_row = if let Some(formation_id) = draft.formation_id {
        sqlx::query("SELECT id, code FROM football.formations WHERE id=$1 AND is_active")
            .bind(formation_id)
            .fetch_optional(&mut **tx)
            .await?
    } else if let Some(formation) = requested_formation {
        sqlx::query(
            r#"
            SELECT id, code FROM football.formations
            WHERE is_active
              AND regexp_replace(lower(trim(code)), '\\s+', '', 'g') =
                  regexp_replace(lower(trim($1)), '\\s+', '', 'g')
            LIMIT 1
            "#,
        )
        .bind(formation)
        .fetch_optional(&mut **tx)
        .await?
    } else {
        None
    };
    if draft.formation_id.is_some() && formation_row.is_none() {
        return Err(PersistenceError::InvalidState(
            "所选阵型不存在或已停用".to_string(),
        ));
    }
    let resolved_formation_id = formation_row
        .as_ref()
        .map(|row| row.try_get::<Uuid, _>("id"))
        .transpose()?;
    let formation_text = formation_row
        .as_ref()
        .and_then(|row| row.try_get::<String, _>("code").ok())
        .or_else(|| requested_formation.map(str::to_string));

    let supersedes_lineup_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id FROM football.lineups
        WHERE match_id=$1 AND team_id=$2 AND snapshot_type=$3
          AND lineup_type=$4 AND status='active'
        ORDER BY captured_at DESC, created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(draft.match_id)
    .bind(draft.team_id)
    .bind(&validated.snapshot_type)
    .bind(draft.lineup_type.as_str())
    .fetch_optional(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE football.lineups
        SET status='superseded', updated_at=now()
        WHERE match_id=$1 AND team_id=$2 AND snapshot_type=$3
          AND lineup_type=$4 AND status='active'
        "#,
    )
    .bind(draft.match_id)
    .bind(draft.team_id)
    .bind(&validated.snapshot_type)
    .bind(draft.lineup_type.as_str())
    .execute(&mut **tx)
    .await?;

    let lineup_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO football.lineups (
            id, match_id, team_id, lineup_type, snapshot_type,
            formation, formation_id, coach_id, captured_at,
            source_document_id, source_urls, supersedes_lineup_id,
            status, quality_score, metadata
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,
            'active',$13,$14
        )
        "#,
    )
    .bind(lineup_id)
    .bind(draft.match_id)
    .bind(draft.team_id)
    .bind(draft.lineup_type.as_str())
    .bind(&validated.snapshot_type)
    .bind(formation_text)
    .bind(resolved_formation_id)
    .bind(draft.coach_id)
    .bind(draft.captured_at)
    .bind(draft.source_document_id)
    .bind(&draft.source_urls)
    .bind(supersedes_lineup_id)
    .bind(draft.quality_score)
    .bind(&draft.metadata)
    .execute(&mut **tx)
    .await?;

    for player in &draft.players {
        let position_code = player
            .position_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_uppercase);
        let inherited_role = resolve_default_tactical_role_in_tx(
            tx,
            player.player_id,
            position_code.as_deref(),
            draft.captured_at.date_naive(),
        )
        .await?;
        let role_resolution =
            resolve_tactical_role(player.role_code.as_deref(), inherited_role.as_ref());
        let player_metadata = metadata_with_role_resolution(&player.metadata, &role_resolution);
        sqlx::query(
            r#"
            INSERT INTO football.lineup_players (
                lineup_id, player_id, position_code, role_code, is_starter,
                shirt_number, expected_minutes, actual_minutes, sequence_no,
                bench_order, availability_status, starting_probability,
                membership_override, source_urls, metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            "#,
        )
        .bind(lineup_id)
        .bind(player.player_id)
        .bind(position_code)
        .bind(role_resolution.role_code.as_deref())
        .bind(player.is_starter)
        .bind(player.shirt_number)
        .bind(player.expected_minutes)
        .bind(player.actual_minutes)
        .bind(player.sequence_no)
        .bind(player.bench_order)
        .bind(player.availability_status.map(AvailabilityStatus::as_str))
        .bind(player.starting_probability)
        .bind(player.membership_override)
        .bind(&player.source_urls)
        .bind(player_metadata)
        .execute(&mut **tx)
        .await?;
    }
    crate::lineup_chain::refresh_lineup_validation_in_tx(tx, lineup_id).await?;
    crate::write_audit_event(
        tx,
        "lineup_created",
        "lineup",
        lineup_id.to_string(),
        json!({
            "match_id": draft.match_id,
            "team_id": draft.team_id,
            "lineup_type": draft.lineup_type.as_str(),
            "snapshot_type": validated.snapshot_type,
            "player_count": draft.players.len(),
            "starter_count": validated.starters,
        }),
    )
    .await?;
    Ok(lineup_id)
}
