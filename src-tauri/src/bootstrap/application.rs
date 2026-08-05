use super::{command_registry, error, state};

pub(crate) fn run() {
    let builder = tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| -> Result<(), Box<dyn std::error::Error>> {
            state::install(app)?;
            Ok(())
        });
    error::expect_startup(command_registry::register(builder).run(tauri::generate_context!()));
}
