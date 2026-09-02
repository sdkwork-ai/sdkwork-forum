use serde::{Deserialize, Serialize};
use serde_json;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumNode {
    pub id: i64,
    pub uuid: String,
    pub space_id: i64,
    pub parent_id: Option<i64>,
    pub node_type: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub path: String,
    pub level_no: i32,
    pub sort_order: i32,
    pub status: String,
    pub settings: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub deleted_by: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumBoard {
    pub id: i64,
    pub uuid: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumTag {
    pub id: i64,
    pub uuid: String,
    pub space_id: i64,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub usage_count: i64,
    pub status: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumTopic {
    pub id: i64,
    pub uuid: String,
    pub space_id: i64,
    pub board_id: i64,
    pub author_user_id: i64,
    pub prefix_id: Option<i64>,
    pub slug: Option<String>,
    pub title: String,
    pub body_format: String,
    pub body: String,
    pub body_excerpt: Option<String>,
    pub content_hash: String,
    pub topic_type: String,
    pub moderation_status: String,
    pub visibility: String,
    pub pinned_at: Option<String>,
    pub featured_at: Option<String>,
    pub locked_at: Option<String>,
    pub locked_by: Option<i64>,
    pub last_reply_id: Option<i64>,
    pub last_activity_at: String,
    pub accepted_reply_id: Option<i64>,
    pub attachment_count: i32,
    pub metadata: String,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub deleted_by: Option<i64>,
}

impl ForumTopic {
    pub fn is_question(&self) -> bool {
        self.topic_type == "question"
    }

    pub fn is_locked(&self) -> bool {
        self.locked_at.is_some()
    }

    pub fn is_visible(&self) -> bool {
        self.moderation_status == "visible"
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumReply {
    pub id: i64,
    pub uuid: String,
    pub topic_id: i64,
    pub board_id: i64,
    pub parent_reply_id: Option<i64>,
    pub author_user_id: i64,
    pub reply_no: i32,
    pub body_format: String,
    pub body: String,
    pub body_excerpt: Option<String>,
    pub content_hash: String,
    pub moderation_status: String,
    pub accepted_at: Option<String>,
    pub accepted_by: Option<i64>,
    pub attachment_count: i32,
    pub metadata: String,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub deleted_at: Option<String>,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub deleted_by: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumTopicRevision {
    pub id: i64,
    pub uuid: String,
    pub topic_id: i64,
    pub revision_no: i32,
    pub editor_user_id: i64,
    pub title: String,
    pub body_format: String,
    pub body: String,
    pub edit_reason: Option<String>,
    pub content_hash: String,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumReplyRevision {
    pub id: i64,
    pub uuid: String,
    pub reply_id: i64,
    pub topic_id: i64,
    pub revision_no: i32,
    pub editor_user_id: i64,
    pub body_format: String,
    pub body: String,
    pub edit_reason: Option<String>,
    pub content_hash: String,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumTopicStats {
    pub id: i64,
    pub uuid: String,
    pub topic_id: i64,
    pub reply_count: i64,
    pub view_count: i64,
    pub reaction_count: i64,
    pub vote_score: i64,
    pub bookmark_count: i64,
    pub report_count: i64,
    pub last_calculated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumBoardStats {
    pub id: i64,
    pub uuid: String,
    pub board_id: i64,
    pub topic_count: i64,
    pub reply_count: i64,
    pub member_count: i64,
    pub last_topic_id: Option<i64>,
    pub last_reply_id: Option<i64>,
    pub last_activity_at: Option<String>,
    pub last_calculated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumModerationCase {
    pub id: i64,
    pub uuid: String,
    pub case_no: String,
    pub target_type: String,
    pub target_id: i64,
    pub case_status: String,
    pub severity: String,
    pub opened_by: i64,
    pub assigned_to: Option<i64>,
    pub summary: Option<String>,
    pub resolved_at: Option<String>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub deleted_at: Option<String>,
    pub deleted_by: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumModerationDecision {
    pub id: i64,
    pub uuid: String,
    pub case_id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub decision_action: String,
    pub reason_code: String,
    pub note: Option<String>,
    pub decided_by: i64,
    pub before_state: String,
    pub after_state: String,
    pub idempotency_key: Option<String>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForumReport {
    pub id: i64,
    pub uuid: String,
    pub target_type: String,
    pub target_id: i64,
    pub reporter_user_id: Option<i64>,
    pub reason_code: String,
    pub description: Option<String>,
    pub report_status: String,
    pub linked_case_id: Option<i64>,
    pub evidence_json: Option<serde_json::Value>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForumOutboxEvent {
    pub id: i64,
    pub uuid: String,
    pub event_key: String,
    pub aggregate_type: String,
    pub aggregate_id: String,
    pub event_type: String,
    pub event_version: i32,
    pub payload_json: String,
    pub headers_json: Option<serde_json::Value>,
    pub status: String,
    pub publish_attempts: i32,
    pub next_attempt_at: Option<String>,
    pub published_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub version: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForumSearchDocument {
    pub id: i64,
    pub uuid: String,
    pub source_type: String,
    pub source_id: i64,
    pub board_id: i64,
    pub title: Option<String>,
    pub body_text: String,
    pub tag_text: Option<String>,
    pub author_user_id: i64,
    pub visibility: String,
    pub source_version: i64,
    pub index_status: String,
    pub indexed_at: Option<String>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumMemberProfile {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub display_name: String,
    pub avatar_media_id: Option<String>,
    pub bio: Option<String>,
    pub trust_level: i32,
    pub reputation_score: i64,
    pub joined_at: String,
    pub suspended_until: Option<String>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumSanction {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub case_id: Option<i64>,
    pub decision_id: Option<i64>,
    pub sanction_type: String,
    pub reason_code: String,
    pub starts_at: String,
    pub expires_at: Option<String>,
    pub lifted_at: Option<String>,
    pub lifted_by: Option<i64>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub deleted_at: Option<String>,
    pub deleted_by: Option<i64>,
}

impl fmt::Display for ForumTopic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ForumTopic(id={}, title={})", self.id, self.title)
    }
}

impl fmt::Display for ForumReply {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ForumReply(id={}, topic_id={})", self.id, self.topic_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumSpace {
    pub id: i64,
    pub uuid: String,
    pub code: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub visibility: String,
    pub default_locale: Option<String>,
    pub settings: String,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub deleted_at: Option<String>,
    pub deleted_by: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumAttachment {
    pub id: i64,
    pub uuid: String,
    pub owner_type: String,
    pub owner_id: i64,
    pub drive_space_id: String,
    pub drive_node_id: String,
    pub media_resource_id: Option<String>,
    pub file_name: String,
    pub mime_type: String,
    pub byte_size: i64,
    pub sort_order: i32,
    pub scan_status: String,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumSubscription {
    pub id: i64,
    pub uuid: String,
    pub target_type: String,
    pub target_id: i64,
    pub user_id: i64,
    pub notify_level: String,
    pub delivery_channels: String,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumIdempotencyRecord {
    pub id: i64,
    pub uuid: String,
    pub idempotency_key: String,
    pub request_hash: String,
    pub operation_id: String,
    pub principal_id: String,
    pub response_status: Option<i32>,
    pub response_body_json: Option<String>,
    pub expires_at: String,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumNotificationPreference {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub event_type: String,
    pub channel: String,
    pub enabled: bool,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumMemberStats {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub topic_count: i64,
    pub reply_count: i64,
    pub accepted_answer_count: i64,
    pub reaction_received_count: i64,
    pub vote_score_received: i64,
    pub last_activity_at: Option<String>,
    pub last_calculated_at: String,
    pub tenant_id: i64,
    pub organization_id: i64,
    pub data_scope: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}
