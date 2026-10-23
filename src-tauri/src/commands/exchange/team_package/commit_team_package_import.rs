use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn commit_team_package_import(
    state: State<'_, AppState>,
    request: football_domain::TeamPackageCommitRequest,
) -> Result<football_domain::TeamPackageCommitResult, String> {
    state
        .service
        .commit_team_package_import(request)
        .await
        .map_err(|error| error.to_string())
}
