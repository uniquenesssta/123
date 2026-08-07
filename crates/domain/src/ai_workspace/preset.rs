use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApiWorkspacePreset {
    pub key: String,
    pub title: String,
    pub description: String,
    pub category: String,
    pub web_search_enabled: bool,
    pub requires_match: bool,
    pub allowed_operation_types: Vec<String>,
    pub suggested_questions: Vec<String>,
}
