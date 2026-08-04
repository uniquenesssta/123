use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, SecondsFormat, Utc};
use football_domain::{
    SpreadsheetAction, SpreadsheetConflictCandidate, SpreadsheetEntityType, SpreadsheetExportData,
    SpreadsheetExternalIdRow, SpreadsheetImportCommitResult, SpreadsheetImportCounts,
    SpreadsheetImportMode, SpreadsheetImportPreview, SpreadsheetImportResolution,
    SpreadsheetImportRow, SpreadsheetParsedWorkbook, SpreadsheetPlayerAbilityRow,
    SpreadsheetPlayerAvailabilityRow, SpreadsheetPlayerDynamicTagRow, SpreadsheetPlayerNameRow,
    SpreadsheetPlayerPositionRow, SpreadsheetPlayerRow, SpreadsheetPlayerTeamPeriodRow,
    SpreadsheetRowStatus, SpreadsheetTeamRow, PLAYER_IMPORT_FORMAT, PLAYER_MONTHLY_FORMAT,
};
use serde_json::{json, Map, Value};
use sqlx::{Postgres, Row, Transaction};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

impl PostgresStore {
    pub async fn preview_spreadsheet_import(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let team_references = HashMap::new();
        self.preview_spreadsheet_import_inner(parsed, mode, &team_references)
            .await
    }

    pub async fn preview_spreadsheet_import_with_team_references(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
        team_references: &HashMap<String, String>,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        self.preview_spreadsheet_import_inner(parsed, mode, team_references)
            .await
    }

