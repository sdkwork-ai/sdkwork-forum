pub const SCHEMA_REGISTRY_PATH: &str = "specs/forum-database.schema.yaml";

pub const TABLES: &[&str] = &[
    "forum_space",
    "forum_node",
    "forum_board_profile",
    "forum_tag",
    "forum_topic_tag",
    "forum_topic_prefix",
    "forum_node_acl",
    "forum_topic",
    "forum_topic_revision",
    "forum_topic_reply",
    "forum_reply_revision",
    "forum_attachment",
    "forum_question_profile",
    "forum_poll",
    "forum_poll_option",
    "forum_poll_vote",
    "forum_reaction",
    "forum_vote",
    "forum_bookmark",
    "forum_subscription",
    "forum_read_state",
    "forum_notification_preference",
    "forum_member_profile",
    "forum_trust_level",
    "forum_privilege_grant",
    "forum_badge",
    "forum_user_badge",
    "forum_reputation_ledger",
    "forum_reputation_rule",
    "forum_report",
    "forum_moderation_queue_item",
    "forum_moderation_case",
    "forum_moderation_decision",
    "forum_moderation_policy",
    "forum_sanction",
    "forum_appeal",
    "forum_topic_stats",
    "forum_board_stats",
    "forum_member_stats",
    "forum_search_document",
    "forum_outbox_event",
    "forum_inbox_event",
    "forum_idempotency_record",
];

pub const TABLE_GROUPS: &[(&str, &str)] = &[
    ("forum_space", "taxonomy"),
    ("forum_node", "taxonomy"),
    ("forum_board_profile", "taxonomy"),
    ("forum_tag", "taxonomy"),
    ("forum_topic_tag", "taxonomy"),
    ("forum_topic_prefix", "taxonomy"),
    ("forum_node_acl", "taxonomy"),
    ("forum_topic", "discussion"),
    ("forum_topic_revision", "discussion"),
    ("forum_topic_reply", "discussion"),
    ("forum_reply_revision", "discussion"),
    ("forum_attachment", "discussion"),
    ("forum_question_profile", "qa_poll"),
    ("forum_poll", "qa_poll"),
    ("forum_poll_option", "qa_poll"),
    ("forum_poll_vote", "qa_poll"),
    ("forum_reaction", "engagement"),
    ("forum_vote", "engagement"),
    ("forum_bookmark", "engagement"),
    ("forum_subscription", "engagement"),
    ("forum_read_state", "engagement"),
    ("forum_notification_preference", "engagement"),
    ("forum_member_profile", "member"),
    ("forum_trust_level", "member"),
    ("forum_privilege_grant", "member"),
    ("forum_badge", "member"),
    ("forum_user_badge", "member"),
    ("forum_reputation_ledger", "member"),
    ("forum_reputation_rule", "member"),
    ("forum_report", "moderation"),
    ("forum_moderation_queue_item", "moderation"),
    ("forum_moderation_case", "moderation"),
    ("forum_moderation_decision", "moderation"),
    ("forum_moderation_policy", "moderation"),
    ("forum_sanction", "moderation"),
    ("forum_appeal", "moderation"),
    ("forum_topic_stats", "projection"),
    ("forum_board_stats", "projection"),
    ("forum_member_stats", "projection"),
    ("forum_search_document", "projection"),
    ("forum_outbox_event", "integration"),
    ("forum_inbox_event", "integration"),
    ("forum_idempotency_record", "integration"),
];

pub const TENANT_ENTITY_FIELD_SET: &[&str] = &[
    "id",
    "uuid",
    "tenant_id",
    "organization_id",
    "data_scope",
    "status",
    "version",
    "created_at",
    "updated_at",
    "deleted_at",
    "deleted_by",
];

pub const INTEGRATION_LOG_FIELD_SET: &[&str] = &[
    "id",
    "uuid",
    "tenant_id",
    "organization_id",
    "status",
    "version",
    "created_at",
    "updated_at",
];

pub fn ensure_known_table(table: &str) -> bool {
    TABLES.contains(&table)
}

pub fn table_group(table: &str) -> Option<&'static str> {
    TABLE_GROUPS
        .iter()
        .find(|(t, _)| *t == table)
        .map(|(_, g)| *g)
}

pub fn tables_in_group(group: &str) -> Vec<&'static str> {
    TABLE_GROUPS
        .iter()
        .filter(|(_, g)| *g == group)
        .map(|(t, _)| *t)
        .collect()
}

pub fn is_tenant_scoped(table: &str) -> bool {
    ensure_known_table(table)
        && !matches!(
            table,
            "forum_outbox_event" | "forum_inbox_event" | "forum_idempotency_record"
        )
}

pub fn requires_idempotency(table: &str) -> bool {
    matches!(
        table,
        "forum_outbox_event"
            | "forum_inbox_event"
            | "forum_idempotency_record"
            | "forum_reputation_ledger"
    )
}
