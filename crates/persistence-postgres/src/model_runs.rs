use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::RouteDecision;
use football_model_api::{ModelOutput, ModelRequest};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRunListItem {
    pub id: Uuid,
    pub match_key: String,
    pub competition_name: Option<String>,
    pub home_team_name: Option<String>,
    pub away_team_name: Option<String>,
    pub kickoff_time: Option<DateTime<Utc>>,
    pub snapshot_type: String,
    pub model_key: String,
    pub model_version: String,
    pub parameter_version: String,
    pub rule_package_name: Option<String>,
    pub summary: Value,
    pub top_scoreline: Option<String>,
    pub top_scoreline_probability: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub input_readiness_level: String,
    pub input_readiness_score: Option<i16>,
    pub input_manifest_sha256: String,
}

impl PostgresStore {
    pub async fn save_successful_run(
        &self,
        decision: &RouteDecision,
        request: &ModelRequest,
        output: &ModelOutput,
        duration_ms: i64,
    ) -> PersistenceResult<Uuid> {
        let run_id = Uuid::new_v4();
        let input_hash = sha256_json(&request.input)?;
        let input_audit = prepared_run_input_audit(&request.input)?;
        let summary = serde_json::to_value(&output.summary)?;
        let database_match_id = optional_uuid(&request.input, "database_match_id")?;
        let feature_snapshot = prepared_feature_snapshot(&request.input, &request.snapshot_type)?;
        let mut feature_snapshot_id = feature_snapshot.as_ref().map(|snapshot| snapshot.id);
        if feature_snapshot.is_some() && database_match_id.is_none() {
            return Err(PersistenceError::InvalidState(
                "特征快照必须关联数据库比赛编号".to_string(),
            ));
        }
        let mut tx = self.pool.begin().await?;

        if let (Some(match_id), Some(snapshot)) = (database_match_id, feature_snapshot.as_ref()) {
            let inserted: Option<Uuid> = sqlx::query_scalar(
                r#"
                INSERT INTO feature.snapshots (
                    id, match_id, match_key, snapshot_type, data_cutoff_time,
                    frozen_at, schema_version, quality_score, input_payload, input_sha256,
                    snapshot_fingerprint, payload_sha256, source_kind, evidence_scope
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'runtime', 'none')
                ON CONFLICT DO NOTHING
                RETURNING id
                "#,
            )
            .bind(snapshot.id)
            .bind(match_id)
            .bind(&request.context.match_key)
            .bind(&request.snapshot_type)
            .bind(snapshot.data_cutoff_time)
            .bind(snapshot.frozen_at)
            .bind(&snapshot.schema_version)
            .bind(snapshot.quality_score)
            .bind(&request.input)
            .bind(&input_hash)
            .bind(&input_audit.manifest_sha256)
            .bind(&input_hash)
            .fetch_optional(&mut *tx)
            .await?;
            let persisted_id = if let Some(id) = inserted {
                id
            } else {
                sqlx::query_scalar(
                    r#"
                    SELECT id
                    FROM feature.snapshots
                    WHERE id = $1
                       OR (
                           match_key = $2 AND snapshot_type = $3 AND input_sha256 = $4
                           AND source_kind IN ('legacy', 'runtime')
                       )
                    ORDER BY CASE WHEN id = $1 THEN 0 ELSE 1 END
                    LIMIT 1
                    "#,
                )
                .bind(snapshot.id)
                .bind(&request.context.match_key)
                .bind(&request.snapshot_type)
                .bind(&input_hash)
                .fetch_one(&mut *tx)
                .await?
            };
            feature_snapshot_id = Some(persisted_id);
        }

        sqlx::query(
            r#"
            INSERT INTO model.runs (
                id, match_id, match_key, feature_snapshot_id,
                model_version_id, parameter_set_id,
                rule_package_id, route_binding_id, snapshot_type,
                route_reason, status, input_payload, output_payload, explanation,
                summary, input_sha256, duration_ms, completed_at,
                input_audit_version, input_readiness_level, input_readiness_score,
                input_manifest, input_manifest_sha256
            ) VALUES (
                $1, $2, $3, $4,
                $5, $6,
                $7, $8, $9,
                $10, 'succeeded', $11, $12, $13,
                $14, $15, $16, now(),
                $17, $18, $19, $20, $21
            )
            "#,
        )
        .bind(run_id)
        .bind(database_match_id)
        .bind(&request.context.match_key)
        .bind(feature_snapshot_id)
        .bind(decision.model_version_id)
        .bind(decision.parameter_set_id)
        .bind(decision.rule_package_id)
        .bind(decision.binding_id)
        .bind(&request.snapshot_type)
        .bind(&decision.reason)
        .bind(&request.input)
        .bind(&output.payload)
        .bind(&output.explanation)
        .bind(summary)
        .bind(&input_hash)
        .bind(duration_ms)
        .bind(&input_audit.audit_version)
        .bind(&input_audit.readiness_level)
        .bind(input_audit.readiness_score)
        .bind(&input_audit.manifest)
        .bind(&input_audit.manifest_sha256)
        .execute(&mut *tx)
        .await?;

        save_model_details(&mut tx, run_id, &output.payload).await?;
        write_audit_event(
            &mut tx,
            "model_run_completed",
            "model_run",
            Some(run_id.to_string()),
            json!({
                "match_key": &request.context.match_key,
                "model": &request.identity,
                "rule_package_id": decision.rule_package_id,
                "binding_id": decision.binding_id,
                "duration_ms": duration_ms,
                "input_readiness_level": &input_audit.readiness_level,
                "input_readiness_score": input_audit.readiness_score,
                "input_manifest_sha256": &input_audit.manifest_sha256,
                "input_sha256": &input_hash,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(run_id)
    }

    pub async fn list_recent_runs(&self, limit: i64) -> PersistenceResult<Vec<ModelRunListItem>> {
        let rows = sqlx::query(
            r#"
            SELECT
                r.id, r.match_key, r.snapshot_type,
                competition.name AS competition_name,
                COALESCE(home.canonical_name, r.input_payload #>> '{team_a,name}') AS home_team_name,
                COALESCE(away.canonical_name, r.input_payload #>> '{team_b,name}') AS away_team_name,
                COALESCE(fixture.kickoff_time, NULLIF(r.input_payload ->> 'kickoff_time', '')::timestamptz) AS kickoff_time,
                d.model_key, v.version AS model_version,
                p.parameter_version, rp.display_name AS rule_package_name,
                r.summary,
                CASE
                    WHEN top_score.home_goals IS NULL THEN NULL
                    ELSE top_score.home_goals::text || '-' || top_score.away_goals::text
                END AS top_scoreline,
                top_score.probability AS top_scoreline_probability,
                r.created_at, r.completed_at, r.duration_ms,
                r.input_readiness_level, r.input_readiness_score,
                r.input_manifest_sha256
            FROM model.runs r
            JOIN model.versions v ON v.id = r.model_version_id
            JOIN model.definitions d ON d.id = v.model_id
            JOIN model.parameter_sets p ON p.id = r.parameter_set_id
            LEFT JOIN model.rule_packages rp ON rp.id = r.rule_package_id
            LEFT JOIN football.matches fixture ON fixture.external_key = r.match_key
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            LEFT JOIN football.teams home ON home.id = fixture.home_team_id
            LEFT JOIN football.teams away ON away.id = fixture.away_team_id
            LEFT JOIN LATERAL (
                SELECT home_goals, away_goals, probability
                FROM model.run_scorelines scoreline
                WHERE scoreline.run_id = r.id
                ORDER BY scoreline.rank ASC, scoreline.probability DESC
                LIMIT 1
            ) top_score ON true
            WHERE r.status = 'succeeded'
              AND r.history_hidden_at IS NULL
            ORDER BY r.created_at DESC, r.id DESC
            LIMIT $1
            "#,
        )
        .bind(limit.clamp(1, 500))
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|row| {
                Ok(ModelRunListItem {
                    id: row.try_get("id")?,
                    match_key: row.try_get("match_key")?,
                    competition_name: row.try_get("competition_name")?,
                    home_team_name: row.try_get("home_team_name")?,
                    away_team_name: row.try_get("away_team_name")?,
                    kickoff_time: row.try_get("kickoff_time")?,
                    snapshot_type: row.try_get("snapshot_type")?,
                    model_key: row.try_get("model_key")?,
                    model_version: row.try_get("model_version")?,
                    parameter_version: row.try_get("parameter_version")?,
                    rule_package_name: row.try_get("rule_package_name")?,
                    summary: row.try_get("summary")?,
                    top_scoreline: row.try_get("top_scoreline")?,
                    top_scoreline_probability: row.try_get("top_scoreline_probability")?,
                    created_at: row.try_get("created_at")?,
                    completed_at: row.try_get("completed_at")?,
                    duration_ms: row.try_get("duration_ms")?,
                    input_readiness_level: row.try_get("input_readiness_level")?,
                    input_readiness_score: row.try_get("input_readiness_score")?,
                    input_manifest_sha256: row.try_get("input_manifest_sha256")?,
                })
            })
            .collect()
    }

    pub async fn hide_run_from_history(
        &self,
        run_id: Uuid,
        reason: Option<&str>,
    ) -> PersistenceResult<()> {
        let reason = reason
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("用户从推演历史列表中删除");
        let mut tx = self.pool.begin().await?;
        let match_key = sqlx::query_scalar::<_, String>(
            r#"
            UPDATE model.runs
            SET history_hidden_at = COALESCE(history_hidden_at, now()),
                history_hidden_reason = $2
            WHERE id = $1 AND status = 'succeeded'
            RETURNING match_key
            "#,
        )
        .bind(run_id)
        .bind(reason)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("推演记录不存在或尚未成功完成".to_string()))?;
        write_audit_event(
            &mut tx,
            "model_run_history_hidden",
            "model_run",
            Some(run_id.to_string()),
            json!({"match_key": match_key, "reason": reason}),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn read_run(&self, run_id: Uuid) -> PersistenceResult<Value> {
        let row = sqlx::query(
            r#"
            SELECT
                r.id, r.match_key, r.snapshot_type, r.route_reason,
                r.input_payload, r.output_payload, r.explanation, r.summary,
                r.input_sha256, r.input_audit_version, r.input_readiness_level,
                r.input_readiness_score, r.input_manifest, r.input_manifest_sha256,
                r.feature_snapshot_id,
                snapshot.snapshot_fingerprint AS feature_snapshot_fingerprint,
                r.duration_ms, r.created_at, r.completed_at,
                d.model_key, v.version AS model_version, p.parameter_version,
                rp.id AS rule_package_id, rp.package_key,
                rp.version AS rule_package_version, rp.display_name AS rule_package_name,
                r.route_binding_id
            FROM model.runs r
            JOIN model.versions v ON v.id = r.model_version_id
            JOIN model.definitions d ON d.id = v.model_id
            JOIN model.parameter_sets p ON p.id = r.parameter_set_id
            LEFT JOIN model.rule_packages rp ON rp.id = r.rule_package_id
            LEFT JOIN feature.snapshots snapshot ON snapshot.id = r.feature_snapshot_id
            WHERE r.id = $1
            "#,
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(json!({
            "id": row.try_get::<Uuid, _>("id")?,
            "match_key": row.try_get::<String, _>("match_key")?,
            "snapshot_type": row.try_get::<String, _>("snapshot_type")?,
            "model_key": row.try_get::<String, _>("model_key")?,
            "model_version": row.try_get::<String, _>("model_version")?,
            "parameter_version": row.try_get::<String, _>("parameter_version")?,
            "rule_package_id": row.try_get::<Option<Uuid>, _>("rule_package_id")?,
            "rule_package_key": row.try_get::<Option<String>, _>("package_key")?,
            "rule_package_version": row.try_get::<Option<String>, _>("rule_package_version")?,
            "rule_package_name": row.try_get::<Option<String>, _>("rule_package_name")?,
            "route_binding_id": row.try_get::<Option<Uuid>, _>("route_binding_id")?,
            "route_reason": row.try_get::<Value, _>("route_reason")?,
            "input_sha256": row.try_get::<String, _>("input_sha256")?,
            "input_audit": {
                "audit_version": row.try_get::<String, _>("input_audit_version")?,
                "readiness_level": row.try_get::<String, _>("input_readiness_level")?,
                "readiness_score": row.try_get::<Option<i16>, _>("input_readiness_score")?,
                "manifest": row.try_get::<Value, _>("input_manifest")?,
                "manifest_sha256": row.try_get::<String, _>("input_manifest_sha256")?,
                "feature_snapshot_id": row.try_get::<Option<Uuid>, _>("feature_snapshot_id")?,
                "feature_snapshot_fingerprint": row.try_get::<Option<String>, _>("feature_snapshot_fingerprint")?,
            },
            "input": row.try_get::<Value, _>("input_payload")?,
            "output": row.try_get::<Option<Value>, _>("output_payload")?,
            "explanation": row.try_get::<Option<Value>, _>("explanation")?,
            "summary": row.try_get::<Option<Value>, _>("summary")?,
            "duration_ms": row.try_get::<Option<i64>, _>("duration_ms")?,
            "created_at": row.try_get::<DateTime<Utc>, _>("created_at")?,
            "completed_at": row.try_get::<Option<DateTime<Utc>>, _>("completed_at")?,
        }))
    }
}

async fn save_model_details(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    run_id: Uuid,
    payload: &Value,
) -> PersistenceResult<()> {
    if let Some(modules) = payload.get("modules").and_then(Value::as_object) {
        for (module_key, details) in modules {
            let side = module_key
                .split_once('_')
                .map(|(side_value, _)| side_value.to_string());
            sqlx::query(
                r#"
                INSERT INTO model.run_modules (
                    run_id, module_key, side, raw_score, confidence,
                    effective_score, multiplier, details
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(run_id)
            .bind(module_key)
            .bind(side)
            .bind(details.get("raw_score").and_then(Value::as_f64))
            .bind(details.get("confidence").and_then(Value::as_f64))
            .bind(details.get("effective_score").and_then(Value::as_f64))
            .bind(details.get("multiplier").and_then(Value::as_f64))
            .bind(details)
            .execute(&mut **tx)
            .await?;
        }
    }

    if let Some(scorelines) = payload.get("scorelines").and_then(Value::as_array) {
        for item in scorelines {
            let home_goals = i16::try_from(required_i64(item, "goals_a")?).map_err(|_| {
                PersistenceError::InvalidState("goals_a 超出 smallint 范围".to_string())
            })?;
            let away_goals = i16::try_from(required_i64(item, "goals_b")?).map_err(|_| {
                PersistenceError::InvalidState("goals_b 超出 smallint 范围".to_string())
            })?;
            let rank = i16::try_from(required_i64(item, "rank")?).map_err(|_| {
                PersistenceError::InvalidState("rank 超出 smallint 范围".to_string())
            })?;
            let probability = required_f64(item, "probability")?;
            let cumulative_probability = required_f64(item, "cumulative_probability")?;
            let route = item
                .get("route")
                .and_then(Value::as_str)
                .unwrap_or("未分类");

            sqlx::query(
                r#"
                INSERT INTO model.run_scorelines (
                    run_id, home_goals, away_goals, probability,
                    rank, cumulative_probability, route, details
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(run_id)
            .bind(home_goals)
            .bind(away_goals)
            .bind(probability)
            .bind(rank)
            .bind(cumulative_probability)
            .bind(route)
            .bind(item)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

fn required_i64(value: &Value, key: &str) -> PersistenceResult<i64> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少整数：{key}")))
}

fn required_f64(value: &Value, key: &str) -> PersistenceResult<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少数值：{key}")))
}

#[derive(Debug)]
struct PreparedRunInputAudit {
    audit_version: String,
    readiness_level: String,
    readiness_score: Option<i16>,
    manifest: Value,
    manifest_sha256: String,
}

fn prepared_run_input_audit(input: &Value) -> PersistenceResult<PreparedRunInputAudit> {
    let Some(audit) = input.get("input_audit") else {
        let manifest = json!({
            "audit_version": "runtime-input-audit-v0",
            "match_key": input.get("match_id"),
            "database_match_id": input.get("database_match_id"),
            "snapshot": input.get("snapshot"),
            "preparation_version": input.get("preparation_version"),
        });
        return Ok(PreparedRunInputAudit {
            audit_version: "runtime-input-audit-v0".to_string(),
            readiness_level: "not_assessed".to_string(),
            readiness_score: None,
            manifest_sha256: sha256_json(&manifest)?,
            manifest,
        });
    };
    let audit_version = audit
        .get("audit_version")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PersistenceError::InvalidState("input_audit.audit_version 不能为空".to_string())
        })?
        .to_string();
    let readiness = audit.get("readiness").ok_or_else(|| {
        PersistenceError::InvalidState("input_audit.readiness 不能为空".to_string())
    })?;
    let readiness_level = readiness
        .get("level")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PersistenceError::InvalidState("input_audit.readiness.level 不能为空".to_string())
        })?
        .to_string();
    if !matches!(
        readiness_level.as_str(),
        "formal_ready" | "ready_with_warnings" | "shadow_only" | "blocked"
    ) {
        return Err(PersistenceError::InvalidState(format!(
            "未知赛前输入完整度状态：{readiness_level}"
        )));
    }
    let readiness_score = readiness
        .get("score")
        .and_then(Value::as_u64)
        .map(|value| {
            i16::try_from(value).map_err(|_| {
                PersistenceError::InvalidState("input_audit.readiness.score 超出范围".to_string())
            })
        })
        .transpose()?;
    if readiness_score.is_some_and(|value| !(0..=100).contains(&value)) {
        return Err(PersistenceError::InvalidState(
            "input_audit.readiness.score 必须在 0..=100 范围内".to_string(),
        ));
    }
    let manifest = audit
        .get("manifest")
        .cloned()
        .ok_or_else(|| PersistenceError::InvalidState("input_audit.manifest 不能为空".to_string()))?;
    let manifest_sha256 = audit
        .get("manifest_sha256")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            PersistenceError::InvalidState("input_audit.manifest_sha256 不能为空".to_string())
        })?
        .to_string();
    let calculated = sha256_json(&manifest)?;
    if calculated != manifest_sha256 {
        return Err(PersistenceError::InvalidState(
            "赛前输入清单 SHA256 与实际清单不一致".to_string(),
        ));
    }
    Ok(PreparedRunInputAudit {
        audit_version,
        readiness_level,
        readiness_score,
        manifest,
        manifest_sha256,
    })
}

#[derive(Debug)]
struct PreparedFeatureSnapshot {
    id: Uuid,
    data_cutoff_time: DateTime<Utc>,
    frozen_at: DateTime<Utc>,
    schema_version: String,
    quality_score: f64,
}

fn prepared_feature_snapshot(
    input: &Value,
    expected_snapshot_type: &str,
) -> PersistenceResult<Option<PreparedFeatureSnapshot>> {
    let Some(id) = optional_uuid(input, "feature_snapshot_id")? else {
        return Ok(None);
    };
    let snapshot = input
        .get("snapshot")
        .and_then(Value::as_object)
        .ok_or_else(|| PersistenceError::InvalidState("缺少特征快照元数据".to_string()))?;
    let snapshot_id = snapshot
        .get("snapshot_id")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            PersistenceError::InvalidState("snapshot.snapshot_id 必须是 UUID 字符串".to_string())
        })?;
    let snapshot_id = Uuid::parse_str(snapshot_id).map_err(|error| {
        PersistenceError::InvalidState(format!("snapshot.snapshot_id 不是有效 UUID：{error}"))
    })?;
    if snapshot_id != id {
        return Err(PersistenceError::InvalidState(
            "feature_snapshot_id 与 snapshot.snapshot_id 不一致".to_string(),
        ));
    }
    let snapshot_type = snapshot
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| PersistenceError::InvalidState("snapshot.type 不能为空".to_string()))?;
    if snapshot_type != expected_snapshot_type {
        return Err(PersistenceError::InvalidState(format!(
            "snapshot.type 与运行快照类型不一致：{snapshot_type} != {expected_snapshot_type}"
        )));
    }
    let data_cutoff_time = required_datetime(
        snapshot.get("data_cutoff_time"),
        "snapshot.data_cutoff_time",
    )?;
    let frozen_at = required_datetime(snapshot.get("frozen_at"), "snapshot.frozen_at")?;
    let schema_version = input
        .get("preparation_version")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| PersistenceError::InvalidState("preparation_version 不能为空".to_string()))?
        .to_string();
    let quality_score = input
        .get("feature_quality_score")
        .and_then(Value::as_f64)
        .ok_or_else(|| {
            PersistenceError::InvalidState("feature_quality_score 必须是数值".to_string())
        })?;
    if !quality_score.is_finite() || !(0.0..=1.0).contains(&quality_score) {
        return Err(PersistenceError::InvalidState(
            "feature_quality_score 必须在 0..=1 范围内".to_string(),
        ));
    }
    Ok(Some(PreparedFeatureSnapshot {
        id,
        data_cutoff_time,
        frozen_at,
        schema_version,
        quality_score,
    }))
}

fn required_datetime(value: Option<&Value>, key: &str) -> PersistenceResult<DateTime<Utc>> {
    let raw = value
        .and_then(Value::as_str)
        .ok_or_else(|| PersistenceError::InvalidState(format!("{key} 必须是 RFC3339 时间")))?;
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| PersistenceError::InvalidState(format!("{key} 时间无效：{error}")))
}