    async fn preview_spreadsheet_import_inner(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
        external_team_references: &HashMap<String, String>,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let import_type = player_import_type(&parsed.format_version)?;
        if let Some(existing) = sqlx::query(
            "SELECT id,status FROM catalog.import_batches WHERE source_sha256=$1 AND import_type=$2 AND status IN ('pending','running','succeeded') ORDER BY started_at DESC NULLS LAST LIMIT 1",
        )
        .bind(&parsed.source_sha256)
        .bind(import_type)
        .fetch_optional(&self.pool)
        .await?
        {
            let existing_id: Uuid = existing.try_get("id")?;
            let existing_status: String = existing.try_get("status")?;
            if existing_status != "pending" {
                return self.read_spreadsheet_import_preview(existing_id).await;
            }
            sqlx::query(
                "UPDATE catalog.import_batches SET status='cancelled',finished_at=now(),metadata=metadata||$2 WHERE id=$1 AND status='pending'",
            )
            .bind(existing_id)
            .bind(json!({
                "cancel_reason": "repreview_same_source",
                "replacement_requested_at": Utc::now(),
            }))
            .execute(&self.pool)
            .await?;
        }
        let batch_id = Uuid::new_v4();
        let mode_text = match mode {
            SpreadsheetImportMode::AddOnly => "add_only",
            SpreadsheetImportMode::AddAndUpdate => "add_and_update",
        };
        let player_keys = collect_keys(&parsed.rows, SpreadsheetEntityType::Player, "player_key");
        let team_keys = collect_keys(&parsed.rows, SpreadsheetEntityType::Team, "team_key");
        let duplicate_player_keys =
            duplicate_keys(&parsed.rows, SpreadsheetEntityType::Player, "player_key");
        let duplicate_team_keys =
            duplicate_keys(&parsed.rows, SpreadsheetEntityType::Team, "team_key");
        let validation_context = SpreadsheetValidationContext {
            mode,
            player_keys: &player_keys,
            team_keys: &team_keys,
            duplicate_player_keys: &duplicate_player_keys,
            duplicate_team_keys: &duplicate_team_keys,
            external_team_references,
        };

        let mut preview_rows = Vec::with_capacity(parsed.rows.len());
        for raw in &parsed.rows {
            let validation = self
                .validate_spreadsheet_row(
                    raw.entity_type,
                    raw.action,
                    &raw.values,
                    &validation_context,
                )
                .await;
            let row = match validation {
                Ok(validation) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: canonical_spreadsheet_import_action(
                        raw.action,
                        raw.entity_type,
                        validation.status,
                    ),
                    status: validation.status,
                    message: validation.message,
                    payload: validation.payload,
                    matched_entity_id: validation.matched_entity_id,
                    conflict_candidates: validation.conflict_candidates,
                },
                Err(error) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: canonical_spreadsheet_import_action(
                        raw.action,
                        raw.entity_type,
                        SpreadsheetRowStatus::Error,
                    ),
                    status: SpreadsheetRowStatus::Error,
                    message: Some(error.to_string()),
                    payload: raw.values.clone(),
                    matched_entity_id: None,
                    conflict_candidates: Vec::new(),
                },
            };
            preview_rows.push(row);
        }
        let counts = count_preview_rows(&preview_rows);
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO catalog.import_batches (
                id, import_type, workbook_kind, format_version, status, source_file_name, source_sha256,
                import_mode, started_at, skipped_count, error_count, metadata
            ) VALUES ($1, $2, $3, $4, 'pending', $5, $6, $7, now(), $8, $9, $10)
            "#,
        )
        .bind(batch_id)
        .bind(import_type)
        .bind(if parsed.format_version == PLAYER_MONTHLY_FORMAT { "player_monthly" } else { "legacy_player" })
        .bind(&parsed.format_version)
        .bind(&parsed.source_file_name)
        .bind(&parsed.source_sha256)
        .bind(mode_text)
        .bind(counts.skipped as i64)
        .bind((counts.error + counts.conflict) as i64)
        .bind(json!({
            "format_version": parsed.format_version,
            "preview_counts": &counts,
        }))
        .execute(&mut *tx)
        .await?;
        for row in &preview_rows {
            insert_import_row(&mut tx, batch_id, row).await?;
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

    pub async fn read_spreadsheet_import_preview(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let batch = sqlx::query(
            r#"
            SELECT source_file_name, source_sha256, import_mode, started_at
            FROM catalog.import_batches
            WHERE id = $1 AND import_type IN ('player_catalog_xlsx','player_monthly_xlsx')
            "#,
        )
        .bind(batch_id)
        .fetch_one(&self.pool)
        .await?;
        let rows = sqlx::query(
            r#"
            SELECT id, sheet_name, row_number, entity_type, requested_action,
                   status, message, payload, matched_entity_id, conflict_candidates
            FROM catalog.import_rows
            WHERE batch_id = $1
            ORDER BY row_number, sheet_name, id
            "#,
        )
        .bind(batch_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(import_row_from_row)
        .collect::<PersistenceResult<Vec<_>>>()?;
        let counts = count_preview_rows(&rows);
        Ok(SpreadsheetImportPreview {
            batch_id,
            source_file_name: batch
                .try_get::<Option<String>, _>("source_file_name")?
                .unwrap_or_default(),
            source_sha256: batch
                .try_get::<Option<String>, _>("source_sha256")?
                .unwrap_or_default(),
            import_mode: parse_import_mode(
                batch
                    .try_get::<Option<String>, _>("import_mode")?
                    .as_deref(),
            )?,
            counts,
            rows,
            created_at: batch
                .try_get::<Option<DateTime<Utc>>, _>("started_at")?
                .unwrap_or_else(Utc::now),
        })
    }

    pub async fn resolve_spreadsheet_import_conflict(
        &self,
        batch_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let mut tx = self.pool.begin().await?;
        let batch = sqlx::query(
            r#"
            SELECT status, import_mode, inserted_count, updated_count,
                   ended_previous_count, skipped_count, error_count, finished_at
            FROM catalog.import_batches
            WHERE id = $1 AND import_type IN ('player_catalog_xlsx','player_monthly_xlsx')
            FOR UPDATE
            "#,
        )
        .bind(batch_id)
        .fetch_one(&mut *tx)
        .await?;
        let batch_status: String = batch.try_get("status")?;
        if batch_status != "pending" {
            return Err(PersistenceError::InvalidState(format!(
                "导入批次状态为 {batch_status}，不能处理冲突"
            )));
        }
        let import_mode = parse_import_mode(
            batch
                .try_get::<Option<String>, _>("import_mode")?
                .as_deref(),
        )?;
        let row = sqlx::query(
            r#"
            SELECT entity_type, requested_action, status, payload, conflict_candidates
            FROM catalog.import_rows
            WHERE id = $1 AND batch_id = $2
            FOR UPDATE
            "#,
        )
        .bind(resolution.row_id)
        .bind(batch_id)
        .fetch_one(&mut *tx)
        .await?;
        let row_status: String = row.try_get("status")?;
        if row_status != "conflict" {
            return Err(PersistenceError::InvalidState(
                "只有冲突记录可以人工处理".to_string(),
            ));
        }
        if resolution.skip {
            sqlx::query(
                r#"
                UPDATE catalog.import_rows SET
                    status = 'skip', message = '用户在预检中选择跳过',
                    matched_entity_id = NULL, conflict_candidates = '[]'::jsonb
                WHERE id = $1
                "#,
            )
            .bind(resolution.row_id)
            .execute(&mut *tx)
            .await?;
        } else {
            let selected = resolution.selected_entity_id.ok_or_else(|| {
                PersistenceError::InvalidState("请选择一个冲突候选记录".to_string())
            })?;
            let candidates_value: Value = row.try_get("conflict_candidates")?;
            let candidates: Vec<SpreadsheetConflictCandidate> =
                serde_json::from_value(candidates_value)?;
            if !candidates
                .iter()
                .any(|candidate| candidate.entity_id == selected)
            {
                return Err(PersistenceError::InvalidState(
                    "所选记录不属于该冲突的候选范围".to_string(),
                ));
            }
            let entity_type = parse_entity_type(&row.try_get::<String, _>("entity_type")?)?;
            let action = parse_action(&row.try_get::<String, _>("requested_action")?)?;
            let mut payload: Value = row.try_get("payload")?;
            let payload_object = payload.as_object_mut().ok_or_else(|| {
                PersistenceError::InvalidState("冲突记录内容不是对象".to_string())
            })?;
            let prefix = payload_object
                .get("_conflict_prefix")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| match entity_type {
                    SpreadsheetEntityType::Team => "team".to_string(),
                    _ => "player".to_string(),
                });
            payload_object.insert(format!("_resolved_{prefix}_id"), json!(selected));
            payload_object.remove("_conflict_prefix");
            let (status, message) = if entity_type == SpreadsheetEntityType::ExternalEntityId {
                let provider_id = payload_uuid(payload_object, "_resolved_provider_id")?;
                let external_entity_type = required_text(payload_object, "entity_type")?;
                let external_id = required_text(payload_object, "external_id")?;
                let existing_entity_id: Option<Uuid> = sqlx::query_scalar(
                    r#"
                    SELECT entity_id
                    FROM football.external_entity_ids
                    WHERE provider_id = $1 AND entity_type = $2 AND external_id = $3
                    "#,
                )
                .bind(provider_id)
                .bind(&external_entity_type)
                .bind(&external_id)
                .fetch_optional(&mut *tx)
                .await?;
                match existing_entity_id {
                    Some(existing) if existing != selected => {
                        return Err(PersistenceError::InvalidState(
                            "该外部 ID 已绑定到另一条数据库记录，禁止自动改绑".to_string(),
                        ));
                    }
                    Some(_)
                        if matches!(
                            action,
                            SpreadsheetAction::Update | SpreadsheetAction::Upsert
                        ) && import_mode == SpreadsheetImportMode::AddAndUpdate =>
                    {
                        (SpreadsheetRowStatus::ReadyUpdate, "已确认现有外部 ID 关联")
                    }
                    Some(_) => (SpreadsheetRowStatus::Skip, "相同外部 ID 关联已存在"),
                    None if action == SpreadsheetAction::Update => {
                        return Err(PersistenceError::InvalidState(
                            "标记为 update，但外部 ID 关联不存在".to_string(),
                        ));
                    }
                    None => (SpreadsheetRowStatus::ReadyAdd, "已选择外部 ID 关联记录"),
                }
            } else if matches!(
                entity_type,
                SpreadsheetEntityType::Team | SpreadsheetEntityType::Player
            ) {
                if matches!(
                    action,
                    SpreadsheetAction::Update
                        | SpreadsheetAction::Upsert
                        | SpreadsheetAction::Clear
                ) && import_mode == SpreadsheetImportMode::AddAndUpdate
                {
                    (
                        SpreadsheetRowStatus::ReadyUpdate,
                        "已关联现有记录，将执行更新",
                    )
                } else {
                    (
                        SpreadsheetRowStatus::Skip,
                        "已关联现有记录；当前动作不会更新",
                    )
                }
            } else {
                (SpreadsheetRowStatus::ReadyAdd, "已选择关联记录")
            };
            sqlx::query(
                r#"
                UPDATE catalog.import_rows SET
                    status = $2, message = $3, payload = $4,
                    matched_entity_id = $5, conflict_candidates = '[]'::jsonb
                WHERE id = $1
                "#,
            )
            .bind(resolution.row_id)
            .bind(status.as_str())
            .bind(message)
            .bind(payload)
            .bind(selected)
            .execute(&mut *tx)
            .await?;
        }
        sqlx::query(
            r#"
            INSERT INTO audit.events (id, event_type, entity_type, entity_id, payload)
            VALUES ($1, 'spreadsheet_import_conflict_resolved', 'import_row', $2, $3)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(resolution.row_id.to_string())
        .bind(json!({
            "batch_id": batch_id,
            "selected_entity_id": resolution.selected_entity_id,
            "skip": resolution.skip,
        }))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        self.read_spreadsheet_import_preview(batch_id).await
    }

    pub async fn commit_spreadsheet_import(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportCommitResult> {
        let mut tx = self.pool.begin().await?;
        let batch = sqlx::query(
            r#"
            SELECT status, import_mode, inserted_count, updated_count,
                   ended_previous_count, skipped_count, error_count, finished_at
            FROM catalog.import_batches
            WHERE id = $1 AND import_type IN ('player_catalog_xlsx','player_monthly_xlsx')
            FOR UPDATE
            "#,
        )
        .bind(batch_id)
        .fetch_one(&mut *tx)
        .await?;
        let status: String = batch.try_get("status")?;
        if status == "succeeded" {
            return Ok(SpreadsheetImportCommitResult {
                batch_id,
                inserted_count: batch.try_get::<i64, _>("inserted_count")? as u64,
                updated_count: batch.try_get::<i64, _>("updated_count")? as u64,
                ended_previous_count: batch.try_get::<i64, _>("ended_previous_count")? as u64,
                skipped_count: batch.try_get::<i64, _>("skipped_count")? as u64,
                error_count: batch.try_get::<i64, _>("error_count")? as u64,
                finished_at: batch
                    .try_get::<Option<DateTime<Utc>>, _>("finished_at")?
                    .unwrap_or_else(Utc::now),
            });
        }
        if status != "pending" {
            return Err(PersistenceError::InvalidState(format!(
                "导入批次状态为 {status}，不能提交"
            )));
        }
        let blocking_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM catalog.import_rows WHERE batch_id = $1 AND status IN ('conflict', 'error')",
        )
        .bind(batch_id)
        .fetch_one(&mut *tx)
        .await?;
        if blocking_count > 0 {
            return Err(PersistenceError::InvalidState(format!(
                "仍有 {blocking_count} 条冲突或错误记录，不能提交"
            )));
        }
        sqlx::query("UPDATE catalog.import_batches SET status = 'running' WHERE id = $1")
            .bind(batch_id)
            .execute(&mut *tx)
            .await?;
        let rows = sqlx::query(
            r#"
            SELECT id, entity_type, requested_action, status, payload, matched_entity_id
            FROM catalog.import_rows
            WHERE batch_id = $1 AND status IN ('ready_add', 'ready_update')
            ORDER BY CASE entity_type
                WHEN 'team' THEN 1 WHEN 'player' THEN 2 WHEN 'player_name' THEN 3
                WHEN 'player_position' THEN 4 WHEN 'player_team_period' THEN 5
                WHEN 'player_ability' THEN 6 WHEN 'player_availability' THEN 7
                WHEN 'external_entity_id' THEN 8 ELSE 99 END,
                row_number, id
            "#,
        )
        .bind(batch_id)
        .fetch_all(&mut *tx)
        .await?;
        let mut player_keys = HashMap::<String, Uuid>::new();
        let mut team_keys = HashMap::<String, Uuid>::new();
        let mut inserted = 0_u64;
        let mut updated = 0_u64;
        let mut context = ImportCommitContext {
            player_keys: &mut player_keys,
            team_keys: &mut team_keys,
        };
        for row in rows {
            let row_id: Uuid = row.try_get("id")?;
            let entity_type = parse_entity_type(&row.try_get::<String, _>("entity_type")?)?;
            let action = parse_action(&row.try_get::<String, _>("requested_action")?)?;
            let row_status = parse_row_status(&row.try_get::<String, _>("status")?)?;
            let payload: Value = row.try_get("payload")?;
            let matched_entity_id: Option<Uuid> = row.try_get("matched_entity_id")?;
            let outcome = apply_import_row(
                &mut tx,
                entity_type,
                action,
                row_status,
                &payload,
                matched_entity_id,
                &mut context,
            )
            .await?;
            match outcome {
                ApplyOutcome::Inserted => inserted += 1,
                ApplyOutcome::Updated => updated += 1,
            }
            sqlx::query("UPDATE catalog.import_rows SET status = 'imported', imported_at = now() WHERE id = $1")
                .bind(row_id)
                .execute(&mut *tx)
                .await?;
        }
        let skipped: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::bigint FROM catalog.import_rows WHERE batch_id = $1 AND status = 'skip'",
        )
        .bind(batch_id)
        .fetch_one(&mut *tx)
        .await?;
        let finished_at = Utc::now();
        sqlx::query(
            r#"
            UPDATE catalog.import_batches SET
                status = 'succeeded', inserted_count = $2, updated_count = $3,
                skipped_count = $4, error_count = 0, finished_at = $5
            WHERE id = $1
            "#,
        )
        .bind(batch_id)
        .bind(inserted as i64)
        .bind(updated as i64)
        .bind(skipped)
        .bind(finished_at)
        .execute(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO audit.events (id, event_type, entity_type, entity_id, payload)
            VALUES ($1, 'spreadsheet_import_committed', 'import_batch', $2, $3)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(batch_id.to_string())
        .bind(json!({"inserted": inserted, "updated": updated, "skipped": skipped}))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(SpreadsheetImportCommitResult {
            batch_id,
            inserted_count: inserted,
            updated_count: updated,
            ended_previous_count: 0,
            skipped_count: skipped as u64,
            error_count: 0,
            finished_at,
        })
    }

    pub async fn spreadsheet_export_data(&self) -> PersistenceResult<SpreadsheetExportData> {
        let teams = sqlx::query(
            "SELECT id, canonical_name, country_code, is_active FROM football.teams ORDER BY normalized_name, id",
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| Ok(SpreadsheetTeamRow {
            team_id: row.try_get("id")?,
            canonical_name: row.try_get("canonical_name")?,
            country_code: row.try_get("country_code")?,
            is_active: row.try_get("is_active")?,
        }))
        .collect::<PersistenceResult<Vec<_>>>()?;
        let players = sqlx::query(
            r#"
            SELECT id, canonical_name, date_of_birth, nationality_code,
                   preferred_foot, height_cm, status
            FROM football.players ORDER BY normalized_name, id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerRow {
                player_id: row.try_get("id")?,
                canonical_name: row.try_get("canonical_name")?,
                date_of_birth: row.try_get("date_of_birth")?,
                nationality_code: row.try_get("nationality_code")?,
                preferred_foot: row.try_get("preferred_foot")?,
                height_cm: row.try_get("height_cm")?,
                status: row.try_get("status")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let names = sqlx::query(
            r#"
            SELECT name.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, name.name,
                   name.language_code, name.is_primary, name.valid_from, name.valid_to
            FROM football.player_names name
            JOIN football.players player ON player.id = name.player_id
            ORDER BY player.normalized_name, name.is_primary DESC, name.name
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerNameRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                name: row.try_get("name")?,
                language_code: row.try_get("language_code")?,
                is_primary: row.try_get("is_primary")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let positions = sqlx::query(
            r#"
            SELECT position.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, position.position_code,
                   position.proficiency, position.default_role_code, position.is_primary,
                   position.valid_from, position.valid_to
            FROM football.player_positions position
            JOIN football.players player ON player.id = position.player_id
            ORDER BY player.normalized_name, position.is_primary DESC, position.position_code
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerPositionRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                position_code: row.try_get("position_code")?,
                proficiency: row.try_get("proficiency")?,
                default_role_code: row.try_get("default_role_code")?,
                is_primary: row.try_get("is_primary")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let team_periods = sqlx::query(
            r#"
            SELECT period.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, period.team_id,
                   team.canonical_name AS team_name, period.season_id, period.squad_number,
                   period.valid_from, period.valid_to, period.registration_status
            FROM football.player_team_periods period
            JOIN football.players player ON player.id = period.player_id
            JOIN football.teams team ON team.id = period.team_id
            ORDER BY player.normalized_name, period.valid_from DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerTeamPeriodRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                season_id: row.try_get("season_id")?,
                squad_number: row.try_get("squad_number")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
                registration_status: row.try_get("registration_status")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let abilities = sqlx::query(
            r#"
            SELECT observation.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, observation.dimension_code,
                   observation.context_type, observation.context_id, observation.value,
                   observation.confidence, observation.sample_size, observation.observed_at,
                   observation.effective_from, observation.effective_to, observation.calculation_version
            FROM feature.player_ability_observations observation
            JOIN football.players player ON player.id = observation.player_id
            ORDER BY player.normalized_name, observation.dimension_code, observation.observed_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| Ok(SpreadsheetPlayerAbilityRow {
            player_id: row.try_get("player_id")?, player_name: row.try_get("player_name")?,
            player_birth_date: row.try_get("player_birth_date")?, dimension_code: row.try_get("dimension_code")?,
            context_type: row.try_get("context_type")?, context_id: row.try_get("context_id")?,
            value: row.try_get("value")?, confidence: row.try_get("confidence")?,
            sample_size: row.try_get("sample_size")?, observed_at: row.try_get("observed_at")?,
            effective_from: row.try_get("effective_from")?, effective_to: row.try_get("effective_to")?,
            calculation_version: row.try_get("calculation_version")?,
        }))
        .collect::<PersistenceResult<Vec<_>>>()?;
        let availability = sqlx::query(
            r#"
            SELECT availability.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, availability.team_id,
                   team.canonical_name AS team_name, availability.competition_id,
                   availability.status, availability.reason, availability.confidence,
                   availability.valid_from, availability.valid_to
            FROM football.player_availability availability
            JOIN football.players player ON player.id = availability.player_id
            LEFT JOIN football.teams team ON team.id = availability.team_id
            ORDER BY player.normalized_name, availability.valid_from DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerAvailabilityRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                competition_id: row.try_get("competition_id")?,
                status: row.try_get("status")?,
                reason: row.try_get("reason")?,
                confidence: row.try_get("confidence")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let dynamic_tags = sqlx::query(
            r#"
            SELECT tag.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, tag.tag_code,
                   tag.value, tag.label, tag.confidence, tag.observed_at,
                   tag.valid_from, tag.valid_to, tag.competition_id,
                   tag.position_code, tag.opponent_team_id, tag.sample_size,
                   tag.source_type, tag.calculation_version
            FROM feature.player_dynamic_tags tag
            JOIN football.players player ON player.id = tag.player_id
            ORDER BY player.normalized_name, tag.tag_code, tag.observed_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerDynamicTagRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                tag_code: row.try_get("tag_code")?,
                value: row.try_get("value")?,
                label: row.try_get("label")?,
                confidence: row.try_get("confidence")?,
                observed_at: row.try_get("observed_at")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
                competition_id: row.try_get("competition_id")?,
                position_code: row.try_get("position_code")?,
                opponent_team_id: row.try_get("opponent_team_id")?,
                sample_size: row.try_get("sample_size")?,
                source_type: row.try_get("source_type")?,
                calculation_version: row.try_get("calculation_version")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let external_ids = sqlx::query(
            r#"
            SELECT provider.code AS provider_code, external.entity_type, external.entity_id,
                   COALESCE(player.canonical_name, team.canonical_name, external.entity_id::text) AS entity_name,
                   external.external_id
            FROM football.external_entity_ids external
            JOIN catalog.data_providers provider ON provider.id = external.provider_id
            LEFT JOIN football.players player ON external.entity_type = 'player' AND player.id = external.entity_id
            LEFT JOIN football.teams team ON external.entity_type = 'team' AND team.id = external.entity_id
            WHERE external.entity_type IN ('player', 'team')
            ORDER BY provider.code, external.entity_type, entity_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| Ok(SpreadsheetExternalIdRow {
            provider_code: row.try_get("provider_code")?, entity_type: row.try_get("entity_type")?,
            entity_id: row.try_get("entity_id")?, entity_name: row.try_get("entity_name")?,
            external_id: row.try_get("external_id")?,
        }))
        .collect::<PersistenceResult<Vec<_>>>()?;
        Ok(SpreadsheetExportData {
            teams,
            players,
            names,
            positions,
            team_periods,
            abilities,
            availability,
            dynamic_tags,
            external_ids,
        })
    }

    async fn validate_spreadsheet_row(
        &self,
        entity_type: SpreadsheetEntityType,
        action: SpreadsheetAction,
        values: &Value,
        context: &SpreadsheetValidationContext<'_>,
    ) -> PersistenceResult<RowValidation> {
        if action == SpreadsheetAction::Skip {
            return Ok(RowValidation::skip(values.clone(), "Excel 行标记为 skip"));
        }
        let mut payload = values
            .as_object()
            .cloned()
            .ok_or_else(|| PersistenceError::InvalidState("Excel 行内容不是对象".to_string()))?;
        normalize_spreadsheet_payload(entity_type, &mut payload)?;
        validate_required_fields(entity_type, action, &payload)?;
        if matches!(action, SpreadsheetAction::Update | SpreadsheetAction::Clear)
            && context.mode == SpreadsheetImportMode::AddOnly
        {
            return Ok(RowValidation::error(
                Value::Object(payload),
                "当前导入模式不允许 update 或 clear",
            ));
        }
        if matches!(action, SpreadsheetAction::Update | SpreadsheetAction::Clear)
            && !matches!(
                entity_type,
                SpreadsheetEntityType::Team
                    | SpreadsheetEntityType::Player
                    | SpreadsheetEntityType::ExternalEntityId
            )
        {
            return Ok(RowValidation::error(
                Value::Object(payload),
                "该工作表保存历史记录，只支持 add 或 skip；修改现状请新增一条生效记录",
            ));
        }
        if action == SpreadsheetAction::Clear
            && entity_type == SpreadsheetEntityType::ExternalEntityId
        {
            return Ok(RowValidation::error(
                Value::Object(payload),
                "外部 ID 关联不支持 clear；需要变更时请使用 update",
            ));
        }
        match entity_type {
            SpreadsheetEntityType::Team => {
                let team_key = text(&payload, "team_key");
                if !team_key.is_empty() && context.duplicate_team_keys.contains(&team_key) {
                    return Ok(RowValidation::error(
                        Value::Object(payload),
                        "team_key 在工作簿中重复",
                    ));
                }
                let matches = self.match_team(&payload).await?;
                decision_from_matches(
                    action,
                    context.mode,
                    Value::Object(payload),
                    matches,
                    "team",
                )
            }
            SpreadsheetEntityType::Player => {
                let player_key = text(&payload, "player_key");
                if !player_key.is_empty() && context.duplicate_player_keys.contains(&player_key) {
                    return Ok(RowValidation::error(
                        Value::Object(payload),
                        "player_key 在工作簿中重复",
                    ));
                }
                validate_player_fields(&payload)?;
                let matches = self.match_player(&payload).await?;
                decision_from_matches(
                    action,
                    context.mode,
                    Value::Object(payload),
                    matches,
                    "player",
                )
            }
            SpreadsheetEntityType::ExternalEntityId => {
                let provider_code = required_text(&payload, "provider_code")?.to_lowercase();
                let provider_id: Option<Uuid> = sqlx::query_scalar(
                    "SELECT id FROM catalog.data_providers WHERE code = $1 AND is_active",
                )
                .bind(&provider_code)
                .fetch_optional(&self.pool)
                .await?;
                let Some(provider_id) = provider_id else {
                    return Ok(RowValidation::error(
                        Value::Object(payload),
                        "数据源代码不存在",
                    ));
                };
                payload.insert("_resolved_provider_id".to_string(), json!(provider_id));
                let entity_kind = required_text(&payload, "entity_type")?;
                if !matches!(entity_kind.as_str(), "player" | "team") {
                    return Ok(RowValidation::error(
                        Value::Object(payload),
                        "entity_type 只能是 player 或 team",
                    ));
                }
                let resolution = if entity_kind == "player" {
                    resolve_player_reference(
                        self,
                        &payload,
                        context.player_keys,
                        context.duplicate_player_keys,
                    )
                    .await?
                } else {
                    resolve_team_reference(
                        self,
                        &payload,
                        context.team_keys,
                        context.duplicate_team_keys,
                        context.external_team_references,
                    )
                    .await?
                };
                let external_id = required_text(&payload, "external_id")?;
                let existing_entity_id: Option<Uuid> = sqlx::query_scalar(
                    r#"
                    SELECT entity_id
                    FROM football.external_entity_ids
                    WHERE provider_id = $1 AND entity_type = $2 AND external_id = $3
                    "#,
                )
                .bind(provider_id)
                .bind(&entity_kind)
                .bind(&external_id)
                .fetch_optional(&self.pool)
                .await?;
                validate_external_id_resolution(
                    &mut payload,
                    resolution,
                    entity_kind.as_str(),
                    existing_entity_id,
                    action,
                    context.mode,
                )
            }
            SpreadsheetEntityType::TeamName
            | SpreadsheetEntityType::Coach
            | SpreadsheetEntityType::CoachName
            | SpreadsheetEntityType::TeamCoachPeriod
            | SpreadsheetEntityType::FormationUsage
            | SpreadsheetEntityType::TeamTacticalObservation
            | SpreadsheetEntityType::TeamAbilityObservation => Ok(RowValidation::error(
                Value::Object(payload),
                "该实体必须使用球队月度工作簿导入",
            )),
            SpreadsheetEntityType::Match
            | SpreadsheetEntityType::Lineup
            | SpreadsheetEntityType::LineupPlayer => Ok(RowValidation::error(
                Value::Object(payload),
                "该实体必须使用比赛与阵容模板导入",
            )),
            SpreadsheetEntityType::PlayerName
            | SpreadsheetEntityType::PlayerPosition
            | SpreadsheetEntityType::PlayerTeamPeriod
            | SpreadsheetEntityType::PlayerAbility
            | SpreadsheetEntityType::PlayerAvailability
            | SpreadsheetEntityType::PlayerDynamicTag => {
                validate_child_fields(entity_type, &payload)?;
                let player_resolution = resolve_player_reference(
                    self,
                    &payload,
                    context.player_keys,
                    context.duplicate_player_keys,
                )
                .await?;
                let validation = apply_resolution(&mut payload, player_resolution, "player")?;
                if validation.status == SpreadsheetRowStatus::Conflict
                    || validation.status == SpreadsheetRowStatus::Error
                {
                    return Ok(validation);
                }
                if matches!(
                    entity_type,
                    SpreadsheetEntityType::PlayerTeamPeriod
                        | SpreadsheetEntityType::PlayerAvailability
                ) {
                    let has_team_reference = !text(&payload, "team_key").is_empty()
                        || !text(&payload, "team_id").is_empty()
                        || !text(&payload, "team_name").is_empty();
                    if has_team_reference {
                        let team_resolution = resolve_team_reference(
                            self,
                            &payload,
                            context.team_keys,
                            context.duplicate_team_keys,
                            context.external_team_references,
                        )
                        .await?;
                        if entity_type == SpreadsheetEntityType::PlayerTeamPeriod
                            && text(&payload, "team_id").is_empty()
                            && text(&payload, "team_key").is_empty()
                            && !text(&payload, "team_name").is_empty()
                            && matches!(&team_resolution, ReferenceResolution::Missing(message) if message == "没有找到匹配球队")
                        {
                            let team_name = text(&payload, "team_name");
                            payload.insert("_auto_create_team".to_string(), json!(true));
                            payload.insert("_auto_create_team_name".to_string(), json!(team_name));
                        } else {
                            let team_validation =
                                apply_resolution(&mut payload, team_resolution, "team")?;
                            if team_validation.status == SpreadsheetRowStatus::Conflict
                                || team_validation.status == SpreadsheetRowStatus::Error
                            {
                                return Ok(team_validation);
                            }
                        }
                    }
                }
                validate_reference_codes(self, entity_type, &mut payload).await?;
                let message =
                    if payload.get("_auto_create_team").and_then(Value::as_bool) == Some(true) {
                        format!(
                            "关联记录已验证；将自动创建球队 {} 并建立球员归属",
                            text(&payload, "_auto_create_team_name")
                        )
                    } else {
                        "关联记录已验证".to_string()
                    };
                Ok(RowValidation {
                    status: SpreadsheetRowStatus::ReadyAdd,
                    message: Some(message),
                    payload: Value::Object(payload),
                    matched_entity_id: validation.matched_entity_id,
                    conflict_candidates: Vec::new(),
                })
            }
        }
    }

    async fn match_team(
        &self,
        payload: &Map<String, Value>,
    ) -> PersistenceResult<Vec<MatchCandidate>> {
        if let Some(id) = optional_uuid(payload, "team_id")? {
            return candidate_by_id(&self.pool, "football.teams", id).await;
        }
        let name = required_text(payload, "official_name")?;
        candidate_teams_by_name(&self.pool, &name).await
    }

    async fn match_player(
        &self,
        payload: &Map<String, Value>,
    ) -> PersistenceResult<Vec<MatchCandidate>> {
        if let Some(id) = optional_uuid(payload, "player_id")? {
            return candidate_by_id(&self.pool, "football.players", id).await;
        }
        let name = required_text(payload, "official_name")?;
        let birth = optional_date(payload, "birth_date")?;
        candidate_players_by_name(&self.pool, &name, birth).await
    }
}

struct SpreadsheetValidationContext<'a> {
    mode: SpreadsheetImportMode,
    player_keys: &'a HashSet<String>,
    team_keys: &'a HashSet<String>,
    duplicate_player_keys: &'a HashSet<String>,
    duplicate_team_keys: &'a HashSet<String>,
    external_team_references: &'a HashMap<String, String>,
}

#[derive(Debug)]
struct RowValidation {
    status: SpreadsheetRowStatus,
    message: Option<String>,
    payload: Value,
    matched_entity_id: Option<Uuid>,
    conflict_candidates: Vec<SpreadsheetConflictCandidate>,
}

impl RowValidation {
    fn skip(payload: Value, message: &str) -> Self {
        Self {
            status: SpreadsheetRowStatus::Skip,
            message: Some(message.to_string()),
            payload,
            matched_entity_id: None,
            conflict_candidates: Vec::new(),
        }
    }
    fn error(payload: Value, message: &str) -> Self {
        Self {
            status: SpreadsheetRowStatus::Error,
            message: Some(message.to_string()),
            payload,
            matched_entity_id: None,
            conflict_candidates: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct MatchCandidate {
    id: Uuid,
    name: String,
    detail: Option<String>,
}

enum ReferenceResolution {
    Resolved(Uuid),
    Deferred(String),
    DeferredExternal { key: String, name: String },
    Conflict(Vec<MatchCandidate>),
    Missing(String),
}

enum ApplyOutcome {
    Inserted,
    Updated,
}

fn canonical_spreadsheet_import_action(
    action: SpreadsheetAction,
    entity_type: SpreadsheetEntityType,
    status: SpreadsheetRowStatus,
) -> SpreadsheetAction {
    if action != SpreadsheetAction::Upsert {
        return action;
    }
    if matches!(
        entity_type,
        SpreadsheetEntityType::Team
            | SpreadsheetEntityType::Player
            | SpreadsheetEntityType::ExternalEntityId
    ) && matches!(
        status,
        SpreadsheetRowStatus::ReadyUpdate | SpreadsheetRowStatus::Conflict
    ) {
        SpreadsheetAction::Update
    } else {
        SpreadsheetAction::Add
    }
}

fn decision_from_matches(
    action: SpreadsheetAction,
    mode: SpreadsheetImportMode,
    mut payload: Value,
    matches: Vec<MatchCandidate>,
    conflict_prefix: &str,
) -> PersistenceResult<RowValidation> {
    if matches.len() > 1 {
        if let Some(object) = payload.as_object_mut() {
            object.insert("_conflict_prefix".to_string(), json!(conflict_prefix));
        }
        return Ok(RowValidation {
            status: SpreadsheetRowStatus::Conflict,
            message: Some("找到多个可能的数据库记录".to_string()),
            payload,
            matched_entity_id: None,
            conflict_candidates: candidates_to_domain(matches),
        });
    }
    if let Some(candidate) = matches.into_iter().next() {
        if action == SpreadsheetAction::Add || mode == SpreadsheetImportMode::AddOnly {
            return Ok(RowValidation {
                status: SpreadsheetRowStatus::Skip,
                message: Some("数据库中已存在匹配记录；当前模式不更新".to_string()),
                payload,
                matched_entity_id: Some(candidate.id),
                conflict_candidates: Vec::new(),
            });
        }
        return Ok(RowValidation {
            status: SpreadsheetRowStatus::ReadyUpdate,
            message: Some(format!("将更新 {}", candidate.name)),
            payload,
            matched_entity_id: Some(candidate.id),
            conflict_candidates: Vec::new(),
        });
    }
    if matches!(action, SpreadsheetAction::Update | SpreadsheetAction::Clear) {
        return Ok(RowValidation::error(
            payload,
            "标记为 update 或 clear，但没有找到数据库记录",
        ));
    }
    Ok(RowValidation {
        status: SpreadsheetRowStatus::ReadyAdd,
        message: Some("将新增记录".to_string()),
        payload,
        matched_entity_id: None,
        conflict_candidates: Vec::new(),
    })
}

async fn resolve_player_reference(
    store: &PostgresStore,
    payload: &Map<String, Value>,
    workbook_keys: &HashSet<String>,
    duplicate_keys: &HashSet<String>,
) -> PersistenceResult<ReferenceResolution> {
    if let Some(id) = optional_uuid(payload, "player_id")? {
        let matches = candidate_by_id(&store.pool, "football.players", id).await?;
        return Ok(matches
            .into_iter()
            .next()
            .map(|item| ReferenceResolution::Resolved(item.id))
            .unwrap_or_else(|| ReferenceResolution::Missing("player_id 不存在".to_string())));
    }
    let key = text(payload, "player_key");
    if !key.is_empty() {
        if duplicate_keys.contains(&key) {
            return Ok(ReferenceResolution::Missing(
                "player_key 在球员基础资料中重复".to_string(),
            ));
        }
        if workbook_keys.contains(&key) {
            return Ok(ReferenceResolution::Deferred(key));
        }
        return Ok(ReferenceResolution::Missing(
            "player_key 未在球员基础资料中定义".to_string(),
        ));
    }
    let name = text(payload, "match_name");
    if name.is_empty() {
        return Ok(ReferenceResolution::Missing(
            "缺少 player_id、player_key 或 match_name".to_string(),
        ));
    }
    let birth = optional_date(payload, "match_birth_date")?;
    let matches = candidate_players_by_name(&store.pool, &name, birth).await?;
    match matches.len() {
        0 => Ok(ReferenceResolution::Missing("没有找到匹配球员".to_string())),
        1 => Ok(ReferenceResolution::Resolved(matches[0].id)),
        _ => Ok(ReferenceResolution::Conflict(matches)),
    }
}

fn normalize_reference_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

async fn resolve_team_reference(
    store: &PostgresStore,
    payload: &Map<String, Value>,
    workbook_keys: &HashSet<String>,
    duplicate_keys: &HashSet<String>,
    external_team_references: &HashMap<String, String>,
) -> PersistenceResult<ReferenceResolution> {
    if let Some(id) = optional_uuid(payload, "team_id")? {
        let matches = candidate_by_id(&store.pool, "football.teams", id).await?;
        return Ok(matches
            .into_iter()
            .next()
            .map(|item| ReferenceResolution::Resolved(item.id))
            .unwrap_or_else(|| ReferenceResolution::Missing("team_id 不存在".to_string())));
    }
    let key = text(payload, "team_key");
    if !key.is_empty() {
        if duplicate_keys.contains(&key) {
            return Ok(ReferenceResolution::Missing(
                "team_key 在球队资料中重复".to_string(),
            ));
        }
        if workbook_keys.contains(&key) {
            return Ok(ReferenceResolution::Deferred(key));
        }
        if let Some(name) = external_team_references.get(&key.to_ascii_uppercase()) {
            return Ok(ReferenceResolution::DeferredExternal {
                key,
                name: name.clone(),
            });
        }
        return Ok(ReferenceResolution::Missing(
            "team_key 未在球队资料中定义".to_string(),
        ));
    }
    let name = text(payload, "team_name");
    if name.is_empty() {
        return Ok(ReferenceResolution::Missing(
            "缺少 team_id、team_key 或 team_name".to_string(),
        ));
    }
    let matches = candidate_teams_by_name(&store.pool, &name).await?;
    match matches.len() {
        0 => {
            let normalized_name = normalize_reference_name(&name);
            let package_matches = external_team_references
                .iter()
                .filter(|(_, package_name)| {
                    normalize_reference_name(package_name) == normalized_name
                })
                .collect::<Vec<_>>();
            if package_matches.len() == 1 {
                let (key, package_name) = package_matches[0];
                Ok(ReferenceResolution::DeferredExternal {
                    key: key.clone(),
                    name: package_name.clone(),
                })
            } else if package_matches.is_empty() {
                Ok(ReferenceResolution::Missing("没有找到匹配球队".to_string()))
            } else {
                Ok(ReferenceResolution::Missing(
                    "完整资料包中存在多个同名球队，无法延迟关联".to_string(),
                ))
            }
        }
        1 => Ok(ReferenceResolution::Resolved(matches[0].id)),
        _ => Ok(ReferenceResolution::Conflict(matches)),
    }
}

fn validate_external_id_resolution(
    payload: &mut Map<String, Value>,
    resolution: ReferenceResolution,
    entity_kind: &str,
    existing_entity_id: Option<Uuid>,
    action: SpreadsheetAction,
    mode: SpreadsheetImportMode,
) -> PersistenceResult<RowValidation> {
    match resolution {
        ReferenceResolution::Resolved(target_id) => {
            if let Some(existing_id) = existing_entity_id {
                if existing_id != target_id {
                    return Ok(RowValidation::error(
                        Value::Object(payload.clone()),
                        "该外部 ID 已绑定到另一条数据库记录，禁止自动改绑",
                    ));
                }
                payload.insert(format!("_resolved_{entity_kind}_id"), json!(target_id));
                if matches!(
                    action,
                    SpreadsheetAction::Update | SpreadsheetAction::Upsert
                ) && mode == SpreadsheetImportMode::AddAndUpdate
                {
                    return Ok(RowValidation {
                        status: SpreadsheetRowStatus::ReadyUpdate,
                        message: Some("外部 ID 已存在，将确认关联信息".to_string()),
                        payload: Value::Object(payload.clone()),
                        matched_entity_id: Some(target_id),
                        conflict_candidates: Vec::new(),
                    });
                }
                return Ok(RowValidation {
                    status: SpreadsheetRowStatus::Skip,
                    message: Some("相同外部 ID 关联已存在".to_string()),
                    payload: Value::Object(payload.clone()),
                    matched_entity_id: Some(target_id),
                    conflict_candidates: Vec::new(),
                });
            }
            if action == SpreadsheetAction::Update {
                return Ok(RowValidation::error(
                    Value::Object(payload.clone()),
                    "标记为 update，但外部 ID 关联不存在",
                ));
            }
            payload.insert(format!("_resolved_{entity_kind}_id"), json!(target_id));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::ReadyAdd,
                message: Some("将新增外部 ID 关联".to_string()),
                payload: Value::Object(payload.clone()),
                matched_entity_id: Some(target_id),
                conflict_candidates: Vec::new(),
            })
        }
        ReferenceResolution::Deferred(key) => {
            if existing_entity_id.is_some() {
                return Ok(RowValidation::error(
                    Value::Object(payload.clone()),
                    "该外部 ID 已存在，不能绑定到工作簿中的新实体",
                ));
            }
            if action == SpreadsheetAction::Update {
                return Ok(RowValidation::error(
                    Value::Object(payload.clone()),
                    "标记为 update，但外部 ID 关联不存在",
                ));
            }
            payload.insert(format!("_deferred_{entity_kind}_key"), json!(key));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::ReadyAdd,
                message: Some(format!("将在同一工作簿中创建并关联{entity_kind}")),
                payload: Value::Object(payload.clone()),
                matched_entity_id: None,
                conflict_candidates: Vec::new(),
            })
        }
        ReferenceResolution::DeferredExternal { key, name } => {
            if existing_entity_id.is_some() {
                return Ok(RowValidation::error(
                    Value::Object(payload.clone()),
                    "该外部 ID 已存在，不能绑定到完整资料包待新增实体",
                ));
            }
            if action == SpreadsheetAction::Update {
                return Ok(RowValidation::error(
                    Value::Object(payload.clone()),
                    "标记为 update，但外部 ID 关联不存在",
                ));
            }
            payload.insert(format!("_deferred_{entity_kind}_key"), json!(key));
            payload.insert(format!("_deferred_{entity_kind}_name"), json!(name));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::ReadyAdd,
                message: Some(format!("将在完整资料包主实体提交后关联{entity_kind}")),
                payload: Value::Object(payload.clone()),
                matched_entity_id: None,
                conflict_candidates: Vec::new(),
            })
        }
        ReferenceResolution::Conflict(matches) => {
            payload.insert("_conflict_prefix".to_string(), json!(entity_kind));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::Conflict,
                message: Some(format!("找到多个可能的{entity_kind}")),
                payload: Value::Object(payload.clone()),
                matched_entity_id: None,
                conflict_candidates: candidates_to_domain(matches),
            })
        }
        ReferenceResolution::Missing(message) => Ok(RowValidation::error(
            Value::Object(payload.clone()),
            &message,
        )),
    }
}

fn apply_resolution(
    payload: &mut Map<String, Value>,
    resolution: ReferenceResolution,
    prefix: &str,
) -> PersistenceResult<RowValidation> {
    match resolution {
        ReferenceResolution::Resolved(id) => {
            payload.insert(format!("_resolved_{prefix}_id"), json!(id));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::ReadyAdd,
                message: None,
                payload: Value::Object(payload.clone()),
                matched_entity_id: Some(id),
                conflict_candidates: Vec::new(),
            })
        }
        ReferenceResolution::Deferred(key) => {
            payload.insert(format!("_deferred_{prefix}_key"), json!(key));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::ReadyAdd,
                message: Some(format!("将在同一工作簿中创建并关联{prefix}")),
                payload: Value::Object(payload.clone()),
                matched_entity_id: None,
                conflict_candidates: Vec::new(),
            })
        }
        ReferenceResolution::DeferredExternal { key, name } => {
            payload.insert(format!("_deferred_{prefix}_key"), json!(key));
            payload.insert(format!("_deferred_{prefix}_name"), json!(name));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::ReadyAdd,
                message: Some(format!("将在完整资料包球队链提交后按名称关联{prefix}")),
                payload: Value::Object(payload.clone()),
                matched_entity_id: None,
                conflict_candidates: Vec::new(),
            })
        }
        ReferenceResolution::Conflict(matches) => {
            payload.insert("_conflict_prefix".to_string(), json!(prefix));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::Conflict,
                message: Some(format!("找到多个可能的{prefix}")),
                payload: Value::Object(payload.clone()),
                matched_entity_id: None,
                conflict_candidates: candidates_to_domain(matches),
            })
        }
        ReferenceResolution::Missing(message) => Ok(RowValidation::error(
            Value::Object(payload.clone()),
            &message,
        )),
    }
}

