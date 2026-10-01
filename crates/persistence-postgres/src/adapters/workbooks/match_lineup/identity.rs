use super::{values::*, Validation};
use crate::{PersistenceError, PersistenceResult};
use chrono::NaiveDate;
use football_domain::{
    SpreadsheetAction, SpreadsheetConflictCandidate, SpreadsheetImportMode, SpreadsheetRowStatus,
};
use serde_json::{json, Map, Value};
use sqlx::Row;
use uuid::Uuid;

pub(super) async fn resolve_competition(
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

pub(super) async fn resolve_team_value(
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

pub(super) async fn resolve_player_value(
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

pub(super) fn resolution_validation(
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

pub(super) async fn resolve_match_reference(
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

pub(super) async fn find_match(
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

pub(super) fn decision(
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

pub(super) fn ready(payload: Value, message: &str) -> Validation {
    Validation {
        status: SpreadsheetRowStatus::ReadyAdd,
        message: Some(message.to_string()),
        payload,
        matched_entity_id: None,
        candidates: vec![],
    }
}

pub(super) fn skip(payload: Value, message: impl Into<String>) -> Validation {
    Validation {
        status: SpreadsheetRowStatus::Skip,
        message: Some(message.into()),
        payload,
        matched_entity_id: None,
        candidates: vec![],
    }
}

pub(super) fn error(payload: Value, message: impl Into<String>) -> Validation {
    Validation {
        status: SpreadsheetRowStatus::Error,
        message: Some(message.into()),
        payload,
        matched_entity_id: None,
        candidates: vec![],
    }
}

pub(super) async fn validate_lineup_match_team(
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

pub(super) async fn resolve_lineup_formation(
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

pub(super) async fn candidate_by_id(
    pool: &sqlx::PgPool,
    table: &str,
    id: Uuid,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    let query = format!("SELECT id,canonical_name FROM {table} WHERE id=$1");
    let rows = sqlx::query(&query).bind(id).fetch_all(pool).await?;
    rows.iter().map(candidate_row).collect()
}

pub(super) async fn candidate_by_match_id(
    pool: &sqlx::PgPool,
    id: Uuid,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    let rows=sqlx::query("SELECT match.id,home.canonical_name||' vs '||away.canonical_name AS name,match.kickoff_time::text AS detail FROM football.matches match JOIN football.teams home ON home.id=match.home_team_id JOIN football.teams away ON away.id=match.away_team_id WHERE match.id=$1").bind(id).fetch_all(pool).await?;
    rows.iter().map(candidate_row).collect()
}

pub(super) async fn candidate_teams(
    pool: &sqlx::PgPool,
    name: &str,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    let rows=sqlx::query("SELECT id,canonical_name AS name,country_code AS detail FROM football.teams WHERE normalized_name=$1 OR EXISTS(SELECT 1 FROM football.team_names n WHERE n.team_id=football.teams.id AND n.normalized_name=$1) ORDER BY canonical_name LIMIT 10").bind(normalize(name)).fetch_all(pool).await?;
    rows.iter().map(candidate_row).collect()
}

pub(super) async fn candidate_players(
    pool: &sqlx::PgPool,
    name: &str,
    birth: Option<NaiveDate>,
) -> PersistenceResult<Vec<SpreadsheetConflictCandidate>> {
    let rows=sqlx::query("SELECT id,canonical_name AS name,COALESCE(date_of_birth::text,'出生日期未知') AS detail FROM football.players WHERE (normalized_name=$1 OR EXISTS(SELECT 1 FROM football.player_names n WHERE n.player_id=football.players.id AND n.normalized_name=$1)) AND ($2::date IS NULL OR date_of_birth=$2) ORDER BY canonical_name LIMIT 10").bind(normalize(name)).bind(birth).fetch_all(pool).await?;
    rows.iter().map(candidate_row).collect()
}

pub(super) fn candidate_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<SpreadsheetConflictCandidate> {
    Ok(SpreadsheetConflictCandidate {
        entity_id: row.try_get("id")?,
        display_name: row
            .try_get("name")
            .or_else(|_| row.try_get("canonical_name"))?,
        detail: row.try_get("detail").ok(),
    })
}
