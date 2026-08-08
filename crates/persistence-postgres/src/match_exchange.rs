use crate::{
    role_resolution::{
        metadata_with_role_resolution, resolve_default_tactical_role_in_tx, resolve_tactical_role,
    },
    PersistenceError, PersistenceResult, PostgresStore,
};
use chrono::{DateTime, NaiveDate, Utc};
use football_domain::{
    AiMatchPackageContext, AiMatchPlayerContext, AvailabilityStatus, CoachListQuery,
    MatchLineupExportData, MatchLineupPlayerReference, MatchRecord, MatchStatus,
    PlayerMatchContributionRequest, SpreadsheetAction, SpreadsheetConflictCandidate,
    SpreadsheetEntityType, SpreadsheetImportCommitResult, SpreadsheetImportCounts,
    SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution,
    SpreadsheetImportRow, SpreadsheetParsedWorkbook, SpreadsheetRowStatus,
};
use serde_json::{json, Map, Value};
use sqlx::{Postgres, Row, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

const IMPORT_TYPE: &str = "match_lineup_xlsx";

#[derive(Debug)]
struct Validation {
    status: SpreadsheetRowStatus,
    message: Option<String>,
    payload: Value,
    matched_entity_id: Option<Uuid>,
    candidates: Vec<SpreadsheetConflictCandidate>,
}

#[derive(Debug, Default)]
struct ApplyOutcome {
    was_update: bool,
    ended_previous: u64,
    lineup_id: Option<Uuid>,
}

impl PostgresStore {
    /// Reads one managed match through the persistence crate's public read API.
    ///
    /// Internal exchange workflows continue to share `read_match_exchange`; callers in
    /// other crates must use this stable boundary instead of depending on crate-private
    /// implementation details.
    pub async fn read_match(&self, match_id: Uuid) -> PersistenceResult<MatchRecord> {
        self.read_match_exchange(match_id).await
    }

    pub async fn match_lineup_export_data(
        &self,
        match_id: Option<Uuid>,
    ) -> PersistenceResult<MatchLineupExportData> {
        let selected_match = match match_id {
            Some(id) => Some(self.read_match_exchange(id).await?),
            None => None,
        };
        let lineups = if let Some(id) = match_id {
            self.hydrated_active_lineups(id).await?
        } else {
            Vec::new()
        };
        let competitions = self.list_competitions().await?;
        let references = self.player_catalog_reference_data().await?;
        let teams = references.teams.clone();
        let formations = references.formations.clone();
        let coaches = self.list_coaches(&CoachListQuery::default()).await?;
        let reference_time = selected_match
            .as_ref()
            .map(|item| item.kickoff_time)
            .unwrap_or_else(Utc::now);
        let players = if match_id.is_some() {
            self.match_player_references(match_id, reference_time)
                .await?
        } else {
            Vec::new()
        };
        let dynamic_tags = if match_id.is_some() {
            let mut tags = Vec::new();
            for player in &players {
                tags.extend(
                    self.list_player_dynamic_tags(player.player_id, reference_time)
                        .await?,
                );
            }
            tags
        } else {
            Vec::new()
        };
        Ok(MatchLineupExportData {
            selected_match,
            lineups,
            competitions,
            teams,
            formations,
            coaches,
            positions: references.positions,
            players,
            dynamic_tag_definitions: references.dynamic_tag_definitions,
            dynamic_tags,
        })
    }

    pub async fn preview_match_lineup_import(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let batch_id = Uuid::new_v4();
        let mut preview_rows = Vec::with_capacity(parsed.rows.len());
        for raw in &parsed.rows {
            let validation = self
                .validate_match_exchange_row(raw.entity_type, raw.action, &raw.values, mode)
                .await;
            let row = match validation {
                Ok(validation) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: raw.action,
                    status: validation.status,
                    message: validation.message,
                    payload: validation.payload,
                    matched_entity_id: validation.matched_entity_id,
                    conflict_candidates: validation.candidates,
                },
                Err(error) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: raw.action,
                    status: SpreadsheetRowStatus::Error,
                    message: Some(error.to_string()),
                    payload: raw.values.clone(),
                    matched_entity_id: None,
                    conflict_candidates: Vec::new(),
                },
            };
            preview_rows.push(row);
        }
        let counts = count_rows(&preview_rows);
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO catalog.import_batches (
                id, import_type, status, source_file_name, source_sha256,
                import_mode, started_at, skipped_count, error_count, metadata
            ) VALUES ($1, $2, 'pending', $3, $4, $5, now(), $6, $7, $8)
            "#,
        )
        .bind(batch_id)
        .bind(IMPORT_TYPE)
        .bind(&parsed.source_file_name)
        .bind(&parsed.source_sha256)
        .bind(mode_text(mode))
        .bind(counts.skipped as i64)
        .bind((counts.error + counts.conflict) as i64)
        .bind(json!({"format_version": parsed.format_version, "preview_counts": counts}))
        .execute(&mut *tx)
        .await
        .map_err(map_duplicate_import)?;
        for row in &preview_rows {
            insert_row(&mut tx, batch_id, row).await?;
        }
        tx.commit().await?;
        Ok(SpreadsheetImportPreview {
            batch_id,
            source_file_name: parsed.source_file_name.clone(),
            source_sha256: parsed.source_sha256.clone(),
            import_mode: mode,
            counts,
            rows: preview_rows,
            created_at: Utc::now(),
        })
    }

    pub async fn read_match_lineup_import_preview(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let batch = sqlx::query(
            "SELECT source_file_name, source_sha256, import_mode, started_at FROM catalog.import_batches WHERE id=$1 AND import_type=$2",
        ).bind(batch_id).bind(IMPORT_TYPE).fetch_one(&self.pool).await?;
        let rows = sqlx::query(
            "SELECT id,sheet_name,row_number,entity_type,requested_action,status,message,payload,matched_entity_id,conflict_candidates FROM catalog.import_rows WHERE batch_id=$1 ORDER BY row_number,sheet_name,id",
        ).bind(batch_id).fetch_all(&self.pool).await?.iter().map(row_from_db).collect::<PersistenceResult<Vec<_>>>()?;
        Ok(SpreadsheetImportPreview {
            batch_id,
            source_file_name: batch
                .try_get::<Option<String>, _>("source_file_name")?
                .unwrap_or_default(),
            source_sha256: batch
                .try_get::<Option<String>, _>("source_sha256")?
                .unwrap_or_default(),
            import_mode: parse_mode(
                batch
                    .try_get::<Option<String>, _>("import_mode")?
                    .as_deref(),
            )?,
            counts: count_rows(&rows),
            rows,
            created_at: batch
                .try_get::<Option<DateTime<Utc>>, _>("started_at")?
                .unwrap_or_else(Utc::now),
        })
    }

    pub async fn resolve_match_lineup_import_conflict(
        &self,
        batch_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let (entity, action, mode, payload) = {
            let mut tx = self.pool.begin().await?;
            let batch = sqlx::query(
                "SELECT status, import_mode FROM catalog.import_batches WHERE id=$1 AND import_type=$2 FOR UPDATE",
            )
            .bind(batch_id)
            .bind(IMPORT_TYPE)
            .fetch_one(&mut *tx)
            .await?;
            let batch_status: String = batch.try_get("status")?;
            if batch_status != "pending" {
                return Err(PersistenceError::InvalidState(
                    "该导入批次已不能修改".to_string(),
                ));
            }
            let mode = parse_mode(
                batch
                    .try_get::<Option<String>, _>("import_mode")?
                    .as_deref(),
            )?;
            let row = sqlx::query(
                "SELECT entity_type,requested_action,status,payload,conflict_candidates FROM catalog.import_rows WHERE id=$1 AND batch_id=$2 FOR UPDATE",
            )
            .bind(resolution.row_id)
            .bind(batch_id)
            .fetch_one(&mut *tx)
            .await?;
            let status: String = row.try_get("status")?;
            if status != "conflict" {
                return Err(PersistenceError::InvalidState(
                    "只有冲突行可以处理".to_string(),
                ));
            }
            if resolution.skip {
                sqlx::query(
                    "UPDATE catalog.import_rows SET status='skip',message='用户选择跳过',matched_entity_id=NULL,conflict_candidates='[]'::jsonb WHERE id=$1",
                )
                .bind(resolution.row_id)
                .execute(&mut *tx)
                .await?;
                tx.commit().await?;
                return self.read_match_lineup_import_preview(batch_id).await;
            }

            let selected = resolution
                .selected_entity_id
                .ok_or_else(|| PersistenceError::InvalidState("请选择候选记录".to_string()))?;
            let candidates: Vec<SpreadsheetConflictCandidate> =
                serde_json::from_value(row.try_get("conflict_candidates")?)?;
            if !candidates.iter().any(|item| item.entity_id == selected) {
                return Err(PersistenceError::InvalidState(
                    "所选记录不在冲突候选中".to_string(),
                ));
            }
            let entity = parse_entity(&row.try_get::<String, _>("entity_type")?)?;
            let action = parse_action(&row.try_get::<String, _>("requested_action")?)?;
            let mut payload: Value = row.try_get("payload")?;
            let object = payload
                .as_object_mut()
                .ok_or_else(|| PersistenceError::InvalidState("导入行无效".to_string()))?;
            let prefix = object
                .get("_conflict_prefix")
                .and_then(Value::as_str)
                .unwrap_or(match entity {
                    SpreadsheetEntityType::Match => "match",
                    SpreadsheetEntityType::Lineup => "team",
                    SpreadsheetEntityType::LineupPlayer
                    | SpreadsheetEntityType::PlayerDynamicTag => "player",
                    _ => "entity",
                });
            let target_field = match prefix {
                "match" => "match_id",
                "home_team" => "home_team_id",
                "away_team" => "away_team_id",
                "team" => "team_id",
                "player" => "player_id",
                _ => {
                    return Err(PersistenceError::InvalidState(format!(
                        "未知冲突类型：{prefix}"
                    )))
                }
            };
            object.insert(target_field.to_string(), json!(selected));
            object.remove("_conflict_prefix");
            tx.commit().await?;
            (entity, action, mode, payload)
        };

        let validation = match self
            .validate_match_exchange_row(entity, action, &payload, mode)
            .await
        {
            Ok(validation) => validation,
            Err(error_value) => error(payload, error_value.to_string()),
        };

        let mut tx = self.pool.begin().await?;
        let batch_status: String = sqlx::query_scalar(
            "SELECT status FROM catalog.import_batches WHERE id=$1 AND import_type=$2 FOR UPDATE",
        )
        .bind(batch_id)
        .bind(IMPORT_TYPE)
        .fetch_one(&mut *tx)
        .await?;
        if batch_status != "pending" {
            return Err(PersistenceError::InvalidState(
                "该导入批次已不能修改".to_string(),
            ));
        }
        let row_status: String = sqlx::query_scalar(
            "SELECT status FROM catalog.import_rows WHERE id=$1 AND batch_id=$2 FOR UPDATE",
        )
        .bind(resolution.row_id)
        .bind(batch_id)
        .fetch_one(&mut *tx)
        .await?;
        if row_status != "conflict" {
            return Err(PersistenceError::InvalidState(
                "该冲突行已被其他操作修改".to_string(),
            ));
        }
        sqlx::query(
            "UPDATE catalog.import_rows SET status=$2,message=$3,payload=$4,matched_entity_id=$5,conflict_candidates=$6 WHERE id=$1",
        )
        .bind(resolution.row_id)
        .bind(validation.status.as_str())
        .bind(&validation.message)
        .bind(&validation.payload)
        .bind(validation.matched_entity_id)
        .bind(serde_json::to_value(&validation.candidates)?)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.read_match_lineup_import_preview(batch_id).await
    }

    pub async fn commit_match_lineup_import(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportCommitResult> {
        let mut tx = self.pool.begin().await?;
        let batch = sqlx::query(
            "SELECT status FROM catalog.import_batches WHERE id=$1 AND import_type=$2 FOR UPDATE",
        )
        .bind(batch_id)
        .bind(IMPORT_TYPE)
        .fetch_one(&mut *tx)
        .await?;
        let status: String = batch.try_get("status")?;
        if status != "pending" {
            return Err(PersistenceError::InvalidState(
                "导入批次不是待确认状态".to_string(),
            ));
        }
        let blocking: i64 = sqlx::query_scalar("SELECT COUNT(*)::bigint FROM catalog.import_rows WHERE batch_id=$1 AND status IN ('conflict','error')")
            .bind(batch_id).fetch_one(&mut *tx).await?;
        if blocking > 0 {
            return Err(PersistenceError::InvalidState(
                "仍存在冲突或错误，不能导入".to_string(),
            ));
        }
        sqlx::query("UPDATE catalog.import_batches SET status='running' WHERE id=$1")
            .bind(batch_id)
            .execute(&mut *tx)
            .await?;
        let rows = sqlx::query("SELECT id,entity_type,status,payload,matched_entity_id FROM catalog.import_rows WHERE batch_id=$1 AND status IN ('ready_add','ready_update','skip') ORDER BY CASE entity_type WHEN 'match' THEN 1 WHEN 'lineup' THEN 2 WHEN 'lineup_player' THEN 3 WHEN 'player_dynamic_tag' THEN 4 ELSE 9 END,row_number,id")
            .bind(batch_id).fetch_all(&mut *tx).await?;
        let mut match_keys = HashMap::new();
        let mut lineup_keys = HashMap::new();
        let mut inserted = 0u64;
        let mut updated = 0u64;
        let mut skipped = 0u64;
        let mut ended_previous = 0u64;
        let mut affected_lineups = Vec::<Uuid>::new();
        for row in rows {
            let row_id: Uuid = row.try_get("id")?;
            let row_status: String = row.try_get("status")?;
            if row_status == "skip" {
                skipped += 1;
                continue;
            }
            let entity = parse_entity(&row.try_get::<String, _>("entity_type")?)?;
            let payload: Value = row.try_get("payload")?;
            let values = payload
                .as_object()
                .ok_or_else(|| PersistenceError::InvalidState("导入内容无效".to_string()))?;
            let matched: Option<Uuid> = row.try_get("matched_entity_id")?;
            let outcome = apply_match_exchange_row(
                &mut tx,
                entity,
                values,
                matched,
                &mut match_keys,
                &mut lineup_keys,
            )
            .await?;
            if outcome.was_update {
                updated += 1
            } else {
                inserted += 1
            }
            ended_previous += outcome.ended_previous;
            if let Some(lineup_id) = outcome.lineup_id {
                affected_lineups.push(lineup_id);
            }
            sqlx::query(
                "UPDATE catalog.import_rows SET status='imported',imported_at=now() WHERE id=$1",
            )
            .bind(row_id)
            .execute(&mut *tx)
            .await?;
        }
        affected_lineups.sort_unstable();
        affected_lineups.dedup();
        for lineup_id in affected_lineups {
            crate::lineup_chain::refresh_lineup_validation_in_tx(&mut tx, lineup_id).await?;
        }
        let finished_at = Utc::now();
        sqlx::query("UPDATE catalog.import_batches SET status='succeeded',finished_at=$2,inserted_count=$3,updated_count=$4,skipped_count=$5,error_count=0 WHERE id=$1")
            .bind(batch_id).bind(finished_at).bind(inserted as i64).bind(updated as i64).bind(skipped as i64).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO audit.events(id,event_type,entity_type,entity_id,payload) VALUES($1,'match_lineup_import_committed','import_batch',$2,$3)")
            .bind(Uuid::new_v4()).bind(batch_id.to_string()).bind(json!({"inserted":inserted,"updated":updated,"ended_previous":ended_previous,"skipped":skipped})).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(SpreadsheetImportCommitResult {
            batch_id,
            inserted_count: inserted,
            updated_count: updated,
            ended_previous_count: ended_previous,
            skipped_count: skipped,
            error_count: 0,
            finished_at,
        })
    }

    pub async fn ai_match_package_context(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<AiMatchPackageContext> {
        let match_record = self.read_match_exchange(match_id).await?;
        let competition = match match_record.competition_id {
            Some(id) => self
                .list_competitions()
                .await?
                .into_iter()
                .find(|item| item.id == id),
            None => None,
        };
        let lineups = self.hydrated_active_lineups(match_id).await?;
        let mut players = Vec::new();
        for lineup in &lineups {
            for lineup_player in &lineup.players {
                if players
                    .iter()
                    .any(|item: &AiMatchPlayerContext| item.player.id == lineup_player.player_id)
                {
                    continue;
                }
                let detail = self.read_player(lineup_player.player_id).await?;
                let opponent_team_id = if lineup.team_id == match_record.home_team_id {
                    Some(match_record.away_team_id)
                } else {
                    Some(match_record.home_team_id)
                };
                let contribution = self
                    .calculate_player_match_contribution(&PlayerMatchContributionRequest {
                        player_id: lineup_player.player_id,
                        match_id: Some(match_id),
                        competition_id: match_record.competition_id,
                        position_code: lineup_player.position_code.clone(),
                        role_code: lineup_player.role_code.clone(),
                        role_origin: Some(lineup_player.role_origin.clone()),
                        role_source_position_code: lineup_player.role_source_position_code.clone(),
                        opponent_team_id,
                        as_of: match_record.kickoff_time,
                        data_cutoff_time: Some(Utc::now()),
                        expected_minutes: lineup_player.expected_minutes,
                    })
                    .await
                    .ok();
                players.push(AiMatchPlayerContext {
                    player: detail.player,
                    team_id: Some(lineup.team_id),
                    team_name: Some(lineup.team_name.clone()),
                    lineup_status: if lineup_player.is_starter {
                        "starter"
                    } else {
                        "bench"
                    }
                    .to_string(),
                    tactical_role_code: lineup_player.role_code.clone(),
                    tactical_role_origin: lineup_player.role_origin.clone(),
                    tactical_role_source_position_code: lineup_player
                        .role_source_position_code
                        .clone(),
                    // 兼容旧 AI 包字段；语义已纠正为战术角色，不再承载首发/替补状态。
                    lineup_role: lineup_player.role_code.clone(),
                    expected_minutes: lineup_player.expected_minutes,
                    ability_profile: detail.ability_profile,
                    availability: detail.availability,
                    dynamic_tags: self
                        .list_player_dynamic_tags(
                            lineup_player.player_id,
                            match_record.kickoff_time,
                        )
                        .await?,
                    contribution,
                });
            }
        }
        let lineup_count = lineups.len();
        let player_context_count = players.len();
        let inherited_role_count = players
            .iter()
            .filter(|item| item.tactical_role_origin == "player_position_default")
            .count();
        let overridden_role_count = players
            .iter()
            .filter(|item| item.tactical_role_origin == "lineup_override")
            .count();
        let missing_role_count = players
            .iter()
            .filter(|item| item.tactical_role_origin == "missing")
            .count();
        Ok(AiMatchPackageContext {
            match_record,
            competition,
            lineups,
            players,
            generated_at: Utc::now(),
            data_quality: json!({
                "lineup_count": lineup_count,
                "player_context_count": player_context_count,
                "inherited_role_count": inherited_role_count,
                "overridden_role_count": overridden_role_count,
                "missing_role_count": missing_role_count,
            }),
        })
    }

    async fn validate_match_exchange_row(
        &self,
        entity: SpreadsheetEntityType,
        action: SpreadsheetAction,
        values: &Value,
        mode: SpreadsheetImportMode,
    ) -> PersistenceResult<Validation> {
        if action == SpreadsheetAction::Skip {
            return Ok(skip(values.clone(), "Excel 行标记为 skip"));
        }
        let mut payload = values
            .as_object()
            .cloned()
            .ok_or_else(|| PersistenceError::InvalidState("Excel 行内容不是对象".to_string()))?;
        match entity {
            SpreadsheetEntityType::Match => {
                required(&payload, "match_key")?;
                required(&payload, "kickoff_time")?;
                let competition = resolve_competition(&self.pool, &payload).await?;
                payload.insert("_resolved_competition_id".to_string(), json!(competition));
                let home =
                    resolve_team_value(&self.pool, &payload, "home_team_id", "home_team_name")
                        .await?;
                if let Some(result) = resolution_validation(&mut payload, home, "home_team") {
                    return Ok(result);
                }
                let away =
                    resolve_team_value(&self.pool, &payload, "away_team_id", "away_team_name")
                        .await?;
                if let Some(result) = resolution_validation(&mut payload, away, "away_team") {
                    return Ok(result);
                }
                if payload_uuid(&payload, "_resolved_home_team_id")?
                    == payload_uuid(&payload, "_resolved_away_team_id")?
                {
                    return Ok(error(Value::Object(payload), "主客队不能相同"));
                }
                required_datetime(&payload, "kickoff_time")?;
                let existing = find_match(&self.pool, &payload).await?;
                decision(action, mode, Value::Object(payload), existing, "match")
            }
            SpreadsheetEntityType::Lineup => {
                required(&payload, "lineup_key")?;
                required(&payload, "match_key")?;
                validate_lineup_type(&payload)?;
                let snapshot_type = default_text(&payload, "snapshot_type", "T-1h");
                crate::lineup_chain::normalize_lineup_snapshot_type(&snapshot_type)?;
                payload.insert("snapshot_type".to_string(), json!(snapshot_type));
                required_datetime(&payload, "captured_at")?;
                let team = resolve_team_value(&self.pool, &payload, "team_id", "team_name").await?;
                if let Some(result) = resolution_validation(&mut payload, team, "team") {
                    return Ok(result);
                }
                resolve_match_reference(&self.pool, &mut payload).await?;
                validate_lineup_match_team(&self.pool, &payload).await?;
                resolve_lineup_formation(&self.pool, &mut payload).await?;
                if let Some(coach_id) = optional_uuid(&payload, "coach_id")? {
                    let exists: bool = sqlx::query_scalar(
                        "SELECT EXISTS(SELECT 1 FROM football.coaches WHERE id=$1 AND status='active')",
                    )
                    .bind(coach_id)
                    .fetch_one(&self.pool)
                    .await?;
                    if !exists {
                        return Ok(error(Value::Object(payload), "coach_id 不存在或已归档"));
                    }
                }
                Ok(ready(
                    Value::Object(payload),
                    "阵容版本已验证；提交后执行完整性门禁",
                ))
            }
            SpreadsheetEntityType::LineupPlayer => {
                required(&payload, "lineup_key")?;
                required(&payload, "match_key")?;
                let player = resolve_player_value(&self.pool, &payload).await?;
                if let Some(result) = resolution_validation(&mut payload, player, "player") {
                    return Ok(result);
                }
                validate_lineup_player(&payload)?;
                let team = resolve_team_value(&self.pool, &payload, "team_id", "team_name").await?;
                if let Some(result) = resolution_validation(&mut payload, team, "team") {
                    return Ok(result);
                }
                resolve_match_reference(&self.pool, &mut payload).await?;
                validate_lineup_match_team(&self.pool, &payload).await?;
                Ok(ready(
                    Value::Object(payload),
                    "阵容球员已验证；球队履历将在提交时校验",
                ))
            }
            SpreadsheetEntityType::PlayerDynamicTag => {
                let player = resolve_player_value(&self.pool, &payload).await?;
                if let Some(result) = resolution_validation(&mut payload, player, "player") {
                    return Ok(result);
                }
                validate_dynamic_tag(&self.pool, &payload).await?;
                Ok(ready(Value::Object(payload), "动态标签已验证"))
            }
            _ => Ok(error(
                Value::Object(payload),
                "该工作表不属于比赛与阵容模板",
            )),
        }
    }

    async fn hydrated_active_lineups(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<Vec<football_domain::LineupRecord>> {
        let summaries = self.list_lineups(Some(match_id), 200).await?;
        let mut lineups = Vec::new();
        for summary in summaries.into_iter().filter(|item| item.status == "active") {
            lineups.push(self.read_lineup(summary.id).await?);
        }
        Ok(lineups)
    }

    pub async fn read_match_exchange(&self, match_id: Uuid) -> PersistenceResult<MatchRecord> {
        let row = sqlx::query(
            r#"SELECT match.id,match.external_key,match.competition_id,competition.name AS competition_name,
                      match.season_id,match.stage_id,match.round_id,match.home_team_id,
                      home.canonical_name AS home_team_name,match.away_team_id,
                      away.canonical_name AS away_team_name,match.kickoff_time,match.status,match.venue
               FROM football.matches match
               LEFT JOIN football.competitions competition ON competition.id=match.competition_id
               JOIN football.teams home ON home.id=match.home_team_id
               JOIN football.teams away ON away.id=match.away_team_id
               WHERE match.id=$1"#,
        ).bind(match_id).fetch_one(&self.pool).await?;
        match_record_from_row(&row)
    }

    async fn match_player_references(
        &self,
        match_id: Option<Uuid>,
        as_of: DateTime<Utc>,
    ) -> PersistenceResult<Vec<MatchLineupPlayerReference>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT player.id AS player_id, player.canonical_name, player.date_of_birth,
                   current_team.team_id AS current_team_id, team.canonical_name AS current_team_name,
                   primary_position.position_code AS primary_position_code,
                   primary_position.default_role_code AS primary_role_code,
                   availability.status AS availability_status,
                   profile.average_value AS ability_average,
                   profile.average_confidence AS ability_confidence
            FROM football.players player
            LEFT JOIN LATERAL (
                SELECT period.team_id FROM football.player_team_periods period
                WHERE period.player_id=player.id AND period.valid_from<=$2::date
                  AND (period.valid_to IS NULL OR period.valid_to>=$2::date)
                ORDER BY period.valid_from DESC LIMIT 1
            ) current_team ON true
            LEFT JOIN football.teams team ON team.id=current_team.team_id
            LEFT JOIN LATERAL (
                SELECT position_code, default_role_code FROM football.player_positions position
                WHERE position.player_id=player.id AND position.is_primary
                  AND (position.valid_from IS NULL OR position.valid_from<=$2::date)
                  AND (position.valid_to IS NULL OR position.valid_to>=$2::date)
                ORDER BY position.valid_from DESC NULLS LAST LIMIT 1
            ) primary_position ON true
            LEFT JOIN LATERAL (
                SELECT status FROM football.player_availability item
                WHERE item.player_id=player.id AND item.valid_from<=$2
                  AND (item.valid_to IS NULL OR item.valid_to>=$2)
                ORDER BY item.valid_from DESC LIMIT 1
            ) availability ON true
            LEFT JOIN feature.player_ability_profiles profile ON profile.player_id=player.id
            WHERE $1::uuid IS NULL OR EXISTS(
                SELECT 1
                FROM football.matches fixture
                JOIN football.player_team_periods period
                  ON period.team_id IN (fixture.home_team_id, fixture.away_team_id)
                 AND period.player_id=player.id
                 AND period.valid_from <= $2::date
                 AND (period.valid_to IS NULL OR period.valid_to >= $2::date)
                WHERE fixture.id=$1
            ) OR EXISTS(
                SELECT 1 FROM football.lineups lineup
                JOIN football.lineup_players lp ON lp.lineup_id=lineup.id
                WHERE lineup.match_id=$1 AND lp.player_id=player.id
            )
            ORDER BY player.canonical_name
            LIMIT 5000
            "#,
        ).bind(match_id).bind(as_of).fetch_all(&self.pool).await?;
        rows.iter()
            .map(|row| {
                Ok(MatchLineupPlayerReference {
                    player_id: row.try_get("player_id")?,
                    canonical_name: row.try_get("canonical_name")?,
                    date_of_birth: row.try_get("date_of_birth")?,
                    current_team_id: row.try_get("current_team_id")?,
                    current_team_name: row.try_get("current_team_name")?,
                    primary_position_code: row.try_get("primary_position_code")?,
                    primary_role_code: row.try_get("primary_role_code")?,
                    availability_status: row
                        .try_get::<Option<String>, _>("availability_status")?
                        .map(|value| availability_from_str(&value))
                        .transpose()?,
                    ability_average: row.try_get("ability_average")?,
                    ability_confidence: row.try_get("ability_confidence")?,
                })
            })
            .collect()
    }
}

