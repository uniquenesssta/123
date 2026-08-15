use crate::PersistenceResult;
use football_model_api::ModelDescriptor;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub(super) async fn upsert_model_definition(
    tx: &mut Transaction<'_, Postgres>,
    descriptor: &ModelDescriptor,
) -> PersistenceResult<Uuid> {
    let generated_id = Uuid::new_v4();
    let returned = sqlx::query_scalar(
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
