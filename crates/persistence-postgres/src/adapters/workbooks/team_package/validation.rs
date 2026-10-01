use super::formation::{
    canonical_formation_code, is_valid_custom_formation_code, normalize_formation_usage_payload,
};
use super::identity::{find_coach_candidates, find_team_candidates, validate_identity};
use super::values::{
    normalize_monthly_datetime_payload, normalize_name, normalize_point_observation_window_payload,
    normalize_team_type_payload, object, object_mut, optional_datetime, optional_uuid,
    parse_bool_default, parse_date, parse_f64_default, parse_i32, require_text, text,
};
use super::RowValidation;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportMode, SpreadsheetRowStatus,
};
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::HashSet;
use uuid::Uuid;

impl PostgresStore {
    pub(super) async fn validate_team_monthly_row(
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
            SpreadsheetEntityType::Coach => {
                self.validate_coach_row(raw.action, mode, payload).await
            }
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
            let window_start = parse_date(require_text(values, "window_start")?, "window_start")?;
            let window_end = parse_date(require_text(values, "window_end")?, "window_end")?;
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
                return Ok(RowValidation::error(payload, "阵型观察 alpha 必须大于 0"));
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
            "system_default" => RowValidation::ready_add(payload, "系统默认阵型分布已通过预检"),
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
            let row = sqlx::query("SELECT code,is_active FROM football.formations WHERE id=$1")
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
                    stale_id_note = Some(format!("阵型ID {formation_id} 在当前数据库不存在"));
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
        object_mut(payload)?.insert("formation_code".into(), Value::String(code.clone()));
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
