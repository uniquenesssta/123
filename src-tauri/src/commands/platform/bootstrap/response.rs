use football_application::BootstrapData;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct BootstrapResponse {
    pub data: BootstrapData,
    pub connection_error: Option<String>,
    pub config_path: String,
    pub runtime_log_path: String,
}
