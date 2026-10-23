use crate::{bootstrap::AppState, config::DesktopConfig};
use football_persistence_postgres::{DatabaseHealth, DatabaseOptions};

pub(in crate::commands::database) async fn execute(
    state: &AppState,
    options: DatabaseOptions,
) -> Result<DatabaseHealth, String> {
    let _ = state.runtime_log.record(
        "info",
        "database",
        "connection_attempt_started",
        None,
        serde_json::json!({"options": &options}),
    );
    let health = state
        .service
        .connect_database(options.clone())
        .await
        .map_err(|error| {
            let message = error.to_string();
            let _ = state.runtime_log.record(
                "error",
                "database",
                "connection_attempt_failed",
                None,
                serde_json::json!({"error": message}),
            );
            message
        })?;
    let _ = state.runtime_log.record(
        "info",
        "database",
        "connection_attempt_succeeded",
        None,
        serde_json::json!({"health": &health}),
    );
    if let Err(error) = (DesktopConfig {
        database: Some(options),
    })
    .save(&state.config_path)
    {
        state.service.disconnect_database().await;
        return Err(error);
    }
    Ok(health)
}
