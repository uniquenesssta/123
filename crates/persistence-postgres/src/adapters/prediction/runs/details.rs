use crate::{PersistenceError, PersistenceResult};
use serde_json::Value;
use sqlx::Transaction;
use uuid::Uuid;

pub(super) async fn save_model_details(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    run_id: Uuid,
    payload: &Value,
) -> PersistenceResult<()> {
    if let Some(modules) = payload.get("modules").and_then(Value::as_object) {
        for (module_key, details) in modules {
            let side = module_key
                .split_once('_')
                .map(|(side_value, _)| side_value.to_string());
            sqlx::query(
                r#"
                INSERT INTO model.run_modules (
                    run_id, module_key, side, raw_score, confidence,
                    effective_score, multiplier, details
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(run_id)
            .bind(module_key)
            .bind(side)
            .bind(details.get("raw_score").and_then(Value::as_f64))
            .bind(details.get("confidence").and_then(Value::as_f64))
            .bind(details.get("effective_score").and_then(Value::as_f64))
            .bind(details.get("multiplier").and_then(Value::as_f64))
            .bind(details)
            .execute(&mut **tx)
            .await?;
        }
    }

    if let Some(scorelines) = payload.get("scorelines").and_then(Value::as_array) {
        for item in scorelines {
            let home_goals = i16::try_from(required_i64(item, "goals_a")?).map_err(|_| {
                PersistenceError::InvalidState("goals_a 超出 smallint 范围".to_string())
            })?;
            let away_goals = i16::try_from(required_i64(item, "goals_b")?).map_err(|_| {
                PersistenceError::InvalidState("goals_b 超出 smallint 范围".to_string())
            })?;
            let rank = i16::try_from(required_i64(item, "rank")?).map_err(|_| {
                PersistenceError::InvalidState("rank 超出 smallint 范围".to_string())
            })?;
            let probability = required_f64(item, "probability")?;
            let cumulative_probability = required_f64(item, "cumulative_probability")?;
            let route = item
                .get("route")
                .and_then(Value::as_str)
                .unwrap_or("未分类");

            sqlx::query(
                r#"
                INSERT INTO model.run_scorelines (
                    run_id, home_goals, away_goals, probability,
                    rank, cumulative_probability, route, details
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(run_id)
            .bind(home_goals)
            .bind(away_goals)
            .bind(probability)
            .bind(rank)
            .bind(cumulative_probability)
            .bind(route)
            .bind(item)
            .execute(&mut **tx)
            .await?;
        }
    }
    Ok(())
}

fn required_i64(value: &Value, key: &str) -> PersistenceResult<i64> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少整数：{key}")))
}

fn required_f64(value: &Value, key: &str) -> PersistenceResult<f64> {
    value
        .get(key)
        .and_then(Value::as_f64)
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少数值：{key}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn model_detail_scalars_preserve_integer_numeric_and_missing_errors() {
        for value in [i64::MIN, 0, i64::MAX] {
            assert_eq!(
                required_i64(&json!({"rank": value}), "rank").unwrap(),
                value
            );
        }
        for value in [json!(1.5), json!("1"), Value::Null, json!(u64::MAX)] {
            assert!(
                matches!(required_i64(&json!({"rank": value}), "rank"), Err(PersistenceError::InvalidState(m)) if m == "缺少整数：rank")
            );
        }
        assert_eq!(
            required_f64(&json!({"probability": 1}), "probability").unwrap(),
            1.0
        );
        assert_eq!(
            required_f64(&json!({"probability": 0.125}), "probability").unwrap(),
            0.125
        );
        for value in [
            json!({}),
            json!({"probability": "0.1"}),
            json!({"probability": null}),
        ] {
            assert!(
                matches!(required_f64(&value, "probability"), Err(PersistenceError::InvalidState(m)) if m == "缺少数值：probability")
            );
        }
    }
}