async fn apply_match_exchange_row(
    tx: &mut Transaction<'_, Postgres>,
    entity: SpreadsheetEntityType,
    values: &Map<String, Value>,
    matched: Option<Uuid>,
    match_keys: &mut HashMap<String, Uuid>,
    lineup_keys: &mut HashMap<String, Uuid>,
) -> PersistenceResult<ApplyOutcome> {
    match entity {
        SpreadsheetEntityType::Match => {
            let id = matched.unwrap_or_else(Uuid::new_v4);
            let match_key = required(values, "match_key")?;
            let competition_id = payload_uuid(values, "_resolved_competition_id")?;
            let home_team_id = payload_uuid(values, "_resolved_home_team_id")?;
            let away_team_id = payload_uuid(values, "_resolved_away_team_id")?;
            let status = default_text(values, "status", "scheduled");
            let metadata = json!({
                "source":"match_lineup_spreadsheet", "snapshot_type": optional_text(values,"snapshot_type"),
                "neutral_venue": optional_bool(values,"neutral_venue")?.unwrap_or(false),
                "weather": optional_text(values,"weather"), "surface": optional_text(values,"surface"),
                "importance": optional_text(values,"importance"), "tactical_notes": optional_text(values,"tactical_notes"),
                "travel_distance_home_km": optional_f64(values,"travel_distance_home_km")?,
                "travel_distance_away_km": optional_f64(values,"travel_distance_away_km")?,
                "rest_days_home": optional_i16(values,"rest_days_home")?, "rest_days_away": optional_i16(values,"rest_days_away")?,
                "schedule_density_home": optional_f64(values,"schedule_density_home")?, "schedule_density_away": optional_f64(values,"schedule_density_away")?,
            });
            if matched.is_some() {
                sqlx::query("UPDATE football.matches SET competition_id=$2,season_id=$3,stage_id=$4,round_id=$5,home_team_id=$6,away_team_id=$7,kickoff_time=$8,status=$9,venue=$10,metadata=metadata||$11,updated_at=now() WHERE id=$1")
                    .bind(id).bind(competition_id).bind(optional_uuid(values,"season_id")?).bind(optional_uuid(values,"stage_id")?).bind(optional_uuid(values,"round_id")?)
                    .bind(home_team_id).bind(away_team_id).bind(required_datetime(values,"kickoff_time")?).bind(status).bind(optional_text(values,"venue")).bind(metadata).execute(&mut **tx).await?;
            } else {
                sqlx::query("INSERT INTO football.matches(id,external_key,competition_id,season_id,stage_id,round_id,home_team_id,away_team_id,kickoff_time,status,venue,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)")
                    .bind(id).bind(&match_key).bind(competition_id).bind(optional_uuid(values,"season_id")?).bind(optional_uuid(values,"stage_id")?).bind(optional_uuid(values,"round_id")?)
                    .bind(home_team_id).bind(away_team_id).bind(required_datetime(values,"kickoff_time")?).bind(status).bind(optional_text(values,"venue")).bind(metadata).execute(&mut **tx).await?;
            }
            match_keys.insert(match_key, id);
            Ok(ApplyOutcome {
                was_update: matched.is_some(),
                ..ApplyOutcome::default()
            })
        }
        SpreadsheetEntityType::Lineup => {
            let match_id = resolve_import_match(values, match_keys)?;
            let team_id = payload_uuid(values, "_resolved_team_id")?;
            let lineup_type = required(values, "lineup_type")?;
            let snapshot_type = default_text(values, "snapshot_type", "T-1h");
            let supersedes_lineup_id: Option<Uuid> = sqlx::query_scalar(
                "SELECT id FROM football.lineups WHERE match_id=$1 AND team_id=$2 AND snapshot_type=$3 AND lineup_type=$4 AND status='active' ORDER BY captured_at DESC,created_at DESC,id DESC LIMIT 1",
            )
            .bind(match_id)
            .bind(team_id)
            .bind(&snapshot_type)
            .bind(&lineup_type)
            .fetch_optional(&mut **tx)
            .await?;
            let superseded = sqlx::query(
                "UPDATE football.lineups SET status='superseded',updated_at=now() WHERE match_id=$1 AND team_id=$2 AND snapshot_type=$3 AND lineup_type=$4 AND status='active'",
            )
            .bind(match_id)
            .bind(team_id)
            .bind(&snapshot_type)
            .bind(&lineup_type)
            .execute(&mut **tx)
            .await?
            .rows_affected();
            let id = Uuid::new_v4();
            sqlx::query(
                r#"INSERT INTO football.lineups(
                    id,match_id,team_id,lineup_type,snapshot_type,formation,formation_id,
                    coach_id,captured_at,status,quality_score,source_urls,
                    supersedes_lineup_id,metadata
                ) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,'active',$10,$11,$12,$13)"#,
            )
            .bind(id)
            .bind(match_id)
            .bind(team_id)
            .bind(lineup_type)
            .bind(snapshot_type)
            .bind(optional_text(values, "formation"))
            .bind(payload_optional_uuid(values, "_resolved_formation_id")?)
            .bind(optional_uuid(values, "coach_id")?)
            .bind(required_datetime(values, "captured_at")?)
            .bind(optional_f64(values, "quality_score")?)
            .bind(parse_source_urls(values, "source_urls"))
            .bind(supersedes_lineup_id)
            .bind(
                json!({"source":"match_lineup_spreadsheet","notes":optional_text(values,"notes")}),
            )
            .execute(&mut **tx)
            .await?;
            lineup_keys.insert(required(values, "lineup_key")?, id);
            Ok(ApplyOutcome {
                was_update: false,
                ended_previous: superseded,
                lineup_id: Some(id),
            })
        }
        SpreadsheetEntityType::LineupPlayer => {
            let lineup_key = required(values, "lineup_key")?;
            let lineup_id = lineup_keys
                .get(&lineup_key)
                .copied()
                .or(optional_uuid(values, "lineup_id")?)
                .ok_or_else(|| {
                    PersistenceError::InvalidState(format!("无法找到阵容：{lineup_key}"))
                })?;
            let player_id = payload_uuid(values, "_resolved_player_id")?;
            let (lineup_match_id, lineup_team_id, lineup_captured_at): (Uuid, Uuid, DateTime<Utc>) =
                sqlx::query_as(
                    "SELECT match_id,team_id,captured_at FROM football.lineups WHERE id=$1",
                )
                .bind(lineup_id)
                .fetch_one(&mut **tx)
                .await?;
            let requested_match_id = resolve_import_match(values, match_keys)?;
            let requested_team_id = payload_uuid(values, "_resolved_team_id")?;
            if lineup_match_id != requested_match_id || lineup_team_id != requested_team_id {
                return Err(PersistenceError::InvalidState(
                    "阵容球员与 lineup_key 指向的比赛或球队不一致".to_string(),
                ));
            }
            let position_code = optional_text(values, "position_code")
                .map(|value| value.trim().to_uppercase())
                .filter(|value| !value.is_empty());
            let inherited_role = resolve_default_tactical_role_in_tx(
                tx,
                player_id,
                position_code.as_deref(),
                lineup_captured_at.date_naive(),
            )
            .await?;
            let role_resolution = resolve_tactical_role(
                optional_text(values, "role_code").as_deref(),
                inherited_role.as_ref(),
            );
            let metadata = metadata_with_role_resolution(
                &json!({
                    "source": "match_lineup_spreadsheet",
                    "notes": optional_text(values, "notes")
                }),
                &role_resolution,
            );
            sqlx::query(r#"INSERT INTO football.lineup_players(
                    lineup_id,player_id,position_code,role_code,is_starter,shirt_number,
                    expected_minutes,actual_minutes,sequence_no,bench_order,
                    availability_status,starting_probability,membership_override,source_urls,metadata
                ) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
                ON CONFLICT(lineup_id,player_id) DO UPDATE SET
                    position_code=EXCLUDED.position_code,role_code=EXCLUDED.role_code,
                    is_starter=EXCLUDED.is_starter,shirt_number=EXCLUDED.shirt_number,
                    expected_minutes=EXCLUDED.expected_minutes,actual_minutes=EXCLUDED.actual_minutes,
                    sequence_no=EXCLUDED.sequence_no,bench_order=EXCLUDED.bench_order,
                    availability_status=EXCLUDED.availability_status,
                    starting_probability=EXCLUDED.starting_probability,
                    membership_override=EXCLUDED.membership_override,
                    source_urls=EXCLUDED.source_urls,metadata=EXCLUDED.metadata"#)
                .bind(lineup_id).bind(player_id).bind(position_code).bind(role_resolution.role_code.as_deref())
                .bind(required_bool(values,"is_starter")?).bind(optional_i16(values,"shirt_number")?).bind(optional_i16(values,"expected_minutes")?).bind(optional_i16(values,"actual_minutes")?)
                .bind(optional_i16(values,"sequence_no")?.unwrap_or(0)).bind(optional_i16(values,"bench_order")?)
                .bind(optional_text(values,"availability_status")).bind(optional_f64(values,"starting_probability")?)
                .bind(optional_bool(values,"membership_override")?.unwrap_or(false)).bind(parse_source_urls(values,"source_urls"))
                .bind(metadata).execute(&mut **tx).await?;
            Ok(ApplyOutcome {
                lineup_id: Some(lineup_id),
                ..ApplyOutcome::default()
            })
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            let player_id = payload_uuid(values, "_resolved_player_id")?;
            sqlx::query("INSERT INTO feature.player_dynamic_tags(id,player_id,tag_code,value,label,confidence,observed_at,valid_from,valid_to,competition_id,position_code,opponent_team_id,sample_size,source_type,calculation_version,metadata) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16)")
                .bind(Uuid::new_v4()).bind(player_id).bind(required(values,"tag_code")?).bind(required_f64(values,"tag_value")?).bind(optional_text(values,"label"))
                .bind(optional_f64(values,"confidence")?.unwrap_or(1.0)).bind(required_datetime(values,"observed_at")?).bind(required_datetime(values,"valid_from")?).bind(required_datetime(values,"valid_to")?)
                .bind(optional_uuid(values,"competition_id")?).bind(optional_text(values,"position_code").map(|value|value.to_uppercase())).bind(optional_uuid(values,"opponent_team_id")?)
                .bind(optional_i32(values,"sample_size")?.unwrap_or(1)).bind(default_text(values,"source_type","lineup_import")).bind(required(values,"calculation_version")?).bind(json!({"source":"match_lineup_spreadsheet"})).execute(&mut **tx).await?;
            Ok(ApplyOutcome::default())
        }
        _ => Err(PersistenceError::InvalidState(
            "不支持的比赛导入实体".to_string(),
        )),
    }
}

