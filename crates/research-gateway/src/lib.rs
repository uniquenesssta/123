mod api_example;
mod client;
mod config;
mod credentials;
mod error;
mod resilience;
mod response;
mod transport;
mod types;
mod validation;

pub use api_example::{parse_api_example, ApiExampleCandidate, ApiExampleParseResult};
pub use client::{test_openai_connection, GatewayAttemptSink, OpenAiResearchGateway};
pub use config::{
    ApiProtocol, ApiWorkspaceWebSearchMode, BudgetConfig, CircuitBreakerConfig, CredentialConfig,
    CredentialMode, GatewayConfig, ModelPricing, ReasoningEffort, SearchContextSize, SourcePolicy,
    TokenLimitField,
};
pub use credentials::{
    delete_windows_api_key, save_windows_api_key, windows_api_key_exists, ApiKey, ApiKeyProvider,
    DefaultApiKeyProvider,
};
pub use error::{GatewayError, GatewayErrorCategory, RecoveryAdvice};
pub use resilience::CancellationToken;
pub use transport::{OpenAiTransport, ReqwestTransport, TransportResponse};
pub use types::{
    CitationLocation, GatewayAttempt, GatewayExecution, GatewayOperation, GatewayRequest,
    GatewayResponse, GatewayUsage, MissingField, OpenAiConnectionTest, PlainTextGatewayExecution,
    PlainTextGatewayRequest, PlainTextGatewayResponse, PlainTextMessage, ResearchFact,
    ResearchOutput, ResearchSubject, ResearchValue, ResearchValueKind, StructuredGatewayExecution,
    StructuredGatewayRequest, StructuredGatewayResponse, WebCitation, WebSource,
};
pub use validation::{validate_research_output, ValidationContext};
