use crate::{PersistenceError, PersistenceResult};
use uuid::Uuid;

pub(super) fn ensure_scope_id(
    label: &str,
    supplied: Option<Uuid>,
    resolved: Uuid,
) -> PersistenceResult<()> {
    if let Some(supplied_id) = supplied {
        if supplied_id != resolved {
            return Err(PersistenceError::InvalidState(format!(
                "{label}层级不一致：提交 {supplied_id}，实际所属 {resolved}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_scope_mismatch() {
        let resolved = Uuid::new_v4();
        assert!(ensure_scope_id("赛事", Some(resolved), resolved).is_ok());
        assert!(ensure_scope_id("赛事", Some(Uuid::new_v4()), resolved).is_err());
    }
}
