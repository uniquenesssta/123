use crate::{CircuitBreakerConfig, GatewayError, GatewayErrorCategory};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

struct CircuitState {
    consecutive_failures: u32,
    open_until: Option<Instant>,
}

pub(crate) struct CircuitBreaker {
    state: Mutex<CircuitState>,
}

impl CircuitBreaker {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(CircuitState {
                consecutive_failures: 0,
                open_until: None,
            }),
        }
    }

    pub(crate) async fn check(&self) -> Result<(), GatewayError> {
        let mut state = self.state.lock().await;
        if let Some(open_until) = state.open_until {
            if Instant::now() < open_until {
                return Err(GatewayError::new(
                    GatewayErrorCategory::CircuitOpen,
                    "OpenAI研究网关因连续失败已暂时熔断",
                    true,
                    "等待熔断窗口结束；历史记录和手工事实仍可继续使用",
                ));
            }
            state.open_until = None;
            state.consecutive_failures = 0;
        }
        Ok(())
    }

    pub(crate) async fn record_success(&self) {
        let mut state = self.state.lock().await;
        state.consecutive_failures = 0;
        state.open_until = None;
    }

    pub(crate) async fn record_failure(&self, error: &GatewayError, config: &CircuitBreakerConfig) {
        if !matches!(
            error.category,
            GatewayErrorCategory::Network
                | GatewayErrorCategory::Timeout
                | GatewayErrorCategory::RateLimit
                | GatewayErrorCategory::ProviderUnavailable
        ) {
            return;
        }
        let mut state = self.state.lock().await;
        state.consecutive_failures = state.consecutive_failures.saturating_add(1);
        if state.consecutive_failures >= config.consecutive_failure_threshold {
            state.open_until = Some(Instant::now() + Duration::from_secs(config.open_seconds));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn policy(threshold: u32) -> CircuitBreakerConfig {
        CircuitBreakerConfig {
            consecutive_failure_threshold: threshold,
            open_seconds: 30,
        }
    }

    fn error(category: GatewayErrorCategory) -> GatewayError {
        GatewayError::new(category, "fixture failure", false, "fixture recovery")
    }

    #[tokio::test]
    async fn circuit_counts_only_original_four_categories_even_when_not_retryable() {
        use GatewayErrorCategory::*;
        for category in [
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
        ] {
            let circuit = CircuitBreaker::new();
            circuit.record_failure(&error(category), &policy(2)).await;
            let counted = matches!(
                category,
                Network | Timeout | RateLimit | ProviderUnavailable
            );
            let state = circuit.state.lock().await;
            assert_eq!(
                state.consecutive_failures,
                if counted { 1 } else { 0 },
                "category: {category:?}"
            );
            assert!(state.open_until.is_none());
        }
    }

    #[tokio::test]
    async fn circuit_opens_at_threshold_preserves_ignored_failures_and_resets_after_expiry() {
        let circuit = CircuitBreaker::new();
        let config = policy(2);
        circuit
            .record_failure(&error(GatewayErrorCategory::Network), &config)
            .await;
        circuit
            .record_failure(&error(GatewayErrorCategory::Authentication), &config)
            .await;
        circuit.check().await.expect("below threshold");
        assert_eq!(circuit.state.lock().await.consecutive_failures, 1);
        circuit
            .record_failure(&error(GatewayErrorCategory::Timeout), &config)
            .await;
        let blocked = circuit.check().await.expect_err("open circuit");
        assert_eq!(
            blocked,
            GatewayError::new(
                GatewayErrorCategory::CircuitOpen,
                "OpenAI研究网关因连续失败已暂时熔断",
                true,
                "等待熔断窗口结束；历史记录和手工事实仍可继续使用"
            )
        );
        let deadline = circuit.state.lock().await.open_until;
        circuit
            .record_failure(&error(GatewayErrorCategory::Cancelled), &config)
            .await;
        assert_eq!(circuit.state.lock().await.open_until, deadline);
        circuit.state.lock().await.open_until = Some(Instant::now() - Duration::from_secs(1));
        circuit.check().await.expect("expired window");
        let state = circuit.state.lock().await;
        assert_eq!(state.consecutive_failures, 0);
        assert!(state.open_until.is_none());
    }

    #[tokio::test]
    async fn circuit_success_clears_open_window_and_partial_failure_count() {
        let circuit = CircuitBreaker::new();
        circuit
            .record_failure(&error(GatewayErrorCategory::RateLimit), &policy(1))
            .await;
        assert!(circuit.check().await.is_err());
        circuit.record_success().await;
        circuit.check().await.expect("success resets open circuit");
        circuit
            .record_failure(&error(GatewayErrorCategory::Network), &policy(2))
            .await;
        circuit.record_success().await;
        let state = circuit.state.lock().await;
        assert_eq!(state.consecutive_failures, 0);
        assert!(state.open_until.is_none());
    }

    #[tokio::test]
    async fn circuit_concurrent_failures_are_serialized_and_counter_saturates() {
        let circuit = std::sync::Arc::new(CircuitBreaker::new());
        let mut handles = Vec::new();
        for _ in 0..8 {
            let circuit = circuit.clone();
            handles.push(tokio::spawn(async move {
                circuit
                    .record_failure(
                        &error(GatewayErrorCategory::ProviderUnavailable),
                        &policy(8),
                    )
                    .await;
            }));
        }
        tokio::time::timeout(Duration::from_secs(2), async {
            for handle in handles {
                handle.await.expect("failure task");
            }
        })
        .await
        .expect("bounded concurrent failures");
        assert_eq!(circuit.state.lock().await.consecutive_failures, 8);
        assert!(circuit.check().await.is_err());
        circuit.state.lock().await.consecutive_failures = u32::MAX;
        circuit
            .record_failure(&error(GatewayErrorCategory::Network), &policy(u32::MAX))
            .await;
        assert_eq!(circuit.state.lock().await.consecutive_failures, u32::MAX);
    }
}
