use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("PostgreSQL 连接或查询失败：{0}")]
    Sqlx(#[from] sqlx::Error),
    #[error("数据库迁移失败：{0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("数据序列化失败：{0}")]
    Serialization(#[from] serde_json::Error),
    #[error("数据库数据不完整：{0}")]
    InvalidState(String),
    #[error("没有匹配到可用的赛事规则包和模型路由")]
    RouteNotFound,
}

pub type PersistenceResult<T> = Result<T, PersistenceError>;
