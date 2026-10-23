use football_domain::ApiWorkspaceAttachment;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct SendApiWorkspaceCommand {
    #[serde(default)]
    pub session_id: Option<String>,
    pub profile_id: String,
    pub preset_key: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub match_id: Option<String>,
    #[serde(default)]
    pub context_entity_type: Option<String>,
    #[serde(default)]
    pub context_entity_id: Option<String>,
    #[serde(default)]
    pub context_entity_label: Option<String>,
    pub message: String,
    #[serde(default)]
    pub include_context: bool,
    #[serde(default)]
    pub request_id: String,
    #[serde(default)]
    pub attachments: Vec<ApiWorkspaceAttachment>,
}