async fn insert_row(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    row: &SpreadsheetImportRow,
) -> PersistenceResult<()> {
    sqlx::query("INSERT INTO catalog.import_rows(id,batch_id,sheet_name,row_number,entity_type,requested_action,status,message,payload,matched_entity_id,conflict_candidates) VALUES($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)")
        .bind(row.id).bind(batch_id).bind(&row.sheet_name).bind(row.row_number as i32).bind(row.entity_type.as_str()).bind(row.action.as_str()).bind(row.status.as_str()).bind(&row.message).bind(&row.payload).bind(row.matched_entity_id).bind(serde_json::to_value(&row.conflict_candidates)?).execute(&mut **tx).await?;
    Ok(())
}
fn map_duplicate_import(error: sqlx::Error) -> PersistenceError {
    match error {
        sqlx::Error::Database(database) if database.is_unique_violation() => {
            PersistenceError::InvalidState("该文件已经存在进行中或已完成的导入批次".to_string())
        }
        other => PersistenceError::Sqlx(other),
    }
}

async fn resolve_competition(
    pool: &sqlx::PgPool,
    values: &Map<String, Value>,
) -> PersistenceResult<Uuid> {
    if let Some(id) = optional_uuid(values, "competition_id")? {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM football.competitions WHERE id=$1 AND is_active)",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;
        if exists {
            return Ok(id);
        }
    }
    let code = required(values, "competition_code")?;
    sqlx::query_scalar("SELECT id FROM football.competitions WHERE code=$1 AND is_active")
        .bind(code)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("赛事代码不存在".to_string()))
}
async fn resolve_team_value(
    pool: &sqlx::PgPool,
    values: &Map<String, Value>,
    id_field: &str,
    name_field: &str,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    if let Some(id) = optional_uuid(values, id_field)? {
        return candidate_by_id(pool, "football.teams", id).await;
    }
    let name = required(values, name_field)?;
    candidate_teams(pool, &name).await
}
async fn resolve_player_value(
    pool: &sqlx::PgPool,
    values: &Map<String, Value>,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    if let Some(id) = optional_uuid(values, "player_id")? {
        return candidate_by_id(pool, "football.players", id).await;
    }
    candidate_players(
        pool,
        &required(values, "player_name")?,
        optional_date(values, "birth_date")?,
    )
    .await
}
fn resolution_validation(
    payload: &mut Map<String, Value>,
    candidates: Vec<SpreadsheetConflictCandidate>,
    prefix: &str,
) -> Option<Validation> {
    match candidates.len() {
        0 => Some(error(
            Value::Object(payload.clone()),
            format!("没有匹配到{prefix}记录"),
        )),
        1 => {
            payload.insert(
                format!("_resolved_{prefix}_id"),
                json!(candidates[0].entity_id),
            );
            None
        }
        _ => {
            payload.insert("_conflict_prefix".to_string(), json!(prefix));
            Some(Validation {
                status: SpreadsheetRowStatus::Conflict,
                message: Some("存在多个匹配候选".to_string()),
                payload: Value::Object(payload.clone()),
                matched_entity_id: None,
                candidates,
            })
        }
    }
}
async fn resolve_match_reference(
    pool: &sqlx::PgPool,
    payload: &mut Map<String, Value>,
) -> PersistenceResult<()> {
    if let Some(id) = optional_uuid(payload, "match_id")? {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM football.matches WHERE id=$1)")
                .bind(id)
                .fetch_one(pool)
                .await?;
        if exists {
            payload.insert("_resolved_match_id".to_string(), json!(id));
            return Ok(());
        }
    }
    let key = required(payload, "match_key")?;
    if let Some(id) =
        sqlx::query_scalar::<_, Uuid>("SELECT id FROM football.matches WHERE external_key=$1")
            .bind(&key)
            .fetch_optional(pool)
            .await?
    {
        payload.insert("_resolved_match_id".to_string(), json!(id));
    } else {
        payload.insert("_deferred_match_key".to_string(), json!(key));
    }
    Ok(())
}
async fn find_match(
    pool: &sqlx::PgPool,
    payload: &Map<String, Value>,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    if let Some(id) = optional_uuid(payload, "match_id")? {
        return candidate_by_match_id(pool, id).await;
    }
    let key = required(payload, "match_key")?;
    let rows=sqlx::query("SELECT match.id,home.canonical_name||' vs '||away.canonical_name AS name,match.kickoff_time::text AS detail FROM football.matches match JOIN football.teams home ON home.id=match.home_team_id JOIN football.teams away ON away.id=match.away_team_id WHERE match.external_key=$1").bind(key).fetch_all(pool).await?;
    rows.iter().map(candidate_row).collect()
}
fn decision(
    action: SpreadsheetAction,
    mode: SpreadsheetImportMode,
    payload: Value,
    matches: Vec<SpreadsheetConflictCandidate>,
    prefix: &str,
) -> PersistenceResult<Validation> {
    match matches.len() {
        0 if action == SpreadsheetAction::Update => {
            Ok(error(payload, "标记为 update，但数据库中不存在"))
        }
        0 => Ok(ready(payload, "将新增记录")),
        1 if action == SpreadsheetAction::Update && mode == SpreadsheetImportMode::AddAndUpdate => {
            Ok(Validation {
                status: SpreadsheetRowStatus::ReadyUpdate,
                message: Some("将更新现有记录".to_string()),
                payload,
                matched_entity_id: Some(matches[0].entity_id),
                candidates: vec![],
            })
        }
        1 => Ok(skip(payload, "相同记录已存在")),
        _ => {
            let mut object = payload.as_object().cloned().unwrap_or_default();
            object.insert("_conflict_prefix".to_string(), json!(prefix));
            Ok(Validation {
                status: SpreadsheetRowStatus::Conflict,
                message: Some("存在多个匹配候选".to_string()),
                payload: Value::Object(object),
                matched_entity_id: None,
                candidates: matches,
            })
        }
    }
}
fn ready(payload: Value, message: &str) -> Validation {
    Validation {
        status: SpreadsheetRowStatus::ReadyAdd,
        message: Some(message.to_string()),
        payload,
        matched_entity_id: None,
        candidates: vec![],
    }
}
fn skip(payload: Value, message: impl Into<String>) -> Validation {
    Validation {
        status: SpreadsheetRowStatus::Skip,
        message: Some(message.into()),
        payload,
        matched_entity_id: None,
        candidates: vec![],
    }
}
fn error(payload: Value, message: impl Into<String>) -> Validation {
    Validation {
        status: SpreadsheetRowStatus::Error,
        message: Some(message.into()),
        payload,
        matched_entity_id: None,
        candidates: vec![],
    }
}

