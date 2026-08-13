use crate::{pool::create_pool, DatabaseOptions, PersistenceResult};
use sqlx::postgres::PgPool;

#[derive(Clone)]
pub struct PostgresStore {
    pub(crate) pool: PgPool,
    redacted_url: String,
}

impl PostgresStore {
    pub async fn connect(options: &DatabaseOptions) -> PersistenceResult<Self> {
        let redacted_url = options.redacted_url();
        Ok(Self {
            pool: create_pool(options).await?,
            redacted_url,
        })
    }

    pub fn redacted_url(&self) -> &str {
        &self.redacted_url
    }

    pub async fn close(&self) {
        self.pool.close().await;
    }
}
