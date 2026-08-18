use crate::{PersistenceError, PersistenceResult};

pub(crate) fn validate_entity_type(value: &str) -> PersistenceResult<()> {
    if matches!(value, "team" | "player" | "coach") {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(format!(
            "不支持的实体类型：{value}"
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::validate_entity_type;

    #[test]
    fn reference_entity_type_contract_is_strict() {
        assert!(validate_entity_type("team").is_ok());
        assert!(validate_entity_type("player").is_ok());
        assert!(validate_entity_type("coach").is_ok());
        assert!(matches!(
            validate_entity_type("match"),
            Err(crate::PersistenceError::InvalidState(message)) if message == "不支持的实体类型：match"
        ));
    }
}