async fn validate_lineup_match_team(
    pool: &sqlx::PgPool,
    values: &Map<String, Value>,
) -> PersistenceResult<()> {
    let side = required(values, "team_side")?.to_lowercase();
    if !matches!(side.as_str(), "home" | "away") {
        return Err(PersistenceError::InvalidState(
            "team_side 必须为 home 或 away".to_string(),
        ));
    }
    let team_id = payload_uuid(values, "_resolved_team_id")?;
    let Some(match_id) = payload_optional_uuid(values, "_resolved_match_id")? else {
        // 同一工作簿中新比赛会在提交事务内先创建，此处只能延迟验证。
        return Ok(());
    };
    let row = sqlx::query("SELECT home_team_id,away_team_id FROM football.matches WHERE id=$1")
        .bind(match_id)
        .fetch_one(pool)
        .await?;
    let expected_team_id: Uuid = if side == "home" {
        row.try_get("home_team_id")?
    } else {
        row.try_get("away_team_id")?
    };
    if team_id != expected_team_id {
        return Err(PersistenceError::InvalidState(format!(
            "阵容球队与比赛 {side} 球队不一致"
        )));
    }
    Ok(())
}

async fn resolve_lineup_formation(
    pool: &sqlx::PgPool,
    values: &mut Map<String, Value>,
) -> PersistenceResult<()> {
    if let Some(id) = optional_uuid(values, "formation_id")? {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM football.formations WHERE id=$1 AND is_active)",
        )
        .bind(id)
        .fetch_one(pool)
        .await?;
        if !exists {
            return Err(PersistenceError::InvalidState(
                "formation_id 不存在或已停用".to_string(),
            ));
        }
        values.insert("_resolved_formation_id".to_string(), json!(id));
        return Ok(());
    }
    let formation = required(values, "formation")?;
    let candidates = sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM football.formations WHERE is_active AND lower(code)=lower($1) ORDER BY is_builtin DESC,sort_order,id LIMIT 2",
    )
    .bind(&formation)
    .fetch_all(pool)
    .await?;
    match candidates.as_slice() {
        [id] => {
            values.insert("_resolved_formation_id".to_string(), json!(id));
            Ok(())
        }
        [] => Err(PersistenceError::InvalidState(format!(
            "阵型不存在：{formation}"
        ))),
        _ => Err(PersistenceError::InvalidState(format!(
            "阵型匹配不唯一：{formation}"
        ))),
    }
}

