use super::values::{normalize_name, optional_date, optional_uuid, text};
use super::{MatchCandidate, ReferenceResolution, RowValidation};
use crate::{PersistenceResult, PostgresStore};
use chrono::NaiveDate;
use football_domain::{
    SpreadsheetAction, SpreadsheetConflictCandidate, SpreadsheetEntityType, SpreadsheetImportMode,
    SpreadsheetRowStatus,
};
use serde_json::{json, Map, Value};
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub(super) fn canonical_spreadsheet_import_action(
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

pub(super) fn decision_from_matches(
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

pub(super) async fn resolve_player_reference(
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

pub(super) fn normalize_reference_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) async fn resolve_team_reference(
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

pub(super) fn validate_external_id_resolution(
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

pub(super) fn apply_resolution(
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

pub(super) fn collect_keys(
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

pub(super) fn duplicate_keys(
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

pub(super) async fn candidate_by_id(
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

pub(super) async fn candidate_teams_by_name(
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

pub(super) async fn candidate_players_by_name(
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

#[cfg(test)]
mod tests {
    use super::*;
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

    fn candidate(id: Uuid, name: &str) -> MatchCandidate {
        MatchCandidate {
            id,
            name: name.into(),
            detail: Some("出生日期未知".into()),
        }
    }

    #[test]
    fn canonical_action_preserves_child_add_and_explicit_operations() {
        for entity in [
            SpreadsheetEntityType::Player,
            SpreadsheetEntityType::Team,
            SpreadsheetEntityType::ExternalEntityId,
        ] {
            for status in [
                SpreadsheetRowStatus::ReadyUpdate,
                SpreadsheetRowStatus::Conflict,
            ] {
                assert_eq!(
                    canonical_spreadsheet_import_action(SpreadsheetAction::Upsert, entity, status),
                    SpreadsheetAction::Update
                );
            }
            assert_eq!(
                canonical_spreadsheet_import_action(
                    SpreadsheetAction::Upsert,
                    entity,
                    SpreadsheetRowStatus::ReadyAdd
                ),
                SpreadsheetAction::Add
            );
        }
        for entity in [
            SpreadsheetEntityType::PlayerName,
            SpreadsheetEntityType::PlayerPosition,
            SpreadsheetEntityType::PlayerTeamPeriod,
            SpreadsheetEntityType::PlayerAbility,
            SpreadsheetEntityType::PlayerAvailability,
            SpreadsheetEntityType::PlayerDynamicTag,
        ] {
            assert_eq!(
                canonical_spreadsheet_import_action(
                    SpreadsheetAction::Upsert,
                    entity,
                    SpreadsheetRowStatus::ReadyUpdate
                ),
                SpreadsheetAction::Add
            );
            for action in [
                SpreadsheetAction::Add,
                SpreadsheetAction::Update,
                SpreadsheetAction::Clear,
                SpreadsheetAction::Skip,
            ] {
                assert_eq!(
                    canonical_spreadsheet_import_action(
                        action,
                        entity,
                        SpreadsheetRowStatus::Conflict
                    ),
                    action
                );
            }
        }
    }

    #[test]
    fn matching_requires_selection_for_names_and_keeps_mode_and_action_rules() {
        let ids = [Uuid::new_v4(), Uuid::new_v4()];
        let conflict = decision_from_matches(
            SpreadsheetAction::Upsert,
            SpreadsheetImportMode::AddAndUpdate,
            json!({"player_key":"P1"}),
            vec![candidate(ids[0], "同名"), candidate(ids[1], "同名")],
            "player",
        )
        .unwrap();
        assert_eq!(conflict.status, SpreadsheetRowStatus::Conflict);
        assert_eq!(conflict.matched_entity_id, None);
        assert_eq!(conflict.payload["_conflict_prefix"], "player");
        assert_eq!(
            conflict
                .conflict_candidates
                .iter()
                .map(|v| v.entity_id)
                .collect::<Vec<_>>(),
            ids
        );
        assert_eq!(
            conflict.conflict_candidates[0].detail.as_deref(),
            Some("出生日期未知")
        );
        for (action, mode, status) in [
            (
                SpreadsheetAction::Add,
                SpreadsheetImportMode::AddAndUpdate,
                SpreadsheetRowStatus::Skip,
            ),
            (
                SpreadsheetAction::Upsert,
                SpreadsheetImportMode::AddOnly,
                SpreadsheetRowStatus::Skip,
            ),
            (
                SpreadsheetAction::Update,
                SpreadsheetImportMode::AddAndUpdate,
                SpreadsheetRowStatus::ReadyUpdate,
            ),
        ] {
            let result = decision_from_matches(
                action,
                mode,
                json!({}),
                vec![candidate(ids[0], "球员")],
                "player",
            )
            .unwrap();
            assert_eq!(result.status, status);
            assert_eq!(result.matched_entity_id, Some(ids[0]));
        }
        for action in [SpreadsheetAction::Update, SpreadsheetAction::Clear] {
            assert_eq!(
                decision_from_matches(
                    action,
                    SpreadsheetImportMode::AddAndUpdate,
                    json!({}),
                    vec![],
                    "player"
                )
                .unwrap()
                .status,
                SpreadsheetRowStatus::Error
            );
        }
    }

    #[test]
    fn external_id_never_rebinds_existing_or_deferred_identity() {
        let existing = Uuid::new_v4();
        let other = Uuid::new_v4();
        for resolution in [
            ReferenceResolution::Resolved(other),
            ReferenceResolution::Deferred("P1".into()),
            ReferenceResolution::DeferredExternal {
                key: "ATM".into(),
                name: "球队".into(),
            },
        ] {
            let mut payload = json!({"external_id":"source-1"})
                .as_object()
                .unwrap()
                .clone();
            let initial = payload.clone();
            let row = validate_external_id_resolution(
                &mut payload,
                resolution,
                "player",
                Some(existing),
                SpreadsheetAction::Upsert,
                SpreadsheetImportMode::AddAndUpdate,
            )
            .unwrap();
            assert_eq!(row.status, SpreadsheetRowStatus::Error);
            assert_eq!(payload, initial);
        }
    }

    #[test]
    fn external_id_same_target_and_new_target_keep_update_and_skip_semantics() {
        let id = Uuid::new_v4();
        for (action, mode, status) in [
            (
                SpreadsheetAction::Add,
                SpreadsheetImportMode::AddAndUpdate,
                SpreadsheetRowStatus::Skip,
            ),
            (
                SpreadsheetAction::Upsert,
                SpreadsheetImportMode::AddOnly,
                SpreadsheetRowStatus::Skip,
            ),
            (
                SpreadsheetAction::Update,
                SpreadsheetImportMode::AddAndUpdate,
                SpreadsheetRowStatus::ReadyUpdate,
            ),
        ] {
            let mut payload = Map::new();
            let row = validate_external_id_resolution(
                &mut payload,
                ReferenceResolution::Resolved(id),
                "player",
                Some(id),
                action,
                mode,
            )
            .unwrap();
            assert_eq!(row.status, status);
            assert_eq!(row.matched_entity_id, Some(id));
            assert_eq!(payload["_resolved_player_id"], json!(id));
        }
        for (action, status) in [
            (SpreadsheetAction::Update, SpreadsheetRowStatus::Error),
            (SpreadsheetAction::Add, SpreadsheetRowStatus::ReadyAdd),
        ] {
            let row = validate_external_id_resolution(
                &mut Map::new(),
                ReferenceResolution::Resolved(id),
                "player",
                None,
                action,
                SpreadsheetImportMode::AddAndUpdate,
            )
            .unwrap();
            assert_eq!(row.status, status);
        }
    }

    #[test]
    fn child_deferred_reference_keeps_team_key_name_and_other_fields() {
        let mut payload = json!({"player_key":"P1","valid_from":"2026-01-01"})
            .as_object()
            .unwrap()
            .clone();
        let row = apply_resolution(
            &mut payload,
            ReferenceResolution::DeferredExternal {
                key: "ATM".into(),
                name: "球队".into(),
            },
            "team",
        )
        .unwrap();
        assert_eq!(row.status, SpreadsheetRowStatus::ReadyAdd);
        assert_eq!(row.matched_entity_id, None);
        assert_eq!(payload["_deferred_team_key"], "ATM");
        assert_eq!(payload["_deferred_team_name"], "球队");
        assert_eq!(payload["player_key"], "P1");
        assert_eq!(payload["valid_from"], "2026-01-01");
        let id = Uuid::new_v4();
        let row =
            apply_resolution(&mut payload, ReferenceResolution::Resolved(id), "player").unwrap();
        assert_eq!(row.matched_entity_id, Some(id));
        assert_eq!(payload["_resolved_player_id"], json!(id));
        assert_eq!(payload["_deferred_team_key"], "ATM");
    }
}
