use crate::{
    adapters::catalog::players::value_mapping::availability_status,
    role_resolution::{
        metadata_with_role_resolution, resolve_default_tactical_role_in_tx, resolve_tactical_role,
    },
    PersistenceError, PersistenceResult, PostgresStore,
};
use football_domain::{
    AvailabilityStatus, LineupDraft, LineupHistoryRemovalResult, LineupPairDraft, LineupPairRecord,
    LineupPlayerRecord, LineupRecord, LineupType, PlayerCatalogReferenceData,
};
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_lineup(&self, draft: &LineupDraft) -> PersistenceResult<LineupRecord> {
        let validated = validate_lineup_draft(draft)?;
        let mut tx = self.pool.begin().await?;
        let lineup_id = insert_lineup_in_tx(&mut tx, draft, &validated).await?;
        tx.commit().await?;
        self.read_lineup(lineup_id).await
    }

    pub async fn create_lineup_pair(
        &self,
        draft: &LineupPairDraft,
    ) -> PersistenceResult<LineupPairRecord> {
        if draft.home.match_id != draft.away.match_id {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须属于同一场比赛".to_string(),
            ));
        }
        if draft.home.team_id == draft.away.team_id {
            return Err(PersistenceError::InvalidState(
                "双方阵容不能使用同一支球队".to_string(),
            ));
        }
        if draft.home.snapshot_type != draft.away.snapshot_type {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须使用同一数据窗口".to_string(),
            ));
        }
        if draft.home.lineup_type != draft.away.lineup_type {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须使用同一阵容类型".to_string(),
            ));
        }
        let home_validated = validate_lineup_draft(&draft.home)?;
        let away_validated = validate_lineup_draft(&draft.away)?;
        let mut tx = self.pool.begin().await?;
        let sides = sqlx::query(
            "SELECT home_team_id, away_team_id FROM football.matches WHERE id=$1 FOR UPDATE",
        )
        .bind(draft.home.match_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛不存在".to_string()))?;
        let home_team_id: Uuid = sides.try_get("home_team_id")?;
        let away_team_id: Uuid = sides.try_get("away_team_id")?;
        if draft.home.team_id != home_team_id || draft.away.team_id != away_team_id {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须分别对应比赛主队和客队".to_string(),
            ));
        }
        let home_id = insert_lineup_in_tx(&mut tx, &draft.home, &home_validated).await?;
        let away_id = insert_lineup_in_tx(&mut tx, &draft.away, &away_validated).await?;
        crate::write_audit_event(
            &mut tx,
            "lineup_pair_created",
            "match",
            draft.home.match_id.to_string(),
            json!({
                "home_lineup_id": home_id,
                "away_lineup_id": away_id,
                "snapshot_type": home_validated.snapshot_type,
                "lineup_type": draft.home.lineup_type.as_str(),
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(LineupPairRecord {
            home: self.read_lineup(home_id).await?,
            away: self.read_lineup(away_id).await?,
        })
    }

    pub async fn list_lineups(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> PersistenceResult<Vec<LineupRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                lineup.id, lineup.match_id, fixture.external_key AS match_key,
                lineup.team_id, team.canonical_name AS team_name,
                lineup.lineup_type, lineup.snapshot_type,
                lineup.formation, lineup.formation_id,
                formation.code AS formation_code, formation.name AS formation_name,
                lineup.coach_id, coach.canonical_name AS coach_name,
                lineup.captured_at, lineup.status, lineup.quality_score,
                lineup.source_urls, lineup.supersedes_lineup_id,
                lineup.model_validation_status, lineup.model_eligible,
                lineup.validation_errors, lineup.validation_warnings,
                count(player.player_id) AS player_count,
                count(player.player_id) FILTER (WHERE player.is_starter) AS starter_count
            FROM football.lineups lineup
            JOIN football.matches fixture ON fixture.id = lineup.match_id
            JOIN football.teams team ON team.id = lineup.team_id
            LEFT JOIN football.formations formation ON formation.id = lineup.formation_id
            LEFT JOIN football.coaches coach ON coach.id = lineup.coach_id
            LEFT JOIN football.lineup_players player ON player.lineup_id = lineup.id
            WHERE ($1::uuid IS NULL OR lineup.match_id = $1)
              AND lineup.history_hidden_at IS NULL
            GROUP BY lineup.id, fixture.external_key, team.canonical_name,
                     formation.code, formation.name, coach.canonical_name
            ORDER BY lineup.captured_at DESC, lineup.id DESC
            LIMIT $2
            "#,
        )
        .bind(match_id)
        .bind(i64::from(limit.clamp(1, 200)))
        .fetch_all(&self.pool)
        .await?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(lineup_record_from_row(&row, Vec::new())?);
        }
        Ok(result)
    }

    pub async fn read_lineup(&self, lineup_id: Uuid) -> PersistenceResult<LineupRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                lineup.id, lineup.match_id, fixture.external_key AS match_key,
                lineup.team_id, team.canonical_name AS team_name,
                lineup.lineup_type, lineup.snapshot_type,
                lineup.formation, lineup.formation_id,
                formation.code AS formation_code, formation.name AS formation_name,
                lineup.coach_id, coach.canonical_name AS coach_name,
                lineup.captured_at, lineup.status, lineup.quality_score,
                lineup.source_urls, lineup.supersedes_lineup_id,
                lineup.model_validation_status, lineup.model_eligible,
                lineup.validation_errors, lineup.validation_warnings,
                count(player.player_id) AS player_count,
                count(player.player_id) FILTER (WHERE player.is_starter) AS starter_count
            FROM football.lineups lineup
            JOIN football.matches fixture ON fixture.id = lineup.match_id
            JOIN football.teams team ON team.id = lineup.team_id
            LEFT JOIN football.formations formation ON formation.id = lineup.formation_id
            LEFT JOIN football.coaches coach ON coach.id = lineup.coach_id
            LEFT JOIN football.lineup_players player ON player.lineup_id = lineup.id
            WHERE lineup.id = $1
            GROUP BY lineup.id, fixture.external_key, team.canonical_name,
                     formation.code, formation.name, coach.canonical_name
            "#,
        )
        .bind(lineup_id)
        .fetch_one(&self.pool)
        .await?;
        let player_rows = sqlx::query(
            r#"
            SELECT player.player_id, football_player.canonical_name AS player_name,
                   player.position_code,
                   COALESCE(NULLIF(btrim(player.role_code), ''), inherited_role.default_role_code)
                       AS role_code,
                   CASE
                     WHEN player.metadata->>'role_origin' IN (
                       'lineup_override', 'player_position_default', 'missing'
                     ) THEN player.metadata->>'role_origin'
                     WHEN NULLIF(btrim(player.role_code), '') IS NOT NULL THEN 'lineup_override'
                     WHEN inherited_role.default_role_code IS NOT NULL THEN 'player_position_default'
                     ELSE 'missing'
                   END AS role_origin,
                   CASE
                     WHEN player.metadata->>'role_origin' = 'player_position_default'
                       THEN COALESCE(
                         NULLIF(btrim(player.metadata->>'role_source_position_code'), ''),
                         inherited_role.position_code
                       )
                     WHEN player.metadata->>'role_origin' IN ('lineup_override', 'missing')
                       THEN NULL
                     WHEN NULLIF(btrim(player.role_code), '') IS NOT NULL THEN NULL
                     WHEN inherited_role.default_role_code IS NOT NULL
                       THEN inherited_role.position_code
                     ELSE NULL
                   END AS role_source_position_code,
                   player.is_starter,
                   player.shirt_number, player.expected_minutes, player.actual_minutes,
                   player.sequence_no, player.bench_order, player.availability_status,
                   player.starting_probability, player.membership_override,
                   player.source_urls, player.validation_warning
            FROM football.lineup_players player
            JOIN football.lineups lineup ON lineup.id = player.lineup_id
            JOIN football.players football_player ON football_player.id = player.player_id
            LEFT JOIN LATERAL (
                SELECT position.default_role_code, position.position_code
                FROM football.player_positions position
                WHERE position.player_id = player.player_id
                  AND position.default_role_code IS NOT NULL
                  AND btrim(position.default_role_code) <> ''
                  AND (position.valid_from IS NULL OR position.valid_from <= lineup.captured_at::date)
                  AND (position.valid_to IS NULL OR position.valid_to >= lineup.captured_at::date)
                ORDER BY
                  CASE
                    WHEN player.position_code IS NOT NULL
                     AND upper(position.position_code) = upper(player.position_code) THEN 0
                    WHEN position.is_primary THEN 1
                    ELSE 2
                  END,
                  position.proficiency DESC,
                  position.valid_from DESC NULLS LAST,
                  position.id DESC
                LIMIT 1
            ) inherited_role ON true
            WHERE player.lineup_id = $1
            ORDER BY player.is_starter DESC, player.sequence_no,
                     player.bench_order NULLS LAST, football_player.normalized_name
            "#,
        )
        .bind(lineup_id)
        .fetch_all(&self.pool)
        .await?;
        let players = player_rows
            .iter()
            .map(lineup_player_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        lineup_record_from_row(&row, players)
    }

    pub async fn remove_lineup_history(
        &self,
        lineup_id: Uuid,
        reason: Option<&str>,
    ) -> PersistenceResult<LineupHistoryRemovalResult> {
        let reason = reason
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("用户从阵容历史中删除");
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            SELECT id, match_id, team_id, snapshot_type, lineup_type, status,
                   history_hidden_at
            FROM football.lineups
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(lineup_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("阵容版本不存在".to_string()))?;

        if row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("history_hidden_at")?
            .is_some()
        {
            return Err(PersistenceError::InvalidState(
                "阵容版本已经从历史列表隐藏".to_string(),
            ));
        }

        let match_id: Uuid = row.try_get("match_id")?;
        let team_id: Uuid = row.try_get("team_id")?;
        let snapshot_type: String = row.try_get("snapshot_type")?;
        let lineup_type: String = row.try_get("lineup_type")?;
        let status: String = row.try_get("status")?;

        let referenced: bool = sqlx::query_scalar(
            r#"
            SELECT
                EXISTS (SELECT 1 FROM football.lineups WHERE supersedes_lineup_id = $1)
                OR EXISTS (SELECT 1 FROM feature.match_player_contributions WHERE lineup_id = $1)
                OR EXISTS (SELECT 1 FROM feature.snapshots WHERE input_payload::text LIKE '%' || $1::text || '%')
                OR EXISTS (SELECT 1 FROM model.runs WHERE input_payload::text LIKE '%' || $1::text || '%')
            "#,
        )
        .bind(lineup_id)
        .fetch_one(&mut *tx)
        .await?;

        let removal_mode = if referenced {
            sqlx::query(
                r#"
                UPDATE football.lineups
                SET history_hidden_at = now(),
                    history_hidden_reason = $2,
                    status = CASE WHEN status = 'active' THEN 'withdrawn' ELSE status END,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(lineup_id)
            .bind(reason)
            .execute(&mut *tx)
            .await?;
            "archived"
        } else {
            sqlx::query("DELETE FROM football.lineups WHERE id = $1")
                .bind(lineup_id)
                .execute(&mut *tx)
                .await?;
            "deleted"
        };

        let restored_lineup_id = if status == "active" {
            let candidate = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT id
                FROM football.lineups
                WHERE match_id = $1
                  AND team_id = $2
                  AND snapshot_type = $3
                  AND lineup_type = $4
                  AND status = 'superseded'
                  AND history_hidden_at IS NULL
                ORDER BY captured_at DESC, created_at DESC, id DESC
                LIMIT 1
                FOR UPDATE
                "#,
            )
            .bind(match_id)
            .bind(team_id)
            .bind(&snapshot_type)
            .bind(&lineup_type)
            .fetch_optional(&mut *tx)
            .await?;
            if let Some(candidate_id) = candidate {
                sqlx::query(
                    "UPDATE football.lineups SET status='active', updated_at=now() WHERE id=$1",
                )
                .bind(candidate_id)
                .execute(&mut *tx)
                .await?;
                Some(candidate_id)
            } else {
                None
            }
        } else {
            None
        };

        crate::write_audit_event(
            &mut tx,
            "lineup_history_removed",
            "lineup",
            lineup_id.to_string(),
            json!({
                "removal_mode": removal_mode,
                "reason": reason,
                "restored_lineup_id": restored_lineup_id,
                "match_id": match_id,
                "team_id": team_id,
                "snapshot_type": snapshot_type,
                "lineup_type": lineup_type,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(LineupHistoryRemovalResult {
            lineup_id,
            removal_mode: removal_mode.to_string(),
            restored_lineup_id,
        })
    }

    pub async fn player_catalog_reference_data(
        &self,
    ) -> PersistenceResult<PlayerCatalogReferenceData> {
        let teams = self.list_team_options(None, 500).await?;
        let season_team_memberships = self.list_season_team_memberships().await?;
        let formations = self.list_formations(true).await?;
        let providers = self.list_data_providers().await?;
        let positions = self.list_positions().await?;
        let ability_dimensions = self.list_ability_dimensions().await?;
        let dynamic_tag_definitions = self.list_dynamic_tag_definitions().await?;
        let upcoming_matches = self.list_upcoming_matches(100).await?;
        let managed_matches = self.list_managed_matches(300).await?;
        Ok(PlayerCatalogReferenceData {
            teams,
            season_team_memberships,
            formations,
            providers,
            positions,
            ability_dimensions,
            dynamic_tag_definitions,
            upcoming_matches,
            managed_matches,
        })
    }
}

struct ValidatedLineupDraft {
    snapshot_type: String,
    starters: usize,
}

fn validate_lineup_draft(draft: &LineupDraft) -> PersistenceResult<ValidatedLineupDraft> {
    if !(1..=30).contains(&draft.players.len()) {
        return Err(PersistenceError::InvalidState(
            "阵容球员数量必须位于 1–30".to_string(),
        ));
    }
    let unique_players: HashSet<Uuid> = draft
        .players
        .iter()
        .map(|player| player.player_id)
        .collect();
    if unique_players.len() != draft.players.len() {
        return Err(PersistenceError::InvalidState(
            "同一阵容中存在重复球员".to_string(),
        ));
    }
    let starters = draft
        .players
        .iter()
        .filter(|player| player.is_starter)
        .count();
    if starters != 11 {
        return Err(PersistenceError::InvalidState(format!(
            "正式阵容必须恰好 11 名首发，当前为 {starters} 名",
        )));
    }
    if draft
        .quality_score
        .is_some_and(|value| !(0.0..=1.0).contains(&value))
    {
        return Err(PersistenceError::InvalidState(
            "阵容质量分必须位于 0–1".to_string(),
        ));
    }
    let snapshot_type =
        crate::lineup_chain::normalize_lineup_snapshot_type(&draft.snapshot_type)?.to_string();
    for player in &draft.players {
        if player
            .shirt_number
            .is_some_and(|number| !(0..=99).contains(&number))
        {
            return Err(PersistenceError::InvalidState(
                "阵容球衣号码必须位于 0–99".to_string(),
            ));
        }
        if player
            .expected_minutes
            .is_some_and(|minutes| !(0..=150).contains(&minutes))
            || player
                .actual_minutes
                .is_some_and(|minutes| !(0..=150).contains(&minutes))
        {
            return Err(PersistenceError::InvalidState(
                "阵容分钟数必须位于 0–150".to_string(),
            ));
        }
        if player.sequence_no < 0 {
            return Err(PersistenceError::InvalidState(
                "阵容排序号不能为负数".to_string(),
            ));
        }
        if player
            .bench_order
            .is_some_and(|value| !(1..=99).contains(&value))
        {
            return Err(PersistenceError::InvalidState(
                "替补顺序必须位于 1–99".to_string(),
            ));
        }
        if player
            .starting_probability
            .is_some_and(|value| !(0.0..=1.0).contains(&value))
        {
            return Err(PersistenceError::InvalidState(
                "首发概率必须位于 0–1".to_string(),
            ));
        }
    }
    Ok(ValidatedLineupDraft {
        snapshot_type,
        starters,
    })
}

async fn insert_lineup_in_tx(
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

fn lineup_type(value: &str) -> PersistenceResult<LineupType> {
    match value {
        "expected" => Ok(LineupType::Expected),
        "confirmed" => Ok(LineupType::Confirmed),
        "actual" => Ok(LineupType::Actual),
        other => Err(PersistenceError::InvalidState(format!(
            "未知阵容类型：{other}"
        ))),
    }
}

pub(crate) fn lineup_player_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<LineupPlayerRecord> {
    let availability: Option<String> = row.try_get("availability_status")?;
    Ok(LineupPlayerRecord {
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        position_code: row.try_get("position_code")?,
        role_code: row.try_get("role_code")?,
        role_origin: row.try_get("role_origin")?,
        role_source_position_code: row.try_get("role_source_position_code")?,
        is_starter: row.try_get("is_starter")?,
        shirt_number: row.try_get("shirt_number")?,
        expected_minutes: row.try_get("expected_minutes")?,
        actual_minutes: row.try_get("actual_minutes")?,
        sequence_no: row.try_get("sequence_no")?,
        bench_order: row.try_get("bench_order")?,
        availability_status: availability
            .as_deref()
            .map(availability_status)
            .transpose()?,
        starting_probability: row.try_get("starting_probability")?,
        membership_override: row.try_get("membership_override")?,
        source_urls: row.try_get("source_urls")?,
        validation_warning: row.try_get("validation_warning")?,
    })
}

pub(crate) fn lineup_record_from_row(
    row: &sqlx::postgres::PgRow,
    players: Vec<LineupPlayerRecord>,
) -> PersistenceResult<LineupRecord> {
    let lineup_type_value: String = row.try_get("lineup_type")?;
    Ok(LineupRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        lineup_type: lineup_type(&lineup_type_value)?,
        snapshot_type: row.try_get("snapshot_type")?,
        formation: row.try_get("formation")?,
        formation_id: row.try_get("formation_id")?,
        formation_code: row.try_get("formation_code")?,
        formation_name: row.try_get("formation_name")?,
        coach_id: row.try_get("coach_id")?,
        coach_name: row.try_get("coach_name")?,
        captured_at: row.try_get("captured_at")?,
        status: row.try_get("status")?,
        quality_score: row.try_get("quality_score")?,
        source_urls: row.try_get("source_urls")?,
        supersedes_lineup_id: row.try_get("supersedes_lineup_id")?,
        model_validation_status: row.try_get("model_validation_status")?,
        model_eligible: row.try_get("model_eligible")?,
        validation_errors: serde_json::from_value(row.try_get("validation_errors")?)?,
        validation_warnings: serde_json::from_value(row.try_get("validation_warnings")?)?,
        player_count: row.try_get("player_count")?,
        starter_count: row.try_get("starter_count")?,
        players,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::catalog::players::{
        normalization::normalize_name,
        value_mapping::{player_status, preferred_foot},
    };
    use football_domain::{PlayerStatus, PreferredFoot};

    #[test]
    fn normalize_name_collapses_case_and_spacing() {
        assert_eq!(normalize_name("  Son   Heung-Min  "), "son heung-min");
    }

    #[test]
    fn persisted_enums_round_trip() {
        assert_eq!(preferred_foot("left").unwrap(), PreferredFoot::Left);
        assert_eq!(player_status("active").unwrap(), PlayerStatus::Active);
        assert_eq!(
            availability_status("returning").unwrap(),
            AvailabilityStatus::Returning
        );
        assert_eq!(lineup_type("confirmed").unwrap(), LineupType::Confirmed);
    }

    #[test]
    fn unknown_persisted_enum_is_rejected() {
        assert!(preferred_foot("ambidextrous").is_err());
        assert!(availability_status("missing").is_err());
        assert!(lineup_type("draft").is_err());
    }
}
