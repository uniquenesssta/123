use super::{sha256_json, PersistenceError, PersistenceResult, PostgresStore};
use football_model_api::ModelDescriptor;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Row, Transaction};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistration {
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
}

impl PostgresStore {
    pub async fn register_model(
        &self,
        descriptor: &ModelDescriptor,
        model_version: &str,
        parameter_version: &str,
        parameters: &Value,
    ) -> PersistenceResult<ModelRegistration> {
        let mut tx = self.pool.begin().await?;
        let registration = register_model_in_tx(
            &mut tx,
            descriptor,
            model_version,
            parameter_version,
            parameters,
        )
        .await?;
        tx.commit().await?;
        Ok(registration)
    }
}

pub(crate) async fn register_model_in_tx(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    descriptor: &ModelDescriptor,
    model_version: &str,
    parameter_version: &str,
    parameters: &Value,
) -> PersistenceResult<ModelRegistration> {
    let model_id = upsert_model_definition(tx, descriptor).await?;
    let model_version_id = register_model_version(tx, model_id, descriptor, model_version).await?;
    let parameter_set_id =
        register_parameter_set(tx, model_version_id, parameter_version, parameters).await?;
    Ok(ModelRegistration {
        model_version_id,
        parameter_set_id,
    })
}

async fn upsert_model_definition(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    descriptor: &ModelDescriptor,
) -> PersistenceResult<Uuid> {
    let generated_id = Uuid::new_v4();
    let returned: Uuid = sqlx::query_scalar(
        r#"
        INSERT INTO model.definitions (id, model_key, display_name, description)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (model_key) DO UPDATE SET
            display_name = EXCLUDED.display_name,
            description = EXCLUDED.description,
            is_active = true
        RETURNING id
        "#,
    )
    .bind(generated_id)
    .bind(&descriptor.model_id)
    .bind(&descriptor.display_name)
    .bind(format!(
        "{}；引擎 {}",
        descriptor.display_name, descriptor.engine_version
    ))
    .fetch_one(&mut **tx)
    .await?;
    Ok(returned)
}

async fn register_model_version(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    model_id: Uuid,
    descriptor: &ModelDescriptor,
    model_version: &str,
) -> PersistenceResult<Uuid> {
    let generated_id = Uuid::new_v4();
    let inserted: Option<Uuid> = sqlx::query_scalar(
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

    let row = sqlx::query(
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

    let existing_id: Uuid = row.try_get("id")?;
    let existing_engine: String = row.try_get("engine_version")?;
    let existing_input: String = row.try_get("input_schema_version")?;
    let existing_output: String = row.try_get("output_schema_version")?;
    if existing_engine != descriptor.engine_version
        || existing_input != descriptor.input_schema_version
        || existing_output != descriptor.output_schema_version
    {
        return Err(PersistenceError::InvalidState(format!(
            "模型版本 {model_version} 已存在但引擎或 Schema 不一致；请创建新模型版本"
        )));
    }
    Ok(existing_id)
}

async fn register_parameter_set(
    tx: &mut Transaction<'_, sqlx::Postgres>,
    model_version_id: Uuid,
    parameter_version: &str,
    parameters: &Value,
) -> PersistenceResult<Uuid> {
    let generated_id = Uuid::new_v4();
    let definition_hash = sha256_json(parameters)?;
    let inserted: Option<Uuid> = sqlx::query_scalar(
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

    let row = sqlx::query(
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

    let existing_id: Uuid = row.try_get("id")?;
    let existing_hash: String = row.try_get("definition_sha256")?;
    if existing_hash != definition_hash {
        return Err(PersistenceError::InvalidState(format!(
            "参数版本 {parameter_version} 已存在但内容不同；请创建新参数版本"
        )));
    }
    Ok(existing_id)
}
