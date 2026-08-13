use crate::PersistenceResult;
use serde::Serialize;
use serde_json::Value;

pub(crate) fn to_json_value<T: Serialize + ?Sized>(value: &T) -> PersistenceResult<Value> {
    Ok(serde_json::to_value(value)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serializes_without_changing_json_shape() {
        let value = vec!["home", "draw", "away"];
        assert_eq!(
            to_json_value(&value).unwrap(),
            json!(["home", "draw", "away"])
        );
    }
}
