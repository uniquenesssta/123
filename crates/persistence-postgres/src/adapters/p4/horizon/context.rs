use super::row::parse_horizon;
use crate::{parse_competition_kind, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    P4PlanningMatchContext, ResearchRunRecord, ResearchRunStatus, SchemaVersionRecord,
};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn p4_planning_match_context(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<P4PlanningMatchContext> {
        let row = sqlx::query(
            r#"
            SELECT fixture.id, fixture.external_key, fixture.kickoff_time,
                   fixture.competition_id, fixture.season_id, fixture.stage_id,
                   COALESCE(stage.stage_kind, competition.competition_kind, 'custom') AS effective_kind,
                   home.canonical_name AS home_team_name,
                   away.canonical_name AS away_team_name
            FROM football.matches fixture
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            LEFT JOIN football.competition_stages stage ON stage.id = fixture.stage_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            WHERE fixture.id = $1
            "#,
        )
        .bind(match_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛不存在".to_string()))?;
        Ok(P4PlanningMatchContext {
            match_id: row.try_get("id")?,
            match_key: row.try_get("external_key")?,
            kickoff_at: row.try_get("kickoff_time")?,
            competition_id: row.try_get("competition_id")?,
            season_id: row.try_get("season_id")?,
            stage_id: row.try_get("stage_id")?,
            competition_kind: parse_competition_kind(
                row.try_get::<String, _>("effective_kind")?.as_str(),
            )?,
            home_team_name: row.try_get("home_team_name")?,
            away_team_name: row.try_get("away_team_name")?,
        })
    }

    pub async fn read_schema_version_by_key(
        &self,
        schema_key: &str,
        version: &str,
    ) -> PersistenceResult<SchemaVersionRecord> {
        let row = sqlx::query(
            r#"
            SELECT id, schema_key, version, schema_kind, content_sha256, created_at
            FROM research.schema_versions
            WHERE schema_key = $1 AND version = $2
            "#,
        )
        .bind(schema_key)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState(format!("未登记Schema版本：{schema_key}@{version}"))
        })?;
        Ok(SchemaVersionRecord {
            id: row.try_get("id")?,
            schema_key: row.try_get("schema_key")?,
            version: row.try_get("version")?,
            schema_kind: row.try_get("schema_kind")?,
            content_sha256: row.try_get("content_sha256")?,
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn read_research_run(
        &self,
        research_run_id: Uuid,
    ) -> PersistenceResult<ResearchRunRecord> {
        let row = sqlx::query(
            r#"
            SELECT id, match_id, horizon, data_cutoff_at, trace_id, idempotency_key,
                   request_fingerprint, status, created_at
            FROM research.runs
            WHERE id = $1
            "#,
        )
        .bind(research_run_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("研究任务不存在".to_string()))?;
        research_run_from_row(&row)
    }
}

fn research_run_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<ResearchRunRecord> {
    Ok(ResearchRunRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        horizon: parse_horizon(row.try_get::<String, _>("horizon")?.as_str())?,
        data_cutoff_at: row.try_get("data_cutoff_at")?,
        trace_id: row.try_get("trace_id")?,
        idempotency_key: row.try_get("idempotency_key")?,
        request_fingerprint: row.try_get("request_fingerprint")?,
        status: parse_research_status(row.try_get::<String, _>("status")?.as_str())?,
        created_at: row.try_get("created_at")?,
    })
}

fn parse_research_status(value: &str) -> PersistenceResult<ResearchRunStatus> {
    match value {
        "planned" => Ok(ResearchRunStatus::Planned),
        "running" => Ok(ResearchRunStatus::Running),
        "succeeded" => Ok(ResearchRunStatus::Succeeded),
        "partial" => Ok(ResearchRunStatus::Partial),
        "failed" => Ok(ResearchRunStatus::Failed),
        "cancelled" => Ok(ResearchRunStatus::Cancelled),
        other => Err(PersistenceError::InvalidState(format!(
            "未知研究任务状态：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn research_status_mapper_keeps_lowercase_storage_contract() {
        for value in [
            "planned",
            "running",
            "succeeded",
            "partial",
            "failed",
            "cancelled",
        ] {
            assert!(parse_research_status(value).is_ok());
        }
        for bad in ["PLANNED", " planned", "unknown", ""] {
            assert!(parse_research_status(bad).is_err());
        }
    }
}