fn normalize_spreadsheet_payload(
    entity_type: SpreadsheetEntityType,
    payload: &mut Map<String, Value>,
) -> PersistenceResult<()> {
    match entity_type {
        SpreadsheetEntityType::PlayerTeamPeriod => {
            if text(payload, "valid_from").is_empty() {
                let verified_at = text(payload, "verified_at");
                if !verified_at.is_empty() {
                    let date =
                        parse_spreadsheet_datetime(&verified_at, "verified_at")?.date_naive();
                    payload.insert("valid_from".to_string(), Value::String(date.to_string()));
                    payload.insert(
                        "_derived_valid_from".to_string(),
                        Value::String("verified_at".to_string()),
                    );
                }
            }
        }
        SpreadsheetEntityType::PlayerAbility => {
            for key in ["observed_at", "effective_from", "effective_to"] {
                canonicalize_datetime_field(payload, key)?;
            }
        }
        SpreadsheetEntityType::PlayerAvailability => {
            for key in ["valid_from", "valid_to"] {
                canonicalize_datetime_field(payload, key)?;
            }
            normalize_availability_status(payload);
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            for key in ["observed_at", "valid_from", "valid_to"] {
                canonicalize_datetime_field(payload, key)?;
            }
            normalize_dynamic_tag_source_type(payload);
        }
        _ => {}
    }
    Ok(())
}

