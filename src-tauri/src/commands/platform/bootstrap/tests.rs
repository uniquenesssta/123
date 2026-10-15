use super::{connection::ensure_saved_connection, response::BootstrapResponse};
use crate::config::DesktopConfig;
use football_application::ApplicationService;
use serde_json::{json, Value};
use std::sync::Arc;

#[tokio::test]
async fn missing_saved_connection_does_not_create_config_or_connect() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("database.json");
    let service = Arc::new(ApplicationService::new());
    assert_eq!(ensure_saved_connection(&service, &path).await, None);
    assert!(!path.exists());
    assert!(!service.is_database_connected().await);
}

#[tokio::test]
async fn saved_config_errors_are_returned_unchanged_without_repair_or_connection() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("database.json");
    let service = Arc::new(ApplicationService::new());
    for content in ["not-json", r#"{"version":999,"database":null}"#] {
        std::fs::write(&path, content).expect("config fixture");
        let expected = DesktopConfig::load(&path).expect_err("invalid config");
        assert_eq!(
            ensure_saved_connection(&service, &path).await,
            Some(expected)
        );
        assert_eq!(
            std::fs::read_to_string(&path).expect("unchanged config"),
            content
        );
        assert!(!service.is_database_connected().await);
    }
}

#[tokio::test]
async fn empty_current_config_is_read_without_rewriting_or_connecting() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("database.json");
    DesktopConfig::default()
        .save(&path)
        .expect("current config");
    let before = std::fs::read(&path).expect("saved bytes");
    let service = Arc::new(ApplicationService::new());
    assert_eq!(ensure_saved_connection(&service, &path).await, None);
    assert_eq!(std::fs::read(&path).expect("same bytes"), before);
    assert!(!service.is_database_connected().await);
}

#[tokio::test]
async fn legacy_empty_config_keeps_original_load_migration() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let path = directory.path().join("database.json");
    std::fs::write(&path, r#"{"database":null}"#).expect("legacy config");
    let service = Arc::new(ApplicationService::new());
    assert_eq!(ensure_saved_connection(&service, &path).await, None);
    let migrated: Value = serde_json::from_slice(&std::fs::read(&path).expect("migrated bytes"))
        .expect("current JSON");
    assert_eq!(migrated, json!({"version": 2, "database": null}));
    assert!(!service.is_database_connected().await);
}

#[tokio::test]
async fn bootstrap_response_keeps_original_wire_fields_nulls_and_paths() {
    let service = ApplicationService::new();
    for connection_error in [None, Some("original restore error".to_string())] {
        let data = service.bootstrap().await.expect("disconnected bootstrap");
        let expected_data = serde_json::to_value(&data).expect("original data");
        let response = BootstrapResponse {
            data,
            connection_error: connection_error.clone(),
            config_path: "原始/配置/database.json".to_string(),
            runtime_log_path: r".\logs\original.jsonl".to_string(),
        };
        assert_eq!(
            serde_json::to_value(response).expect("response"),
            json!({
                "data": expected_data,
                "connection_error": connection_error,
                "config_path": "原始/配置/database.json",
                "runtime_log_path": r".\logs\original.jsonl",
            })
        );
    }
}
