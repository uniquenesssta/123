use crate::{ApiKey, GatewayError};
use async_trait::async_trait;
use serde_json::Value;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct TransportResponse {
    pub status: u16,
    pub provider_request_id: Option<String>,
    pub body: Value,
}

#[async_trait]
pub trait OpenAiTransport: Send + Sync {
    async fn post_json(
        &self,
        url: &str,
        api_key: &ApiKey,
        body: &Value,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError>;

    async fn get_json(
        &self,
        url: &str,
        api_key: &ApiKey,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError>;

    async fn post_empty(
        &self,
        url: &str,
        api_key: &ApiKey,
        timeout: Duration,
    ) -> Result<TransportResponse, GatewayError>;
}