fn canonicalize_datetime_field(
    payload: &mut Map<String, Value>,
    key: &str,
) -> PersistenceResult<()> {
    let current = text(payload, key);
    if current.is_empty() {
        return Ok(());
    }
    let canonical =
        parse_spreadsheet_datetime(&current, key)?.to_rfc3339_opts(SecondsFormat::AutoSi, true);
    if canonical != current {
        payload.insert(key.to_string(), Value::String(canonical));
    }
    Ok(())
}

fn normalize_availability_status(payload: &mut Map<String, Value>) {
    let original = text(payload, "availability_status");
    if original.is_empty() {
        return;
    }
    let normalized = match original.trim().to_ascii_lowercase().as_str() {
        "questionable" => "doubtful".to_string(),
        "unavailable" => "unavailable".to_string(),
        "available" | "doubtful" | "injured" | "suspended" | "rested" | "returning" | "unknown" => {
            original.trim().to_ascii_lowercase()
        }
        _ => return,
    };
    if normalized != original {
        payload.insert(
            "_availability_status_original".to_string(),
            Value::String(original),
        );
        payload.insert("availability_status".to_string(), Value::String(normalized));
    }
}

fn normalize_dynamic_tag_source_type(payload: &mut Map<String, Value>) {
    let original = text(payload, "source_type");
    if original.is_empty() {
        return;
    }
    let normalized = match original.trim().to_ascii_lowercase().as_str() {
        "manual" | "provider" | "lineup_import" | "ai_analysis" | "match_review"
        | "calculation" => original.trim().to_ascii_lowercase(),
        "official_web_plus_role_model"
        | "public_roster_initialization"
        | "role_model"
        | "model"
        | "computed"
        | "derived" => "calculation".to_string(),
        "official_web" | "web" | "official_source" => "provider".to_string(),
        _ => return,
    };
    if normalized != original {
        payload.insert("_source_type_original".to_string(), Value::String(original));
        payload.insert("source_type".to_string(), Value::String(normalized));
    }
}

