use super::models::*;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

impl<T> CursorPage<T> {
    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            next_cursor: None,
            has_more: false,
        }
    }

    pub fn new(items: Vec<T>, next_cursor: Option<String>, has_more: bool) -> Self {
        Self {
            items,
            next_cursor,
            has_more,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommandResult {
    pub success: bool,
    pub id: Option<i64>,
    pub uuid: Option<String>,
    pub status: Option<String>,
}

impl CommandResult {
    pub fn success(id: i64, uuid: impl Into<String>) -> Self {
        Self {
            success: true,
            id: Some(id),
            uuid: Some(uuid.into()),
            status: None,
        }
    }

    pub fn success_with_status(
        id: i64,
        uuid: impl Into<String>,
        status: impl Into<String>,
    ) -> Self {
        Self {
            success: true,
            id: Some(id),
            uuid: Some(uuid.into()),
            status: Some(status.into()),
        }
    }

    pub fn no_id() -> Self {
        Self {
            success: true,
            id: None,
            uuid: None,
            status: None,
        }
    }
}

pub type NodeTreeResult = Vec<ForumNode>;
pub type TopicPageResult = CursorPage<ForumTopic>;
pub type ReplyPageResult = CursorPage<ForumReply>;
pub type SearchResult = CursorPage<ForumSearchDocument>;
pub type ModerationQueueResult = CursorPage<ForumModerationCase>;
pub type ModerationDecisionResult = ForumModerationDecision;
pub type TopicRevisionPageResult = CursorPage<ForumTopicRevision>;
pub type ReplyRevisionPageResult = CursorPage<ForumReplyRevision>;
pub type ModerationCasePageResult = CursorPage<ForumModerationCase>;
pub type SanctionPageResult = CursorPage<ForumSanction>;
pub type ReputationRulePageResult = CursorPage<ForumReputationRule>;
pub type ReputationLedgerPageResult = CursorPage<ForumReputationLedger>;
pub type TrustLevelPageResult = CursorPage<ForumTrustLevel>;
pub type BadgePageResult = CursorPage<ForumBadge>;
pub type BoardStatsPageResult = CursorPage<ForumBoardStats>;
pub type TopicStatsPageResult = CursorPage<ForumTopicStats>;
pub type TopicPrefixPageResult = CursorPage<ForumTopicPrefix>;
pub type AuditActionPageResult = CursorPage<ForumAuditAction>;
pub type NodePageResult = CursorPage<ForumNode>;
pub type TagPageResult = CursorPage<ForumTag>;
pub type SubscriptionPageResult = CursorPage<ForumSubscription>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForumReputationRule {
    pub id: i64,
    pub uuid: String,
    pub code: String,
    pub event_type: String,
    pub points: i64,
    pub daily_limit: Option<i64>,
    pub rule_json: Option<serde_json::Value>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumReputationLedger {
    pub id: i64,
    pub uuid: String,
    pub user_id: i64,
    pub source_type: String,
    pub source_id: Option<i64>,
    pub direction: String,
    pub points: i64,
    pub balance_after: i64,
    pub reason_code: String,
    pub idempotency_key: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForumTrustLevel {
    pub id: i64,
    pub uuid: String,
    pub level_no: i32,
    pub code: String,
    pub name: String,
    pub threshold_rules: Option<serde_json::Value>,
    pub privileges: Option<serde_json::Value>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ForumBadge {
    pub id: i64,
    pub uuid: String,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub grant_mode: String,
    pub icon_media_id: Option<String>,
    pub rule_json: Option<serde_json::Value>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumTopicPrefix {
    pub id: i64,
    pub uuid: String,
    pub board_id: i64,
    pub code: String,
    pub label: String,
    pub color: Option<String>,
    pub sort_order: i32,
    pub required_trust_level: Option<i32>,
    pub status: String,
    pub version: i64,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForumAuditAction {
    pub id: i64,
    pub uuid: String,
    pub action: String,
    pub target_type: String,
    pub target_id: i64,
    pub operator_id: i64,
    pub detail: Option<String>,
    pub request_id: Option<String>,
    pub created_at: String,
}
