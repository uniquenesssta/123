use crate::PersistenceError;

pub(crate) fn invalid_state(message: impl Into<String>) -> PersistenceError {
    PersistenceError::InvalidState(message.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_invalid_state_message() {
        let error = invalid_state("映射状态无效");
        assert!(
            matches!(error, PersistenceError::InvalidState(message) if message == "映射状态无效")
        );
    }
}
