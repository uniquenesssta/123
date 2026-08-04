mod api_example;
mod cancellation;
mod client;
mod config;
mod credentials;
mod error;
mod response;
mod types;
mod validation;

pub use api_example::{parse_api_example, ApiExampleCandidate, ApiExampleParseResult};
pub use cancellation::CancellationToken;
pub use client::{
    test_openai_connection, GatewayAttemptSink, OpenAiResearchGateway, OpenAiTransport,
    ReqwestTransport, TransportResponse,
};
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
pub use types::{
    CitationLocation, GatewayAttempt, GatewayExecution, GatewayOperation, GatewayRequest,
    GatewayResponse, GatewayUsage, MissingField, OpenAiConnectionTest, PlainTextGatewayExecution,
    PlainTextGatewayRequest, PlainTextGatewayResponse, PlainTextMessage, ResearchFact,
    ResearchOutput, ResearchSubject, ResearchValue, ResearchValueKind, StructuredGatewayExecution,
    StructuredGatewayRequest, StructuredGatewayResponse, WebCitation, WebSource,
};
pub use validation::{validate_research_output, ValidationContext};
