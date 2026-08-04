use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    MonthlyWorkbookExportSummary, MonthlyWorkbookKind, SpreadsheetEntityType, SpreadsheetExportSummary,
    SpreadsheetImportCommitResult, SpreadsheetImportMode, SpreadsheetImportPreview,
    SpreadsheetImportResolution, SpreadsheetParsedWorkbook, TeamPackageCommitRequest,
    TeamPackageCommitResult, TeamPackageCoverage, TeamPackageExportSummary,
    TeamPackageImportPreview, TeamPackagePreviewExportSummary, PLAYER_MONTHLY_FORMAT,
    TEAM_MONTHLY_FORMAT, TEAM_PACKAGE_FORMAT, TEAM_PACKAGE_PREVIEW_EXPORT_FORMAT,
};
use football_spreadsheet_io::{
    read_player_catalog_workbook, read_player_monthly_workbook, read_team_monthly_workbook,
    read_team_package_workbook, write_player_monthly_export, write_player_monthly_template,
    write_team_monthly_export, write_team_monthly_template, write_team_package_template,
};
use chrono::Utc;
use serde_json::json;
use std::{collections::HashMap, path::PathBuf};
use uuid::Uuid;

impl ApplicationService {
    pub async fn export_team_package_template(
        &self,
        output_path: String,
    ) -> ApplicationResult<TeamPackageExportSummary> {
        let path = validate_xlsx_path(&output_path)?;
        let store = self.active_store().await?;
        let references = store.player_catalog_reference_data().await?;
        let output = path.clone();
        tokio::task::spawn_blocking(move || write_team_package_template(&output, &references))
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!("球队完整资料包模板导出失败：{error}"))
            })??;
        Ok(TeamPackageExportSummary {
            output_path: path.to_string_lossy().to_string(),
            format_version: TEAM_PACKAGE_FORMAT.to_string(),
            visible_sheet_count: 7,
        })
    }


    pub async fn export_team_package_preview_json(
        &self,
        output_path: String,
        preview: TeamPackageImportPreview,
    ) -> ApplicationResult<TeamPackagePreviewExportSummary> {
        let path = validate_json_path(&output_path)?;
        let team_row_count = preview
            .team_preview
            .as_ref()
            .map(|value| value.rows.len() as u64)
            .unwrap_or(0);
        let player_row_count = preview
            .player_preview
            .as_ref()
            .map(|value| value.rows.len() as u64)
            .unwrap_or(0);
        let exported_row_count = team_row_count + player_row_count;
        let payload = json!({
            "format_version": TEAM_PACKAGE_PREVIEW_EXPORT_FORMAT,
            "exported_at": Utc::now(),
            "source": {
                "file_name": preview.source_file_name,
                "sha256": preview.source_sha256,
            },
            "summary": {
                "team_row_count": team_row_count,
                "player_row_count": player_row_count,
                "exported_row_count": exported_row_count,
                "coverage": preview.coverage,
            },
            "team_preview": preview.team_preview,
            "player_preview": preview.player_preview,
        });
        let output = path.clone();
        tokio::task::spawn_blocking(move || -> ApplicationResult<()> {
            let bytes = serde_json::to_vec_pretty(&payload).map_err(|error| {
                ApplicationError::Validation(format!("完整预检 JSON 序列化失败：{error}"))
            })?;
            std::fs::write(&output, bytes).map_err(|error| {
                ApplicationError::Validation(format!(
                    "完整预检 JSON 写入失败 {}：{error}",
                    output.display()
                ))
            })?;
            Ok(())
        })
        .await
        .map_err(|error| {
            ApplicationError::Validation(format!("完整预检 JSON 导出任务失败：{error}"))
        })??;
        Ok(TeamPackagePreviewExportSummary {
            output_path: path.to_string_lossy().to_string(),
            format_version: TEAM_PACKAGE_PREVIEW_EXPORT_FORMAT.to_string(),
            exported_row_count,
        })
    }

    pub async fn preview_team_package_import(
        &self,
        input_path: String,
        mode: SpreadsheetImportMode,
    ) -> ApplicationResult<TeamPackageImportPreview> {
        let path = validate_existing_xlsx_path(&input_path)?;
        let parsed = tokio::task::spawn_blocking(move || {
            match read_team_package_workbook(&path) {
                Ok(parsed) => Ok(parsed),
                Err(_original) if read_team_monthly_workbook(&path).is_ok() => Err(
                    "检测到 football.team-monthly.v1 球队月度工作包。请使用“Excel 工作包 → 球队月度”导入；完整资料包入口仅接受 football.team-package.v1。".to_string(),
                ),
                Err(_original) if read_player_monthly_workbook(&path).is_ok() || read_player_catalog_workbook(&path).is_ok() => Err(
                    "检测到球员工作包。请切换到“球队与人员 → 球员 → 球员工作包”导入；当前入口仅接受球队完整资料包。".to_string(),
                ),
                Err(original) => Err(format!("球队完整资料包读取失败：{original}")),
            }
        })
        .await
        .map_err(|error| ApplicationError::Validation(format!("球队完整资料包读取任务失败：{error}")))?
        .map_err(ApplicationError::Validation)?;
        let package_team_references = collect_package_team_references(&parsed)?;
        let store = self.active_store().await?;
        let team_rows = parsed
            .rows
            .iter()
            .filter(|row| is_team_package_team_entity(row.entity_type))
            .cloned()
            .collect::<Vec<_>>();
        let player_rows = parsed
            .rows
            .iter()
            .filter(|row| is_team_package_player_entity(row.entity_type))
            .cloned()
            .collect::<Vec<_>>();
        let team_parsed = SpreadsheetParsedWorkbook {
            format_version: TEAM_MONTHLY_FORMAT.to_string(),
            source_file_name: parsed.source_file_name.clone(),
            source_sha256: parsed.source_sha256.clone(),
            rows: team_rows,
        };
        let player_parsed = SpreadsheetParsedWorkbook {
            format_version: PLAYER_MONTHLY_FORMAT.to_string(),
            source_file_name: parsed.source_file_name.clone(),
            source_sha256: parsed.source_sha256.clone(),
            rows: player_rows,
        };
        let team_preview = if team_parsed.rows.is_empty() {
            None
        } else {
            Some(store.preview_team_monthly_import(&team_parsed, mode).await?)
        };
        let player_preview = if player_parsed.rows.is_empty() {
            None
        } else {
            Some(
                store
                    .preview_spreadsheet_import_with_team_references(
                        &player_parsed,
                        mode,
                        &package_team_references,
                    )
                    .await?,
            )
        };
        let coverage = team_package_coverage(&parsed, team_preview.as_ref(), player_preview.as_ref());
        Ok(TeamPackageImportPreview {
            source_file_name: parsed.source_file_name,
            source_sha256: parsed.source_sha256,
            team_preview,
            player_preview,
            coverage,
        })
    }

    pub async fn commit_team_package_import(
        &self,
        request: TeamPackageCommitRequest,
    ) -> ApplicationResult<TeamPackageCommitResult> {
        if request.team_batch_id.is_none() && request.player_batch_id.is_none() {
            return Err(ApplicationError::Validation(
                "球队完整资料包没有可提交的预检批次".to_string(),
            ));
        }
        let store = self.active_store().await?;
        if let Some(batch_id) = request.team_batch_id {
            let preview = store
                .read_team_monthly_import_preview(batch_id)
                .await
                .map_err(|error| {
                    ApplicationError::Validation(format!(
                        "完整资料包球队链预检批次 {batch_id} 无法读取：{error}"
                    ))
                })?;
            ensure_preview_committable(&preview, "球队、教练和阵型")?;
        }
        if let Some(batch_id) = request.player_batch_id {
            let preview = store
                .read_spreadsheet_import_preview(batch_id)
                .await
                .map_err(|error| {
                    ApplicationError::Validation(format!(
                        "完整资料包球员链预检批次 {batch_id} 无法读取：{error}"
                    ))
                })?;
            ensure_preview_committable(&preview, "球员、评分和动态状态")?;
        }
        let team_result = match request.team_batch_id {
            Some(batch_id) => Some(
                store
                    .commit_team_monthly_import(batch_id)
                    .await
                    .map_err(|error| {
                        ApplicationError::Validation(format!(
                            "完整资料包球队、教练与阵型链提交失败（批次 {batch_id}）：{error}"
                        ))
                    })?,
            ),
            None => None,
        };
        let player_result = match request.player_batch_id {
            Some(batch_id) => Some(
                store
                    .commit_spreadsheet_import(batch_id)
                    .await
                    .map_err(|error| {
                        let team_state = if team_result.is_some() {
                            "球队、教练与阵型链已经提交成功；可修复后直接重试同一完整资料包批次。"
                        } else {
                            ""
                        };
                        ApplicationError::Validation(format!(
                            "{team_state}完整资料包球员、评分与动态状态链提交失败（批次 {batch_id}）：{error}"
                        ))
                    })?,
            ),
            None => None,
        };
        let inserted_count = team_result
            .iter()
            .chain(player_result.iter())
            .map(|value| value.inserted_count)
            .sum();
        let updated_count = team_result
            .iter()
            .chain(player_result.iter())
            .map(|value| value.updated_count)
            .sum();
        let ended_previous_count = team_result
            .iter()
            .chain(player_result.iter())
            .map(|value| value.ended_previous_count)
            .sum();
        let skipped_count = team_result
            .iter()
            .chain(player_result.iter())
            .map(|value| value.skipped_count)
            .sum();
        let error_count = team_result
            .iter()
            .chain(player_result.iter())
            .map(|value| value.error_count)
            .sum();
        Ok(TeamPackageCommitResult {
            team_result,
            player_result,
            inserted_count,
            updated_count,
            ended_previous_count,
            skipped_count,
            error_count,
        })
    }

    pub async fn export_player_catalog_template(
        &self,
        output_path: String,
    ) -> ApplicationResult<SpreadsheetExportSummary> {
        let path = validate_xlsx_path(&output_path)?;
        let store = self.active_store().await?;
        let references = store.player_catalog_reference_data().await?;
        let output = path.clone();
        tokio::task::spawn_blocking(move || write_player_monthly_template(&output, &references))
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!("模板导出任务失败：{error}"))
            })??;
        Ok(SpreadsheetExportSummary {
            output_path: path.to_string_lossy().to_string(),
            team_count: 0,
            player_count: 0,
            related_row_count: 0,
        })
    }

    pub async fn export_player_catalog_data(
        &self,
        output_path: String,
    ) -> ApplicationResult<SpreadsheetExportSummary> {
        let path = validate_xlsx_path(&output_path)?;
        let store = self.active_store().await?;
        let references = store.player_catalog_reference_data().await?;
        let data = store.spreadsheet_export_data().await?;
        let gaps = store.player_monthly_data_gaps().await?;
        let team_count = data.teams.len() as u64;
        let player_count = data.players.len() as u64;
        let related_row_count = (data.names.len()
            + data.positions.len()
            + data.team_periods.len()
            + data.abilities.len()
            + data.availability.len()
            + data.dynamic_tags.len()) as u64;
        let output = path.clone();
        tokio::task::spawn_blocking(move || {
            write_player_monthly_export(&output, &references, &data, &gaps)
        })
        .await
        .map_err(|error| ApplicationError::Validation(format!("数据导出任务失败：{error}")))??;
        Ok(SpreadsheetExportSummary {
            output_path: path.to_string_lossy().to_string(),
            team_count,
            player_count,
            related_row_count,
        })
    }

    pub async fn preview_player_catalog_import(
        &self,
        input_path: String,
        mode: SpreadsheetImportMode,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let path = validate_existing_xlsx_path(&input_path)?;
        let parsed = tokio::task::spawn_blocking(move || {
            match read_player_monthly_workbook(&path).or_else(|_| read_player_catalog_workbook(&path)) {
                Ok(parsed) => Ok(parsed),
                Err(_original) if read_team_monthly_workbook(&path).is_ok() => Err(
                    "检测到 football.team-monthly.v1 球队月度工作包。请使用“Excel 工作包 → 球队月度”导入；球员入口不会把球队文件误写为球员。".to_string(),
                ),
                Err(_original) if read_team_package_workbook(&path).is_ok() => Err(
                    "检测到 football.team-package.v1 球队完整资料包。请使用“球队 → 导入资料包”统一预检球队与球员链路。".to_string(),
                ),
                Err(original) => Err(format!("球员工作包读取失败：{original}")),
            }
        })
        .await
        .map_err(|error| ApplicationError::Validation(format!("球员 Excel 读取任务失败：{error}")))?
        .map_err(ApplicationError::Validation)?;
        let store = self.active_store().await?;
        Ok(store.preview_spreadsheet_import(&parsed, mode).await?)
    }

    pub async fn resolve_player_catalog_import_conflict(
        &self,
        batch_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let store = self.active_store().await?;
        Ok(store
            .resolve_spreadsheet_import_conflict(batch_id, resolution)
            .await?)
    }

    pub async fn commit_player_catalog_import(
        &self,
        batch_id: Uuid,
    ) -> ApplicationResult<SpreadsheetImportCommitResult> {
        let store = self.active_store().await?;
        Ok(store.commit_spreadsheet_import(batch_id).await?)
    }

    pub async fn read_player_catalog_import_preview(
        &self,
        batch_id: Uuid,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let store = self.active_store().await?;
        Ok(store.read_spreadsheet_import_preview(batch_id).await?)
    }

    pub async fn export_team_monthly_template(
        &self,
        output_path: String,
    ) -> ApplicationResult<MonthlyWorkbookExportSummary> {
        let path = validate_xlsx_path(&output_path)?;
        let store = self.active_store().await?;
        let references = store.player_catalog_reference_data().await?;
        let output = path.clone();
        tokio::task::spawn_blocking(move || write_team_monthly_template(&output, &references))
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!("球队模板导出失败：{error}"))
            })??;
        Ok(MonthlyWorkbookExportSummary {
            output_path: path.to_string_lossy().to_string(),
            workbook_kind: MonthlyWorkbookKind::Team,
            team_count: 0,
            player_count: 0,
            coach_count: 0,
            related_row_count: 0,
            data_gap_count: 0,
        })
    }

    pub async fn export_team_monthly_data(
        &self,
        output_path: String,
    ) -> ApplicationResult<MonthlyWorkbookExportSummary> {
        let path = validate_xlsx_path(&output_path)?;
        let store = self.active_store().await?;
        let references = store.player_catalog_reference_data().await?;
        let data = store.team_monthly_workbook_data().await?;
        let summary = MonthlyWorkbookExportSummary {
            output_path: path.to_string_lossy().to_string(),
            workbook_kind: MonthlyWorkbookKind::Team,
            team_count: data.teams.len() as u64,
            player_count: 0,
            coach_count: data.coaches.len() as u64,
            related_row_count: (data.names.len()
                + data.coach_periods.len()
                + data.formation_usage.len()
                + data.tactical_observations.len()
                + data.ability_observations.len()) as u64,
            data_gap_count: data.data_gaps.len() as u64,
        };
        let output = path.clone();
        tokio::task::spawn_blocking(move || write_team_monthly_export(&output, &references, &data))
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!("球队数据导出失败：{error}"))
            })??;
        Ok(summary)
    }

    pub async fn preview_team_monthly_import(
        &self,
        input_path: String,
        mode: SpreadsheetImportMode,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let path = validate_existing_xlsx_path(&input_path)?;
        let parsed = tokio::task::spawn_blocking(move || read_team_monthly_workbook(&path))
            .await
            .map_err(|error| {
                ApplicationError::Validation(format!("球队 Excel 读取失败：{error}"))
            })??;
        let store = self.active_store().await?;
        Ok(store.preview_team_monthly_import(&parsed, mode).await?)
    }

    pub async fn read_team_monthly_import_preview(
        &self,
        batch_id: Uuid,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let store = self.active_store().await?;
        Ok(store.read_team_monthly_import_preview(batch_id).await?)
    }

    pub async fn resolve_team_monthly_import_conflict(
        &self,
        batch_id: Uuid,
        resolution: SpreadsheetImportResolution,
    ) -> ApplicationResult<SpreadsheetImportPreview> {
        let store = self.active_store().await?;
        Ok(store
            .resolve_team_monthly_import_conflict(batch_id, resolution)
            .await?)
    }

    pub async fn commit_team_monthly_import(
        &self,
        batch_id: Uuid,
    ) -> ApplicationResult<SpreadsheetImportCommitResult> {
        let store = self.active_store().await?;
        Ok(store.commit_team_monthly_import(batch_id).await?)
    }
}

