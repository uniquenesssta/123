use super::super::batch_ledger::rows as ledger_rows;
use super::super::team_package::values::{
    normalize_name, normalize_team_type, object, optional_uuid, text,
};
use crate::{PersistenceError, PersistenceResult};
use football_domain::{SpreadsheetEntityType, SpreadsheetImportRow, SpreadsheetRowStatus};
use serde_json::{Map, Value};
use sqlx::{Postgres, Transaction};
use std::collections::HashMap;

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

pub(crate) async fn consolidate_duplicate_ready_add_team_rows(
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

pub(crate) fn normalized_source_urls(values: &Map<String, Value>) -> Vec<String> {
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

pub(crate) async fn consolidate_duplicate_ready_add_team_rows_by_source(
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

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::SpreadsheetAction;
    use serde_json::json;
    use uuid::Uuid;
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
}
