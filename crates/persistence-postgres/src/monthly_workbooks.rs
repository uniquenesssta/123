use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, SecondsFormat, Utc};
use football_domain::{
    MonthlyDataGapRow, SpreadsheetAction, SpreadsheetConflictCandidate, SpreadsheetEntityType,
    SpreadsheetImportCommitResult, SpreadsheetImportCounts, SpreadsheetImportMode,
    SpreadsheetImportPreview, SpreadsheetImportResolution, SpreadsheetImportRow,
    SpreadsheetParsedWorkbook, SpreadsheetRowStatus, TeamAbilityObservationRow,
    TeamMonthlyCoachPeriodRow, TeamMonthlyCoachRow, TeamMonthlyFormationUsageRow,
    TeamMonthlyNameRow, TeamMonthlyTeamRow, TeamMonthlyWorkbookData, TeamTacticalObservationRow,
    TEAM_MONTHLY_FORMAT,
};
use serde_json::{json, Map, Value};
use sqlx::{Postgres, Row, Transaction};
use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

const TEAM_IMPORT_TYPE: &str = "team_monthly_xlsx";
const UNKNOWN_FORMATION_ID: Uuid = Uuid::from_u128(0x076720d204f05b3bad4787f0bfe290bd);

#[derive(Debug, Clone, Eq, Ord, PartialEq, PartialOrd)]
struct FormationGroupKey {
    scope_type: String,
    team_reference: String,
    coach_reference: String,
    competition_reference: String,
    window_start: String,
    window_end: String,
    observed_at: String,
}

