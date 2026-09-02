use crate::domain::events::ForumDomainEvent;
use crate::domain::models::ForumOutboxEvent;
use crate::error::ForumServiceError;
use crate::ports::repository::ForumRepository;
use crate::value_objects::ForumRequestContext;
use crate::ForumService;
use serde_json::Value;
use uuid::Uuid;

impl<R: ForumRepository> ForumService<R> {
    pub(super) fn record_domain_event(
        &self,
        ctx: &ForumRequestContext,
        event: ForumDomainEvent,
        payload: Value,
    ) -> Result<(), ForumServiceError> {
        let outbox = ForumOutboxEvent {
            id: 0,
            uuid: Uuid::new_v4().to_string(),
            event_key: format!("{}:{}", event.aggregate_type, event.aggregate_id),
            aggregate_type: event.aggregate_type.to_string(),
            aggregate_id: event.aggregate_id.clone(),
            event_type: event.event_type.to_string(),
            event_version: event.event_version as i32,
            payload_json: payload.to_string(),
            headers_json: None,
            status: "pending".to_string(),
            publish_attempts: 0,
            next_attempt_at: None,
            published_at: None,
            created_at: String::new(),
            updated_at: String::new(),
            tenant_id: ctx.tenant_id_value(),
            organization_id: ctx.organization_id_value(),
            version: 1,
        };

        self.repository.insert_outbox_event(ctx, &outbox)?;

        let source_type = match event.aggregate_type {
            "forum_topic" => "topic",
            "forum_topic_reply" => "reply",
            _ => event.aggregate_type,
        };
        self.index_search_best_effort(source_type, &event.aggregate_id);
        self.notify_forum_event_best_effort(&event.event_type, &event.aggregate_id);
        Ok(())
    }

    pub(super) fn remove_search_document_best_effort(&self, source_type: &str, source_id: &str) {
        if let Err(error) = self.search_port.delete_document(source_type, source_id) {
            tracing::warn!(source_type, source_id, error, "forum search delete failed");
        }
    }

    fn index_search_best_effort(&self, source_type: &str, source_id: &str) {
        if let Err(error) = self.search_port.index_document(source_type, source_id) {
            tracing::warn!(source_type, source_id, error, "forum search index failed");
        }
    }

    pub(super) fn notify_forum_event_best_effort(&self, event_type: &str, aggregate_id: &str) {
        if let Err(error) = self
            .notification_port
            .publish_forum_event(event_type, aggregate_id)
        {
            tracing::warn!(
                event_type,
                aggregate_id,
                error,
                "forum notification publish failed"
            );
        }
    }

    pub(super) fn notify_moderation_alert_best_effort(&self, case_id: i64, severity: &str) {
        if let Err(error) = self
            .notification_port
            .publish_moderation_alert(case_id, severity)
        {
            tracing::warn!(case_id, severity, error, "forum moderation alert failed");
        }
    }
}
