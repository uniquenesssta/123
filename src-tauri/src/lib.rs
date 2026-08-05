mod bootstrap;
mod commands;
mod config;
mod file_store;
mod issue_log;
mod openai_profiles;
mod runtime_log;
mod workspace_state;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    bootstrap::run();
}