impl PostgresStore {
    pub async fn team_monthly_workbook_data(&self) -> PersistenceResult<TeamMonthlyWorkbookData> {
        let teams = sqlx::query(
            r#"
            SELECT team.id, team.canonical_name, team.country_code, team.is_active, team.metadata,
                   profile.short_name, COALESCE(profile.team_type, 'club') AS team_type,
                   profile.city, profile.founded_year, profile.stadium,
                   profile.updated_at AS profile_observed_at,
                   COALESCE(profile.data_confidence, 0.5) AS data_confidence,
                   profile.notes
            FROM football.teams team
            LEFT JOIN football.team_profiles profile ON profile.team_id = team.id
            ORDER BY team.canonical_name, team.id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyTeamRow {
                team_id: row.try_get("id")?,
                official_name: row.try_get("canonical_name")?,
                short_name: row.try_get("short_name")?,
                team_type: row.try_get("team_type")?,
                country_code: row.try_get("country_code")?,
                city: row.try_get("city")?,
                founded_year: row.try_get("founded_year")?,
                stadium: row.try_get("stadium")?,
                is_active: row.try_get("is_active")?,
                profile_observed_at: row.try_get("profile_observed_at")?,
                data_confidence: row.try_get("data_confidence")?,
                notes: row.try_get("notes")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let names = sqlx::query(
            r#"
            SELECT name.team_id, team.canonical_name AS official_name, name.name AS name_value,
                   name.language_code, name.valid_from, name.valid_to, name.metadata
            FROM football.team_names name
            JOIN football.teams team ON team.id = name.team_id
            ORDER BY team.canonical_name, name.name, name.id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyNameRow {
                team_id: row.try_get("team_id")?,
                official_name: row.try_get("official_name")?,
                name_value: row.try_get("name_value")?,
                language_code: row.try_get("language_code")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let coaches = sqlx::query(
            r#"
            SELECT id, canonical_name, nationality_code, status, metadata
            FROM football.coaches ORDER BY canonical_name, id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyCoachRow {
                coach_id: row.try_get("id")?,
                official_name: row.try_get("canonical_name")?,
                nationality_code: row.try_get("nationality_code")?,
                status: row.try_get("status")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let coach_periods = sqlx::query(
            r#"
            SELECT period.team_id, team.canonical_name AS team_name,
                   period.coach_id, coach.canonical_name AS coach_name,
                   period.role, period.valid_from, period.valid_to, period.is_interim,
                   period.confidence, period.metadata
            FROM football.team_coach_periods period
            JOIN football.teams team ON team.id = period.team_id
            JOIN football.coaches coach ON coach.id = period.coach_id
            ORDER BY team.canonical_name, period.valid_from DESC, period.id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyCoachPeriodRow {
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                coach_id: row.try_get("coach_id")?,
                coach_name: row.try_get("coach_name")?,
                role: row.try_get("role")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
                is_interim: row.try_get("is_interim")?,
                confidence: row.try_get("confidence")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let formation_usage = sqlx::query(
            r#"
            SELECT observation.scope_type, observation.team_id, team.canonical_name AS team_name,
                   observation.coach_id, coach.canonical_name AS coach_name,
                   observation.competition_id, observation.formation_id, formation.code AS formation_code,
                   observation.window_preset, observation.window_start, observation.window_end,
                   observation.observed_matches, observation.usage_count, observation.raw_probability,
                   observation.smoothed_probability, observation.confidence,
                   observation.smoothing_alpha AS alpha, observation.observed_at, observation.metadata
            FROM feature.formation_usage_observations observation
            JOIN football.formations formation ON formation.id = observation.formation_id
            LEFT JOIN football.teams team ON team.id = observation.team_id
            LEFT JOIN football.coaches coach ON coach.id = observation.coach_id
            ORDER BY observation.observed_at DESC, observation.scope_type, formation.sort_order
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyFormationUsageRow {
                scope_type: row.try_get("scope_type")?,
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                coach_id: row.try_get("coach_id")?,
                coach_name: row.try_get("coach_name")?,
                competition_id: row.try_get("competition_id")?,
                formation_id: row.try_get("formation_id")?,
                formation_code: row.try_get("formation_code")?,
                window_preset: row.try_get("window_preset")?,
                window_start: row.try_get("window_start")?,
                window_end: row.try_get("window_end")?,
                observed_matches: row.try_get("observed_matches")?,
                usage_count: row.try_get("usage_count")?,
                raw_probability: row.try_get("raw_probability")?,
                smoothed_probability: row.try_get("smoothed_probability")?,
                confidence: row.try_get("confidence")?,
                alpha: row.try_get("alpha")?,
                observed_at: row.try_get("observed_at")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let tactical_observations = sqlx::query(
            r#"
            SELECT observation.team_id, team.canonical_name AS team_name,
                   observation.coach_id, coach.canonical_name AS coach_name,
                   observation.window_start, observation.window_end,
                   observation.build_up_style, observation.progression_style,
                   observation.attacking_width, observation.pressing_intensity,
                   observation.defensive_block, observation.transition_speed,
                   observation.set_piece_tendency, observation.tactical_summary,
                   observation.confidence, observation.observed_at,
                   observation.metadata || jsonb_build_object(
                       'source_urls', observation.source_urls,
                       'verified_at', observation.verified_at
                   ) AS metadata
            FROM feature.team_tactical_observations observation
            JOIN football.teams team ON team.id = observation.team_id
            LEFT JOIN football.coaches coach ON coach.id = observation.coach_id
            ORDER BY observation.observed_at DESC, team.canonical_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamTacticalObservationRow {
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                coach_id: row.try_get("coach_id")?,
                coach_name: row.try_get("coach_name")?,
                window_start: row.try_get("window_start")?,
                window_end: row.try_get("window_end")?,
                build_up_style: row.try_get("build_up_style")?,
                progression_style: row.try_get("progression_style")?,
                attacking_width: row.try_get("attacking_width")?,
                pressing_intensity: row.try_get("pressing_intensity")?,
                defensive_block: row.try_get("defensive_block")?,
                transition_speed: row.try_get("transition_speed")?,
                set_piece_tendency: row.try_get("set_piece_tendency")?,
                tactical_summary: row.try_get("tactical_summary")?,
                confidence: row.try_get("confidence")?,
                observed_at: row.try_get("observed_at")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let ability_observations = sqlx::query(
            r#"
            SELECT observation.team_id, team.canonical_name AS team_name,
                   observation.observed_at, observation.window_start, observation.window_end,
                   observation.attack_rating, observation.midfield_rating,
                   observation.defence_rating, observation.goalkeeper_rating,
                   observation.squad_depth_rating, observation.stability_rating,
                   observation.sample_size, observation.methodology, observation.confidence,
                   observation.metadata || jsonb_build_object(
                       'source_urls', observation.source_urls,
                       'verified_at', observation.verified_at
                   ) AS metadata
            FROM feature.team_ability_observations observation
            JOIN football.teams team ON team.id = observation.team_id
            ORDER BY observation.observed_at DESC, team.canonical_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamAbilityObservationRow {
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                observed_at: row.try_get("observed_at")?,
                window_start: row.try_get("window_start")?,
                window_end: row.try_get("window_end")?,
                attack_rating: row.try_get("attack_rating")?,
                midfield_rating: row.try_get("midfield_rating")?,
                defence_rating: row.try_get("defence_rating")?,
                goalkeeper_rating: row.try_get("goalkeeper_rating")?,
                squad_depth_rating: row.try_get("squad_depth_rating")?,
                stability_rating: row.try_get("stability_rating")?,
                sample_size: row.try_get("sample_size")?,
                methodology: row.try_get("methodology")?,
                confidence: row.try_get("confidence")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let data_gaps = self.team_monthly_data_gaps().await?;
        Ok(TeamMonthlyWorkbookData {
            teams,
            names,
            coaches,
            coach_periods,
            formation_usage,
            tactical_observations,
            ability_observations,
            data_gaps,
        })
    }

    pub async fn team_monthly_data_gaps(&self) -> PersistenceResult<Vec<MonthlyDataGapRow>> {
        let rows = sqlx::query(
            r#"
            SELECT 'team'::text AS entity_type, team.id AS entity_id,
                   team.canonical_name AS entity_name,
                   gap.missing_field, profile.updated_at AS last_observed_at,
                   CASE WHEN profile.updated_at IS NULL THEN NULL
                        ELSE GREATEST(0, EXTRACT(day FROM now() - profile.updated_at)::bigint) END AS stale_days,
                   gap.priority, gap.recommended_action
            FROM football.teams team
            LEFT JOIN football.team_profiles profile ON profile.team_id = team.id
            CROSS JOIN LATERAL (
                VALUES
                    ('profile', CASE WHEN profile.team_id IS NULL THEN 'high' ELSE NULL END, '补全球队基础资料'),
                    ('country_code', CASE WHEN team.country_code IS NULL THEN 'high' ELSE NULL END, '填写国家或地区代码'),
                    ('current_coach', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM football.team_coach_periods period
                        WHERE period.team_id=team.id AND period.valid_from<=current_date
                          AND (period.valid_to IS NULL OR period.valid_to>=current_date)
                    ) THEN 'medium' ELSE NULL END, '维护当前教练任期'),
                    ('formation_usage', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM feature.formation_usage_observations usage
                        WHERE usage.team_id=team.id AND usage.observed_at>=now()-interval '90 days'
                    ) THEN 'medium' ELSE NULL END, '更新最近阵型使用观察')
            ) AS gap(missing_field, priority, recommended_action)
            WHERE gap.priority IS NOT NULL
            ORDER BY CASE gap.priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 ELSE 2 END,
                     team.canonical_name, gap.missing_field
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(monthly_gap_from_row).collect()
    }

    pub async fn player_monthly_data_gaps(&self) -> PersistenceResult<Vec<MonthlyDataGapRow>> {
        let rows = sqlx::query(
            r#"
            SELECT 'player'::text AS entity_type, player.id AS entity_id,
                   player.canonical_name AS entity_name, gap.missing_field,
                   player.updated_at AS last_observed_at,
                   GREATEST(0, EXTRACT(day FROM now() - player.updated_at)::bigint) AS stale_days,
                   gap.priority, gap.recommended_action
            FROM football.players player
            CROSS JOIN LATERAL (
                VALUES
                    ('birth_date', CASE WHEN player.date_of_birth IS NULL THEN 'high' ELSE NULL END, '填写出生日期以避免同名误匹配'),
                    ('nationality_code', CASE WHEN player.nationality_code IS NULL THEN 'medium' ELSE NULL END, '填写国籍代码'),
                    ('position', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM football.player_positions position WHERE position.player_id=player.id
                    ) THEN 'high' ELSE NULL END, '维护主要位置与熟练度'),
                    ('team_period', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM football.player_team_periods period
                        WHERE period.player_id=player.id AND period.valid_from<=current_date
                          AND (period.valid_to IS NULL OR period.valid_to>=current_date)
                    ) THEN 'medium' ELSE NULL END, '维护当前球队履历'),
                    ('ability_observation', CASE WHEN NOT EXISTS (
                        SELECT 1 FROM feature.player_ability_observations observation
                        WHERE observation.player_id=player.id AND observation.observed_at>=now()-interval '120 days'
                    ) THEN 'medium' ELSE NULL END, '补充近期能力观察')
            ) AS gap(missing_field, priority, recommended_action)
            WHERE gap.priority IS NOT NULL
            ORDER BY CASE gap.priority WHEN 'high' THEN 0 WHEN 'medium' THEN 1 ELSE 2 END,
                     player.canonical_name, gap.missing_field
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(monthly_gap_from_row).collect()
    }

    pub async fn preview_team_monthly_import(
        &self,
        parsed: &SpreadsheetParsedWorkbook,
        mode: SpreadsheetImportMode,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        if parsed.format_version != TEAM_MONTHLY_FORMAT {
            return Err(PersistenceError::InvalidState(format!(
                "球队月度工作簿版本错误：{}",
                parsed.format_version
            )));
        }
        if let Some(existing_id) = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id FROM catalog.import_batches
            WHERE source_sha256=$1 AND import_type=$2
              AND status IN ('pending','running','succeeded')
            ORDER BY started_at DESC NULLS LAST LIMIT 1
            "#,
        )
        .bind(&parsed.source_sha256)
        .bind(TEAM_IMPORT_TYPE)
        .fetch_optional(&self.pool)
        .await?
        {
            return self.read_team_monthly_import_preview(existing_id).await;
        }

        let add_teams = parsed
            .rows
            .iter()
            .filter(|row| {
                row.entity_type == SpreadsheetEntityType::Team
                    && matches!(row.action, SpreadsheetAction::Add | SpreadsheetAction::Upsert)
            })
            .filter_map(|row| text(object(&row.values).ok()?, "official_name"))
            .map(|value| normalize_name(&value))
            .collect::<HashSet<_>>();
        let add_coaches = parsed
            .rows
            .iter()
            .filter(|row| {
                row.entity_type == SpreadsheetEntityType::Coach
                    && matches!(row.action, SpreadsheetAction::Add | SpreadsheetAction::Upsert)
            })
            .filter_map(|row| text(object(&row.values).ok()?, "official_name"))
            .map(|value| normalize_name(&value))
            .collect::<HashSet<_>>();

        let mut preview_rows = Vec::with_capacity(parsed.rows.len());
        for raw in &parsed.rows {
            let validation = self
                .validate_team_monthly_row(raw, mode, &add_teams, &add_coaches)
                .await;
            let row = match validation {
                Ok(validation) => SpreadsheetImportRow {
                    id: Uuid::new_v4(),
                    sheet_name: raw.sheet_name.clone(),
                    row_number: raw.row_number,
                    entity_type: raw.entity_type,
                    action: canonical_team_import_action(
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
                    action: canonical_team_import_action(
                        raw.action,
                        raw.entity_type,
                        SpreadsheetRowStatus::Error,
                    ),
                    status: SpreadsheetRowStatus::Error,
                    message: Some(error.to_string()),
                    payload: raw.values.clone(),
                    matched_entity_id: None,
                    conflict_candidates: vec![],
                },
            };
            preview_rows.push(row);
        }
        validate_formation_group_rows(&mut preview_rows);
        let counts = count_rows(&preview_rows);
        let batch_id = Uuid::new_v4();
        let mode_text = import_mode_text(mode);
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO catalog.import_batches (
                id, import_type, workbook_kind, format_version, status,
                source_file_name, source_sha256, import_mode, started_at,
                skipped_count, error_count, metadata
            ) VALUES ($1,$2,'team_monthly',$3,'pending',$4,$5,$6,now(),$7,$8,$9)
            "#,
        )
        .bind(batch_id)
        .bind(TEAM_IMPORT_TYPE)
        .bind(&parsed.format_version)
        .bind(&parsed.source_file_name)
        .bind(&parsed.source_sha256)
        .bind(mode_text)
        .bind(counts.skipped as i64)
        .bind((counts.error + counts.conflict) as i64)
        .bind(json!({"preview_counts": counts}))
        .execute(&mut *tx)
        .await?;
        for row in &preview_rows {
            insert_team_import_row(&mut tx, batch_id, row).await?;
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

    pub async fn read_team_monthly_import_preview(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let batch = sqlx::query(
            "SELECT source_file_name,source_sha256,import_mode,started_at FROM catalog.import_batches WHERE id=$1 AND import_type=$2",
        ).bind(batch_id).bind(TEAM_IMPORT_TYPE).fetch_one(&self.pool).await?;
        let rows = sqlx::query(
            r#"SELECT id,sheet_name,row_number,entity_type,requested_action,status,message,payload,matched_entity_id,conflict_candidates
               FROM catalog.import_rows WHERE batch_id=$1 ORDER BY
               CASE entity_type WHEN 'team' THEN 0 WHEN 'coach' THEN 1 WHEN 'team_name' THEN 2
                    WHEN 'team_coach_period' THEN 3 WHEN 'formation_usage' THEN 4 ELSE 5 END,
               row_number, id"#,
        ).bind(batch_id).fetch_all(&self.pool).await?
        .iter().map(team_import_row_from_db).collect::<PersistenceResult<Vec<_>>>()?;
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
            counts: count_rows(&rows),
            rows,
            created_at: batch
                .try_get::<Option<DateTime<Utc>>, _>("started_at")?
                .unwrap_or_else(Utc::now),
        })
    }

    pub async fn resolve_team_monthly_import_conflict(
        &self,
        batch_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> PersistenceResult<SpreadsheetImportPreview> {
        let mut tx = self.pool.begin().await?;
        let status: String = sqlx::query_scalar(
            "SELECT status FROM catalog.import_batches WHERE id=$1 AND import_type=$2 FOR UPDATE",
        )
        .bind(batch_id)
        .bind(TEAM_IMPORT_TYPE)
        .fetch_one(&mut *tx)
        .await?;
        if status != "pending" {
            return Err(PersistenceError::InvalidState(
                "导入批次已不允许处理冲突".into(),
            ));
        }
        let row = sqlx::query(
            "SELECT status,payload,conflict_candidates FROM catalog.import_rows WHERE id=$1 AND batch_id=$2 FOR UPDATE",
        ).bind(resolution.row_id).bind(batch_id).fetch_one(&mut *tx).await?;
        if row.try_get::<String, _>("status")? != "conflict" {
            return Err(PersistenceError::InvalidState("只有冲突行可处理".into()));
        }
        if resolution.skip {
            sqlx::query("UPDATE catalog.import_rows SET status='skip',message='用户选择跳过',matched_entity_id=NULL,conflict_candidates='[]'::jsonb WHERE id=$1")
                .bind(resolution.row_id).execute(&mut *tx).await?;
        } else {
            let selected = resolution
                .selected_entity_id
                .ok_or_else(|| PersistenceError::InvalidState("请选择候选记录".into()))?;
            let candidates: Vec<SpreadsheetConflictCandidate> =
                serde_json::from_value(row.try_get("conflict_candidates")?)?;
            if !candidates
                .iter()
                .any(|candidate| candidate.entity_id == selected)
            {
                return Err(PersistenceError::InvalidState(
                    "所选记录不在候选范围".into(),
                ));
            }
            let mut payload: Value = row.try_get("payload")?;
            let object = object_mut(&mut payload)?;
            let prefix = object
                .get("_conflict_prefix")
                .and_then(Value::as_str)
                .unwrap_or("entity")
                .to_string();
            object.insert(format!("_resolved_{prefix}_id"), json!(selected));
            object.remove("_conflict_prefix");
            sqlx::query("UPDATE catalog.import_rows SET status='ready_update',message='已人工选择唯一记录',payload=$2,matched_entity_id=$3,conflict_candidates='[]'::jsonb WHERE id=$1")
                .bind(resolution.row_id).bind(payload).bind(selected).execute(&mut *tx).await?;
        }
        tx.commit().await?;
        self.read_team_monthly_import_preview(batch_id).await
    }

    pub async fn commit_team_monthly_import(
        &self,
        batch_id: Uuid,
    ) -> PersistenceResult<SpreadsheetImportCommitResult> {
        let mut tx = self.pool.begin().await?;
        let batch_status: String = sqlx::query_scalar(
            "SELECT status FROM catalog.import_batches WHERE id=$1 AND import_type=$2 FOR UPDATE",
        )
        .bind(batch_id)
        .bind(TEAM_IMPORT_TYPE)
        .fetch_one(&mut *tx)
        .await?;
        if batch_status == "succeeded" {
            let batch = sqlx::query("SELECT inserted_count,updated_count,ended_previous_count,skipped_count,error_count,finished_at FROM catalog.import_batches WHERE id=$1")
                .bind(batch_id).fetch_one(&mut *tx).await?;
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
        if batch_status != "pending" {
            return Err(PersistenceError::InvalidState(
                "导入批次状态不可提交".into(),
            ));
        }
        let blockers: i64 = sqlx::query_scalar("SELECT count(*)::bigint FROM catalog.import_rows WHERE batch_id=$1 AND status IN ('conflict','error')")
            .bind(batch_id).fetch_one(&mut *tx).await?;
        if blockers > 0 {
            return Err(PersistenceError::InvalidState(format!(
                "仍有 {blockers} 条冲突或错误记录"
            )));
        }
        sqlx::query("UPDATE catalog.import_batches SET status='running' WHERE id=$1")
            .bind(batch_id)
            .execute(&mut *tx)
            .await?;

        let mut rows = sqlx::query(
            r#"SELECT id,sheet_name,row_number,entity_type,requested_action,status,message,payload,matched_entity_id,conflict_candidates
               FROM catalog.import_rows WHERE batch_id=$1 ORDER BY
               CASE entity_type WHEN 'team' THEN 0 WHEN 'coach' THEN 1 WHEN 'team_name' THEN 2
                    WHEN 'team_coach_period' THEN 3 WHEN 'team_tactical_observation' THEN 5
                    WHEN 'team_ability_observation' THEN 6 ELSE 7 END,
               row_number,id FOR UPDATE"#,
        ).bind(batch_id).fetch_all(&mut *tx).await?
        .iter().map(team_import_row_from_db).collect::<PersistenceResult<Vec<_>>>()?;

        // Existing pending batches may have been previewed by an older client and can still
        // contain semantic aliases or Excel-rendered date/time strings. Canonicalize the stored
        // payload at the transaction boundary so retrying the same batch is safe after an upgrade.
        for row in rows.iter_mut() {
            let mut payload = row.payload.clone();
            let mut changed = normalize_monthly_datetime_payload(&mut payload)?;
            if matches!(
                row.entity_type,
                SpreadsheetEntityType::TeamTacticalObservation
                    | SpreadsheetEntityType::TeamAbilityObservation
            ) {
                changed |= normalize_point_observation_window_payload(&mut payload)?;
            }
            if row.entity_type == SpreadsheetEntityType::Team {
                changed |= normalize_team_type_payload(&mut payload)?;
            }
            if row.entity_type == SpreadsheetEntityType::FormationUsage {
                changed |= normalize_formation_usage_payload(&mut payload)?;
            }
            if changed {
                sqlx::query("UPDATE catalog.import_rows SET payload=$2 WHERE id=$1")
                    .bind(row.id)
                    .bind(&payload)
                    .execute(&mut *tx)
                    .await?;
                row.payload = payload;
            }
        }

        // Pending previews created by earlier clients may contain both an explicit
        // team overview row and an implicit club row derived from the player sheet.
        // Merge those rows before any insert so dependent rows always resolve one team.
        consolidate_duplicate_ready_add_team_rows(&mut tx, &mut rows).await?;
        consolidate_duplicate_ready_add_team_rows_by_source(&mut tx, &mut rows).await?;
        bind_batch_team_references(&mut tx, &mut rows).await?;

        let mut inserted = 0_u64;
        let mut updated = 0_u64;
        let mut ended_previous = 0_u64;
        let mut skipped = 0_u64;
        for row in rows
            .iter()
            .filter(|row| row.entity_type != SpreadsheetEntityType::FormationUsage)
        {
            if row.status == SpreadsheetRowStatus::Skip
                || row.status == SpreadsheetRowStatus::Imported
            {
                skipped += 1;
                continue;
            }
            let outcome = execute_team_monthly_row(&mut tx, row).await?;
            inserted += outcome.inserted;
            updated += outcome.updated;
            ended_previous += outcome.ended_previous;
            sqlx::query(
                "UPDATE catalog.import_rows SET status='imported',imported_at=now() WHERE id=$1",
            )
            .bind(row.id)
            .execute(&mut *tx)
            .await?;
        }
        let formation_rows = rows
            .iter()
            .filter(|row| {
                row.entity_type == SpreadsheetEntityType::FormationUsage && row.status.is_ready()
            })
            .cloned()
            .collect::<Vec<_>>();
        if !formation_rows.is_empty() {
            let formation_inserted = execute_formation_groups(&mut tx, &formation_rows).await?;
            inserted += formation_inserted;
            for row in formation_rows {
                sqlx::query("UPDATE catalog.import_rows SET status='imported',imported_at=now() WHERE id=$1")
                    .bind(row.id).execute(&mut *tx).await?;
            }
        }
        let finished_at = Utc::now();
        sqlx::query(
            "UPDATE catalog.import_batches SET status='succeeded',inserted_count=$2,updated_count=$3,ended_previous_count=$4,skipped_count=$5,error_count=0,finished_at=$6 WHERE id=$1",
        ).bind(batch_id).bind(inserted as i64).bind(updated as i64).bind(ended_previous as i64).bind(skipped as i64).bind(finished_at)
        .execute(&mut *tx).await?;
        write_audit_event(&mut tx, "team_monthly_workbook_imported", "import_batch", Some(batch_id.to_string()), json!({
            "inserted": inserted, "updated": updated, "ended_previous": ended_previous, "skipped": skipped
        })).await?;
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

    async fn validate_team_monthly_row(
        &self,
        raw: &football_domain::SpreadsheetRawRow,
        mode: SpreadsheetImportMode,
        add_teams: &HashSet<String>,
        add_coaches: &HashSet<String>,
    ) -> PersistenceResult<RowValidation> {
        let mut payload = raw.values.clone();
        normalize_monthly_datetime_payload(&mut payload)?;
        if matches!(
            raw.entity_type,
            SpreadsheetEntityType::TeamTacticalObservation
                | SpreadsheetEntityType::TeamAbilityObservation
        ) {
            normalize_point_observation_window_payload(&mut payload)?;
        }
        let values = object(&payload)?;
        if raw.action == SpreadsheetAction::Skip {
            return Ok(RowValidation::skip(payload));
        }
        if raw.action == SpreadsheetAction::Update && mode == SpreadsheetImportMode::AddOnly {
            return Ok(RowValidation::error(payload, "当前模式不允许 update"));
        }
        if raw.action == SpreadsheetAction::Clear && mode == SpreadsheetImportMode::AddOnly {
            return Ok(RowValidation::error(payload, "当前模式不允许 clear"));
        }
        match raw.entity_type {
            SpreadsheetEntityType::Team => self.validate_team_row(raw.action, mode, payload).await,
            SpreadsheetEntityType::Coach => self.validate_coach_row(raw.action, mode, payload).await,
            SpreadsheetEntityType::TeamName => {
                require_text(values, "name_value")?;
                parse_bool_default(text(values, "is_primary").as_deref(), false)?;
                self.validate_dependent_row(payload, "team", add_teams, None)
                    .await
            }
            SpreadsheetEntityType::TeamCoachPeriod => {
                require_text(values, "role")?;
                parse_date(require_text(values, "valid_from")?, "valid_from")?;
                let first = self
                    .validate_dependent_row(payload.clone(), "team", add_teams, None)
                    .await?;
                if first.status == SpreadsheetRowStatus::Conflict
                    || first.status == SpreadsheetRowStatus::Error
                {
                    return Ok(first);
                }
                let second = self
                    .validate_dependent_row(first.payload, "coach", add_coaches, None)
                    .await?;
                if second.status == SpreadsheetRowStatus::ReadyAdd {
                    Ok(RowValidation {
                        status: SpreadsheetRowStatus::ReadyEndPrevious,
                        message: Some("将新增任期，并在需要时结束上一当前任期".into()),
                        ..second
                    })
                } else {
                    Ok(second)
                }
            }
            SpreadsheetEntityType::FormationUsage => {
                self.validate_formation_usage_row(payload, add_teams, add_coaches)
                    .await
            }
            SpreadsheetEntityType::TeamTacticalObservation
            | SpreadsheetEntityType::TeamAbilityObservation => {
                self.validate_dependent_row(payload, "team", add_teams, None)
                    .await
            }
            _ => Ok(RowValidation::error(payload, "该实体不属于球队月度工作簿")),
        }
    }

    async fn validate_team_row(
        &self,
        action: SpreadsheetAction,
        mode: SpreadsheetImportMode,
        mut payload: Value,
    ) -> PersistenceResult<RowValidation> {
        normalize_team_type_payload(&mut payload)?;
        let values = object(&payload)?;
        let id = optional_uuid(values, "team_id")?;
        let name = text(values, "official_name").unwrap_or_default();
        let country = text(values, "country_code");
        let candidates = find_team_candidates(&self.pool, id, &name, country.as_deref()).await?;
        validate_identity(action, mode, payload, candidates, "team")
    }

    async fn validate_coach_row(
        &self,
        action: SpreadsheetAction,
        mode: SpreadsheetImportMode,
        payload: Value,
    ) -> PersistenceResult<RowValidation> {
        let values = object(&payload)?;
        let id = optional_uuid(values, "coach_id")?;
        let name = text(values, "official_name").unwrap_or_default();
        let nationality = text(values, "nationality_code");
        let candidates =
            find_coach_candidates(&self.pool, id, &name, nationality.as_deref()).await?;
        validate_identity(action, mode, payload, candidates, "coach")
    }

    async fn validate_dependent_row(
        &self,
        mut payload: Value,
        prefix: &str,
        workbook_add_names: &HashSet<String>,
        country_or_nationality: Option<&str>,
    ) -> PersistenceResult<RowValidation> {
        let values = object_mut(&mut payload)?;
        let id_key = format!("{prefix}_id");
        let name_key = format!("{prefix}_name");
        let id = optional_uuid(values, &id_key)?;
        let name = text(values, &name_key)
            .or_else(|| text(values, "official_name"))
            .unwrap_or_default();
        let candidates = if prefix == "team" {
            find_team_candidates(&self.pool, id, &name, country_or_nationality).await?
        } else {
            find_coach_candidates(&self.pool, id, &name, country_or_nationality).await?
        };
        if candidates.len() == 1 {
            let entity_id = candidates[0].entity_id;
            values.insert(format!("_resolved_{prefix}_id"), json!(entity_id));
            return Ok(RowValidation {
                status: SpreadsheetRowStatus::ReadyAdd,
                message: Some(format!("已唯一匹配{prefix}")),
                payload,
                matched_entity_id: Some(entity_id),
                conflict_candidates: vec![],
            });
        }
        if candidates.len() > 1 {
            values.insert("_conflict_prefix".into(), json!(prefix));
            return Ok(RowValidation {
                status: SpreadsheetRowStatus::Conflict,
                message: Some(format!("{prefix}存在多个候选，请人工选择")),
                payload,
                matched_entity_id: None,
                conflict_candidates: candidates,
            });
        }
        if !name.trim().is_empty() && workbook_add_names.contains(&normalize_name(&name)) {
            return Ok(RowValidation::ready_add(
                payload,
                &format!("将在同一批次创建{prefix}后关联"),
            ));
        }
        Ok(RowValidation::error(
            payload,
            &format!("无法唯一匹配{prefix}，请填写 UUID 或先新增实体"),
        ))
    }

    async fn validate_formation_usage_row(
        &self,
        mut payload: Value,
        add_teams: &HashSet<String>,
        add_coaches: &HashSet<String>,
    ) -> PersistenceResult<RowValidation> {
        normalize_formation_usage_payload(&mut payload)?;
        let scope = {
            let values = object(&payload)?;
            let scope = require_text(values, "scope_type")?;
            let window_start = parse_date(
                require_text(values, "window_start")?,
                "window_start",
            )?;
            let window_end =
                parse_date(require_text(values, "window_end")?, "window_end")?;
            if window_end < window_start {
                return Ok(RowValidation::error(
                    payload,
                    "阵型观察 window_end 不能早于 window_start",
                ));
            }
            let observed = parse_i32(
                require_text(values, "observed_matches")?,
                "observed_matches",
            )?;
            let usage = parse_i32(require_text(values, "usage_count")?, "usage_count")?;
            if observed < 0 || usage < 0 || usage > observed {
                return Ok(RowValidation::error(
                    payload,
                    "阵型使用次数必须位于 0 到观察场数之间",
                ));
            }
            if let Some(window_preset) = text(values, "window_preset") {
                if !matches!(
                    window_preset.as_str(),
                    "last_5"
                        | "last_10"
                        | "last_20"
                        | "current_season"
                        | "current_coach_term"
                        | "custom"
                ) {
                    return Ok(RowValidation::error(
                        payload,
                        "阵型观察窗口预设无效；允许 last_5/last_10/last_20/current_season/current_coach_term/custom",
                    ));
                }
            }
            let confidence =
                parse_f64_default(text(values, "confidence").as_deref(), 0.5, "confidence")?;
            if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
                return Ok(RowValidation::error(
                    payload,
                    "阵型观察 confidence 必须位于 0 到 1",
                ));
            }
            let alpha = parse_f64_default(text(values, "alpha").as_deref(), 3.0, "alpha")?;
            if !alpha.is_finite() || alpha <= 0.0 {
                return Ok(RowValidation::error(
                    payload,
                    "阵型观察 alpha 必须大于 0",
                ));
            }
            optional_datetime(values, "observed_at")?;
            scope
        };

        let formation_note = self
            .validate_import_formation_reference(&mut payload)
            .await?;
        let mut validation = match scope.as_str() {
            "team" => {
                self.validate_dependent_row(payload, "team", add_teams, None)
                    .await?
            }
            "coach" => {
                self.validate_dependent_row(payload, "coach", add_coaches, None)
                    .await?
            }
            "team_coach" => {
                let team = self
                    .validate_dependent_row(payload, "team", add_teams, None)
                    .await?;
                if matches!(
                    team.status,
                    SpreadsheetRowStatus::Conflict | SpreadsheetRowStatus::Error
                ) {
                    return Ok(team);
                }
                self.validate_dependent_row(team.payload, "coach", add_coaches, None)
                    .await?
            }
            "competition_default" => {
                let values = object(&payload)?;
                let competition_id = optional_uuid(values, "competition_id")?.ok_or_else(|| {
                    PersistenceError::InvalidState(
                        "competition_default 阵型分布必须填写 competition_id".into(),
                    )
                })?;
                let exists: bool = sqlx::query_scalar(
                    "SELECT EXISTS(SELECT 1 FROM football.competitions WHERE id=$1)",
                )
                .bind(competition_id)
                .fetch_one(&self.pool)
                .await?;
                if !exists {
                    return Ok(RowValidation::error(
                        payload,
                        "阵型分布引用的 competition_id 不存在",
                    ));
                }
                RowValidation::ready_add(payload, "赛事默认阵型分布已通过预检")
            }
            "system_default" => {
                RowValidation::ready_add(payload, "系统默认阵型分布已通过预检")
            }
            _ => {
                return Ok(RowValidation::error(
                    payload,
                    "阵型观察范围无效；允许 team/coach/team_coach/competition_default/system_default",
                ));
            }
        };
        if let Some(note) = formation_note {
            validation.message = Some(match validation.message.take() {
                Some(message) => format!("{message}；{note}"),
                None => note,
            });
        } else if validation.message.is_none() {
            validation.message = Some("阵型概率将在提交时按观察窗口自动归一化".into());
        }
        Ok(validation)
    }

    async fn validate_import_formation_reference(
        &self,
        payload: &mut Value,
    ) -> PersistenceResult<Option<String>> {
        let (source_formation_id, raw_code) = {
            let values = object(payload)?;
            (
                optional_uuid(values, "formation_id")?,
                text(values, "formation_code"),
            )
        };
        let mut stale_id_note = None;
        if let Some(formation_id) = source_formation_id {
            let row = sqlx::query(
                "SELECT code,is_active FROM football.formations WHERE id=$1",
            )
            .bind(formation_id)
            .fetch_optional(&self.pool)
            .await?;
            match row {
                Some(row) if row.try_get::<bool, _>("is_active")? => {
                    let code: String = row.try_get("code")?;
                    let values = object_mut(payload)?;
                    values.insert("formation_code".into(), Value::String(code));
                    values.insert(
                        "_resolved_formation_id".into(),
                        Value::String(formation_id.to_string()),
                    );
                    return Ok(None);
                }
                Some(_) => {
                    stale_id_note = Some(format!("阵型ID {formation_id} 已停用"));
                }
                None => {
                    stale_id_note = Some(format!(
                        "阵型ID {formation_id} 在当前数据库不存在"
                    ));
                }
            }
        }

        let raw_code = raw_code.ok_or_else(|| {
            PersistenceError::InvalidState(match stale_id_note.as_deref() {
                Some(note) => format!("{note}，且缺少可用于重新绑定的 formation_code"),
                None => "缺少必填字段 formation_code".into(),
            })
        })?;
        let code = canonical_formation_code(&raw_code);
        object_mut(payload)?
            .insert("formation_code".into(), Value::String(code.clone()));
        let rows = sqlx::query(
            "SELECT id,is_active FROM football.formations WHERE lower(code)=lower($1) ORDER BY is_active DESC,id LIMIT 2",
        )
        .bind(&code)
        .fetch_all(&self.pool)
        .await?;
        match rows.as_slice() {
            [row] if row.try_get::<bool, _>("is_active")? => {
                let id: Uuid = row.try_get("id")?;
                object_mut(payload)?.insert(
                    "_resolved_formation_id".into(),
                    Value::String(id.to_string()),
                );
                Ok(stale_id_note.map(|note| {
                    format!("{note}，已按阵型代码 {code} 重新绑定到当前目录ID {id}")
                }))
            }
            [row] => {
                let id: Uuid = row.try_get("id")?;
                Err(PersistenceError::InvalidState(format!(
                    "阵型 {code}（{id}）已停用"
                )))
            }
            [] if is_valid_custom_formation_code(&code) => {
                let registration_note = format!(
                    "阵型 {code} 不在内置目录中，提交时将保留原代码并登记为自定义阵型"
                );
                Ok(Some(match stale_id_note {
                    Some(note) => format!("{note}；{registration_note}"),
                    None => registration_note,
                }))
            }
            [] => Err(PersistenceError::InvalidState(format!(
                "阵型代码 {raw_code} 无法识别；请使用目录中的阵型，或填写各线人数合计为 10 的代码（例如 3-4-1-2）"
            ))),
            _ => Err(PersistenceError::InvalidState(format!(
                "阵型代码 {code} 在目录中存在多个大小写重复项"
            ))),
        }
    }
}

#[derive(Default)]
struct CommitOutcome {
    inserted: u64,
    updated: u64,
    ended_previous: u64,
}
struct RowValidation {
    status: SpreadsheetRowStatus,
    message: Option<String>,
    payload: Value,
    matched_entity_id: Option<Uuid>,
    conflict_candidates: Vec<SpreadsheetConflictCandidate>,
}
impl RowValidation {
    fn skip(payload: Value) -> Self {
        Self {
            status: SpreadsheetRowStatus::Skip,
            message: Some("action=skip".into()),
            payload,
            matched_entity_id: None,
            conflict_candidates: vec![],
        }
    }
    fn error(payload: Value, message: &str) -> Self {
        Self {
            status: SpreadsheetRowStatus::Error,
            message: Some(message.into()),
            payload,
            matched_entity_id: None,
            conflict_candidates: vec![],
        }
    }
    fn ready_add(payload: Value, message: &str) -> Self {
        Self {
            status: SpreadsheetRowStatus::ReadyAdd,
            message: Some(message.into()),
            payload,
            matched_entity_id: None,
            conflict_candidates: vec![],
        }
    }
}

fn validate_identity(
    action: SpreadsheetAction,
    mode: SpreadsheetImportMode,
    mut payload: Value,
    candidates: Vec<SpreadsheetConflictCandidate>,
    prefix: &str,
) -> PersistenceResult<RowValidation> {
    let values = object_mut(&mut payload)?;
    match candidates.len() {
        0 if matches!(action, SpreadsheetAction::Add | SpreadsheetAction::Upsert) => {
            Ok(RowValidation::ready_add(payload, "将新增实体"))
        }
        0 => Ok(RowValidation::error(
            payload,
            "标记为更新或清空，但数据库不存在匹配实体",
        )),
        1 if action == SpreadsheetAction::Add || mode == SpreadsheetImportMode::AddOnly => {
            Ok(RowValidation {
                status: SpreadsheetRowStatus::Skip,
                message: Some("相同实体已存在；当前动作或导入模式不更新".into()),
                payload,
                matched_entity_id: Some(candidates[0].entity_id),
                conflict_candidates: vec![],
            })
        }
        1 => {
            let id = candidates[0].entity_id;
            values.insert(format!("_resolved_{prefix}_id"), json!(id));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::ReadyUpdate,
                message: Some(if action == SpreadsheetAction::Upsert {
                    "upsert 已自动转换为 update，将按非空字段更新".into()
                } else {
                    "已唯一匹配，将按非空字段更新".into()
                }),
                payload,
                matched_entity_id: Some(id),
                conflict_candidates: vec![],
            })
        }
        _ => {
            values.insert("_conflict_prefix".into(), json!(prefix));
            Ok(RowValidation {
                status: SpreadsheetRowStatus::Conflict,
                message: Some("存在多个候选，请人工选择".into()),
                payload,
                matched_entity_id: None,
                conflict_candidates: candidates,
            })
        }
    }
}

fn team_ready_add_identity(row: &SpreadsheetImportRow) -> PersistenceResult<Option<String>> {
    if row.entity_type != SpreadsheetEntityType::Team
        || row.status != SpreadsheetRowStatus::ReadyAdd
    {
        return Ok(None);
    }
    let values = object(&row.payload)?;
    if let Some(team_id) = optional_uuid(values, "team_id")? {
        return Ok(Some(format!("id:{team_id}")));
    }
    let Some(name) = text(values, "official_name") else {
        return Ok(None);
    };
    let normalized_name = normalize_name(&name);
    if normalized_name.is_empty() {
        return Ok(None);
    }
    let country = text(values, "country_code")
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    let team_type = normalize_team_type(text(values, "team_type"))?
        .unwrap_or_else(|| "club".to_string());
    Ok(Some(format!(
        "name:{normalized_name}|country:{country}|type:{team_type}"
    )))
}

fn payload_value_is_present(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::String(value) => !value.trim().is_empty(),
        _ => true,
    }
}

fn team_row_preference(row: &SpreadsheetImportRow) -> usize {
    let explicit_sheet_bonus = if row.sheet_name == "球队总览" {
        10_000
    } else {
        0
    };
    let populated_fields = row
        .payload
        .as_object()
        .map(|values| {
            values
                .iter()
                .filter(|(key, value)| {
                    !matches!(key.as_str(), "action" | "clear_fields")
                        && payload_value_is_present(value)
                })
                .count()
        })
        .unwrap_or_default();
    explicit_sheet_bonus + populated_fields
}

fn merge_missing_team_payload_fields(
    target: &mut Map<String, Value>,
    source: &Map<String, Value>,
) {
    for (key, value) in source {
        if !payload_value_is_present(value) {
            continue;
        }
        let should_fill = match target.get(key) {
            Some(current) => !payload_value_is_present(current),
            None => true,
        };
        if should_fill {
            target.insert(key.clone(), value.clone());
        }
    }
}

async fn consolidate_duplicate_ready_add_team_rows(
    tx: &mut Transaction<'_, Postgres>,
    rows: &mut [SpreadsheetImportRow],
) -> PersistenceResult<()> {
    let mut groups = HashMap::<String, Vec<usize>>::new();
    for (index, row) in rows.iter().enumerate() {
        if let Some(identity) = team_ready_add_identity(row)? {
            groups.entry(identity).or_default().push(index);
        }
    }

    for indices in groups.values().filter(|indices| indices.len() > 1) {
        let canonical_index = *indices
            .iter()
            .max_by_key(|index| team_row_preference(&rows[**index]))
            .expect("duplicate team group is non-empty");
        let mut canonical_payload = rows[canonical_index]
            .payload
            .as_object()
            .cloned()
            .ok_or_else(|| PersistenceError::InvalidState("球队导入载荷不是对象".into()))?;

        for index in indices.iter().copied().filter(|index| *index != canonical_index) {
            let source = rows[index]
                .payload
                .as_object()
                .ok_or_else(|| PersistenceError::InvalidState("球队导入载荷不是对象".into()))?;
            merge_missing_team_payload_fields(&mut canonical_payload, source);
        }

        let canonical_payload = Value::Object(canonical_payload);
        rows[canonical_index].payload = canonical_payload.clone();
        rows[canonical_index].message = Some("同一资料包重复球队行已合并".into());
        sqlx::query(
            "UPDATE catalog.import_rows SET payload=$2,message='同一资料包重复球队行已合并' WHERE id=$1",
        )
        .bind(rows[canonical_index].id)
        .bind(&canonical_payload)
        .execute(&mut **tx)
        .await?;

        for index in indices.iter().copied().filter(|index| *index != canonical_index) {
            rows[index].status = SpreadsheetRowStatus::Skip;
            rows[index].message = Some("同一资料包重复球队行已合并并跳过".into());
            sqlx::query(
                "UPDATE catalog.import_rows SET status='skip',message='同一资料包重复球队行已合并并跳过',matched_entity_id=NULL,conflict_candidates='[]'::jsonb WHERE id=$1",
            )
            .bind(rows[index].id)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

fn canonical_team_import_action(
    action: SpreadsheetAction,
    entity_type: SpreadsheetEntityType,
    status: SpreadsheetRowStatus,
) -> SpreadsheetAction {
    if action != SpreadsheetAction::Upsert {
        return action;
    }
    if matches!(entity_type, SpreadsheetEntityType::Team | SpreadsheetEntityType::Coach)
        && matches!(status, SpreadsheetRowStatus::ReadyUpdate | SpreadsheetRowStatus::Conflict)
    {
        SpreadsheetAction::Update
    } else {
        SpreadsheetAction::Add
    }
}

fn normalized_source_urls(values: &Map<String, Value>) -> Vec<String> {
    let Some(raw) = text(values, "source_urls") else {
        return Vec::new();
    };
    let mut urls = raw
        .split(['\n', '\r', ',', ';', '；'])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.trim_end_matches('/').to_ascii_lowercase())
        .collect::<Vec<_>>();
    urls.sort();
    urls.dedup();
    urls
}

fn team_ready_add_source_identity(
    row: &SpreadsheetImportRow,
) -> PersistenceResult<Option<String>> {
    if row.entity_type != SpreadsheetEntityType::Team
        || row.status != SpreadsheetRowStatus::ReadyAdd
    {
        return Ok(None);
    }
    let values = object(&row.payload)?;
    let urls = normalized_source_urls(values);
    if urls.is_empty() {
        return Ok(None);
    }
    let country = text(values, "country_code")
        .unwrap_or_default()
        .trim()
        .to_ascii_uppercase();
    let team_type = normalize_team_type(text(values, "team_type"))?
        .unwrap_or_else(|| "club".to_string());
    Ok(Some(format!(
        "source:{}|country:{country}|type:{team_type}",
        urls.join("|")
    )))
}

async fn consolidate_duplicate_ready_add_team_rows_by_source(
    tx: &mut Transaction<'_, Postgres>,
    rows: &mut [SpreadsheetImportRow],
) -> PersistenceResult<()> {
    let mut groups = HashMap::<String, Vec<usize>>::new();
    for (index, row) in rows.iter().enumerate() {
        if let Some(identity) = team_ready_add_source_identity(row)? {
            groups.entry(identity).or_default().push(index);
        }
    }

    let duplicate_source_groups = groups
        .into_values()
        .filter(|indices| {
            indices.len() > 1
                && indices
                    .iter()
                    .any(|index| rows[*index].sheet_name == "球队总览")
                && indices
                    .iter()
                    .any(|index| rows[*index].sheet_name == "球员与评分")
        })
        .collect::<Vec<_>>();

    for indices in duplicate_source_groups {
        let canonical_index = *indices
            .iter()
            .max_by_key(|index| team_row_preference(&rows[**index]))
            .expect("duplicate source group is non-empty");
        let mut canonical_payload = rows[canonical_index]
            .payload
            .as_object()
            .cloned()
            .ok_or_else(|| PersistenceError::InvalidState("球队导入载荷不是对象".into()))?;

        for index in indices.iter().copied().filter(|index| *index != canonical_index) {
            let source = rows[index]
                .payload
                .as_object()
                .ok_or_else(|| PersistenceError::InvalidState("球队导入载荷不是对象".into()))?;
            merge_missing_team_payload_fields(&mut canonical_payload, source);
        }

        let canonical_payload = Value::Object(canonical_payload);
        rows[canonical_index].payload = canonical_payload.clone();
        rows[canonical_index].message = Some("同一资料包同来源球队行已合并".into());
        sqlx::query(
            "UPDATE catalog.import_rows SET payload=$2,message='同一资料包同来源球队行已合并' WHERE id=$1",
        )
        .bind(rows[canonical_index].id)
        .bind(&canonical_payload)
        .execute(&mut **tx)
        .await?;

        for index in indices.iter().copied().filter(|index| *index != canonical_index) {
            rows[index].status = SpreadsheetRowStatus::Skip;
            rows[index].message = Some("同一资料包同来源球队行已合并并跳过".into());
            sqlx::query(
                "UPDATE catalog.import_rows SET status='skip',message='同一资料包同来源球队行已合并并跳过',matched_entity_id=NULL,conflict_candidates='[]'::jsonb WHERE id=$1",
            )
            .bind(rows[index].id)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct BatchTeamReference {
    id: Uuid,
    aliases: HashSet<String>,
    source_urls: HashSet<String>,
}

fn insert_unique_reference(
    references: &mut HashMap<String, Option<Uuid>>,
    key: String,
    id: Uuid,
) {
    if key.is_empty() {
        return;
    }
    references
        .entry(key)
        .and_modify(|current| {
            if current.is_some_and(|current_id| current_id != id) {
                *current = None;
            }
        })
        .or_insert(Some(id));
}

fn build_batch_team_reference_indexes(
    teams: &[BatchTeamReference],
) -> (HashMap<String, Option<Uuid>>, HashMap<String, Option<Uuid>>) {
    let mut aliases = HashMap::new();
    let mut sources = HashMap::new();
    for team in teams {
        for alias in &team.aliases {
            insert_unique_reference(&mut aliases, alias.clone(), team.id);
        }
        for source in &team.source_urls {
            insert_unique_reference(&mut sources, source.clone(), team.id);
        }
    }
    (aliases, sources)
}

fn resolve_batch_team_reference(
    values: &Map<String, Value>,
    aliases: &HashMap<String, Option<Uuid>>,
    sources: &HashMap<String, Option<Uuid>>,
) -> PersistenceResult<Option<Uuid>> {
    if let Some(id) = optional_uuid(values, "_resolved_team_id")?
        .or(optional_uuid(values, "team_id")?)
    {
        return Ok(Some(id));
    }
    for key in ["team_name", "official_name", "short_name", "name_value"] {
        let Some(name) = text(values, key) else {
            continue;
        };
        if let Some(Some(id)) = aliases.get(&normalize_name(&name)) {
            return Ok(Some(*id));
        }
    }
    let matched_sources = normalized_source_urls(values)
        .into_iter()
        .filter_map(|source| sources.get(&source).copied().flatten())
        .collect::<HashSet<_>>();
    if matched_sources.len() == 1 {
        return Ok(matched_sources.into_iter().next());
    }
    Ok(None)
}

async fn bind_batch_team_references(
    tx: &mut Transaction<'_, Postgres>,
    rows: &mut [SpreadsheetImportRow],
) -> PersistenceResult<()> {
    let mut teams = Vec::<BatchTeamReference>::new();

    for row in rows.iter_mut().filter(|row| {
        row.entity_type == SpreadsheetEntityType::Team
            && row.status != SpreadsheetRowStatus::Skip
            && row.status != SpreadsheetRowStatus::Imported
    }) {
        let values = object_mut(&mut row.payload)?;
        let id = optional_uuid(values, "_resolved_team_id")?
            .or(optional_uuid(values, "team_id")?)
            .or(row.matched_entity_id)
            .unwrap_or_else(Uuid::new_v4);
        values.insert("team_id".into(), json!(id));
        values.insert("_resolved_team_id".into(), json!(id));
        let aliases = ["official_name", "short_name", "team_name"]
            .into_iter()
            .filter_map(|key| text(values, key))
            .map(|value| normalize_name(&value))
            .filter(|value| !value.is_empty())
            .collect::<HashSet<_>>();
        let source_urls = normalized_source_urls(values).into_iter().collect();
        teams.push(BatchTeamReference {
            id,
            aliases,
            source_urls,
        });
        sqlx::query("UPDATE catalog.import_rows SET payload=$2 WHERE id=$1")
            .bind(row.id)
            .bind(&row.payload)
            .execute(&mut **tx)
            .await?;
    }

    if teams.is_empty() {
        return Ok(());
    }

    let (mut alias_index, mut source_index) = build_batch_team_reference_indexes(&teams);

    for row in rows.iter_mut().filter(|row| {
        row.entity_type == SpreadsheetEntityType::TeamName
            && row.status != SpreadsheetRowStatus::Skip
            && row.status != SpreadsheetRowStatus::Imported
    }) {
        let values = object_mut(&mut row.payload)?;
        let resolved = resolve_batch_team_reference(values, &alias_index, &source_index)?;
        let Some(team_id) = resolved else {
            continue;
        };
        values.insert("_resolved_team_id".into(), json!(team_id));
        if let Some(team) = teams.iter_mut().find(|team| team.id == team_id) {
            for key in ["team_name", "name_value"] {
                if let Some(name) = text(values, key) {
                    let normalized = normalize_name(&name);
                    if !normalized.is_empty() {
                        team.aliases.insert(normalized);
                    }
                }
            }
        }
        sqlx::query("UPDATE catalog.import_rows SET payload=$2 WHERE id=$1")
            .bind(row.id)
            .bind(&row.payload)
            .execute(&mut **tx)
            .await?;
    }

    (alias_index, source_index) = build_batch_team_reference_indexes(&teams);

    for row in rows.iter_mut().filter(|row| {
        row.entity_type != SpreadsheetEntityType::Team
            && row.status != SpreadsheetRowStatus::Skip
            && row.status != SpreadsheetRowStatus::Imported
    }) {
        let values = object_mut(&mut row.payload)?;
        if optional_uuid(values, "_resolved_team_id")?.is_some() {
            continue;
        }
        let Some(team_id) =
            resolve_batch_team_reference(values, &alias_index, &source_index)?
        else {
            continue;
        };
        values.insert("_resolved_team_id".into(), json!(team_id));
        sqlx::query("UPDATE catalog.import_rows SET payload=$2 WHERE id=$1")
            .bind(row.id)
            .bind(&row.payload)
            .execute(&mut **tx)
            .await?;
    }
    Ok(())
}

async fn ensure_team_name_alias(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    name: &str,
    language_code: Option<&str>,
    valid_from: Option<NaiveDate>,
    valid_to: Option<NaiveDate>,
    metadata: &Value,
) -> PersistenceResult<()> {
    let name = name.trim();
    if name.is_empty() {
        return Ok(());
    }
    let normalized_name = normalize_name(name);
    sqlx::query(
        r#"WITH updated AS (
            UPDATE football.team_names
            SET name = $3,
                language_code = COALESCE($5, language_code),
                valid_from = COALESCE($6, valid_from),
                valid_to = COALESCE($7, valid_to),
                metadata = metadata || $8
            WHERE team_id = $2 AND normalized_name = $4
            RETURNING id
        )
        INSERT INTO football.team_names (
            id, team_id, name, normalized_name, language_code, valid_from, valid_to, metadata
        )
        SELECT $1, $2, $3, $4, $5, $6, $7, $8
        WHERE NOT EXISTS (SELECT 1 FROM updated)"#,
    )
    .bind(Uuid::new_v4())
    .bind(team_id)
    .bind(name)
    .bind(normalized_name)
    .bind(language_code)
    .bind(valid_from)
    .bind(valid_to)
    .bind(metadata)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn preserve_current_team_canonical_alias(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    metadata: &Value,
) -> PersistenceResult<()> {
    let current_name = sqlx::query_scalar::<_, String>(
        "SELECT canonical_name FROM football.teams WHERE id=$1",
    )
    .bind(team_id)
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(current_name) = current_name {
        ensure_team_name_alias(tx, team_id, &current_name, None, None, None, metadata).await?;
    }
    Ok(())
}

async fn execute_team_monthly_row(
    tx: &mut Transaction<'_, Postgres>,
    row: &SpreadsheetImportRow,
) -> PersistenceResult<CommitOutcome> {
    let values = object(&row.payload)?;
    let metadata = common_metadata(values);
    match row.entity_type {
        SpreadsheetEntityType::Team => {
            if row.status == SpreadsheetRowStatus::ReadyAdd {
                let id = optional_uuid(values, "team_id")?.unwrap_or_else(Uuid::new_v4);
                let name = require_text(values, "official_name")?;
                sqlx::query("INSERT INTO football.teams (id,canonical_name,normalized_name,country_code,is_active,metadata) VALUES ($1,$2,$3,$4,$5,$6)")
                    .bind(id).bind(&name).bind(normalize_name(&name)).bind(text(values,"country_code"))
                    .bind(parse_bool_default(text(values,"is_active").as_deref(), true)?).bind(&metadata)
                    .execute(&mut **tx).await?;
                upsert_team_profile_from_values(tx, id, values, &metadata).await?;
                ensure_team_name_alias(tx, id, &name, None, None, None, &metadata).await?;
                if let Some(short_name) = text(values, "short_name") {
                    ensure_team_name_alias(tx, id, &short_name, None, None, None, &metadata).await?;
                }
                Ok(CommitOutcome {
                    inserted: 1,
                    ..Default::default()
                })
            } else {
                let id = resolved_uuid(values, "team")?;
                apply_team_update(tx, id, row.action, values, &metadata).await?;
                Ok(CommitOutcome {
                    updated: 1,
                    ..Default::default()
                })
            }
        }
        SpreadsheetEntityType::TeamName => {
            let team_id = resolve_entity_id_tx(tx, values, "team").await?;
            let name = require_text(values, "name_value")?;
            let normalized_name = normalize_name(&name);
            let is_primary = parse_bool_default(text(values, "is_primary").as_deref(), false)?;
            if is_primary {
                preserve_current_team_canonical_alias(tx, team_id, &metadata).await?;
                sqlx::query(
                    "UPDATE football.teams SET canonical_name=$2, normalized_name=$3, updated_at=now() WHERE id=$1",
                )
                .bind(team_id)
                .bind(&name)
                .bind(&normalized_name)
                .execute(&mut **tx)
                .await?;
            }
            let language_code = text(values, "language_code");
            let valid_from = optional_date(values, "valid_from")?;
            let valid_to = optional_date(values, "valid_to")?;
            ensure_team_name_alias(
                tx,
                team_id,
                &name,
                language_code.as_deref(),
                valid_from,
                valid_to,
                &metadata,
            )
            .await?;
            Ok(CommitOutcome {
                inserted: 1,
                ..Default::default()
            })
        }
        SpreadsheetEntityType::Coach => {
            if row.status == SpreadsheetRowStatus::ReadyAdd {
                let id = optional_uuid(values, "coach_id")?.unwrap_or_else(Uuid::new_v4);
                let name = require_text(values, "official_name")?;
                sqlx::query("INSERT INTO football.coaches (id,canonical_name,normalized_name,nationality_code,status,metadata) VALUES ($1,$2,$3,$4,$5,$6)")
                    .bind(id).bind(&name).bind(normalize_name(&name)).bind(text(values,"nationality_code"))
                    .bind(text(values,"coach_status").unwrap_or_else(|| "active".into())).bind(metadata)
                    .execute(&mut **tx).await?;
                Ok(CommitOutcome {
                    inserted: 1,
                    ..Default::default()
                })
            } else {
                let id = resolved_uuid(values, "coach")?;
                let clear = clear_fields(values);
                sqlx::query(r#"UPDATE football.coaches SET
                    canonical_name=COALESCE(NULLIF($2,''),canonical_name),
                    normalized_name=CASE WHEN NULLIF($2,'') IS NULL THEN normalized_name ELSE $3 END,
                    nationality_code=CASE WHEN $4 THEN NULL ELSE COALESCE(NULLIF($5,''),nationality_code) END,
                    status=COALESCE(NULLIF($6,''),status), metadata=metadata || $7, updated_at=now() WHERE id=$1"#)
                    .bind(id).bind(text(values,"official_name").unwrap_or_default()).bind(normalize_name(&text(values,"official_name").unwrap_or_default()))
                    .bind(clear.contains("nationality_code")).bind(text(values,"nationality_code"))
                    .bind(text(values,"coach_status").unwrap_or_default()).bind(metadata)
                    .execute(&mut **tx).await?;
                Ok(CommitOutcome {
                    updated: 1,
                    ..Default::default()
                })
            }
        }
        SpreadsheetEntityType::TeamCoachPeriod => {
            let team_id = resolve_entity_id_tx(tx, values, "team").await?;
            let coach_id = resolve_entity_id_tx(tx, values, "coach").await?;
            let role = require_text(values, "role")?;
            let valid_from = parse_date(require_text(values, "valid_from")?, "valid_from")?;
            let valid_to = optional_date(values, "valid_to")?;
            let mut ended = 0;
            if valid_to.is_none()
                && matches!(
                    role.as_str(),
                    "head_coach" | "interim_head_coach" | "caretaker"
                )
            {
                ended = sqlx::query(r#"UPDATE football.team_coach_periods SET valid_to=$2 - 1
                    WHERE team_id=$1 AND valid_to IS NULL AND valid_from < $2
                      AND role IN ('head_coach','interim_head_coach','caretaker') AND coach_id<>$3"#)
                    .bind(team_id).bind(valid_from).bind(coach_id).execute(&mut **tx).await?.rows_affected();
            }
            sqlx::query(r#"INSERT INTO football.team_coach_periods
                (id,team_id,coach_id,role,valid_from,valid_to,is_interim,confidence,metadata)
                VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
                ON CONFLICT (team_id,coach_id,role,valid_from) DO UPDATE SET
                    valid_to=EXCLUDED.valid_to,is_interim=EXCLUDED.is_interim,
                    confidence=EXCLUDED.confidence,metadata=football.team_coach_periods.metadata || EXCLUDED.metadata"#)
                .bind(Uuid::new_v4()).bind(team_id).bind(coach_id).bind(role).bind(valid_from).bind(valid_to)
                .bind(parse_bool_default(text(values,"is_interim").as_deref(), false)?)
                .bind(parse_f64_default(text(values,"confidence").as_deref(),0.5,"confidence")?).bind(metadata)
                .execute(&mut **tx).await?;
            Ok(CommitOutcome {
                inserted: 1,
                ended_previous: ended,
                ..Default::default()
            })
        }
        SpreadsheetEntityType::TeamTacticalObservation => {
            let team_id = resolve_entity_id_tx(tx, values, "team").await?;
            let coach_id = resolve_optional_entity_id_tx(tx, values, "coach").await?;
            sqlx::query(r#"INSERT INTO feature.team_tactical_observations (
                id,team_id,coach_id,window_start,window_end,build_up_style,progression_style,
                attacking_width,pressing_intensity,defensive_block,transition_speed,set_piece_tendency,
                tactical_summary,confidence,source_urls,verified_at,observed_at,metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18)
            ON CONFLICT DO NOTHING"#)
                .bind(Uuid::new_v4()).bind(team_id).bind(coach_id)
                .bind(parse_date(require_text(values,"window_start")?,"window_start")?)
                .bind(parse_date(require_text(values,"window_end")?,"window_end")?)
                .bind(text(values,"build_up_style")).bind(text(values,"progression_style"))
                .bind(text(values,"attacking_width")).bind(text(values,"pressing_intensity"))
                .bind(text(values,"defensive_block")).bind(text(values,"transition_speed"))
                .bind(text(values,"set_piece_tendency")).bind(text(values,"tactical_summary"))
                .bind(parse_f64_default(text(values,"confidence").as_deref(),0.5,"confidence")?)
                .bind(source_urls(values)).bind(optional_datetime(values,"verified_at")?)
                .bind(optional_datetime(values,"observed_at")?.unwrap_or_else(Utc::now)).bind(metadata)
                .execute(&mut **tx).await?;
            Ok(CommitOutcome {
                inserted: 1,
                ..Default::default()
            })
        }
        SpreadsheetEntityType::TeamAbilityObservation => {
            let team_id = resolve_entity_id_tx(tx, values, "team").await?;
            sqlx::query(r#"INSERT INTO feature.team_ability_observations (
                id,team_id,observed_at,window_start,window_end,attack_rating,midfield_rating,
                defence_rating,goalkeeper_rating,squad_depth_rating,stability_rating,sample_size,
                methodology,confidence,source_urls,verified_at,metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            ON CONFLICT (team_id,observed_at,window_start,window_end) DO UPDATE SET
                attack_rating=EXCLUDED.attack_rating,midfield_rating=EXCLUDED.midfield_rating,
                defence_rating=EXCLUDED.defence_rating,goalkeeper_rating=EXCLUDED.goalkeeper_rating,
                squad_depth_rating=EXCLUDED.squad_depth_rating,stability_rating=EXCLUDED.stability_rating,
                sample_size=EXCLUDED.sample_size,methodology=EXCLUDED.methodology,confidence=EXCLUDED.confidence,
                source_urls=EXCLUDED.source_urls,verified_at=EXCLUDED.verified_at,
                metadata=feature.team_ability_observations.metadata || EXCLUDED.metadata"#)
                .bind(Uuid::new_v4()).bind(team_id)
                .bind(optional_datetime(values,"observed_at")?.unwrap_or_else(Utc::now))
                .bind(parse_date(require_text(values,"window_start")?,"window_start")?)
                .bind(parse_date(require_text(values,"window_end")?,"window_end")?)
                .bind(optional_f64(values,"attack_rating")?).bind(optional_f64(values,"midfield_rating")?)
                .bind(optional_f64(values,"defence_rating")?).bind(optional_f64(values,"goalkeeper_rating")?)
                .bind(optional_f64(values,"squad_depth_rating")?).bind(optional_f64(values,"stability_rating")?)
                .bind(parse_i32_default(text(values,"sample_size").as_deref(),0,"sample_size")?)
                .bind(text(values,"methodology"))
                .bind(parse_f64_default(text(values,"confidence").as_deref(),0.5,"confidence")?)
                .bind(source_urls(values)).bind(optional_datetime(values,"verified_at")?).bind(metadata)
                .execute(&mut **tx).await?;
            Ok(CommitOutcome {
                inserted: 1,
                ..Default::default()
            })
        }
        _ => Err(PersistenceError::InvalidState(
            "球队月度工作簿包含不支持的实体".into(),
        )),
    }
}

fn formation_entity_reference(values: &Map<String, Value>, prefix: &str) -> String {
    text(values, &format!("_resolved_{prefix}_id"))
        .or_else(|| text(values, &format!("{prefix}_id")))
        .map(|value| format!("id:{}", value.trim().to_ascii_lowercase()))
        .or_else(|| {
            text(values, &format!("{prefix}_name"))
                .map(|value| format!("name:{}", normalize_name(&value)))
        })
        .unwrap_or_default()
}

fn formation_group_key(values: &Map<String, Value>) -> FormationGroupKey {
    let scope_type = text(values, "scope_type")
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    FormationGroupKey {
        team_reference: if matches!(scope_type.as_str(), "team" | "team_coach") {
            formation_entity_reference(values, "team")
        } else {
            String::new()
        },
        coach_reference: if matches!(scope_type.as_str(), "coach" | "team_coach") {
            formation_entity_reference(values, "coach")
        } else {
            String::new()
        },
        competition_reference: if scope_type == "competition_default" {
            text(values, "competition_id")
                .unwrap_or_default()
                .trim()
                .to_ascii_lowercase()
        } else {
            String::new()
        },
        scope_type,
        window_start: text(values, "window_start").unwrap_or_default(),
        window_end: text(values, "window_end").unwrap_or_default(),
        observed_at: text(values, "observed_at").unwrap_or_default(),
    }
}

fn formation_record_reference(values: &Map<String, Value>) -> String {
    text(values, "_resolved_formation_id")
        .or_else(|| text(values, "formation_id"))
        .map(|value| format!("id:{}", value.trim().to_ascii_lowercase()))
        .or_else(|| {
            text(values, "formation_code")
                .map(|value| format!("code:{}", normalize_name(&value)))
        })
        .unwrap_or_default()
}

fn validate_formation_group_rows(rows: &mut [SpreadsheetImportRow]) {
    let mut groups: BTreeMap<FormationGroupKey, Vec<usize>> = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        if row.entity_type != SpreadsheetEntityType::FormationUsage || !row.status.is_ready() {
            continue;
        }
        let Ok(values) = object(&row.payload) else {
            continue;
        };
        groups
            .entry(formation_group_key(values))
            .or_default()
            .push(index);
    }

    for indices in groups.values() {
        let mut observed_matches = None;
        let mut total_usage = 0_i32;
        let mut formation_references = HashSet::new();
        let mut error = None;

        for index in indices {
            let Ok(values) = object(&rows[*index].payload) else {
                error = Some("阵型观察载荷不是有效对象".to_string());
                break;
            };
            let observed = text(values, "observed_matches")
                .and_then(|value| value.parse::<i32>().ok());
            let usage = text(values, "usage_count").and_then(|value| value.parse::<i32>().ok());
            let Some(observed) = observed else {
                error = Some("阵型观察场数必须是整数".to_string());
                break;
            };
            let Some(usage) = usage else {
                error = Some("阵型使用次数必须是整数".to_string());
                break;
            };
            if observed_matches.is_some_and(|current| current != observed) {
                error = Some("同一阵型观察分布的观察场数不一致".to_string());
                break;
            }
            observed_matches = Some(observed);
            total_usage += usage;

            let formation_reference = formation_record_reference(values);
            if !formation_reference.is_empty()
                && !formation_references.insert(formation_reference)
            {
                error = Some("同一阵型在观察窗口中重复".to_string());
                break;
            }
        }

        if error.is_none() && total_usage > observed_matches.unwrap_or_default() {
            error = Some(format!(
                "阵型使用次数合计超过观察场数（使用 {total_usage}，观察 {}）",
                observed_matches.unwrap_or_default()
            ));
        }

        if let Some(message) = error {
            for index in indices {
                rows[*index].status = SpreadsheetRowStatus::Error;
                rows[*index].message = Some(message.clone());
            }
        }
    }
}

async fn execute_formation_groups(
    tx: &mut Transaction<'_, Postgres>,
    rows: &[SpreadsheetImportRow],
) -> PersistenceResult<u64> {
    let mut groups: BTreeMap<FormationGroupKey, Vec<&SpreadsheetImportRow>> = BTreeMap::new();
    for row in rows {
        let values = object(&row.payload)?;
        groups
            .entry(formation_group_key(values))
            .or_default()
            .push(row);
    }
    let mut inserted = 0;
    for group in groups.values() {
        let first = object(&group[0].payload)?;
        let scope = require_text(first, "scope_type")?.to_ascii_lowercase();
        let (team_id, coach_id, competition_id) = match scope.as_str() {
            "team" => (
                Some(resolve_entity_id_tx(tx, first, "team").await?),
                None,
                None,
            ),
            "coach" => (
                None,
                Some(resolve_entity_id_tx(tx, first, "coach").await?),
                None,
            ),
            "team_coach" => (
                Some(resolve_entity_id_tx(tx, first, "team").await?),
                Some(resolve_entity_id_tx(tx, first, "coach").await?),
                None,
            ),
            "competition_default" => (
                None,
                None,
                Some(optional_uuid(first, "competition_id")?.ok_or_else(|| {
                    PersistenceError::InvalidState(
                        "competition_default 阵型分布缺少 competition_id".into(),
                    )
                })?),
            ),
            "system_default" => (None, None, None),
            _ => {
                return Err(PersistenceError::InvalidState(format!(
                    "阵型观察范围 {scope} 无效"
                )));
            }
        };
        let window_start = parse_date(require_text(first, "window_start")?, "window_start")?;
        let window_end = parse_date(require_text(first, "window_end")?, "window_end")?;
        let observed_matches =
            parse_i32(require_text(first, "observed_matches")?, "observed_matches")?;
        let confidence =
            parse_f64_default(text(first, "confidence").as_deref(), 0.5, "confidence")?;
        let alpha = parse_f64_default(text(first, "alpha").as_deref(), 3.0, "alpha")?;
        let observed_at = optional_datetime(first, "observed_at")?.unwrap_or_else(Utc::now);
        let mut counts = HashMap::<Uuid, i32>::new();
        let mut metadata = common_metadata(first);
        for row in group {
            let values = object(&row.payload)?;
            let formation_id = resolve_or_register_import_formation_tx(tx, values, row).await?;
            let usage = parse_i32(require_text(values, "usage_count")?, "usage_count")?;
            if counts.insert(formation_id, usage).is_some() {
                return Err(PersistenceError::InvalidState(
                    "同一阵型在观察窗口中重复".into(),
                ));
            }
        }
        let total: i32 = counts.values().sum();
        if total > observed_matches {
            return Err(PersistenceError::InvalidState(
                "阵型使用次数合计超过观察场数".into(),
            ));
        }
        if observed_matches == 0 {
            counts.clear();
            counts.insert(UNKNOWN_FORMATION_ID, 0);
        } else if total < observed_matches {
            *counts.entry(UNKNOWN_FORMATION_ID).or_default() += observed_matches - total;
        }
        let n = counts.len().max(1) as f64;
        metadata["source_urls"] = json!(source_urls(first));
        metadata["verified_at"] = json!(text(first, "verified_at"));
        for (formation_id, usage_count) in counts {
            let raw = if observed_matches == 0 {
                if formation_id == UNKNOWN_FORMATION_ID {
                    1.0
                } else {
                    0.0
                }
            } else {
                usage_count as f64 / observed_matches as f64
            };
            let smooth = if observed_matches == 0 {
                raw
            } else {
                (usage_count as f64 + alpha / n) / (observed_matches as f64 + alpha)
            };
            sqlx::query(
                r#"INSERT INTO feature.formation_usage_observations (
                id,scope_type,team_id,coach_id,competition_id,formation_id,window_preset,
                window_start,window_end,observed_matches,usage_count,raw_probability,
                smoothed_probability,confidence,smoothing_alpha,observed_at,metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17)
            ON CONFLICT DO NOTHING"#,
            )
            .bind(Uuid::new_v4())
            .bind(&scope)
            .bind(team_id)
            .bind(coach_id)
            .bind(competition_id)
            .bind(formation_id)
            .bind(text(first, "window_preset").unwrap_or_else(|| "custom".into()))
            .bind(window_start)
            .bind(window_end)
            .bind(observed_matches)
            .bind(usage_count)
            .bind(raw)
            .bind(smooth)
            .bind(confidence)
            .bind(alpha)
            .bind(observed_at)
            .bind(&metadata)
            .execute(&mut **tx)
            .await?;
            inserted += 1;
        }
    }
    Ok(inserted)
}

async fn resolve_or_register_import_formation_tx(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
    row: &SpreadsheetImportRow,
) -> PersistenceResult<Uuid> {
    let row_label = format!("工作表“{}”第 {} 行", row.sheet_name, row.row_number);
    if let Some(id) = optional_uuid(values, "_resolved_formation_id")? {
        let active = sqlx::query_scalar::<_, bool>(
            "SELECT is_active FROM football.formations WHERE id=$1",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?;
        return match active {
            Some(true) => Ok(id),
            Some(false) => Err(PersistenceError::InvalidState(format!(
                "{row_label}引用的已解析阵型ID {id} 已停用"
            ))),
            None => Err(PersistenceError::InvalidState(format!(
                "{row_label}引用的已解析阵型ID {id} 不存在"
            ))),
        };
    }

    if let Some(id) = optional_uuid(values, "formation_id")? {
        let active = sqlx::query_scalar::<_, bool>(
            "SELECT is_active FROM football.formations WHERE id=$1",
        )
        .bind(id)
        .fetch_optional(&mut **tx)
        .await?;
        if matches!(active, Some(true)) {
            return Ok(id);
        }
        if text(values, "formation_code").is_none() {
            return match active {
                Some(false) => Err(PersistenceError::InvalidState(format!(
                    "{row_label}引用的阵型ID {id} 已停用，且缺少 formation_code，无法重新绑定"
                ))),
                None => Err(PersistenceError::InvalidState(format!(
                    "{row_label}引用的阵型ID {id} 不存在，且缺少 formation_code，无法重新绑定"
                ))),
                Some(true) => unreachable!(),
            };
        }
    }

    let raw_code = require_text(values, "formation_code")?;
    let code = canonical_formation_code(&raw_code);
    let matches = sqlx::query(
        "SELECT id,is_active FROM football.formations WHERE lower(code)=lower($1) ORDER BY is_active DESC,id LIMIT 2",
    )
    .bind(&code)
    .fetch_all(&mut **tx)
    .await?;
    match matches.as_slice() {
        [match_row] if match_row.try_get::<bool, _>("is_active")? => {
            return Ok(match_row.try_get("id")?);
        }
        [match_row] => {
            let id: Uuid = match_row.try_get("id")?;
            return Err(PersistenceError::InvalidState(format!(
                "{row_label}的阵型 {code}（{id}）已停用"
            )));
        }
        [] => {}
        _ => {
            return Err(PersistenceError::InvalidState(format!(
                "{row_label}的阵型代码 {code} 在目录中存在多个大小写重复项"
            )));
        }
    }
    if !is_valid_custom_formation_code(&code) {
        return Err(PersistenceError::InvalidState(format!(
            "{row_label}的阵型代码 {raw_code} 无法识别；请使用目录中的阵型，或填写各线人数合计为 10 的代码（例如 3-4-1-2）"
        )));
    }

    let id = Uuid::new_v4();
    let inserted = sqlx::query_scalar::<_, Uuid>(
        r#"INSERT INTO football.formations (
               id,code,name,line_structure,slot_definition,is_builtin,is_active,sort_order,metadata
           ) VALUES ($1,$2,$2,$2,'[]'::jsonb,false,true,800,$3)
           ON CONFLICT (code) DO NOTHING
           RETURNING id"#,
    )
    .bind(id)
    .bind(&code)
    .bind(json!({
        "auto_registered": true,
        "source": TEAM_IMPORT_TYPE,
        "source_sheet": &row.sheet_name,
        "source_row": row.row_number,
    }))
    .fetch_optional(&mut **tx)
    .await?;
    if let Some(inserted_id) = inserted {
        write_audit_event(
            tx,
            "formation_auto_registered",
            "formation",
            Some(inserted_id.to_string()),
            json!({
                "code": &code,
                "source": TEAM_IMPORT_TYPE,
                "source_sheet": &row.sheet_name,
                "source_row": row.row_number,
            }),
        )
        .await?;
        return Ok(inserted_id);
    }

    sqlx::query_scalar::<_, Uuid>(
        "SELECT id FROM football.formations WHERE code=$1 AND is_active",
    )
    .bind(&code)
    .fetch_optional(&mut **tx)
    .await?
    .ok_or_else(|| {
        PersistenceError::InvalidState(format!(
            "{row_label}的自定义阵型 {code} 未能登记到阵型目录"
        ))
    })
}

async fn upsert_team_profile_from_values(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    values: &Map<String, Value>,
    metadata: &Value,
) -> PersistenceResult<()> {
    let team_type = normalize_team_type(text(values, "team_type"))?
        .unwrap_or_else(|| "club".to_string());
    sqlx::query(r#"INSERT INTO football.team_profiles (
        team_id,short_name,team_type,founded_year,city,stadium,data_confidence,notes,metadata,updated_at
    ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,now())
    ON CONFLICT (team_id) DO UPDATE SET
        short_name=COALESCE(EXCLUDED.short_name,football.team_profiles.short_name),
        team_type=COALESCE(NULLIF(EXCLUDED.team_type,''),football.team_profiles.team_type),
        founded_year=COALESCE(EXCLUDED.founded_year,football.team_profiles.founded_year),
        city=COALESCE(EXCLUDED.city,football.team_profiles.city),stadium=COALESCE(EXCLUDED.stadium,football.team_profiles.stadium),
        data_confidence=EXCLUDED.data_confidence,notes=COALESCE(EXCLUDED.notes,football.team_profiles.notes),
        metadata=football.team_profiles.metadata || EXCLUDED.metadata,updated_at=now()"#)
        .bind(team_id).bind(text(values,"short_name")).bind(team_type)
        .bind(optional_i16(values,"founded_year")?).bind(text(values,"city")).bind(text(values,"stadium"))
        .bind(parse_f64_default(text(values,"data_confidence").as_deref(),0.5,"data_confidence")?)
        .bind(text(values,"notes")).bind(metadata).execute(&mut **tx).await?;
    Ok(())
}

async fn apply_team_update(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    action: SpreadsheetAction,
    values: &Map<String, Value>,
    metadata: &Value,
) -> PersistenceResult<()> {
    let clear = if action == SpreadsheetAction::Clear {
        clear_fields(values)
    } else {
        HashSet::new()
    };
    let name = text(values, "official_name").unwrap_or_default();
    let team_type = normalize_team_type(text(values, "team_type"))?.unwrap_or_default();
    if !name.trim().is_empty() {
        preserve_current_team_canonical_alias(tx, team_id, metadata).await?;
    }
    sqlx::query(
        r#"UPDATE football.teams SET
        canonical_name=COALESCE(NULLIF($2,''),canonical_name),
        normalized_name=CASE WHEN NULLIF($2,'') IS NULL THEN normalized_name ELSE $3 END,
        country_code=CASE WHEN $4 THEN NULL ELSE COALESCE(NULLIF($5,''),country_code) END,
        is_active=COALESCE($6,is_active), metadata=metadata || $7, updated_at=now() WHERE id=$1"#,
    )
    .bind(team_id)
    .bind(&name)
    .bind(normalize_name(&name))
    .bind(clear.contains("country_code"))
    .bind(text(values, "country_code"))
    .bind(optional_bool(values, "is_active")?)
    .bind(metadata)
    .execute(&mut **tx)
    .await?;
    if !name.trim().is_empty() {
        ensure_team_name_alias(tx, team_id, &name, None, None, None, metadata).await?;
    }
    if let Some(short_name) = text(values, "short_name") {
        ensure_team_name_alias(tx, team_id, &short_name, None, None, None, metadata).await?;
    }
    sqlx::query(
        r#"INSERT INTO football.team_profiles (team_id, team_type, data_confidence, metadata, updated_at)
        VALUES ($1, 'club', 0.5, $2, now())
        ON CONFLICT (team_id) DO UPDATE SET
            short_name=CASE WHEN $3 THEN NULL ELSE COALESCE(NULLIF($4,''),football.team_profiles.short_name) END,
            team_type=COALESCE(NULLIF($5,''),football.team_profiles.team_type),
            founded_year=CASE WHEN $6 THEN NULL ELSE COALESCE($7,football.team_profiles.founded_year) END,
            city=CASE WHEN $8 THEN NULL ELSE COALESCE(NULLIF($9,''),football.team_profiles.city) END,
            stadium=CASE WHEN $10 THEN NULL ELSE COALESCE(NULLIF($11,''),football.team_profiles.stadium) END,
            data_confidence=COALESCE($12,football.team_profiles.data_confidence),
            notes=CASE WHEN $13 THEN NULL ELSE COALESCE(NULLIF($14,''),football.team_profiles.notes) END,
            metadata=football.team_profiles.metadata || EXCLUDED.metadata,
            updated_at=now()"#,
    )
    .bind(team_id)
    .bind(metadata)
    .bind(clear.contains("short_name"))
    .bind(text(values, "short_name").unwrap_or_default())
    .bind(team_type)
    .bind(clear.contains("founded_year"))
    .bind(optional_i16(values, "founded_year")?)
    .bind(clear.contains("city"))
    .bind(text(values, "city").unwrap_or_default())
    .bind(clear.contains("stadium"))
    .bind(text(values, "stadium").unwrap_or_default())
    .bind(optional_f64(values, "data_confidence")?)
    .bind(clear.contains("notes"))
    .bind(text(values, "notes").unwrap_or_default())
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn resolve_entity_id_tx(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
    prefix: &str,
) -> PersistenceResult<Uuid> {
    if let Some(id) = optional_uuid(values, &format!("_resolved_{prefix}_id"))?
        .or(optional_uuid(values, &format!("{prefix}_id"))?)
    {
        return Ok(id);
    }
    let name = text(values, &format!("{prefix}_name"))
        .or_else(|| text(values, "official_name"))
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少 {prefix} 名称")))?;
    let table = if prefix == "team" {
        "football.teams"
    } else {
        "football.coaches"
    };
    let ids = sqlx::query_scalar::<_, Uuid>(&format!(
        "SELECT id FROM {table} WHERE normalized_name=$1 ORDER BY id LIMIT 2"
    ))
    .bind(normalize_name(&name))
    .fetch_all(&mut **tx)
    .await?;
    match ids.as_slice() {
        [id] => Ok(*id),
        [] => Err(PersistenceError::InvalidState(format!(
            "提交时无法匹配 {prefix}: {name}"
        ))),
        _ => Err(PersistenceError::InvalidState(format!(
            "提交时 {prefix} 仍存在多个候选: {name}"
        ))),
    }
}
async fn resolve_optional_entity_id_tx(
    tx: &mut Transaction<'_, Postgres>,
    values: &Map<String, Value>,
    prefix: &str,
) -> PersistenceResult<Option<Uuid>> {
    let has = optional_uuid(values, &format!("_resolved_{prefix}_id"))?.is_some()
        || optional_uuid(values, &format!("{prefix}_id"))?.is_some()
        || text(values, &format!("{prefix}_name")).is_some();
    if has {
        Ok(Some(resolve_entity_id_tx(tx, values, prefix).await?))
    } else {
        Ok(None)
    }
}

async fn find_team_candidates(
    pool: &sqlx::PgPool,
    id: Option<Uuid>,
    name: &str,
    country: Option<&str>,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    if let Some(id) = id {
        let rows =
            sqlx::query("SELECT id,canonical_name,country_code FROM football.teams WHERE id=$1")
                .bind(id)
                .fetch_all(pool)
                .await?;
        return candidate_rows(&rows, "country_code");
    }
    if name.trim().is_empty() {
        return Ok(vec![]);
    }
    let rows = sqlx::query(
        r#"SELECT DISTINCT team.id,team.canonical_name,team.country_code
        FROM football.teams team LEFT JOIN football.team_names alias ON alias.team_id=team.id
        WHERE (team.normalized_name=$1 OR alias.normalized_name=$1)
          AND ($2::text IS NULL OR team.country_code IS NOT DISTINCT FROM $2)
        ORDER BY team.canonical_name,team.id LIMIT 20"#,
    )
    .bind(normalize_name(name))
    .bind(country)
    .fetch_all(pool)
    .await?;
    candidate_rows(&rows, "country_code")
}
async fn find_coach_candidates(
    pool: &sqlx::PgPool,
    id: Option<Uuid>,
    name: &str,
    nationality: Option<&str>,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    if let Some(id) = id {
        let rows = sqlx::query(
            "SELECT id,canonical_name,nationality_code FROM football.coaches WHERE id=$1",
        )
        .bind(id)
        .fetch_all(pool)
        .await?;
        return candidate_rows(&rows, "nationality_code");
    }
    if name.trim().is_empty() {
        return Ok(vec![]);
    }
    let rows = sqlx::query(
        r#"SELECT DISTINCT coach.id,coach.canonical_name,coach.nationality_code
        FROM football.coaches coach LEFT JOIN football.coach_names alias ON alias.coach_id=coach.id
        WHERE (coach.normalized_name=$1 OR alias.normalized_name=$1)
          AND ($2::text IS NULL OR coach.nationality_code IS NOT DISTINCT FROM $2)
        ORDER BY coach.canonical_name,coach.id LIMIT 20"#,
    )
    .bind(normalize_name(name))
    .bind(nationality)
    .fetch_all(pool)
    .await?;
    candidate_rows(&rows, "nationality_code")
}
fn candidate_rows(
    rows: &[sqlx::postgres::PgRow],
    detail_field: &str,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    rows.iter()
        .map(|row| {
            Ok(SpreadsheetConflictCandidate {
                entity_id: row.try_get("id")?,
                display_name: row.try_get("canonical_name")?,
                detail: row.try_get::<Option<String>, _>(detail_field)?,
            })
        })
        .collect()
}

async fn insert_team_import_row(
    tx: &mut Transaction<'_, Postgres>,
    batch_id: Uuid,
    row: &SpreadsheetImportRow,
) -> PersistenceResult<()> {
    sqlx::query(r#"INSERT INTO catalog.import_rows
        (id,batch_id,sheet_name,row_number,entity_type,requested_action,status,message,payload,matched_entity_id,conflict_candidates)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)"#)
        .bind(row.id).bind(batch_id).bind(&row.sheet_name).bind(row.row_number as i32)
        .bind(row.entity_type.as_str()).bind(row.action.as_str()).bind(row.status.as_str())
        .bind(&row.message).bind(&row.payload).bind(row.matched_entity_id).bind(serde_json::to_value(&row.conflict_candidates)?)
        .execute(&mut **tx).await?;
    Ok(())
}
fn team_import_row_from_db(row: &sqlx::postgres::PgRow) -> PersistenceResult<SpreadsheetImportRow> {
    Ok(SpreadsheetImportRow {
        id: row.try_get("id")?,
        sheet_name: row.try_get("sheet_name")?,
        row_number: row.try_get::<i32, _>("row_number")? as u32,
        entity_type: parse_entity_type(&row.try_get::<String, _>("entity_type")?)?,
        action: parse_action(&row.try_get::<String, _>("requested_action")?)?,
        status: parse_status(&row.try_get::<String, _>("status")?)?,
        message: row.try_get("message")?,
        payload: row.try_get("payload")?,
        matched_entity_id: row.try_get("matched_entity_id")?,
        conflict_candidates: serde_json::from_value(row.try_get("conflict_candidates")?)?,
    })
}
fn count_rows(rows: &[SpreadsheetImportRow]) -> SpreadsheetImportCounts {
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

fn monthly_gap_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<MonthlyDataGapRow> {
    Ok(MonthlyDataGapRow {
        entity_type: row.try_get("entity_type")?,
        entity_id: row.try_get("entity_id")?,
        entity_name: row.try_get("entity_name")?,
        missing_field: row.try_get("missing_field")?,
        last_observed_at: row.try_get("last_observed_at")?,
        stale_days: row.try_get("stale_days")?,
        priority: row.try_get("priority")?,
        recommended_action: row.try_get("recommended_action")?,
    })
}
fn object(value: &Value) -> PersistenceResult<&Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| PersistenceError::InvalidState("Excel 行内容不是对象".into()))
}
fn object_mut(value: &mut Value) -> PersistenceResult<&mut Map<String, Value>> {
    value
        .as_object_mut()
        .ok_or_else(|| PersistenceError::InvalidState("Excel 行内容不是对象".into()))
}
fn text(values: &Map<String, Value>, key: &str) -> Option<String> {
    values
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}
fn normalize_team_type(value: Option<String>) -> PersistenceResult<Option<String>> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let key = raw
        .trim()
        .to_lowercase()
        .chars()
        .filter(|character| {
            !character.is_whitespace()
                && !matches!(character, '_' | '-' | '\'' | '’' | '.' | '/')
        })
        .collect::<String>();
    let canonical = match key.as_str() {
        "club" | "clubs" | "clubteam" | "俱乐部" | "俱乐部队" => "club",
        "national"
        | "nationalteam"
        | "seniornational"
        | "seniornationalteam"
        | "国家队"
        | "国家代表队"
        | "成年国家队" => "national",
        "reserve"
        | "reserves"
        | "reserveteam"
        | "bteam"
        | "secondteam"
        | "预备队"
        | "二队"
        | "b队" => "reserve",
        "youth"
        | "youthteam"
        | "academy"
        | "academyteam"
        | "u18"
        | "u19"
        | "u20"
        | "u21"
        | "u23"
        | "青年队"
        | "青训队"
        | "梯队" => "youth",
        "women"
        | "womens"
        | "womenteam"
        | "womensteam"
        | "female"
        | "女足"
        | "女子队"
        | "女子足球队" => "women",
        "other" | "others" | "其他" | "其它" => "other",
        _ => {
            return Err(PersistenceError::InvalidState(format!(
                "球队类型 team_type={raw} 无效；允许 club/national/reserve/youth/women/other，常见别名 national_team、国家队、俱乐部、预备队、青年队、女足也可自动识别"
            )));
        }
    };
    Ok(Some(canonical.to_string()))
}

fn normalize_team_type_payload(payload: &mut Value) -> PersistenceResult<bool> {
    let values = object_mut(payload)?;
    let current = text(values, "team_type");
    let Some(canonical) = normalize_team_type(current.clone())? else {
        return Ok(false);
    };
    if current.as_deref() == Some(canonical.as_str()) {
        return Ok(false);
    }
    values.insert("team_type".into(), Value::String(canonical));
    Ok(true)
}

fn canonical_formation_code(value: &str) -> String {
    let mut canonical = String::with_capacity(value.len());
    for character in value.trim().chars() {
        match character {
            '０'..='９' => {
                let digit = (character as u32) - ('０' as u32) + ('0' as u32);
                if let Some(digit) = char::from_u32(digit) {
                    canonical.push(digit);
                }
            }
            '-' | '_' | '‐' | '‑' | '‒' | '–' | '—' | '−' | '﹘' | '﹣' | '－' => {
                canonical.push('-');
            }
            character if character.is_whitespace() => {}
            character => canonical.push(character.to_ascii_uppercase()),
        }
    }
    canonical
}

fn is_valid_custom_formation_code(value: &str) -> bool {
    let parts = value.split('-').collect::<Vec<_>>();
    if !(2..=5).contains(&parts.len()) {
        return false;
    }
    let mut total = 0_u8;
    for part in parts {
        if part.is_empty() || !part.chars().all(|character| character.is_ascii_digit()) {
            return false;
        }
        let Ok(count) = part.parse::<u8>() else {
            return false;
        };
        if count == 0 || count > 9 {
            return false;
        }
        total = total.saturating_add(count);
    }
    total == 10
}

fn normalize_formation_usage_payload(payload: &mut Value) -> PersistenceResult<bool> {
    let values = object_mut(payload)?;
    let mut changed = false;
    if let Some(current) = text(values, "formation_code") {
        let canonical = canonical_formation_code(&current);
        if current != canonical {
            values.insert("formation_code".into(), Value::String(canonical));
            changed = true;
        }
    }
    for key in ["scope_type", "window_preset"] {
        let Some(current) = text(values, key) else {
            continue;
        };
        let canonical = current.trim().to_ascii_lowercase();
        if current != canonical {
            values.insert(key.into(), Value::String(canonical));
            changed = true;
        }
    }
    Ok(changed)
}

fn require_text(values: &Map<String, Value>, key: &str) -> PersistenceResult<String> {
    text(values, key).ok_or_else(|| PersistenceError::InvalidState(format!("缺少必填字段 {key}")))
}
fn optional_uuid(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<Uuid>> {
    match text(values, key) {
        Some(v) => Uuid::parse_str(&v)
            .map(Some)
            .map_err(|_| PersistenceError::InvalidState(format!("{key} 不是有效 UUID"))),
        None => Ok(None),
    }
}
fn resolved_uuid(values: &Map<String, Value>, prefix: &str) -> PersistenceResult<Uuid> {
    optional_uuid(values, &format!("_resolved_{prefix}_id"))?
        .or(optional_uuid(values, &format!("{prefix}_id"))?)
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少已解析的 {prefix}_id")))
}
fn parse_date(value: String, key: &str) -> PersistenceResult<NaiveDate> {
    NaiveDate::parse_from_str(&value, "%Y-%m-%d")
        .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是 YYYY-MM-DD")))
}
fn optional_date(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<NaiveDate>> {
    text(values, key).map(|v| parse_date(v, key)).transpose()
}
fn parse_datetime(value: String, key: &str) -> PersistenceResult<DateTime<Utc>> {
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

fn canonical_datetime(value: String, key: &str) -> PersistenceResult<String> {
    parse_datetime(value, key)
        .map(|date_time| date_time.to_rfc3339_opts(SecondsFormat::AutoSi, true))
}

fn normalize_monthly_datetime_payload(payload: &mut Value) -> PersistenceResult<bool> {
    let values = object_mut(payload)?;
    let mut changed = false;
    for key in ["verified_at", "observed_at"] {
        let Some(current) = text(values, key) else {
            continue;
        };
        let canonical = canonical_datetime(current.clone(), key)?;
        if current != canonical {
            values.insert(key.into(), Value::String(canonical));
            changed = true;
        }
    }
    Ok(changed)
}

fn canonical_date(value: String, key: &str) -> PersistenceResult<String> {
    parse_datetime(value, key).map(|date_time| date_time.date_naive().to_string())
}

fn normalize_point_observation_window_payload(
    payload: &mut Value,
) -> PersistenceResult<bool> {
    let values = object_mut(payload)?;
    let current_start = text(values, "window_start");
    let current_end = text(values, "window_end");
    let anchor_date = text(values, "observed_at")
        .or_else(|| text(values, "verified_at"))
        .map(|value| canonical_date(value, "observed_at"))
        .transpose()?;

    let start = current_start
        .clone()
        .map(|value| canonical_date(value, "window_start"))
        .transpose()?;
    let end = current_end
        .clone()
        .map(|value| canonical_date(value, "window_end"))
        .transpose()?;

    let (normalized_start, normalized_end) = match (start, end) {
        (Some(start), Some(end)) => (start, end),
        (Some(start), None) => (start.clone(), start),
        (None, Some(end)) => (end.clone(), end),
        (None, None) => {
            let date = anchor_date.ok_or_else(|| {
                PersistenceError::InvalidState(
                    "球队能力或战术观察缺少 window_start/window_end，且没有 observed_at/verified_at 可用于生成点时窗口".into(),
                )
            })?;
            (date.clone(), date)
        }
    };

    let start_date = parse_date(normalized_start.clone(), "window_start")?;
    let end_date = parse_date(normalized_end.clone(), "window_end")?;
    if end_date < start_date {
        return Err(PersistenceError::InvalidState(
            "window_end 不能早于 window_start".into(),
        ));
    }

    let mut changed = false;
    if current_start.as_deref() != Some(normalized_start.as_str()) {
        values.insert("window_start".into(), Value::String(normalized_start));
        changed = true;
    }
    if current_end.as_deref() != Some(normalized_end.as_str()) {
        values.insert("window_end".into(), Value::String(normalized_end));
        changed = true;
    }
    Ok(changed)
}

fn optional_datetime(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<DateTime<Utc>>> {
    text(values, key)
        .map(|value| parse_datetime(value, key))
        .transpose()
}
fn parse_i32(value: String, key: &str) -> PersistenceResult<i32> {
    value
        .parse()
        .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是整数")))
}
fn parse_i32_default(value: Option<&str>, default: i32, key: &str) -> PersistenceResult<i32> {
    value
        .map(|v| parse_i32(v.to_string(), key))
        .transpose()
        .map(|v| v.unwrap_or(default))
}
fn optional_i16(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<i16>> {
    text(values, key)
        .map(|v| {
            v.parse()
                .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是整数")))
        })
        .transpose()
}
fn optional_f64(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<f64>> {
    text(values, key)
        .map(|v| {
            v.parse()
                .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是数字")))
        })
        .transpose()
}
fn parse_f64_default(value: Option<&str>, default: f64, key: &str) -> PersistenceResult<f64> {
    value
        .map(|v| {
            v.parse()
                .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是数字")))
        })
        .transpose()
        .map(|v| v.unwrap_or(default))
}
fn optional_bool(values: &Map<String, Value>, key: &str) -> PersistenceResult<Option<bool>> {
    text(values, key).map(|v| parse_bool(&v, key)).transpose()
}
fn parse_bool_default(value: Option<&str>, default: bool) -> PersistenceResult<bool> {
    value
        .map(|v| parse_bool(v, "boolean"))
        .transpose()
        .map(|v| v.unwrap_or(default))
}
fn parse_bool(value: &str, key: &str) -> PersistenceResult<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "是" => Ok(true),
        "false" | "0" | "no" | "否" => Ok(false),
        _ => Err(PersistenceError::InvalidState(format!(
            "{key} 必须是 true/false"
        ))),
    }
}
fn source_urls(values: &Map<String, Value>) -> Vec<String> {
    text(values, "source_urls")
        .map(|v| {
            v.split(['\n', ';'])
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
fn clear_fields(values: &Map<String, Value>) -> HashSet<String> {
    text(values, "clear_fields")
        .map(|v| {
            v.split([',', '，', ';'])
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}
fn common_metadata(values: &Map<String, Value>) -> Value {
    json!({
        "source_urls": source_urls(values),
        "verified_at": text(values, "verified_at"),
        "confidence": text(values, "confidence").and_then(|value| value.parse::<f64>().ok()),
        "formation_familiarity": text(values, "formation_familiarity")
            .and_then(|value| value.parse::<f64>().ok()),
        "notes": text(values, "notes"),
        "monthly_workbook": true
    })
}
fn normalize_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
fn import_mode_text(mode: SpreadsheetImportMode) -> &'static str {
    match mode {
        SpreadsheetImportMode::AddOnly => "add_only",
        SpreadsheetImportMode::AddAndUpdate => "add_and_update",
    }
}
fn parse_import_mode(value: Option<&str>) -> PersistenceResult<SpreadsheetImportMode> {
    match value.unwrap_or("add_and_update") {
        "add_only" => Ok(SpreadsheetImportMode::AddOnly),
        "add_and_update" => Ok(SpreadsheetImportMode::AddAndUpdate),
        other => Err(PersistenceError::InvalidState(format!(
            "未知导入模式 {other}"
        ))),
    }
}
fn parse_action(value: &str) -> PersistenceResult<SpreadsheetAction> {
    match value {
        "add" => Ok(SpreadsheetAction::Add),
        "update" => Ok(SpreadsheetAction::Update),
        "clear" => Ok(SpreadsheetAction::Clear),
        "skip" => Ok(SpreadsheetAction::Skip),
        other => Err(PersistenceError::InvalidState(format!("未知动作 {other}"))),
    }
}
fn parse_status(value: &str) -> PersistenceResult<SpreadsheetRowStatus> {
    match value {
        "ready_add" => Ok(SpreadsheetRowStatus::ReadyAdd),
        "ready_update" => Ok(SpreadsheetRowStatus::ReadyUpdate),
        "ready_end_previous" => Ok(SpreadsheetRowStatus::ReadyEndPrevious),
        "conflict" => Ok(SpreadsheetRowStatus::Conflict),
        "error" => Ok(SpreadsheetRowStatus::Error),
        "skip" => Ok(SpreadsheetRowStatus::Skip),
        "imported" => Ok(SpreadsheetRowStatus::Imported),
        other => Err(PersistenceError::InvalidState(format!("未知状态 {other}"))),
    }
}
fn parse_entity_type(value: &str) -> PersistenceResult<SpreadsheetEntityType> {
    match value {
        "team" => Ok(SpreadsheetEntityType::Team),
        "team_name" => Ok(SpreadsheetEntityType::TeamName),
        "coach" => Ok(SpreadsheetEntityType::Coach),
        "coach_name" => Ok(SpreadsheetEntityType::CoachName),
        "team_coach_period" => Ok(SpreadsheetEntityType::TeamCoachPeriod),
        "formation_usage" => Ok(SpreadsheetEntityType::FormationUsage),
        "team_tactical_observation" => Ok(SpreadsheetEntityType::TeamTacticalObservation),
        "team_ability_observation" => Ok(SpreadsheetEntityType::TeamAbilityObservation),
        other => Err(PersistenceError::InvalidState(format!(
            "球队月度批次包含未知实体 {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_urls_are_split_and_deduplicatable() {
        let values = Map::from_iter([(
            "source_urls".into(),
            Value::String("https://a.example\nhttps://b.example;https://c.example".into()),
        )]);
        assert_eq!(source_urls(&values).len(), 3);
    }
    #[test]
    fn blank_is_not_a_clear_instruction() {
        let values = Map::from_iter([("clear_fields".into(), Value::String(String::new()))]);
        assert!(clear_fields(&values).is_empty());
    }
    #[test]
    fn player_monthly_format_is_distinct_from_team_format() {
        assert_ne!(football_domain::PLAYER_MONTHLY_FORMAT, TEAM_MONTHLY_FORMAT);
    }
    #[test]
    fn team_type_aliases_are_normalized_before_database_write() {
        assert_eq!(
            normalize_team_type(Some("national_team".into())).expect("national alias"),
            Some("national".into())
        );
        assert_eq!(
            normalize_team_type(Some("国家队".into())).expect("Chinese national alias"),
            Some("national".into())
        );
        assert_eq!(
            normalize_team_type(Some("俱乐部".into())).expect("Chinese club alias"),
            Some("club".into())
        );
    }
    #[test]
    fn invalid_team_type_is_rejected_before_sql_constraint() {
        let error = normalize_team_type(Some("unsupported".into())).expect_err("invalid type");
        assert!(error.to_string().contains("球队类型"));
    }

    #[test]
    fn existing_batch_payload_is_canonicalized_before_commit() {
        let mut payload = json!({"team_type": "national_team"});
        assert!(normalize_team_type_payload(&mut payload).expect("normalize payload"));
        assert_eq!(payload["team_type"], "national");
        assert!(!normalize_team_type_payload(&mut payload).expect("idempotent payload"));
    }

    #[test]
    fn monthly_datetime_text_is_canonicalized_before_commit() {
        let mut payload = json!({
            "verified_at": "2026-07-01 00:00:00",
            "observed_at": "2026/07/02"
        });
        assert!(normalize_monthly_datetime_payload(&mut payload).expect("normalize datetime"));
        assert_eq!(payload["verified_at"], "2026-07-01T00:00:00Z");
        assert_eq!(payload["observed_at"], "2026-07-02T00:00:00Z");
        assert!(!normalize_monthly_datetime_payload(&mut payload).expect("idempotent datetime"));
    }

    #[test]
    fn monthly_datetime_accepts_excel_serial_cells() {
        assert_eq!(
            canonical_datetime("46204".into(), "verified_at").expect("Excel date"),
            "2026-07-01T00:00:00Z"
        );
        assert_eq!(
            canonical_datetime("46204.5".into(), "verified_at").expect("Excel datetime"),
            "2026-07-01T12:00:00Z"
        );
    }

    #[test]
    fn monthly_datetime_rejects_unrecognized_text_before_database_write() {
        let error = canonical_datetime("not-a-date".into(), "verified_at")
            .expect_err("invalid datetime");
        assert!(error.to_string().contains("Excel 日期单元格"));
    }


    #[test]
    fn point_observation_window_uses_end_for_missing_start() {
        let mut payload = json!({
            "window_start": "",
            "window_end": "2026-07-18",
            "observed_at": "2026-07-18T00:00:00Z"
        });
        assert!(normalize_point_observation_window_payload(&mut payload)
            .expect("normalize point window"));
        assert_eq!(payload["window_start"], "2026-07-18");
        assert_eq!(payload["window_end"], "2026-07-18");
        assert!(!normalize_point_observation_window_payload(&mut payload)
            .expect("idempotent point window"));
    }

    #[test]
    fn point_observation_window_uses_observed_at_when_both_dates_are_blank() {
        let mut payload = json!({
            "observed_at": "2026-07-18T15:30:00Z"
        });
        assert!(normalize_point_observation_window_payload(&mut payload)
            .expect("derive point window"));
        assert_eq!(payload["window_start"], "2026-07-18");
        assert_eq!(payload["window_end"], "2026-07-18");
    }

    #[test]
    fn point_observation_window_rejects_inverted_range() {
        let mut payload = json!({
            "window_start": "2026-07-19",
            "window_end": "2026-07-18",
            "observed_at": "2026-07-18T00:00:00Z"
        });
        let error = normalize_point_observation_window_payload(&mut payload)
            .expect_err("inverted point window");
        assert!(error.to_string().contains("window_end 不能早于 window_start"));
    }

    fn ready_add_team_row(sheet_name: &str, payload: Value) -> SpreadsheetImportRow {
        SpreadsheetImportRow {
            id: Uuid::new_v4(),
            sheet_name: sheet_name.into(),
            row_number: 4,
            entity_type: SpreadsheetEntityType::Team,
            action: SpreadsheetAction::Add,
            status: SpreadsheetRowStatus::ReadyAdd,
            message: None,
            payload,
            matched_entity_id: None,
            conflict_candidates: vec![],
        }
    }

    #[test]
    fn duplicate_package_team_identity_matches_explicit_and_implicit_rows() {
        let explicit = ready_add_team_row(
            "球队总览",
            json!({
                "official_name": "Atlético Mineiro",
                "country_code": "BRA",
                "team_type": "club",
                "stadium": "Arena MRV"
            }),
        );
        let implicit = ready_add_team_row(
            "球员与评分",
            json!({
                "official_name": "  ATLÉTICO   MINEIRO ",
                "country_code": "BRA",
                "team_type": "club"
            }),
        );
        assert_eq!(
            team_ready_add_identity(&explicit).expect("explicit identity"),
            team_ready_add_identity(&implicit).expect("implicit identity")
        );
        assert!(team_row_preference(&explicit) > team_row_preference(&implicit));
    }

    #[test]
    fn duplicate_package_team_source_identity_matches_translated_names() {
        let original = ready_add_team_row(
            "球队总览",
            json!({
                "official_name": "Atlético Mineiro",
                "country_code": "BRA",
                "team_type": "club",
                "source_urls": "https://atletico.com.br/futebol/masculino/elenco/"
            }),
        );
        let translated = ready_add_team_row(
            "球员与评分",
            json!({
                "official_name": "米内罗竞技",
                "country_code": "BRA",
                "team_type": "club",
                "source_urls": "https://atletico.com.br/futebol/masculino/elenco"
            }),
        );
        assert_eq!(
            team_ready_add_source_identity(&original).expect("original source identity"),
            team_ready_add_source_identity(&translated).expect("translated source identity")
        );
    }

    #[test]
    fn package_team_reference_uses_unique_source_when_name_changed() {
        let team_id = Uuid::new_v4();
        let teams = vec![BatchTeamReference {
            id: team_id,
            aliases: HashSet::from([normalize_name("米内罗竞技")]),
            source_urls: HashSet::from([
                "https://atletico.com.br/futebol/masculino/elenco".to_string(),
            ]),
        }];
        let (aliases, sources) = build_batch_team_reference_indexes(&teams);
        let values = serde_json::from_value::<Map<String, Value>>(json!({
            "team_name": "Atlético Mineiro",
            "source_urls": "https://atletico.com.br/futebol/masculino/elenco/"
        }))
        .expect("dependent payload");
        assert_eq!(
            resolve_batch_team_reference(&values, &aliases, &sources)
                .expect("resolve package team"),
            Some(team_id)
        );
    }

    #[test]
    fn duplicate_package_team_payload_only_fills_missing_fields() {
        let mut target = serde_json::from_value::<Map<String, Value>>(json!({
            "official_name": "Atlético Mineiro",
            "country_code": "BRA",
            "stadium": "Arena MRV",
            "notes": "完整球队资料"
        }))
        .expect("target payload");
        let source = serde_json::from_value::<Map<String, Value>>(json!({
            "official_name": "Atlético Mineiro",
            "country_code": "BRA",
            "team_type": "club",
            "notes": "球员名单推导资料"
        }))
        .expect("source payload");
        merge_missing_team_payload_fields(&mut target, &source);
        assert_eq!(target["team_type"], "club");
        assert_eq!(target["notes"], "完整球队资料");
        assert_eq!(target["stadium"], "Arena MRV");
    }

    fn formation_import_row(
        team_name: &str,
        coach_name: &str,
        formation_code: &str,
        observed_matches: i32,
        usage_count: i32,
    ) -> SpreadsheetImportRow {
        SpreadsheetImportRow {
            id: Uuid::new_v4(),
            sheet_name: "教练与阵型".into(),
            row_number: 4,
            entity_type: SpreadsheetEntityType::FormationUsage,
            action: SpreadsheetAction::Add,
            status: SpreadsheetRowStatus::ReadyAdd,
            message: None,
            payload: json!({
                "scope_type": "team_coach",
                "team_id": "",
                "team_name": team_name,
                "coach_id": "",
                "coach_name": coach_name,
                "formation_code": formation_code,
                "window_start": "2026-06-11",
                "window_end": "2026-07-18",
                "observed_at": "2026-07-18T15:30:00Z",
                "observed_matches": observed_matches.to_string(),
                "usage_count": usage_count.to_string()
            }),
            matched_entity_id: None,
            conflict_candidates: vec![],
        }
    }

    #[test]
    fn formation_group_key_separates_blank_ids_by_team_and_coach_names() {
        let first = formation_import_row("法国", "Didier Deschamps", "4-2-3-1", 7, 7);
        let second = formation_import_row("英格兰", "Thomas Tuchel", "4-2-3-1", 7, 7);
        assert_ne!(
            formation_group_key(object(&first.payload).expect("first payload")),
            formation_group_key(object(&second.payload).expect("second payload"))
        );
    }

    #[test]
    fn formation_group_preview_keeps_distinct_teams_separate() {
        let mut rows = vec![
            formation_import_row("法国", "Didier Deschamps", "4-2-3-1", 7, 7),
            formation_import_row("英格兰", "Thomas Tuchel", "4-2-3-1", 7, 7),
        ];
        validate_formation_group_rows(&mut rows);
        assert!(rows
            .iter()
            .all(|row| row.status == SpreadsheetRowStatus::ReadyAdd));
    }

    #[test]
    fn formation_group_preview_rejects_aggregate_overflow() {
        let mut rows = vec![
            formation_import_row("法国", "Didier Deschamps", "4-2-3-1", 7, 5),
            formation_import_row("法国", "Didier Deschamps", "4-3-3", 7, 4),
        ];
        validate_formation_group_rows(&mut rows);
        assert!(rows
            .iter()
            .all(|row| row.status == SpreadsheetRowStatus::Error));
        assert!(rows.iter().all(|row| {
            row.message
                .as_deref()
                .is_some_and(|message| message.contains("使用 9，观察 7"))
        }));
    }

    #[test]
    fn formation_code_normalizes_excel_unicode_before_lookup() {
        assert_eq!(canonical_formation_code(" ３–４–１–２ "), "3-4-1-2");
        assert_eq!(canonical_formation_code("unknown"), "UNKNOWN");
    }

    #[test]
    fn custom_formation_requires_ten_outfield_players() {
        assert!(is_valid_custom_formation_code("3-4-1-2"));
        assert!(is_valid_custom_formation_code("4-3-2-1"));
        assert!(!is_valid_custom_formation_code("4-4-2-1"));
        assert!(!is_valid_custom_formation_code("4-3-X-3"));
    }

    #[test]
    fn formation_payload_normalization_is_idempotent() {
        let mut payload = json!({
            "scope_type": " TEAM_COACH ",
            "window_preset": " LAST_10 ",
            "formation_code": "３‑４‑１‑２"
        });
        assert!(normalize_formation_usage_payload(&mut payload).expect("normalize formation"));
        assert_eq!(payload["scope_type"], "team_coach");
        assert_eq!(payload["window_preset"], "last_10");
        assert_eq!(payload["formation_code"], "3-4-1-2");
        assert!(!normalize_formation_usage_payload(&mut payload).expect("idempotent formation"));
    }
}
