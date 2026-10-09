use crate::{GatewayError, GatewayErrorCategory};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Clone, Default)]
pub struct CancellationToken {
    inner: Arc<CancellationState>,
}

#[derive(Default)]
struct CancellationState {
    cancelled: AtomicBool,
    notify: Notify,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn cancel(&self) {
        if !self.inner.cancelled.swap(true, Ordering::SeqCst) {
            self.inner.notify.notify_waiters();
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }

    pub async fn cancelled(&self) {
        let notified = self.inner.notify.notified();
        if self.is_cancelled() {
            return;
        }
        notified.await;
    }
}

pub(crate) fn cancelled_error() -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::Cancelled,
        "OpenAI研究任务已取消",
        false,
        "可从已保存的研究任务重新发起；已存在证据和快照不会被删除",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::future::{poll_fn, Future};
    use std::task::Poll;
    use std::time::Duration;

    #[test]
    fn cancellation_clones_share_state_and_independent_tokens_do_not() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<CancellationToken>();
        let original = CancellationToken::default();
        let clone = original.clone();
        let independent = CancellationToken::new();
        assert!(!clone.is_cancelled());
        clone.cancel();
        original.cancel();
        assert!(original.is_cancelled());
        assert!(clone.is_cancelled());
        assert!(!independent.is_cancelled());
    }

    #[tokio::test]
    async fn cancellation_before_first_poll_completes_every_later_waiter() {
        let token = CancellationToken::new();
        let first = token.cancelled();
        let second = token.cancelled();
        token.cancel();
        tokio::time::timeout(Duration::from_secs(2), async {
            first.await;
            second.await;
            token.cancelled().await;
        })
        .await
        .expect("cancelled waits do not hang");
    }

    #[tokio::test]
    async fn cancellation_notifies_all_registered_and_unpolled_waiters() {
        let token = CancellationToken::new();
        let registered = token.cancelled();
        tokio::pin!(registered);
        poll_fn(|cx| {
            assert!(registered.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        // Exercise the creation/check/await race window of the production token.
        let unpolled = token.inner.notify.notified();
        assert!(!token.is_cancelled());
        let clone = token.clone();
        let second = clone.cancelled();
        tokio::pin!(second);
        poll_fn(|cx| {
            assert!(second.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        clone.cancel();
        tokio::time::timeout(Duration::from_secs(2), async {
            registered.await;
            second.await;
            unpolled.await;
        })
        .await
        .expect("all waiters notified");
    }

    #[tokio::test]
    async fn dropping_one_cancellation_waiter_does_not_consume_other_wakeups() {
        let token = CancellationToken::new();
        {
            let dropped = token.cancelled();
            tokio::pin!(dropped);
            poll_fn(|cx| {
                assert!(dropped.as_mut().poll(cx).is_pending());
                Poll::Ready(())
            })
            .await;
        }
        assert!(!token.is_cancelled());
        token.cancel();
        token.cancel();
        tokio::time::timeout(Duration::from_secs(2), token.cancelled())
            .await
            .expect("later waiter notified");
    }
}
