use super::SqlxForumRepository;
use md5::{Digest, Md5};
use sdkwork_communication_forum_service::domain::commands::*;
use sdkwork_communication_forum_service::domain::models::*;
use sdkwork_communication_forum_service::domain::results::*;
use sdkwork_communication_forum_service::ports::repository::ForumRepository;
use sdkwork_communication_forum_service::value_objects::ForumRequestContext;
use sdkwork_communication_forum_service::ForumServiceError;
use sqlx::Row;
use uuid::Uuid;

macro_rules! run_db {
    ($block:expr) => {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on($block))
    };
}

fn parse_cursor(cursor: &Option<String>) -> i64 {
    cursor
        .as_deref()
        .unwrap_or("0")
        .parse::<i64>()
        .unwrap_or(0)
        .max(0)
}

fn compute_hash(body: &str) -> String {
    let mut h = Md5::new();
    h.update(body.as_bytes());
    format!("{:x}", h.finalize())
}

fn compute_excerpt(body: &str) -> Option<String> {
    let s: String = body.chars().take(500).collect();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn fmt_ts(dt: chrono::DateTime<chrono::Utc>) -> String {
    dt.to_rfc3339()
}

fn fmt_opt_ts(opt: Option<chrono::DateTime<chrono::Utc>>) -> Option<String> {
    opt.map(|d| d.to_rfc3339())
}

fn fmt_json(v: serde_json::Value) -> String {
    v.to_string()
}

fn row_to_topic(row: &sqlx::postgres::PgRow) -> ForumTopic {
    ForumTopic {
        id: row.get("id"),
        uuid: row.get("uuid"),
        space_id: row.get("space_id"),
        board_id: row.get("board_id"),
        author_user_id: row.get("author_user_id"),
        prefix_id: row.get("prefix_id"),
        slug: row.get("slug"),
        title: row.get("title"),
        body_format: row.get("body_format"),
        body: row.get("body"),
        body_excerpt: row.get("body_excerpt"),
        content_hash: row.get("content_hash"),
        topic_type: row.get("topic_type"),
        moderation_status: row.get("moderation_status"),
        visibility: row.get("visibility"),
        pinned_at: fmt_opt_ts(row.get("pinned_at")),
        featured_at: fmt_opt_ts(row.get("featured_at")),
        locked_at: fmt_opt_ts(row.get("locked_at")),
        locked_by: row.get("locked_by"),
        last_reply_id: row.get("last_reply_id"),
        last_activity_at: fmt_ts(row.get("last_activity_at")),
        accepted_reply_id: row.get("accepted_reply_id"),
        attachment_count: row.get("attachment_count"),
        metadata: fmt_json(row.get("metadata")),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        deleted_at: fmt_opt_ts(row.get("deleted_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        deleted_by: row.get("deleted_by"),
    }
}

fn row_to_reply(row: &sqlx::postgres::PgRow) -> ForumReply {
    ForumReply {
        id: row.get("id"),
        uuid: row.get("uuid"),
        topic_id: row.get("topic_id"),
        board_id: row.get("board_id"),
        parent_reply_id: row.get("parent_reply_id"),
        author_user_id: row.get("author_user_id"),
        reply_no: row.get("reply_no"),
        body_format: row.get("body_format"),
        body: row.get("body"),
        body_excerpt: row.get("body_excerpt"),
        content_hash: row.get("content_hash"),
        moderation_status: row.get("moderation_status"),
        accepted_at: fmt_opt_ts(row.get("accepted_at")),
        accepted_by: row.get("accepted_by"),
        attachment_count: row.get("attachment_count"),
        metadata: fmt_json(row.get("metadata")),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        deleted_at: fmt_opt_ts(row.get("deleted_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        deleted_by: row.get("deleted_by"),
    }
}

fn row_to_node(row: &sqlx::postgres::PgRow) -> ForumNode {
    ForumNode {
        id: row.get("id"),
        uuid: row.get("uuid"),
        space_id: row.get("space_id"),
        parent_id: row.get("parent_id"),
        node_type: row.get("node_type"),
        slug: row.get("slug"),
        name: row.get("name"),
        description: row.get("description"),
        path: row.get("path"),
        level_no: row.get("level_no"),
        sort_order: row.get("sort_order"),
        status: row.get("status"),
        settings: fmt_json(row.get("settings")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        deleted_at: fmt_opt_ts(row.get("deleted_at")),
        deleted_by: row.get("deleted_by"),
    }
}

fn row_to_sanction(row: &sqlx::postgres::PgRow) -> ForumSanction {
    ForumSanction {
        id: row.get("id"),
        uuid: row.get("uuid"),
        user_id: row.get("user_id"),
        case_id: row.get("case_id"),
        decision_id: row.get("decision_id"),
        sanction_type: row.get("sanction_type"),
        reason_code: row.get("reason_code"),
        starts_at: fmt_ts(row.get("starts_at")),
        expires_at: fmt_opt_ts(row.get("expires_at")),
        lifted_at: fmt_opt_ts(row.get("lifted_at")),
        lifted_by: row.get("lifted_by"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        deleted_at: fmt_opt_ts(row.get("deleted_at")),
        deleted_by: row.get("deleted_by"),
    }
}

fn row_to_idempotency_record(row: &sqlx::postgres::PgRow) -> ForumIdempotencyRecord {
    ForumIdempotencyRecord {
        id: row.get("id"),
        uuid: row.get("uuid"),
        idempotency_key: row.get("idempotency_key"),
        request_hash: row.get("request_hash"),
        operation_id: row.get("operation_id"),
        principal_id: row.get("principal_id"),
        response_status: row.get("response_status"),
        response_body_json: row.get("response_body_json"),
        expires_at: fmt_ts(row.get("expires_at")),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
    }
}

fn row_to_topic_stats(row: &sqlx::postgres::PgRow) -> ForumTopicStats {
    ForumTopicStats {
        id: row.get("id"),
        uuid: row.get("uuid"),
        topic_id: row.get("topic_id"),
        reply_count: row.get("reply_count"),
        view_count: row.get("view_count"),
        reaction_count: row.get("reaction_count"),
        vote_score: row.get("vote_score"),
        bookmark_count: row.get("bookmark_count"),
        report_count: row.get("report_count"),
        last_calculated_at: fmt_ts(row.get("last_calculated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
    }
}

fn row_to_board_stats(row: &sqlx::postgres::PgRow) -> ForumBoardStats {
    ForumBoardStats {
        id: row.get("id"),
        uuid: row.get("uuid"),
        board_id: row.get("board_id"),
        topic_count: row.get("topic_count"),
        reply_count: row.get("reply_count"),
        member_count: row.get("member_count"),
        last_topic_id: row.get("last_topic_id"),
        last_reply_id: row.get("last_reply_id"),
        last_activity_at: fmt_opt_ts(row.get("last_activity_at")),
        last_calculated_at: fmt_ts(row.get("last_calculated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
    }
}

fn row_to_member_stats(row: &sqlx::postgres::PgRow) -> ForumMemberStats {
    ForumMemberStats {
        id: row.get("id"),
        uuid: row.get("uuid"),
        user_id: row.get("user_id"),
        topic_count: row.get("topic_count"),
        reply_count: row.get("reply_count"),
        accepted_answer_count: row.get("accepted_answer_count"),
        reaction_received_count: row.get("reaction_received_count"),
        vote_score_received: row.get("vote_score_received"),
        last_activity_at: fmt_opt_ts(row.get("last_activity_at")),
        last_calculated_at: fmt_ts(row.get("last_calculated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
    }
}

fn row_to_notification_preference(row: &sqlx::postgres::PgRow) -> ForumNotificationPreference {
    ForumNotificationPreference {
        id: row.get("id"),
        uuid: row.get("uuid"),
        user_id: row.get("user_id"),
        event_type: row.get("event_type"),
        channel: row.get("channel"),
        enabled: row.get("enabled"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
    }
}

fn row_to_space(row: &sqlx::postgres::PgRow) -> ForumSpace {
    ForumSpace {
        id: row.get("id"),
        uuid: row.get("uuid"),
        code: row.get("code"),
        slug: row.get("slug"),
        name: row.get("name"),
        description: row.get("description"),
        visibility: row.get("visibility"),
        default_locale: row.get("default_locale"),
        settings: fmt_json(row.get("settings")),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        deleted_at: fmt_opt_ts(row.get("deleted_at")),
        deleted_by: row.get("deleted_by"),
    }
}

fn row_to_attachment(row: &sqlx::postgres::PgRow) -> ForumAttachment {
    ForumAttachment {
        id: row.get("id"),
        uuid: row.get("uuid"),
        owner_type: row.get("owner_type"),
        owner_id: row.get("owner_id"),
        drive_space_id: row.get("drive_space_id"),
        drive_node_id: row.get("drive_node_id"),
        media_resource_id: row.get("media_resource_id"),
        file_name: row.get("file_name"),
        mime_type: row.get("mime_type"),
        byte_size: row.get("byte_size"),
        sort_order: row.get("sort_order"),
        scan_status: row.get("scan_status"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
    }
}

fn row_to_subscription(row: &sqlx::postgres::PgRow) -> ForumSubscription {
    ForumSubscription {
        id: row.get("id"),
        uuid: row.get("uuid"),
        target_type: row.get("target_type"),
        target_id: row.get("target_id"),
        user_id: row.get("user_id"),
        notify_level: row.get("notify_level"),
        delivery_channels: row.get("delivery_channels"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
    }
}

fn row_to_moderation_case(row: &sqlx::postgres::PgRow) -> ForumModerationCase {
    ForumModerationCase {
        id: row.get("id"),
        uuid: row.get("uuid"),
        case_no: row.get("case_no"),
        target_type: row.get("target_type"),
        target_id: row.get("target_id"),
        case_status: row.get("case_status"),
        severity: row.get("severity"),
        opened_by: row.get("opened_by"),
        assigned_to: row.get("assigned_to"),
        summary: row.get("summary"),
        resolved_at: fmt_opt_ts(row.get("resolved_at")),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        deleted_at: fmt_opt_ts(row.get("deleted_at")),
        deleted_by: row.get("deleted_by"),
    }
}

fn row_to_moderation_decision(row: &sqlx::postgres::PgRow) -> ForumModerationDecision {
    ForumModerationDecision {
        id: row.get("id"),
        uuid: row.get("uuid"),
        case_id: row.get("case_id"),
        target_type: row.get("target_type"),
        target_id: row.get("target_id"),
        decision_action: row.get("decision_action"),
        reason_code: row.get("reason_code"),
        note: row.get("note"),
        decided_by: row.get("decided_by"),
        before_state: fmt_json(row.get("before_state")),
        after_state: fmt_json(row.get("after_state")),
        idempotency_key: row.get("idempotency_key"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
    }
}

fn row_to_search_document(row: &sqlx::postgres::PgRow) -> ForumSearchDocument {
    ForumSearchDocument {
        id: row.get("id"),
        uuid: row.get("uuid"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        board_id: row.get("board_id"),
        title: row.get("title"),
        body_text: row.get("body_text"),
        tag_text: row.get("tag_text"),
        author_user_id: row.get("author_user_id"),
        visibility: row.get("visibility"),
        source_version: row.get("source_version"),
        index_status: row.get("index_status"),
        indexed_at: fmt_opt_ts(row.get("indexed_at")),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
    }
}

fn row_to_topic_revision(row: &sqlx::postgres::PgRow) -> ForumTopicRevision {
    ForumTopicRevision {
        id: row.get("id"),
        uuid: row.get("uuid"),
        topic_id: row.get("topic_id"),
        revision_no: row.get("revision_no"),
        editor_user_id: row.get("editor_user_id"),
        title: row.get("title"),
        body_format: row.get("body_format"),
        body: row.get("body"),
        edit_reason: row.get("edit_reason"),
        content_hash: row.get("content_hash"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
    }
}

fn row_to_reply_revision(row: &sqlx::postgres::PgRow) -> ForumReplyRevision {
    ForumReplyRevision {
        id: row.get("id"),
        uuid: row.get("uuid"),
        reply_id: row.get("reply_id"),
        topic_id: row.get("topic_id"),
        revision_no: row.get("revision_no"),
        editor_user_id: row.get("editor_user_id"),
        body_format: row.get("body_format"),
        body: row.get("body"),
        edit_reason: row.get("edit_reason"),
        content_hash: row.get("content_hash"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
    }
}

fn row_to_reputation_rule(row: &sqlx::postgres::PgRow) -> ForumReputationRule {
    ForumReputationRule {
        id: row.get("id"),
        uuid: row.get("uuid"),
        code: row.get("code"),
        event_type: row.get("event_type"),
        points: row.get("points"),
        daily_limit: row.get("daily_limit"),
        rule_json: row.get("rule_json"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
    }
}

fn row_to_reputation_ledger(row: &sqlx::postgres::PgRow) -> ForumReputationLedger {
    ForumReputationLedger {
        id: row.get("id"),
        uuid: row.get("uuid"),
        user_id: row.get("user_id"),
        source_type: row.get("source_type"),
        source_id: row.get("source_id"),
        direction: row.get("direction"),
        points: row.get("points"),
        balance_after: row.get("balance_after"),
        reason_code: row.get("reason_code"),
        idempotency_key: row.get("idempotency_key"),
        created_at: fmt_ts(row.get("created_at")),
    }
}

fn row_to_trust_level(row: &sqlx::postgres::PgRow) -> ForumTrustLevel {
    ForumTrustLevel {
        id: row.get("id"),
        uuid: row.get("uuid"),
        level_no: row.get("level_no"),
        code: row.get("code"),
        name: row.get("name"),
        threshold_rules: row.get("threshold_rules"),
        privileges: row.get("privileges"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
    }
}

fn row_to_badge(row: &sqlx::postgres::PgRow) -> ForumBadge {
    ForumBadge {
        id: row.get("id"),
        uuid: row.get("uuid"),
        code: row.get("code"),
        name: row.get("name"),
        description: row.get("description"),
        grant_mode: row.get("grant_mode"),
        icon_media_id: row.get("icon_media_id"),
        rule_json: row.get("rule_json"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
    }
}

fn row_to_topic_prefix(row: &sqlx::postgres::PgRow) -> ForumTopicPrefix {
    ForumTopicPrefix {
        id: row.get("id"),
        uuid: row.get("uuid"),
        board_id: row.get("board_id"),
        code: row.get("code"),
        label: row.get("label"),
        color: row.get("color"),
        sort_order: row.get("sort_order"),
        required_trust_level: row.get("required_trust_level"),
        status: row.get("status"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
    }
}

fn row_to_audit_action(row: &sqlx::postgres::PgRow) -> ForumAuditAction {
    ForumAuditAction {
        id: row.get("id"),
        uuid: row.get("uuid"),
        action: row.get("action"),
        target_type: row.get("target_type"),
        target_id: row.get("target_id"),
        operator_id: row.get("operator_id"),
        detail: row.get("detail"),
        request_id: row.get("request_id"),
        created_at: fmt_ts(row.get("created_at")),
    }
}

fn row_to_tag(row: &sqlx::postgres::PgRow) -> ForumTag {
    ForumTag {
        id: row.get("id"),
        uuid: row.get("uuid"),
        space_id: row.get("space_id"),
        slug: row.get("slug"),
        name: row.get("name"),
        description: row.get("description"),
        color: row.get("color"),
        usage_count: row.get("usage_count"),
        status: row.get("status"),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        data_scope: row.get("data_scope"),
        version: row.get("version"),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
    }
}

fn row_to_outbox_event(row: &sqlx::postgres::PgRow) -> ForumOutboxEvent {
    ForumOutboxEvent {
        id: row.get("id"),
        uuid: row.get("uuid"),
        event_key: row.get("event_key"),
        aggregate_type: row.get("aggregate_type"),
        aggregate_id: row.get("aggregate_id"),
        event_type: row.get("event_type"),
        event_version: row.get("event_version"),
        payload_json: row.get::<serde_json::Value, _>("payload_json").to_string(),
        headers_json: row.get("headers_json"),
        status: row.get("status"),
        publish_attempts: row.get("publish_attempts"),
        next_attempt_at: fmt_opt_ts(row.get("next_attempt_at")),
        published_at: fmt_opt_ts(row.get("published_at")),
        created_at: fmt_ts(row.get("created_at")),
        updated_at: fmt_ts(row.get("updated_at")),
        tenant_id: row.get("tenant_id"),
        organization_id: row.get("organization_id"),
        version: row.get("version"),
    }
}

impl ForumRepository for SqlxForumRepository {
    fn list_node_tree(
        &self,
        ctx: &ForumRequestContext,
        command: &ListNodeTreeCommand,
    ) -> Result<NodeTreeResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_node
                 WHERE tenant_id = $1
                   AND deleted_at IS NULL
                   AND ($2::bigint IS NULL OR space_id = $2)
                   AND ($3::bigint IS NULL OR parent_id = $3)
                 ORDER BY sort_order ASC",
            )
            .bind(tenant_id)
            .bind(command.space_id)
            .bind(command.parent_id)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(rows.iter().map(row_to_node).collect())
    }

    fn list_nodes(
        &self,
        ctx: &ForumRequestContext,
        command: &ListNodesCommand,
    ) -> Result<NodePageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_node
                 WHERE tenant_id = $1
                   AND deleted_at IS NULL
                   AND ($2::bigint IS NULL OR space_id = $2)
                   AND ($3::text IS NULL OR node_type = $3)
                 ORDER BY sort_order ASC, id ASC
                 LIMIT $4 OFFSET $5",
            )
            .bind(tenant_id)
            .bind(command.space_id)
            .bind(command.node_type.as_deref())
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumNode> = rows.iter().take(limit as usize).map(row_to_node).collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn list_topics(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTopicsCommand,
    ) -> Result<TopicPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();
        let board_filter = command.board_id;
        let status_filter = command.status_filter.clone();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_topic
                 WHERE tenant_id = $1
                   AND deleted_at IS NULL
                   AND ($2::bigint IS NULL OR board_id = $2)
                   AND ($3::text IS NULL OR moderation_status = $3)
                 ORDER BY last_activity_at DESC
                 LIMIT $4 OFFSET $5",
            )
            .bind(tenant_id)
            .bind(board_filter)
            .bind(status_filter.as_deref())
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumTopic> = rows.iter().take(limit as usize).map(row_to_topic).collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateTopicCommand,
    ) -> Result<ForumTopic, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();

        let space_id: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT space_id FROM forum_node WHERE id = $1 AND tenant_id = $2 AND node_type = 'board' AND deleted_at IS NULL"
            )
            .bind(command.board_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| match e {
            sqlx::Error::RowNotFound => ForumServiceError::not_found("board", command.board_id.to_string()),
            e => ForumServiceError::internal(e.to_string()),
        })?;

        let uuid = Uuid::new_v4().to_string();
        let hash = compute_hash(&command.body);
        let excerpt = compute_excerpt(&command.body);
        let topic_type = command.topic_type.as_deref().unwrap_or("discussion");
        let visibility = command.visibility.as_deref().unwrap_or("public");
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_topic (
                    id, uuid, space_id, board_id, author_user_id, prefix_id, title, body_format,
                    body, body_excerpt, content_hash, topic_type, moderation_status, visibility,
                    attachment_count, metadata, status, version, created_at, updated_at,
                    last_activity_at, tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, 'visible', $13,
                    0, $14, 'active', 1, NOW(), NOW(), NOW(), $15, $16, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(space_id)
            .bind(command.board_id)
            .bind(user_id)
            .bind(command.prefix_id)
            .bind(&command.title)
            .bind(&command.body_format)
            .bind(&command.body)
            .bind(excerpt.as_deref())
            .bind(&hash)
            .bind(topic_type)
            .bind(visibility)
            .bind(serde_json::json!({}))
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        Ok(row_to_topic(&row))
    }

    fn retrieve_topic(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<ForumTopic, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_topic WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL",
            )
            .bind(topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ForumServiceError::not_found("topic", topic_id.to_string()),
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_topic(&row))
    }

    fn retrieve_topic_by_slug(
        &self,
        ctx: &ForumRequestContext,
        command: &RetrieveTopicBySlugCommand,
    ) -> Result<ForumTopic, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_topic
                 WHERE tenant_id = $1
                   AND slug = $2
                   AND deleted_at IS NULL
                   AND ($3::bigint IS NULL OR board_id = $3)",
            )
            .bind(tenant_id)
            .bind(&command.slug)
            .bind(command.board_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => ForumServiceError::not_found("topic", command.slug.clone()),
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_topic(&row))
    }

    fn update_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateTopicCommand,
    ) -> Result<ForumTopic, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let new_hash = command.body.as_deref().map(compute_hash);
        let new_excerpt = command.body.as_ref().and_then(|b| compute_excerpt(b));

        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET title = COALESCE($1, title),
                     body = COALESCE($2, body),
                     body_format = COALESCE($3, body_format),
                     content_hash = COALESCE($6, content_hash),
                     body_excerpt = COALESCE($7, body_excerpt),
                     version = version + 1,
                     updated_at = NOW()
                 WHERE id = $4 AND tenant_id = $5 AND deleted_at IS NULL
                 RETURNING *",
            )
            .bind(command.title.as_deref())
            .bind(command.body.as_deref())
            .bind(command.body_format.as_deref())
            .bind(command.topic_id)
            .bind(tenant_id)
            .bind(new_hash.as_deref())
            .bind(new_excerpt.as_deref())
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_topic(&row))
    }

    fn delete_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &DeleteTopicCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let user_id = ctx.user_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET deleted_at = NOW(), deleted_by = $1, status = 'deleted'
                 WHERE id = $2 AND tenant_id = $3 AND deleted_at IS NULL
                 RETURNING id, uuid",
            )
            .bind(user_id)
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn list_replies(
        &self,
        ctx: &ForumRequestContext,
        command: &ListRepliesCommand,
    ) -> Result<ReplyPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_topic_reply
                 WHERE topic_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
                 ORDER BY reply_no ASC
                 LIMIT $3 OFFSET $4",
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumReply> = rows.iter().take(limit as usize).map(row_to_reply).collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateReplyCommand,
    ) -> Result<ForumReply, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();

        let board_id: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT board_id FROM forum_topic WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| match e {
            sqlx::Error::RowNotFound => ForumServiceError::not_found("topic", command.topic_id.to_string()),
            e => ForumServiceError::internal(e.to_string()),
        })?;

        let reply_no: i32 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COALESCE(MAX(reply_no), 0) + 1 FROM forum_topic_reply WHERE topic_id = $1 AND tenant_id = $2"
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let uuid = Uuid::new_v4().to_string();
        let hash = compute_hash(&command.body);
        let excerpt = compute_excerpt(&command.body);
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_topic_reply (
                    id, uuid, topic_id, board_id, parent_reply_id, author_user_id, reply_no,
                    body_format, body, body_excerpt, content_hash, moderation_status,
                    attachment_count, metadata, status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 'visible',
                    0, $12, 'active', 1, NOW(), NOW(), $13, $14, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(command.topic_id)
            .bind(board_id)
            .bind(command.parent_reply_id)
            .bind(user_id)
            .bind(reply_no)
            .bind(&command.body_format)
            .bind(&command.body)
            .bind(excerpt.as_deref())
            .bind(&hash)
            .bind(serde_json::json!({}))
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        Ok(row_to_reply(&row))
    }

    fn update_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateReplyCommand,
    ) -> Result<ForumReply, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic_reply
                 SET body = COALESCE($1, body),
                     body_format = COALESCE($2, body_format),
                     version = version + 1,
                     updated_at = NOW()
                 WHERE id = $3 AND tenant_id = $4 AND deleted_at IS NULL
                 RETURNING *",
            )
            .bind(command.body.as_deref())
            .bind(command.body_format.as_deref())
            .bind(command.reply_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("reply", command.reply_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_reply(&row))
    }

    fn delete_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &DeleteReplyCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let user_id = ctx.user_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic_reply
                 SET deleted_at = NOW(), deleted_by = $1, status = 'deleted'
                 WHERE id = $2 AND tenant_id = $3 AND deleted_at IS NULL
                 RETURNING id, uuid",
            )
            .bind(user_id)
            .bind(command.reply_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("reply", command.reply_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn accept_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &AcceptReplyCommand,
    ) -> Result<ForumTopic, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET accepted_reply_id = $1
                 WHERE id = $2 AND tenant_id = $3 AND topic_type = 'question'
                 RETURNING *",
            )
            .bind(command.reply_id)
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_topic(&row))
    }

    fn clear_accepted_reply(
        &self,
        ctx: &ForumRequestContext,
        command: &ClearAcceptedReplyCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET accepted_reply_id = NULL
                 WHERE id = $1 AND tenant_id = $2
                 RETURNING id, uuid",
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn create_report(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateReportCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_report (
                    id, uuid, target_type, target_id, reporter_user_id, reason_code, description,
                    report_status, status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    'open', 'active', 1, NOW(), NOW(), $8, $9, 'default'
                ) RETURNING id, uuid",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.target_type)
            .bind(command.target_id)
            .bind(user_id)
            .bind(&command.reason_code)
            .bind(command.description.as_deref())
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn query_search(
        &self,
        ctx: &ForumRequestContext,
        command: &QuerySearchCommand,
    ) -> Result<SearchResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();
        let pattern = format!("%{}%", command.query);

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_search_document
                 WHERE tenant_id = $1
                   AND deleted_at IS NULL
                   AND status = 'active'
                   AND ($2::bigint IS NULL OR board_id = $2)
                   AND (title ILIKE $3 OR body_text ILIKE $3 OR COALESCE(tag_text, '') ILIKE $3)
                 ORDER BY updated_at DESC, id DESC
                 LIMIT $4 OFFSET $5",
            )
            .bind(tenant_id)
            .bind(command.board_id)
            .bind(&pattern)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumSearchDocument> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_search_document)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn list_moderation_queue(
        &self,
        ctx: &ForumRequestContext,
        command: &ListModerationQueueCommand,
    ) -> Result<ModerationQueueResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT c.* FROM forum_moderation_case c
                 WHERE c.tenant_id = $1
                   AND c.deleted_at IS NULL
                   AND ($2::text IS NULL OR c.case_status = $2)
                   AND ($3::text IS NULL OR c.severity = $3)
                   AND EXISTS (
                     SELECT 1 FROM forum_moderation_queue_item q
                     WHERE q.tenant_id = c.tenant_id
                       AND q.case_id = c.id
                       AND q.deleted_at IS NULL
                       AND q.queue_status IN ('open', 'assigned', 'in_review')
                   )
                 ORDER BY c.updated_at DESC, c.id DESC
                 LIMIT $4 OFFSET $5",
            )
            .bind(tenant_id)
            .bind(command.status_filter.as_deref())
            .bind(command.severity_filter.as_deref())
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumModerationCase> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_moderation_case)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_moderation_decision(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateModerationDecisionCommand,
    ) -> Result<ModerationDecisionResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let case_row = run_db!(async {
            sqlx::query(
                "SELECT target_type, target_id, case_status FROM forum_moderation_case
                 WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL",
            )
            .bind(command.case_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("moderation_case", command.case_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;

        let target_type: String = case_row.get("target_type");
        let target_id: i64 = case_row.get("target_id");
        let before_state =
            serde_json::json!({ "case_status": case_row.get::<String, _>("case_status") });
        let new_case_status = match command.decision_action.as_str() {
            "escalate" => "escalated",
            "dismiss" => "dismissed",
            _ => "resolved",
        };
        let after_state = serde_json::json!({ "case_status": new_case_status });

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_moderation_decision (
                    id, uuid, case_id, target_type, target_id, decision_action, reason_code, note,
                    decided_by, before_state, after_state, status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8,
                    $9, $10, $11, 'active', 1, NOW(), NOW(), $12, $13, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(command.case_id)
            .bind(&target_type)
            .bind(target_id)
            .bind(&command.decision_action)
            .bind(&command.reason_code)
            .bind(command.note.as_deref())
            .bind(user_id)
            .bind(&before_state)
            .bind(&after_state)
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        run_db!(async {
            sqlx::query(
                "UPDATE forum_moderation_case
                 SET case_status = $1, resolved_at = NOW(), updated_at = NOW(), version = version + 1
                 WHERE id = $2 AND tenant_id = $3"
            )
            .bind(new_case_status)
            .bind(command.case_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;

        run_db!(async {
            sqlx::query(
                "UPDATE forum_moderation_queue_item
                 SET queue_status = 'closed', updated_at = NOW(), version = version + 1
                 WHERE case_id = $1 AND tenant_id = $2 AND queue_status IN ('open', 'assigned', 'in_review')"
            )
            .bind(command.case_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;

        Ok(row_to_moderation_decision(&row))
    }

    fn rebuild_search_projection(
        &self,
        ctx: &ForumRequestContext,
        command: &RebuildSearchProjectionCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let scope = command.scope.as_deref().unwrap_or("all");

        if scope == "all" || scope == "topic" {
            let topics = run_db!(async {
                sqlx::query(
                    "SELECT t.id, t.board_id, t.title, t.body, t.author_user_id, t.visibility, t.version, t.data_scope
                     FROM forum_topic t
                     WHERE t.tenant_id = $1 AND t.deleted_at IS NULL
                       AND ($2::bigint IS NULL OR t.board_id = $2)"
                )
                .bind(tenant_id)
                .bind(command.board_id)
                .fetch_all(&self.pool)
                .await
            }).map_err(|e| ForumServiceError::internal(e.to_string()))?;

            for t in &topics {
                let topic_id: i64 = t.get("id");
                let doc_id = self.next_id()?;
                let doc_uuid = Uuid::new_v4().to_string();
                run_db!(async {
                    sqlx::query(
                        "INSERT INTO forum_search_document (
                            id, uuid, source_type, source_id, board_id, title, body_text,
                            author_user_id, visibility, source_version, index_status, status, version,
                            created_at, updated_at, tenant_id, organization_id, data_scope
                        ) VALUES (
                            $1, $2, 'topic', $3, $4, $5, $6,
                            $7, $8, $9, 'pending', 'active', 1,
                            NOW(), NOW(), $10, $11, $12
                        )
                        ON CONFLICT (tenant_id, source_type, source_id) DO UPDATE SET
                            title = EXCLUDED.title,
                            body_text = EXCLUDED.body_text,
                            source_version = EXCLUDED.source_version,
                            index_status = 'pending',
                            updated_at = NOW(),
                            version = forum_search_document.version + 1"
                    )
                    .bind(doc_id)
                    .bind(&doc_uuid)
                    .bind(topic_id)
                    .bind(t.get::<i64, _>("board_id"))
                    .bind(t.get::<String, _>("title"))
                    .bind(t.get::<String, _>("body"))
                    .bind(t.get::<i64, _>("author_user_id"))
                    .bind(t.get::<String, _>("visibility"))
                    .bind(t.get::<i64, _>("version"))
                    .bind(tenant_id)
                    .bind(org_id)
                    .bind(t.get::<String, _>("data_scope"))
                    .execute(&self.pool)
                    .await
                }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
            }
        }

        if scope == "all" || scope == "reply" {
            let topics = run_db!(async {
                sqlx::query(
                    "SELECT r.id, r.topic_id, r.board_id, r.body, r.author_user_id, r.version,
                            r.tenant_id, r.organization_id, r.data_scope
                     FROM forum_topic_reply r
                     WHERE r.tenant_id = $1 AND r.deleted_at IS NULL
                       AND ($2::bigint IS NULL OR r.board_id = $2)",
                )
                .bind(tenant_id)
                .bind(command.board_id)
                .fetch_all(&self.pool)
                .await
            })
            .map_err(|e| ForumServiceError::internal(e.to_string()))?;

            for r in &topics {
                let reply_id: i64 = r.get("id");
                let doc_id = self.next_id()?;
                let doc_uuid = Uuid::new_v4().to_string();
                run_db!(async {
                    sqlx::query(
                        "INSERT INTO forum_search_document (
                            id, uuid, source_type, source_id, board_id, title, body_text,
                            author_user_id, visibility, source_version, index_status, status, version,
                            created_at, updated_at, tenant_id, organization_id, data_scope
                        ) VALUES (
                            $1, $2, 'reply', $3, $4, NULL, $5,
                            $6, 'public', $7, 'pending', 'active', 1,
                            NOW(), NOW(), $8, $9, $10
                        )
                        ON CONFLICT (tenant_id, source_type, source_id) DO UPDATE SET
                            body_text = EXCLUDED.body_text,
                            source_version = EXCLUDED.source_version,
                            index_status = 'pending',
                            updated_at = NOW(),
                            version = forum_search_document.version + 1"
                    )
                    .bind(doc_id)
                    .bind(&doc_uuid)
                    .bind(reply_id)
                    .bind(r.get::<i64, _>("board_id"))
                    .bind(r.get::<String, _>("body"))
                    .bind(r.get::<i64, _>("author_user_id"))
                    .bind(r.get::<i64, _>("version"))
                    .bind(tenant_id)
                    .bind(org_id)
                    .bind(r.get::<String, _>("data_scope"))
                    .execute(&self.pool)
                    .await
                }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
            }
        }

        Ok(CommandResult::no_id())
    }

    fn rebuild_stats(
        &self,
        ctx: &ForumRequestContext,
        command: &RebuildStatsCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let scope = command.scope.as_deref().unwrap_or("all");

        if scope == "all" || scope == "boards" {
            let board_ids = run_db!(async {
                sqlx::query_scalar::<_, i64>(
                    "SELECT id FROM forum_node
                     WHERE tenant_id = $1 AND node_type = 'board' AND deleted_at IS NULL",
                )
                .bind(tenant_id)
                .fetch_all(&self.pool)
                .await
            })
            .map_err(|e| ForumServiceError::internal(e.to_string()))?;

            for board_id in board_ids {
                self.update_board_stats(ctx, board_id)?;
            }
        }

        if scope == "all" || scope == "topics" {
            let topic_ids = run_db!(async {
                sqlx::query_scalar::<_, i64>(
                    "SELECT id FROM forum_topic
                     WHERE tenant_id = $1 AND deleted_at IS NULL",
                )
                .bind(tenant_id)
                .fetch_all(&self.pool)
                .await
            })
            .map_err(|e| ForumServiceError::internal(e.to_string()))?;

            for topic_id in topic_ids {
                self.update_topic_stats(ctx, topic_id)?;
            }
        }

        Ok(CommandResult::no_id())
    }

    fn list_pending_outbox_events(
        &self,
        ctx: &ForumRequestContext,
        command: &PublishOutboxCommand,
    ) -> Result<Vec<ForumOutboxEvent>, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let limit = command.limit.max(1) as i64;

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_outbox_event
                 WHERE tenant_id = $1
                   AND status = 'pending'
                   AND (next_attempt_at IS NULL OR next_attempt_at <= NOW())
                 ORDER BY id ASC
                 LIMIT $2",
            )
            .bind(tenant_id)
            .bind(limit)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        Ok(rows.iter().map(row_to_outbox_event).collect())
    }

    fn mark_outbox_published(
        &self,
        ctx: &ForumRequestContext,
        event_id: i64,
    ) -> Result<(), ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        run_db!(async {
            sqlx::query(
                "UPDATE forum_outbox_event
                 SET status = 'published',
                     published_at = NOW(),
                     updated_at = NOW(),
                     version = version + 1,
                     publish_attempts = publish_attempts + 1
                 WHERE id = $1 AND tenant_id = $2 AND status = 'pending'",
            )
            .bind(event_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(())
    }

    fn list_topic_revisions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTopicRevisionsCommand,
    ) -> Result<TopicRevisionPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_topic_revision
                 WHERE topic_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
                 ORDER BY revision_no DESC
                 LIMIT $3 OFFSET $4",
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumTopicRevision> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_topic_revision)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn list_reply_revisions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListReplyRevisionsCommand,
    ) -> Result<ReplyRevisionPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_reply_revision
                 WHERE reply_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
                 ORDER BY revision_no DESC
                 LIMIT $3 OFFSET $4",
            )
            .bind(command.reply_id)
            .bind(tenant_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumReplyRevision> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_reply_revision)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_poll_vote(
        &self,
        ctx: &ForumRequestContext,
        command: &CreatePollVoteCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let mut last_id = 0i64;
        let mut last_uuid = String::new();

        for option_id in &command.option_ids {
            let id = self.next_id()?;
            let uuid = Uuid::new_v4().to_string();
            let row = run_db!(async {
                sqlx::query(
                    "INSERT INTO forum_poll_vote (
                        id, uuid, poll_id, option_id, voter_user_id, vote_weight,
                        status, version, created_at, updated_at, tenant_id, organization_id, data_scope
                    ) VALUES (
                        $1, $2, $3, $4, $5, 1,
                        'active', 1, NOW(), NOW(), $6, $7, 'default'
                    ) RETURNING id, uuid"
                )
                .bind(id)
                .bind(&uuid)
                .bind(command.poll_id)
                .bind(option_id)
                .bind(user_id)
                .bind(tenant_id)
                .bind(org_id)
                .fetch_one(&self.pool)
                .await
            }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
            last_id = row.get("id");
            last_uuid = row.get::<String, _>("uuid");

            run_db!(async {
                sqlx::query(
                    "UPDATE forum_poll_option SET vote_count = vote_count + 1, updated_at = NOW()
                     WHERE id = $1 AND poll_id = $2 AND tenant_id = $3",
                )
                .bind(option_id)
                .bind(command.poll_id)
                .bind(tenant_id)
                .execute(&self.pool)
                .await
            })
            .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        }

        run_db!(async {
            sqlx::query(
                "UPDATE forum_poll SET total_vote_count = total_vote_count + $1, updated_at = NOW()
                 WHERE id = $2 AND tenant_id = $3",
            )
            .bind(command.option_ids.len() as i64)
            .bind(command.poll_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        Ok(CommandResult::success(last_id, last_uuid))
    }

    fn create_reaction(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateReactionCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_reaction (
                    id, uuid, target_type, target_id, actor_user_id, reaction_type,
                    status, version, created_at, updated_at, tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    'active', 1, NOW(), NOW(), $7, $8, 'default'
                ) RETURNING id, uuid",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.target_type)
            .bind(command.target_id)
            .bind(user_id)
            .bind(&command.reaction_type)
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn create_vote(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateVoteCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_vote (
                    id, uuid, target_type, target_id, actor_user_id, vote_value, reason_code,
                    status, version, created_at, updated_at, tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    'active', 1, NOW(), NOW(), $8, $9, 'default'
                ) RETURNING id, uuid",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.target_type)
            .bind(command.target_id)
            .bind(user_id)
            .bind(command.vote_value)
            .bind(command.reason_code.as_deref())
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn update_bookmark(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateBookmarkCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_bookmark (
                    id, uuid, target_type, target_id, user_id, note,
                    status, version, created_at, updated_at, tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6,
                    'active', 1, NOW(), NOW(), $7, $8, 'default'
                )
                ON CONFLICT (tenant_id, target_type, target_id, user_id)
                DO UPDATE SET note = EXCLUDED.note, updated_at = NOW()
                RETURNING id, uuid",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.target_type)
            .bind(command.target_id)
            .bind(user_id)
            .bind(command.note.as_deref())
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn update_read_state(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateReadStateCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_read_state (
                    id, uuid, topic_id, user_id, last_read_reply_id, last_read_at, unread_count,
                    status, version, created_at, updated_at, tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, NOW(), 0,
                    'active', 1, NOW(), NOW(), $6, $7, 'default'
                )
                ON CONFLICT (tenant_id, topic_id, user_id)
                DO UPDATE SET last_read_reply_id = EXCLUDED.last_read_reply_id, last_read_at = NOW(), unread_count = 0
                RETURNING id, uuid"
            )
            .bind(id)
            .bind(&uuid)
            .bind(command.topic_id)
            .bind(user_id)
            .bind(command.last_read_reply_id)
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn pin_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &PinTopicCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET pinned_at = CASE WHEN pinned_at IS NULL THEN NOW() ELSE NULL END,
                     updated_at = NOW()
                 WHERE id = $1 AND tenant_id = $2
                 RETURNING id, uuid",
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn unpin_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &PinTopicCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET pinned_at = NULL, updated_at = NOW()
                 WHERE id = $1 AND tenant_id = $2 AND pinned_at IS NOT NULL
                 RETURNING id, uuid",
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn feature_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &FeatureTopicCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET featured_at = CASE WHEN featured_at IS NULL THEN NOW() ELSE NULL END,
                     updated_at = NOW()
                 WHERE id = $1 AND tenant_id = $2
                 RETURNING id, uuid",
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn unfeature_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &FeatureTopicCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET featured_at = NULL, updated_at = NOW()
                 WHERE id = $1 AND tenant_id = $2 AND featured_at IS NOT NULL
                 RETURNING id, uuid",
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn lock_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &LockTopicCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let user_id = ctx.user_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET locked_at = CASE WHEN locked_at IS NULL THEN NOW() ELSE NULL END,
                     locked_by = CASE WHEN locked_at IS NULL THEN $2 ELSE NULL END,
                     updated_at = NOW()
                 WHERE id = $1 AND tenant_id = $3
                 RETURNING id, uuid",
            )
            .bind(command.topic_id)
            .bind(user_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn unlock_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &LockTopicCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET locked_at = NULL, locked_by = NULL, updated_at = NOW()
                 WHERE id = $1 AND tenant_id = $2 AND locked_at IS NOT NULL
                 RETURNING id, uuid",
            )
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn move_topic(
        &self,
        ctx: &ForumRequestContext,
        command: &MoveTopicCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_topic
                 SET board_id = $1, updated_at = NOW()
                 WHERE id = $2 AND tenant_id = $3
                 RETURNING id, uuid",
            )
            .bind(command.target_board_id)
            .bind(command.topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic", command.topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn create_node(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateNodeCommand,
    ) -> Result<ForumNode, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let uuid = Uuid::new_v4().to_string();

        let (parent_path, parent_level) = if let Some(parent_id) = command.parent_id {
            let prow = run_db!(async {
                sqlx::query("SELECT path, level_no FROM forum_node WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL")
                    .bind(parent_id)
                    .bind(tenant_id)
                    .fetch_one(&self.pool)
                    .await
            }).map_err(|e| match e {
                sqlx::Error::RowNotFound => ForumServiceError::not_found("node", parent_id.to_string()),
                e => ForumServiceError::internal(e.to_string()),
            })?;
            (
                prow.get::<String, _>("path"),
                prow.get::<i32, _>("level_no"),
            )
        } else {
            (String::new(), -1i32)
        };

        let id = self.next_id()?;
        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_node (
                    id, uuid, space_id, parent_id, node_type, slug, name, description,
                    path, level_no, sort_order, settings, status, version,
                    created_at, updated_at, tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8,
                    '', $9, $10, $11, 'active', 1,
                    NOW(), NOW(), $12, $13, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(command.space_id)
            .bind(command.parent_id)
            .bind(&command.node_type)
            .bind(&command.slug)
            .bind(&command.name)
            .bind(command.description.as_deref())
            .bind(parent_level + 1)
            .bind(command.sort_order)
            .bind(serde_json::json!({}))
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let node_id: i64 = row.get("id");
        let new_path = if parent_path.is_empty() {
            format!("/{}", node_id)
        } else {
            format!("{}/{}", parent_path, node_id)
        };

        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_node SET path = $1 WHERE id = $2 AND tenant_id = $3 RETURNING *",
            )
            .bind(&new_path)
            .bind(node_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        Ok(row_to_node(&row))
    }

    fn update_node(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateNodeCommand,
    ) -> Result<ForumNode, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();

        if let Some(new_parent_id) = command.parent_id {
            let (parent_path, parent_level) = if new_parent_id == 0 {
                (String::new(), -1i32)
            } else {
                let prow = run_db!(async {
                    sqlx::query("SELECT path, level_no FROM forum_node WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL")
                        .bind(new_parent_id)
                        .bind(tenant_id)
                        .fetch_one(&self.pool)
                        .await
                }).map_err(|e| match e {
                    sqlx::Error::RowNotFound => ForumServiceError::not_found("node", new_parent_id.to_string()),
                    e => ForumServiceError::internal(e.to_string()),
                })?;
                (
                    prow.get::<String, _>("path"),
                    prow.get::<i32, _>("level_no"),
                )
            };

            let new_path = if parent_path.is_empty() {
                format!("/{}", command.node_id)
            } else {
                format!("{}/{}", parent_path, command.node_id)
            };
            let new_level = parent_level + 1;

            let row = run_db!(async {
                sqlx::query(
                    "UPDATE forum_node
                     SET name = COALESCE($1, name),
                         description = COALESCE($2, description),
                         sort_order = COALESCE($3, sort_order),
                         parent_id = CASE WHEN $4 = 0 THEN NULL ELSE $4 END,
                         path = $5,
                         level_no = $6,
                         version = version + 1,
                         updated_at = NOW()
                     WHERE id = $7 AND tenant_id = $8 AND deleted_at IS NULL
                     RETURNING *",
                )
                .bind(command.name.as_deref())
                .bind(command.description.as_deref())
                .bind(command.sort_order)
                .bind(new_parent_id)
                .bind(&new_path)
                .bind(new_level)
                .bind(command.node_id)
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
            })
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => {
                    ForumServiceError::not_found("node", command.node_id.to_string())
                }
                e => ForumServiceError::internal(e.to_string()),
            })?;
            return Ok(row_to_node(&row));
        }

        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_node
                 SET name = COALESCE($1, name),
                     description = COALESCE($2, description),
                     sort_order = COALESCE($3, sort_order),
                     version = version + 1,
                     updated_at = NOW()
                 WHERE id = $4 AND tenant_id = $5 AND deleted_at IS NULL
                 RETURNING *",
            )
            .bind(command.name.as_deref())
            .bind(command.description.as_deref())
            .bind(command.sort_order)
            .bind(command.node_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("node", command.node_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_node(&row))
    }

    fn delete_node(
        &self,
        ctx: &ForumRequestContext,
        command: &DeleteNodeCommand,
    ) -> Result<CommandResult, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let user_id = ctx.user_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_node
                 SET status = 'archived', deleted_at = NOW(), deleted_by = $1, updated_at = NOW()
                 WHERE id = $2 AND tenant_id = $3 AND deleted_at IS NULL
                 RETURNING id, uuid",
            )
            .bind(user_id)
            .bind(command.node_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("node", command.node_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(CommandResult::success(
            row.get("id"),
            row.get::<String, _>("uuid"),
        ))
    }

    fn list_moderation_cases(
        &self,
        ctx: &ForumRequestContext,
        command: &ListModerationCasesCommand,
    ) -> Result<ModerationCasePageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_moderation_case
                 WHERE tenant_id = $1 AND deleted_at IS NULL
                   AND ($2::text IS NULL OR case_status = $2)
                 ORDER BY updated_at DESC, id DESC
                 LIMIT $3 OFFSET $4",
            )
            .bind(tenant_id)
            .bind(command.status_filter.as_deref())
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumModerationCase> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_moderation_case)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_moderation_case(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateModerationCaseCommand,
    ) -> Result<ForumModerationCase, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;
        let case_no = self.get_next_case_no(ctx, tenant_id)?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_moderation_case (
                    id, uuid, case_no, target_type, target_id, case_status, severity,
                    opened_by, summary, status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, 'open', $6,
                    $7, $8, 'active', 1, NOW(), NOW(), $9, $10, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&case_no)
            .bind(&command.target_type)
            .bind(command.target_id)
            .bind(&command.severity)
            .bind(user_id)
            .bind(command.summary.as_deref())
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row_to_moderation_case(&row))
    }

    fn retrieve_moderation_case(
        &self,
        ctx: &ForumRequestContext,
        command: &RetrieveModerationCaseCommand,
    ) -> Result<ForumModerationCase, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_moderation_case WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"
            )
            .bind(command.case_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| match e {
            sqlx::Error::RowNotFound => ForumServiceError::not_found("moderation_case", command.case_id.to_string()),
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_moderation_case(&row))
    }

    fn list_sanctions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListSanctionsCommand,
    ) -> Result<SanctionPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_sanction
                 WHERE tenant_id = $1 AND deleted_at IS NULL
                   AND ($2::bigint IS NULL OR user_id = $2)
                 ORDER BY created_at DESC, id DESC
                 LIMIT $3 OFFSET $4",
            )
            .bind(tenant_id)
            .bind(command.user_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumSanction> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_sanction)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_sanction(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateSanctionCommand,
    ) -> Result<ForumSanction, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_sanction (
                    id, uuid, user_id, case_id, decision_id, sanction_type, reason_code,
                    starts_at, expires_at, status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8::timestamptz, $9::timestamptz, 'active', 1, NOW(), NOW(), $10, $11, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(command.user_id)
            .bind(command.case_id)
            .bind(command.decision_id)
            .bind(&command.sanction_type)
            .bind(&command.reason_code)
            .bind(&command.starts_at)
            .bind(command.expires_at.as_deref())
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row_to_sanction(&row))
    }

    fn update_sanction(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateSanctionCommand,
    ) -> Result<ForumSanction, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_sanction
                 SET expires_at = COALESCE($1::timestamptz, expires_at),
                     version = version + 1,
                     updated_at = NOW()
                 WHERE id = $2 AND tenant_id = $3 AND deleted_at IS NULL
                 RETURNING *",
            )
            .bind(command.expires_at.as_deref())
            .bind(command.sanction_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("sanction", command.sanction_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_sanction(&row))
    }

    fn list_reputation_rules(
        &self,
        ctx: &ForumRequestContext,
        command: &ListReputationRulesCommand,
    ) -> Result<ReputationRulePageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_reputation_rule
                 WHERE tenant_id = $1 AND deleted_at IS NULL AND status = 'active'
                 ORDER BY created_at DESC, id DESC
                 LIMIT $2 OFFSET $3",
            )
            .bind(tenant_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumReputationRule> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_reputation_rule)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_reputation_rule(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateReputationRuleCommand,
    ) -> Result<ForumReputationRule, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_reputation_rule (
                    id, uuid, code, event_type, points, daily_limit, rule_json,
                    status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    'active', 1, NOW(), NOW(), $8, $9, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.code)
            .bind(&command.event_type)
            .bind(command.points)
            .bind(command.daily_limit)
            .bind(&command.rule_json)
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row_to_reputation_rule(&row))
    }

    fn list_reputation_ledger(
        &self,
        ctx: &ForumRequestContext,
        command: &ListReputationLedgerCommand,
    ) -> Result<ReputationLedgerPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_reputation_ledger
                 WHERE tenant_id = $1 AND deleted_at IS NULL
                   AND ($2::bigint IS NULL OR user_id = $2)
                 ORDER BY created_at DESC, id DESC
                 LIMIT $3 OFFSET $4",
            )
            .bind(tenant_id)
            .bind(command.user_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumReputationLedger> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_reputation_ledger)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn list_trust_levels(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTrustLevelsCommand,
    ) -> Result<TrustLevelPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_trust_level
                 WHERE tenant_id = $1 AND deleted_at IS NULL AND status = 'active'
                 ORDER BY level_no ASC
                 LIMIT $2 OFFSET $3",
            )
            .bind(tenant_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumTrustLevel> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_trust_level)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_trust_level(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateTrustLevelCommand,
    ) -> Result<ForumTrustLevel, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_trust_level (
                    id, uuid, level_no, code, name, threshold_rules, privileges,
                    status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    'active', 1, NOW(), NOW(), $8, $9, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(command.level_no)
            .bind(&command.code)
            .bind(&command.name)
            .bind(&command.threshold_rules)
            .bind(&command.privileges)
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row_to_trust_level(&row))
    }

    fn list_badges(
        &self,
        ctx: &ForumRequestContext,
        command: &ListBadgesCommand,
    ) -> Result<BadgePageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_badge
                 WHERE tenant_id = $1 AND deleted_at IS NULL AND status = 'active'
                 ORDER BY created_at DESC, id DESC
                 LIMIT $2 OFFSET $3",
            )
            .bind(tenant_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumBadge> = rows.iter().take(limit as usize).map(row_to_badge).collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_badge(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateBadgeCommand,
    ) -> Result<ForumBadge, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_badge (
                    id, uuid, code, name, description, grant_mode, icon_media_id, rule_json,
                    status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8,
                    'active', 1, NOW(), NOW(), $9, $10, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.code)
            .bind(&command.name)
            .bind(command.description.as_deref())
            .bind(&command.grant_mode)
            .bind(command.icon_media_id.as_deref())
            .bind(command.rule_json.as_ref())
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row_to_badge(&row))
    }

    fn list_board_stats(
        &self,
        ctx: &ForumRequestContext,
        command: &ListBoardStatsCommand,
    ) -> Result<BoardStatsPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_board_stats
                 WHERE tenant_id = $1 AND deleted_at IS NULL
                 ORDER BY last_activity_at DESC NULLS LAST, id DESC
                 LIMIT $2 OFFSET $3",
            )
            .bind(tenant_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumBoardStats> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_board_stats)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn list_topic_stats(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTopicStatsCommand,
    ) -> Result<TopicStatsPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_topic_stats
                 WHERE tenant_id = $1 AND deleted_at IS NULL
                 ORDER BY vote_score DESC, topic_id DESC
                 LIMIT $2 OFFSET $3",
            )
            .bind(tenant_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumTopicStats> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_topic_stats)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_audit_action(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateAuditActionCommand,
    ) -> Result<ForumAuditAction, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_audit_action (
                    id, uuid, action, target_type, target_id, operator_id, detail,
                    status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    'active', 1, NOW(), NOW(), $8, $9, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.action)
            .bind(&command.target_type)
            .bind(command.target_id)
            .bind(user_id)
            .bind(command.detail.as_deref())
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row_to_audit_action(&row))
    }

    fn list_audit_actions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListAuditActionsCommand,
    ) -> Result<AuditActionPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_audit_action
                 WHERE tenant_id = $1 AND status = 'active'
                 ORDER BY created_at DESC, id DESC
                 LIMIT $2 OFFSET $3",
            )
            .bind(tenant_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumAuditAction> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_audit_action)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn list_tags(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTagsCommand,
    ) -> Result<TagPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_tag
                 WHERE tenant_id = $1
                   AND deleted_at IS NULL
                   AND status = 'active'
                   AND ($2::bigint IS NULL OR space_id = $2)
                 ORDER BY usage_count DESC, id ASC
                 LIMIT $3 OFFSET $4",
            )
            .bind(tenant_id)
            .bind(command.space_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumTag> = rows.iter().take(limit as usize).map(row_to_tag).collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn list_topic_prefixes(
        &self,
        ctx: &ForumRequestContext,
        command: &ListTopicPrefixesCommand,
    ) -> Result<TopicPrefixPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_topic_prefix
                 WHERE tenant_id = $1 AND deleted_at IS NULL AND status = 'active'
                   AND ($2::bigint IS NULL OR board_id = $2)
                 ORDER BY sort_order ASC, id ASC
                 LIMIT $3 OFFSET $4",
            )
            .bind(tenant_id)
            .bind(command.board_id)
            .bind(limit + 1)
            .bind(offset)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumTopicPrefix> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_topic_prefix)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn create_topic_prefix(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateTopicPrefixCommand,
    ) -> Result<ForumTopicPrefix, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_topic_prefix (
                    id, uuid, board_id, code, label, color, sort_order, required_trust_level,
                    status, version, created_at, updated_at,
                    tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8,
                    'active', 1, NOW(), NOW(), $9, $10, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(command.board_id)
            .bind(&command.code)
            .bind(&command.label)
            .bind(command.color.as_deref())
            .bind(command.sort_order)
            .bind(command.required_trust_level)
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row_to_topic_prefix(&row))
    }

    fn create_space(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateSpaceCommand,
    ) -> Result<ForumSpace, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_space (
                    id, uuid, code, slug, name, description, visibility, default_locale, settings,
                    status, version, created_at, updated_at, tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7, $8, $9,
                    'active', 1, NOW(), NOW(), $10, $11, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.code)
            .bind(&command.slug)
            .bind(&command.name)
            .bind(command.description.as_deref())
            .bind(&command.visibility)
            .bind(command.default_locale.as_deref())
            .bind(&command.settings)
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        Ok(row_to_space(&row))
    }

    fn update_space(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateSpaceCommand,
    ) -> Result<ForumSpace, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_space
                 SET name = COALESCE($1, name),
                     description = COALESCE($2, description),
                     visibility = COALESCE($3, visibility),
                     version = version + 1,
                     updated_at = NOW()
                 WHERE id = $4 AND tenant_id = $5
                 RETURNING *",
            )
            .bind(command.name.as_deref())
            .bind(command.description.as_deref())
            .bind(command.visibility.as_deref())
            .bind(command.space_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("space", command.space_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_space(&row))
    }

    fn create_attachment(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateAttachmentCommand,
    ) -> Result<ForumAttachment, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_attachment (
                    id, uuid, owner_type, owner_id, drive_space_id, drive_node_id, media_resource_id,
                    file_name, mime_type, byte_size, sort_order, scan_status,
                    status, version, created_at, updated_at, tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    $8, $9, $10, $11, 'pending',
                    'active', 1, NOW(), NOW(), $12, $13, 'default'
                ) RETURNING *"
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.owner_type)
            .bind(command.owner_id)
            .bind(&command.drive_space_id)
            .bind(&command.drive_node_id)
            .bind(command.media_resource_id.as_deref())
            .bind(&command.file_name)
            .bind(&command.mime_type)
            .bind(command.byte_size)
            .bind(command.sort_order)
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row_to_attachment(&row))
    }

    fn create_subscription(
        &self,
        ctx: &ForumRequestContext,
        command: &CreateSubscriptionCommand,
    ) -> Result<ForumSubscription, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let user_id = ctx.user_id_value();
        let uuid = Uuid::new_v4().to_string();
        let id = self.next_id()?;

        let row = run_db!(async {
            sqlx::query(
                "INSERT INTO forum_subscription (
                    id, uuid, target_type, target_id, user_id, notify_level, delivery_channels,
                    status, version, created_at, updated_at, tenant_id, organization_id, data_scope
                ) VALUES (
                    $1, $2, $3, $4, $5, $6, $7,
                    'active', 1, NOW(), NOW(), $8, $9, 'default'
                ) RETURNING *",
            )
            .bind(id)
            .bind(&uuid)
            .bind(&command.target_type)
            .bind(command.target_id)
            .bind(user_id)
            .bind(&command.notify_level)
            .bind(command.delivery_channels.as_deref())
            .bind(tenant_id)
            .bind(org_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row_to_subscription(&row))
    }

    fn update_subscription(
        &self,
        ctx: &ForumRequestContext,
        command: &UpdateSubscriptionCommand,
    ) -> Result<ForumSubscription, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "UPDATE forum_subscription
                 SET notify_level = COALESCE($1, notify_level),
                     delivery_channels = COALESCE($2, delivery_channels),
                     version = version + 1,
                     updated_at = NOW()
                 WHERE id = $3 AND tenant_id = $4 AND status = 'active'
                 RETURNING *",
            )
            .bind(command.notify_level.as_deref())
            .bind(command.delivery_channels.as_deref())
            .bind(command.subscription_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("subscription", command.subscription_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_subscription(&row))
    }

    fn list_subscriptions(
        &self,
        ctx: &ForumRequestContext,
        command: &ListSubscriptionsCommand,
    ) -> Result<SubscriptionPageResult, ForumServiceError> {
        let offset = parse_cursor(&command.cursor);
        let limit = command.limit.max(1) as i64;
        let tenant_id = ctx.tenant_id_value();

        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_subscription
                 WHERE tenant_id = $1
                   AND ($2::text IS NULL OR target_type = $2)
                   AND ($5::bigint IS NULL OR target_id = $5)
                   AND status = 'active'
                 ORDER BY created_at DESC
                 LIMIT $3 OFFSET $4",
            )
            .bind(tenant_id)
            .bind(command.target_type.as_deref())
            .bind(limit + 1)
            .bind(offset)
            .bind(command.target_id)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;

        let has_more = rows.len() as i64 > limit;
        let items: Vec<ForumSubscription> = rows
            .iter()
            .take(limit as usize)
            .map(row_to_subscription)
            .collect();
        let next_cursor = if has_more {
            Some((offset + limit).to_string())
        } else {
            None
        };
        Ok(CursorPage::new(items, next_cursor, has_more))
    }

    fn check_space_has_topics(
        &self,
        ctx: &ForumRequestContext,
        space_id: i64,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_topic WHERE space_id = $1 AND tenant_id = $2 AND deleted_at IS NULL"
            )
            .bind(space_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn check_node_cycle(
        &self,
        ctx: &ForumRequestContext,
        node_id: i64,
        new_parent_id: i64,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "WITH RECURSIVE ancestors AS (
                    SELECT id, parent_id FROM forum_node WHERE id = $1 AND tenant_id = $2
                    UNION ALL
                    SELECT n.id, n.parent_id FROM forum_node n JOIN ancestors a ON n.id = a.parent_id WHERE n.tenant_id = $2
                ) SELECT COUNT(*) FROM ancestors WHERE id = $3"
            )
            .bind(node_id)
            .bind(tenant_id)
            .bind(new_parent_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn check_node_is_board(
        &self,
        ctx: &ForumRequestContext,
        node_id: i64,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let node_type: Option<String> = run_db!(async {
            sqlx::query_scalar(
                "SELECT node_type FROM forum_node WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"
            )
            .bind(node_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| match e {
            sqlx::Error::RowNotFound => ForumServiceError::not_found("node", node_id.to_string()),
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(node_type.as_deref() == Some("board"))
    }

    fn check_board_exists(
        &self,
        ctx: &ForumRequestContext,
        board_id: i64,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_node WHERE id = $1 AND tenant_id = $2 AND node_type = 'board' AND deleted_at IS NULL"
            )
            .bind(board_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn check_owner_exists(
        &self,
        ctx: &ForumRequestContext,
        owner_type: &str,
        owner_id: i64,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let sql = match owner_type {
            "topic" => "SELECT COUNT(*) FROM forum_topic WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL",
            "reply" => "SELECT COUNT(*) FROM forum_topic_reply WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL",
            _ => return Err(ForumServiceError::validation(format!("unsupported owner_type: {}", owner_type))),
        };
        let count: i64 = run_db!(async {
            sqlx::query_scalar(sql)
                .bind(owner_id)
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn check_poll_exists(
        &self,
        ctx: &ForumRequestContext,
        poll_id: i64,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_poll WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"
            )
            .bind(poll_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn count_poll_votes(
        &self,
        ctx: &ForumRequestContext,
        poll_id: i64,
    ) -> Result<i64, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_poll_vote WHERE poll_id = $1 AND tenant_id = $2 AND deleted_at IS NULL"
            )
            .bind(poll_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count)
    }

    fn check_poll_selection_mode(
        &self,
        ctx: &ForumRequestContext,
        poll_id: i64,
    ) -> Result<String, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let mode: Option<String> = run_db!(async {
            sqlx::query_scalar(
                "SELECT selection_mode FROM forum_poll WHERE id = $1 AND tenant_id = $2 AND deleted_at IS NULL"
            )
            .bind(poll_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| match e {
            sqlx::Error::RowNotFound => ForumServiceError::not_found("poll", poll_id.to_string()),
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(mode.unwrap_or_else(|| "single".to_string()))
    }

    fn check_active_vote(
        &self,
        ctx: &ForumRequestContext,
        target_type: &str,
        target_id: i64,
        actor_user_id: i64,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_vote WHERE target_type = $1 AND target_id = $2 AND actor_user_id = $3 AND tenant_id = $4 AND status = 'active'"
            )
            .bind(target_type)
            .bind(target_id)
            .bind(actor_user_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn check_active_sanctions(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
    ) -> Result<Vec<ForumSanction>, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_sanction
                 WHERE user_id = $1 AND tenant_id = $2
                   AND status = 'active'
                   AND (expires_at IS NULL OR expires_at > NOW())
                   AND lifted_at IS NULL",
            )
            .bind(user_id)
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(rows.iter().map(row_to_sanction).collect())
    }

    fn check_active_appeal(
        &self,
        ctx: &ForumRequestContext,
        sanction_id: Option<i64>,
        case_id: Option<i64>,
        appellant_user_id: i64,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_appeal
                 WHERE appellant_user_id = $1 AND tenant_id = $2
                   AND appeal_status IN ('open', 'reviewing')
                   AND ((sanction_id = $3 AND $3 IS NOT NULL) OR (case_id = $4 AND $4 IS NOT NULL))"
            )
            .bind(appellant_user_id)
            .bind(tenant_id)
            .bind(sanction_id)
            .bind(case_id)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn count_topics_in_space(
        &self,
        ctx: &ForumRequestContext,
        space_id: i64,
    ) -> Result<i64, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_topic WHERE space_id = $1 AND tenant_id = $2 AND deleted_at IS NULL"
            )
            .bind(space_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count)
    }

    fn get_next_revision_no(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<i32, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let next_no: i32 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COALESCE(MAX(revision_no), 0) + 1 FROM forum_topic_revision WHERE topic_id = $1 AND tenant_id = $2"
            )
            .bind(topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(next_no)
    }

    fn get_next_reply_no(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<i32, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let next_no: i32 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COALESCE(MAX(reply_no), 0) + 1 FROM forum_topic_reply WHERE topic_id = $1 AND tenant_id = $2 AND deleted_at IS NULL"
            )
            .bind(topic_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(next_no)
    }

    fn get_next_case_no(
        &self,
        _ctx: &ForumRequestContext,
        tenant_id: i64,
    ) -> Result<String, ForumServiceError> {
        let case_no: String = run_db!(async {
            sqlx::query_scalar(
                "SELECT 'MOD-' || TO_CHAR(NOW(), 'YYYY') || '-' || LPAD(COALESCE(MAX(CAST(SUBSTRING(case_no FROM 10) AS INTEGER)), 0) + 1, 4, '0')
                 FROM forum_moderation_case
                 WHERE tenant_id = $1 AND case_no LIKE 'MOD-' || TO_CHAR(NOW(), 'YYYY') || '-%'"
            )
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(case_no)
    }

    fn check_duplicate_queue_item(
        &self,
        ctx: &ForumRequestContext,
        target_type: &str,
        target_id: i64,
        source_type: &str,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_moderation_queue_item
                 WHERE tenant_id = $1 AND target_type = $2 AND target_id = $3 AND source_type = $4
                   AND queue_status IN ('open', 'assigned', 'in_review')",
            )
            .bind(tenant_id)
            .bind(target_type)
            .bind(target_id)
            .bind(source_type)
            .fetch_one(&self.pool)
            .await
        })
        .map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn check_idempotency_key(
        &self,
        ctx: &ForumRequestContext,
        key: &str,
        operation_id: &str,
    ) -> Result<Option<ForumIdempotencyRecord>, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_idempotency_record
                 WHERE tenant_id = $1 AND idempotency_key = $2 AND operation_id = $3 AND expires_at > NOW()"
            )
            .bind(tenant_id)
            .bind(key)
            .bind(operation_id)
            .fetch_optional(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(row.map(|r| row_to_idempotency_record(&r)))
    }

    fn check_message_id_exists(
        &self,
        ctx: &ForumRequestContext,
        source_system: &str,
        message_id: &str,
        consumer_name: &str,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_inbox_event
                 WHERE tenant_id = $1 AND source_system = $2 AND message_id = $3 AND consumer_name = $4"
            )
            .bind(tenant_id)
            .bind(source_system)
            .bind(message_id)
            .bind(consumer_name)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn check_message_payload_hash(
        &self,
        ctx: &ForumRequestContext,
        source_system: &str,
        message_id: &str,
        consumer_name: &str,
        payload_hash: &str,
    ) -> Result<bool, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let count: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COUNT(*) FROM forum_inbox_event
                 WHERE tenant_id = $1 AND source_system = $2 AND message_id = $3 AND consumer_name = $4 AND payload_hash = $5"
            )
            .bind(tenant_id)
            .bind(source_system)
            .bind(message_id)
            .bind(consumer_name)
            .bind(payload_hash)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(count > 0)
    }

    fn get_reputation_balance(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
    ) -> Result<i64, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let balance: i64 = run_db!(async {
            sqlx::query_scalar(
                "SELECT COALESCE(SUM(CASE WHEN direction = 'credit' THEN points ELSE -points END), 0)
                 FROM forum_reputation_ledger WHERE user_id = $1 AND tenant_id = $2"
            )
            .bind(user_id)
            .bind(tenant_id)
            .fetch_one(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(balance)
    }

    fn get_topic_stats(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<ForumTopicStats, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query("SELECT * FROM forum_topic_stats WHERE topic_id = $1 AND tenant_id = $2")
                .bind(topic_id)
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("topic_stats", topic_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_topic_stats(&row))
    }

    fn get_board_stats(
        &self,
        ctx: &ForumRequestContext,
        board_id: i64,
    ) -> Result<ForumBoardStats, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query("SELECT * FROM forum_board_stats WHERE board_id = $1 AND tenant_id = $2")
                .bind(board_id)
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("board_stats", board_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_board_stats(&row))
    }

    fn get_member_stats(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
    ) -> Result<ForumMemberStats, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let row = run_db!(async {
            sqlx::query("SELECT * FROM forum_member_stats WHERE user_id = $1 AND tenant_id = $2")
                .bind(user_id)
                .bind(tenant_id)
                .fetch_one(&self.pool)
                .await
        })
        .map_err(|e| match e {
            sqlx::Error::RowNotFound => {
                ForumServiceError::not_found("member_stats", user_id.to_string())
            }
            e => ForumServiceError::internal(e.to_string()),
        })?;
        Ok(row_to_member_stats(&row))
    }

    fn update_tag_usage_count(
        &self,
        ctx: &ForumRequestContext,
        tag_id: i64,
    ) -> Result<(), ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        run_db!(async {
            sqlx::query(
                "UPDATE forum_tag SET usage_count = (
                    SELECT COUNT(*) FROM forum_topic_tag WHERE tag_id = $1 AND tenant_id = $2 AND deleted_at IS NULL
                ) WHERE id = $1 AND tenant_id = $2"
            )
            .bind(tag_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(())
    }

    fn update_unread_count(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
        user_id: i64,
    ) -> Result<(), ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        run_db!(async {
            sqlx::query(
                "UPDATE forum_read_state SET unread_count = (
                    SELECT COUNT(*) FROM forum_topic_reply
                    WHERE topic_id = $1
                      AND id > COALESCE((SELECT last_read_reply_id FROM forum_read_state WHERE topic_id = $1 AND user_id = $2 AND tenant_id = $3), 0)
                      AND tenant_id = $3 AND deleted_at IS NULL
                ) WHERE topic_id = $1 AND user_id = $2 AND tenant_id = $3"
            )
            .bind(topic_id)
            .bind(user_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(())
    }

    fn get_notification_preferences(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
        event_type: &str,
    ) -> Result<Vec<ForumNotificationPreference>, ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let rows = run_db!(async {
            sqlx::query(
                "SELECT * FROM forum_notification_preference WHERE user_id = $1 AND event_type = $2 AND tenant_id = $3 AND status = 'active'"
            )
            .bind(user_id)
            .bind(event_type)
            .bind(tenant_id)
            .fetch_all(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(rows.iter().map(row_to_notification_preference).collect())
    }

    fn insert_outbox_event(
        &self,
        ctx: &ForumRequestContext,
        event: &ForumOutboxEvent,
    ) -> Result<(), ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        let org_id = ctx.organization_id_value();
        let id = self.next_id()?;
        run_db!(async {
            sqlx::query(
                "INSERT INTO forum_outbox_event (
                    id, uuid, event_key, aggregate_type, aggregate_id, event_type, event_version,
                    payload_json, headers_json, status, publish_attempts, version, created_at, updated_at,
                    tenant_id, organization_id
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, 1, NOW(), NOW(), $12, $13)"
            )
            .bind(id)
            .bind(&event.uuid)
            .bind(&event.event_key)
            .bind(&event.aggregate_type)
            .bind(&event.aggregate_id)
            .bind(&event.event_type)
            .bind(event.event_version)
            .bind(&event.payload_json)
            .bind(&event.headers_json)
            .bind(&event.status)
            .bind(event.publish_attempts)
            .bind(tenant_id)
            .bind(org_id)
            .execute(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(())
    }

    fn update_topic_stats(
        &self,
        ctx: &ForumRequestContext,
        topic_id: i64,
    ) -> Result<(), ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        run_db!(async {
            sqlx::query(
                "UPDATE forum_topic_stats SET
                    reply_count = (SELECT COUNT(*) FROM forum_topic_reply WHERE topic_id = $1 AND tenant_id = $2 AND deleted_at IS NULL),
                    last_calculated_at = NOW()
                 WHERE topic_id = $1 AND tenant_id = $2"
            )
            .bind(topic_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(())
    }

    fn update_board_stats(
        &self,
        ctx: &ForumRequestContext,
        board_id: i64,
    ) -> Result<(), ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        run_db!(async {
            sqlx::query(
                "UPDATE forum_board_stats SET
                    topic_count = (SELECT COUNT(*) FROM forum_topic WHERE board_id = $1 AND tenant_id = $2 AND deleted_at IS NULL),
                    reply_count = (SELECT COUNT(*) FROM forum_topic_reply WHERE board_id = $1 AND tenant_id = $2 AND deleted_at IS NULL),
                    last_calculated_at = NOW()
                 WHERE board_id = $1 AND tenant_id = $2"
            )
            .bind(board_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(())
    }

    fn update_member_stats(
        &self,
        ctx: &ForumRequestContext,
        user_id: i64,
    ) -> Result<(), ForumServiceError> {
        let tenant_id = ctx.tenant_id_value();
        run_db!(async {
            sqlx::query(
                "UPDATE forum_member_stats SET
                    topic_count = (SELECT COUNT(*) FROM forum_topic WHERE author_user_id = $1 AND tenant_id = $2 AND deleted_at IS NULL),
                    reply_count = (SELECT COUNT(*) FROM forum_topic_reply WHERE author_user_id = $1 AND tenant_id = $2 AND deleted_at IS NULL),
                    last_calculated_at = NOW()
                 WHERE user_id = $1 AND tenant_id = $2"
            )
            .bind(user_id)
            .bind(tenant_id)
            .execute(&self.pool)
            .await
        }).map_err(|e| ForumServiceError::internal(e.to_string()))?;
        Ok(())
    }
}
