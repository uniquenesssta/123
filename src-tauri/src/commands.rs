mod analytics;
mod api_workspace;
mod catalog;
mod competition;
mod database;
mod exchange;
mod issues;
mod logging;
mod openai;
mod orchestration;
mod prediction;
mod postmatch;
mod release_acceptance;
mod review;
mod workspace;

pub use analytics::*;
pub use api_workspace::*;
pub use catalog::*;
pub use competition::*;
pub use database::*;
pub use exchange::*;
pub use issues::*;
pub use logging::*;
pub use openai::*;
pub use orchestration::*;
pub use prediction::*;
pub use postmatch::*;
pub use release_acceptance::*;
pub use review::*;
pub use workspace::*;

use crate::issue_log::IssueLogStore;
use crate::openai_profiles::OpenAiProfileStore;
use crate::runtime_log::RuntimeLogStore;
use crate::workspace_state::WorkspaceStateStore;
use football_application::ApplicationService;
use football_research_gateway::CancellationToken;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use uuid::Uuid;

pub struct AppState {
    pub service: Arc<ApplicationService>,
    pub config_path: PathBuf,
    pub issue_log: Arc<IssueLogStore>,
    pub runtime_log: Arc<RuntimeLogStore>,
    pub openai_profiles: Arc<OpenAiProfileStore>,
    pub workspace_state: Arc<WorkspaceStateStore>,
    pub api_workspace_requests: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

pub(super) fn parse_uuid(value: &str, label: &str) -> Result<Uuid, String> {
    Uuid::parse_str(value).map_err(|error| format!("{label} 无效：{error}"))
}

#[cfg(test)]
mod tests {
    use super::parse_uuid;

    #[test]
    fn parse_uuid_preserves_context_in_error() {
        let error = parse_uuid("not-a-uuid", "比赛 ID").expect_err("invalid UUID must fail");
        assert!(error.starts_with("比赛 ID 无效："));
    }
}
