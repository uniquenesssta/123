use crate::{PersistenceError, PersistenceResult};
use football_model_api::ModelDescriptor;
use sqlx::{FromRow, Postgres, Transaction};
use uuid::Uuid;

#[derive(Debug, FromRow)]
struct ExistingModelVersionRow {
    id: Uuid,
    engine_version: String,
    input_schema_version: String,
    output_schema_version: String,
}

pub(super) async fn register_model_version(
    tx: &mut Transaction<'_, Postgres>,
    model_id: Uuid,
    descriptor: &ModelDescriptor,
    model_version: &str,
) -> PersistenceResult<Uuid> {
    let generated_id = Uuid::new_v4();
    let inserted = sqlx::query_scalar(
        r#"
        INSERT INTO model.versions (
            id, model_id, version, engine_version,
            input_schema_version, output_schema_version, status
        ) VALUES ($1, $2, $3, $4, $5, $6, 'active')
        ON CONFLICT (model_id, version) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(generated_id)
    .bind(model_id)
    .bind(model_version)
    .bind(&descriptor.engine_version)
    .bind(&descriptor.input_schema_version)
    .bind(&descriptor.output_schema_version)
    .fetch_optional(&mut **tx)
    .await?;

    if let Some(inserted_id) = inserted {
        return Ok(inserted_id);
    }

    let existing = sqlx::query_as::<_, ExistingModelVersionRow>(
        r#"
        SELECT id, engine_version, input_schema_version, output_schema_version
        FROM model.versions
        WHERE model_id = $1 AND version = $2
        "#,
    )
    .bind(model_id)
    .bind(model_version)
    .fetch_one(&mut **tx)
    .await?;

    if existing.engine_version != descriptor.engine_version
        || existing.input_schema_version != descriptor.input_schema_version
        || existing.output_schema_version != descriptor.output_schema_version
    {
        return Err(PersistenceError::InvalidState(format!(
            "模型版本 {model_version} 已存在但引擎或 Schema 不一致；请创建新模型版本"
        )));
    }
    Ok(existing.id)
}