fn collect_package_team_references(
    parsed: &SpreadsheetParsedWorkbook,
) -> ApplicationResult<HashMap<String, String>> {
    let mut references = HashMap::new();
    for row in parsed.rows.iter().filter(|row| {
        row.entity_type == SpreadsheetEntityType::Team
            && row.action != football_domain::SpreadsheetAction::Skip
    }) {
        let Some(values) = row.values.as_object() else {
            continue;
        };
        let key = values
            .get("short_name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let name = values
            .get("official_name")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty());
        let (Some(key), Some(name)) = (key, name) else {
            continue;
        };
        let normalized_key = key.to_ascii_uppercase();
        if let Some(existing) = references.insert(normalized_key, name.to_string()) {
            if existing != name {
                return Err(ApplicationError::Validation(format!(
                    "球队简称 {key} 在资料包中对应多个球队：{existing} / {name}"
                )));
            }
        }
    }
    Ok(references)
}

fn is_team_package_team_entity(entity_type: SpreadsheetEntityType) -> bool {
    matches!(
        entity_type,
        SpreadsheetEntityType::Team
            | SpreadsheetEntityType::TeamName
            | SpreadsheetEntityType::Coach
            | SpreadsheetEntityType::CoachName
            | SpreadsheetEntityType::TeamCoachPeriod
            | SpreadsheetEntityType::FormationUsage
            | SpreadsheetEntityType::TeamTacticalObservation
            | SpreadsheetEntityType::TeamAbilityObservation
    )
}

