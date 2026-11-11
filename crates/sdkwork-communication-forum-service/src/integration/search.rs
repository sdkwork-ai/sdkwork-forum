//! Search integration port for Forum projections.
//!
//! MODULE_SPEC.md §4.2: the Forum host projects topics/replies into sdkwork-search
//! in-process (provider SPI on the shared PostgreSQL) and only falls back to
//! sdkwork-search's HTTP surface when a split deployment is explicitly opted into.
//! Every method therefore receives the [`ForumRequestContext`]: the tenant and
//! organization the indexed document belongs to come from the request, never from
//! an environment default.

use std::sync::Arc;

use crate::value_objects::ForumRequestContext;

/// The document id Forum projects for a source: `{source_type}:{source_id}`.
pub fn forum_search_document_id(source_type: &str, source_id: &str) -> String {
    format!("{source_type}:{source_id}")
}

/// The document payload Forum projects for a source.
///
/// One DTO for both transports: the in-process provider adapter uses it as the
/// indexed document's `source`, and the opt-in HTTP adapter sends it verbatim, so
/// the two paths cannot drift (MODULE_SPEC.md §4.2.7 "no duplicated DTO mapping").
pub fn forum_search_document_payload(source_type: &str, source_id: &str) -> serde_json::Value {
    let document_id = forum_search_document_id(source_type, source_id);
    serde_json::json!({
        "id": document_id,
        "title": document_id,
        "kind": "forum",
        "capability": "communication",
        "scope": source_type,
        "source": source_id,
        "metadata": {
            "sourceType": source_type,
            "sourceId": source_id,
        },
    })
}

pub trait ForumSearchPort: Send + Sync {
    fn index_document(
        &self,
        ctx: &ForumRequestContext,
        source_type: &str,
        source_id: &str,
    ) -> Result<(), String>;

    fn delete_document(
        &self,
        ctx: &ForumRequestContext,
        source_type: &str,
        source_id: &str,
    ) -> Result<(), String>;

    fn rebuild_index(
        &self,
        ctx: &ForumRequestContext,
        board_id: Option<i64>,
    ) -> Result<(), String>;
}

pub struct NoopForumSearchPort;

