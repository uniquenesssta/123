use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOptions {
    pub connection_url: String,
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
    #[serde(default = "default_connect_timeout_seconds")]
    pub connect_timeout_seconds: u64,
}

fn default_max_connections() -> u32 {
    10
}

fn default_connect_timeout_seconds() -> u64 {
    10
}

impl DatabaseOptions {
    pub fn redacted_url(&self) -> String {
        redact_database_url(&self.connection_url)
    }
}

fn redact_database_url(url: &str) -> String {
    let Some(scheme_end) = url.find("://") else {
        return "已配置".to_string();
    };
    let scheme = &url[..scheme_end + 3];
    let rest = &url[scheme_end + 3..];
    let Some(at) = rest.rfind('@') else {
        return url.to_string();
    };
    let credentials = &rest[..at];
    let host = &rest[at + 1..];
    let user = credentials.split(':').next().unwrap_or("user");
    format!("{scheme}{user}:***@{host}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn database_url_redaction_hides_password() {
        assert_eq!(
            redact_database_url("postgres://football_app:secret@localhost:5432/football_model"),
            "postgres://football_app:***@localhost:5432/football_model"
        );
    }
}