fn is_team_package_player_entity(entity_type: SpreadsheetEntityType) -> bool {
    matches!(
        entity_type,
        SpreadsheetEntityType::Player
            | SpreadsheetEntityType::PlayerName
            | SpreadsheetEntityType::PlayerPosition
            | SpreadsheetEntityType::PlayerTeamPeriod
            | SpreadsheetEntityType::PlayerAbility
            | SpreadsheetEntityType::PlayerAvailability
            | SpreadsheetEntityType::PlayerDynamicTag
            | SpreadsheetEntityType::ExternalEntityId
    )
}

fn ensure_preview_committable(
    preview: &SpreadsheetImportPreview,
    label: &str,
) -> ApplicationResult<()> {
    let blocking = preview.counts.conflict + preview.counts.error;
    let ready = preview.counts.ready_add
        + preview.counts.ready_update
        + preview.counts.ready_end_previous;
    if blocking > 0 {
        return Err(ApplicationError::Validation(format!(
            "{label}预检仍有 {} 条冲突或错误，不能提交",
            blocking
        )));
    }
    if ready == 0 && preview.counts.imported == 0 {
        return Err(ApplicationError::Validation(format!(
            "{label}预检没有可写入或已完成记录"
        )));
    }
    Ok(())
}

fn team_package_coverage(
    parsed: &SpreadsheetParsedWorkbook,
    team_preview: Option<&SpreadsheetImportPreview>,
    player_preview: Option<&SpreadsheetImportPreview>,
) -> TeamPackageCoverage {
    let count = |entity_type| {
        parsed
            .rows
            .iter()
            .filter(|row| row.entity_type == entity_type && row.action != football_domain::SpreadsheetAction::Skip)
            .count() as u64
    };
    let team_count = count(SpreadsheetEntityType::Team);
    let player_count = count(SpreadsheetEntityType::Player);
    let coach_count = count(SpreadsheetEntityType::Coach);
    let formation_usage_count = count(SpreadsheetEntityType::FormationUsage);
    let team_ability_count = count(SpreadsheetEntityType::TeamAbilityObservation);
    let player_ability_count = count(SpreadsheetEntityType::PlayerAbility);
    let player_dynamic_tag_count = count(SpreadsheetEntityType::PlayerDynamicTag);
    let player_role_count = parsed
        .rows
        .iter()
        .filter(|row| {
            row.entity_type == SpreadsheetEntityType::PlayerPosition
                && row.action != football_domain::SpreadsheetAction::Skip
                && row
                    .values
                    .get("default_role_code")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| !value.trim().is_empty())
        })
        .count() as u64;
    let availability_count = count(SpreadsheetEntityType::PlayerAvailability);
    let preview_blocking = team_preview
        .map(|value| value.counts.conflict + value.counts.error)
        .unwrap_or_default()
        + player_preview
            .map(|value| value.counts.conflict + value.counts.error)
            .unwrap_or_default();
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    if preview_blocking > 0 {
        blockers.push(format!(
            "完整资料包预检仍有 {preview_blocking} 条冲突或错误，修复后才能正式提交"
        ));
    }
    if team_count == 0 { blockers.push("缺少球队总览记录".to_string()); }
    if player_count < 11 { blockers.push(format!("有效球员只有 {player_count} 人，低于 P4 阵容输入下限 11 人")); }
    if coach_count == 0 { warnings.push("没有教练记录，阵型只能回退到球队或系统默认".to_string()); }
    if formation_usage_count == 0 { warnings.push("没有阵型使用分布，P4 将回退未知阵型".to_string()); }
    if team_ability_count == 0 { warnings.push("没有球队能力观察".to_string()); }
    if player_count > 0 && player_ability_count < player_count * 4 {
        warnings.push(format!("球员能力观察只有 {player_ability_count} 条，建议至少每名球员填写 4 个核心维度"));
    }
    if player_count > 0 && player_dynamic_tag_count < player_count * 3 {
        warnings.push(format!("球员动态标签只有 {player_dynamic_tag_count} 条，建议至少覆盖准备度、状态和体能"));
    }
    if player_count > 0 && player_role_count < player_count.min(11) {
        warnings.push(format!(
            "默认战术角色只有 {player_role_count} 条；未覆盖球员仍可保存阵容，但会降低角色输入完整度"
        ));
    }
    let player_factor = (player_count.min(26) as f64 / 26.0 * 20.0).round() as u8;
    let ability_target = (player_count * 8).max(1);
    let ability_factor = ((player_ability_count.min(ability_target) as f64 / ability_target as f64) * 25.0).round() as u8;
    let tag_target = (player_count * 5).max(1);
    let tag_factor = ((player_dynamic_tag_count.min(tag_target) as f64 / tag_target as f64) * 20.0).round() as u8;
    let mut readiness_score = 0_u8;
    if team_count > 0 { readiness_score += 15; }
    readiness_score += player_factor;
    if coach_count > 0 { readiness_score += 8; }
    if formation_usage_count > 0 { readiness_score += 7; }
    if team_ability_count > 0 { readiness_score += 5; }
    readiness_score += ability_factor;
    readiness_score += tag_factor;
    readiness_score = readiness_score.min(100);
    if preview_blocking > 0 {
        readiness_score = readiness_score.min(69);
    }
    TeamPackageCoverage {
        team_count,
        player_count,
        coach_count,
        formation_usage_count,
        team_ability_count,
        player_ability_count,
        player_dynamic_tag_count,
        player_role_count,
        availability_count,
        readiness_score,
        p4_input_ready: blockers.is_empty() && preview_blocking == 0 && readiness_score >= 70,
        blockers,
        warnings,
    }
}

