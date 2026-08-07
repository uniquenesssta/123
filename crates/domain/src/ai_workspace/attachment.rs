use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiWorkspaceAttachment {
    pub name: String,
    pub media_type: String,
    pub content: String,
    pub content_sha256: String,
    pub original_size_bytes: u64,
    pub truncated: bool,
}
