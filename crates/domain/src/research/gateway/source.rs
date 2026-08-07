use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebCitationDraft {
    pub research_run_id: Uuid,
    pub response_id: String,
    pub url: String,
    pub title: String,
    pub domain: String,
    pub output_index: u32,
    #[serde(default)]
    pub start_index: Option<u32>,
    #[serde(default)]
    pub end_index: Option<u32>,
    pub retrieved_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSourceDraft {
    pub research_run_id: Uuid,
    pub response_id: String,
    pub url: String,
    #[serde(default)]
    pub title: Option<String>,
    pub domain: String,
    pub retrieved_at: DateTime<Utc>,
}
