mod analytics;
mod api_workspace;
mod catalog;
mod competition;
mod database;
mod exchange;
mod lineups;
mod openai;
mod platform;
mod postmatch;
mod prediction;
mod release_acceptance;
mod review;

pub use analytics::*;
pub use api_workspace::*;
pub use catalog::*;
pub use competition::*;
pub use database::*;
pub use exchange::*;
pub use lineups::*;
pub use openai::*;
pub use platform::*;
pub use postmatch::*;
pub use prediction::*;
pub use release_acceptance::*;
pub use review::*;

use platform::record_failed_background_jobs;

pub(crate) use crate::bootstrap::AppState;
use uuid::Uuid;

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
