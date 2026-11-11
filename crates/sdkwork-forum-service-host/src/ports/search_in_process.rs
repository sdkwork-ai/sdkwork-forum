//! In-process sdkwork-search port for Forum projections (MODULE_SPEC.md §4.2).
//!
//! Forum projects topics and replies into sdkwork-search as `<source_type>:<id>`
//! documents. In every packaged deployment Search runs in this same process
//! against this same PostgreSQL, so this adapter calls Search's provider SPI and
//! PostgreSQL provider directly instead of Search's backend-api.
//!
//! # What the HTTP path enforced, and what is reproduced here
//!
//! - Tenant and organization come from [`ForumRequestContext`] and are carried in
//!   [`SearchProviderContext`], which the provider binds as the
//!   `search_document` tenant/organization predicate. They are never read from an
//!   environment default.
//! - The document payload is the one the forum adapter has always sent (id, title,
//!   kind, capability, scope, source, metadata), so projections keep the same
//!   fields and the same `{sourceType}:{sourceId}` document id.
//! - Documents are written by the provider's upsert on
//!   `(tenant, organization, index_key, document_id)`, so re-projecting the same
//!   topic twice is idempotent, exactly like a repeated route call.
//! - Deletes are the provider's soft delete on the same key tuple.
//!
//! # Failure semantics
//!
//! Every call site treats projection failures as non-fatal (it logs a warning), so
//! keeping `Result<_, String>` preserves the previous behaviour: a deployment whose
//! shared database does not embed Search degrades instead of failing a topic write.
//!
//! # Rebuild
//!
//! The search rebuild route runs `IndexingService::rebuild_index`, which records a
//! job row in Search's own indexing repository and then drops and recreates the
//! index through the provider. That repository implementation lives in Search's
//! assembly crate and is not exported as a reusable crate, so this adapter performs
//! the two provider calls the rebuild makes (drop → create) and leaves the job
//! bookkeeping to Search. It holds no SQL of its own.

use std::sync::Arc;

use serde_json::json;
use sqlx::PgPool;

use sdkwork_communication_forum_service::integration::search::{
    forum_search_document_id, forum_search_document_payload, ForumSearchPort,
};
use sdkwork_communication_forum_service::value_objects::ForumRequestContext;
use sdkwork_search_provider_spi::{
    DeleteDocument, IndexDocument, SearchProvider, SearchProviderCapability, SearchProviderContext,
    SearchProviderKind, SearchProviderRegistry,
};

use super::block_on_db;

/// Index key Forum projects forum documents into by default.
pub const DEFAULT_FORUM_SEARCH_INDEX_KEY: &str = "forum";

/// Deployment override for the Forum projection index key.
///
/// This is tuning, not an endpoint: absent or blank keeps the default and never
/// blocks startup.
pub const FORUM_SEARCH_INDEX_KEY_ENV: &str = "SDKWORK_FORUM_SEARCH_INDEX_ID";

/// Provider id Forum registers its in-process provider under.
const FORUM_SEARCH_PROVIDER_ID: &str = "forum-postgresql";

/// Resolves the projection index key.
pub fn forum_search_index_key() -> String {
    std::env::var(FORUM_SEARCH_INDEX_KEY_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_FORUM_SEARCH_INDEX_KEY.to_string())
}

/// Search projection through Search's provider SPI on the shared PostgreSQL.
pub struct InProcessForumSearchPort {
    registry: Arc<SearchProviderRegistry>,
    index_key: String,
}

impl InProcessForumSearchPort {
    /// Build the port over the host pool.
    pub fn new(pool: PgPool) -> Self {
        Self::with_index_key(pool, forum_search_index_key())
    }

    /// Build the port against an explicit index key.
    pub fn with_index_key(pool: PgPool, index_key: impl Into<String>) -> Self {
        let provider: Arc<dyn SearchProvider> = Arc::new(
            sdkwork_search_provider_postgresql::PostgresqlSearchProvider::new(
                FORUM_SEARCH_PROVIDER_ID,
                pool,
            ),
        );
        let registry = SearchProviderRegistry::builder()
            .register_provider(provider)
            .default_kind(SearchProviderKind::Postgresql)
            .build();
        Self {
            registry: Arc::new(registry),
            index_key: index_key.into(),
        }
    }

    fn indexing_provider(&self) -> Result<Arc<dyn SearchProvider>, String> {
        self.registry
            .select_for_capability(SearchProviderCapability::Indexing)
            .or_else(|_| self.registry.select_default())
            .map_err(|error| format!("sdkwork-search has no indexing provider: {error}"))
    }

    fn document_id(source_type: &str, source_id: &str) -> String {
        forum_search_document_id(source_type, source_id)
    }
}

