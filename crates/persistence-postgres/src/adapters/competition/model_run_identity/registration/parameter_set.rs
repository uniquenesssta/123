use crate::{sha256_json, PersistenceError, PersistenceResult};
use serde_json::Value;
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct ExistingParameterSetRow {
    id: Uuid,
    definition_sha256: String,
}

pub(super) async fn register_parameter_set(
    tx: &mut Transaction<'_, Postgres>,
    model_version_id: Uuid,
    parameter_version: &str,
    parameters: &Value,
) -> PersistenceResult<Uuid> {
    let generated_id = Uuid::new_v4();
    let definition_hash = sha256_json(parameters)?;
    let inserted = sqlx::query_scalar(
        r#"
        INSERT INTO model.parameter_sets (
            id, model_version_id, parameter_version, name,
            definition, definition_sha256, status
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active')
        ON CONFLICT (model_version_id, parameter_version) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(generated_id)
    .bind(model_version_id)
    .bind(parameter_version)
    .bind(parameter_version)
    .bind(parameters)
    .bind(&definition_hash)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(inserted_id) = inserted {
        return Ok(inserted_id);
    }

    let existing = sqlx::query_as::<_, ExistingParameterSetRow>(
        r#"
        SELECT id, definition_sha256
        FROM model.parameter_sets
        WHERE model_version_id = $1 AND parameter_version = $2
        "#,
    )
    .bind(model_version_id)
    .bind(parameter_version)
    .fetch_one(&mut **tx)
    .await?;

    if existing.definition_sha256 != definition_hash {
        return Err(PersistenceError::InvalidState(format!(
            "参数版本 {parameter_version} 已存在但内容不同；请创建新参数版本"
        )));
    }
    Ok(existing.id)
}
