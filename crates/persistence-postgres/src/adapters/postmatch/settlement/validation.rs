use crate::{PersistenceError, PersistenceResult};
use serde_json::Value;

pub(super) fn validate_probabilities(home: f64, draw: f64, away: f64) -> PersistenceResult<()> {
    let values = [home, draw, away];
    let sum = values.iter().sum::<f64>();
    if values
        .iter()
        .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        && sum.is_finite()
        && (sum - 1.0).abs() <= 0.0001
    {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(
            "正式结算样本的胜平负概率非法或未归一化".to_string(),
        ))
    }
}

pub(super) fn json_text(value: &Value, key: &str) -> PersistenceResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .ok_or_else(|| PersistenceError::InvalidState(format!("预测评价缺少字段 {key}")))
}

pub(super) fn json_f64(value: &Value, key: &str) -> PersistenceResult<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .filter(|item| item.is_finite())
        .ok_or_else(|| PersistenceError::InvalidState(format!("预测评价字段 {key} 非法")))
}

pub(super) fn required<T>(value: Option<T>, message: &str) -> PersistenceResult<T> {
    value.ok_or_else(|| PersistenceError::InvalidState(message.to_string()))
}
