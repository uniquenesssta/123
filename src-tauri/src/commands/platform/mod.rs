mod background_jobs;
mod bootstrap;
mod clear_issue_logs;
mod clear_workspace_state;
mod export_issue_logs;
mod list_issue_logs;
mod read_workspace_state;
mod record_issue;
mod runtime_operation;
mod save_workspace_state;

pub use bootstrap::{__cmd__bootstrap, __tauri_command_name_bootstrap, bootstrap};
pub use clear_issue_logs::{
    __cmd__clear_issue_logs, __tauri_command_name_clear_issue_logs, clear_issue_logs,
};
pub use clear_workspace_state::{
    __cmd__clear_workspace_state, __tauri_command_name_clear_workspace_state, clear_workspace_state,
};
pub use export_issue_logs::{
    __cmd__export_issue_logs, __tauri_command_name_export_issue_logs, export_issue_logs,
};
pub use list_issue_logs::{
    __cmd__list_issue_logs, __tauri_command_name_list_issue_logs, list_issue_logs,
};
pub use read_workspace_state::{
    __cmd__read_workspace_state, __tauri_command_name_read_workspace_state, read_workspace_state,
};
pub use record_issue::{__cmd__record_issue, __tauri_command_name_record_issue, record_issue};
pub use runtime_operation::{
    __cmd__record_runtime_operation, __tauri_command_name_record_runtime_operation,
    record_runtime_operation,
};
pub use save_workspace_state::{
    __cmd__save_workspace_state, __tauri_command_name_save_workspace_state, save_workspace_state,
};

pub(super) use background_jobs::record_failed_background_jobs;

#[cfg(test)]
mod tests;