impl ForumSearchPort for NoopForumSearchPort {
    fn index_document(
        &self,
        _ctx: &ForumRequestContext,
        _source_type: &str,
        _source_id: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    fn delete_document(
        &self,
        _ctx: &ForumRequestContext,
        _source_type: &str,
        _source_id: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    fn rebuild_index(
        &self,
        _ctx: &ForumRequestContext,
        _board_id: Option<i64>,
    ) -> Result<(), String> {
        Ok(())
    }
}

pub struct LoggingForumSearchPort;

impl ForumSearchPort for LoggingForumSearchPort {
    fn index_document(
        &self,
        _ctx: &ForumRequestContext,
        source_type: &str,
        source_id: &str,
    ) -> Result<(), String> {
        tracing::info!(source_type, source_id, "forum search index requested");
        Ok(())
    }

    fn delete_document(
        &self,
        _ctx: &ForumRequestContext,
        source_type: &str,
        source_id: &str,
    ) -> Result<(), String> {
        tracing::info!(source_type, source_id, "forum search delete requested");
        Ok(())
    }

    fn rebuild_index(
        &self,
        _ctx: &ForumRequestContext,
        board_id: Option<i64>,
    ) -> Result<(), String> {
        tracing::info!(?board_id, "forum search rebuild requested");
        Ok(())
    }
}

/// sdkwork-search HTTP adapter for a **deliberate split deployment**.
///
/// Never the default: the host selects it only when
/// `SDKWORK_FORUM_SEARCH_MODE=http` is configured (see the service-host
/// `ports::search_http_adapter` module), and validates its endpoint and service
/// credential at startup.
///
/// The search backend derives tenant/organization from its own request context for
/// a backend-api call, so this adapter keeps the document payload it always sent
/// and does not project identity through request headers (AGENTS.md HTTP request
/// header contract).
pub struct HttpForumSearchPort {
    base_url: Arc<String>,
    index_id: Arc<String>,
    auth_token: Option<Arc<String>>,
    access_token: Option<Arc<String>>,
    client: ureq::Agent,
}

impl HttpForumSearchPort {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self::configured(base_url, "forum", None, None)
    }

    pub fn configured(
        base_url: impl Into<String>,
        index_id: impl Into<String>,
        auth_token: Option<String>,
        access_token: Option<String>,
    ) -> Self {
        Self {
            base_url: Arc::new(base_url.into().trim_end_matches('/').to_string()),
            index_id: Arc::new(index_id.into()),
            auth_token: auth_token
                .filter(|value| !value.trim().is_empty())
                .map(Arc::new),
            access_token: access_token
                .filter(|value| !value.trim().is_empty())
                .map(Arc::new),
            client: ureq::Agent::new(),
        }
    }

    fn document_id(source_type: &str, source_id: &str) -> String {
        forum_search_document_id(source_type, source_id)
    }

    fn document_path(&self, document_id: &str) -> String {
        format!(
            "{}/backend/v3/api/search/indexes/{}/documents/{}",
            self.base_url, self.index_id, document_id
        )
    }

    fn apply_backend_auth(&self, mut request: ureq::Request) -> ureq::Request {
        if let Some(token) = &self.auth_token {
            request = request.set("Authorization", token.as_str());
        }
        if let Some(token) = &self.access_token {
            request = request.set("Access-Token", token.as_str());
        }
        request
    }

    fn upsert_document(&self, source_type: &str, source_id: &str) -> Result<(), String> {
        let document_id = Self::document_id(source_type, source_id);
        let url = self.document_path(&document_id);
        let body = serde_json::json!({
            "document": forum_search_document_payload(source_type, source_id),
        });

        self.apply_backend_auth(
            self.client
                .put(&url)
                .set("Content-Type", "application/json"),
        )
        .send_json(body)
        .map_err(|error| error.to_string())?;
        Ok(())
    }
}

impl ForumSearchPort for HttpForumSearchPort {
    fn index_document(
        &self,
        _ctx: &ForumRequestContext,
        source_type: &str,
        source_id: &str,
    ) -> Result<(), String> {
        self.upsert_document(source_type, source_id)
    }

    fn delete_document(
        &self,
        _ctx: &ForumRequestContext,
        source_type: &str,
        source_id: &str,
    ) -> Result<(), String> {
        let document_id = Self::document_id(source_type, source_id);
        let url = self.document_path(&document_id);
        self.apply_backend_auth(self.client.delete(&url))
            .call()
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn rebuild_index(
        &self,
        _ctx: &ForumRequestContext,
        board_id: Option<i64>,
    ) -> Result<(), String> {
        let url = format!("{}/backend/v3/api/search/jobs/rebuild", self.base_url);
        let body = serde_json::json!({
            "indexId": self.index_id.as_str(),
            "full": true,
        });
        if board_id.is_some() {
            tracing::debug!(
                ?board_id,
                "forum search rebuild scoped board ignored by search job API"
            );
        }

        self.apply_backend_auth(
            self.client
                .post(&url)
                .set("Content-Type", "application/json"),
        )
        .send_json(body)
        .map_err(|error| error.to_string())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn document_id_joins_source_type_and_id() {
        assert_eq!(HttpForumSearchPort::document_id("topic", "42"), "topic:42");
    }

    #[test]
    fn document_path_uses_search_backend_api_shape() {
        let port = HttpForumSearchPort::configured("http://search.local", "forum-main", None, None);
        assert_eq!(
            port.document_path("topic:7"),
            "http://search.local/backend/v3/api/search/indexes/forum-main/documents/topic:7"
        );
    }
}
