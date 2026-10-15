use super::{connection::ensure_saved_connection, response::BootstrapResponse};
use crate::{bootstrap::AppState, issue_log::IssueLogDraft};

pub(super) async fn execute(state: &AppState) -> Result<BootstrapResponse, String> {
    let connection_error = ensure_saved_connection(&state.service, &state.config_path).await;
    if let Some(message) = connection_error.as_ref() {
        let _ = state.issue_log.record(IssueLogDraft {
            severity: "error".to_string(),
            source: "startup".to_string(),
            operation: "恢复数据库连接".to_string(),
            user_message: "启动时未能恢复数据库连接".to_string(),
            technical_message: message.clone(),
            occurrence_key: None,
        });
        let _ = state.runtime_log.record(
            "error",
            "database",
            "saved_connection_restore_failed",
            None,
            serde_json::json!({"error": message}),
        );
    }
    let data = match state.service.bootstrap().await {
        Ok(data) => data,
        Err(error) => {
            let message = error.to_string();
            let _ = state.issue_log.record(IssueLogDraft {
                severity: "critical".to_string(),
                source: "startup".to_string(),
                operation: "平台初始化".to_string(),
                user_message: "平台初始化失败".to_string(),
                technical_message: message.clone(),
                occurrence_key: None,
            });
            let _ = state.runtime_log.record(
                "critical",
                "application",
                "bootstrap_failed",
                None,
                serde_json::json!({"error": message}),
            );
            return Err(message);
        }
    };
    Ok(BootstrapResponse {
        data,
        connection_error,
        config_path: state.config_path.to_string_lossy().to_string(),
        runtime_log_path: state.runtime_log.relative_display_path(),
    })
}
