use football_research_gateway::CancellationToken;
use std::{
    collections::{hash_map::Entry, HashMap},
    sync::{Arc, Mutex, MutexGuard},
};

/// Owns active AI workspace request IDs for one application instance.
#[derive(Clone, Default)]
pub struct AiWorkspaceRequestRegistry {
    requests: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

/// Releases the request ID and cancels its token when the request future ends.
/// This guard is deliberately not cloneable: each registration has one owner.
pub struct AiWorkspaceRequestLease {
    registry: AiWorkspaceRequestRegistry,
    request_id: String,
    cancellation: CancellationToken,
}

impl AiWorkspaceRequestRegistry {
    /// The caller validates the ID; duplicates retain the original token.
    pub fn register(&self, request_id: &str) -> Result<AiWorkspaceRequestLease, &'static str> {
        let mut requests = self.lock();
        match requests.entry(request_id.to_owned()) {
            Entry::Occupied(_) => Err("AI问答请求 ID 已在使用"),
            Entry::Vacant(entry) => {
                let cancellation = CancellationToken::new();
                entry.insert(cancellation.clone());
                Ok(AiWorkspaceRequestLease {
                    registry: self.clone(),
                    request_id: request_id.to_owned(),
                    cancellation,
                })
            }
        }
    }

    /// Repeated cancellation succeeds while the request still owns its ID.
    pub fn cancel(&self, request_id: &str) -> bool {
        let token = self.lock().get(request_id).cloned();
        if let Some(token) = token {
            token.cancel();
            true
        } else {
            false
        }
    }

    // Only short map operations hold this lock; no guard crosses an await.
    // Recovery also keeps lease cleanup safe during panic unwinding.
    fn lock(&self) -> MutexGuard<'_, HashMap<String, CancellationToken>> {
        self.requests
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl AiWorkspaceRequestLease {
    pub fn cancellation(&self) -> &CancellationToken {
        &self.cancellation
    }
}

impl Drop for AiWorkspaceRequestLease {
    fn drop(&mut self) {
        self.registry.lock().remove(&self.request_id);
        self.cancellation.cancel();
    }
}
