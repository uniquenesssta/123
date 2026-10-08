use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::FactPipelineContext;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn fact_pipeline_context(
        &self,
        research_run_id: Uuid,
    ) -> PersistenceResult<FactPipelineContext> {
        let row = sqlx::query(
            r#"
            SELECT
                run.id AS research_run_id,
                run.match_id,
                match.external_key AS match_key,
                run.horizon,
                run.data_cutoff_at,
                run.trace_id,
                run.prompt_version_id,
                prompt.version AS prompt_version,
                run.schema_version_id,
                schema.version AS schema_version,
                match.home_team_id,
                home.canonical_name AS home_team_name,
                match.away_team_id,
                away.canonical_name AS away_team_name,
                match.competition_id,
                competition.name AS competition_name,
                competition.code AS competition_code
            FROM research.runs run
            JOIN football.matches match ON match.id = run.match_id
            JOIN research.schema_versions schema ON schema.id = run.schema_version_id
            LEFT JOIN research.prompt_versions prompt ON prompt.id = run.prompt_version_id
            LEFT JOIN football.teams home ON home.id = match.home_team_id
            LEFT JOIN football.teams away ON away.id = match.away_team_id
            LEFT JOIN football.competitions competition ON competition.id = match.competition_id
            WHERE run.id = $1
            "#,
        )
        .bind(research_run_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("研究任务不存在".to_string()))?;
        Ok(FactPipelineContext {
            research_run_id: row.try_get("research_run_id")?,
            match_id: row.try_get("match_id")?,
            match_key: row.try_get("match_key")?,
            horizon: row.try_get("horizon")?,
            data_cutoff_at: row.try_get("data_cutoff_at")?,
            trace_id: row.try_get("trace_id")?,
            prompt_version_id: row.try_get("prompt_version_id")?,
            prompt_version: row.try_get("prompt_version")?,
            schema_version_id: row.try_get("schema_version_id")?,
            schema_version: row.try_get("schema_version")?,
            home_team_id: row.try_get("home_team_id")?,
            home_team_name: row.try_get("home_team_name")?,
            away_team_id: row.try_get("away_team_id")?,
            away_team_name: row.try_get("away_team_name")?,
            competition_id: row.try_get("competition_id")?,
            competition_name: row.try_get("competition_name")?,
            competition_code: row.try_get("competition_code")?,
        })
    }
}
