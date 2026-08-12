use super::DatabaseOptions;
use crate::PersistenceResult;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::time::Duration;

pub(crate) async fn create_pool(options: &DatabaseOptions) -> PersistenceResult<PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(options.max_connections.max(1))
        .acquire_timeout(Duration::from_secs(options.connect_timeout_seconds.max(1)))
        .connect(&options.connection_url)
        .await?)
}
