mod decide_ai_analysis_suggestion;
mod export_ai_analysis_package;
mod export_ai_analysis_response_template;
mod import_ai_analysis_response;
mod list_ai_analysis_suggestions;
mod preview_ai_analysis_response;

pub use decide_ai_analysis_suggestion::{
    __cmd__decide_ai_analysis_suggestion, __tauri_command_name_decide_ai_analysis_suggestion,
    decide_ai_analysis_suggestion,
};
pub use export_ai_analysis_package::{
    __cmd__export_ai_analysis_package, __tauri_command_name_export_ai_analysis_package,
    export_ai_analysis_package,
};
pub use export_ai_analysis_response_template::{
    __cmd__export_ai_analysis_response_template,
    __tauri_command_name_export_ai_analysis_response_template,
    export_ai_analysis_response_template,
};
pub use import_ai_analysis_response::{
    __cmd__import_ai_analysis_response, __tauri_command_name_import_ai_analysis_response,
    import_ai_analysis_response,
};
pub use list_ai_analysis_suggestions::{
    __cmd__list_ai_analysis_suggestions, __tauri_command_name_list_ai_analysis_suggestions,
    list_ai_analysis_suggestions,
};
pub use preview_ai_analysis_response::{
    __cmd__preview_ai_analysis_response, __tauri_command_name_preview_ai_analysis_response,
    preview_ai_analysis_response,
};
