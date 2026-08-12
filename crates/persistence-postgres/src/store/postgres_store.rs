use crate::{pool::create_pool, DatabaseOptions, PersistenceResult};
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PostgresStore {
    pub(crate) pool: PgPool,
}

impl PostgresStore {
    pub async fn connect(options: &DatabaseOptions) -> PersistenceResult<Self> {
        Ok(Self {
            pool: create_pool(options).await?,
        })
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
