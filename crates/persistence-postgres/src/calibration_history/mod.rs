mod catalog;
mod packet;
mod write;

use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use serde_json::{json, Value};
use sqlx::postgres::PgConnectOptions;
use std::str::FromStr;
use uuid::Uuid;

const RESEARCH_DATABASE: &str = "football_r17_research";
const RESEARCH_OWNER: &str = "r17_research_owner";
const RESEARCH_HOST: &str = "127.0.0.1";
const RESEARCH_PORT: u16 = 55438;

fn invalid(message: impl Into<String>) -> PersistenceError {
    PersistenceError::InvalidState(message.into())
}

/// Connect only to the dedicated loopback research store, never the application or QA database.
pub async fn connect_research_database(url: &str) -> PersistenceResult<PostgresStore> {
    let options = PgConnectOptions::from_str(url)?;
    if options.get_host() != RESEARCH_HOST
        || options.get_port() != RESEARCH_PORT
        || options.get_database() != Some(RESEARCH_DATABASE)
        || options.get_username() != RESEARCH_OWNER
    {
        return Err(invalid("仅允许使用隔离的 R17 研究库连接"));
    }
    let store = PostgresStore::connect(&crate::DatabaseOptions {
        connection_url: url.to_owned(),
        max_connections: 2,
        connect_timeout_seconds: 10,
    })
    .await?;
    store.verify_research_database().await?;
    Ok(store)
}

impl PostgresStore {
    async fn verify_research_database(&self) -> PersistenceResult<()> {
        let valid: bool = sqlx::query_scalar(
            "SELECT coalesce(inet_server_addr() = '127.0.0.1'::inet AND inet_server_port() = $3, false) \
             AND current_database() = $1 AND current_user = $2 \
             AND EXISTS (SELECT 1 FROM pg_roles WHERE rolname = current_user \
                 AND NOT (rolsuper OR rolcreatedb OR rolcreaterole OR rolreplication OR rolbypassrls))",
        ).bind(RESEARCH_DATABASE).bind(RESEARCH_OWNER).bind(i32::from(RESEARCH_PORT))
            .fetch_one(&self.pool).await?;
        if !valid {
            return Err(invalid("数据库服务身份与隔离研究库不匹配"));
        }
        Ok(())
    }

    /// Record only verified fixture and score facts for prospective calibration history.
    pub async fn import_calibration_history(&self, bytes: &[u8]) -> PersistenceResult<Value> {
        let packet = packet::Packet::open(bytes)?;
        self.verify_research_database().await?;
        write::import(self, packet).await
    }

    /// Read historical feature quality through the same calculation used by model input preparation.
    pub async fn inspect_calibration_history(
        &self,
        match_id: Uuid,
        cutoff: DateTime<Utc>,
    ) -> PersistenceResult<Value> {
        self.verify_research_database().await?;
        let fixture = self.read_match(match_id).await?;
        let now: DateTime<Utc> = sqlx::query_scalar("SELECT clock_timestamp()")
            .fetch_one(&self.pool)
            .await?;
        if !fixture.external_key.starts_with("FPL-EPL-2026-27-")
            || fixture.competition_id
                != Some(catalog::identity("competition", catalog::COMPETITION_CODE))
            || fixture.season_id != Some(catalog::identity("season", packet::SEASON))
            || cutoff > now
            || cutoff >= fixture.kickoff_time
        {
            return Err(invalid("仅可检查已绑定研究比赛在当前时间之前的赛前历史"));
        }
        let home = self
            .calculate_team_pre_match_features(&fixture, fixture.home_team_id, true, cutoff)
            .await?;
        let away = self
            .calculate_team_pre_match_features(&fixture, fixture.away_team_id, false, cutoff)
            .await?;
        Ok(json!({
            "status": "HISTORY_INSPECTED", "match_id": match_id, "cutoff": cutoff,
            "home": home.quality, "away": away.quality,
        }))
    }
}
