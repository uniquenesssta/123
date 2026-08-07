use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const API_WORKSPACE_SCHEMA_VERSION: &str = "football.api-workspace-response.v2";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceAssistantOperation {
    pub proposal_key: String,
    pub operation_type: String,
    pub payload: Value,
    pub rationale: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceAssistantFile {
    pub filename: String,
    pub media_type: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ApiWorkspaceAssistantOutput {
    pub schema_version: String,
    pub answer: String,
    pub summary: String,
    pub key_points: Vec<String>,
    pub missing_information: Vec<String>,
    pub warnings: Vec<String>,
    pub proposed_operations: Vec<ApiWorkspaceAssistantOperation>,
    pub generated_files: Vec<ApiWorkspaceAssistantFile>,
}
