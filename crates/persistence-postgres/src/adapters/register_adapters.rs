use crate::{DatabaseOptions, PersistenceResult, PostgresStore};

pub async fn register_adapters(options: &DatabaseOptions) -> PersistenceResult<PostgresStore> {
    PostgresStore::connect(options).await
}
