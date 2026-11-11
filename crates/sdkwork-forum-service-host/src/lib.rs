use std::sync::Arc;

use sdkwork_communication_forum_repository_sqlx::SqlxForumRepository;
use sdkwork_communication_forum_service::value_objects::ForumRequestContext;
use sdkwork_communication_forum_service::ForumService;
use sdkwork_database_ops::DatabaseOpsService;
use sdkwork_database_spi::{DefaultDatabaseModule, LocaleTag, SeedProfile};
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_forum_database_host::{bootstrap_forum_database, bootstrap_forum_database_from_env};
use sqlx::PgPool;
use tracing;

mod ports;

pub struct ForumServiceHost {
    service: ForumService<SqlxForumRepository>,
    pool: DatabasePool,
    pg_pool: PgPool,
    iam_pool: Option<PgPool>,
    database_module: Arc<DefaultDatabaseModule>,
}

impl ForumServiceHost {
    /// Builds the host from the environment, including the outbound ports.
    ///
    /// # Errors
    ///
    /// Fails when the database cannot be bootstrapped, or when
    /// `SDKWORK_FORUM_SEARCH_MODE=http` was selected without its endpoint and
    /// service credential. The default in-process ports never fail on missing
    /// configuration (MODULE_SPEC.md §4.2.3).
    pub async fn new() -> Result<Self, String> {
        let _ = dotenvy::dotenv();

        tracing::info!("Connecting to database...");

        let database_host = bootstrap_forum_database_from_env().await?;

        Self::from_database_host(database_host)
    }

    /// Build the host against a caller-provided database pool so the platform
    /// cloud gateway can share its process-wide PostgreSQL pool.
    pub async fn from_database_pool(pool: DatabasePool) -> Result<Self, String> {
        let database_host = bootstrap_forum_database(pool).await?;
        Self::from_database_host(database_host)
    }

    fn from_database_host(
        database_host: sdkwork_forum_database_host::ForumDatabaseHost,
    ) -> Result<Self, String> {
        let pool = database_host.pool().clone();
        let database_module = database_host.module();

        let pg_pool = pool
            .as_postgres()
            .ok_or_else(|| "Expected PostgreSQL pool for forum service".to_string())?
            .clone();

        let iam_pool = if iam_enabled_from_env() {
            Some(load_iam_pool(&pg_pool))
        } else {
            None
        };

        tracing::info!("Database connected successfully");

        // Sibling-module ports are composed on this host pool (MODULE_SPEC.md
        // §4.2): Drive and Search are called in-process by default, and the only
        // failure mode here is an explicitly configured split deployment whose
        // endpoint or credential is missing. That fails now, at startup.
        let drive_port = ports::build_drive_port(&pg_pool);
        let search_port = ports::build_search_port(&pg_pool)?;

        let repository = SqlxForumRepository::new(pg_pool.clone());
        let service = ForumService::new_with_ports(
            repository,
            drive_port,
            search_port,
            ports::build_notification_port(),
        );

        tracing::info!("Forum service initialized");

        Ok(Self {
            service,
            pool,
            pg_pool,
            iam_pool,
            database_module,
        })
    }

    pub fn service(&self) -> &ForumService<SqlxForumRepository> {
        &self.service
    }

    pub fn database_pool(&self) -> DatabasePool {
        self.pool.clone()
    }

    pub fn postgres_pool(&self) -> &PgPool {
        &self.pg_pool
    }

    pub fn iam_pool(&self) -> Option<&PgPool> {
        self.iam_pool.as_ref()
    }

    pub fn database_module(&self) -> Arc<DefaultDatabaseModule> {
        self.database_module.clone()
    }

    pub fn database_ops_service(&self) -> DatabaseOpsService {
        DatabaseOpsService::new(self.pool.clone(), self.database_module.clone())
    }

    pub fn build_request_context(
        &self,
        tenant_id: i64,
        organization_id: i64,
        user_id: i64,
    ) -> ForumRequestContext {
        ForumRequestContext::new(tenant_id, organization_id, user_id)
    }
}

fn iam_enabled_from_env() -> bool {
    matches!(
        std::env::var("SDKWORK_FORUM_IAM_ENABLED").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

fn load_iam_pool(forum_pool: &PgPool) -> PgPool {
    forum_pool.clone()
}

pub fn build_forum_service() -> ForumService<SqlxForumRepository> {
    ForumService::new(SqlxForumRepository::new_placeholder())
}

pub fn default_seed_locale() -> LocaleTag {
    LocaleTag(std::env::var("SDKWORK_DATABASE_SEED_LOCALE").unwrap_or_else(|_| "zh-CN".to_string()))
}

pub fn default_seed_profile() -> SeedProfile {
    SeedProfile(
        std::env::var("SDKWORK_DATABASE_SEED_PROFILE").unwrap_or_else(|_| "standard".to_string()),
    )
}
