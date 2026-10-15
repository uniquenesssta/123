use crate::config::DesktopConfig;
use football_application::ApplicationService;
use std::{path::Path, sync::Arc};

pub(super) async fn ensure_saved_connection(
    service: &Arc<ApplicationService>,
    config_path: &Path,
) -> Option<String> {
    if service.is_database_connected().await {
        return None;
    }
    let config = match DesktopConfig::load(config_path) {
        Ok(config) => config,
        Err(error) => return Some(error),
    };
    let options = config.database?;
    service
        .connect_database(options)
        .await
        .err()
        .map(|error| error.to_string())
}