fn validate_json_path(value: &str) -> ApplicationResult<PathBuf> {
    let path = PathBuf::from(value.trim());
    if path.as_os_str().is_empty() {
        return Err(ApplicationError::Validation(
            "请选择 JSON 输出位置".to_string(),
        ));
    }
    if path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_lowercase)
        .as_deref()
        != Some("json")
    {
        return Err(ApplicationError::Validation(
            "输出文件必须使用 .json 扩展名".to_string(),
        ));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            ApplicationError::Validation(format!(
                "无法创建输出目录 {}：{error}",
                parent.display()
            ))
        })?;
    }
    Ok(path)
}

fn validate_xlsx_path(value: &str) -> ApplicationResult<PathBuf> {
    let path = PathBuf::from(value.trim());
    if path.as_os_str().is_empty() {
        return Err(ApplicationError::Validation(
            "请选择 Excel 输出位置".to_string(),
        ));
    }
    if path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_lowercase)
        .as_deref()
        != Some("xlsx")
    {
        return Err(ApplicationError::Validation(
            "输出文件必须使用 .xlsx 扩展名".to_string(),
        ));
    }
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent).map_err(|error| {
            ApplicationError::Validation(format!("无法创建输出目录 {}：{error}", parent.display()))
        })?;
    }
    Ok(path)
}