fn validate_lineup_type(values: &Map<String, Value>) -> PersistenceResult<()> {
    let value = required(values, "lineup_type")?;
    if matches!(value.as_str(), "expected" | "confirmed" | "actual") {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(
            "lineup_type 无效".to_string(),
        ))
    }
}
fn validate_lineup_player(values: &Map<String, Value>) -> PersistenceResult<()> {
    let is_starter = required_bool(values, "is_starter")?;
    if let Some(v) = optional_i16(values, "shirt_number")? {
        if !(0..=99).contains(&v) {
            return Err(PersistenceError::InvalidState(
                "球衣号码必须为 0–99".to_string(),
            ));
        }
    }
    for field in ["expected_minutes", "actual_minutes"] {
        if let Some(v) = optional_i16(values, field)? {
            if !(0..=150).contains(&v) {
                return Err(PersistenceError::InvalidState(format!(
                    "{field} 必须为 0–150"
                )));
            }
        }
    }
    if let Some(probability) = optional_f64(values, "starting_probability")? {
        if !(0.0..=1.0).contains(&probability) {
            return Err(PersistenceError::InvalidState(
                "starting_probability 必须位于 0–1".to_string(),
            ));
        }
    }
    if let Some(order) = optional_i16(values, "bench_order")? {
        if !(1..=99).contains(&order) {
            return Err(PersistenceError::InvalidState(
                "bench_order 必须位于 1–99".to_string(),
            ));
        }
        if is_starter {
            return Err(PersistenceError::InvalidState(
                "首发球员不能设置 bench_order".to_string(),
            ));
        }
    }
    if let Some(status) = optional_text(values, "availability_status") {
        availability_from_str(&status)?;
    }
    optional_bool(values, "membership_override")?;
    Ok(())
}
async fn validate_dynamic_tag(
    pool: &sqlx::PgPool,
    values: &Map<String, Value>,
) -> PersistenceResult<()> {
    let code = required(values, "tag_code")?;
    let row=sqlx::query("SELECT minimum_value,maximum_value FROM feature.player_dynamic_tag_definitions WHERE code=$1").bind(&code).fetch_optional(pool).await?.ok_or_else(||PersistenceError::InvalidState(format!("动态标签不存在：{code}")))?;
    let value = required_f64(values, "tag_value")?;
    let min: f64 = row.try_get("minimum_value")?;
    let max: f64 = row.try_get("maximum_value")?;
    if value < min || value > max {
        return Err(PersistenceError::InvalidState(format!(
            "标签值超出允许范围 {min}–{max}"
        )));
    }
    let from = required_datetime(values, "valid_from")?;
    let to = required_datetime(values, "valid_to")?;
    if to <= from {
        return Err(PersistenceError::InvalidState(
            "动态标签失效时间必须晚于生效时间".to_string(),
        ));
    }
    Ok(())
}
fn resolve_import_match(
    values: &Map<String, Value>,
    map: &HashMap<String, Uuid>,
) -> PersistenceResult<Uuid> {
    if let Some(id) = payload_optional_uuid(values, "_resolved_match_id")? {
        return Ok(id);
    }
    let key = values
        .get("_deferred_match_key")
        .and_then(Value::as_str)
        .unwrap_or_else(|| {
            values
                .get("match_key")
                .and_then(Value::as_str)
                .unwrap_or("")
        });
    map.get(key)
        .copied()
        .ok_or_else(|| PersistenceError::InvalidState(format!("无法解析比赛：{key}")))
}

