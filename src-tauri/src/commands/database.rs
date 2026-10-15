use super::AppState;
use crate::config::DesktopConfig;
use football_persistence_postgres::{DatabaseHealth, DatabaseOptions};
use tauri::State;

#[tauri::command]
pub async fn configure_database(
    state: State<'_, AppState>,
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
#[tauri::command]
pub async fn disconnect_database(state: State<'_, AppState>) -> Result<(), String> {
    DesktopConfig::default().save(&state.config_path)?;
    state.service.disconnect_database().await;
    Ok(())
}

#[tauri::command]
pub async fn reset_database(
    state: State<'_, AppState>,
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