fn validate_existing_xlsx_path(value: &str) -> ApplicationResult<PathBuf> {
    let path = validate_xlsx_path(value)?;
    if !path.is_file() {
        return Err(ApplicationError::Validation(format!(
            "Excel 文件不存在：{}",
            path.display()
        )));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::SpreadsheetImportCounts;

    fn preview_with_counts(counts: SpreadsheetImportCounts) -> SpreadsheetImportPreview {
        SpreadsheetImportPreview {
            batch_id: Uuid::nil(),
            source_file_name: "fixture.xlsx".to_string(),
            source_sha256: "fixture".to_string(),
            import_mode: SpreadsheetImportMode::AddAndUpdate,
            counts,
            rows: Vec::new(),
            created_at: Utc::now(),
        }
    }

    #[test]
    fn already_imported_team_chain_remains_retryable() {
        let preview = preview_with_counts(SpreadsheetImportCounts {
            total: 8,
            imported: 8,
            ..Default::default()
        });
        ensure_preview_committable(&preview, "球队链")
            .expect("已提交球队链应允许继续重试完整资料包球员链");
    }

    #[test]
    fn truly_empty_preview_is_rejected() {
        let preview = preview_with_counts(SpreadsheetImportCounts::default());
        let error = ensure_preview_committable(&preview, "球队链")
            .expect_err("没有待写入或已导入记录时必须拒绝提交");
        assert!(error.to_string().contains("没有可写入或已完成记录"));
    }
}
