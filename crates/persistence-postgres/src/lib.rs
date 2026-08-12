mod analytics;
mod api_workspace;
mod competitions;
mod dynamic_tags;
mod entity_catalog;
mod error;
mod fact_pipeline_records;
mod formation_catalog;
mod health;
mod jobs;
mod lineup_chain;
mod match_exchange;
mod match_prediction;
mod match_review_package;
mod migrations;
mod model_runs;
mod monthly_workbooks;
mod name_search;
mod p4_orchestration;
mod p4_records;
mod p4_workbench;
mod parameter_lifecycle;
mod player_catalog;
mod pool;
mod postmatch;
mod release_acceptance;
mod research_gateway_records;
mod review;
mod role_resolution;
mod routing;
mod spreadsheet_exchange;
mod statistics;
mod store;
mod team_catalog;
mod team_features;
mod team_force_delete;
mod team_lineup_presets;

pub use error::{PersistenceError, PersistenceResult};
pub use health::DatabaseHealth;
pub use model_runs::ModelRunListItem;
pub use pool::DatabaseOptions;
pub use routing::ModelRegistration;
pub use statistics::DatabaseStats;
pub use store::PostgresStore;

use football_domain::CompetitionKind;
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::Transaction;
use uuid::Uuid;

async fn write_audit_event(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    event_type: &str,
    entity_type: &str,
    entity_id: Option<String>,
    payload: Value,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
        INSERT INTO audit.events (id, event_type, entity_type, entity_id, payload)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(event_type)
    .bind(entity_type)
    .bind(entity_id)
    .bind(payload)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn parse_competition_kind(value: &str) -> PersistenceResult<CompetitionKind> {
    match value {
        "league" => Ok(CompetitionKind::League),
        "group_stage" => Ok(CompetitionKind::GroupStage),
        "knockout_single_leg" => Ok(CompetitionKind::KnockoutSingleLeg),
        "knockout_two_leg" => Ok(CompetitionKind::KnockoutTwoLeg),
        "friendly" => Ok(CompetitionKind::Friendly),
        "custom" => Ok(CompetitionKind::Custom),
        other => Err(PersistenceError::InvalidState(format!(
            "未知赛事类型：{other}"
        ))),
    }
}

fn sha256_json<T: serde::Serialize + ?Sized>(value: &T) -> PersistenceResult<String> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}
