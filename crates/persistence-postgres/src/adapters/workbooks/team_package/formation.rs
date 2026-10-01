use super::values::{
    common_metadata, normalize_name, object, object_mut, optional_datetime, optional_uuid,
    parse_date, parse_f64_default, parse_i32, require_text, source_urls, text,
};
use super::write::resolve_entity_id_tx;
use crate::{write_audit_event, PersistenceError, PersistenceResult};
use chrono::Utc;
use football_domain::{SpreadsheetEntityType, SpreadsheetImportRow, SpreadsheetRowStatus};
use serde_json::{json, Map, Value};
use sqlx::{Postgres, Row, Transaction};
use std::collections::{BTreeMap, HashMap, HashSet};
use uuid::Uuid;

const TEAM_IMPORT_TYPE: &str = "team_monthly_xlsx";

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
            text(values, "formation_code").map(|value| format!("code:{}", normalize_name(&value)))
        })
        .unwrap_or_default()
}

pub(super) fn validate_formation_group_rows(rows: &mut [SpreadsheetImportRow]) {
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
            let observed =
                text(values, "observed_matches").and_then(|value| value.parse::<i32>().ok());
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
            if !formation_reference.is_empty() && !formation_references.insert(formation_reference)
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

pub(super) async fn execute_formation_groups(
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
        let active =
            sqlx::query_scalar::<_, bool>("SELECT is_active FROM football.formations WHERE id=$1")
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
        let active =
            sqlx::query_scalar::<_, bool>("SELECT is_active FROM football.formations WHERE id=$1")
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

    sqlx::query_scalar::<_, Uuid>("SELECT id FROM football.formations WHERE code=$1 AND is_active")
        .bind(&code)
        .fetch_optional(&mut **tx)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState(format!(
                "{row_label}的自定义阵型 {code} 未能登记到阵型目录"
            ))
        })
}

pub(super) fn canonical_formation_code(value: &str) -> String {
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

pub(super) fn is_valid_custom_formation_code(value: &str) -> bool {
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

pub(super) fn normalize_formation_usage_payload(payload: &mut Value) -> PersistenceResult<bool> {
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
const UNKNOWN_FORMATION_ID: Uuid = Uuid::from_u128(0x076720d204f05b3bad4787f0bfe290bd);

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::SpreadsheetAction;
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