async fn candidate_by_id(
    pool: &sqlx::PgPool,
    table: &str,
    id: Uuid,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    let query = format!("SELECT id,canonical_name FROM {table} WHERE id=$1");
    let rows = sqlx::query(&query).bind(id).fetch_all(pool).await?;
    rows.iter().map(candidate_row).collect()
}
async fn candidate_by_match_id(
    pool: &sqlx::PgPool,
    id: Uuid,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    let rows=sqlx::query("SELECT match.id,home.canonical_name||' vs '||away.canonical_name AS name,match.kickoff_time::text AS detail FROM football.matches match JOIN football.teams home ON home.id=match.home_team_id JOIN football.teams away ON away.id=match.away_team_id WHERE match.id=$1").bind(id).fetch_all(pool).await?;
    rows.iter().map(candidate_row).collect()
}
async fn candidate_teams(
    pool: &sqlx::PgPool,
    name: &str,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    let rows=sqlx::query("SELECT id,canonical_name AS name,country_code AS detail FROM football.teams WHERE normalized_name=$1 OR EXISTS(SELECT 1 FROM football.team_names n WHERE n.team_id=football.teams.id AND n.normalized_name=$1) ORDER BY canonical_name LIMIT 10").bind(normalize(name)).fetch_all(pool).await?;
    rows.iter().map(candidate_row).collect()
}
async fn candidate_players(
    pool: &sqlx::PgPool,
    name: &str,
    birth: Option<NaiveDate>,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    let rows=sqlx::query("SELECT id,canonical_name AS name,COALESCE(date_of_birth::text,'出生日期未知') AS detail FROM football.players WHERE (normalized_name=$1 OR EXISTS(SELECT 1 FROM football.player_names n WHERE n.player_id=football.players.id AND n.normalized_name=$1)) AND ($2::date IS NULL OR date_of_birth=$2) ORDER BY canonical_name LIMIT 10").bind(normalize(name)).bind(birth).fetch_all(pool).await?;
    rows.iter().map(candidate_row).collect()
}
fn candidate_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<SpreadsheetConflictCandidate> {
    Ok(SpreadsheetConflictCandidate {
        entity_id: row.try_get("id")?,
        display_name: row
            .try_get("name")
            .or_else(|_| row.try_get("canonical_name"))?,
        detail: row.try_get("detail").ok(),
    })
}

