use super::{cancelled_error, CancellationToken};
use crate::GatewayError;
use std::time::Duration;

pub(crate) fn attempt_limit(max_retries: u32) -> u32 {
    max_retries.saturating_add(1)
}

pub(crate) fn retry_delay(base_ms: u64, attempt_number: u32) -> Duration {
    let exponent = attempt_number.saturating_sub(1).min(10);
    Duration::from_millis(base_ms.saturating_mul(1u64 << exponent))
}

pub(crate) async fn wait_retry(
    delay: Duration,
    cancellation: &CancellationToken,
) -> Result<(), GatewayError> {
    tokio::select! {
        _ = tokio::time::sleep(delay) => Ok(()),
        _ = cancellation.cancelled() => Err(cancelled_error()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::{poll_fn, Future};
    use std::task::Poll;

    #[test]
    fn retry_attempt_budget_includes_initial_attempt_and_saturates() {
        assert_eq!(attempt_limit(0), 1);
        assert_eq!(attempt_limit(10), 11);
        assert_eq!(attempt_limit(u32::MAX), u32::MAX);
    }

    #[test]
    fn retry_delay_preserves_exponent_cap_and_millisecond_saturation() {
        for (attempt, millis) in [
            (0, 3),
            (1, 3),
            (2, 6),
            (11, 3072),
            (12, 3072),
            (u32::MAX, 3072),
        ] {
            assert_eq!(retry_delay(3, attempt), Duration::from_millis(millis));
        }
        assert_eq!(retry_delay(0, u32::MAX), Duration::ZERO);
        assert_eq!(retry_delay(u64::MAX, 2), Duration::from_millis(u64::MAX));
    }

    #[tokio::test]
    async fn retry_wait_handles_completion_and_preexisting_cancellation() {
        let token = CancellationToken::new();
        wait_retry(Duration::ZERO, &token)
            .await
            .expect("completed wait");
        token.cancel();
        let error = tokio::time::timeout(
            Duration::from_secs(2),
            wait_retry(Duration::from_secs(60), &token),
        )
        .await
        .expect("bounded cancellation")
        .expect_err("cancelled wait");
        assert_eq!(error, cancelled_error());
    }

    #[tokio::test]
    async fn cancelling_pending_retry_and_dropping_wait_preserve_token() {
        let token = CancellationToken::new();
        {
            let waiting = wait_retry(Duration::from_secs(60), &token);
            tokio::pin!(waiting);
            poll_fn(|cx| {
                assert!(waiting.as_mut().poll(cx).is_pending());
                Poll::Ready(())
            })
            .await;
        }
        assert!(!token.is_cancelled());
        let waiting = wait_retry(Duration::from_secs(60), &token);
        tokio::pin!(waiting);
        poll_fn(|cx| {
            assert!(waiting.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        token.clone().cancel();
        assert_eq!(
            tokio::time::timeout(Duration::from_secs(2), waiting)
                .await
                .expect("bounded wait")
                .expect_err("cancelled"),
            cancelled_error()
        );
    }
}
