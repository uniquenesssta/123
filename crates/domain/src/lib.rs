pub mod coach;
pub mod competition;
pub mod formation;
pub mod lineup;
pub mod match_record;
pub mod player;
pub mod routing;
pub mod shared;
pub mod team;

mod analytics;
mod api_workspace;
mod exchange;
mod fact_pipeline;
mod match_event;
mod match_review_package;
mod match_review_workflow;
mod monthly_workbook;
mod p4_orchestration;
mod p4_persistence;
mod p4_workbench;
mod postmatch;
mod prediction_readiness;
mod release_acceptance;
mod research_gateway;
mod review;
mod spreadsheet;
mod team_package;
pub use coach::*;
pub use competition::*;
pub use formation::*;
pub use lineup::*;
pub use match_record::*;
pub use player::*;
pub use routing::*;
pub use shared::*;
pub use team::*;

pub use analytics::*;
pub use api_workspace::*;
pub use exchange::*;
pub use fact_pipeline::*;
pub use match_event::*;
pub use match_review_package::*;
pub use match_review_workflow::*;
pub use monthly_workbook::*;
pub use p4_orchestration::*;
pub use p4_persistence::*;
pub use p4_workbench::*;
pub use postmatch::*;
pub use prediction_readiness::*;
pub use release_acceptance::*;
pub use research_gateway::*;
pub use review::*;
pub use spreadsheet::*;
pub use team_package::*;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchContext {
    pub match_key: String,
    pub kickoff_time: DateTime<Utc>,
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub season_id: Option<Uuid>,
    #[serde(default)]
    pub stage_id: Option<Uuid>,
    #[serde(default)]
    pub competition_kind: CompetitionKind,
    pub home_team_name: String,
    pub away_team_name: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionSummary {
    pub home_win: f64,
    pub draw: f64,
    pub away_win: f64,
    pub btts: Option<f64>,
    pub over_2_5: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistedModelRun {
    pub id: Uuid,
    pub match_key: String,
    pub identity: ModelIdentity,
    pub snapshot_type: String,
    pub created_at: DateTime<Utc>,
    pub summary: PredictionSummary,
    pub output: Value,
}

fn default_true() -> bool {
    true
}

fn default_team_page_limit() -> u32 {
    50
}

fn default_confidence() -> f64 {
    1.0
}
