use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayErrorCategory {
    MissingCredential,
    InvalidConfiguration,
    Authentication,
    Permission,
    RateLimit,
    Timeout,
    Network,
    ProviderUnavailable,
    ModelUnavailable,
    Refused,
    NoResult,
    SchemaValidation,
    SourcePolicy,
    BudgetExceeded,
    ConcurrencyLimit,
    CircuitOpen,
    Cancelled,
    Persistence,
    Unknown,
}

impl GatewayErrorCategory {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MissingCredential => "missing_credential",
            Self::InvalidConfiguration => "invalid_configuration",
            Self::Authentication => "authentication",
            Self::Permission => "permission",
            Self::RateLimit => "rate_limit",
            Self::Timeout => "timeout",
            Self::Network => "network",
            Self::ProviderUnavailable => "provider_unavailable",
            Self::ModelUnavailable => "model_unavailable",
            Self::Refused => "refused",
            Self::NoResult => "no_result",
            Self::SchemaValidation => "schema_validation",
            Self::SourcePolicy => "source_policy",
            Self::BudgetExceeded => "budget_exceeded",
            Self::ConcurrencyLimit => "concurrency_limit",
            Self::CircuitOpen => "circuit_open",
            Self::Cancelled => "cancelled",
            Self::Persistence => "persistence",
            Self::Unknown => "unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecoveryAdvice {
    pub retryable: bool,
    pub action: String,
}

#[derive(Debug, Error, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[error("{user_message}")]
pub struct GatewayError {
    pub category: GatewayErrorCategory,
    pub user_message: String,
    pub recovery: RecoveryAdvice,
    #[serde(default)]
    pub provider_status: Option<u16>,
    #[serde(default)]
    pub provider_code: Option<String>,
}

impl GatewayError {
    pub fn new(
        category: GatewayErrorCategory,
        user_message: impl Into<String>,
        retryable: bool,
        action: impl Into<String>,
    ) -> Self {
        Self {
            category,
            user_message: user_message.into(),
            recovery: RecoveryAdvice {
                retryable,
                action: action.into(),
            },
            provider_status: None,
            provider_code: None,
        }
    }

    pub fn with_provider(mut self, status: Option<u16>, code: Option<String>) -> Self {
        self.provider_status = status;
        self.provider_code = code;
        self
    }
}