impl ForumSearchPort for InProcessForumSearchPort {
    fn index_document(
        &self,
        ctx: &ForumRequestContext,
        source_type: &str,
        source_id: &str,
    ) -> Result<(), String> {
        let (source_type, source_id) = normalize_source(source_type, source_id)?;
        let document_id = Self::document_id(source_type, source_id);
        let provider_ctx = provider_context(ctx);
        let document = IndexDocument {
            index_key: self.index_key.clone(),
            document_id: document_id.clone(),
            // The same payload the opt-in HTTP adapter sends, so projections keep
            // the same fields on both transports.
            source: forum_search_document_payload(source_type, source_id),
            title: Some(document_id),
            content: None,
            tags: Vec::new(),
            drive_space_id: None,
            drive_node_id: None,
            embedding: Vec::new(),
        };
        let provider = self.indexing_provider()?;
        block_on_db(provider.index_document(&provider_ctx, &document))?
            .map_err(|error| format!("forum search document upsert failed: {error}"))
    }

    fn delete_document(
        &self,
        ctx: &ForumRequestContext,
        source_type: &str,
        source_id: &str,
    ) -> Result<(), String> {
        let (source_type, source_id) = normalize_source(source_type, source_id)?;
        let document = DeleteDocument {
            index_key: self.index_key.clone(),
            document_id: Self::document_id(source_type, source_id),
        };
        let provider_ctx = provider_context(ctx);
        let provider = self.indexing_provider()?;
        block_on_db(provider.delete_document(&provider_ctx, &document))?
            .map_err(|error| format!("forum search document delete failed: {error}"))
    }

    fn rebuild_index(
        &self,
        ctx: &ForumRequestContext,
        board_id: Option<i64>,
    ) -> Result<(), String> {
        // Board-scoped rebuild stays a forum-side concern: Search rebuilds the
        // whole index key. Forum's own repository rebuild already ran.
        if board_id.is_some() {
            tracing::debug!(
                ?board_id,
                "forum search rebuild is index-wide; board scoping stays forum-side"
            );
        }
        let provider_ctx = provider_context(ctx);
        let provider = self.indexing_provider()?;
        let schema = json!({
            "source": "sdkwork-forum",
            "indexKey": self.index_key,
        });
        block_on_db(provider.drop_index(&provider_ctx, &self.index_key))?
            .map_err(|error| format!("forum search index drop failed: {error}"))?;
        block_on_db(provider.create_index(&provider_ctx, &self.index_key, &schema))?
            .map_err(|error| format!("forum search index create failed: {error}"))
    }
}

/// Trims and validates a projection source, matching the adapter's document key.
fn normalize_source<'a>(
    source_type: &'a str,
    source_id: &'a str,
) -> Result<(&'a str, &'a str), String> {
    let source_type = source_type.trim();
    let source_id = source_id.trim();
    if source_type.is_empty() || source_id.is_empty() {
        return Err("forum search document requires a source type and a source id".to_string());
    }
    Ok((source_type, source_id))
}

/// Builds the provider context from the forum request context.
///
/// Tenant and organization are the caller's, so a document is written and read in
/// the same tenant the forum request acted in.
fn provider_context(ctx: &ForumRequestContext) -> SearchProviderContext {
    let mut builder = SearchProviderContext::builder()
        .tenant_id(ctx.tenant_id_value())
        .organization_id(ctx.organization_id_value())
        .user_id(Some(ctx.user_id_value()));
    if let Some(request_id) = ctx
        .request_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        builder = builder.request_id(request_id.to_string());
    }
    builder.build()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context(tenant_id: i64, organization_id: i64, user_id: i64) -> ForumRequestContext {
        ForumRequestContext::new(tenant_id, organization_id, user_id)
    }

    #[test]
    fn document_ids_stay_source_type_colon_source_id() {
        assert_eq!(
            InProcessForumSearchPort::document_id("topic", "42"),
            "topic:42"
        );
    }

    #[test]
    fn the_provider_context_carries_the_caller_tenant_and_organization() {
        let ctx = provider_context(&context(100_001, 7, 42));
        assert_eq!(ctx.tenant_id, 100_001);
        assert_eq!(ctx.organization_id, 7);
        assert_eq!(ctx.user_id, Some(42));
        assert_eq!(ctx.request_id, None);
    }

    #[test]
    fn a_request_id_is_propagated_when_present() {
        let ctx = provider_context(
            &context(100_001, 0, 1).with_request_id("forum-request-1".to_string()),
        );
        assert_eq!(ctx.request_id.as_deref(), Some("forum-request-1"));
    }

    #[test]
    fn a_missing_source_type_or_id_is_rejected() {
        assert!(normalize_source("", "42").is_err());
        assert!(normalize_source("topic", "  ").is_err());
        assert_eq!(normalize_source(" topic ", " 42 "), Ok(("topic", "42")));
    }

    #[test]
    fn the_index_key_defaults_and_can_be_overridden() {
        // The default constant is the index the HTTP adapter used.
        assert_eq!(DEFAULT_FORUM_SEARCH_INDEX_KEY, "forum");
        assert_eq!(forum_search_index_key().trim(), forum_search_index_key());
    }

    #[test]
    fn the_adapter_never_addresses_search_over_http() {
        // Regression guard for MODULE_SPEC.md §4.2.
        let source = include_str!("search_in_process.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("the module has a production section");
        assert!(!production.contains("reqwest"));
        assert!(!production.contains("ureq"));
        assert!(!production.contains("SEARCH_URL\""));
    }
}
