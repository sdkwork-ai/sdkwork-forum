pub mod bootstrap;
pub mod repo_impl;
pub mod schema;

use std::sync::Arc;

use sdkwork_communication_forum_service::ForumServiceError;
use sdkwork_database_id::{IdGenerator, SnowflakeIdGenerator};
use sqlx::PgPool;

#[derive(Clone)]
pub struct SqlxForumRepository {
    pool: PgPool,
    id_gen: Arc<dyn IdGenerator>,
}

impl SqlxForumRepository {
    pub fn new(pool: PgPool) -> Self {
        let node_id = std::env::var("SDKWORK_FORUM_SNOWFLAKE_NODE_ID")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(1);
        let gen =
            SnowflakeIdGenerator::new(node_id).expect("invalid SDKWORK_FORUM_SNOWFLAKE_NODE_ID");
        Self {
            pool,
            id_gen: Arc::new(gen),
        }
    }

    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    pub(crate) fn next_id(&self) -> Result<i64, ForumServiceError> {
        let id_str = self
            .id_gen
            .next_id()
            .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        id_str
            .parse::<i64>()
            .map_err(|_| ForumServiceError::internal("invalid snowflake id"))
    }

    pub fn new_placeholder() -> Self {
        Self::new(
            PgPool::connect_lazy("postgres://localhost:5432/sdkwork_ai_dev")
                .expect("Failed to create placeholder pool"),
        )
    }
}