fn parse_spreadsheet_datetime(value: &str, key: &str) -> PersistenceResult<DateTime<Utc>> {
    let value = value.trim();
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Ok(parsed.with_timezone(&Utc));
    }
    for format in [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S%.f",
    ] {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(DateTime::from_naive_utc_and_offset(parsed, Utc));
        }
    }
    for format in ["%Y-%m-%d", "%Y/%m/%d", "%Y年%m月%d日"] {
        if let Ok(parsed) = NaiveDate::parse_from_str(value, format) {
            let midnight = parsed.and_hms_opt(0, 0, 0).ok_or_else(|| {
                PersistenceError::InvalidState(format!("{key} 日期无法转换为时间"))
            })?;
            return Ok(DateTime::from_naive_utc_and_offset(midnight, Utc));
        }
    }
    if let Ok(serial) = value.parse::<f64>() {
        if serial.is_finite() && (1.0..=1_000_000.0).contains(&serial) {
            let whole_days = serial.floor() as i64;
            let seconds = ((serial - whole_days as f64) * 86_400.0).round() as i64;
            let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)
                .and_then(|date| date.and_hms_opt(0, 0, 0))
                .ok_or_else(|| PersistenceError::InvalidState("Excel 时间基准无效".into()))?;
            if let Some(parsed) = epoch
                .checked_add_signed(Duration::days(whole_days))
                .and_then(|date_time| date_time.checked_add_signed(Duration::seconds(seconds)))
            {
                return Ok(DateTime::from_naive_utc_and_offset(parsed, Utc));
            }
        }
    }
    Err(PersistenceError::InvalidState(format!(
        "{key} 必须是有效时间；支持 ISO 8601、YYYY-MM-DD、YYYY-MM-DD HH:MM:SS 或 Excel 日期单元格"
    )))
}

