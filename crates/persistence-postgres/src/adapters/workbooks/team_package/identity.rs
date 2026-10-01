use super::super::identity::teams::normalized_source_urls;
use super::values::{normalize_name, object_mut, optional_uuid, text};
use super::RowValidation;
use crate::adapters::workbooks::batch_ledger::rows as ledger_rows;
use crate::PersistenceResult;
use football_domain::{
    SpreadsheetAction, SpreadsheetConflictCandidate, SpreadsheetEntityType, SpreadsheetImportMode,
    SpreadsheetImportRow, SpreadsheetRowStatus,
};
use serde_json::{json, Map, Value};
use sqlx::{Postgres, Row, Transaction};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

pub(super) fn validate_identity(
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

pub(super) fn canonical_team_import_action(
    action: SpreadsheetAction,
    entity_type: SpreadsheetEntityType,
    status: SpreadsheetRowStatus,
) -> SpreadsheetAction {
    if action != SpreadsheetAction::Upsert {
        return action;
    }
    if matches!(
        entity_type,
        SpreadsheetEntityType::Team | SpreadsheetEntityType::Coach
    ) && matches!(
        status,
        SpreadsheetRowStatus::ReadyUpdate | SpreadsheetRowStatus::Conflict
    ) {
        SpreadsheetAction::Update
    } else {
        SpreadsheetAction::Add
    }
}

fn insert_unique_reference(references: &mut HashMap<String, Option<Uuid>>, key: String, id: Uuid) {
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
    if let Some(id) =
        optional_uuid(values, "_resolved_team_id")?.or(optional_uuid(values, "team_id")?)
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

pub(super) async fn bind_batch_team_references(
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
        ledger_rows::set_import_payload_in_tx(tx, row.id, &row.payload, None).await?;
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
        ledger_rows::set_import_payload_in_tx(tx, row.id, &row.payload, None).await?;
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
        let Some(team_id) = resolve_batch_team_reference(values, &alias_index, &source_index)?
        else {
            continue;
        };
        values.insert("_resolved_team_id".into(), json!(team_id));
        ledger_rows::set_import_payload_in_tx(tx, row.id, &row.payload, None).await?;
    }
    Ok(())
}

pub(super) async fn find_team_candidates(
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

pub(super) async fn find_coach_candidates(
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

#[derive(Debug, Clone)]
struct BatchTeamReference {
    id: Uuid,
    aliases: HashSet<String>,
    source_urls: HashSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_team_reference_uses_unique_source_when_name_changed() {
        let team_id = Uuid::new_v4();
        let teams = vec![BatchTeamReference {
            id: team_id,
            aliases: HashSet::from([normalize_name("米内罗竞技")]),
            source_urls: HashSet::from([
                "https://atletico.com.br/futebol/masculino/elenco".to_string()
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
    fn package_ambiguous_alias_and_source_do_not_choose_a_team() {
        let ids = [Uuid::from_u128(1), Uuid::from_u128(2)];
        let teams: Vec<_> = ids
            .iter()
            .map(|id| BatchTeamReference {
                id: *id,
                aliases: HashSet::from([normalize_name("United")]),
                source_urls: HashSet::from(["https://example.test/shared".into()]),
            })
            .collect();
        let (aliases, sources) = build_batch_team_reference_indexes(&teams);
        let mut payload =
            json!({"team_name":"United", "source_urls":"https://example.test/shared/"});
        assert_eq!(
            resolve_batch_team_reference(payload.as_object().unwrap(), &aliases, &sources).unwrap(),
            None
        );
        payload["team_id"] = json!(ids[1]);
        assert_eq!(
            resolve_batch_team_reference(payload.as_object().unwrap(), &aliases, &sources).unwrap(),
            Some(ids[1])
        );
        payload["_resolved_team_id"] = json!(ids[0]);
        assert_eq!(
            resolve_batch_team_reference(payload.as_object().unwrap(), &aliases, &sources).unwrap(),
            Some(ids[0])
        );
    }

    #[test]
    fn package_identity_gates_zero_unique_and_ambiguous_candidates() {
        let candidate = SpreadsheetConflictCandidate {
            entity_id: Uuid::from_u128(1),
            display_name: "Team".into(),
            detail: None,
        };
        for action in [SpreadsheetAction::Add, SpreadsheetAction::Upsert] {
            assert_eq!(
                validate_identity(
                    action,
                    SpreadsheetImportMode::AddAndUpdate,
                    json!({}),
                    vec![],
                    "team"
                )
                .unwrap()
                .status,
                SpreadsheetRowStatus::ReadyAdd
            );
        }
        for action in [SpreadsheetAction::Update, SpreadsheetAction::Clear] {
            assert_eq!(
                validate_identity(
                    action,
                    SpreadsheetImportMode::AddAndUpdate,
                    json!({}),
                    vec![],
                    "team"
                )
                .unwrap()
                .status,
                SpreadsheetRowStatus::Error
            );
        }
        let skip = validate_identity(
            SpreadsheetAction::Upsert,
            SpreadsheetImportMode::AddOnly,
            json!({}),
            vec![candidate.clone()],
            "team",
        )
        .unwrap();
        assert_eq!(skip.status, SpreadsheetRowStatus::Skip);
        let unique = validate_identity(
            SpreadsheetAction::Upsert,
            SpreadsheetImportMode::AddAndUpdate,
            json!({}),
            vec![candidate.clone()],
            "team",
        )
        .unwrap();
        assert_eq!(unique.status, SpreadsheetRowStatus::ReadyUpdate);
        assert_eq!(
            unique.payload["_resolved_team_id"],
            json!(candidate.entity_id)
        );
        let second = SpreadsheetConflictCandidate {
            entity_id: Uuid::from_u128(2),
            display_name: "Other".into(),
            detail: None,
        };
        let conflict = validate_identity(
            SpreadsheetAction::Upsert,
            SpreadsheetImportMode::AddAndUpdate,
            json!({}),
            vec![candidate, second],
            "team",
        )
        .unwrap();
        assert_eq!(conflict.status, SpreadsheetRowStatus::Conflict);
        assert_eq!(conflict.payload["_conflict_prefix"], "team");
        assert_eq!(conflict.conflict_candidates.len(), 2);
        assert!(conflict.matched_entity_id.is_none());
    }
}
