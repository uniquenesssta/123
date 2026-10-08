use chrono::{DateTime, Utc};
use football_research_gateway::{ResearchOutput, WebCitation, WebSource};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResearchEvidenceCommand {
    pub research_run_id: Uuid,
    pub response_id: String,
    pub retrieved_at: DateTime<Utc>,
    pub output: ResearchOutput,
    #[serde(default)]
    pub citations: Vec<WebCitation>,
    #[serde(default)]
    pub sources: Vec<WebSource>,
}
