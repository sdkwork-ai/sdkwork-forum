use std::sync::Arc;

pub trait ForumNotificationPort: Send + Sync {
    fn publish_forum_event(&self, event_type: &str, aggregate_id: &str) -> Result<(), String>;
    fn publish_moderation_alert(&self, case_id: i64, severity: &str) -> Result<(), String>;
    fn publish_subscription_notification(
        &self,
        user_id: i64,
        event_type: &str,
        target_id: i64,
    ) -> Result<(), String>;
}

pub struct NoopForumNotificationPort;

impl ForumNotificationPort for NoopForumNotificationPort {
    fn publish_forum_event(&self, _event_type: &str, _aggregate_id: &str) -> Result<(), String> {
        Ok(())
    }

    fn publish_moderation_alert(&self, _case_id: i64, _severity: &str) -> Result<(), String> {
        Ok(())
    }

    fn publish_subscription_notification(
        &self,
        _user_id: i64,
        _event_type: &str,
        _target_id: i64,
    ) -> Result<(), String> {
        Ok(())
    }
}

pub struct LoggingForumNotificationPort;

impl ForumNotificationPort for LoggingForumNotificationPort {
    fn publish_forum_event(&self, event_type: &str, aggregate_id: &str) -> Result<(), String> {
        tracing::info!(event_type, aggregate_id, "forum notification event");
        Ok(())
    }

    fn publish_moderation_alert(&self, case_id: i64, severity: &str) -> Result<(), String> {
        tracing::info!(case_id, severity, "forum moderation alert");
        Ok(())
    }

    fn publish_subscription_notification(
        &self,
        user_id: i64,
        event_type: &str,
        target_id: i64,
    ) -> Result<(), String> {
        tracing::info!(
            user_id,
            event_type,
            target_id,
            "forum subscription notification"
        );
        Ok(())
    }
}

pub struct HttpForumNotificationPort {
    base_url: Arc<String>,
    client: ureq::Agent,
}

impl HttpForumNotificationPort {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: Arc::new(base_url.into().trim_end_matches('/').to_string()),
            client: ureq::Agent::new(),
        }
    }

    fn post_event(&self, path: &str, body: serde_json::Value) -> Result<(), String> {
        let url = format!("{}/forum/v1/notifications/{}", self.base_url, path);
        self.client
            .post(&url)
            .set("Content-Type", "application/json")
            .send_json(body)
            .map_err(|error| error.to_string())?;
        Ok(())
    }
}

impl ForumNotificationPort for HttpForumNotificationPort {
    fn publish_forum_event(&self, event_type: &str, aggregate_id: &str) -> Result<(), String> {
        self.post_event(
            "events",
            serde_json::json!({ "eventType": event_type, "aggregateId": aggregate_id }),
        )
    }

    fn publish_moderation_alert(&self, case_id: i64, severity: &str) -> Result<(), String> {
        self.post_event(
            "moderation-alerts",
            serde_json::json!({ "caseId": case_id, "severity": severity }),
        )
    }

    fn publish_subscription_notification(
        &self,
        user_id: i64,
        event_type: &str,
        target_id: i64,
    ) -> Result<(), String> {
        self.post_event(
            "subscriptions",
            serde_json::json!({
                "userId": user_id,
                "eventType": event_type,
                "targetId": target_id,
            }),
        )
    }
}
