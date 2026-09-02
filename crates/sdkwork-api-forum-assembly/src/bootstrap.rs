//! Application API assembly bootstrap for sdkwork-forum.
//!
//! The assembly exports the indivisible `ApiAssemblyContribution` contract
//! (API_ASSEMBLY_SPEC.md section 4) as the `contribution` field of a
//! host-neutral bundle; the platform cloud gateway passes that field intact
//! into profile composition.

use std::sync::Arc;

use axum::Router;
use sdkwork_database_spi::{DefaultDatabaseModule, LocaleTag, SeedProfile};
use sdkwork_database_sqlx::DatabasePool;
use sdkwork_forum_http_support::{context::ForumContextInjector, AppState};
use sdkwork_forum_service_host::{default_seed_locale, default_seed_profile, ForumServiceHost};
use sdkwork_web_bootstrap::{
    ApiAssemblyContribution, DatabasePoolReadinessCheck, ReadinessCheck, WebModule,
};
use sdkwork_web_core::HttpRouteManifest;

/// Host-neutral API assembly bundle: the indivisible contribution plus the
/// database operations metadata owned by the Forum standalone host.
pub struct ApiAssembly {
    pub contribution: ApiAssemblyContribution,
    pub database_pool: DatabasePool,
    pub database_module: Arc<DefaultDatabaseModule>,
    pub seed_locale: LocaleTag,
    pub seed_profile: SeedProfile,
}

fn combined_route_manifest() -> HttpRouteManifest {
    let manifests = [
        sdkwork_routes_forum_app_api::gateway_route_manifest(),
        sdkwork_routes_forum_backend_api::gateway_route_manifest(),
    ];
    HttpRouteManifest::from_owned_routes(
        manifests
            .into_iter()
            .flat_map(|manifest| manifest.routes().to_vec())
            .collect(),
    )
}

fn contribution_from(
    router: Router,
    readiness_check: Arc<dyn ReadinessCheck>,
) -> Result<ApiAssemblyContribution, String> {
    ApiAssemblyContribution::from_manifest(
        "sdkwork-forum",
        "SDKWork Forum API",
        router,
        combined_route_manifest(),
        vec![Arc::new(ForumContextInjector)],
        readiness_check,
    )
}

fn forum_router(service_host: Arc<ForumServiceHost>) -> Router {
    let state = AppState::new(Arc::clone(&service_host));
    Router::new()
        .merge(sdkwork_routes_forum_app_api::gateway_mount())
        .merge(sdkwork_routes_forum_backend_api::gateway_mount())
        .with_state(state)
}

pub async fn assemble_api_router() -> ApiAssembly {
    let service_host = Arc::new(ForumServiceHost::new().await);
    let contribution = contribution_from(
        forum_router(Arc::clone(&service_host)),
        Arc::new(sdkwork_web_bootstrap::AlwaysReady),
    )
    .expect("forum contribution contract is valid");

    ApiAssembly {
        contribution,
        database_pool: service_host.database_pool(),
        database_module: service_host.database_module(),
        seed_locale: default_seed_locale(),
        seed_profile: default_seed_profile(),
    }
}

/// Assemble the Forum contribution against a caller-provided database pool so
/// the platform cloud gateway can share its process-wide PostgreSQL pool.
pub async fn assemble_api_router_with_pool(pool: DatabasePool) -> Result<ApiAssembly, String> {
    let service_host = Arc::new(ForumServiceHost::from_database_pool(pool.clone()).await?);
    let contribution = contribution_from(
        forum_router(Arc::clone(&service_host)),
        Arc::new(DatabasePoolReadinessCheck::new(pool)),
    )?;

    Ok(ApiAssembly {
        contribution,
        database_pool: service_host.database_pool(),
        database_module: service_host.database_module(),
        seed_locale: default_seed_locale(),
        seed_profile: default_seed_profile(),
    })
}

/// Canonical Web Module definition for this application
/// (API_ASSEMBLY_SPEC §4.1.1): the complete HTTP surface — every route,
/// manifest, and OpenAPI document of this owner — as one installable module.
pub async fn web_module() -> Result<WebModule, String> {
    Ok(WebModule::from_contribution(
        assemble_api_router().await.contribution,
    ))
}

/// Same as [`web_module`] but composed on a process-shared database pool
/// (platform gateways, API_ASSEMBLY_SPEC §4.1.1).
pub async fn web_module_with_pool(pool: DatabasePool) -> Result<WebModule, String> {
    Ok(WebModule::from_contribution(
        assemble_api_router_with_pool(pool).await?.contribution,
    ))
}
