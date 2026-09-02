use std::sync::Arc;

pub trait ForumSearchPort: Send + Sync {
    fn index_document(&self, source_type: &str, source_id: &str) -> Result<(), String>;
    fn delete_document(&self, source_type: &str, source_id: &str) -> Result<(), String>;
    fn rebuild_index(&self, board_id: Option<i64>) -> Result<(), String>;
}

pub struct NoopForumSearchPort;

impl ForumSearchPort for NoopForumSearchPort {
    fn index_document(&self, _source_type: &str, _source_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn delete_document(&self, _source_type: &str, _source_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn rebuild_index(&self, _board_id: Option<i64>) -> Result<(), String> {
        Ok(())
    }
}

pub struct LoggingForumSearchPort;

impl ForumSearchPort for LoggingForumSearchPort {
    fn index_document(&self, source_type: &str, source_id: &str) -> Result<(), String> {
        tracing::info!(source_type, source_id, "forum search index requested");
        Ok(())
    }

    fn delete_document(&self, source_type: &str, source_id: &str) -> Result<(), String> {
        tracing::info!(source_type, source_id, "forum search delete requested");
        Ok(())
    }

    fn rebuild_index(&self, board_id: Option<i64>) -> Result<(), String> {
        tracing::info!(?board_id, "forum search rebuild requested");
        Ok(())
    }
}

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
        format!("{source_type}:{source_id}")
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
            "document": {
                "id": document_id,
                "title": document_id,
                "kind": "forum",
                "capability": "communication",
                "scope": source_type,
                "source": source_id,
                "metadata": {
                    "sourceType": source_type,
                    "sourceId": source_id,
                }
            }
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
    fn index_document(&self, source_type: &str, source_id: &str) -> Result<(), String> {
        self.upsert_document(source_type, source_id)
    }

    fn delete_document(&self, source_type: &str, source_id: &str) -> Result<(), String> {
        let document_id = Self::document_id(source_type, source_id);
        let url = self.document_path(&document_id);
        self.apply_backend_auth(self.client.delete(&url))
            .call()
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    fn rebuild_index(&self, board_id: Option<i64>) -> Result<(), String> {
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
