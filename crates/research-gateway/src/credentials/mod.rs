mod blob;
mod error;
mod key;
mod provider;
mod redaction;
mod store;
mod windows;

pub use key::ApiKey;
pub use provider::{ApiKeyProvider, DefaultApiKeyProvider};
pub(crate) use redaction::{is_placeholder_key, sanitized_api_example};
pub use store::{delete_windows_api_key, save_windows_api_key, windows_api_key_exists};

#[cfg(test)]
pub(crate) use redaction::API_KEY_PLACEHOLDER;
