use super::{
    configure::coordinator as configure, disconnect_database, reset::coordinator as reset,
};
use crate::{
    bootstrap::AppState, config::DesktopConfig, issue_log::IssueLogStore,
    openai_profiles::OpenAiProfileStore, runtime_log::tests::test_store,
    workspace_state::WorkspaceStateStore,
};
use football_application::{AiWorkspaceRequestRegistry, ApplicationService};
use football_persistence_postgres::DatabaseOptions;
use serde_json::Value;
use std::{fs, sync::Arc};
use tempfile::TempDir;

struct Fixture {
    state: AppState,
    directory: TempDir,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("isolated desktop directory");
        let state = AppState {
            service: Arc::new(ApplicationService::new()),
            config_path: directory.path().join("database.json"),
            issue_log: Arc::new(IssueLogStore::new(directory.path().join("issues.json"))),
            runtime_log: Arc::new(test_store(directory.path().join("runtime.jsonl"))),
            openai_profiles: Arc::new(OpenAiProfileStore::new(
                directory.path().join("profiles.json"),
            )),
            workspace_state: Arc::new(WorkspaceStateStore::new(
                directory.path().join("workspace.json"),
            )),
            api_workspace_requests: AiWorkspaceRequestRegistry::default(),
        };
        Self { state, directory }
    }

    fn events(&self) -> Vec<Value> {
        match fs::read_to_string(self.state.runtime_log.path()) {
            Ok(text) => text
                .lines()
                .map(|line| serde_json::from_str(line).expect("runtime JSON"))
                .collect(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Vec::new(),
            Err(error) => panic!("cannot read isolated log: {error}"),
        }
    }

    fn save_options(&self) {
        DesktopConfig {
            database: Some(invalid_options()),
        }
        .save(&self.state.config_path)
        .expect("original encrypted config store");
    }
}

// Invalid URI syntax fails during parsing; these tests never connect to a server.
fn invalid_options() -> DatabaseOptions {
    DatabaseOptions {
        connection_url: "postgres://[invalid".to_string(),
        max_connections: 1,
        connect_timeout_seconds: 1,
    }
}

#[tokio::test]
async fn configure_parse_failure_keeps_config_and_original_error_events() {
    let fixture = Fixture::new();
    DesktopConfig::default()
        .save(&fixture.state.config_path)
        .expect("existing empty config");
    let before = fs::read(&fixture.state.config_path).expect("original config bytes");
    let expected = fixture
        .state
        .service
        .connect_database(invalid_options())
        .await
        .expect_err("invalid URI")
        .to_string();
    let actual = configure::execute(&fixture.state, invalid_options())
        .await
        .expect_err("configure rejected");
    assert_eq!(actual, expected);
    assert_eq!(
        fs::read(&fixture.state.config_path).expect("unchanged config"),
        before
    );
    assert!(!fixture.state.service.is_database_connected().await);
    let events = fixture.events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0]["event"], "connection_attempt_started");
    assert_eq!(events[1]["event"], "connection_attempt_failed");
    assert_eq!(events[1]["level"], "error");
}

#[tokio::test]
async fn configure_logging_failure_does_not_replace_connection_error() {
    let mut fixture = Fixture::new();
    fixture.state.runtime_log = Arc::new(test_store(fixture.directory.path().to_path_buf()));
    let expected = fixture
        .state
        .service
        .connect_database(invalid_options())
        .await
        .expect_err("invalid URI")
        .to_string();
    assert_eq!(
        configure::execute(&fixture.state, invalid_options())
            .await
            .expect_err("original error"),
        expected
    );
    assert!(!fixture.state.config_path.exists());
    assert!(!fixture.state.service.is_database_connected().await);
}

#[tokio::test]
async fn disconnect_clears_saved_metadata_and_credentials_through_original_store() {
    let fixture = Fixture::new();
    fixture.save_options();
    let credentials = fixture.directory.path().join("database.credentials");
    assert!(credentials.is_file());
    disconnect_database::execute(&fixture.state)
        .await
        .expect("disconnect");
    assert!(DesktopConfig::load(&fixture.state.config_path)
        .expect("cleared metadata")
        .database
        .is_none());
    assert!(!credentials.exists());
    assert!(!fixture.state.service.is_database_connected().await);
    assert!(fixture.events().is_empty());
}

