//! Opt-in Windows acceptance in the existing lib test target. Real commands and WebView,
//! with every mutable store redirected to the explicitly supplied project QA directory.
use crate::{
    issue_log::IssueLogStore, openai_profiles::OpenAiProfileStore, runtime_log::RuntimeLogStore,
    workspace_state::WorkspaceStateStore,
};
use football_application::{AiWorkspaceRequestRegistry, ApplicationService};
use std::{path::PathBuf, sync::Arc};
use tauri::Manager;

#[test]
#[ignore = "requires explicit Windows GUI/CDP acceptance and a project QA root"]
fn isolated_log_commands_window() {
    let root = PathBuf::from(
        std::env::var_os("FOOTBALL_NATIVE_LOG_ACCEPTANCE_ROOT").expect("explicit QA root"),
    );
    assert_eq!(
        root,
        PathBuf::from(r"O:\Project\FOODBALL\tmp\r15-12-native-config")
    );
    std::fs::create_dir_all(&root).expect("QA directory");
    use std::os::windows::fs::MetadataExt;
    assert_eq!(
        std::fs::symlink_metadata(&root).unwrap().file_attributes() & 0x400,
        0,
        "no QA root reparse point"
    );
    let mut context = tauri::generate_context!();
    for window in &mut context.config_mut().app.windows {
        window.data_directory = Some(root.join("webview"));
    }
    let builder = tauri::Builder::default()
        .any_thread()
        .plugin(tauri_plugin_dialog::init())
        .setup(move |app| {
            app.manage(super::AppState {
                service: Arc::new(ApplicationService::new()),
                config_path: root.join("database.json"),
                issue_log: Arc::new(IssueLogStore::new(root.join("issue-log.json"))),
                runtime_log: Arc::new(RuntimeLogStore::discover(&root)),
                openai_profiles: Arc::new(OpenAiProfileStore::new(
                    root.join("openai-profiles.json"),
                )),
                workspace_state: Arc::new(WorkspaceStateStore::new(
                    root.join("workspace-state.json"),
                )),
                api_workspace_requests: AiWorkspaceRequestRegistry::default(),
            });
            Ok(())
        });
    super::command_registry::register(builder)
        .run(context)
        .expect("isolated real native application");
}