fn row_from_db(row: &sqlx::postgres::PgRow) -> PersistenceResult<SpreadsheetImportRow> {
    let candidates: Value = row.try_get("conflict_candidates")?;
    Ok(SpreadsheetImportRow {
        id: row.try_get("id")?,
        sheet_name: row.try_get("sheet_name")?,
        row_number: row.try_get::<i32, _>("row_number")? as u32,
        entity_type: parse_entity(&row.try_get::<String, _>("entity_type")?)?,
        action: parse_action(&row.try_get::<String, _>("requested_action")?)?,
        status: parse_status(&row.try_get::<String, _>("status")?)?,
        message: row.try_get("message")?,
        payload: row.try_get("payload")?,
        matched_entity_id: row.try_get("matched_entity_id")?,
        conflict_candidates: serde_json::from_value(candidates)?,
    })
}
fn parse_entity(value: &str) -> PersistenceResult<SpreadsheetEntityType> {
    match value {
        "match" => Ok(SpreadsheetEntityType::Match),
        "lineup" => Ok(SpreadsheetEntityType::Lineup),
        "lineup_player" => Ok(SpreadsheetEntityType::LineupPlayer),
        "player_dynamic_tag" => Ok(SpreadsheetEntityType::PlayerDynamicTag),
        _ => Err(PersistenceError::InvalidState(format!(
            "未知比赛导入实体：{value}"
        ))),
    }
}
fn parse_action(value: &str) -> PersistenceResult<SpreadsheetAction> {
    match value {
        "add" => Ok(SpreadsheetAction::Add),
        "update" => Ok(SpreadsheetAction::Update),
        "skip" => Ok(SpreadsheetAction::Skip),
        _ => Err(PersistenceError::InvalidState("未知导入动作".to_string())),
    }
}
fn parse_status(value: &str) -> PersistenceResult<SpreadsheetRowStatus> {
    match value {
        "ready_add" => Ok(SpreadsheetRowStatus::ReadyAdd),
        "ready_update" => Ok(SpreadsheetRowStatus::ReadyUpdate),
        "conflict" => Ok(SpreadsheetRowStatus::Conflict),
        "error" => Ok(SpreadsheetRowStatus::Error),
        "skip" => Ok(SpreadsheetRowStatus::Skip),
        "imported" => Ok(SpreadsheetRowStatus::Imported),
        _ => Err(PersistenceError::InvalidState("未知导入状态".to_string())),
    }
}
fn parse_mode(value: Option<&str>) -> PersistenceResult<SpreadsheetImportMode> {
    match value.unwrap_or("add_and_update") {
        "add_only" => Ok(SpreadsheetImportMode::AddOnly),
        "add_and_update" => Ok(SpreadsheetImportMode::AddAndUpdate),
        _ => Err(PersistenceError::InvalidState("未知导入模式".to_string())),
    }
}
fn mode_text(mode: SpreadsheetImportMode) -> &'static str {
    match mode {
        SpreadsheetImportMode::AddOnly => "add_only",
        SpreadsheetImportMode::AddAndUpdate => "add_and_update",
    }
}
fn count_rows(rows: &[SpreadsheetImportRow]) -> SpreadsheetImportCounts {
    let mut counts = SpreadsheetImportCounts {
        total: rows.len() as u64,
        ..Default::default()
    };
    for row in rows {
        match row.status {
            SpreadsheetRowStatus::ReadyAdd => counts.ready_add += 1,
            SpreadsheetRowStatus::ReadyUpdate => counts.ready_update += 1,
            SpreadsheetRowStatus::ReadyEndPrevious => counts.ready_end_previous += 1,
            SpreadsheetRowStatus::Conflict => counts.conflict += 1,
            SpreadsheetRowStatus::Error => counts.error += 1,
            SpreadsheetRowStatus::Skip => counts.skipped += 1,
            SpreadsheetRowStatus::Imported => counts.imported += 1,
        }
    }
    counts
}

