use std::sync::Arc;

use sdkwork_api_forum_assembly::assemble_api_router;
use sdkwork_database_ops_http::{attach_ops_routes, DatabaseOpsHttpState};
use sdkwork_iam_web_adapter::{
    build_web_framework_builder, iam_web_request_context_resolver_from_database_pool_for_audiences,
    iam_web_request_context_resolver_from_env, IamAuditEmitter, IamSecurityEventEmitter,
};
use sdkwork_web_bootstrap::{infra_public_path_prefixes, ApiModuleRegistry, ComposedApiAssembly};

const APPLICATION_ID: &str = "sdkwork-forum";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    sdkwork_web_bootstrap::init_tracing_from_env();

    let assembly = assemble_api_router().await;
    let ops_state = DatabaseOpsHttpState::new_with_default_auth(
        assembly.database_pool.clone(),
        assembly.database_module,
        assembly.seed_locale,
        assembly.seed_profile,
    );
    let environment = std::env::var("SDKWORK_ENVIRONMENT")
        .or_else(|_| std::env::var("SDKWORK_FORUM_ENVIRONMENT"))
        .unwrap_or_else(|_| "development".to_owned());
    let production = matches!(
        environment.trim().to_ascii_lowercase().as_str(),
        "prod" | "production"
    );
    let resolver = if production {
        iam_web_request_context_resolver_from_database_pool_for_audiences(
            assembly.database_pool.clone(),
            &[APPLICATION_ID, "forum"],
        )
        .await?
    } else {
        iam_web_request_context_resolver_from_env().await
    };
    let contribution = assembly.contribution;
    let mut framework = build_web_framework_builder(
        resolver,
        contribution.route_manifest.clone(),
        infra_public_path_prefixes(),
    );
    if production {
        let postgres_pool = assembly
            .database_pool
            .as_postgres()
            .cloned()
            .ok_or("production Forum gateway requires PostgreSQL")?;
        framework = framework
            .audit_emitter(Arc::new(IamAuditEmitter::new(
                postgres_pool.clone(),
                APPLICATION_ID,
                environment.clone(),
            )))
            .security_event_emitter(Arc::new(IamSecurityEventEmitter::new(
                postgres_pool,
                environment,
            )));
    }
    let mut module_registry = ApiModuleRegistry::new();
    module_registry.add_modules(vec![contribution]);
    let hosted = module_registry
        .try_compose("SDKWork Forum API")?
        .into_hosted(framework);
    let app = attach_ops_routes(hosted.router, ops_state);

    let bind_address = std::env::var("SDKWORK_FORUM_APPLICATION_PUBLIC_INGRESS_BIND")?;
    let listener = tokio::net::TcpListener::bind(&bind_address).await?;
    tracing::info!(%bind_address, "sdkwork-api-forum-standalone-gateway listening");
    axum::serve(listener, app).await?;
    Ok(())
}
