use crate::bootstrap::AppState;
use football_domain::{RulePackageDraft, RulePackageSummary};
use tauri::State;

#[tauri::command]
pub async fn register_rule_package(
    state: State<'_, AppState>,
    draft: RulePackageDraft,
) -> Result<RulePackageSummary, String> {
    state
        .service
        .register_rule_package(draft)
        .await
        .map_err(|error| error.to_string())
}
