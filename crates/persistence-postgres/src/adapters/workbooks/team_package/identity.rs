use super::values::{normalize_name, normalize_team_type, object, object_mut, optional_uuid, text};
use super::RowValidation;
use crate::adapters::workbooks::batch_ledger::rows as ledger_rows;
use crate::{PersistenceError, PersistenceResult};
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
    let team_type =
        normalize_team_type(text(values, "team_type"))?.unwrap_or_else(|| "club".to_string());
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

fn merge_missing_team_payload_fields(target: &mut Map<String, Value>, source: &Map<String, Value>) {
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

pub(super) async fn consolidate_duplicate_ready_add_team_rows(
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

        for index in indices
            .iter()
            .copied()
            .filter(|index| *index != canonical_index)
        {
            let source = rows[index]
                .payload
                .as_object()
                .ok_or_else(|| PersistenceError::InvalidState("球队导入载荷不是对象".into()))?;
            merge_missing_team_payload_fields(&mut canonical_payload, source);
        }

        let canonical_payload = Value::Object(canonical_payload);
        rows[canonical_index].payload = canonical_payload.clone();
        rows[canonical_index].message = Some("同一资料包重复球队行已合并".into());
        ledger_rows::set_import_payload_in_tx(
            tx,
            rows[canonical_index].id,
            &canonical_payload,
            Some("同一资料包重复球队行已合并"),
        )
        .await?;

        for index in indices
            .iter()
            .copied()
            .filter(|index| *index != canonical_index)
        {
            rows[index].status = SpreadsheetRowStatus::Skip;
            rows[index].message = Some("同一资料包重复球队行已合并并跳过".into());
            ledger_rows::skip_duplicate_import_row_in_tx(
                tx,
                rows[index].id,
                "同一资料包重复球队行已合并并跳过",
            )
            .await?;
        }
    }
    Ok(())
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

fn team_ready_add_source_identity(row: &SpreadsheetImportRow) -> PersistenceResult<Option<String>> {
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
    let team_type =
        normalize_team_type(text(values, "team_type"))?.unwrap_or_else(|| "club".to_string());
    Ok(Some(format!(
        "source:{}|country:{country}|type:{team_type}",
        urls.join("|")
    )))
}

pub(super) async fn consolidate_duplicate_ready_add_team_rows_by_source(
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

        for index in indices
            .iter()
            .copied()
            .filter(|index| *index != canonical_index)
        {
            let source = rows[index]
                .payload
                .as_object()
                .ok_or_else(|| PersistenceError::InvalidState("球队导入载荷不是对象".into()))?;
            merge_missing_team_payload_fields(&mut canonical_payload, source);
        }

        let canonical_payload = Value::Object(canonical_payload);
        rows[canonical_index].payload = canonical_payload.clone();
        rows[canonical_index].message = Some("同一资料包同来源球队行已合并".into());
        ledger_rows::set_import_payload_in_tx(
            tx,
            rows[canonical_index].id,
            &canonical_payload,
            Some("同一资料包同来源球队行已合并"),
        )
        .await?;

        for index in indices
            .iter()
            .copied()
            .filter(|index| *index != canonical_index)
        {
            rows[index].status = SpreadsheetRowStatus::Skip;
            rows[index].message = Some("同一资料包同来源球队行已合并并跳过".into());
            ledger_rows::skip_duplicate_import_row_in_tx(
                tx,
                rows[index].id,
                "同一资料包同来源球队行已合并并跳过",
            )
            .await?;
        }
    }
    Ok(())
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

    #[test]
    fn package_same_name_keeps_country_and_team_type_distinct() {
        let base = ready_add_team_row(
            "球队总览",
            json!({"official_name":"United", "country_code":"BRA", "team_type":"club"}),
        );
        for payload in [
            json!({"official_name":"United", "country_code":"ARG", "team_type":"club"}),
            json!({"official_name":"United", "country_code":"BRA", "team_type":"national"}),
        ] {
            let other = ready_add_team_row("球员与评分", payload);
            assert_ne!(
                team_ready_add_identity(&base).unwrap(),
                team_ready_add_identity(&other).unwrap()
            );
        }
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
    fn package_merge_preserves_zero_false_and_existing_display_name() {
        let mut target = json!({"official_name":"中文球队", "is_active":false, "attack_rating":0, "city":"  ", "stadium":null}).as_object().unwrap().clone();
        let source = json!({"official_name":"Original Team", "is_active":true, "attack_rating":80, "city":"City", "stadium":"Arena"});
        merge_missing_team_payload_fields(&mut target, source.as_object().unwrap());
        assert_eq!(target["official_name"], "中文球队");
        assert_eq!(target["is_active"], false);
        assert_eq!(target["attack_rating"], 0);
        assert_eq!(target["city"], "City");
        assert_eq!(target["stadium"], "Arena");
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