async fn validate_reference_codes(
    store: &PostgresStore,
    entity_type: SpreadsheetEntityType,
    payload: &mut Map<String, Value>,
) -> PersistenceResult<()> {
    if entity_type == SpreadsheetEntityType::PlayerPosition {
        let code = required_text(payload, "position_code")?.to_uppercase();
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM football.positions WHERE code = $1)")
                .bind(&code)
                .fetch_one(&store.pool)
                .await?;
        if !exists {
            return Err(PersistenceError::InvalidState(format!(
                "位置代码不存在：{code}"
            )));
        }
    }
    if entity_type == SpreadsheetEntityType::PlayerAbility {
        let code = required_text(payload, "dimension_code")?;
        let bounds = sqlx::query(
            "SELECT minimum_value, maximum_value FROM feature.player_ability_dimensions WHERE code = $1",
        )
        .bind(&code)
        .fetch_optional(&store.pool)
        .await?;
        let Some(bounds) = bounds else {
            return Err(PersistenceError::InvalidState(format!(
                "能力维度不存在：{code}"
            )));
        };
        let minimum: f64 = bounds.try_get("minimum_value")?;
        let maximum: f64 = bounds.try_get("maximum_value")?;
        let value = required_f64(payload, "value")?;
        if value < minimum || value > maximum {
            return Err(PersistenceError::InvalidState(format!(
                "能力值 {value} 超出 {code} 的允许范围 {minimum}–{maximum}"
            )));
        }
    }
    if entity_type == SpreadsheetEntityType::PlayerDynamicTag {
        let code = required_text(payload, "tag_code")?;
        let bounds = sqlx::query(
            "SELECT minimum_value, maximum_value, default_ttl_hours FROM feature.player_dynamic_tag_definitions WHERE code = $1",
        )
        .bind(&code)
        .fetch_optional(&store.pool)
        .await?;
        let Some(bounds) = bounds else {
            return Err(PersistenceError::InvalidState(format!(
                "动态标签不存在：{code}"
            )));
        };
        let minimum: f64 = bounds.try_get("minimum_value")?;
        let maximum: f64 = bounds.try_get("maximum_value")?;
        let default_ttl_hours: i32 = bounds.try_get("default_ttl_hours")?;
        let value = required_f64(payload, "tag_value")?;
        if value < minimum || value > maximum {
            return Err(PersistenceError::InvalidState(format!(
                "动态标签值 {value} 超出 {code} 的允许范围 {minimum}–{maximum}"
            )));
        }
        let valid_from = required_datetime(payload, "valid_from")?;
        let valid_to = match optional_datetime(payload, "valid_to")? {
            Some(value) => value,
            None => {
                let value = valid_from
                    .checked_add_signed(Duration::hours(i64::from(default_ttl_hours)))
                    .ok_or_else(|| {
                        PersistenceError::InvalidState(
                            "动态标签默认失效时间超出允许范围".to_string(),
                        )
                    })?;
                payload.insert(
                    "valid_to".to_string(),
                    Value::String(value.to_rfc3339_opts(SecondsFormat::AutoSi, true)),
                );
                value
            }
        };
        if valid_to <= valid_from {
            return Err(PersistenceError::InvalidState(
                "动态标签失效时间必须晚于生效时间".to_string(),
            ));
        }
        let source_type = default_text(payload, "source_type", "manual");
        if !matches!(
            source_type.as_str(),
            "manual"
                | "provider"
                | "lineup_import"
                | "ai_analysis"
                | "match_review"
                | "calculation"
        ) {
            return Err(PersistenceError::InvalidState(format!(
                "动态标签来源类型不受支持：{source_type}"
            )));
        }
        if let Some(position_code) = optional_text(payload, "position_code") {
            let exists: bool = sqlx::query_scalar(
                "SELECT EXISTS(SELECT 1 FROM football.positions WHERE code = $1)",
            )
            .bind(position_code.to_uppercase())
            .fetch_one(&store.pool)
            .await?;
            if !exists {
                return Err(PersistenceError::InvalidState(
                    "动态标签位置代码不存在".to_string(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_required_fields(
    entity_type: SpreadsheetEntityType,
    action: SpreadsheetAction,
    payload: &Map<String, Value>,
) -> PersistenceResult<()> {
    let fields: &[&str] = match entity_type {
        SpreadsheetEntityType::Team
            if matches!(action, SpreadsheetAction::Add | SpreadsheetAction::Upsert) =>
        {
            &["official_name"]
        }
        SpreadsheetEntityType::Player
            if matches!(action, SpreadsheetAction::Add | SpreadsheetAction::Upsert) =>
        {
            &["official_name"]
        }
        SpreadsheetEntityType::Team | SpreadsheetEntityType::Player => &[],
        SpreadsheetEntityType::PlayerName => &["name_value"],
        SpreadsheetEntityType::PlayerPosition => &["position_code", "proficiency"],
        SpreadsheetEntityType::PlayerTeamPeriod => &["valid_from", "registration_status"],
        SpreadsheetEntityType::PlayerAbility => &[
            "dimension_code",
            "value",
            "observed_at",
            "effective_from",
            "calculation_version",
        ],
        SpreadsheetEntityType::PlayerAvailability => &["availability_status", "valid_from"],
        SpreadsheetEntityType::PlayerDynamicTag => &[
            "tag_code",
            "tag_value",
            "observed_at",
            "valid_from",
            "calculation_version",
        ],
        SpreadsheetEntityType::ExternalEntityId => &["provider_code", "entity_type", "external_id"],
        SpreadsheetEntityType::TeamName
        | SpreadsheetEntityType::Coach
        | SpreadsheetEntityType::CoachName
        | SpreadsheetEntityType::TeamCoachPeriod
        | SpreadsheetEntityType::FormationUsage
        | SpreadsheetEntityType::TeamTacticalObservation
        | SpreadsheetEntityType::TeamAbilityObservation
        | SpreadsheetEntityType::Match
        | SpreadsheetEntityType::Lineup
        | SpreadsheetEntityType::LineupPlayer => &[],
    };
    for field in fields {
        required_text(payload, field)?;
    }
    Ok(())
}

fn validate_player_fields(payload: &Map<String, Value>) -> PersistenceResult<()> {
    if let Some(height) = optional_i16(payload, "height_cm")? {
        if !(120..=230).contains(&height) {
            return Err(PersistenceError::InvalidState(
                "身高必须为 120–230 cm".to_string(),
            ));
        }
    }
    let foot = text(payload, "preferred_foot");
    if !foot.is_empty() && !matches!(foot.as_str(), "left" | "right" | "both" | "unknown") {
        return Err(PersistenceError::InvalidState(format!(
            "未知惯用脚：{foot}"
        )));
    }
    let status = text(payload, "player_status");
    if !status.is_empty()
        && !matches!(
            status.as_str(),
            "active" | "inactive" | "retired" | "unknown"
        )
    {
        return Err(PersistenceError::InvalidState(format!(
            "未知球员状态：{status}"
        )));
    }
    optional_date(payload, "birth_date")?;
    Ok(())
}

fn validate_child_fields(
    entity_type: SpreadsheetEntityType,
    payload: &Map<String, Value>,
) -> PersistenceResult<()> {
    match entity_type {
        SpreadsheetEntityType::PlayerName => {
            let from = optional_date(payload, "valid_from")?;
            let to = optional_date(payload, "valid_to")?;
            validate_date_range(from, to, "名称有效期")?;
        }
        SpreadsheetEntityType::PlayerPosition => {
            let proficiency = required_f64(payload, "proficiency")?;
            if !(0.0..=1.0).contains(&proficiency) {
                return Err(PersistenceError::InvalidState(
                    "位置熟练度必须为 0–1".to_string(),
                ));
            }
            if let Some(role_code) = optional_text(payload, "default_role_code") {
                if role_code.chars().count() > 80 {
                    return Err(PersistenceError::InvalidState(
                        "默认战术角色不能超过 80 个字符".to_string(),
                    ));
                }
            }
            let from = optional_date(payload, "valid_from")?;
            let to = optional_date(payload, "valid_to")?;
            validate_date_range(from, to, "位置有效期")?;
        }
        SpreadsheetEntityType::PlayerTeamPeriod => {
            let from = required_date(payload, "valid_from")?;
            let to = optional_date(payload, "valid_to")?;
            validate_date_range(Some(from), to, "球队履历")?;
            let registration = required_text(payload, "registration_status")?;
            if !matches!(
                registration.as_str(),
                "registered" | "loan" | "trial" | "released" | "unknown"
            ) {
                return Err(PersistenceError::InvalidState(format!(
                    "未知注册状态：{registration}"
                )));
            }
            if let Some(number) = optional_i16(payload, "squad_number")? {
                if !(0..=99).contains(&number) {
                    return Err(PersistenceError::InvalidState(
                        "球衣号码必须为 0–99".to_string(),
                    ));
                }
            }
        }
        SpreadsheetEntityType::PlayerAbility => {
            let confidence = optional_f64(payload, "confidence")?.unwrap_or(1.0);
            if !(0.0..=1.0).contains(&confidence) {
                return Err(PersistenceError::InvalidState(
                    "能力可信度必须为 0–1".to_string(),
                ));
            }
            if optional_i32(payload, "sample_size")?.unwrap_or(1) < 0 {
                return Err(PersistenceError::InvalidState(
                    "能力样本量不能为负数".to_string(),
                ));
            }
            required_datetime(payload, "observed_at")?;
            let from = required_datetime(payload, "effective_from")?;
            let to = optional_datetime(payload, "effective_to")?;
            validate_datetime_range(Some(from), to, "能力有效期")?;
        }
        SpreadsheetEntityType::PlayerAvailability => {
            let status = required_text(payload, "availability_status")?;
            if !matches!(
                status.as_str(),
                "available"
                    | "doubtful"
                    | "unavailable"
                    | "injured"
                    | "suspended"
                    | "rested"
                    | "returning"
                    | "unknown"
            ) {
                return Err(PersistenceError::InvalidState(format!(
                    "未知可用状态：{status}"
                )));
            }
            let confidence = optional_f64(payload, "confidence")?.unwrap_or(1.0);
            if !(0.0..=1.0).contains(&confidence) {
                return Err(PersistenceError::InvalidState(
                    "可用状态可信度必须为 0–1".to_string(),
                ));
            }
            let from = required_datetime(payload, "valid_from")?;
            let to = optional_datetime(payload, "valid_to")?;
            validate_datetime_range(Some(from), to, "可用状态有效期")?;
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            let confidence = optional_f64(payload, "confidence")?.unwrap_or(1.0);
            if !(0.0..=1.0).contains(&confidence) {
                return Err(PersistenceError::InvalidState(
                    "动态标签可信度必须为 0–1".to_string(),
                ));
            }
            if optional_i32(payload, "sample_size")?.unwrap_or(1) < 0 {
                return Err(PersistenceError::InvalidState(
                    "动态标签样本量不能为负数".to_string(),
                ));
            }
            required_datetime(payload, "observed_at")?;
            let from = required_datetime(payload, "valid_from")?;
            if let Some(to) = optional_datetime(payload, "valid_to")? {
                if to <= from {
                    return Err(PersistenceError::InvalidState(
                        "动态标签失效时间必须晚于生效时间".to_string(),
                    ));
                }
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_date_range(
    from: Option<NaiveDate>,
    to: Option<NaiveDate>,
    label: &str,
) -> PersistenceResult<()> {
    if let (Some(from), Some(to)) = (from, to) {
        if to < from {
            return Err(PersistenceError::InvalidState(format!(
                "{label}的结束日期不能早于开始日期"
            )));
        }
    }
    Ok(())
}

fn validate_datetime_range(
    from: Option<DateTime<Utc>>,
    to: Option<DateTime<Utc>>,
    label: &str,
) -> PersistenceResult<()> {
    if let (Some(from), Some(to)) = (from, to) {
        if to < from {
            return Err(PersistenceError::InvalidState(format!(
                "{label}的结束时间不能早于开始时间"
            )));
        }
    }
    Ok(())
}

async fn insert_import_row(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    row: &SpreadsheetImportRow,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
        INSERT INTO catalog.import_rows (
            id, batch_id, sheet_name, row_number, entity_type,
            requested_action, status, message, payload, matched_entity_id,
            conflict_candidates
        ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
        "#,
    )
    .bind(row.id)
    .bind(batch_id)
    .bind(&row.sheet_name)
    .bind(row.row_number as i32)
    .bind(row.entity_type.as_str())
    .bind(row.action.as_str())
    .bind(row.status.as_str())
    .bind(&row.message)
    .bind(&row.payload)
    .bind(row.matched_entity_id)
    .bind(serde_json::to_value(&row.conflict_candidates)?)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

struct ImportCommitContext<'a> {
    player_keys: &'a mut HashMap<String, Uuid>,
    team_keys: &'a mut HashMap<String, Uuid>,
}

async fn apply_import_row(
    tx: &mut Transaction<'_, Postgres>,
    entity_type: SpreadsheetEntityType,
    action: SpreadsheetAction,
    status: SpreadsheetRowStatus,
    payload: &Value,
    matched_entity_id: Option<Uuid>,
    context: &mut ImportCommitContext<'_>,
) -> PersistenceResult<ApplyOutcome> {
    let values = payload
        .as_object()
        .ok_or_else(|| PersistenceError::InvalidState("导入 payload 无效".to_string()))?;
    match entity_type {
        SpreadsheetEntityType::TeamName
        | SpreadsheetEntityType::Coach
        | SpreadsheetEntityType::CoachName
        | SpreadsheetEntityType::TeamCoachPeriod
        | SpreadsheetEntityType::FormationUsage
        | SpreadsheetEntityType::TeamTacticalObservation
        | SpreadsheetEntityType::TeamAbilityObservation => Err(PersistenceError::InvalidState(
            "球队月度实体必须使用球队月度导入流程".to_string(),
        )),
        SpreadsheetEntityType::Match
        | SpreadsheetEntityType::Lineup
        | SpreadsheetEntityType::LineupPlayer => Err(PersistenceError::InvalidState(
            "比赛与阵容必须使用专用导入流程".to_string(),
        )),
        SpreadsheetEntityType::Team => {
            let id = match matched_entity_id {
                Some(id) => id,
                None => optional_uuid(values, "team_id")?.unwrap_or_else(Uuid::new_v4),
            };
            let metadata = spreadsheet_row_metadata(values);
            let outcome = if matched_entity_id.is_some() {
                let name = text(values, "official_name");
                let clear = if action == SpreadsheetAction::Clear {
                    spreadsheet_clear_fields(values)
                } else {
                    HashSet::new()
                };
                sqlx::query(
                    r#"UPDATE football.teams SET
                    canonical_name=COALESCE(NULLIF($2,''),canonical_name),
                    normalized_name=CASE WHEN NULLIF($2,'') IS NULL THEN normalized_name ELSE $3 END,
                    country_code=CASE WHEN $4 THEN NULL ELSE COALESCE(NULLIF($5,''),country_code) END,
                    is_active=COALESCE($6,is_active), metadata=metadata || $7, updated_at=now()
                    WHERE id=$1"#,
                )
                .bind(id)
                .bind(&name)
                .bind(normalize_name(&name))
                .bind(clear.contains("country_code"))
                .bind(optional_text(values, "country_code"))
                .bind(optional_bool(values, "is_active")?)
                .bind(&metadata)
                .execute(&mut **tx)
                .await?;
                ApplyOutcome::Updated
            } else {
                let name = required_text(values, "official_name")?;
                sqlx::query("INSERT INTO football.teams (id, canonical_name, normalized_name, country_code, is_active, metadata) VALUES ($1,$2,$3,$4,$5,$6)")
                    .bind(id).bind(&name).bind(normalize_name(&name))
                    .bind(optional_text(values, "country_code"))
                    .bind(optional_bool(values, "is_active")?.unwrap_or(true))
                    .bind(&metadata).execute(&mut **tx).await?;
                ApplyOutcome::Inserted
            };
            let key = text(values, "team_key");
            if !key.is_empty() {
                context.team_keys.insert(key, id);
            }
            Ok(outcome)
        }
        SpreadsheetEntityType::Player => {
            let id = match matched_entity_id {
                Some(id) => id,
                None => optional_uuid(values, "player_id")?.unwrap_or_else(Uuid::new_v4),
            };
            let metadata = spreadsheet_row_metadata(values);
            let outcome = if matched_entity_id.is_some() {
                let name = text(values, "official_name");
                let normalized_name = normalize_name(&name);
                let clear = if action == SpreadsheetAction::Clear {
                    spreadsheet_clear_fields(values)
                } else {
                    HashSet::new()
                };
                let previous_normalized_name: String = sqlx::query_scalar(
                    "SELECT normalized_name FROM football.players WHERE id = $1 FOR UPDATE",
                )
                .bind(id)
                .fetch_one(&mut **tx)
                .await?;
                sqlx::query(
                    r#"UPDATE football.players SET
                    canonical_name=COALESCE(NULLIF($2,''),canonical_name),
                    normalized_name=CASE WHEN NULLIF($2,'') IS NULL THEN normalized_name ELSE $3 END,
                    date_of_birth=CASE WHEN $4 THEN NULL ELSE COALESCE($5,date_of_birth) END,
                    nationality_code=CASE WHEN $6 THEN NULL ELSE COALESCE(NULLIF($7,''),nationality_code) END,
                    preferred_foot=CASE WHEN $8 THEN 'unknown' ELSE COALESCE(NULLIF($9,''),preferred_foot) END,
                    height_cm=CASE WHEN $10 THEN NULL ELSE COALESCE($11,height_cm) END,
                    status=CASE WHEN $12 THEN 'unknown' ELSE COALESCE(NULLIF($13,''),status) END,
                    metadata=metadata || $14, updated_at=now() WHERE id=$1"#,
                )
                .bind(id)
                .bind(&name)
                .bind(&normalized_name)
                .bind(clear.contains("birth_date"))
                .bind(optional_date(values, "birth_date")?)
                .bind(clear.contains("nationality_code"))
                .bind(optional_text(values, "nationality_code"))
                .bind(clear.contains("preferred_foot"))
                .bind(optional_text(values, "preferred_foot"))
                .bind(clear.contains("height_cm"))
                .bind(optional_i16(values, "height_cm")?)
                .bind(clear.contains("player_status"))
                .bind(optional_text(values, "player_status"))
                .bind(&metadata)
                .execute(&mut **tx)
                .await?;
                if !name.is_empty() && previous_normalized_name != normalized_name {
                    sqlx::query(
                        "UPDATE football.player_names SET is_primary=false WHERE player_id=$1",
                    )
                    .bind(id)
                    .execute(&mut **tx)
                    .await?;
                    sqlx::query("INSERT INTO football.player_names (id,player_id,name,normalized_name,is_primary,metadata) VALUES ($1,$2,$3,$4,true,$5)")
                        .bind(Uuid::new_v4()).bind(id).bind(&name).bind(&normalized_name)
                        .bind(&metadata).execute(&mut **tx).await?;
                }
                ApplyOutcome::Updated
            } else {
                let name = required_text(values, "official_name")?;
                let birth = optional_date(values, "birth_date")?;
                let nationality = optional_text(values, "nationality_code");
                let foot = default_text(values, "preferred_foot", "unknown");
                let height = optional_i16(values, "height_cm")?;
                let player_status = default_text(values, "player_status", "active");
                let normalized_name = normalize_name(&name);
                sqlx::query("INSERT INTO football.players (id, canonical_name, normalized_name, date_of_birth, nationality_code, preferred_foot, height_cm, status, metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)")
                    .bind(id).bind(&name).bind(&normalized_name).bind(birth).bind(nationality)
                    .bind(&foot).bind(height).bind(&player_status).bind(&metadata)
                    .execute(&mut **tx).await?;
                sqlx::query("INSERT INTO football.player_names (id, player_id, name, normalized_name, is_primary, metadata) VALUES ($1,$2,$3,$4,true,$5)")
                    .bind(Uuid::new_v4()).bind(id).bind(&name).bind(&normalized_name)
                    .bind(&metadata).execute(&mut **tx).await?;
                ApplyOutcome::Inserted
            };
            let key = text(values, "player_key");
            if !key.is_empty() {
                context.player_keys.insert(key, id);
            }
            Ok(outcome)
        }
        SpreadsheetEntityType::PlayerName => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            let name = required_text(values, "name_value")?;
            let is_primary = optional_bool(values, "is_primary")?.unwrap_or(false);
            if is_primary {
                sqlx::query("UPDATE football.player_names SET is_primary=false WHERE player_id=$1")
                    .bind(player_id)
                    .execute(&mut **tx)
                    .await?;
                sqlx::query("UPDATE football.players SET canonical_name=$2, normalized_name=$3, updated_at=now() WHERE id=$1").bind(player_id).bind(&name).bind(normalize_name(&name)).execute(&mut **tx).await?;
            }
            sqlx::query("INSERT INTO football.player_names (id,player_id,name,normalized_name,language_code,is_primary,valid_from,valid_to,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)")
                .bind(Uuid::new_v4()).bind(player_id).bind(&name).bind(normalize_name(&name)).bind(optional_text(values,"language_code"))
                .bind(is_primary).bind(optional_date(values,"valid_from")?).bind(optional_date(values,"valid_to")?)
                .bind(spreadsheet_row_metadata(values)).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerPosition => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            let code = required_text(values, "position_code")?.to_uppercase();
            let is_primary = optional_bool(values, "is_primary")?.unwrap_or(false);
            if is_primary {
                sqlx::query(
                    "UPDATE football.player_positions SET is_primary=false WHERE player_id=$1",
                )
                .bind(player_id)
                .execute(&mut **tx)
                .await?;
            }
            let default_role_code = optional_text(values, "default_role_code")
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty());
            let clear_default_role = action == SpreadsheetAction::Clear
                && spreadsheet_clear_fields(values).contains("default_role_code");
            sqlx::query("INSERT INTO football.player_positions (id,player_id,position_code,proficiency,default_role_code,is_primary,valid_from,valid_to,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9) ON CONFLICT (player_id, position_code, valid_from) DO UPDATE SET proficiency=EXCLUDED.proficiency,default_role_code=CASE WHEN $10 THEN NULL WHEN EXCLUDED.default_role_code IS NULL THEN football.player_positions.default_role_code ELSE EXCLUDED.default_role_code END,is_primary=EXCLUDED.is_primary,valid_to=EXCLUDED.valid_to,metadata=football.player_positions.metadata||EXCLUDED.metadata")
                .bind(Uuid::new_v4()).bind(player_id).bind(code).bind(required_f64(values,"proficiency")?).bind(default_role_code).bind(is_primary)
                .bind(optional_date(values,"valid_from")?).bind(optional_date(values,"valid_to")?)
                .bind(spreadsheet_row_metadata(values)).bind(clear_default_role).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerTeamPeriod => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            let team_id = if values.get("_auto_create_team").and_then(Value::as_bool) == Some(true)
            {
                resolve_or_create_import_team(tx, values).await?
            } else {
                resolve_committed_team_id(tx, values, context).await?
            };
            sqlx::query("INSERT INTO football.player_team_periods (id,player_id,team_id,season_id,squad_number,valid_from,valid_to,registration_status,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)")
                .bind(Uuid::new_v4()).bind(player_id).bind(team_id).bind(optional_uuid(values,"season_id")?).bind(optional_i16(values,"squad_number")?)
                .bind(required_date(values,"valid_from")?).bind(optional_date(values,"valid_to")?).bind(default_text(values,"registration_status","registered"))
                .bind(spreadsheet_row_metadata(values)).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerAbility => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            sqlx::query("INSERT INTO feature.player_ability_observations (id,player_id,dimension_code,context_type,context_id,value,confidence,sample_size,observed_at,effective_from,effective_to,calculation_version,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)")
                .bind(Uuid::new_v4()).bind(player_id).bind(required_text(values,"dimension_code")?).bind(default_text(values,"context_type","general"))
                .bind(optional_uuid(values,"context_id")?).bind(required_f64(values,"value")?).bind(optional_f64(values,"confidence")?.unwrap_or(1.0))
                .bind(optional_i32(values,"sample_size")?.unwrap_or(1)).bind(required_datetime(values,"observed_at")?).bind(required_datetime(values,"effective_from")?)
                .bind(optional_datetime(values,"effective_to")?).bind(required_text(values,"calculation_version")?).bind(spreadsheet_row_metadata(values)).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerAvailability => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            let team_id = resolve_optional_committed_team_id(tx, values, context).await?;
            sqlx::query("INSERT INTO football.player_availability (id,player_id,team_id,competition_id,status,reason,confidence,valid_from,valid_to,metadata) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10)")
                .bind(Uuid::new_v4()).bind(player_id).bind(team_id).bind(optional_uuid(values,"competition_id")?).bind(required_text(values,"availability_status")?)
                .bind(optional_text(values,"reason")).bind(optional_f64(values,"confidence")?.unwrap_or(1.0)).bind(required_datetime(values,"valid_from")?)
                .bind(optional_datetime(values,"valid_to")?).bind(spreadsheet_row_metadata(values)).execute(&mut **tx).await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            let player_id =
                resolve_committed_id(values, "player", context.player_keys, context.team_keys)?;
            sqlx::query(
                r#"
                INSERT INTO feature.player_dynamic_tags (
                    id, player_id, tag_code, value, label, confidence,
                    observed_at, valid_from, valid_to, competition_id,
                    position_code, opponent_team_id, sample_size, source_type,
                    calculation_version, metadata
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    $7, $8, $9, $10,
                    $11, $12, $13, $14,
                    $15, $16
                )
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(player_id)
            .bind(required_text(values, "tag_code")?)
            .bind(required_f64(values, "tag_value")?)
            .bind(optional_text(values, "label"))
            .bind(optional_f64(values, "confidence")?.unwrap_or(1.0))
            .bind(required_datetime(values, "observed_at")?)
            .bind(required_datetime(values, "valid_from")?)
            .bind(required_datetime(values, "valid_to")?)
            .bind(optional_uuid(values, "competition_id")?)
            .bind(optional_text(values, "position_code").map(|value| value.to_uppercase()))
            .bind(optional_uuid(values, "opponent_team_id")?)
            .bind(optional_i32(values, "sample_size")?.unwrap_or(1))
            .bind(default_text(values, "source_type", "manual"))
            .bind(required_text(values, "calculation_version")?)
            .bind(spreadsheet_row_metadata(values))
            .execute(&mut **tx)
            .await?;
            Ok(ApplyOutcome::Inserted)
        }
        SpreadsheetEntityType::ExternalEntityId => {
            let provider_id = payload_uuid(values, "_resolved_provider_id")?;
            let entity_type = required_text(values, "entity_type")?;
            let entity_id = if entity_type == "team" {
                resolve_committed_team_id(tx, values, context).await?
            } else {
                resolve_committed_id(values, &entity_type, context.player_keys, context.team_keys)?
            };
            sqlx::query("INSERT INTO football.external_entity_ids (id,provider_id,entity_type,entity_id,external_id,metadata) VALUES ($1,$2,$3,$4,$5,$6) ON CONFLICT (provider_id,entity_type,external_id) DO UPDATE SET entity_id=EXCLUDED.entity_id, metadata=football.external_entity_ids.metadata||EXCLUDED.metadata")
                .bind(Uuid::new_v4()).bind(provider_id).bind(&entity_type).bind(entity_id).bind(required_text(values,"external_id")?)
                .bind(spreadsheet_row_metadata(values)).execute(&mut **tx).await?;
            Ok(if status == SpreadsheetRowStatus::ReadyUpdate {
                ApplyOutcome::Updated
            } else {
                ApplyOutcome::Inserted
            })
        }
    }
}

async fn resolve_or_create_import_team(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
) -> PersistenceResult<Uuid> {
    let name = required_text(values, "_auto_create_team_name")?;
    let normalized = normalize_name(&name);
    sqlx::query("SELECT pg_advisory_xact_lock(hashtextextended($1, 0))")
        .bind(&normalized)
        .execute(&mut **tx)
        .await?;
    let matches = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT team.id
        FROM football.teams team
        WHERE team.normalized_name = $1
           OR EXISTS (
                SELECT 1
                FROM football.team_names alias
                WHERE alias.team_id = team.id
                  AND alias.normalized_name = $1
           )
        ORDER BY team.id
        LIMIT 2
        "#,
    )
    .bind(&normalized)
    .fetch_all(&mut **tx)
    .await?;
    match matches.as_slice() {
        [team_id] => Ok(*team_id),
        [] => {
            let team_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO football.teams (id,canonical_name,normalized_name,is_active,metadata) VALUES ($1,$2,$3,true,$4)",
            )
            .bind(team_id)
            .bind(&name)
            .bind(&normalized)
            .bind(json!({"source":"player_spreadsheet_auto_create","requires_profile_completion":true}))
            .execute(&mut **tx)
            .await?;
            sqlx::query(
                "INSERT INTO audit.events (id,event_type,entity_type,entity_id,payload) VALUES ($1,'team_created_from_player_import','team',$2,$3)",
            )
            .bind(Uuid::new_v4())
            .bind(team_id.to_string())
            .bind(json!({"canonical_name":name,"normalized_name":normalized}))
            .execute(&mut **tx)
            .await?;
            Ok(team_id)
        }
        _ => Err(PersistenceError::InvalidState(format!(
            "球队名称 {name} 匹配到多条记录，不能自动创建或关联"
        ))),
    }
}

async fn resolve_committed_team_id(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
    context: &ImportCommitContext<'_>,
) -> PersistenceResult<Uuid> {
    if let Some(id) = payload_optional_uuid(values, "_resolved_team_id")? {
        return Ok(id);
    }
    if let Some(id) = optional_uuid(values, "team_id")? {
        return Ok(id);
    }
    let key = text(values, "_deferred_team_key");
    if !key.is_empty() {
        if let Some(id) = context.team_keys.get(&key) {
            return Ok(*id);
        }
        let normalized_key = normalize_name(&key);
        let key_matches = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT team.id
            FROM football.teams team
            LEFT JOIN football.team_profiles profile ON profile.team_id = team.id
            WHERE team.normalized_name = $1
               OR lower(btrim(COALESCE(profile.short_name, ''))) = $1
               OR EXISTS (
                    SELECT 1
                    FROM football.team_names alias
                    WHERE alias.team_id = team.id
                      AND alias.normalized_name = $1
               )
            ORDER BY team.id
            LIMIT 2
            "#,
        )
        .bind(&normalized_key)
        .fetch_all(&mut **tx)
        .await?;
        match key_matches.as_slice() {
            [team_id] => return Ok(*team_id),
            [] => {}
            _ => {
                return Err(PersistenceError::InvalidState(format!(
                    "完整资料包球队简称 {key} 匹配到多条记录，不能自动关联"
                )));
            }
        }
    }
    let name = ["_deferred_team_name", "team_name"]
        .into_iter()
        .map(|field| text(values, field))
        .find(|value| !value.is_empty())
        .unwrap_or_default();
    if name.is_empty() {
        return Err(PersistenceError::InvalidState(format!(
            "无法解析导入关联：team {key}"
        )));
    }
    let normalized = normalize_name(&name);
    let matches = sqlx::query_scalar::<_, Uuid>(
        r#"
        SELECT team.id
        FROM football.teams team
        WHERE team.normalized_name = $1
           OR EXISTS (
                SELECT 1
                FROM football.team_names alias
                WHERE alias.team_id = team.id
                  AND alias.normalized_name = $1
           )
        ORDER BY team.id
        LIMIT 2
        "#,
    )
    .bind(&normalized)
    .fetch_all(&mut **tx)
    .await?;
    match matches.as_slice() {
        [team_id] => Ok(*team_id),
        [] => Err(PersistenceError::InvalidState(format!(
            "完整资料包球队链提交后仍无法解析球队：{name}"
        ))),
        _ => Err(PersistenceError::InvalidState(format!(
            "球队名称 {name} 匹配到多条记录，不能自动关联"
        ))),
    }
}

async fn resolve_optional_committed_team_id(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
    context: &ImportCommitContext<'_>,
) -> PersistenceResult<Option<Uuid>> {
    let has_reference = payload_optional_uuid(values, "_resolved_team_id")?.is_some()
        || optional_uuid(values, "team_id")?.is_some()
        || !text(values, "_deferred_team_key").is_empty()
        || !text(values, "_deferred_team_name").is_empty()
        || !text(values, "team_name").is_empty();
    if has_reference {
        resolve_committed_team_id(tx, values, context)
            .await
            .map(Some)
    } else {
        Ok(None)
    }
}

fn resolve_committed_id(
    values: &Map<String, Value>,
    prefix: &str,
    player_keys: &HashMap<String, Uuid>,
    team_keys: &HashMap<String, Uuid>,
) -> PersistenceResult<Uuid> {
    if let Some(id) = payload_optional_uuid(values, &format!("_resolved_{prefix}_id"))? {
        return Ok(id);
    }
    let key = text(values, &format!("_deferred_{prefix}_key"));
    let value = if prefix == "player" {
        player_keys.get(&key)
    } else {
        team_keys.get(&key)
    };
    value
        .copied()
        .ok_or_else(|| PersistenceError::InvalidState(format!("无法解析导入关联：{prefix} {key}")))
}
fn collect_keys(
    rows: &[football_domain::SpreadsheetRawRow],
    entity: SpreadsheetEntityType,
    field: &str,
) -> HashSet<String> {
    rows.iter()
        .filter(|r| r.entity_type == entity)
        .filter_map(|r| r.values.get(field).and_then(Value::as_str))
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
        .collect()
}
fn duplicate_keys(
    rows: &[football_domain::SpreadsheetRawRow],
    entity: SpreadsheetEntityType,
    field: &str,
) -> HashSet<String> {
    let mut seen = HashSet::new();
    let mut duplicate = HashSet::new();
    for key in rows
        .iter()
        .filter(|r| r.entity_type == entity)
        .filter_map(|r| r.values.get(field).and_then(Value::as_str))
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        if !seen.insert(key.to_string()) {
            duplicate.insert(key.to_string());
        }
    }
    duplicate
}

async fn candidate_by_id(
    pool: &sqlx::PgPool,
    table: &str,
    id: Uuid,
) -> PersistenceResult<Vec<MatchCandidate>> {
    let sql = format!("SELECT id, canonical_name FROM {table} WHERE id=$1");
    let rows = sqlx::query(&sql).bind(id).fetch_all(pool).await?;
    rows.iter()
        .map(|row| {
            Ok(MatchCandidate {
                id: row.try_get("id")?,
                name: row.try_get("canonical_name")?,
                detail: None,
            })
        })
        .collect()
}
async fn candidate_teams_by_name(
    pool: &sqlx::PgPool,
    name: &str,
) -> PersistenceResult<Vec<MatchCandidate>> {
    let rows=sqlx::query("SELECT id,canonical_name,country_code FROM football.teams WHERE normalized_name=$1 OR EXISTS(SELECT 1 FROM football.team_names n WHERE n.team_id=football.teams.id AND n.normalized_name=$1) ORDER BY canonical_name LIMIT 10")
        .bind(normalize_name(name)).fetch_all(pool).await?;
    rows.iter()
        .map(|row| {
            Ok(MatchCandidate {
                id: row.try_get("id")?,
                name: row.try_get("canonical_name")?,
                detail: row.try_get::<Option<String>, _>("country_code")?,
            })
        })
        .collect()
}
async fn candidate_players_by_name(
    pool: &sqlx::PgPool,
    name: &str,
    birth: Option<NaiveDate>,
) -> PersistenceResult<Vec<MatchCandidate>> {
    let rows=sqlx::query("SELECT id,canonical_name,date_of_birth,nationality_code FROM football.players WHERE (normalized_name=$1 OR EXISTS(SELECT 1 FROM football.player_names n WHERE n.player_id=football.players.id AND n.normalized_name=$1)) AND ($2::date IS NULL OR date_of_birth=$2) ORDER BY canonical_name LIMIT 10")
        .bind(normalize_name(name)).bind(birth).fetch_all(pool).await?;
    rows.iter()
        .map(|row| {
            let date: Option<NaiveDate> = row.try_get("date_of_birth")?;
            let nationality: Option<String> = row.try_get("nationality_code")?;
            Ok(MatchCandidate {
                id: row.try_get("id")?,
                name: row.try_get("canonical_name")?,
                detail: Some(format!(
                    "{} · {}",
                    date.map(|v| v.to_string())
                        .unwrap_or_else(|| "出生日期未知".to_string()),
                    nationality.unwrap_or_else(|| "国籍未知".to_string())
                )),
            })
        })
        .collect()
}
fn candidates_to_domain(values: Vec<MatchCandidate>) -> Vec<SpreadsheetConflictCandidate> {
    values
        .into_iter()
        .map(|v| SpreadsheetConflictCandidate {
            entity_id: v.id,
            display_name: v.name,
            detail: v.detail,
        })
        .collect()
}

fn count_preview_rows(rows: &[SpreadsheetImportRow]) -> SpreadsheetImportCounts {
    let mut c = SpreadsheetImportCounts {
        total: rows.len() as u64,
        ..Default::default()
    };
    for row in rows {
        match row.status {
            SpreadsheetRowStatus::ReadyAdd => c.ready_add += 1,
            SpreadsheetRowStatus::ReadyUpdate => c.ready_update += 1,
            SpreadsheetRowStatus::ReadyEndPrevious => c.ready_end_previous += 1,
            SpreadsheetRowStatus::Conflict => c.conflict += 1,
            SpreadsheetRowStatus::Error => c.error += 1,
            SpreadsheetRowStatus::Skip => c.skipped += 1,
            SpreadsheetRowStatus::Imported => c.imported += 1,
        }
    }
    c
}
fn import_row_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<SpreadsheetImportRow> {
    let candidates: Value = row.try_get("conflict_candidates")?;
    Ok(SpreadsheetImportRow {
        id: row.try_get("id")?,
        sheet_name: row.try_get("sheet_name")?,
        row_number: row.try_get::<i32, _>("row_number")? as u32,
        entity_type: parse_entity_type(&row.try_get::<String, _>("entity_type")?)?,
        action: parse_action(&row.try_get::<String, _>("requested_action")?)?,
        status: parse_row_status(&row.try_get::<String, _>("status")?)?,
        message: row.try_get("message")?,
        payload: row.try_get("payload")?,
        matched_entity_id: row.try_get("matched_entity_id")?,
        conflict_candidates: serde_json::from_value(candidates)?,
    })
}
fn parse_entity_type(v: &str) -> PersistenceResult<SpreadsheetEntityType> {
    match v {
        "team" => Ok(SpreadsheetEntityType::Team),
        "team_name" => Ok(SpreadsheetEntityType::TeamName),
        "coach" => Ok(SpreadsheetEntityType::Coach),
        "coach_name" => Ok(SpreadsheetEntityType::CoachName),
        "team_coach_period" => Ok(SpreadsheetEntityType::TeamCoachPeriod),
        "formation_usage" => Ok(SpreadsheetEntityType::FormationUsage),
        "team_tactical_observation" => Ok(SpreadsheetEntityType::TeamTacticalObservation),
        "team_ability_observation" => Ok(SpreadsheetEntityType::TeamAbilityObservation),
        "player" => Ok(SpreadsheetEntityType::Player),
        "player_name" => Ok(SpreadsheetEntityType::PlayerName),
        "player_position" => Ok(SpreadsheetEntityType::PlayerPosition),
        "player_team_period" => Ok(SpreadsheetEntityType::PlayerTeamPeriod),
        "player_ability" => Ok(SpreadsheetEntityType::PlayerAbility),
        "player_availability" => Ok(SpreadsheetEntityType::PlayerAvailability),
        "player_dynamic_tag" => Ok(SpreadsheetEntityType::PlayerDynamicTag),
        "external_entity_id" => Ok(SpreadsheetEntityType::ExternalEntityId),
        _ => Err(PersistenceError::InvalidState(format!("未知导入实体：{v}"))),
    }
}
fn parse_action(v: &str) -> PersistenceResult<SpreadsheetAction> {
    match v {
        "add" => Ok(SpreadsheetAction::Add),
        "update" => Ok(SpreadsheetAction::Update),
        "clear" => Ok(SpreadsheetAction::Clear),
        "skip" => Ok(SpreadsheetAction::Skip),
        _ => Err(PersistenceError::InvalidState(format!("未知导入动作：{v}"))),
    }
}
fn parse_row_status(v: &str) -> PersistenceResult<SpreadsheetRowStatus> {
    match v {
        "ready_add" => Ok(SpreadsheetRowStatus::ReadyAdd),
        "ready_update" => Ok(SpreadsheetRowStatus::ReadyUpdate),
        "ready_end_previous" => Ok(SpreadsheetRowStatus::ReadyEndPrevious),
        "conflict" => Ok(SpreadsheetRowStatus::Conflict),
        "error" => Ok(SpreadsheetRowStatus::Error),
        "skip" => Ok(SpreadsheetRowStatus::Skip),
        "imported" => Ok(SpreadsheetRowStatus::Imported),
        _ => Err(PersistenceError::InvalidState(format!("未知导入状态：{v}"))),
    }
}
fn parse_import_mode(v: Option<&str>) -> PersistenceResult<SpreadsheetImportMode> {
    match v.unwrap_or("add_and_update") {
        "add_only" => Ok(SpreadsheetImportMode::AddOnly),
        "add_and_update" => Ok(SpreadsheetImportMode::AddAndUpdate),
        other => Err(PersistenceError::InvalidState(format!(
            "未知导入模式：{other}"
        ))),
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
fn required_text(values: &Map<String, Value>, key: &str) -> PersistenceResult<String> {
    let v = text(values, key);
    if v.is_empty() {
        Err(PersistenceError::InvalidState(format!(
            "缺少必填字段：{key}"
        )))
    } else {
        Ok(v)
    }
}
fn optional_text(values: &Map<String, Value>, key: &str) -> Option<String> {
    let v = text(values, key);
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}
fn default_text(values: &Map<String, Value>, key: &str, default: &str) -> String {
    optional_text(values, key).unwrap_or_else(|| default.to_string())
}
fn optional_uuid(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<Uuid>> {
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        Uuid::parse_str(&v)
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 不是有效 UUID：{e}")))
    }
}
fn payload_optional_uuid(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<Uuid>> {
    match values.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(v)) if v.is_empty() => Ok(None),
        Some(v) => serde_json::from_value(v.clone())
            .map(Some)
            .map_err(PersistenceError::Serialization),
    }
}
fn payload_uuid(values: &Map<String, Value>, key: &str) -> PersistenceResult<Uuid> {
    payload_optional_uuid(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少内部字段：{key}")))
}
fn optional_date(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<NaiveDate>> {
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        NaiveDate::parse_from_str(&v, "%Y-%m-%d")
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 日期格式错误：{e}")))
    }
}
fn required_date(values: &Map<String, Value>, key: &str) -> PersistenceResult<NaiveDate> {
    optional_date(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少日期：{key}")))
}
fn optional_datetime(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<DateTime<Utc>>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        parse_spreadsheet_datetime(&value, key).map(Some)
    }
}
fn required_datetime(values: &Map<String, Value>, key: &str) -> PersistenceResult<DateTime<Utc>> {
    optional_datetime(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少时间：{key}")))
}
fn optional_f64(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<f64>> {
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        v.parse::<f64>()
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 数值格式错误：{e}")))
    }
}
fn required_f64(values: &Map<String, Value>, key: &str) -> PersistenceResult<f64> {
    optional_f64(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少数值：{key}")))
}
fn optional_i16(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<i16>> {
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        v.parse::<i16>()
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 整数格式错误：{e}")))
    }
}
fn optional_i32(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<i32>> {
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        v.parse::<i32>()
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 整数格式错误：{e}")))
    }
}
fn optional_bool(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<bool>> {
    let v = text(values, key).to_lowercase();
    if v.is_empty() {
        Ok(None)
    } else {
        match v.as_str() {
            "true" | "1" | "yes" | "是" => Ok(Some(true)),
            "false" | "0" | "no" | "否" => Ok(Some(false)),
            _ => Err(PersistenceError::InvalidState(format!(
                "{key} 布尔格式错误"
            ))),
        }
    }
}
fn spreadsheet_source_urls(values: &Map<String, Value>) -> Vec<String> {
    optional_text(values, "source_urls")
        .map(|value| {
            value
                .split(['\n', ';'])
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
fn spreadsheet_clear_fields(values: &Map<String, Value>) -> HashSet<String> {
    optional_text(values, "clear_fields")
        .map(|value| {
            value
                .split([',', '，', ';'])
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
fn spreadsheet_row_metadata(values: &Map<String, Value>) -> Value {
    let is_monthly_workbook =
        values.contains_key("source_urls") || values.contains_key("verified_at");
    json!({
        "source": "spreadsheet",
        "monthly_workbook": is_monthly_workbook,
        "source_urls": spreadsheet_source_urls(values),
        "verified_at": optional_text(values, "verified_at"),
        "confidence": optional_text(values, "confidence")
            .and_then(|value| value.parse::<f64>().ok()),
        "notes": optional_text(values, "notes"),
    })
}

fn player_import_type(format_version: &str) -> PersistenceResult<&'static str> {
    match format_version {
        PLAYER_IMPORT_FORMAT => Ok("player_catalog_xlsx"),
        PLAYER_MONTHLY_FORMAT => Ok("player_monthly_xlsx"),
        other => Err(PersistenceError::InvalidState(format!(
            "不支持的球员工作簿版本：{other}"
        ))),
    }
}

fn normalize_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spreadsheet_datetime_accepts_date_only_and_excel_serial() {
        let date_only =
            parse_spreadsheet_datetime("2026-07-18", "observed_at").expect("date-only timestamp");
        assert_eq!(
            date_only.to_rfc3339_opts(SecondsFormat::Secs, true),
            "2026-07-18T00:00:00Z"
        );
        let serial =
            parse_spreadsheet_datetime("46221", "observed_at").expect("Excel serial timestamp");
        assert_eq!(serial.date_naive().to_string(), "2026-07-18");
    }

    #[test]
    fn player_team_period_derives_start_date_from_verification_time() {
        let mut payload = json!({
            "valid_from": "",
            "verified_at": "2026-07-18T15:30:00Z"
        })
        .as_object()
        .expect("object")
        .clone();
        normalize_spreadsheet_payload(SpreadsheetEntityType::PlayerTeamPeriod, &mut payload)
            .expect("normalize period");
        assert_eq!(text(&payload, "valid_from"), "2026-07-18");
        assert_eq!(text(&payload, "_derived_valid_from"), "verified_at");
    }

    #[test]
    fn dynamic_tag_source_alias_is_normalized() {
        let mut payload = json!({"source_type": "official_web_plus_role_model"})
            .as_object()
            .expect("object")
            .clone();
        normalize_dynamic_tag_source_type(&mut payload);
        assert_eq!(text(&payload, "source_type"), "calculation");
        assert_eq!(
            text(&payload, "_source_type_original"),
            "official_web_plus_role_model"
        );
    }

    #[test]
    fn public_roster_initialization_source_is_audited_as_calculation() {
        let mut payload = json!({"source_type": "public_roster_initialization"})
            .as_object()
            .expect("object")
            .clone();
        normalize_dynamic_tag_source_type(&mut payload);
        assert_eq!(text(&payload, "source_type"), "calculation");
        assert_eq!(
            text(&payload, "_source_type_original"),
            "public_roster_initialization"
        );
    }

    #[test]
    fn availability_aliases_are_normalized_without_losing_original_value() {
        let mut questionable = json!({"availability_status": "questionable"})
            .as_object()
            .expect("object")
            .clone();
        normalize_availability_status(&mut questionable);
        assert_eq!(text(&questionable, "availability_status"), "doubtful");
        assert_eq!(
            text(&questionable, "_availability_status_original"),
            "questionable"
        );

        let mut unavailable = json!({"availability_status": "unavailable"})
            .as_object()
            .expect("object")
            .clone();
        normalize_availability_status(&mut unavailable);
        assert_eq!(text(&unavailable, "availability_status"), "unavailable");
    }

    #[test]
    fn package_team_name_can_resolve_to_deferred_external_reference() {
        let references = HashMap::from([("ATM".to_string(), "Atlético Mineiro".to_string())]);
        let target = normalize_reference_name("  ATLÉTICO   MINEIRO ");
        let matches = references
            .iter()
            .filter(|(_, name)| normalize_reference_name(name) == target)
            .collect::<Vec<_>>();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].0, "ATM");
    }

    #[test]
    fn dynamic_tag_child_validation_allows_default_ttl() {
        let payload = json!({
            "confidence": 0.8,
            "sample_size": 7,
            "observed_at": "2026-07-18T15:30:00Z",
            "valid_from": "2026-07-18T00:00:00Z"
        })
        .as_object()
        .expect("object")
        .clone();
        validate_child_fields(SpreadsheetEntityType::PlayerDynamicTag, &payload)
            .expect("missing valid_to should defer to tag default TTL");
    }

    #[test]
    fn dynamic_tag_child_validation_rejects_non_increasing_explicit_range() {
        let payload = json!({
            "confidence": 0.8,
            "sample_size": 7,
            "observed_at": "2026-07-18T15:30:00Z",
            "valid_from": "2026-07-18T00:00:00Z",
            "valid_to": "2026-07-18T00:00:00Z"
        })
        .as_object()
        .expect("object")
        .clone();
        let error = validate_child_fields(SpreadsheetEntityType::PlayerDynamicTag, &payload)
            .expect_err("explicit non-increasing range must fail");
        assert!(error
            .to_string()
            .contains("动态标签失效时间必须晚于生效时间"));
    }
}