fn required(values: &Map<String, Value>, key: &str) -> PersistenceResult<String> {
    let value = text(values, key);
    if value.is_empty() {
        Err(PersistenceError::InvalidState(format!(
            "缺少必填字段：{key}"
        )))
    } else {
        Ok(value)
    }
}
fn text(values: &Map<String, Value>, key: &str) -> String {
    values
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}
fn optional_text(values: &Map<String, Value>, key: &str) -> Option<String> {
    let value = text(values, key);
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}
fn default_text(values: &Map<String, Value>, key: &str, default: &str) -> String {
    optional_text(values, key).unwrap_or_else(|| default.to_string())
}
fn optional_uuid(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<Uuid>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        Uuid::parse_str(&value)
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} UUID 无效：{error}")))
    }
}
fn payload_optional_uuid(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<Uuid>> {
    match values.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if value.is_empty() => Ok(None),
        Some(value) => serde_json::from_value(value.clone())
            .map(Some)
            .map_err(PersistenceError::Serialization),
    }
}
fn payload_uuid(values: &Map<String, Value>, key: &str) -> PersistenceResult<Uuid> {
    payload_optional_uuid(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少内部字段：{key}")))
}
fn optional_date(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<NaiveDate>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        NaiveDate::parse_from_str(&value, "%Y-%m-%d")
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} 日期无效：{error}")))
    }
}
fn required_datetime(values: &Map<String, Value>, key: &str) -> PersistenceResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&required(values, key)?)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| PersistenceError::InvalidState(format!("{key} 时间无效：{error}")))
}
fn optional_f64(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<f64>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        value
            .parse()
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} 数值无效：{error}")))
    }
}
fn required_f64(values: &Map<String, Value>, key: &str) -> PersistenceResult<f64> {
    optional_f64(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少数值：{key}")))
}
fn optional_i16(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<i16>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        value
            .parse()
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} 整数无效：{error}")))
    }
}
fn optional_i32(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<i32>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        value
            .parse()
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} 整数无效：{error}")))
    }
}
fn optional_bool(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<bool>> {
    let value = text(values, key).to_lowercase();
    match value.as_str() {
        "" => Ok(None),
        "true" | "1" | "yes" => Ok(Some(true)),
        "false" | "0" | "no" => Ok(Some(false)),
        _ => Err(PersistenceError::InvalidState(format!("{key} 布尔值无效"))),
    }
}
fn required_bool(values: &Map<String, Value>, key: &str) -> PersistenceResult<bool> {
    optional_bool(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少布尔字段：{key}")))
}
fn parse_source_urls(values: &Map<String, Value>, key: &str) -> Vec<String> {
    let raw = text(values, key);
    let mut urls = raw
        .split([';', '\n', '\r', ','])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    urls.sort();
    urls.dedup();
    urls
}

fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
fn availability_from_str(value: &str) -> PersistenceResult<AvailabilityStatus> {
    match value {
        "available" => Ok(AvailabilityStatus::Available),
        "doubtful" => Ok(AvailabilityStatus::Doubtful),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        "injured" => Ok(AvailabilityStatus::Injured),
        "suspended" => Ok(AvailabilityStatus::Suspended),
        "rested" => Ok(AvailabilityStatus::Rested),
        "returning" => Ok(AvailabilityStatus::Returning),
        "unknown" => Ok(AvailabilityStatus::Unknown),
        _ => Err(PersistenceError::InvalidState("未知可用状态".to_string())),
    }
}
fn match_status_from_str(value: &str) -> PersistenceResult<MatchStatus> {
    match value {
        "scheduled" => Ok(MatchStatus::Scheduled),
        "live" => Ok(MatchStatus::Live),
        "finished" => Ok(MatchStatus::Finished),
        "postponed" => Ok(MatchStatus::Postponed),
        "cancelled" => Ok(MatchStatus::Cancelled),
        _ => Err(PersistenceError::InvalidState("未知比赛状态".to_string())),
    }
}
pub(crate) fn match_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<MatchRecord> {
    Ok(MatchRecord {
        id: row.try_get("id")?,
        external_key: row.try_get("external_key")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        round_id: row.try_get("round_id")?,
        home_team_id: row.try_get("home_team_id")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_id: row.try_get("away_team_id")?,
        away_team_name: row.try_get("away_team_name")?,
        kickoff_time: row.try_get("kickoff_time")?,
        status: match_status_from_str(&row.try_get::<String, _>("status")?)?,
        venue: row.try_get("venue")?,
    })
}