#[tokio::test]
async fn disconnect_save_failure_preserves_credentials_and_returns_file_error() {
    let fixture = Fixture::new();
    fs::create_dir(&fixture.state.config_path).expect("unwritable file destination");
    let credentials = fixture.directory.path().join("database.credentials");
    fs::write(&credentials, b"preserved credential fixture").expect("credential marker");
    let error = disconnect_database::execute(&fixture.state)
        .await
        .expect_err("save must fail");
    assert!(error.starts_with("无法原子替换文件 "));
    assert_eq!(
        fs::read(credentials).expect("credential preserved"),
        b"preserved credential fixture"
    );
    assert!(fixture.state.config_path.is_dir());
    assert!(fixture.events().is_empty());
}

#[tokio::test]
async fn reset_missing_or_empty_saved_connection_stops_before_backend_and_logs() {
    for saved in [false, true] {
        let fixture = Fixture::new();
        if saved {
            DesktopConfig::default()
                .save(&fixture.state.config_path)
                .expect("empty config");
        }
        assert_eq!(
            reset::execute(&fixture.state, "任意确认".to_string())
                .await
                .expect_err("no saved connection"),
            "尚未保存数据库连接，无法执行彻底清空"
        );
        assert_eq!(fixture.state.config_path.exists(), saved);
        assert!(!fixture.state.service.is_database_connected().await);
        assert!(fixture.events().is_empty());
    }
}

#[tokio::test]
async fn reset_config_errors_preserve_original_strings_without_reset_side_effects() {
    for content in ["invalid-json", r#"{"version":999,"database":null}"#] {
        let fixture = Fixture::new();
        fs::write(&fixture.state.config_path, content).expect("invalid config");
        let expected = DesktopConfig::load(&fixture.state.config_path).expect_err("config error");
        assert_eq!(
            reset::execute(&fixture.state, "确认".to_string())
                .await
                .expect_err("original config error"),
            expected
        );
        assert_eq!(
            fs::read_to_string(&fixture.state.config_path).expect("same config"),
            content
        );
        assert!(fixture.events().is_empty());
    }
}

#[tokio::test]
async fn reset_missing_credential_stops_before_reset_events_and_keeps_metadata() {
    let fixture = Fixture::new();
    fixture.save_options();
    fs::remove_file(fixture.directory.path().join("database.credentials"))
        .expect("missing credential");
    let before = fs::read(&fixture.state.config_path).expect("metadata");
    let expected =
        DesktopConfig::load(&fixture.state.config_path).expect_err("credential required");
    assert_eq!(
        reset::execute(&fixture.state, "确认".to_string())
            .await
            .expect_err("credential error"),
        expected
    );
    assert_eq!(
        fs::read(&fixture.state.config_path).expect("same metadata"),
        before
    );
    assert!(fixture.events().is_empty());
}

#[tokio::test]
async fn reset_preflight_parse_failure_keeps_config_without_destructive_events() {
    let fixture = Fixture::new();
    fixture.save_options();
    let before = fs::read(&fixture.state.config_path).expect("metadata");
    let credentials =
        fs::read(fixture.directory.path().join("database.credentials")).expect("credentials");
    let expected = fixture
        .state
        .service
        .preflight_database_reset(&invalid_options(), "raw confirmation")
        .await
        .expect_err("invalid URI preflight")
        .to_string();
    assert_eq!(
        reset::execute(&fixture.state, "raw confirmation".to_string())
            .await
            .expect_err("preflight error"),
        expected
    );
    assert_eq!(
        fs::read(&fixture.state.config_path).expect("same metadata"),
        before
    );
    assert_eq!(
        fs::read(fixture.directory.path().join("database.credentials")).expect("same credentials"),
        credentials
    );
    assert!(!fixture.state.service.is_database_connected().await);
    assert!(fixture.events().is_empty());
}
