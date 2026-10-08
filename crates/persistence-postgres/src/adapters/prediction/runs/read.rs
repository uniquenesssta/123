use crate::adapters::competition::model_run_identity::read_model_run_identity;
use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::Row;
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

    pub async fn read_run(&self, run_id: Uuid) -> PersistenceResult<Value> {
        let identity = read_model_run_identity(self, run_id).await?;
        let row = sqlx::query(
            r#"
            SELECT
                r.match_key, r.snapshot_type, r.route_reason,
                r.input_payload, r.output_payload, r.explanation, r.summary,
                r.input_sha256, r.input_audit_version, r.input_readiness_level,
                r.input_readiness_score, r.input_manifest, r.input_manifest_sha256,
                r.feature_snapshot_id,
                snapshot.snapshot_fingerprint AS feature_snapshot_fingerprint,
                r.duration_ms, r.created_at, r.completed_at
            FROM model.runs r
            LEFT JOIN feature.snapshots snapshot ON snapshot.id = r.feature_snapshot_id
            WHERE r.id = $1
            "#,
        )
        .bind(run_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(json!({
            "id": identity.id,
            "match_key": row.try_get::<String, _>("match_key")?,
            "snapshot_type": row.try_get::<String, _>("snapshot_type")?,
            "model_key": identity.model_key,
            "model_version": identity.model_version,
            "parameter_version": identity.parameter_version,
            "rule_package_id": identity.rule_package_id,
            "rule_package_key": identity.rule_package_key,
            "rule_package_version": identity.rule_package_version,
            "rule_package_name": identity.rule_package_name,
            "route_binding_id": identity.route_binding_id,
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
