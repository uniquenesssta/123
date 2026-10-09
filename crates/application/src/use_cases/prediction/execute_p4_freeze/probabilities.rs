use crate::{ApplicationError, ApplicationResult};
use football_domain::SnapshotProbabilityDraft;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};

pub(super) fn snapshot_probabilities(
    payload: &Value,
) -> ApplicationResult<Vec<SnapshotProbabilityDraft>> {
    let matrices = payload
        .get("matrices")
        .and_then(Value::as_object)
        .ok_or_else(|| ApplicationError::Validation("外部模型输出缺少 matrices".to_string()))?;
    if matrices.is_empty() {
        return Err(ApplicationError::Validation(
            "外部模型输出至少需要一条概率矩阵".to_string(),
        ));
    }
    let clean_sheet_home = payload.get("clean_sheet_a").and_then(Value::as_f64);
    let clean_sheet_away = payload.get("clean_sheet_b").and_then(Value::as_f64);
    matrices
        .iter()
        .map(|(chain_key, matrix)| {
            let outcome = matrix
                .get("outcome")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    ApplicationError::Validation(format!("外部模型矩阵 {chain_key} 缺少 outcome"))
                })?;
            let scorelines = matrix
                .get("scorelines")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    ApplicationError::Validation(format!(
                        "外部模型矩阵 {chain_key} 缺少 scorelines"
                    ))
                })?;
            let matrix_cell_count = u16::try_from(scorelines.len()).map_err(|_| {
                ApplicationError::Validation(format!(
                    "外部模型矩阵 {chain_key} 的比分单元数量超出支持范围"
                ))
            })?;
            if matrix_cell_count == 0 {
                return Err(ApplicationError::Validation(format!(
                    "外部模型矩阵 {chain_key} 不得为空"
                )));
            }
            let is_formal = matrix
                .get("formal")
                .and_then(Value::as_bool)
                .unwrap_or(chain_key == "full");
            Ok(SnapshotProbabilityDraft {
                chain_key: chain_key.to_string(),
                home_win: required_probability(outcome, "a_win", chain_key)?,
                draw: required_probability(outcome, "draw", chain_key)?,
                away_win: required_probability(outcome, "b_win", chain_key)?,
                btts: matrix.get("btts").and_then(Value::as_f64),
                over_2_5: matrix.get("over_2_5").and_then(Value::as_f64),
                clean_sheet_home: is_formal.then_some(clean_sheet_home).flatten(),
                clean_sheet_away: is_formal.then_some(clean_sheet_away).flatten(),
                matrix_sha256: sha256_value(matrix)?,
                matrix_cell_count,
                metadata: json!({
                    "formal": is_formal,
                    "provider_owned_topology": true
                }),
            })
        })
        .collect()
}

fn required_probability(
    outcome: &Map<String, Value>,
    key: &str,
    chain_key: &str,
) -> ApplicationResult<f64> {
    outcome
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
        .ok_or_else(|| {
            ApplicationError::Validation(format!("外部模型矩阵 {chain_key} 的概率字段 {key} 无效"))
        })
}

fn sha256_value(value: &Value) -> ApplicationResult<String> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    fn matrix() -> Value {
        json!({"outcome":{"a_win":0.4,"draw":0.3,"b_win":0.3}, "scorelines":[{"a":1,"b":0,"probability":0.4}], "btts":0.5,"over_2_5":0.45})
    }
    #[test]
    fn probability_projection_preserves_provider_topology_formal_flags_and_matrix_bytes() {
        let mut payload = json!({"matrices":{"full":matrix(),"custom":matrix(),"explicit":matrix()}, "clean_sheet_a":0.2,"clean_sheet_b":0.1});
        payload["matrices"]["full"]["formal"] = json!(false);
        payload["matrices"]["explicit"]["formal"] = json!(true);
        let probabilities = snapshot_probabilities(&payload).unwrap();
        assert_eq!(probabilities.len(), 3);
        for probability in &probabilities {
            let formal = probability.chain_key == "explicit";
            assert_eq!(
                probability.metadata,
                json!({"formal":formal,"provider_owned_topology":true})
            );
            assert_eq!(probability.home_win, 0.4);
            assert_eq!(probability.btts, Some(0.5));
            assert_eq!(probability.clean_sheet_home, formal.then_some(0.2));
            assert_eq!(probability.clean_sheet_away, formal.then_some(0.1));
            assert_eq!(probability.matrix_cell_count, 1);
            let bytes = serde_json::to_vec(&payload["matrices"][&probability.chain_key]).unwrap();
            assert_eq!(
                probability.matrix_sha256,
                hex::encode(Sha256::digest(bytes))
            );
        }
        payload["matrices"]["full"]
            .as_object_mut()
            .unwrap()
            .remove("formal");
        assert_eq!(
            snapshot_probabilities(&payload)
                .unwrap()
                .iter()
                .find(|p| p.chain_key == "full")
                .unwrap()
                .clean_sheet_home,
            Some(0.2)
        );
    }
    #[test]
    fn probability_projection_rejects_missing_invalid_and_oversized_provider_matrices() {
        for bad in [
            json!({}),
            json!({"matrices":[]}),
            json!({"matrices":{}}),
            json!({"matrices":{"x":{"outcome":{},"scorelines":[1]}}}),
            json!({"matrices":{"x":{"outcome":{"a_win":1.1,"draw":0.0,"b_win":0.0},"scorelines":[1]}}}),
        ] {
            assert!(matches!(
                snapshot_probabilities(&bad),
                Err(ApplicationError::Validation(_))
            ));
        }
        let mut payload = json!({"matrices":{"x":matrix()}});
        for count in [0, usize::from(u16::MAX) + 1] {
            payload["matrices"]["x"]["scorelines"] = Value::Array(vec![Value::Null; count]);
            assert!(matches!(
                snapshot_probabilities(&payload),
                Err(ApplicationError::Validation(_))
            ));
        }
        payload["matrices"]["x"]["scorelines"] = json!([null]);
        payload["matrices"]["x"]["outcome"] = json!({"a_win":1.0,"draw":0.0,"b_win":0.0});
        assert_eq!(snapshot_probabilities(&payload).unwrap()[0].home_win, 1.0);
    }
}
