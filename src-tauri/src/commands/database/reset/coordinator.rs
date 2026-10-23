use crate::{bootstrap::AppState, config::DesktopConfig};
use football_persistence_postgres::DatabaseHealth;

pub(in crate::commands::database) async fn execute(
    state: &AppState,
    confirmation: String,
) -> Result<DatabaseHealth, String> {
    let config = DesktopConfig::load(&state.config_path)?;
    let options = config
        .database
        .ok_or_else(|| "尚未保存数据库连接，无法执行彻底清空".to_string())?;

    let current_health = state
        .service
        .preflight_database_reset(&options, &confirmation)
        .await
        .map_err(|error| error.to_string())?;

    let _ = state.runtime_log.record(
        "warning",
        "database",
        "destructive_reset_started",
        None,
        serde_json::json!({"database_name": current_health.database_name.as_str()}),
    );

    let health = state
        .service
        .reset_database(options, confirmation)
        .await
        .map_err(|error| {
            let message = error.to_string();
            if message.starts_with("彻底清空数据库失败：") {
                let _ = state.runtime_log.record(
                    "critical",
                    "database",
                    "destructive_reset_failed",
                    None,
                    serde_json::json!({
                        "database_name": current_health.database_name.as_str(),
                        "error": message.as_str(),
                    }),
                );
            }
            message
        })?;

    tokio::task::yield_now().await;
    state.service.ensure_p4_orchestration_worker();
    let _ = state.runtime_log.record(
        "warning",
        "database",
        "destructive_reset_succeeded",
        None,
        serde_json::json!({
            "database_name": health.database_name.as_str(),
            "migration_count": health.migration_count,
        }),
    );
    Ok(health)
}
