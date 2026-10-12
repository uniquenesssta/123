use super::GatewayAttempt;
use crate::GatewayError;
use async_trait::async_trait;

#[async_trait]
pub trait GatewayAttemptSink: Send + Sync {
    async fn record(&self, attempt: &GatewayAttempt) -> Result<(), GatewayError>;
}

#[derive(Debug, Default)]
pub(crate) struct NoopAttemptSink;

#[async_trait]
impl GatewayAttemptSink for NoopAttemptSink {
    async fn record(&self, _attempt: &GatewayAttempt) -> Result<(), GatewayError> {
        Ok(())
    }
}
