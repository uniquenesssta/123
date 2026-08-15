use super::{
    definition::upsert_model_definition, parameter_set::register_parameter_set,
    record::ModelRegistration, version::register_model_version,
};
use crate::{PersistenceResult, PostgresStore};
use football_model_api::ModelDescriptor;
use serde_json::Value;
use sqlx::{Postgres, Transaction};

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
    tx: &mut Transaction<'_, Postgres>,
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
