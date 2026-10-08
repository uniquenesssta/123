use crate::{PersistenceError, PersistenceResult};

pub(super) fn ensure_fingerprint(
    label: &str,
    idempotency_key: &str,
    existing: &str,
    expected: &str,
) -> PersistenceResult<()> {
    if existing == expected {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(format!(
            "{label}幂等键{idempotency_key}已存在但载荷不同"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::ensure_fingerprint;
    use crate::PersistenceError;

    #[test]
    fn idempotency_conflict_keeps_exact_error_and_compares_full_fingerprint() {
        ensure_fingerprint("时间审计", "key", "sha-abc", "sha-abc").unwrap();
        for changed in ["sha-ab", "SHA-ABC", "sha-abc "] {
            assert!(
                matches!(ensure_fingerprint("时间审计", "key", "sha-abc", changed),
                Err(PersistenceError::InvalidState(message)) if message == "时间审计幂等键key已存在但载荷不同")
            );
        }
    }
}