fn optional_uuid(value: &Value, key: &str) -> PersistenceResult<Option<Uuid>> {
    let Some(raw) = value.get(key) else {
        return Ok(None);
    };
    if raw.is_null() {
        return Ok(None);
    }
    let raw = raw
        .as_str()
        .ok_or_else(|| PersistenceError::InvalidState(format!("{key} 必须是 UUID 字符串")))?
        .trim();
    if raw.is_empty() {
        return Ok(None);
    }
    Uuid::parse_str(raw)
        .map(Some)
        .map_err(|error| PersistenceError::InvalidState(format!("{key} 不是有效 UUID：{error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optional_uuid_accepts_missing_null_and_blank_values() {
        assert_eq!(optional_uuid(&json!({}), "id").unwrap(), None);
        assert_eq!(optional_uuid(&json!({"id": null}), "id").unwrap(), None);
        assert_eq!(optional_uuid(&json!({"id": "  "}), "id").unwrap(), None);
    }

    #[test]
    fn prepared_input_audit_validates_manifest_hash() {
        let manifest = json!({"match_key": "MATCH-1", "snapshot_type": "T-1h"});
        let sha = sha256_json(&manifest).unwrap();
        let input = json!({
            "input_audit": {
                "audit_version": "prematch-input-audit-v1",
                "readiness": {"level": "formal_ready", "score": 96},
                "manifest": manifest,
                "manifest_sha256": sha
            }
        });
        let prepared = prepared_run_input_audit(&input).unwrap();
        assert_eq!(prepared.readiness_level, "formal_ready");
        assert_eq!(prepared.readiness_score, Some(96));

        let mut modified = input;
        modified["input_audit"]["manifest"]["snapshot_type"] = json!("T-6h");
        assert!(prepared_run_input_audit(&modified).is_err());
    }
}
