use super::AiWorkspaceRequestRegistry;
use std::sync::Barrier;

#[test]
fn registry_clones_share_requests_but_application_instances_are_isolated() {
    let registry = AiWorkspaceRequestRegistry::default();
    let same_instance = registry.clone();
    let other_instance = AiWorkspaceRequestRegistry::default();
    let lease = registry.register("request").unwrap();

    assert!(!registry.cancel("unknown"));
    assert!(!other_instance.cancel("request"));
    assert!(same_instance.cancel("request"));
    assert!(registry.cancel("request"));
    assert!(lease.cancellation().is_cancelled());
    drop(lease);
    assert!(!same_instance.cancel("request"));
}

#[test]
fn duplicate_registration_preserves_the_running_requests_cancellation() {
    let registry = AiWorkspaceRequestRegistry::default();
    let lease = registry.register("request").unwrap();
    let duplicate = registry.register("request");

    assert_eq!(duplicate.err(), Some("AI问答请求 ID 已在使用"));
    assert!(!lease.cancellation().is_cancelled());
    assert!(registry.cancel("request"));
    assert!(lease.cancellation().is_cancelled());
}

#[test]
fn cancellation_and_cleanup_leave_other_requests_active() {
    let registry = AiWorkspaceRequestRegistry::default();
    let first = registry.register("first").unwrap();
    let second = registry.register("second").unwrap();

    assert!(registry.cancel("first"));
    assert!(first.cancellation().is_cancelled());
    assert!(!second.cancellation().is_cancelled());
    drop(first);
    assert!(!registry.cancel("first"));
    assert!(!second.cancellation().is_cancelled());
    assert!(registry.cancel("second"));
    assert!(second.cancellation().is_cancelled());
}

#[test]
fn completed_request_releases_its_id_for_a_fresh_uncancelled_request() {
    let registry = AiWorkspaceRequestRegistry::default();
    let finished = registry.register("request").unwrap();
    let old_token = finished.cancellation().clone();
    drop(finished);

    assert!(old_token.is_cancelled());
    assert!(!registry.cancel("request"));
    let next = registry.register("request").unwrap();
    old_token.cancel();
    assert!(!next.cancellation().is_cancelled());
    assert!(registry.cancel("request"));
    assert!(next.cancellation().is_cancelled());
}

#[test]
fn preparation_error_releases_the_request_without_reaching_execution() {
    fn prepare(registry: &AiWorkspaceRequestRegistry) -> Result<(), &'static str> {
        let _lease = registry.register("request")?;
        Err("profile unavailable")
    }

    let registry = AiWorkspaceRequestRegistry::default();
    assert_eq!(prepare(&registry), Err("profile unavailable"));
    assert!(!registry.cancel("request"));
    assert!(registry.register("request").is_ok());
}

#[tokio::test]
async fn aborting_a_request_task_cancels_its_token_and_releases_its_id() {
    let registry = AiWorkspaceRequestRegistry::default();
    let task_registry = registry.clone();
    let (ready, receiver) = tokio::sync::oneshot::channel();
    let task = tokio::spawn(async move {
        let lease = task_registry.register("request").unwrap();
        assert!(ready.send(lease.cancellation().clone()).is_ok());
        std::future::pending::<()>().await;
        drop(lease);
    });

    let token = receiver.await.unwrap();
    assert!(!token.is_cancelled());
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert!(token.is_cancelled());
    assert!(!registry.cancel("request"));
    assert!(registry.register("request").is_ok());
}

#[test]
fn panic_unwinding_cleans_up_the_request_lease() {
    let registry = AiWorkspaceRequestRegistry::default();
    let lease = registry.register("request").unwrap();
    let token = lease.cancellation().clone();
    let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || {
        let _lease = lease;
        panic!("request future panicked");
    }));

    assert!(panic.is_err());
    assert!(token.is_cancelled());
    assert!(!registry.cancel("request"));
    assert!(registry.register("request").is_ok());
}

#[test]
fn concurrent_duplicate_ids_have_one_owner_and_one_cancellable_token() {
    let registry = AiWorkspaceRequestRegistry::default();
    let barrier = Barrier::new(2);
    let results = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            barrier.wait();
            registry.register("request")
        });
        let second = scope.spawn(|| {
            barrier.wait();
            registry.register("request")
        });
        [first.join().unwrap(), second.join().unwrap()]
    });

    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    assert_eq!(
        results
            .iter()
            .filter_map(|result| result.as_ref().err())
            .copied()
            .collect::<Vec<_>>(),
        vec!["AI问答请求 ID 已在使用"]
    );
    assert!(registry.cancel("request"));
    assert!(results
        .iter()
        .filter_map(|result| result.as_ref().ok())
        .all(|lease| lease.cancellation().is_cancelled()));
    drop(results);
    assert!(!registry.cancel("request"));
    assert!(registry.register("request").is_ok());
}
