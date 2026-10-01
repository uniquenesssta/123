use super::identity::{
    apply_resolution, candidate_by_id, candidate_players_by_name, candidate_teams_by_name,
    decision_from_matches, resolve_player_reference, resolve_team_reference,
    validate_external_id_resolution,
};
use super::values::{
    default_text, normalize_spreadsheet_payload, optional_date, optional_datetime, optional_f64,
    optional_i16, optional_i32, optional_text, optional_uuid, required_date, required_datetime,
    required_f64, required_text, text,
};
use super::{MatchCandidate, ReferenceResolution, RowValidation, SpreadsheetValidationContext};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Duration, NaiveDate, SecondsFormat, Utc};
use football_domain::{
    SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportMode, SpreadsheetRowStatus,
};
use serde_json::{json, Map, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub(super) async fn validate_spreadsheet_row(
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

#[cfg(test)]
mod tests {
    use super::*;
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
